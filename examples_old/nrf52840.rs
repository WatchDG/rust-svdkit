#![allow(unsafe_code)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_imports)]

mod nrf52840_pac {
    include!("../generated/nrf52840/nrf52840_pac/lib.rs");
}

mod nrf52840_hal {
    include!("../generated/nrf52840/nrf52840_hal/lib.rs");
}

fn example_gpio_pin_output() {
    use nrf52840_hal::gpio::p0;

    const LED_PIN: u8 = 15;

    let pin = unsafe { p0::pin(LED_PIN) };

    let configured = pin
        .configure()
        .dir(p0::Dir::Output)
        .pull(p0::Pull::Disabled)
        .drive(p0::Drive::S0s1)
        .sense(p0::Sense::Disabled)
        .apply();

    let out = match configured {
        p0::PinConfigured::Output(out) => out,
        _ => return,
    };

    out.set_high();
    out.set_low();
}

fn example_timer_config() {
    use nrf52840_hal::timer::timer0;

    let prescaler = 4u32;
    let ticks_per_us = 16u32 >> prescaler;
    let cc0 = 1u32 * ticks_per_us;

    let configured = timer0::timer()
        .configure()
        .mode(timer0::Mode::Timer)
        .bitmode(timer0::Bitmode::_32bit)
        .prescaler(prescaler)
        .cc(0, cc0)
        .clear_on_compare(0, true)
        .enable_interrupt_on_compare(0)
        .apply();

    match configured {
        timer0::TimerConfigured::Timer(t) => {
            let _ = t.clear_event_compare(0).clear().start();
        }
        _ => loop {},
    }
}

fn main() {
    example_gpio_pin_output();
    example_timer_config();
}
