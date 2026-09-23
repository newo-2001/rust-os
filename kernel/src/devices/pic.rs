use log::trace;

use crate::io::IoPort;

pub const MASTER_COMMAND_PORT: IoPort = IoPort::new(0x20);
pub const MASTER_DATA_PORT: IoPort = IoPort::new(0x21);

pub const SLAVE_COMMAND_PORT: IoPort = IoPort::new(0xa0);
pub const SLAVE_DATA_PORT: IoPort = IoPort::new(0xa1);

pub const END_OF_INTERRUPT: u8 = 0x20;

pub fn initialize() {
    configure_pic(PicConfiguration {
        command_port: MASTER_COMMAND_PORT,
        data_port: MASTER_DATA_PORT,
        starting_vector: 0x20,
        // For master we set bit 2 to identify IRQ2 as the slave
        connection: (1 << 2),
        // We enable IRQ2 (slave) and IRQ1(keyboard)
        port_mask: 0b1111_1001,
    });

    configure_pic(PicConfiguration {
        command_port: SLAVE_COMMAND_PORT,
        data_port: SLAVE_DATA_PORT,
        starting_vector: 0x28,
        // For slave we set the cascade identity to IRQ2
        connection: 2,
        port_mask: 0b1111_1111,
    });

    trace!("PIC initialized");
}

fn configure_pic(config: PicConfiguration) {
    const ICW1_INIT: u8 = 0x11;
    const ICW4_MODE_8086: u8 = 0x01;

    unsafe {
        config.command_port.out_byte(ICW1_INIT);
        config.data_port.out_byte(config.starting_vector);
        config.data_port.out_byte(config.connection);
        config.data_port.out_byte(ICW4_MODE_8086);
        config.data_port.out_byte(config.port_mask);
    }
}

#[derive(Clone, Copy)]
struct PicConfiguration {
    command_port: IoPort,
    data_port: IoPort,
    starting_vector: u8,
    connection: u8,
    port_mask: u8,
}
