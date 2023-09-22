//! Implementation of trap handling

mod context;

use core::arch::global_asm;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt ,Trap},
    sie, stval, stvec,
};
use crate::syscall::syscall;
use crate::task::{exit_current_and_run_next,
                  suspend_current_and_run_next};
use crate::timer::set_next_trigger;
use log::{*};

global_asm!(include_str!("trap.S"));

// init trap entry(`stvec`)
pub fn init() {
    extern "C" { fn __alltraps(); }
    unsafe {
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

/// timer interrupt enabled
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[no_mangle]
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            cx.sepc += 4;
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize; 
        }
        Trap::Exception(Exception::StoreFault) |
        Trap::Exception(Exception::StorePageFault) => {
            error!("[kernel] PageFault in application, kernel killed it!");
            exit_current_and_run_next();
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            error!("[kernel] IllegalInstruction in application,
                   kernel killed it!");
            exit_current_and_run_next();
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
    cx
}

pub use context::TrapContext;
