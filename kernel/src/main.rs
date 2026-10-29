#![no_std]
#![no_main]

use core::fmt::Write;
use core::panic::PanicInfo;
use serial::SERIAL1;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    SERIAL1
        .lock()
        .write_str("Kernel successfully started\n")
        .expect("Cannot write to QEMU port.");

    loop {
        core::hint::spin_loop();
    }
}


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
