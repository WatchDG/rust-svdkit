#![allow(unsafe_code)]
#![allow(unsafe_op_in_unsafe_fn)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_imports)]

mod nrf52840_pac {
    include!("../generated/nrf52840/nrf52840_pac.rs");
}

mod nrf52840_hal {
    include!("../generated/nrf52840/nrf52840_hal.rs");
}

fn main() {
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
