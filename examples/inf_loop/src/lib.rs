#![no_std]

#[no_mangle]
pub unsafe extern "C" fn handle() {
    gstd::debug!("loop begin");
    loop {
    }
    gstd::debug!("fib end");
}

#[no_mangle]
pub unsafe extern "C" fn init() {}
