#![no_std]
#![no_main]

use chopin_kernel as _;
use chopin_kernel_stage0 as _;


extern "C" {
    fn uart_print(s: *const core::ffi::c_char);
}

#[no_mangle]
extern "C" fn CHOPIN_kern_start() {

    unsafe {
        uart_print(c"Hello, World!".as_ptr() as *const _);
    }

    loop {}
}
