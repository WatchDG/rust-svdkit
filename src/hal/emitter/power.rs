use crate::Result;
use crate::hal::ir::*;
use crate::hal::common::*;

pub fn emit_power_file(power_devices: &[PowerIr], _pac_crate: &str) -> Result<String> {
    let mut s = String::new();
    s.push_str("#[allow(dead_code)]\n");
    s.push_str("#[allow(non_snake_case)]\n\n");
    s.push_str("use super::pac;\n\n");
    for p in power_devices {
        s.push_str(&emit_power(p, _pac_crate));
        s.push('\n');
    }
    Ok(s)
}

pub fn emit_power(p: &PowerIr, _pac_crate: &str) -> String {
    let mut s = String::new();
    let type_name = sanitize_type_name(&p.hal_mod);
    s.push_str(&format!("pub type {}Register = crate::pac::peripherals::{}::{};\n\n", type_name, p.periph_mod, type_name));
    s.push_str("use core::marker::PhantomData;\n\n");
    s.push_str("pub trait PowerState {}\n");
    s.push_str("pub struct Unconfigured;\n");
    s.push_str("pub struct ConstLat;\n");
    s.push_str("pub struct LowPower;\n\n");
    s.push_str("impl PowerState for Unconfigured {}\n");
    s.push_str("impl PowerState for ConstLat {}\n");
    s.push_str("impl PowerState for LowPower {}\n\n");
    s.push_str("#[repr(u8)]\n#[derive(Copy, Clone, Debug, PartialEq, Eq)]\npub enum PowerMode { ConstantLatency = 0, LowPower = 1 }\n\n");
    s.push_str(&format!("pub struct Power<'a, S: PowerState> {{ power: &'a {}Register, _state: PhantomData<S> }}\n\n", type_name));

    s.push_str("impl<'a, S: PowerState> Power<'a, S> {\n");
    s.push_str("    #[inline(always)]\n    pub fn is_power_failure_warning(&self) -> bool { self.power.");
    s.push_str(&p.field_events_pofwarn);
    s.push_str(".read() != 0 }\n\n");
    s.push_str("    #[inline(always)]\n    pub fn is_sleep_enter(&self) -> bool { self.power.");
    s.push_str(&p.field_events_sleepenter);
    s.push_str(".read() != 0 }\n\n");
    s.push_str("    #[inline(always)]\n    pub fn is_sleep_exit(&self) -> bool { self.power.");
    s.push_str(&p.field_events_sleepexit);
    s.push_str(".read() != 0 }\n\n");
    s.push_str("    #[inline(always)]\n    pub fn is_usb_detected(&self) -> bool { self.power.");
    s.push_str(&p.field_events_usbdetected);
    s.push_str(".read() != 0 }\n\n");
    s.push_str("    #[inline(always)]\n    pub fn is_usb_removed(&self) -> bool { self.power.");
    s.push_str(&p.field_events_usbremoved);
    s.push_str(".read() != 0 }\n\n");
    s.push_str("    #[inline(always)]\n    pub fn is_usb_power_ready(&self) -> bool { self.power.");
    s.push_str(&p.field_events_usbpwrrdy);
    s.push_str(".read() != 0 }\n");
    s.push_str("}\n\n");

    s.push_str("impl<'a> Power<'a, Unconfigured> {\n");
    s.push_str("    #[inline(always)]\n");
    s.push_str(&format!("    pub unsafe fn steal() -> Power<'static, Unconfigured> {{ Power {{ power: &*crate::pac::peripherals::{}::PTR, _state: PhantomData }} }}\n\n", p.periph_mod));
    s.push_str(&format!("    pub fn power() -> Power<'static, Unconfigured> {{ Power {{ power: unsafe {{ &*crate::pac::peripherals::{}::PTR }}, _state: PhantomData }} }}\n\n", p.periph_mod));

    s.push_str("    #[inline(always)]\n    pub fn enable_constant_latency(self) -> Power<'a, ConstLat> { self.power.");
    s.push_str(&p.field_tasks_constlat);
    s.push_str(".write(1); Power { power: self.power, _state: PhantomData } }\n\n");

    s.push_str("    #[inline(always)]\n    pub fn enable_low_power(self) -> Power<'a, LowPower> { self.power.");
    s.push_str(&p.field_tasks_lowpwr);
    s.push_str(".write(1); Power { power: self.power, _state: PhantomData } }\n\n");

    s.push_str("    #[inline(always)]\n    pub fn enable_interrupts(&mut self) { self.power.");
    s.push_str(&p.field_intenset);
    s.push_str(".write(0x3F); }\n\n");

    s.push_str("    #[inline(always)]\n    pub fn disable_interrupts(&mut self) { self.power.");
    s.push_str(&p.field_intenclr);
    s.push_str(".write(0x3F); }\n");
    s.push_str("}\n\n");

    s.push_str("impl<'a> Power<'a, ConstLat> {\n    #[inline(always)]\n    pub fn to_unconfigured(self) -> Power<'a, Unconfigured> { Power { power: self.power, _state: PhantomData } }\n    #[inline(always)]\n    pub fn enable_low_power(self) -> Power<'a, LowPower> { self.power.");
    s.push_str(&p.field_tasks_lowpwr);
    s.push_str(".write(1); Power { power: self.power, _state: PhantomData } }\n}\n\n");

    s.push_str("impl<'a> Power<'a, LowPower> {\n    #[inline(always)]\n    pub fn to_unconfigured(self) -> Power<'a, Unconfigured> { Power { power: self.power, _state: PhantomData } }\n    #[inline(always)]\n    pub fn enable_constant_latency(self) -> Power<'a, ConstLat> { self.power.");
    s.push_str(&p.field_tasks_constlat);
    s.push_str(".write(1); Power { power: self.power, _state: PhantomData } }\n}\n");

    s
}
