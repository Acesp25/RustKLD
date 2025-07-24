use kernel::*;
use alloc:: boxed::Box;
use libc::{c_int, c_void, EINVAL, EFAULT};
use core::{mem, ptr, cmp::min};

const BUFFERSIZE: usize = 256;

#[repr(C)]
struct EchoMsg {
    len: usize,
    msg: [u8; BUFFERSIZE],
}
impl EchoMsg {
    pub fn get_len(&self) -> usize {
        self.len   
    }
    pub fn reset_msg(&mut self, pos: usize) {
        self.msg[pos] = 0;
    }
    pub fn set_len(&mut self, new_length: usize) {
        self.len = new_length 
    }
}

pub struct CharacterDevice {
    cdevsw_ptr: *mut cdevsw,
    echo_dev: *mut cdev,
    echo_buf: Box<EchoMsg>,
}

impl CharacterDevice {
    fn cdevsw_init() -> cdevsw {
        cdevsw {
            d_version: D_VERSION,
            d_name: cstr_ptr!("echo"),
            d_open: Some(echo_open),
            d_close: Some(echo_close),
            d_read: Some(echo_read),
            d_write: Some(echo_write),
            .. unsafe { mem::zeroed() }
        }
    }
    
    pub fn new() -> Result<Box<Self>, c_int> {
        // move cdevsw to the heap using Box
        let boxed_cdevsw = Box::new(Self::cdevsw_init());
        // get raw pointer to give to kernel during make_dev_p call
        let cdevsw_ptr: *mut cdevsw = Box::into_raw(boxed_cdevsw);

        let mut echo_dev: *mut cdev = ptr::null_mut();
        let error = unsafe {
            make_dev_p(MAKEDEV_CHECKNAME | MAKEDEV_WAITOK,
		        &mut echo_dev,
		        cdevsw_ptr,
		        core::ptr::null_mut(),
		        UID_ROOT.try_into().unwrap(),
		        GID_WHEEL.try_into().unwrap(),
		        0600,
		        cstr_ptr!("echo"),
            )
        };

        if error != 0 {
            unsafe { let _ = Box::from_raw(cdevsw_ptr); } // reclaim and free cdevsw on failure
            return Err(error);
        }

        let echo_buf = Box::new(EchoMsg {len: 0, msg: [0; BUFFERSIZE]});
    
        let mut me = Box::new(Self {
            cdevsw_ptr,
            echo_dev,
            echo_buf,
        });
        unsafe {
            (*echo_dev).si_drv1 = (&mut *me as *mut CharacterDevice).cast();
        }

        Ok(me) 
    }
}
impl Drop for CharacterDevice {
    fn drop(&mut self) {
        unsafe {
            destroy_dev(self.echo_dev);
            let _ = Box::from_raw(self.cdevsw_ptr);
        }
    }
}
    
extern "C" fn echo_open(
    dev: *mut cdev,
    _oflags: c_int,
    _devtype: c_int,
    _td: *mut thread, 
) -> c_int {
    let error = 0;

    unsafe { dev_ref(dev) };

    println!("Echo Opened");
        
    error 
}

extern "C" fn echo_close(
    dev: *mut cdev,
    _oflags: c_int,
    _devtype: c_int,
    _td: *mut thread,
) -> c_int {
    let error = 0;

    unsafe { dev_rel(dev) };

    println!("Echo Closed");
    
    error
}

extern "C" fn echo_read(
    dev: *mut cdev,
    uio_ptr: *mut uio,
    _ioflag: c_int
) -> c_int {
    if uio_ptr.is_null() {
        println!("[echo_read] uio_ptr is NULL");
        return EFAULT;
    }
    unsafe { 
        let state = &mut *(*dev).si_drv1.cast::<CharacterDevice>();

        let resid = (*uio_ptr).uio_resid as usize;
        let offset = (*uio_ptr).uio_offset as usize;
        let length = state.echo_buf.len;
        
        let remain: usize;

        if offset >= length + 1 {
            remain = 0;
        } else {
            remain = length + 1 - offset;
        }
    
        let amt = min(resid, remain);

        let error = uiomove(state.echo_buf.msg.as_mut_ptr() as *mut c_void, 
                    amt as c_int,
                    uio_ptr,
        );

        error
    }
}

extern "C" fn echo_write(
    dev: *mut cdev,
    uio_ptr: *mut uio,
    _ioflag: c_int,
) -> c_int {
    if uio_ptr.is_null() {
        println!("[echo_write] uio_ptr is NULL");
        return EFAULT;
    }
    unsafe {
        let state = &mut *(*dev).si_drv1.cast::<CharacterDevice>();

        let offset = (*uio_ptr).uio_offset as usize;
        let length = state.echo_buf.get_len();
        let resid = (*uio_ptr).uio_resid as usize;
        
        if offset != 0 && offset != length {
            return EINVAL;
        }        
        
        if offset == 0 {
            state.echo_buf.set_len(0);
        }
        
        let amt = min(resid, BUFFERSIZE - length);
    
        let error = uiomove(state.echo_buf.msg.as_mut_ptr().add(offset) as *mut c_void,
                    amt as c_int,
                    uio_ptr,
        );
        
        state.echo_buf.set_len(offset + amt);
        state.echo_buf.reset_msg(state.echo_buf.get_len()); 

        error
    }
}
