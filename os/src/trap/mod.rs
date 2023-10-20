//! Implementation of trap handling
mod context;

use core::arch::{asm, global_asm};
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt ,Trap},
    sie, stval, stvec,
};
use crate::syscall::syscall;
use crate::task::{
    exit_current_and_run_next,
    suspend_current_and_run_next,
    current_user_token,
    current_trap_cx,
};
use crate::timer::set_next_trigger;
use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use log::{*};

global_asm!(include_str!("trap.S"));

// initialize CSR `stvec` as the "__alltraps"
pub fn init() {
    extern "C" { fn __alltraps(); }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

#[no_mangle]
/// Unimplemented: traps from kernel mode
/// TODO: ch9_`IO_device`
pub fn trap_from_kernel() -> ! {
    panic!("a trap from kernel is not implemented in current rCore!");
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE, TrapMode::Direct);
    }
}

/// timer interrupt enabled
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[no_mangle]
/**
    When the kernel calls 'trap_return' to indicate that the process 
    has switched or processed some other traps (system calls, etc.), 
    it is about to enter the trap_context recovery process ''__restore'. 
    We need to provide two pieces of information:
    1) user space token => a0
    2) *TrapContext => a1 
    __restore switches the address space on the basis of the above 
    information and restores the saved TrapContext to the registers.
*/
pub fn trap_return() -> ! {
    /*
        We need to make sure that our application jumps to the correct 
        location when it fires traps in user mode.
    */
    set_user_trap_entry();

    let user_satp = current_user_token();
    let trap_cx_ptr = TRAP_CONTEXT;
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            options(noreturn)
        );
    }
}

#[no_mangle]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let scause = scause::read();
    let stval = stval::read();
    // syscall distribute 
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            // jump to next instruction
            let mut cx = current_trap_cx();
            cx.sepc += 4;
            // get syscall return value
            let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize; 
            // cx is changed probably during `sys_exec`, so we have to call it again
            cx = current_trap_cx();
            cx.x[10] = result as usize;
        }
        Trap::Exception(Exception::InstructionFault) |
        Trap::Exception(Exception::InstructionPageFault) |
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) |
        Trap::Exception(Exception::LoadFault) |
        Trap::Exception(Exception::LoadPageFault) => {
            error!("[kernel] {:?} in application,
                   addr_bad = {:#x}, instruction_bad = {:#x},
                   kernel killed it.",
                   scause.cause(),
                   stval,
                   current_trap_cx().sepc,
            );
            exit_current_and_run_next(-2);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            error!("[kernel] IllegalInstruction in application,
                   kernel killed it!");
            exit_current_and_run_next(-3);
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Current rCore unsupportied trap {:?}, stval = {:#x}",
                scause.cause(),
                stval
            );
        }
    }
    trap_return();
}

pub use context::TrapContext;
