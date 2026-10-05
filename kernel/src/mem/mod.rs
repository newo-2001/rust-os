pub mod pagetable;
mod address;
mod physical_memory_manager;

pub use address::{Address, PhysicalAddress};
pub use physical_memory_manager::PHYSICAL_MEMORY_MANAGER;
