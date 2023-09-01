#![feature(panic_info_message)]
#![no_main]
#![no_std]

#[macro_use]
mod console;
mod lang_items;
mod sbi;
mod logging;

use core::arch::global_asm;
use log::{*};
global_asm!(include_str!("entry.asm"));

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
    clear_bss();

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
