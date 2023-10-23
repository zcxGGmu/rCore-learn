//! The main module and as real kernel entrypoint
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![no_main]
#![no_std]
#![deny(warnings)]
#![allow(unused)]

use core::arch::global_asm;
use log::{*};
extern crate alloc;
extern crate bitflags;

//#[cfg(not(any(feature = "board_k210")))]
#[path = "boards/qemu.rs"]
mod board;

#[macro_use]
mod console;
mod logging;
mod config;
mod lang_items;
mod sbi;
mod sync;
mod loader;
mod timer;
mod mm;
pub mod syscall;
pub mod trap;
pub mod task;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

/// clear .bss segment
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }

    (sbss as usize..ebss as usize).for_each(|a| {
        unsafe {
            (a as *mut u8).write_volatile(0)
        }
    });
}

#[no_mangle]
/// the entry point of rCore
pub fn rust_main() -> ! {
    clear_bss();
    logging::init();

    info!("[kernel] hello, rCore!");
    
    info!("------------------------------");
    info!("------------------------------");

    info!("[kernel] begin mm init...");
    mm::init();
    info!("[kernel] mm init completed!");
    task::add_initproc();
    info!("[kernel] after initproc!");

    info!("------------------------------");
    info!("------------------------------");

    info!("[kernel] begin remap_test...");
    mm::remap_test();

    info!("------------------------------");
    info!("------------------------------");

    info!("[kernel] begin trap init...");
    trap::init();
    info!("[kernel] trap init completed!");

    info!("------------------------------");
    info!("------------------------------");

    info!("[kernel] begin load apps...");

    trap::enable_timer_interrupt();
    timer::set_next_trigger();
     
    info!("[kernel] run first task...");
    loader::list_apps();
    task::run_tasks();

    panic!("Unreachable in rust_main!");
}
