use crate::Result;
use crate::svd;
use crate::hal::common::*;
use crate::hal::ir::*;

pub fn analyze(device: &svd::Device) -> HalIr {
    HalIr {
        gpio_ports: collect_gpio_ports(device),
        timers: collect_timers(device),
        usb_devices: usb::collect(device),
        clocks: collect_clocks(device),
        power_devices: collect_power_devices(device),
    }
}

// ─── GPIO ────────────────────────────────────────────────────────

fn collect_gpio_ports(device: &svd::Device) -> Vec<GpioPortIr> {
    let mut out = Vec::new();
    for p in &device.peripherals {
        if !is_gpio_like_port(&p.name) {
            continue;
        }
        let items = peripheral_register_items(device, p);
        if items.is_empty() {
            continue;
        }
        let Some((pin_cnf_name, pin_cnf_reg)) = find_register(items, "PIN_CNF") else {
            continue;
        };
        let Some((outset_name, _)) = find_register(items, "OUTSET") else {
            continue;
        };
        let Some((outclr_name, _)) = find_register(items, "OUTCLR") else {
            continue;
        };
        let Some((out_name, out_reg)) = find_register(items, "OUT") else {
            continue;
        };
        let level_enum_name = find_level_enum_for_out_reg(p, out_reg);

        let mut pin_fields = Vec::new();
        for field in &pin_cnf_reg.field {
            let upper = field.name.to_ascii_uppercase();
            let (alias, name) = if upper.contains("DIR") {
                ("Dir".to_string(), "DIR".to_string())
            } else if upper.contains("DRIVE") {
                ("Drive".to_string(), "DRIVE".to_string())
            } else if upper.contains("SENSE") {
                ("Sense".to_string(), "SENSE".to_string())
            } else if upper.contains("PULL") {
                ("Pull".to_string(), "PULL".to_string())
            } else {
                continue;
            };

            let (lsb, width) = field_lsb_width(field);
            let mask: u32 = (1u32 << width) - 1;
            let output_dir_value = if name == "DIR" {
                infer_output_value(field)
            } else {
                None
            };

            let pac_enum_binding = pac_enum_type_name_for_field(
                &p.name,
                &pin_cnf_reg.name,
                field,
            );
            let local_enum_def = if pac_enum_binding.is_none() {
                render_field_enum(&name, field)
            } else {
                None
            };

            pin_fields.push(PinFieldIr {
                name,
                alias,
                lsb,
                width,
                mask,
                pac_enum_binding,
                local_enum_def,
                output_dir_value,
            });
        }

        out.push(GpioPortIr {
            periph_name: p.name.clone(),
            periph_mod: sanitize_module_name(&p.name),
            hal_mod: sanitize_field_name(&p.name),
            port_type_name: sanitize_type_name(&p.name),
            field_outset: sanitize_field_name(&outset_name),
            field_outclr: sanitize_field_name(&outclr_name),
            field_out: sanitize_field_name(&out_name),
            field_pin_cnf: sanitize_field_name(&pin_cnf_name),
            pin_cnf_reg_path: pin_cnf_reg.name.clone(),
            pin_fields,
            level_enum_name,
        });
    }
    out
}

fn is_gpio_like_port(name: &str) -> bool {
    let b = name.as_bytes();
    if b.len() < 2 {
        return false;
    }
    if b[0] != b'P' && b[0] != b'p' {
        return false;
    }
    b[1..].iter().all(|c| c.is_ascii_digit())
}

fn find_level_enum_for_out_reg(
    p: &svd::Peripheral,
    out_reg: &svd::Register,
) -> Option<String> {
    let pin0_field = out_reg.field.iter().find(|f| {
        let name_upper = f.name.to_ascii_uppercase();
        name_upper == "PIN0" || name_upper.ends_with("_PIN0")
    })?;
    let evs = pin0_field.enumerated_values.first()?;
    if evs.enumerated_value.is_empty() {
        return None;
    }
    let has_low = evs
        .enumerated_value
        .iter()
        .any(|v| v.name.to_ascii_lowercase() == "low");
    let has_high = evs
        .enumerated_value
        .iter()
        .any(|v| v.name.to_ascii_lowercase() == "high");
    if has_low && has_high {
        pac_enum_type_name_for_field(
            &p.name,
            &out_reg.name,
            pin0_field,
        )
    } else {
        None
    }
}

