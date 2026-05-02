use crate::svd;

#[derive(Debug, Clone)]
pub struct HalIr {
    pub gpio_ports: Vec<GpioPortIr>,
    pub timers: Vec<TimerIr>,
    pub usb_devices: Vec<UsbIr>,
    pub clocks: Vec<ClockIr>,
    pub power_devices: Vec<PowerIr>,
}

// ─── GPIO ────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GpioPortIr {
    pub periph_name: String,
    pub periph_mod: String,
    pub hal_mod: String,
    pub port_type_name: String,
    pub field_outset: String,
    pub field_outclr: String,
    pub field_out: String,
    pub field_pin_cnf: String,
    pub pin_cnf_reg_path: String,
    pub pin_fields: Vec<PinFieldIr>,
    pub level_enum_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PinFieldIr {
    pub name: String,
    pub alias: String,
    pub lsb: u32,
    pub width: u32,
    pub mask: u32,
    pub pac_enum_binding: Option<String>,
    pub local_enum_def: Option<String>,
    pub output_dir_value: Option<u32>,
}

// ─── Timer ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct TimerIr {
    pub periph_name: String,
    pub periph_mod: String,
    pub hal_mod: String,
    pub timer_type_name: String,
    pub field_mode: String,
    pub field_bitmode: String,
    pub field_prescaler: String,
    pub field_cc: String,
    pub field_shorts: String,
    pub field_intenset: String,
    pub field_events_compare: String,
    pub field_tasks_clear: String,
    pub field_tasks_start: String,
    pub mode_reg_path: String,
    pub bitmode_reg_path: String,
    pub mode_lsb: u32,
    pub mode_width: u32,
    pub mode_mask: u32,
    pub bitmode_lsb: u32,
    pub bitmode_width: u32,
    pub bitmode_mask: u32,
    pub mode_enum_is_pac: bool,
    pub mode_enum_name: String,
    pub mode_enum_local_def: Option<String>,
    pub bitmode_enum_is_pac: bool,
    pub bitmode_enum_name: String,
    pub bitmode_enum_local_def: Option<String>,
    pub compare_channels: Vec<usize>,
    pub shorts_fields: Vec<CompareField>,
    pub intenset_fields: Vec<CompareField>,
}

#[derive(Debug, Clone)]
pub struct CompareField {
    pub index: usize,
    pub lsb: u32,
    pub width: u32,
    pub mask: u32,
}

// ─── Clock ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ClockIr {
    pub periph_name: String,
    pub periph_mod: String,
    pub hal_mod: String,
    pub field_tasks_hfclk_start: String,
    pub field_tasks_hfclk_stop: String,
    pub field_tasks_lfclk_start: String,
    pub field_tasks_lfclk_stop: String,
    pub field_tasks_cal: String,
    pub field_tasks_ctstart: String,
    pub field_tasks_ctstop: String,
    pub field_events_hfclk_started: String,
    pub field_events_lfclk_started: String,
    pub field_events_done: String,
    pub field_events_ctto: String,
    pub field_events_ctstarted: String,
    pub field_hfclkstat: String,
    pub field_lfclkstat: String,
}

// ─── Power ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PowerIr {
    pub periph_name: String,
    pub periph_mod: String,
    pub hal_mod: String,
    pub field_tasks_constlat: String,
    pub field_tasks_lowpwr: String,
    pub field_events_pofwarn: String,
    pub field_events_sleepenter: String,
    pub field_events_sleepexit: String,
    pub field_events_usbdetected: String,
    pub field_events_usbremoved: String,
    pub field_events_usbpwrrdy: String,
    pub field_intenset: String,
    pub field_intenclr: String,
}

// ─── USB ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct UsbIr {
    pub periph_name: String,
    pub periph_mod: String,
    pub hal_mod: String,
    pub field_enable: String,
    pub field_usb_pullup: String,
    pub field_epinen: String,
    pub field_epouten: String,
    pub field_epin: Option<String>,
    pub field_epout: Option<String>,
    pub field_tasks_startein: String,
    pub field_tasks_staroutep: String,
    pub field_events_ep0setup: String,
    pub field_events_ep0datadone: String,
    pub field_events_endepin: String,
    pub field_events_endepout: String,
    pub field_events_usbreset: String,
    pub field_events_usbevent: String,
    pub field_tasks_startepin: String,
    pub field_tasks_startepout: String,
    pub field_events_endepin_array: String,
    pub field_events_endepout_array: String,
    pub has_ep_arrays: bool,
}
