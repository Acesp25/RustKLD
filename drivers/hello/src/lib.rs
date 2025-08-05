#![no_std]
#![no_main]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod hello;

use core::ffi::c_void;
use libc::{c_int, EOPNOTSUPP};
use kernel::*;
use hello::*;

use core::panic::PanicInfo;
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn module_event(
    _mod: *mut module,
    event: i32,
    _arg: *mut c_void,
) -> c_int {
    let mut error = 0;
    match ModEventType::from(event) {
        ModEventType::Load => {
            helloWorld::load();
        },
        ModEventType::Unload => {
            helloWorld::unload();
        },
        _ => {
            error = EOPNOTSUPP;
        }
    }
    error
}

#[no_mangle]
pub static mut hello_mod: moduledata_t = moduledata_t {
    name: cstr_ptr!("hello"),
    evhand: Some(module_event),
    priv_: core::ptr::null_mut(),
};
