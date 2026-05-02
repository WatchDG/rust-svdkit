use crate::Result;
use crate::hal::ir::*;

pub fn emit_clock_file(clocks: &[ClockIr], _pac_crate: &str) -> Result<String> {
    let mut s = String::new();
    s.push_str("#[allow(dead_code)]\n");
    s.push_str("#[allow(non_snake_case)]\n\n");
    s.push_str("use super::pac;\n\n");
    for c in clocks {
        s.push_str(&emit_clock(c, _pac_crate));
        s.push('\n');
    }
    Ok(s)
}

pub fn emit_clock(c: &ClockIr, _pac_crate: &str) -> String {
    let mut s = String::new();
    let type_name = crate::hal::common::sanitize_type_name(&c.hal_mod);

    s.push_str(&format!("pub type {}Register = crate::pac::peripherals::{}::{};\n\n", type_name, c.periph_mod, type_name));
    s.push_str("use core::marker::PhantomData;\n\n");
    s.push_str("pub trait ClockState {}\n");
    s.push_str("pub struct Unconfigured;\n");
    s.push_str("pub struct HfRunning;\n");
    s.push_str("pub struct LfRunning;\n");
    s.push_str("pub struct Calibrating;\n\n");
    s.push_str("impl ClockState for Unconfigured {}\n");
    s.push_str("impl ClockState for HfRunning {}\n");
    s.push_str("impl ClockState for LfRunning {}\n");
    s.push_str("impl ClockState for Calibrating {}\n\n");
    s.push_str("#[repr(u8)]\n#[derive(Copy, Clone, Debug, PartialEq, Eq)]\npub enum HfClockSource { Xtal = 0 }\n\n");
    s.push_str("#[repr(u8)]\n#[derive(Copy, Clone, Debug, PartialEq, Eq)]\npub enum LfClockSource { Rc = 0, Xtal = 1, Synth = 2 }\n\n");
    s.push_str(&format!("pub struct Clock<'a, S: ClockState> {{ c: &'a {}Register, _state: PhantomData<S> }}\n\n", type_name));

    s.push_str("impl<'a, S: ClockState> Clock<'a, S> {\n");
    s.push_str("    #[inline(always)]\n    pub fn is_hfclk_running(&self) -> bool { (self.c.");
    s.push_str(&c.field_hfclkstat);
    s.push_str(".read() >> 16) & 1 == 1 }\n\n");
    s.push_str("    #[inline(always)]\n    pub fn is_lfclk_running(&self) -> bool { (self.c.");
    s.push_str(&c.field_lfclkstat);
    s.push_str(".read() >> 16) & 1 == 1 }\n\n");
    s.push_str("    #[inline(always)]\n    pub fn is_hfclk_started(&self) -> bool { self.c.");
    s.push_str(&c.field_events_hfclk_started);
    s.push_str(".read() != 0 }\n\n");
    s.push_str("    #[inline(always)]\n    pub fn is_lfclk_started(&self) -> bool { self.c.");
    s.push_str(&c.field_events_lfclk_started);
    s.push_str(".read() != 0 }\n\n");
    s.push_str("    #[inline(always)]\n    pub fn is_cal_done(&self) -> bool { self.c.");
    s.push_str(&c.field_events_done);
    s.push_str(".read() != 0 }\n\n");
    s.push_str("    #[inline(always)]\n    pub fn is_cal_timeout(&self) -> bool { self.c.");
    s.push_str(&c.field_events_ctto);
    s.push_str(".read() != 0 }\n");
    s.push_str("}\n\n");

    s.push_str(&format!("impl<'a> Clock<'a, Unconfigured> {{\n"));
    s.push_str("    #[inline(always)]\n");
    s.push_str(&format!("    pub unsafe fn steal() -> Clock<'static, Unconfigured> {{ Clock {{ c: &*crate::pac::peripherals::{}::PTR, _state: PhantomData }} }}\n\n", c.periph_mod));
    s.push_str(&format!("    pub fn clock() -> Clock<'static, Unconfigured> {{ Clock {{ c: unsafe {{ &*crate::pac::peripherals::{}::PTR }}, _state: PhantomData }} }}\n\n", c.periph_mod));

    s.push_str("    #[inline(always)]\n    pub fn start_hfclk(self) -> Clock<'a, HfRunning> { self.c.");
    s.push_str(&c.field_tasks_hfclk_start);
    s.push_str(".write(1); Clock { c: self.c, _state: PhantomData } }\n\n");

    s.push_str("    #[inline(always)]\n    pub fn stop_hfclk(self) -> Clock<'a, Unconfigured> { self.c.");
    s.push_str(&c.field_tasks_hfclk_stop);
    s.push_str(".write(1); Clock { c: self.c, _state: PhantomData } }\n\n");

    s.push_str("    #[inline(always)]\n    pub fn start_lfclk(self) -> Clock<'a, LfRunning> { self.c.");
    s.push_str(&c.field_tasks_lfclk_start);
    s.push_str(".write(1); Clock { c: self.c, _state: PhantomData } }\n\n");

    s.push_str("    #[inline(always)]\n    pub fn stop_lfclk(self) -> Clock<'a, Unconfigured> { self.c.");
    s.push_str(&c.field_tasks_lfclk_stop);
    s.push_str(".write(1); Clock { c: self.c, _state: PhantomData } }\n\n");

    s.push_str("    #[inline(always)]\n    pub fn start_calibration(self) -> Clock<'a, Calibrating> { self.c.");
    s.push_str(&c.field_tasks_cal);
    s.push_str(".write(1); Clock { c: self.c, _state: PhantomData } }\n\n");

    s.push_str("    #[inline(always)]\n    pub fn start_calibration_timer(self) { self.c.");
    s.push_str(&c.field_tasks_ctstart);
    s.push_str(".write(1); }\n\n");

    s.push_str("    #[inline(always)]\n    pub fn stop_calibration_timer(self) { self.c.");
    s.push_str(&c.field_tasks_ctstop);
    s.push_str(".write(1); }\n");
    s.push_str("}\n\n");

    s.push_str("impl<'a> Clock<'a, HfRunning> {\n    #[inline(always)]\n    pub fn stop(self) -> Clock<'a, Unconfigured> { self.c.");
    s.push_str(&c.field_tasks_hfclk_stop);
    s.push_str(".write(1); Clock { c: self.c, _state: PhantomData } }\n}\n\n");

    s.push_str("impl<'a> Clock<'a, LfRunning> {\n    #[inline(always)]\n    pub fn stop(self) -> Clock<'a, Unconfigured> { self.c.");
    s.push_str(&c.field_tasks_lfclk_stop);
    s.push_str(".write(1); Clock { c: self.c, _state: PhantomData } }\n}\n\n");

    s.push_str("impl<'a> Clock<'a, Calibrating> {\n    #[inline(always)]\n    pub fn stop(self) -> Clock<'a, Unconfigured> { self.c.");
    s.push_str(&c.field_tasks_ctstop);
    s.push_str(".write(1); Clock { c: self.c, _state: PhantomData } }\n}\n");

    s
}
