use libkernel::{datastructures::SpscQueue, sync::SpinLock};
use log::warn;

use crate::io::IoPort;

pub struct Keyboard {
    port: IoPort,
    scancode_buffer: SpscQueue<u8, 17>,
    state: SpinLock<KeyboardState>,
}

pub struct KeyboardState {
    scancode_decoder: ScanCodeDecoder,
}

pub static KEYBOARD: Keyboard = Keyboard {
    port: IoPort::new(0x60),
    scancode_buffer: SpscQueue::new(),
    state: SpinLock::new(KeyboardState {
        scancode_decoder: ScanCodeDecoder::new(),
    }),
};

impl Keyboard {
    pub fn on_data(&self) {
        let byte = unsafe { self.port.in_byte() };

        // Drop the scancode if the buffer is full
        // This panic is avoidable
        let _ = self.scancode_buffer.push_back(byte);
    }

    pub fn poll_event(&self) -> Option<KeyEvent> {
        if self.scancode_buffer.is_full() {
            warn!("Scancode buffer was full, we may have dropped some");
        }

        let scancode = self.scancode_buffer.pop_front()?;

        let mut lock = self.state.lock();
        let decoder = &mut lock.scancode_decoder;

        decoder.push_scancode(scancode)
    }
}

#[derive(Clone, Copy)]
pub struct KeyEvent {
    pub key_code: KeyCode,
    pub action: KeyAction,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Press,
    Release,
}

struct ScanCodeDecoder {
    state: DecodeState,
}

#[derive(Clone, Copy)]
enum DecodeState {
    Start,
    TwoBytes,
    PrintScreen(KeyAction, u8),
    Pause(u8),
}

#[derive(Clone, Copy)]
enum DecodeResult {
    Ok(KeyCode, KeyAction),
    Incomplete(DecodeState),
    Unrecognized,
}

impl ScanCodeDecoder {
    const fn new() -> Self {
        Self {
            state: DecodeState::Start,
        }
    }

    fn push_scancode(&mut self, scancode: u8) -> Option<KeyEvent> {
        use DecodeResult as Result;
        use DecodeState as State;

        let result = match (self.state, scancode) {
            (State::Start, 0xE0) => Result::Incomplete(State::TwoBytes),
            (State::Start, 0xE1) => Result::Incomplete(State::Pause(1)),
            (State::Start, scancode) => Self::decode_single_byte(scancode),
            (State::TwoBytes, 0x2A) => Result::Incomplete(State::PrintScreen(KeyAction::Press, 2)),
            (State::TwoBytes, 0xB7) => {
                Result::Incomplete(State::PrintScreen(KeyAction::Release, 2))
            }
            (State::TwoBytes, scancode) => Self::decode_double_byte(scancode),
            (State::Pause(1), 0x1D) => Result::Incomplete(State::Pause(2)),
            (State::Pause(2), 0x45) => Result::Incomplete(State::Pause(3)),
            (State::Pause(3), 0xE1) => Result::Incomplete(State::Pause(4)),
            (State::Pause(4), 0x9D) => Result::Incomplete(State::Pause(5)),
            (State::Pause(5), 0xC5) => Result::Ok(KeyCode::KeyPause, KeyAction::Press),
            (State::PrintScreen(action, 2), 0xE0) => {
                Result::Incomplete(State::PrintScreen(action, 3))
            }
            (State::PrintScreen(KeyAction::Press, 3), 0x37) => {
                Result::Ok(KeyCode::KeyPrintScreen, KeyAction::Press)
            }
            (State::PrintScreen(KeyAction::Release, 3), 0xAA) => {
                Result::Ok(KeyCode::KeyPrintScreen, KeyAction::Release)
            }
            _ => Result::Unrecognized,
        };

        match result {
            DecodeResult::Ok(key_code, action) => {
                self.state = DecodeState::Start;
                Some(KeyEvent { key_code, action })
            }
            DecodeResult::Incomplete(state) => {
                self.state = state;
                None
            }
            DecodeResult::Unrecognized => {
                self.state = DecodeState::Start;
                None
            }
        }
    }

