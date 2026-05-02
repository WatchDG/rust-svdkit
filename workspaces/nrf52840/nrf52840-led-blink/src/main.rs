#![no_std]
#![no_main]
#![allow(unsafe_code)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_imports)]

use core::panic::PanicInfo;
use nrf52840_pac::rt;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { rt::DefaultHandler() }
    loop {}
}

use nrf52840_hal::gpio::p0;

const CHIP_FREQ_HZ: u32 = 64_000_000;
const BLINK_PERIOD_MS: u32 = 1_000;
const NOP_DELAY_PERIOD: u32 = (CHIP_FREQ_HZ / 1_000) * BLINK_PERIOD_MS;
const LED_PIN: u8 = 15;

fn delay_nops(iterations: u32) {
    for _ in 0..iterations {
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub extern "C" fn main() -> ! {
    let led = unsafe {
        let pin = p0::pin(LED_PIN);
        let configured = pin
            .configure()
            .dir(p0::Dir::Output)
            .pull(p0::Pull::Disabled)
            .drive(p0::Drive::S0s1)
            .sense(p0::Sense::Disabled)
            .apply();
        match configured {
            p0::PinConfigured::Output(out) => out,
            _ => loop {},
        }
    };

    loop {
        led.set_high();
        delay_nops(NOP_DELAY_PERIOD);
        led.set_low();
        delay_nops(NOP_DELAY_PERIOD);
    }
}
