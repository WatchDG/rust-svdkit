use crate::Result;
use crate::hal::ir::*;
use crate::hal::common::*;

pub fn emit_usb_file(devices: &[UsbIr], _pac_crate: &str, include_cdc_acm: bool) -> Result<String> {
    let mut s = String::new();
    s.push_str("#[allow(dead_code)]\n");
    s.push_str("#[allow(non_snake_case)]\n\n");
    s.push_str("use super::pac;\n\n");
    for u in devices {
        s.push_str(&emit_usb(u, _pac_crate, include_cdc_acm));
        s.push('\n');
    }
    Ok(s)
}

pub fn emit_usb(u: &UsbIr, _pac_crate: &str, include_cdc_acm: bool) -> String {
    let mut s = String::new();
    let type_name = sanitize_type_name(&u.hal_mod);
    s.push_str(&format!("pub type {}Register = crate::pac::peripherals::{}::{};\n\n", type_name, u.periph_mod, type_name));

    if include_cdc_acm {
        s.push_str(&cdc_acm_types());
    }

    s.push_str("use core::marker::PhantomData;\n\n");
    s.push_str("pub trait UsbState {}\n");
    s.push_str("pub struct Unconfigured;\n");
    s.push_str("pub struct Enabled;\n\n");
    s.push_str("impl UsbState for Unconfigured {}\n");
    s.push_str("impl UsbState for Enabled {}\n\n");

    s.push_str(&format!("pub struct Usb<'a, S: UsbState> {{ usb: &'a {}Register, _state: PhantomData<S> }}\n\n", type_name));

    s.push_str("impl<'a, S: UsbState> Usb<'a, S> {\n");
    s.push_str("    #[inline(always)]\n    pub fn is_reset(&self) -> bool { self.usb.");
    s.push_str(&u.field_events_usbreset);
    s.push_str(".read() != 0 }\n\n");
    s.push_str("    #[inline(always)]\n    pub fn is_event(&self) -> bool { self.usb.");
    s.push_str(&u.field_events_usbevent);
    s.push_str(".read() != 0 }\n");
    s.push_str("}\n\n");

    s.push_str(&format!("impl<'a> Usb<'a, Unconfigured> {{\n"));
    s.push_str("    #[inline(always)]\n");
    s.push_str(&format!("    pub unsafe fn steal() -> Usb<'static, Unconfigured> {{ Usb {{ usb: &*crate::pac::peripherals::{}::PTR, _state: PhantomData }} }}\n\n", u.periph_mod));
    s.push_str(&format!("    pub fn usb() -> Usb<'static, Unconfigured> {{ Usb {{ usb: unsafe {{ &*crate::pac::peripherals::{}::PTR }}, _state: PhantomData }} }}\n\n", u.periph_mod));

    s.push_str("    pub fn init(self) -> Usb<'a, Enabled> {\n");
    s.push_str(&format!("        self.usb.{}.write(1);\n", u.field_usb_pullup));
    s.push_str(&format!("        self.usb.{}.write(1);\n", u.field_enable));
    s.push_str("        Usb { usb: self.usb, _state: PhantomData }\n    }\n");
    s.push_str("}\n\n");

    s.push_str("impl<'a> Usb<'a, Enabled> {\n");
    s.push_str("    pub fn disable(self) -> Usb<'a, Unconfigured> {\n");
    s.push_str(&format!("        self.usb.{}.write(0);\n", u.field_enable));
    s.push_str("        Usb { usb: self.usb, _state: PhantomData }\n    }\n");

    s.push_str("    #[inline(always)]\n    pub fn ep0_write(&self, buf: &[u8]) {\n        for (i, &b) in buf.iter().enumerate() {\n");
    s.push_str("            unsafe {\n                let ptr = (crate::pac::peripherals::");
    s.push_str(&u.periph_mod);
    s.push_str("::BASE + 0x800) as *mut u32;\n                core::ptr::write_volatile(ptr.add(i), b as u32);\n            }\n        }\n");
    s.push_str(&format!("        self.usb.{}.write(1);\n", u.field_tasks_startein));
    s.push_str(&format!("        while self.usb.{}.read() == 0 {{}}\n        self.usb.{}.write(0);\n    }}\n\n", u.field_events_endepin, u.field_events_endepin));

    s.push_str("    #[inline(always)]\n    pub fn ep0_read(&self, buf: &mut [u8]) -> usize {\n");
    s.push_str(&format!("        self.usb.{}.write(1);\n", u.field_tasks_staroutep));
    s.push_str(&format!("        while self.usb.{}.read() == 0 {{}}\n        self.usb.{}.write(0);\n", u.field_events_endepout, u.field_events_endepout));
    s.push_str("        let mut count = 0;\n        loop {\n            let v = self.usb.");
    s.push_str(&u.field_events_ep0datadone);
    s.push_str(".read();\n            if v != 0 {\n                self.usb.");
    s.push_str(&u.field_events_ep0datadone);
    s.push_str(".write(0);\n                break;\n            }\n            let d = unsafe { core::ptr::read_volatile((crate::pac::peripherals::");
    s.push_str(&u.periph_mod);
    s.push_str("::BASE + 0x800) as *const u32) };\n            if count < buf.len() { buf[count] = d as u8; }\n            count += 1;\n        }\n        count\n    }\n");

    s.push_str(&format!("    pub fn ep_write(&self, ep: usize, buf: &[u8]) {{\n        self.usb.{}.write(1u32 << ep);\n        for (i, &b) in buf.iter().enumerate() {{\n", u.field_epinen));
    if let Some(ref epin) = u.field_epin {
        s.push_str(&format!("            let ptr = unsafe {{ &*((&raw const self.usb.{epin}[ep]) as *const u32) }};\n"));
    } else {
        s.push_str("            let ptr = 0x800 as *const u32;\n");
    }
    s.push_str("            unsafe { core::ptr::write_volatile(ptr as *mut u32, b as u32); }\n        }\n");
    s.push_str(&format!("        self.usb.{}.write(1u32 << ep);\n", u.field_tasks_startepin));
    s.push_str(&format!("        while self.usb.{}.read() & (1u32 << ep) == 0 {{}}\n", u.field_events_endepin_array));
    s.push_str(&format!("        self.usb.{}.write(1u32 << ep);\n", u.field_events_endepin_array));
    s.push_str("    }\n");
    s.push_str("}\n");

    s
}

