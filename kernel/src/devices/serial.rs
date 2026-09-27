use core::fmt::Write;

use libkernel::sync::SpinLock;
use num_enum::TryFromPrimitive;

use crate::io::{self, IoPort};

pub static COM1: SpinLock<SerialPort> = SpinLock::new(SerialPort::new(0x3f8));

pub struct SerialPort {
    base_port: u16,
}

#[derive(Clone, Copy)]
pub struct BaudRate {
    divisor: u16,
}

impl BaudRate {
    // DESIGN: Do we want to return Option here or perform best effort rounding?
    pub fn new(baud_rate: u32) -> Option<Self> {
        const UART_CLOCK_HZ: u32 = 115_200;

        UART_CLOCK_HZ
            .div_exact(baud_rate)
            .and_then(|divisor| u16::try_from(divisor).ok())
            .map(|divisor| Self { divisor })
    }
}

impl SerialPort {
    pub const fn new(base_port: u16) -> Self {
        assert!(base_port <= u16::MAX - 7);

        Self { base_port }
    }

    pub fn initialize(&mut self) {
        self.write_register::<InterruptEnableRegister>(EnabledInterrupts {
            modem_status: InterruptActive::Disabled,
            receiver_line_status: InterruptActive::Disabled,
            transmitter_holding_register_empty: InterruptActive::Disabled,
            received_data_available: InterruptActive::Disabled,
        });

        self.write_register::<LineControlRegister>(LineControls {
            data_bits: DataBits::Bits8,
            parity_bits: ParityBits::None,
            stop_bits: StopBits::Bits1,
            divisor_latch_access_enabled: false,
            break_enabled: false,
        });

        self.set_baud_rate(BaudRate::new(38_400).unwrap());

        self.write_register::<ModemControlRegister>(ModemControls {
            data_terminal_ready: Readiness::Ready,
            request_to_send: true,
            irq_enabled: true,
            loopback_enabled: false,
        });
    }

    pub fn write_byte(&mut self, byte: u8) {
        self.write_register::<DataRegister>(byte);
    }

    pub fn set_baud_rate(&mut self, baud_rate: BaudRate) {
        let mut line_controls = self.read_register::<LineControlRegister>();
        line_controls.divisor_latch_access_enabled = true;

        let low_byte = (baud_rate.divisor & 0xff) as u8;
        let high_byte = (baud_rate.divisor >> 8) as u8;

        self.write_register::<DivisorLatchRegisterLow>(low_byte);
        self.write_register::<DivisorLatchRegisterHigh>(high_byte);

        line_controls.divisor_latch_access_enabled = false;
        self.write_register::<LineControlRegister>(line_controls);
    }

    // We take &mut self to represent that this induces an external side-effect
    #[expect(clippy::needless_pass_by_ref_mut)]
    fn write_register<R>(&mut self, value: R::Value)
    where
        R: WriteRegister,
    {
        let port = IoPort::<io::ModeWrite>::new(self.base_port + R::PORT_OFFSET);
        let byte = R::serialize(value);

        unsafe {
            port.out_byte(byte);
        }
    }

    fn read_register<R>(&self) -> R::Value
    where
        R: ReadRegister,
    {
        let port = IoPort::<io::ModeRead>::new(self.base_port + R::PORT_OFFSET);
        let byte = unsafe { port.in_byte() };
        R::deserialize(byte)
    }
}

impl Write for SerialPort {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            self.write_byte(c);
        }

        Ok(())
    }
}

trait Register {
    const PORT_OFFSET: u16;
}

trait ReadRegister: Register {
    type Value;

    fn deserialize(byte: u8) -> Self::Value;
}

trait WriteRegister: Register {
    type Value;

    fn serialize(value: Self::Value) -> u8;
}

macro_rules! raw_register {
    ($type:ident, $port_offset:literal) => {
        #[allow(unused)]
        struct $type;

        impl Register for $type {
            const PORT_OFFSET: u16 = $port_offset;
        }

        impl ReadRegister for $type {
            type Value = u8;

            fn deserialize(byte: u8) -> Self::Value {
                byte
            }
        }

        impl WriteRegister for $type {
            type Value = u8;

            fn serialize(value: Self::Value) -> u8 {
                value
            }
        }
    };
}

raw_register!(DataRegister, 0);
raw_register!(DivisorLatchRegisterLow, 0);
raw_register!(DivisorLatchRegisterHigh, 1);
raw_register!(ScratchRegister, 7);

struct InterruptEnableRegister;

impl Register for InterruptEnableRegister {
    const PORT_OFFSET: u16 = 1;
}

#[derive(Clone, Copy)]
struct EnabledInterrupts {
    pub modem_status: InterruptActive,
    pub receiver_line_status: InterruptActive,
    pub transmitter_holding_register_empty: InterruptActive,
    pub received_data_available: InterruptActive,
}

#[derive(Clone, Copy)]
enum InterruptActive {
    Disabled = 0,
    Enabled = 1,
}

impl From<bool> for InterruptActive {
    fn from(value: bool) -> Self {
        if value { Self::Enabled } else { Self::Disabled }
    }
}

impl ReadRegister for InterruptEnableRegister {
    type Value = EnabledInterrupts;

    fn deserialize(value: u8) -> Self::Value {
        EnabledInterrupts {
            modem_status: InterruptActive::from((value >> 3) & 1 == 1),
            receiver_line_status: InterruptActive::from((value >> 2) & 1 == 1),
            transmitter_holding_register_empty: InterruptActive::from((value >> 1) & 1 == 1),
            received_data_available: InterruptActive::from(value & 1 == 1)
        }
    }
}

impl WriteRegister for InterruptEnableRegister {
    type Value = EnabledInterrupts;

