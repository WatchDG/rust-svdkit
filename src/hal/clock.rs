use super::gpio;
use crate::{Result, svd};

#[derive(Debug, Clone)]
pub struct ClockInfo {
    periph_name: String,
    periph_mod: String,
    hal_mod: String,

    field_tasks_hfclk_start: String,
    field_tasks_hfclk_stop: String,
    field_tasks_lfclk_start: String,
    field_tasks_lfclk_stop: String,
    field_tasks_cal: String,
    field_tasks_ctstart: String,
    field_tasks_ctstop: String,

    field_events_hfclk_started: String,
    field_events_lfclk_started: String,
    field_events_done: String,
    field_events_ctto: String,
    field_events_ctstarted: String,

    field_hfclkstat: String,
    field_lfclkstat: String,
    field_hfclksrc: Option<svd::Field>,
    field_lfclksrc: Option<svd::Field>,
}

impl ClockInfo {
    pub fn render(&self) -> Result<String> {
        let mut s = String::new();

        let type_name = sanitize_type_name(self.hal_mod.as_str());
        s.push_str(&format!(
            "pub type {}Register = crate::pac::peripherals::{}::{};\n\n",
            type_name, self.periph_mod, type_name,
        ));

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

        s.push_str("#[repr(u8)]\n");
        s.push_str("#[derive(Copy, Clone, Debug, PartialEq, Eq)]\n");
        s.push_str("pub enum HfClockSource {\n");
        s.push_str("    Xtal = 0,\n");
        s.push_str("}\n\n");

        s.push_str("#[repr(u8)]\n");
        s.push_str("#[derive(Copy, Clone, Debug, PartialEq, Eq)]\n");
        s.push_str("pub enum LfClockSource {\n");
        s.push_str("    Rc = 0,\n");
        s.push_str("    Xtal = 1,\n");
        s.push_str("    Synth = 2,\n");
        s.push_str("}\n\n");

        s.push_str(&format!("pub struct Clock<'a, S: ClockState> {{\n"));
        s.push_str(&format!("    c: &'a {}Register,\n", type_name));
        s.push_str("    _state: PhantomData<S>,\n");
        s.push_str("}\n\n");

        s.push_str("impl<'a, S: ClockState> Clock<'a, S> {\n");
        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn is_hfclk_running(&self) -> bool {\n");
        s.push_str(&format!(
            "        (self.c.{}.read() >> 16) & 1 == 1\n",
            self.field_hfclkstat
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn is_lfclk_running(&self) -> bool {\n");
        s.push_str(&format!(
            "        (self.c.{}.read() >> 16) & 1 == 1\n",
            self.field_lfclkstat
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn is_hfclk_started(&self) -> bool {\n");
        s.push_str(&format!(
            "        self.c.{}.read() != 0\n",
            self.field_events_hfclk_started
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn is_lfclk_started(&self) -> bool {\n");
        s.push_str(&format!(
            "        self.c.{}.read() != 0\n",
            self.field_events_lfclk_started
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn is_cal_done(&self) -> bool {\n");
        s.push_str(&format!(
            "        self.c.{}.read() != 0\n",
            self.field_events_done
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn is_cal_timeout(&self) -> bool {\n");
        s.push_str(&format!(
            "        self.c.{}.read() != 0\n",
            self.field_events_ctto
        ));
        s.push_str("    }\n");
        s.push_str("}\n\n");

        s.push_str(&format!("impl<'a> Clock<'a, Unconfigured> {{\n"));
        s.push_str("    #[inline(always)]\n");
        s.push_str(&format!(
            "    pub unsafe fn steal() -> Clock<'static, Unconfigured> {{\n"
        ));
        s.push_str(&format!(
            "        Clock {{ c: &*crate::pac::peripherals::{}::PTR, _state: PhantomData }}\n",
            self.periph_mod
        ));
        s.push_str("    }\n\n");

        s.push_str(&format!(
            "    pub fn clock() -> Clock<'static, Unconfigured> {{\n"
        ));
        s.push_str(&format!(
            "        Clock {{ c: unsafe {{ &*crate::pac::peripherals::{}::PTR }}, _state: PhantomData }}\n",
            self.periph_mod
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn start_hfclk(self) -> Clock<'a, HfRunning> {\n");
        s.push_str(&format!(
            "        self.c.{}.write(1);\n",
            self.field_tasks_hfclk_start
        ));
        s.push_str(&format!(
            "        Clock {{ c: self.c, _state: PhantomData }}\n"
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn stop_hfclk(self) -> Clock<'a, Unconfigured> {\n");
        s.push_str(&format!(
            "        self.c.{}.write(1);\n",
            self.field_tasks_hfclk_stop
        ));
        s.push_str(&format!(
            "        Clock {{ c: self.c, _state: PhantomData }}\n"
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn start_lfclk(self) -> Clock<'a, LfRunning> {\n");
        s.push_str(&format!(
            "        self.c.{}.write(1);\n",
            self.field_tasks_lfclk_start
        ));
        s.push_str(&format!(
            "        Clock {{ c: self.c, _state: PhantomData }}\n"
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn stop_lfclk(self) -> Clock<'a, Unconfigured> {\n");
        s.push_str(&format!(
            "        self.c.{}.write(1);\n",
            self.field_tasks_lfclk_stop
        ));
        s.push_str(&format!(
            "        Clock {{ c: self.c, _state: PhantomData }}\n"
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn start_calibration(self) -> Clock<'a, Calibrating> {\n");
        s.push_str(&format!(
            "        self.c.{}.write(1);\n",
            self.field_tasks_cal
        ));
        s.push_str(&format!(
            "        Clock {{ c: self.c, _state: PhantomData }}\n"
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn start_calibration_timer(self) {\n");
        s.push_str(&format!(
            "        self.c.{}.write(1);\n",
            self.field_tasks_ctstart
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn stop_calibration_timer(self) {\n");
        s.push_str(&format!(
            "        self.c.{}.write(1);\n",
            self.field_tasks_ctstop
        ));
        s.push_str("    }\n");
        s.push_str("}\n\n");

        s.push_str(&format!("impl<'a> Clock<'a, HfRunning> {{\n"));
        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn stop(self) -> Clock<'a, Unconfigured> {\n");
        s.push_str(&format!(
            "        self.c.{}.write(1);\n",
            self.field_tasks_hfclk_stop
        ));
        s.push_str(&format!(
            "        Clock {{ c: self.c, _state: PhantomData }}\n"
        ));
        s.push_str("    }\n");
        s.push_str("}\n\n");

        s.push_str(&format!("impl<'a> Clock<'a, LfRunning> {{\n"));
        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn stop(self) -> Clock<'a, Unconfigured> {\n");
        s.push_str(&format!(
            "        self.c.{}.write(1);\n",
            self.field_tasks_lfclk_stop
        ));
        s.push_str(&format!(
            "        Clock {{ c: self.c, _state: PhantomData }}\n"
        ));
        s.push_str("    }\n");
        s.push_str("}\n\n");

        s.push_str(&format!("impl<'a> Clock<'a, Calibrating> {{\n"));
        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn stop(self) -> Clock<'a, Unconfigured> {\n");
        s.push_str(&format!(
            "        self.c.{}.write(1);\n",
            self.field_tasks_ctstop
        ));
        s.push_str(&format!(
            "        Clock {{ c: self.c, _state: PhantomData }}\n"
        ));
        s.push_str("    }\n");
        s.push_str("}\n");

        Ok(s)
    }
}

pub fn collect_clocks(device: &svd::Device) -> Vec<ClockInfo> {
    let mut out = Vec::new();
    for p in &device.peripherals {
        if !is_clock_like(&p.name) {
            continue;
        }
        let items = gpio::peripheral_register_items(device, p);
        if items.is_empty() {
            continue;
        }

        let Some((hf_start_name, _)) = gpio::find_register(items, "TASKS_HFCLKSTART") else {
            continue;
        };
        let Some((hf_stop_name, _)) = gpio::find_register(items, "TASKS_HFCLKSTOP") else {
            continue;
        };
        let Some((lf_start_name, _)) = gpio::find_register(items, "TASKS_LFCLKSTART") else {
            continue;
        };
        let Some((lf_stop_name, _)) = gpio::find_register(items, "TASKS_LFCLKSTOP") else {
            continue;
        };
        let Some((cal_name, _)) = gpio::find_register(items, "TASKS_CAL") else {
            continue;
        };
        let Some((ctstart_name, _)) = gpio::find_register(items, "TASKS_CTSTART") else {
            continue;
        };
        let Some((ctstop_name, _)) = gpio::find_register(items, "TASKS_CTSTOP") else {
            continue;
        };
        let Some((hf_started_name, _)) = gpio::find_register(items, "EVENTS_HFCLKSTARTED") else {
            continue;
        };
        let Some((lf_started_name, _)) = gpio::find_register(items, "EVENTS_LFCLKSTARTED") else {
            continue;
        };
        let Some((done_name, _)) = gpio::find_register(items, "EVENTS_DONE") else {
            continue;
        };
        let Some((ctto_name, _)) = gpio::find_register(items, "EVENTS_CTTO") else {
            continue;
        };
        let Some((ctstarted_name, _)) = gpio::find_register(items, "EVENTS_CTSTARTED") else {
            continue;
        };
        let Some((hfstat_name, hfstat_reg)) = gpio::find_register(items, "HFCLKSTAT") else {
            continue;
        };
        let Some((lfstat_name, lfstat_reg)) = gpio::find_register(items, "LFCLKSTAT") else {
            continue;
        };

        let hfclksrc_field = hfstat_reg
            .field
            .iter()
            .find(|f| {
                f.name.to_ascii_uppercase() == "SRC"
                    || f.name.to_ascii_uppercase().ends_with("_SRC")
            })
            .cloned();

        let lfclksrc_field = lfstat_reg
            .field
            .iter()
            .find(|f| {
                f.name.to_ascii_uppercase() == "SRC"
                    || f.name.to_ascii_uppercase().ends_with("_SRC")
            })
            .cloned();

        out.push(ClockInfo {
            periph_name: p.name.clone(),
            periph_mod: gpio::sanitize_module_name(&p.name),
            hal_mod: gpio::sanitize_field_name(&p.name),

            field_tasks_hfclk_start: gpio::sanitize_field_name(&hf_start_name),
            field_tasks_hfclk_stop: gpio::sanitize_field_name(&hf_stop_name),
            field_tasks_lfclk_start: gpio::sanitize_field_name(&lf_start_name),
            field_tasks_lfclk_stop: gpio::sanitize_field_name(&lf_stop_name),
            field_tasks_cal: gpio::sanitize_field_name(&cal_name),
            field_tasks_ctstart: gpio::sanitize_field_name(&ctstart_name),
            field_tasks_ctstop: gpio::sanitize_field_name(&ctstop_name),

            field_events_hfclk_started: gpio::sanitize_field_name(&hf_started_name),
            field_events_lfclk_started: gpio::sanitize_field_name(&lf_started_name),
            field_events_done: gpio::sanitize_field_name(&done_name),
            field_events_ctto: gpio::sanitize_field_name(&ctto_name),
            field_events_ctstarted: gpio::sanitize_field_name(&ctstarted_name),

            field_hfclkstat: gpio::sanitize_field_name(&hfstat_name),
            field_lfclkstat: gpio::sanitize_field_name(&lfstat_name),
            field_hfclksrc: hfclksrc_field,
            field_lfclksrc: lfclksrc_field,
        });
    }
    out
}

fn is_clock_like(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    upper.contains("CLOCK")
}

fn sanitize_type_name(s: &str) -> String {
    gpio::sanitize_type_name(s)
}
