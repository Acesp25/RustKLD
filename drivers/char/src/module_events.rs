use kernel::*;
use libc::{c_int, ENXIO};
use alloc::boxed::Box;
use crate::char_device::CharacterDevice;

static mut ECHO_DEVICE: Option<Box<CharacterDevice>> = None;

pub struct Events;

impl Events {
    pub fn load() -> c_int{
        match CharacterDevice::new() {
            Ok(dev) => unsafe {
                ECHO_DEVICE = Some(dev);
                println!("[module_events.rs] Echo device loaded");
                0
            },
            Err(_err) => {
                println!("[module_events.rs] Echo device make failed");
                ENXIO
            },
        }
    }
    pub fn unload() -> c_int {
        unsafe {
            // deref raw ptr to get pointed Option<CharacterDevice>
            let ptr: *mut Option<Box<CharacterDevice>> = &raw mut ECHO_DEVICE;

            // call Option::take() to move Some(dev) out, leaving nothing behind
            let dev_out = (*ptr).take();

            // drop dev
            if let Some(dev) = dev_out {
                drop(dev);
            }
        }
        println!("[module_events.rs] Echo device unloaded");
        0
    }
    
    pub fn quiesce() -> c_int {
        println!("[module_events.rs] Quiesce!");
        0
    }
    
    pub fn shutdown() -> c_int {
        println!("[module_events.rs] Shutdown!");
        0
    }
}