fn cdc_acm_types() -> String {
    r#"pub struct LineCoding {
    pub dwDTERate: u32,
    pub bCharFormat: u8,
    pub bParityType: u8,
    pub bDataBits: u8,
}

pub struct ControlLineState {
    pub dtr: bool,
    pub rts: bool,
}

pub enum SerialStateBit {
    RxCarrier = 0x40,
    TxCarrier = 0x20,
    Break = 0x10,
    RingSignal = 0x08,
    FramingError = 0x04,
    ParityError = 0x02,
    Overrun = 0x01,
}

pub struct SerialState {
    pub bits: u8,
}

impl SerialState {
    pub fn empty() -> Self { Self { bits: 0 } }
    pub fn has(&self, bit: SerialStateBit) -> bool { self.bits & (bit as u8) != 0 }
    pub fn set(&mut self, bit: SerialStateBit) { self.bits |= bit as u8; }
}

pub struct CdcAcmConfig {
    pub line_coding: Option<LineCoding>,
    pub control_line_state: Option<ControlLineState>,
}

pub struct CdcAcmConfigurator<'a> {
    pub line_coding: Option<&'a LineCoding>,
    pub control_line_state: Option<&'a ControlLineState>,
}

pub enum ClassRequestResult {
    Handled,
    NotHandled,
    Stall,
}

"#.to_string()
}
