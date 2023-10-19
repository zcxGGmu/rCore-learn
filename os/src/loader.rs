//! Loading user applications into memory

use alloc::vec::Vec;
use lazy_static::*;

/// get the total number of applications
pub fn get_num_app() -> usize {
    extern "C" {
        fn _num_app();
    }
    unsafe {
        (_num_app as usize as *const usize).read_volatile()
    }
}

/// get elf_data by app_id 
pub fn get_app_data(app_id: usize) -> &'static [u8] {
   extern "C" {
       fn _num_app();
   }
   let num_app_ptr = _num_app as usize as *const usize;
   let num_app = get_num_app();
   assert!(app_id < num_app);

   // app_start records all address_begin of each app
   let app_start = unsafe {
       core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1)
   };
   unsafe {
       core::slice::from_raw_parts(
           app_start[app_id] as *const u8,
           app_start[app_id + 1] - app_start[app_id]
       )
   }
}

lazy_static! {
    /// all of app's name
    static ref APP_NAMES: Vec<&'static str> = {
        let num_app = get_num_app();
        extern "C" {
            fn _app_names();
        }
        let mut start = _app_names as usize as *const u8;
        let mut v = Vec::new();
        unsafe {
            for _ in 0..num_app {
                let mut end = start;
                while end.read_volatile() != '\0' as u8 {
                    end = end.add(1);
                }   
                let str_slice = core::slice::from_raw_parts(start, end as usize - start as usize);
                let str = core::str::from_utf8(str_slice).unwrap();
                v.push(str);
                start = end.add(1);
            }
        }
        v
    };
}

#[allow(unused)]
/// get app data from name
pub fn get_app_data_by_name(name: &str) -> Option<&'static [u8]> {
    let num_app: usize = get_num_app();
    (0..num_app)
        .find(|&i| APP_NAMES[i] == name)
        .map(|i| get_app_data(i))
}

/// list of all apps
pub fn list_apps() {
    println!("/**** APPS_LIST ****/");
    for app in APP_NAMES.iter() {
        println!("{}", app);
    }
    println!("/*******************/");
}
