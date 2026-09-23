use core::fmt::Write;

use libkernel::sync::SpinLock;
use num_enum::TryFromPrimitive;

use crate::io::{self, IoPort};

pub static COM1: SpinLock<SerialPort> = SpinLock::new(SerialPort::new(0x3f8));

pub struct SerialPort {
    base_port: u16,
}

pub struct BaudRate {
    divisor: u16,
}

impl BaudRate {
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
            modem_status: false,
            receiver_line_status: false,
            transmitter_holding_register_empty: false,
            received_data_available: false,
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
            data_terminal_ready: true,
            request_to_send: true,
            out_1: true,
            out_2: true,
            loopback: false,
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
        self.write_register::<DivisorLatchRegistorHigh>(high_byte);

        line_controls.divisor_latch_access_enabled = false;
        self.write_register::<LineControlRegister>(line_controls);
    }

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
raw_register!(DivisorLatchRegistorHigh, 1);
raw_register!(ScratchRegister, 7);

struct InterruptEnableRegister;

impl Register for InterruptEnableRegister {
    const PORT_OFFSET: u16 = 1;
}

#[derive(Clone, Copy)]
struct EnabledInterrupts {
    pub modem_status: bool,
    pub receiver_line_status: bool,
    pub transmitter_holding_register_empty: bool,
    pub received_data_available: bool,
}

impl ReadRegister for InterruptEnableRegister {
    type Value = EnabledInterrupts;

    fn deserialize(value: u8) -> Self::Value {
        EnabledInterrupts {
            modem_status: ((value >> 3) & 1) == 1,
            receiver_line_status: ((value >> 2) & 1) == 1,
            transmitter_holding_register_empty: ((value >> 1) & 1) == 1,
            received_data_available: ((value >> 0) & 1) == 1,
        }
    }
}

impl WriteRegister for InterruptEnableRegister {
    type Value = EnabledInterrupts;

    fn serialize(value: Self::Value) -> u8 {
        u8::from(value.modem_status) << 3
            | u8::from(value.receiver_line_status) << 2
            | u8::from(value.transmitter_holding_register_empty) << 1
            | u8::from(value.received_data_available) << 0
    }
}

struct FifoControlRegister;

impl Register for FifoControlRegister {
    const PORT_OFFSET: u16 = 2;
}

// DESIGN: interrupt_trigger_level is only applicable if enable_fifos is set.
// Ideally the type system would enforce this
#[derive(Clone, Copy)]
struct FifoControls {
    pub enable_fifos: bool,
    pub interrupt_trigger_level: InterruptTriggerLevel,
    pub clear_buffers: ClearBufferOptions,
}

#[derive(Clone, Copy)]
enum InterruptTriggerLevel {
    Bytes1 = 0,
    Bytes4 = 1,
    Bytes8 = 2,
    Bytes14 = 3,
}

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
        u8::from(value.enable_fifos) << 0
            | (value.clear_buffers as u8) << 1
            | (value.interrupt_trigger_level as u8) << 6
    }
}

struct InterruptIdentificationRegister;

impl Register for InterruptIdentificationRegister {
    const PORT_OFFSET: u16 = 2;
}

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
        (value.data_bits as u8) << 0
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
    pub data_terminal_ready: bool,
    pub request_to_send: bool,
    pub out_1: bool,
    pub out_2: bool,
    pub loopback: bool,
}

impl WriteRegister for ModemControlRegister {
    type Value = ModemControls;

    fn serialize(value: Self::Value) -> u8 {
        u8::from(value.data_terminal_ready) << 0
            | u8::from(value.request_to_send) << 1
            | u8::from(value.out_1) << 2
            | u8::from(value.out_2) << 3
            | u8::from(value.loopback) << 4
    }
}

impl ReadRegister for ModemControlRegister {
    type Value = ModemControls;

    fn deserialize(byte: u8) -> Self::Value {
        ModemControls {
            data_terminal_ready: (byte >> 0) & 1 == 1,
            request_to_send: (byte >> 1) & 1 == 1,
            out_1: (byte >> 2) & 1 == 1,
            out_2: (byte >> 3) & 1 == 1,
            loopback: (byte >> 4) & 1 == 1,
        }
    }
}
