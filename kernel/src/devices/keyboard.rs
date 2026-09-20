use libkernel::datastructures::RingBuffer;

use crate::io::IoPort;

pub struct Keyboard {
    port: IoPort,
    scancode_buffer: RingBuffer<u8, 6>,
    key_event_buffer: RingBuffer<KeyEvent, 10>,
}

pub static mut KEYBOARD: Keyboard = Keyboard {
    port: IoPort::new(0x60),
    scancode_buffer: RingBuffer::new(),
    key_event_buffer: RingBuffer::new(),
};

impl Keyboard {
    pub fn on_data(&mut self) {
        let byte = unsafe { self.port.in_byte() };
        self.scancode_buffer
            .push_back(byte)
            .expect("Keycodes should be no longer than 6 bytes");

        // DESIGN: Maybe we shouldn't do this in the interrupt handler?
        match parse_keycode(&self.scancode_buffer) {
            ParseResult::Ok(event) => {
                // DESIGN: Maybe this should drop old events instead of panicking?
                self.key_event_buffer
                    .push_back(event)
                    .expect("Key event buffer should be emptied");

                self.scancode_buffer.clear();
            }
            ParseResult::Unrecognized => self.scancode_buffer.clear(),
            ParseResult::Incomplete => {}
        }
    }

    pub fn poll_event(&mut self) -> Option<KeyEvent> {
        self.key_event_buffer.pop_front()
    }
}

#[derive(Clone, Copy)]
pub struct KeyEvent {
    pub key_code: KeyCode,
    pub action: KeyAction,
}

#[derive(Clone, Copy)]
enum ParseResult {
    Ok(KeyEvent),
    Unrecognized,
    Incomplete,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Press,
    Release,
}

fn parse_keycode<const N: usize>(codes: &RingBuffer<u8, N>) -> ParseResult {
    use KeyCode as KC;

    let Some(first_byte) = codes.get(0) else {
        return ParseResult::Incomplete;
    };

    // NOTE: This is not necessarily true, but for the subset of supported keys this holds
    let action = if first_byte & (1 << 7) == (1 << 7) {
        KeyAction::Release
    } else {
        KeyAction::Press
    };

    // TODO: Many physical keys are not yet mapped, i.e. numpad, media keys, arrows
    let key_code = match first_byte & (!(1 << 7)) {
        0x00 => return ParseResult::Unrecognized,
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
        0x37 => return ParseResult::Unrecognized,
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
        0x45 => return ParseResult::Unrecognized,
        0x46 => KC::KeyScrollLock,
        0x47..=0x53 => return ParseResult::Unrecognized,
        0x54..=0x56 => return ParseResult::Unrecognized,
        0x57 => KC::KeyF11,
        0x58 => KC::KeyF12,
        0x59..=0x7f => return ParseResult::Unrecognized,
        0x80..=0xff => unreachable!(),
    };

    let event = KeyEvent { key_code, action };
    ParseResult::Ok(event)
}

pub fn key_code_to_ascii(key_code: KeyCode) -> Option<u8> {
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
    KeyHelp,
    KeyHome,
    KeyPageUp,
    KeyDelete,
    KeyEnd,
    KeyPageDown,
    KeyArrowUp,
    KeyArrowLeft,
    KeyArrowDown,
    KeyArrowRight,
}
