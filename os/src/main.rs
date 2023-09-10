#![feature(panic_info_message)]
#![no_main]
#![no_std]
#![deny(warnings)]

#[macro_use]
mod console;

pub mod batch;
mod lang_items;
mod sbi;
mod sync;
//mod logging;
pub mod syscall;
pub mod trap;
mod logging;
use core::arch::global_asm;
use log::{*};

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
    batch::init();
    
    batch::run_next_app();
}
