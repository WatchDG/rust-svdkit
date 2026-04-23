use super::gpio;
use crate::{Result, svd};

#[derive(Debug, Clone)]
pub struct TimerInfo {
    periph_name: String,
    periph_mod: String,
    hal_mod: String,

    field_mode: String,
    field_bitmode: String,
    field_prescaler: String,
    field_cc: String,
    field_shorts: String,
    field_intenset: String,
    field_events_compare: String,
    field_tasks_clear: String,
    field_tasks_start: String,

    mode_reg_path: String,
    bitmode_reg_path: String,
    mode_field: svd::Field,
    bitmode_field: svd::Field,
    shorts_fields: Vec<(usize, svd::Field)>,
    intenset_fields: Vec<(usize, svd::Field)>,
}

impl TimerInfo {
    pub fn render(&self) -> Result<String> {
        let mut s = String::new();
        let timer_ty = sanitize_type_name(&self.periph_mod);

        s.push_str(&format!("    pub mod {} {{\n", self.hal_mod));
        s.push_str("        use super::pac;\n\n");
        s.push_str(&format!(
            "        pub type {timer_ty} = pac::{}::RegisterBlock;\n\n",
            self.periph_mod,
        ));

        s.push_str("        use core::marker::PhantomData;\n\n");

        s.push_str("        pub trait TimerModeTrait {}\n");
        s.push_str("        pub struct Unconfigured;\n");
        s.push_str("        impl TimerModeTrait for Unconfigured {}\n\n");

        s.push_str("        #[inline(always)]\n");
        s.push_str(&format!(
            "        pub unsafe fn steal() -> &'static {timer_ty} {{\n"
        ));
        s.push_str(&format!("            &*pac::{}::PTR\n", self.periph_mod));
        s.push_str("        }\n\n");

        s.push_str(&format!(
            "        pub fn timer() -> Timer<'static, Unconfigured> {{\n"
        ));
        s.push_str(&format!(
            "            Timer {{ t: unsafe {{ &*pac::{}::PTR }}, _mode: PhantomData }}\n",
            self.periph_mod
        ));
        s.push_str("        }\n\n");

        let (mode_lsb, mode_width) = field_lsb_width(&self.mode_field);
        let mode_mask: u32 = if mode_width >= 32 {
            u32::MAX
        } else {
            ((1u64 << mode_width) - 1) as u32
        };
        let mode_alias = sanitize_type_name("MODE");
        let pac_mode_ty =
            pac_enum_type_name_for_field(&self.periph_name, &self.mode_reg_path, &self.mode_field);
        let mode_ty = if let Some(ty) = &pac_mode_ty {
            s.push_str(&indent_block(
                &format!(
                    "pub use super::super::pac::{}::enums::{ty} as {mode_alias};\n",
                    self.periph_mod
                ),
                8,
            ));
            s.push('\n');
            mode_alias.clone()
        } else if let Some(src) = render_field_enum("MODE", &self.mode_field) {
            s.push_str(&indent_block(&src, 8));
            s.push('\n');
            mode_alias.clone()
        } else {
            "u32".to_string()
        };
        let mode_arg = if pac_mode_ty.is_some() || mode_ty == mode_alias {
            "v as u32"
        } else {
            "v"
        };

        let (bitmode_lsb, bitmode_width) = field_lsb_width(&self.bitmode_field);
        let bitmode_mask: u32 = if bitmode_width >= 32 {
            u32::MAX
        } else {
            ((1u64 << bitmode_width) - 1) as u32
        };
        let bitmode_alias = sanitize_type_name("BITMODE");
        let pac_bitmode_ty = pac_enum_type_name_for_field(
            &self.periph_name,
            &self.bitmode_reg_path,
            &self.bitmode_field,
        );
        let bitmode_ty = if let Some(ty) = &pac_bitmode_ty {
            s.push_str(&indent_block(
                &format!(
                    "pub use super::super::pac::{}::enums::{ty} as {bitmode_alias};\n",
                    self.periph_mod
                ),
                8,
            ));
            s.push('\n');
            bitmode_alias.clone()
        } else if let Some(src) = render_field_enum("BITMODE", &self.bitmode_field) {
            s.push_str(&indent_block(&src, 8));
            s.push('\n');
            bitmode_alias.clone()
        } else {
            "u32".to_string()
        };
        let bitmode_arg = if pac_bitmode_ty.is_some() || bitmode_ty == bitmode_alias {
            "v as u32"
        } else {
            "v"
        };

        s.push_str(&format!(
            "        pub struct Timer<'a, M: TimerModeTrait> {{\n"
        ));
        s.push_str(&format!("            t: &'a {timer_ty},\n"));
        s.push_str("            _mode: PhantomData<M>,\n");
        s.push_str("        }\n\n");

        s.push_str(&format!("        impl<'a> Timer<'a, Unconfigured> {{\n"));
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn configure(self) -> TimerConfigurator<'a> {\n");
        s.push_str("                TimerConfigurator {\n");
        s.push_str("                    t: self.t,\n");
        s.push_str(&format!("                    mode_val: None,\n"));
        s.push_str(&format!("                    bitmode_val: None,\n"));
        s.push_str("                    prescaler_val: None,\n");
        s.push_str("                    cc_vals: [None, None, None, None, None, None],\n");
        s.push_str("                    clear_on_compare_mask: None,\n");
        s.push_str("                    interrupt_mask: None,\n");
        s.push_str("                }\n");
        s.push_str("            }\n");
        s.push_str("        }\n\n");

        s.push_str("        pub struct TimerConfigurator<'a> {\n");
        s.push_str(&format!("            t: &'a {timer_ty},\n"));
        s.push_str("            mode_val: Option<u32>,\n");
        s.push_str("            bitmode_val: Option<u32>,\n");
        s.push_str("            prescaler_val: Option<u32>,\n");
        s.push_str("            cc_vals: [Option<u32>; 6],\n");
        s.push_str("            clear_on_compare_mask: Option<u32>,\n");
        s.push_str("            interrupt_mask: Option<u32>,\n");
        s.push_str("        }\n\n");

        s.push_str("        impl<'a> TimerConfigurator<'a> {\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str(&format!(
            "            pub fn mode(self, v: {mode_ty}) -> Self {{\n"
        ));
        s.push_str(&format!(
            "                TimerConfigurator {{\n                    t: self.t,\n                    mode_val: Some({mode_arg}),\n                    bitmode_val: self.bitmode_val,\n                    prescaler_val: self.prescaler_val,\n                    cc_vals: self.cc_vals,\n                    clear_on_compare_mask: self.clear_on_compare_mask,\n                    interrupt_mask: self.interrupt_mask,\n                }}\n"
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str(&format!(
            "            pub fn bitmode(self, v: {bitmode_ty}) -> Self {{\n"
        ));
        s.push_str(&format!(
            "                TimerConfigurator {{\n                    t: self.t,\n                    mode_val: self.mode_val,\n                    bitmode_val: Some({bitmode_arg}),\n                    prescaler_val: self.prescaler_val,\n                    cc_vals: self.cc_vals,\n                    clear_on_compare_mask: self.clear_on_compare_mask,\n                    interrupt_mask: self.interrupt_mask,\n                }}\n"
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn prescaler(self, v: u32) -> Self {\n");
        s.push_str("                TimerConfigurator {\n");
        s.push_str("                    t: self.t,\n");
        s.push_str("                    mode_val: self.mode_val,\n");
        s.push_str("                    bitmode_val: self.bitmode_val,\n");
        s.push_str("                    prescaler_val: Some(v),\n");
        s.push_str("                    cc_vals: self.cc_vals,\n");
        s.push_str("                    clear_on_compare_mask: self.clear_on_compare_mask,\n");
        s.push_str("                    interrupt_mask: self.interrupt_mask,\n");
        s.push_str("                }\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn cc(self, index: usize, v: u32) -> Self {\n");
        s.push_str("                let mut vals = self.cc_vals;\n");
        s.push_str("                if index < vals.len() {\n");
        s.push_str("                    vals[index] = Some(v);\n");
        s.push_str("                }\n");
        s.push_str("                TimerConfigurator {\n");
        s.push_str("                    t: self.t,\n");
        s.push_str("                    mode_val: self.mode_val,\n");
        s.push_str("                    bitmode_val: self.bitmode_val,\n");
        s.push_str("                    prescaler_val: self.prescaler_val,\n");
        s.push_str("                    cc_vals: vals,\n");
        s.push_str("                    clear_on_compare_mask: self.clear_on_compare_mask,\n");
        s.push_str("                    interrupt_mask: self.interrupt_mask,\n");
        s.push_str("                }\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str(
            "            pub fn clear_on_compare(self, index: usize, enable: bool) -> Self {\n",
        );
        s.push_str("                let mut mask = self.clear_on_compare_mask.unwrap_or(0);\n");
        s.push_str("                match index {\n");
        for (idx, f) in &self.shorts_fields {
            let (lsb, width) = field_lsb_width(f);
            let mask_field: u32 = if width >= 32 {
                u32::MAX
            } else {
                ((1u64 << width) - 1) as u32
            };
            s.push_str(&format!("                    {idx} => {{\n"));
            if lsb == 0 {
                s.push_str(&format!(
                    "                        mask = (mask & !0x{mask_field:X}u32) | (if enable {{ 1 }} else {{ 0 }});\n"
                ));
            } else {
                s.push_str(&format!(
                    "                        mask = (mask & !(0x{mask_field:X}u32 << {lsb})) | ((if enable {{ 1 }} else {{ 0 }}) << {lsb});\n"
                ));
            }
            s.push_str("                    }\n");
        }
        s.push_str("                    _ => {}\n");
        s.push_str("                }\n");
        s.push_str("                TimerConfigurator {\n");
        s.push_str("                    t: self.t,\n");
        s.push_str("                    mode_val: self.mode_val,\n");
        s.push_str("                    bitmode_val: self.bitmode_val,\n");
        s.push_str("                    prescaler_val: self.prescaler_val,\n");
        s.push_str("                    cc_vals: self.cc_vals,\n");
        s.push_str("                    clear_on_compare_mask: Some(mask),\n");
        s.push_str("                    interrupt_mask: self.interrupt_mask,\n");
        s.push_str("                }\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str(
            "            pub fn enable_interrupt_on_compare(self, index: usize) -> Self {\n",
        );
        s.push_str("                let mut mask = self.interrupt_mask.unwrap_or(0);\n");
        s.push_str("                match index {\n");
        for (idx, f) in &self.intenset_fields {
            let (lsb, width) = field_lsb_width(f);
            let mask_field: u32 = if width >= 32 {
                u32::MAX
            } else {
                ((1u64 << width) - 1) as u32
            };
            s.push_str(&format!("                    {idx} => {{\n"));
            s.push_str(&format!(
                "                        mask = (mask & !(0x{mask_field:X}u32 << {lsb})) | (1u32 << {lsb});\n"
            ));
            s.push_str("                    }\n");
        }
        s.push_str("                    _ => {}\n");
        s.push_str("                }\n");
        s.push_str("                TimerConfigurator {\n");
        s.push_str("                    t: self.t,\n");
        s.push_str("                    mode_val: self.mode_val,\n");
        s.push_str("                    bitmode_val: self.bitmode_val,\n");
        s.push_str("                    prescaler_val: self.prescaler_val,\n");
        s.push_str("                    cc_vals: self.cc_vals,\n");
        s.push_str("                    clear_on_compare_mask: self.clear_on_compare_mask,\n");
        s.push_str("                    interrupt_mask: Some(mask),\n");
        s.push_str("                }\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn apply(self) -> TimerConfigured<'a> {\n");
        s.push_str(&format!(
            "                if let Some(v) = self.bitmode_val {{\n"
        ));
        s.push_str(&format!(
            "                    let cur = self.t.{}.read();\n",
            self.field_bitmode
        ));
        s.push_str(&format!(
            "                    let new = (cur & !(0x{bitmode_mask:X}u32 << {bitmode_lsb})) | ((v & 0x{bitmode_mask:X}u32) << {bitmode_lsb});\n"
        ));
        s.push_str(&format!(
            "                    self.t.{}.write(new);\n",
            self.field_bitmode
        ));
        s.push_str("                }\n");

        s.push_str("                if let Some(v) = self.prescaler_val {\n");
        s.push_str(&format!(
            "                    self.t.{}.write(v);\n",
            self.field_prescaler
        ));
        s.push_str("                }\n");

        s.push_str("                for (i, v) in self.cc_vals.iter().enumerate() {\n");
        s.push_str(&format!(
            "                    if i < self.t.{}.len() {{\n",
            self.field_cc
        ));
        s.push_str("                        if let Some(val) = v {\n");
        s.push_str(&format!(
            "                            self.t.{}[i].write(*val);\n",
            self.field_cc
        ));
        s.push_str("                        }\n");
        s.push_str("                    }\n");
        s.push_str("                }\n");

        s.push_str("                if let Some(v) = self.clear_on_compare_mask {\n");
        s.push_str(&format!(
            "                    self.t.{}.write(v);\n",
            self.field_shorts
        ));
        s.push_str("                }\n");

        s.push_str("                if let Some(v) = self.interrupt_mask {\n");
        s.push_str(&format!(
            "                    self.t.{}.write(v);\n",
            self.field_intenset
        ));
        s.push_str("                }\n");

        s.push_str(&format!(
            "                if let Some(v) = self.mode_val {{\n"
        ));
        s.push_str(&format!(
            "                    let cur = self.t.{}.read();\n",
            self.field_mode
        ));
        s.push_str(&format!(
            "                    let new = (cur & !(0x{mode_mask:X}u32 << {mode_lsb})) | ((v & 0x{mode_mask:X}u32) << {mode_lsb});\n"
        ));
        s.push_str(&format!(
            "                    self.t.{}.write(new);\n",
            self.field_mode
        ));
        s.push_str("                }\n");

        s.push_str(&format!(
            "                if let Some(v) = self.mode_val {{\n"
        ));
        s.push_str("                    match v {\n");
        s.push_str(&format!(
            "                        0 => TimerConfigured::Timer(TimerModeTimer(self.t)),\n"
        ));
        s.push_str(&format!(
            "                        1 => TimerConfigured::Counter(TimerModCounter(self.t)),\n"
        ));
        s.push_str(&format!(
                    "                        2 => TimerConfigured::LowPowerCounter(TimerModeLowPowerCounter(self.t)),\n"
                ));
        s.push_str(
            "                        _ => TimerConfigured::Timer(TimerModeTimer(self.t)),\n",
        );
        s.push_str("                    }\n");
        s.push_str("                } else {\n");
        s.push_str("                    TimerConfigured::Timer(TimerModeTimer(self.t))\n");
        s.push_str("                }\n");
        s.push_str("            }\n");
        s.push_str("        }\n\n");

        s.push_str("        pub enum TimerConfigured<'a> {\n");
        s.push_str(&format!("            Timer(TimerModeTimer<'a>),\n"));
        s.push_str(&format!("            Counter(TimerModCounter<'a>),\n"));
        s.push_str(&format!(
            "            LowPowerCounter(TimerModeLowPowerCounter<'a>),\n"
        ));
        s.push_str("        }\n\n");

        s.push_str("        pub struct TimerModeTimer<'a>(pub &'a ");
        s.push_str(&timer_ty);
        s.push_str(");\n");
        s.push_str("        pub struct TimerModCounter<'a>(pub &'a ");
        s.push_str(&timer_ty);
        s.push_str(");\n");
        s.push_str("        pub struct TimerModeLowPowerCounter<'a>(pub &'a ");
        s.push_str(&timer_ty);
        s.push_str(");\n\n");

        s.push_str("        impl<'a> TimerModeTimer<'a> {\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn clear_event_compare(self, index: usize) -> Self {\n");
        s.push_str(&format!(
            "                if index < self.0.{}.len() {{\n",
            self.field_events_compare
        ));
        s.push_str(&format!(
            "                    self.0.{}[index].write(0);\n",
            self.field_events_compare
        ));
        s.push_str("                }\n");
        s.push_str("                self\n");
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn clear(self) -> Self {\n");
        s.push_str(&format!(
            "                self.0.{}.write(1);\n",
            self.field_tasks_clear
        ));
        s.push_str("                self\n");
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn start(self) -> Self {\n");
        s.push_str(&format!(
            "                self.0.{}.write(1);\n",
            self.field_tasks_start
        ));
        s.push_str("                self\n");
        s.push_str("            }\n");
        s.push_str("        }\n\n");

        s.push_str("        impl<'a> TimerModCounter<'a> {\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn clear_event_compare(self, index: usize) -> Self {\n");
        s.push_str(&format!(
            "                if index < self.0.{}.len() {{\n",
            self.field_events_compare
        ));
        s.push_str(&format!(
            "                    self.0.{}[index].write(0);\n",
            self.field_events_compare
        ));
        s.push_str("                }\n");
        s.push_str("                self\n");
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn clear(self) -> Self {\n");
        s.push_str(&format!(
            "                self.0.{}.write(1);\n",
            self.field_tasks_clear
        ));
        s.push_str("                self\n");
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn start(self) -> Self {\n");
        s.push_str(&format!(
            "                self.0.{}.write(1);\n",
            self.field_tasks_start
        ));
        s.push_str("                self\n");
        s.push_str("            }\n");
        s.push_str("        }\n\n");

        s.push_str("        impl<'a> TimerModeLowPowerCounter<'a> {\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn clear_event_compare(self, index: usize) -> Self {\n");
        s.push_str(&format!(
            "                if index < self.0.{}.len() {{\n",
            self.field_events_compare
        ));
        s.push_str(&format!(
            "                    self.0.{}[index].write(0);\n",
            self.field_events_compare
        ));
        s.push_str("                }\n");
        s.push_str("                self\n");
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn clear(self) -> Self {\n");
        s.push_str(&format!(
            "                self.0.{}.write(1);\n",
            self.field_tasks_clear
        ));
        s.push_str("                self\n");
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn start(self) -> Self {\n");
        s.push_str(&format!(
            "                self.0.{}.write(1);\n",
            self.field_tasks_start
        ));
        s.push_str("                self\n");
        s.push_str("            }\n");
        s.push_str("        }\n");

        s.push_str("    }\n");
        Ok(s)
    }
}

pub fn collect_timers(device: &svd::Device) -> Vec<TimerInfo> {
    let mut out = Vec::new();
    for p in &device.peripherals {
        if !is_timer_like(&p.name) {
            continue;
        }
        let items = gpio::peripheral_register_items(device, p);
        if items.is_empty() {
            continue;
        }

        let Some((mode_name, mode_reg)) = gpio::find_register_prefer_exact(items, "MODE") else {
            continue;
        };
        let Some((bitmode_name, bitmode_reg)) = gpio::find_register_prefer_exact(items, "BITMODE")
        else {
            continue;
        };
        let Some((prescaler_name, _)) = gpio::find_register(items, "PRESCALER") else {
            continue;
        };
        let Some((cc_name, cc_reg)) = gpio::find_register(items, "CC") else {
            continue;
        };
        let Some((shorts_name, shorts_reg)) = gpio::find_register(items, "SHORTS") else {
            continue;
        };
        let Some((intenset_name, intenset_reg)) = gpio::find_register(items, "INTENSET") else {
            continue;
        };
        let Some((events_compare_name, events_compare_reg)) =
            gpio::find_register(items, "EVENTS_COMPARE")
        else {
            continue;
        };
        let Some((tasks_clear_name, _)) = gpio::find_register(items, "TASKS_CLEAR") else {
            continue;
        };
        let Some((tasks_start_name, _)) = gpio::find_register(items, "TASKS_START") else {
            continue;
        };

        let mode_field = mode_reg
            .field
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case("MODE"))
            .cloned()
            .or_else(|| mode_reg.field.first().cloned());
        let bitmode_field = bitmode_reg
            .field
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case("BITMODE"))
            .cloned()
            .or_else(|| bitmode_reg.field.first().cloned());

        let (Some(mode_field), Some(bitmode_field)) = (mode_field, bitmode_field) else {
            continue;
        };

        let shorts_fields = collect_compare_fields(&shorts_reg.field, true);
        let intenset_fields = collect_compare_fields(&intenset_reg.field, false);

        let cc_len = cc_reg.dim.as_ref().map(|d| d.dim as usize).unwrap_or(1);
        let events_compare_len = events_compare_reg
            .dim
            .as_ref()
            .map(|d| d.dim as usize)
            .unwrap_or(1);

        let _ = (cc_len, events_compare_len);

        out.push(TimerInfo {
            periph_name: p.name.clone(),
            periph_mod: gpio::sanitize_module_name(&p.name),
            hal_mod: gpio::sanitize_field_name(&p.name),

            field_mode: gpio::sanitize_field_name(&mode_name),
            field_bitmode: gpio::sanitize_field_name(&bitmode_name),
            field_prescaler: gpio::sanitize_field_name(&prescaler_name),
            field_cc: gpio::sanitize_field_name(&cc_name),
            field_shorts: gpio::sanitize_field_name(&shorts_name),
            field_intenset: gpio::sanitize_field_name(&intenset_name),
            field_events_compare: gpio::sanitize_field_name(&events_compare_name),
            field_tasks_clear: gpio::sanitize_field_name(&tasks_clear_name),
            field_tasks_start: gpio::sanitize_field_name(&tasks_start_name),

            mode_reg_path: mode_reg.name.clone(),
            bitmode_reg_path: bitmode_reg.name.clone(),
            mode_field,
            bitmode_field,
            shorts_fields,
            intenset_fields,
        });
    }
    out
}

fn is_timer_like(name: &str) -> bool {
    let b = name.as_bytes();
    if b.len() < 6 {
        return false;
    }
    let (pfx, rest) = b.split_at(5);
    if pfx != b"TIMER" && pfx != b"timer" {
        return false;
    }
    rest.iter().all(|c| c.is_ascii_digit())
}

fn collect_compare_fields(fields: &[svd::Field], needs_clear: bool) -> Vec<(usize, svd::Field)> {
    let mut out: Vec<(usize, svd::Field)> = Vec::new();
    for f in fields {
        let name = f.name.to_ascii_uppercase();
        let Some(idx) = parse_compare_index(&name) else {
            continue;
        };
        if needs_clear && !name.contains("CLEAR") {
            continue;
        }
        if !needs_clear && name.contains("CLEAR") {
            continue;
        }
        out.push((idx, f.clone()));
    }
    out.sort_by_key(|(idx, _)| *idx);
    out
}

fn parse_compare_index(name_upper: &str) -> Option<usize> {
    let p = name_upper.find("COMPARE")?;
    let rest = &name_upper[p + "COMPARE".len()..];
    let digits = rest
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<usize>().ok()
}

fn pac_enum_type_name_for_field(
    periph_name: &str,
    reg_path: &str,
    f: &svd::Field,
) -> Option<String> {
    let evs = f.enumerated_values.first()?;
    let has_numeric = evs.enumerated_value.iter().any(|v| match &v.spec {
        svd::EnumeratedValueSpec::Value { value } => gpio::parse_enum_u64(value).is_some(),
        svd::EnumeratedValueSpec::IsDefault { .. } => false,
    });
    if !has_numeric {
        return None;
    }

    let base = evs
        .header_enum_name
        .as_deref()
        .or(evs.name.as_deref())
        .map(gpio::sanitize_type_name)
        .unwrap_or_else(|| {
            gpio::sanitize_type_name(&format!(
                "{}_{}_{}",
                periph_name,
                reg_path.replace('.', "_"),
                f.name
            ))
        });
    Some(base)
}

fn render_field_enum(field_name: &str, f: &svd::Field) -> Option<String> {
    let evs = f.enumerated_values.first()?;
    let mut vars: Vec<(String, u64)> = Vec::new();
    for v in &evs.enumerated_value {
        let Some(val) = (match &v.spec {
            svd::EnumeratedValueSpec::Value { value } => gpio::parse_enum_u64(value),
            svd::EnumeratedValueSpec::IsDefault { .. } => None,
        }) else {
            continue;
        };
        vars.push((gpio::sanitize_variant_name(&v.name), val));
    }
    if vars.is_empty() {
        return None;
    }

    let ty = gpio::sanitize_type_name(field_name);
    let mut s = String::new();
    s.push_str("#[repr(u32)]\n");
    s.push_str("#[derive(Copy, Clone, Debug, PartialEq, Eq)]\n");
    s.push_str(&format!("pub enum {ty} {{\n"));
    for (n, v) in &vars {
        s.push_str(&format!("    {n} = {v},\n"));
    }
    s.push_str("}\n");
    Some(s)
}

fn field_lsb_width(f: &svd::Field) -> (u32, u32) {
    match f.bit_range {
        svd::BitRange::BitRangeString { msb, lsb } => (lsb, msb.saturating_sub(lsb) + 1),
        svd::BitRange::LsbMsb { lsb, msb } => (lsb, msb.saturating_sub(lsb) + 1),
        svd::BitRange::BitOffsetWidth {
            bit_offset,
            bit_width,
        } => (bit_offset, bit_width.unwrap_or(1)),
    }
}

fn indent_block(s: &str, spaces: usize) -> String {
    let pad = " ".repeat(spaces);
    s.lines()
        .map(|l| {
            if l.is_empty() {
                "\n".to_string()
            } else {
                format!("{pad}{l}\n")
            }
        })
        .collect()
}

fn sanitize_type_name(s: &str) -> String {
    gpio::sanitize_type_name(s)
}
