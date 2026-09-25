use core::fmt::Write;

use log::Log;

pub static LOGGER: SerialLogger = SerialLogger;

pub struct SerialLogger;

impl Log for SerialLogger {
    fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        let mut serial = crate::devices::serial::COM1.lock();
        writeln!(
            serial,
            "[{:<5}] ({}) {}",
            record.level(),
            record.target(),
            record.args()
        )
        .unwrap();
    }

    fn flush(&self) {}
}
