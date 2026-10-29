#![no_std]

use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::{Config, Uart16550Tty};
use uart_16550::backend::PioBackend;

lazy_static! {
    pub static ref SERIAL1: Mutex<Uart16550Tty<PioBackend>> = {

        let uart = unsafe {
            Uart16550Tty::new_port(0x3F8, Config::DEFAULT)
                .expect("Failed to initialize UART port")
        };
        Mutex::new(uart)

    };
}