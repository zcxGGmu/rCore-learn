use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)]
pub struct TrapContext {
    pub x: [usize; 32], //x0~x31
    pub sstatus: Sstatus, //CSR sstatus
    pub sepc: usize, //CSR sepc
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        
    }

    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        
    }
}
