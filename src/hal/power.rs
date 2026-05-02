use super::gpio;
use crate::{Result, svd};

#[derive(Debug, Clone)]
pub struct PowerInfo {
    periph_name: String,
    periph_mod: String,
    hal_mod: String,

    field_tasks_constlat: String,
    field_tasks_lowpwr: String,

    field_events_pofwarn: String,
    field_events_sleepenter: String,
    field_events_sleepexit: String,
    field_events_usbdetected: String,
    field_events_usbremoved: String,
    field_events_usbpwrrdy: String,

    field_inten_pofwarn: String,
    field_inten_sleepenter: String,
    field_inten_sleepexit: String,
    field_inten_usbdetected: String,
    field_inten_usbremoved: String,
    field_inten_usbpwrrdy: String,

    field_intenset: String,
    field_intenclr: String,
}

impl PowerInfo {
    pub fn render(&self) -> Result<String> {
        let mut s = String::new();

        let type_name = gpio::sanitize_type_name(self.hal_mod.as_str());
        s.push_str(&format!(
            "pub type {}Register = crate::pac::peripherals::{}::{};\n\n",
            type_name, self.periph_mod, type_name,
        ));

        s.push_str("use core::marker::PhantomData;\n\n");

        s.push_str("pub trait PowerState {}\n");
        s.push_str("pub struct Unconfigured;\n");
        s.push_str("pub struct ConstLat;\n");
        s.push_str("pub struct LowPower;\n\n");
        s.push_str("impl PowerState for Unconfigured {}\n");
        s.push_str("impl PowerState for ConstLat {}\n");
        s.push_str("impl PowerState for LowPower {}\n\n");

        s.push_str("#[repr(u8)]\n");
        s.push_str("#[derive(Copy, Clone, Debug, PartialEq, Eq)]\n");
        s.push_str("pub enum PowerMode {\n");
        s.push_str("    ConstantLatency = 0,\n");
        s.push_str("    LowPower = 1,\n");
        s.push_str("}\n\n");

        s.push_str(&format!("pub struct Power<'a, S: PowerState> {{\n"));
        s.push_str(&format!("    power: &'a {}Register,\n", type_name));
        s.push_str("    _state: PhantomData<S>,\n");
        s.push_str("}\n\n");

        s.push_str("impl<'a, S: PowerState> Power<'a, S> {\n");
        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn is_power_failure_warning(&self) -> bool {\n");
        s.push_str(&format!(
            "        self.power.{}.read() != 0\n",
            self.field_events_pofwarn
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn is_sleep_enter(&self) -> bool {\n");
        s.push_str(&format!(
            "        self.power.{}.read() != 0\n",
            self.field_events_sleepenter
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn is_sleep_exit(&self) -> bool {\n");
        s.push_str(&format!(
            "        self.power.{}.read() != 0\n",
            self.field_events_sleepexit
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn is_usb_detected(&self) -> bool {\n");
        s.push_str(&format!(
            "        self.power.{}.read() != 0\n",
            self.field_events_usbdetected
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn is_usb_removed(&self) -> bool {\n");
        s.push_str(&format!(
            "        self.power.{}.read() != 0\n",
            self.field_events_usbremoved
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn is_usb_power_ready(&self) -> bool {\n");
        s.push_str(&format!(
            "        self.power.{}.read() != 0\n",
            self.field_events_usbpwrrdy
        ));
        s.push_str("    }\n");
        s.push_str("}\n\n");

        s.push_str("impl<'a> Power<'a, Unconfigured> {\n");
        s.push_str("    #[inline(always)]\n");
        s.push_str(&format!(
            "    pub unsafe fn steal() -> Power<'static, Unconfigured> {{\n"
        ));
        s.push_str(&format!(
            "        Power {{ power: &*crate::pac::peripherals::{}::PTR, _state: PhantomData }}\n",
            self.periph_mod
        ));
        s.push_str("    }\n\n");

        s.push_str(&format!(
            "    pub fn power() -> Power<'static, Unconfigured> {{\n"
        ));
        s.push_str(&format!(
            "        Power {{ power: unsafe {{ &*crate::pac::peripherals::{}::PTR }}, _state: PhantomData }}\n",
            self.periph_mod
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn enable_constant_latency(self) -> Power<'a, ConstLat> {\n");
        s.push_str(&format!(
            "        self.power.{}.write(1);\n",
            self.field_tasks_constlat
        ));
        s.push_str(&format!(
            "        Power {{ power: self.power, _state: PhantomData }}\n"
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn enable_low_power(self) -> Power<'a, LowPower> {\n");
        s.push_str(&format!(
            "        self.power.{}.write(1);\n",
            self.field_tasks_lowpwr
        ));
        s.push_str(&format!(
            "        Power {{ power: self.power, _state: PhantomData }}\n"
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn enable_interrupts(&mut self) {\n");
        s.push_str(&format!(
            "        self.power.{}.write(0x3F);\n",
            self.field_intenset
        ));
        s.push_str("    }\n\n");

        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn disable_interrupts(&mut self) {\n");
        s.push_str(&format!(
            "        self.power.{}.write(0x3F);\n",
            self.field_intenclr
        ));
        s.push_str("    }\n");
        s.push_str("}\n\n");

        s.push_str(&format!("impl<'a> Power<'a, ConstLat> {{\n"));
        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn to_unconfigured(self) -> Power<'a, Unconfigured> {\n");
        s.push_str(&format!(
            "        Power {{ power: self.power, _state: PhantomData }}\n"
        ));
        s.push_str("    }\n");
        s.push_str("}\n\n");

        s.push_str(&format!("impl<'a> Power<'a, LowPower> {{\n"));
        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn to_unconfigured(self) -> Power<'a, Unconfigured> {\n");
        s.push_str(&format!(
            "        Power {{ power: self.power, _state: PhantomData }}\n"
        ));
        s.push_str("    }\n");
        s.push_str("}\n\n");

        s.push_str("impl<'a> Power<'a, ConstLat> {\n");
        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn enable_low_power(self) -> Power<'a, LowPower> {\n");
        s.push_str(&format!(
            "        self.power.{}.write(1);\n",
            self.field_tasks_lowpwr
        ));
        s.push_str(&format!(
            "        Power {{ power: self.power, _state: PhantomData }}\n"
        ));
        s.push_str("    }\n");
        s.push_str("}\n\n");

        s.push_str("impl<'a> Power<'a, LowPower> {\n");
        s.push_str("    #[inline(always)]\n");
        s.push_str("    pub fn enable_constant_latency(self) -> Power<'a, ConstLat> {\n");
        s.push_str(&format!(
            "        self.power.{}.write(1);\n",
            self.field_tasks_constlat
        ));
        s.push_str(&format!(
            "        Power {{ power: self.power, _state: PhantomData }}\n"
        ));
        s.push_str("    }\n");
        s.push_str("}\n");

        Ok(s)
    }
}

pub fn collect_power_devices(device: &svd::Device) -> Vec<PowerInfo> {
    let mut out = Vec::new();
    for p in &device.peripherals {
        if !is_power_peripheral(&p.name) {
            continue;
        }
        let items = gpio::peripheral_register_items(device, p);
        if items.is_empty() {
            continue;
        }

        let Some((tasks_constlat_name, _)) = gpio::find_register(items, "TASKS_CONSTLAT") else {
            continue;
        };
        let Some((tasks_lowpwr_name, _)) = gpio::find_register(items, "TASKS_LOWPWR") else {
            continue;
        };

        let Some((events_pofwarn_name, _)) = gpio::find_register(items, "EVENTS_POFWARN") else {
            continue;
        };
        let Some((events_sleepenter_name, _)) = gpio::find_register(items, "EVENTS_SLEEPENTER")
        else {
            continue;
        };
        let Some((events_sleepexit_name, _)) = gpio::find_register(items, "EVENTS_SLEEPEXIT")
        else {
            continue;
        };
        let Some((events_usbdetected_name, _)) = gpio::find_register(items, "EVENTS_USBDETECTED")
        else {
            continue;
        };
        let Some((events_usbremoved_name, _)) = gpio::find_register(items, "EVENTS_USBREMOVED")
        else {
            continue;
        };
        let Some((events_usbpwrrdy_name, _)) = gpio::find_register(items, "EVENTS_USBPWRRDY")
        else {
            continue;
        };

        let Some((intenset_name, _)) = gpio::find_register(items, "INTENSET") else {
            continue;
        };
        let Some((intenclr_name, _)) = gpio::find_register(items, "INTENCLR") else {
            continue;
        };

        let Some((inten_pofwarn_name, inten_pofwarn_reg)) =
            gpio::find_register_prefer_exact(items, "POFWARN")
        else {
            continue;
        };
        let inten_sleepenter_name = inten_pofwarn_reg
            .field
            .iter()
            .find(|f| f.name.to_ascii_uppercase().contains("SLEEPENTER"))
            .map(|f| f.name.clone())
            .unwrap_or_else(|| "SLEEPENTER".to_string());
        let inten_sleepexit_name = inten_pofwarn_reg
            .field
            .iter()
            .find(|f| f.name.to_ascii_uppercase().contains("SLEEPEXIT"))
            .map(|f| f.name.clone())
            .unwrap_or_else(|| "SLEEPEXIT".to_string());
        let inten_usbdetected_name = inten_pofwarn_reg
            .field
            .iter()
            .find(|f| f.name.to_ascii_uppercase().contains("USBDETECTED"))
            .map(|f| f.name.clone())
            .unwrap_or_else(|| "USBDETECTED".to_string());
        let inten_usbremoved_name = inten_pofwarn_reg
            .field
            .iter()
            .find(|f| f.name.to_ascii_uppercase().contains("USBREMOVED"))
            .map(|f| f.name.clone())
            .unwrap_or_else(|| "USBREMOVED".to_string());
        let inten_usbpwrrdy_name = inten_pofwarn_reg
            .field
            .iter()
            .find(|f| f.name.to_ascii_uppercase().contains("USBPWRRDY"))
            .map(|f| f.name.clone())
            .unwrap_or_else(|| "USBPWRRDY".to_string());

        out.push(PowerInfo {
            periph_name: p.name.clone(),
            periph_mod: gpio::sanitize_module_name(&p.name),
            hal_mod: gpio::sanitize_field_name(&p.name),

            field_tasks_constlat: gpio::sanitize_field_name(&tasks_constlat_name),
            field_tasks_lowpwr: gpio::sanitize_field_name(&tasks_lowpwr_name),

            field_events_pofwarn: gpio::sanitize_field_name(&events_pofwarn_name),
            field_events_sleepenter: gpio::sanitize_field_name(&events_sleepenter_name),
            field_events_sleepexit: gpio::sanitize_field_name(&events_sleepexit_name),
            field_events_usbdetected: gpio::sanitize_field_name(&events_usbdetected_name),
            field_events_usbremoved: gpio::sanitize_field_name(&events_usbremoved_name),
            field_events_usbpwrrdy: gpio::sanitize_field_name(&events_usbpwrrdy_name),

            field_intenset: gpio::sanitize_field_name(&intenset_name),
            field_intenclr: gpio::sanitize_field_name(&intenclr_name),

            field_inten_pofwarn: gpio::sanitize_field_name(&inten_pofwarn_name),
            field_inten_sleepenter: gpio::sanitize_field_name(&inten_sleepenter_name),
            field_inten_sleepexit: gpio::sanitize_field_name(&inten_sleepexit_name),
            field_inten_usbdetected: gpio::sanitize_field_name(&inten_usbdetected_name),
            field_inten_usbremoved: gpio::sanitize_field_name(&inten_usbremoved_name),
            field_inten_usbpwrrdy: gpio::sanitize_field_name(&inten_usbpwrrdy_name),
        });
    }
    out
}

fn is_power_peripheral(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    upper.contains("POWER")
}