    fn serialize(value: Self::Value) -> u8 {
        (value.modem_status as u8) << 3
            | (value.receiver_line_status as u8) << 2
            | (value.transmitter_holding_register_empty as u8) << 1
            | (value.received_data_available as u8)
    }
}

#[expect(unused)]
struct FifoControlRegister;

impl Register for FifoControlRegister {
    const PORT_OFFSET: u16 = 2;
}

// DESIGN: interrupt_trigger_level is only applicable if enable_fifos is set.
// Ideally the type system would enforce this
#[expect(unused)]
#[derive(Clone, Copy)]
struct FifoControls {
    pub enable_fifos: bool,
    pub interrupt_trigger_level: InterruptTriggerLevel,
    pub clear_buffers: ClearBufferOptions,
}

#[expect(unused)]
#[derive(Clone, Copy)]
enum InterruptTriggerLevel {
    Bytes1 = 0,
    Bytes4 = 1,
    Bytes8 = 2,
    Bytes14 = 3,
}

#[expect(unused)]
#[derive(Clone, Copy)]
enum ClearBufferOptions {
    None = 0,
    ReceiveBuffer = 1,
    TransmitBuffer = 2,
    Both = 3,
}

impl WriteRegister for FifoControlRegister {
    type Value = FifoControls;

    fn serialize(value: Self::Value) -> u8 {
        u8::from(value.enable_fifos)
            | (value.clear_buffers as u8) << 1
            | (value.interrupt_trigger_level as u8) << 6
    }
}

#[expect(unused)]
struct InterruptIdentificationRegister;

impl Register for InterruptIdentificationRegister {
    const PORT_OFFSET: u16 = 2;
}

#[expect(unused)]
#[derive(Clone, Copy)]
struct InterruptIdentification {
    pub interrupt_pending: bool,
    pub interrupt_state: InterruptState,
    pub fifo_buffer_state: FifoBufferState,
}

#[derive(Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
enum InterruptState {
    ModemStatus = 0,
    TransmitterHoldingRegisterEmpty = 1,
    ReceivedDataAvailable = 2,
    ReceiverLineStatus = 3,
}

#[derive(Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
enum FifoBufferState {
    NoFifo = 0,
    EnabledButUnusable = 1,
    Enabled = 2,
}

impl ReadRegister for InterruptIdentificationRegister {
    type Value = InterruptIdentification;

    fn deserialize(byte: u8) -> Self::Value {
        InterruptIdentification {
            interrupt_pending: (byte & 1) == 0,
            interrupt_state: ((byte >> 1) & 0b11).try_into().unwrap(),
            fifo_buffer_state: ((byte >> 6) & 0b11).try_into().unwrap(),
        }
    }
}

struct LineControlRegister;

impl Register for LineControlRegister {
    const PORT_OFFSET: u16 = 3;
}

#[derive(Clone, Copy)]
struct LineControls {
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity_bits: ParityBits,
    pub break_enabled: bool,
    pub divisor_latch_access_enabled: bool,
}

#[derive(Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
enum DataBits {
    Bits5 = 0,
    Bits6 = 1,
    Bits7 = 2,
    Bits8 = 3,
}

#[derive(Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
enum StopBits {
    Bits1 = 0,
    // Technically this can also be 1.5 if data bits is 5
    // but the name would be _really_ awkward
    Bits2 = 1,
}

#[derive(Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
enum ParityBits {
    None = 0b000,
    Odd = 0b001,
    Even = 0b0011,
    Mark = 0b101,
    Space = 0b111,
}

impl WriteRegister for LineControlRegister {
    type Value = LineControls;

    fn serialize(value: Self::Value) -> u8 {
        (value.data_bits as u8)
            | (value.stop_bits as u8) << 2
            | (value.parity_bits as u8) << 3
            | u8::from(value.break_enabled) << 6
            | u8::from(value.divisor_latch_access_enabled) << 7
    }
}

impl ReadRegister for LineControlRegister {
    type Value = LineControls;

    fn deserialize(byte: u8) -> Self::Value {
        LineControls {
            data_bits: (byte & 0b11).try_into().unwrap(),
            stop_bits: ((byte >> 2) & 1).try_into().unwrap(),
            parity_bits: ((byte >> 3) & 0b111).try_into().unwrap(),
            break_enabled: (byte >> 6) & 1 == 1,
            divisor_latch_access_enabled: (byte >> 7) & 1 == 1,
        }
    }
}

struct ModemControlRegister;

impl Register for ModemControlRegister {
    const PORT_OFFSET: u16 = 4;
}

#[derive(Clone, Copy)]
struct ModemControls {
    pub data_terminal_ready: Readiness,
    pub request_to_send: bool,
    pub irq_enabled: bool,
    pub loopback_enabled: bool,
}

#[derive(Clone, Copy)]
enum Readiness {
    NotReady = 0,
    Ready = 1,
}

impl From<bool> for Readiness {
    fn from(value: bool) -> Self {
        if value { Self::Ready } else { Self::NotReady }
    }
}

impl WriteRegister for ModemControlRegister {
    type Value = ModemControls;

    fn serialize(value: Self::Value) -> u8 {
        (value.data_terminal_ready as u8)
            | u8::from(value.request_to_send) << 1
            | u8::from(value.irq_enabled) << 3
            | u8::from(value.loopback_enabled) << 4
    }
}

impl ReadRegister for ModemControlRegister {
    type Value = ModemControls;

    fn deserialize(byte: u8) -> Self::Value {
        ModemControls {
            data_terminal_ready: Readiness::from(byte & 1 == 1),
            request_to_send: (byte >> 1) & 1 == 1,
            irq_enabled: (byte >> 3) & 1 == 1,
            loopback_enabled: (byte >> 4) & 1 == 1,
        }
    }
}