    fn decode_single_byte(scancode: u8) -> DecodeResult {
        use KeyCode as KC;

        let (action, lower_7_bits) = extract_action(scancode);

        #[expect(clippy::match_same_arms)]
        let key_code = match lower_7_bits {
            0x00 => return DecodeResult::Unrecognized,
            0x01 => KC::KeyEsc,
            0x02 => KC::Key1,
            0x03 => KC::Key2,
            0x04 => KC::Key3,
            0x05 => KC::Key4,
            0x06 => KC::Key5,
            0x07 => KC::Key6,
            0x08 => KC::Key7,
            0x09 => KC::Key8,
            0x0a => KC::Key9,
            0x0b => KC::Key0,
            0x0c => KC::KeyMinus,
            0x0d => KC::KeyEquals,
            0x0e => KC::KeyBackspace,
            0x0f => KC::KeyTab,
            0x10 => KC::KeyQ,
            0x11 => KC::KeyW,
            0x12 => KC::KeyE,
            0x13 => KC::KeyR,
            0x14 => KC::KeyT,
            0x15 => KC::KeyY,
            0x16 => KC::KeyU,
            0x17 => KC::KeyI,
            0x18 => KC::KeyO,
            0x19 => KC::KeyP,
            0x1a => KC::KeyLeftSquareBracket,
            0x1b => KC::KeyRightSquareBracket,
            0x1c => KC::KeyEnter,
            0x1d => KC::KeyLeftControl,
            0x1e => KC::KeyA,
            0x1f => KC::KeyS,
            0x20 => KC::KeyD,
            0x21 => KC::KeyF,
            0x22 => KC::KeyG,
            0x23 => KC::KeyH,
            0x24 => KC::KeyJ,
            0x25 => KC::KeyK,
            0x26 => KC::KeyL,
            0x27 => KC::KeySemicolon,
            0x28 => KC::KeyApostrophe,
            0x29 => KC::KeyBackTick,
            0x2a => KC::KeyLeftShift,
            0x2b => KC::KeyBackslash,
            0x2c => KC::KeyZ,
            0x2d => KC::KeyX,
            0x2e => KC::KeyC,
            0x2f => KC::KeyV,
            0x30 => KC::KeyB,
            0x31 => KC::KeyN,
            0x32 => KC::KeyM,
            0x33 => KC::KeyComma,
            0x34 => KC::KeyPeriod,
            0x35 => KC::KeySlash,
            0x36 => KC::KeyRightShift,
            0x37 => return DecodeResult::Unrecognized,
            0x38 => KC::KeyLeftAlt,
            0x39 => KC::KeySpace,
            0x3a => KC::KeyCapsLock,
            0x3b => KC::KeyF1,
            0x3c => KC::KeyF2,
            0x3d => KC::KeyF3,
            0x3e => KC::KeyF4,
            0x3f => KC::KeyF5,
            0x40 => KC::KeyF6,
            0x41 => KC::KeyF7,
            0x42 => KC::KeyF8,
            0x43 => KC::KeyF9,
            0x44 => KC::KeyF10,
            0x45 => return DecodeResult::Unrecognized,
            0x46 => KC::KeyScrollLock,
            0x47..=0x53 => return DecodeResult::Unrecognized,
            0x54..=0x56 => return DecodeResult::Unrecognized,
            0x57 => KC::KeyF11,
            0x58 => KC::KeyF12,
            0x59..=0x7f => return DecodeResult::Unrecognized,
            0x80..=0xff => unreachable!(),
        };

        DecodeResult::Ok(key_code, action)
    }

    const fn decode_double_byte(scancode: u8) -> DecodeResult {
        let (action, lower_7_bits) = extract_action(scancode);

        let key_code = match lower_7_bits {
            0x38 => KeyCode::KeyRightAlt,
            0x47 => KeyCode::KeyHome,
            0x48 => KeyCode::KeyArrowUp,
            0x49 => KeyCode::KeyPageUp,
            0x4b => KeyCode::KeyArrowLeft,
            0x4d => KeyCode::KeyArrowRight,
            0x4f => KeyCode::KeyEnd,
            0x50 => KeyCode::KeyArrowDown,
            0x51 => KeyCode::KeyPageDown,
            0x52 => KeyCode::KeyInsert,
            0x53 => KeyCode::KeyDelete,
            0x5b => KeyCode::KeyLeftBoss,
            0x5c => KeyCode::KeyRightBoss,
            0x1d => KeyCode::KeyRightControl,

            _ => return DecodeResult::Unrecognized,
        };

        DecodeResult::Ok(key_code, action)
    }
}

