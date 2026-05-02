#[derive(Debug, Clone, Copy)]
pub struct HalOptions {
    pub emit_gpio: bool,
    pub emit_timer: bool,
    pub emit_clock: bool,
    pub emit_power: bool,
    pub emit_usb: bool,
    pub emit_usb_cdc_acm: bool,
    pub emit_async_rt: bool,
}

impl Default for HalOptions {
    fn default() -> Self {
        Self {
            emit_gpio: true,
            emit_timer: true,
            emit_clock: true,
            emit_power: true,
            emit_usb: true,
            emit_usb_cdc_acm: true,
            emit_async_rt: false,
        }
    }
}

impl HalOptions {
    pub const fn all() -> Self {
        Self {
            emit_gpio: true,
            emit_timer: true,
            emit_clock: true,
            emit_power: true,
            emit_usb: true,
            emit_usb_cdc_acm: true,
            emit_async_rt: true,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HalOutputMode {
    SingleFile,
    Directory,
}

#[derive(Debug, Clone)]
pub struct HalGenerationPlan {
    pub output_mode: HalOutputMode,
    pub options: HalOptions,
    pub pac_crate_name: String,
    pub dir_name: String,
    pub enabled_modules: Vec<HalModulePlan>,
}

#[derive(Debug, Clone)]
pub enum HalModulePlan {
    Gpio,
    Timer,
    Clock,
    Power,
    Usb,
    AsyncRt,
}

impl HalGenerationPlan {
    pub fn has_module(&self, m: &HalModulePlan) -> bool {
        let name = match m {
            HalModulePlan::Gpio => "gpio",
            HalModulePlan::Timer => "timer",
            HalModulePlan::Clock => "clock",
            HalModulePlan::Power => "power",
            HalModulePlan::Usb => "usb",
            HalModulePlan::AsyncRt => "async_rt",
        };
        self.enabled_modules
            .iter()
            .any(|x| std::mem::discriminant(x) == std::mem::discriminant(m))
            || self.enabled_modules.iter().any(|x| match (x, m) {
                (HalModulePlan::Gpio, HalModulePlan::Gpio) => true,
                (HalModulePlan::Timer, HalModulePlan::Timer) => true,
                (HalModulePlan::Clock, HalModulePlan::Clock) => true,
                (HalModulePlan::Power, HalModulePlan::Power) => true,
                (HalModulePlan::Usb, HalModulePlan::Usb) => true,
                (HalModulePlan::AsyncRt, HalModulePlan::AsyncRt) => true,
                _ => false,
            })
    }
}
