#![feature(panic_info_message)]
#![no_std]

#[macro_use]
mod console;
mod lang_items;
mod sbi;
mod logging;

use core::arch::global_asm;
use log::{*};
global_asm!(include_str!("entry.asm"));

pub fn print_memory_log() -> ! { 
    extern "C" {
        fn stext();
        fn etext();
        fn srodata();
        fn erodata();
        fn sdata();
        fn edata();
        fn sbss();
        fn ebss();
        fn boot_stack_lower_bound();
        fn boot_stack_top();
    }

    logging::init();
    warn!("Deallocate frame: {:#x}",
          stext as usize);
    info!(".text [{:#x}, {:#x})",
            stext as usize, 
            etext as usize);
    debug!(".rodata [{:#x}, {:#x})",
            srodata as usize,
            erodata as usize);
    trace!(".data [{:#x}, {:#x})",
            sdata as usize,
            edata as usize);
    error!("boot_stack [{:#x}, {:#x})",
            boot_stack_lower_bound as usize,
            boot_stack_top as usize);
    println!(".bss [{:#x}, {:#x})",
            sbss as usize,
            ebss as usize);
    panic!("Shutdown machine!!!");
}
