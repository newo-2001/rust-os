use core::arch::asm;

use log::trace;

use crate::{
    gdt::{self, PrivilegeLevel},
    interrupts::{InterruptHandler, keyboard_interrupt_handler},
};

type Idt = [IdtEntry; 256];

// Table needs to be mutable because the final handler addresses are not known at compile time
static mut INTERRUPT_DESCRIPTOR_TABLE: Idt = [IdtEntry(0); 256];

pub fn load() {
    let keyboard_interrupt_vector = IdtEntry::new(
        keyboard_interrupt_handler,
        gdt::KERNEL_CODE_SELECTOR,
        TypeAttributes {
            gate_type: GateType::InterruptGate32,
            privilege_level: PrivilegeLevel::Kernel,
        },
    );

    unsafe { INTERRUPT_DESCRIPTOR_TABLE[33] = keyboard_interrupt_vector };

    let pointer = IdtPointer {
        limit: (core::mem::size_of::<Idt>() - 1) as u16,
        base: (&raw const INTERRUPT_DESCRIPTOR_TABLE) as *const _ as u32,
    };

    unsafe {
        asm!(
            "lidt [{pointer}]",
            pointer = in(reg) &pointer
        )
    }

    trace!("IDT loaded")
}

pub fn read() -> IdtPointer {
    let mut idt = IdtPointer { limit: 0, base: 0 };

    unsafe {
        asm!(
            "sidt [{idt_out}]",
            idt_out = in(reg) &mut idt
        )
    }

    idt
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct IdtPointer {
    pub limit: u16,
    pub base: u32,
}

#[repr(transparent)]
#[derive(Clone, Copy)]
struct IdtEntry(u64);

impl IdtEntry {
    fn new(
        handler: InterruptHandler,
        segment_selector: u16,
        type_attributes: TypeAttributes,
    ) -> Self {
        let offset = handler as *const () as u64;
        let offset_low = offset & 0xffff;
        let offset_high = (offset >> 16) & 0xffff;

        let reserved = 0u64 << 0;
        let segment_selector = u64::from(segment_selector);
        let attribute_bits = u64::from(type_attributes.bits());

        let value = (offset_low << 0)
            | (segment_selector << 16)
            | (reserved << 32)
            | (attribute_bits << 40)
            | (offset_high << 48);

        Self(value)
    }
}

#[derive(Clone, Copy)]
struct TypeAttributes {
    gate_type: GateType,
    privilege_level: PrivilegeLevel,
}

impl TypeAttributes {
    const fn bits(self) -> u8 {
        let gate_type = (self.gate_type as u8) << 0;
        let zero_bit = 0u8 << 4;
        let privilege_bits = (self.privilege_level as u8) << 5;
        let present_bit = 1u8 << 7;

        gate_type | zero_bit | privilege_bits | present_bit
    }
}

#[derive(Clone, Copy)]
enum GateType {
    #[expect(unused)]
    TaskGate = 0x5,
    #[expect(unused)]
    InterruptGate16 = 0x6,
    #[expect(unused)]
    TrapGate16 = 0x7,
    InterruptGate32 = 0xe,
    #[expect(unused)]
    TrapGate32 = 0xf,
}