const fn extract_action(scancode: u8) -> (KeyAction, u8) {
    let action = if scancode >> 7 == 1 {
        KeyAction::Release
    } else {
        KeyAction::Press
    };

    let lower_7_bits = scancode & !(1 << 7);
    (action, lower_7_bits)
}

pub const fn key_code_to_ascii(key_code: KeyCode) -> Option<u8> {
    Some(match key_code {
        KeyCode::Key0 => b'0',
        KeyCode::Key1 => b'1',
        KeyCode::Key2 => b'2',
        KeyCode::Key3 => b'3',
        KeyCode::Key4 => b'4',
        KeyCode::Key5 => b'5',
        KeyCode::Key6 => b'6',
        KeyCode::Key7 => b'7',
        KeyCode::Key8 => b'8',
        KeyCode::Key9 => b'9',
        KeyCode::KeyA => b'a',
        KeyCode::KeyB => b'b',
        KeyCode::KeyC => b'c',
        KeyCode::KeyD => b'd',
        KeyCode::KeyE => b'e',
        KeyCode::KeyF => b'f',
        KeyCode::KeyG => b'g',
        KeyCode::KeyH => b'h',
        KeyCode::KeyI => b'i',
        KeyCode::KeyJ => b'j',
        KeyCode::KeyK => b'k',
        KeyCode::KeyL => b'l',
        KeyCode::KeyM => b'm',
        KeyCode::KeyN => b'n',
        KeyCode::KeyO => b'o',
        KeyCode::KeyP => b'p',
        KeyCode::KeyQ => b'q',
        KeyCode::KeyR => b'r',
        KeyCode::KeyS => b's',
        KeyCode::KeyT => b't',
        KeyCode::KeyU => b'u',
        KeyCode::KeyV => b'v',
        KeyCode::KeyW => b'w',
        KeyCode::KeyX => b'x',
        KeyCode::KeyY => b'y',
        KeyCode::KeyZ => b'z',
        KeyCode::KeySpace => b' ',
        KeyCode::KeyEnter => b'\n',
        KeyCode::KeyTab => b'\t',
        KeyCode::KeyMinus => b'-',
        KeyCode::KeyEquals => b'=',
        KeyCode::KeyApostrophe => b'\'',
        KeyCode::KeyBackTick => b'`',
        KeyCode::KeyComma => b',',
        KeyCode::KeyPeriod => b'.',
        KeyCode::KeySlash => b'/',
        KeyCode::KeyBackslash => b'\\',
        KeyCode::KeyLeftSquareBracket => b'[',
        KeyCode::KeyRightSquareBracket => b']',
        _ => return None,
    })
}

#[derive(Clone, Copy)]
pub enum KeyCode {
    KeyEsc,
    KeyF1,
    KeyF2,
    KeyF3,
    KeyF4,
    KeyF5,
    KeyF6,
    KeyF7,
    KeyF8,
    KeyF9,
    KeyF10,
    KeyF11,
    KeyF12,
    KeyBackTick,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Key0,
    KeyMinus,
    KeyEquals,
    KeyBackspace,
    KeyTab,
    KeyQ,
    KeyW,
    KeyE,
    KeyR,
    KeyT,
    KeyY,
    KeyU,
    KeyI,
    KeyO,
    KeyP,
    KeyLeftSquareBracket,
    KeyRightSquareBracket,
    KeyBackslash,
    KeyCapsLock,
    KeyA,
    KeyS,
    KeyD,
    KeyF,
    KeyG,
    KeyH,
    KeyJ,
    KeyK,
    KeyL,
    KeySemicolon,
    KeyApostrophe,
    KeyEnter,
    KeyLeftShift,
    KeyZ,
    KeyX,
    KeyC,
    KeyV,
    KeyN,
    KeyB,
    KeyM,
    KeyComma,
    KeyPeriod,
    KeySlash,
    KeyRightShift,
    KeyLeftControl,
    KeyLeftBoss,
    KeyLeftAlt,
    KeySpace,
    KeyRightAlt,
    KeyRightBoss,
    KeyRightControl,
    KeyPrintScreen,
    KeyScrollLock,
    KeyPause,
    KeyHome,
    KeyPageUp,
    KeyDelete,
    KeyEnd,
    KeyPageDown,
    KeyArrowUp,
    KeyArrowLeft,
    KeyArrowDown,
    KeyArrowRight,
    KeyInsert,
}
