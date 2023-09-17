//! The main module and as real kernel entrypoint

#![feature(panic_info_imessage)]
#![no_main]
#![no_std]
#![deny(warnings)]
#![deny(missing_docs)]

use core::arch::global_asm;
use log::{*}

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
pub mod syscall;
pub mod trap;
pub mod task;

global_asm!(include_str!("entry.asm"));
global_asm!(include_str!("link_app.S"));

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
pub fn rust_main() -> ! {
    clear_bss();
    logging::init();
    info!("[kernel] Hello, rCore!");
    
    trap::init();
    loader::load_apps();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    task::run_first_task();
    panic!("Unreachable in rust_main!");
}