// ─── Timer ───────────────────────────────────────────────────────

fn collect_timers(device: &svd::Device) -> Vec<TimerIr> {
    let mut out = Vec::new();
    for p in &device.peripherals {
        if !is_timer_like(&p.name) {
            continue;
        }
        let items = peripheral_register_items(device, p);
        if items.is_empty() {
            continue;
        }
        let Some((mode_name, mode_reg)) = find_register_prefer_exact(items, "MODE") else {
            continue;
        };
        let Some((bitmode_name, bitmode_reg)) = find_register_prefer_exact(items, "BITMODE")
        else {
            continue;
        };
        let Some((prescaler_name, _)) = find_register(items, "PRESCALER") else {
            continue;
        };
        let Some((cc_name, cc_reg)) = find_register(items, "CC") else {
            continue;
        };
        let Some((shorts_name, shorts_reg)) = find_register(items, "SHORTS") else {
            continue;
        };
        let Some((intenset_name, intenset_reg)) = find_register(items, "INTENSET") else {
            continue;
        };
        let Some((events_compare_name, events_compare_reg)) =
            find_register(items, "EVENTS_COMPARE")
        else {
            continue;
        };
        let Some((tasks_clear_name, _)) = find_register(items, "TASKS_CLEAR") else {
            continue;
        };
        let Some((tasks_start_name, _)) = find_register(items, "TASKS_START") else {
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

        let (mode_lsb, mode_width) = field_lsb_width(&mode_field);
        let mode_mask: u32 = if mode_width >= 32 { u32::MAX } else { ((1u64 << mode_width) - 1) as u32 };
        let (bitmode_lsb, bitmode_width) = field_lsb_width(&bitmode_field);
        let bitmode_mask: u32 = if bitmode_width >= 32 { u32::MAX } else { ((1u64 << bitmode_width) - 1) as u32 };

        let mode_enum_is_pac;
        let mode_enum_name;
        let mode_enum_local_def;
        if let Some(ty) = pac_enum_type_name_for_field(&p.name, &mode_reg.name, &mode_field) {
            mode_enum_is_pac = true;
            mode_enum_name = sanitize_type_name("MODE");
            mode_enum_local_def = None;
            let _ = ty;
        } else if let Some(def) = render_field_enum("MODE", &mode_field) {
            mode_enum_is_pac = false;
            mode_enum_name = sanitize_type_name("MODE");
            mode_enum_local_def = Some(def);
        } else {
            mode_enum_is_pac = false;
            mode_enum_name = "u32".to_string();
            mode_enum_local_def = None;
        }

        let bitmode_enum_is_pac;
        let bitmode_enum_name;
        let bitmode_enum_local_def;
        if let Some(ty) = pac_enum_type_name_for_field(&p.name, &bitmode_reg.name, &bitmode_field) {
            bitmode_enum_is_pac = true;
            bitmode_enum_name = sanitize_type_name("BITMODE");
            bitmode_enum_local_def = None;
            let _ = ty;
        } else if let Some(def) = render_field_enum("BITMODE", &bitmode_field) {
            bitmode_enum_is_pac = false;
            bitmode_enum_name = sanitize_type_name("BITMODE");
            bitmode_enum_local_def = Some(def);
        } else {
            bitmode_enum_is_pac = false;
            bitmode_enum_name = "u32".to_string();
            bitmode_enum_local_def = None;
        }

        let shorts_raw = collect_compare_fields(&shorts_reg.field, true);
        let intenset_raw = collect_compare_fields(&intenset_reg.field, false);

        let shorts_fields: Vec<CompareField> = shorts_raw.into_iter().map(|(idx, f)| {
            let (lsb, width) = field_lsb_width(&f);
            let mask: u32 = if width >= 32 { u32::MAX } else { ((1u64 << width) - 1) as u32 };
            CompareField { index: idx, lsb, width, mask }
        }).collect();

        let intenset_fields: Vec<CompareField> = intenset_raw.into_iter().map(|(idx, f)| {
            let (lsb, width) = field_lsb_width(&f);
            let mask: u32 = if width >= 32 { u32::MAX } else { ((1u64 << width) - 1) as u32 };
            CompareField { index: idx, lsb, width, mask }
        }).collect();

        let mut compare_channels: Vec<usize> = shorts_fields.iter().map(|f| f.index).collect();
        for f in &intenset_fields {
            if !compare_channels.contains(&f.index) {
                compare_channels.push(f.index);
            }
        }
        compare_channels.sort();

        out.push(TimerIr {
            periph_name: p.name.clone(),
            periph_mod: sanitize_module_name(&p.name),
            hal_mod: sanitize_field_name(&p.name),
            timer_type_name: sanitize_type_name(&p.name),
            field_mode: sanitize_field_name(&mode_name),
            field_bitmode: sanitize_field_name(&bitmode_name),
            field_prescaler: sanitize_field_name(&prescaler_name),
            field_cc: sanitize_field_name(&cc_name),
            field_shorts: sanitize_field_name(&shorts_name),
            field_intenset: sanitize_field_name(&intenset_name),
            field_events_compare: sanitize_field_name(&events_compare_name),
            field_tasks_clear: sanitize_field_name(&tasks_clear_name),
            field_tasks_start: sanitize_field_name(&tasks_start_name),
            mode_reg_path: mode_reg.name.clone(),
            bitmode_reg_path: bitmode_reg.name.clone(),
            mode_lsb, mode_width, mode_mask,
            bitmode_lsb, bitmode_width, bitmode_mask,
            mode_enum_is_pac, mode_enum_name, mode_enum_local_def,
            bitmode_enum_is_pac, bitmode_enum_name, bitmode_enum_local_def,
            compare_channels,
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

// ─── Clock ───────────────────────────────────────────────────────

fn collect_clocks(device: &svd::Device) -> Vec<ClockIr> {
    let mut out = Vec::new();
    for p in &device.peripherals {
        if !is_clock_like(&p.name) {
            continue;
        }
        let items = peripheral_register_items(device, p);
        if items.is_empty() {
            continue;
        }
        let Some((hf_start_name, _)) = find_register(items, "TASKS_HFCLKSTART") else { continue; };
        let Some((hf_stop_name, _)) = find_register(items, "TASKS_HFCLKSTOP") else { continue; };
        let Some((lf_start_name, _)) = find_register(items, "TASKS_LFCLKSTART") else { continue; };
        let Some((lf_stop_name, _)) = find_register(items, "TASKS_LFCLKSTOP") else { continue; };
        let Some((cal_name, _)) = find_register(items, "TASKS_CAL") else { continue; };
        let Some((ctstart_name, _)) = find_register(items, "TASKS_CTSTART") else { continue; };
        let Some((ctstop_name, _)) = find_register(items, "TASKS_CTSTOP") else { continue; };
        let Some((hf_started_name, _)) = find_register(items, "EVENTS_HFCLKSTARTED") else { continue; };
        let Some((lf_started_name, _)) = find_register(items, "EVENTS_LFCLKSTARTED") else { continue; };
        let Some((done_name, _)) = find_register(items, "EVENTS_DONE") else { continue; };
        let Some((ctto_name, _)) = find_register(items, "EVENTS_CTTO") else { continue; };
        let Some((ctstarted_name, _)) = find_register(items, "EVENTS_CTSTARTED") else { continue; };
        let Some((hfstat_name, _)) = find_register(items, "HFCLKSTAT") else { continue; };
        let Some((lfstat_name, _)) = find_register(items, "LFCLKSTAT") else { continue; };

        out.push(ClockIr {
            periph_name: p.name.clone(),
            periph_mod: sanitize_module_name(&p.name),
            hal_mod: sanitize_field_name(&p.name),
            field_tasks_hfclk_start: sanitize_field_name(&hf_start_name),
            field_tasks_hfclk_stop: sanitize_field_name(&hf_stop_name),
            field_tasks_lfclk_start: sanitize_field_name(&lf_start_name),
            field_tasks_lfclk_stop: sanitize_field_name(&lf_stop_name),
            field_tasks_cal: sanitize_field_name(&cal_name),
            field_tasks_ctstart: sanitize_field_name(&ctstart_name),
            field_tasks_ctstop: sanitize_field_name(&ctstop_name),
            field_events_hfclk_started: sanitize_field_name(&hf_started_name),
            field_events_lfclk_started: sanitize_field_name(&lf_started_name),
            field_events_done: sanitize_field_name(&done_name),
            field_events_ctto: sanitize_field_name(&ctto_name),
            field_events_ctstarted: sanitize_field_name(&ctstarted_name),
            field_hfclkstat: sanitize_field_name(&hfstat_name),
            field_lfclkstat: sanitize_field_name(&lfstat_name),
        });
    }
    out
}

fn is_clock_like(name: &str) -> bool {
    name.to_ascii_uppercase().contains("CLOCK")
}

// ─── Power ───────────────────────────────────────────────────────

fn collect_power_devices(device: &svd::Device) -> Vec<PowerIr> {
    let mut out = Vec::new();
    for p in &device.peripherals {
        if !is_power_peripheral(&p.name) {
            continue;
        }
        let items = peripheral_register_items(device, p);
        if items.is_empty() {
            continue;
        }
        let Some((tasks_constlat_name, _)) = find_register(items, "TASKS_CONSTLAT") else { continue; };
        let Some((tasks_lowpwr_name, _)) = find_register(items, "TASKS_LOWPWR") else { continue; };
        let Some((events_pofwarn_name, _)) = find_register(items, "EVENTS_POFWARN") else { continue; };
        let Some((events_sleepenter_name, _)) = find_register(items, "EVENTS_SLEEPENTER") else { continue; };
        let Some((events_sleepexit_name, _)) = find_register(items, "EVENTS_SLEEPEXIT") else { continue; };
        let Some((events_usbdetected_name, _)) = find_register(items, "EVENTS_USBDETECTED") else { continue; };
        let Some((events_usbremoved_name, _)) = find_register(items, "EVENTS_USBREMOVED") else { continue; };
        let Some((events_usbpwrrdy_name, _)) = find_register(items, "EVENTS_USBPWRRDY") else { continue; };
        let Some((intenset_name, _)) = find_register(items, "INTENSET") else { continue; };
        let Some((intenclr_name, _)) = find_register(items, "INTENCLR") else { continue; };

        out.push(PowerIr {
            periph_name: p.name.clone(),
            periph_mod: sanitize_module_name(&p.name),
            hal_mod: sanitize_field_name(&p.name),
            field_tasks_constlat: sanitize_field_name(&tasks_constlat_name),
            field_tasks_lowpwr: sanitize_field_name(&tasks_lowpwr_name),
            field_events_pofwarn: sanitize_field_name(&events_pofwarn_name),
            field_events_sleepenter: sanitize_field_name(&events_sleepenter_name),
            field_events_sleepexit: sanitize_field_name(&events_sleepexit_name),
            field_events_usbdetected: sanitize_field_name(&events_usbdetected_name),
            field_events_usbremoved: sanitize_field_name(&events_usbremoved_name),
            field_events_usbpwrrdy: sanitize_field_name(&events_usbpwrrdy_name),
            field_intenset: sanitize_field_name(&intenset_name),
            field_intenclr: sanitize_field_name(&intenclr_name),
        });
    }
    out
}

fn is_power_peripheral(name: &str) -> bool {
    name.to_ascii_uppercase().contains("POWER")
}

// ─── USB (delegates to existing module) ──────────────────────────

mod usb {
    use crate::svd;
    use crate::hal::common::*;
    use crate::hal::ir::UsbIr;
    use std::collections::BTreeSet;

    pub fn collect(device: &svd::Device) -> Vec<UsbIr> {
        let mut out = Vec::new();
        for p in &device.peripherals {
            if !is_usbd_like(&p.name) {
                continue;
            }
            let items = peripheral_register_items(device, p);
            if items.is_empty() {
                continue;
            }
            let Some((enable_name, _)) = find_register(items, "ENABLE") else { continue; };
            let Some((usb_pullup_name, _)) = find_register(items, "USBPULLUP") else { continue; };
            let Some((epinen_name, _)) = find_register(items, "EPINEN") else { continue; };
            let Some((epouten_name, _)) = find_register(items, "EPOUTEN") else { continue; };
            let Some((tasks_startein_name, _)) = find_register(items, "STARTEPIN") else { continue; };
            let Some((tasks_startepout_name, _)) = find_register(items, "STARTEPOUT") else { continue; };
            let Some((events_ep0setup_name, _)) = find_register(items, "EP0SETUP") else { continue; };
            let Some((events_ep0datadone_name, _)) = find_register(items, "EP0DATADONE") else { continue; };
            let Some((events_endepin_name, _)) = find_register(items, "ENDEPIN") else { continue; };
            let Some((events_endepout_name, _)) = find_register(items, "ENDEPOUT") else { continue; };
            let Some((events_usbreset_name, _)) = find_register(items, "USBRESET") else { continue; };
            let Some((events_usbevent_name, _)) = find_register(items, "USBEVENT") else { continue; };
            let Some((events_endepin_array_name, _)) = find_register(items, "ENDEPIN") else { continue; };
            let Some((events_endepout_array_name, _)) = find_register(items, "ENDEPOUT") else { continue; };
            let tasks_startein_field = sanitize_field_name(&tasks_startein_name);
            let tasks_startepout_field = sanitize_field_name(&tasks_startepout_name);

            let has_ep_arrays = items.iter().any(|it| {
                matches!(it, svd::RegisterBlockItem::Cluster { .. })
            });

            let epin_field = items.iter().find_map(|it| {
                if let svd::RegisterBlockItem::Cluster { cluster } = it {
                    if cluster.name.to_ascii_uppercase().contains("EPIN") {
                        return Some(sanitize_field_name(&cluster.name));
                    }
                }
                None
            });
            let epout_field = items.iter().find_map(|it| {
                if let svd::RegisterBlockItem::Cluster { cluster } = it {
                    if cluster.name.to_ascii_uppercase().contains("EPOUT") {
                        return Some(sanitize_field_name(&cluster.name));
                    }
                }
                None
            });

            out.push(UsbIr {
                periph_name: p.name.clone(),
                periph_mod: sanitize_module_name(&p.name),
                hal_mod: sanitize_field_name(&p.name),
                field_enable: sanitize_field_name(&enable_name),
                field_usb_pullup: sanitize_field_name(&usb_pullup_name),
                field_epinen: sanitize_field_name(&epinen_name),
                field_epouten: sanitize_field_name(&epouten_name),
                field_epin: epin_field,
                field_epout: epout_field,
                field_tasks_startein: tasks_startein_field.clone(),
                field_tasks_staroutep: sanitize_field_name(&tasks_startepout_name),
                field_events_ep0setup: sanitize_field_name(&events_ep0setup_name),
                field_events_ep0datadone: sanitize_field_name(&events_ep0datadone_name),
                field_events_endepin: sanitize_field_name(&events_endepin_name),
                field_events_endepout: sanitize_field_name(&events_endepout_name),
                field_events_usbreset: sanitize_field_name(&events_usbreset_name),
                field_events_usbevent: sanitize_field_name(&events_usbevent_name),
                field_tasks_startepin: tasks_startein_field,
                field_tasks_startepout: tasks_startepout_field,
                field_events_endepin_array: sanitize_field_name(&events_endepin_array_name),
                field_events_endepout_array: sanitize_field_name(&events_endepout_array_name),
                has_ep_arrays,
            });
        }
        out
    }

    fn is_usbd_like(name: &str) -> bool {
        let upper = name.to_ascii_uppercase();
        upper.starts_with("USBD")
    }
}
