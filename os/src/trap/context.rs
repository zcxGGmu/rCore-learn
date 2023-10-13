//! Implementation of TrapContext

use riscv::register::sstatus::{self, Sstatus, SPP};

#[repr(C)]
/**
    Currently, sscratch only saves the pointer to the TrapContext structure.
    Therefore, TrapContext must provide necessary information for application
    and kernel switching, such as {kernel_satp/kernel_sp/trap_handler}. Once
    they are written, they will not be changed again.
*/
pub struct TrapContext {
    /// general regs => x0~x31
    pub x: [usize; 32],
    
    /// CSR sstatus
    pub sstatus: Sstatus,
    
    /// CSR sepc
    pub sepc: usize,

    /// Base address of PageTabe
    pub kernel_satp: usize,

    /// Kernel stack
    pub kernel_sp: usize,
    
    /// Address of trap_handler
    pub trap_handler: usize,
}

impl TrapContext {
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    /// return initial TrapContext of application
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User); // we will ret to user!
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,
            kernel_satp,
            kernel_sp,
            trap_handler,
        };
        cx.set_sp(sp); // set user sp of application
        cx
    }
}
