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
const BLINK_HZ: u32 = 1;
const LED_PIN: u8 = 15;

const TIMER_FREQ_HZ: u32 = 16_000_000;
const PRESCALER: u32 = (CHIP_FREQ_HZ / TIMER_FREQ_HZ).trailing_zeros();
const TOGGLE_PERIOD_TICKS: u32 = TIMER_FREQ_HZ / BLINK_HZ / 2;

static mut LED_STATE: bool = false;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    unsafe {
        let led = {
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

        LED_STATE = false;
        
        let timer = nrf52840_hal::timer::timer2::steal();
        timer.tasks_stop.write(1);
        
        timer.mode.write(0);
        timer.bitmode.write(0);
        timer.prescaler.write(PRESCALER);
        timer.cc[0].write(TOGGLE_PERIOD_TICKS);
        timer.tasks_clear.write(1);
        timer.tasks_start.write(1);

        loop {
            if timer.events_compare[0].read() != 0 {
                timer.events_compare[0].write(0);
                timer.tasks_clear.write(1);
                timer.tasks_start.write(1);
                
                LED_STATE = !LED_STATE;
                if LED_STATE {
                    led.set_high();
                } else {
                    led.set_low();
                }
            }
        }
    }
}
