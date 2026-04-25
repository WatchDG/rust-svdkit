use crate::{Result, svd};

#[derive(Debug, Clone)]
pub struct GpioPortInfo {
    periph_name: String,
    periph_mod: String,
    hal_mod: String,
    field_outset: String,
    field_outclr: String,
    field_out: String,
    field_pin_cnf: String,
    pin_cnf_reg_path: String,
    pin_fields: Vec<svd::Field>,
    level_enum_name: Option<String>,
}

impl GpioPortInfo {
    pub fn render(&self) -> Result<String> {
        let mut s = String::new();
        let port_ty = gpio_port_type_name(&self.periph_name);

        s.push_str(&format!("    pub mod {} {{\n", self.hal_mod));
        s.push_str("        use super::pac;\n\n");
        s.push_str(&format!(
            "        pub type {port_ty} = pac::peripherals::{}::RegisterBlock;\n\n",
            self.periph_mod,
        ));
        s.push_str("        use core::marker::PhantomData;\n\n");

        s.push_str("        pub trait PinState {}\n");
        s.push_str("        pub struct Unconfigured;\n\n");
        s.push_str("        impl PinState for Unconfigured {}\n\n");

        s.push_str("        #[repr(u8)]\n");
        s.push_str("        #[derive(Copy, Clone, Debug, PartialEq, Eq)]\n");
        s.push_str("        pub enum Level {\n");
        s.push_str("            Low = 0,\n");
        s.push_str("            High = 1,\n");
        s.push_str("        }\n\n");

        s.push_str("        #[inline(always)]\n");
        s.push_str(&format!(
            "        pub unsafe fn steal() -> &'static {port_ty} {{\n"
        ));
        s.push_str(&format!(
            "            &*pac::peripherals::{}::PTR\n",
            self.periph_mod
        ));
        s.push_str("        }\n\n");

        let field_enums = [
            ("DIR", "Dir", String::new()),
            ("DRIVE", "Drive", String::new()),
            ("SENSE", "Sense", String::new()),
            ("PULL", "Pull", String::new()),
        ];

        let mut generated_any = false;
        for (fname, alias, extra) in field_enums {
            let f = match self
                .pin_fields
                .iter()
                .find(|f| f.name.eq_ignore_ascii_case(fname))
            {
                Some(f) => f,
                None => continue,
            };

            let pac_enum_ty =
                pac_enum_type_name_for_field(&self.periph_name, &self.pin_cnf_reg_path, f);
            if let Some(ty) = &pac_enum_ty {
                s.push_str(&format!(
                    "        pub use super::super::pac::peripherals::{}::enums::{} as {};\n",
                    self.periph_mod, ty, alias
                ));
                generated_any = true;
            } else if let Some(e) = render_field_enum(fname, f) {
                s.push_str(&indent_block(&e, 8));
                if !extra.is_empty() {
                    s.push_str(&indent_block(&extra, 8));
                }
                generated_any = true;
            }
        }
        if generated_any {
            s.push('\n');
        }

        let dir_field = self
            .pin_fields
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case("DIR"));
        let pull_field = self
            .pin_fields
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case("PULL"));
        let drive_field = self
            .pin_fields
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case("DRIVE"));

        let dir_info = dir_field.map(|f| {
            let (lsb, width) = field_lsb_width(f);
            let mask: u32 = (1u32 << width) - 1;
            let out_val = infer_output_value(f).unwrap_or(1);
            (lsb, mask, out_val)
        });
        let pull_info = pull_field.map(|f| {
            let (lsb, width) = field_lsb_width(f);
            let mask: u32 = (1u32 << width) - 1;
            (lsb, mask)
        });
        let drive_info = drive_field.map(|f| {
            let (lsb, width) = field_lsb_width(f);
            let mask: u32 = (1u32 << width) - 1;
            (lsb, mask)
        });
        let sense_info = self
            .pin_fields
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case("SENSE"))
            .map(|f| {
                let (lsb, width) = field_lsb_width(f);
                let mask: u32 = (1u32 << width) - 1;
                (lsb, mask)
            });

        s.push_str(&format!(
            "        pub struct Pin<'a, S: PinState> {{\n            port: &'a {port_ty},\n            pin: u8,\n            _state: PhantomData<S>,\n        }}\n\n"
        ));

        s.push_str("        #[inline(always)]\n");
        s.push_str(&format!(
            "        pub unsafe fn pin(pin: u8) -> Pin<'static, Unconfigured> {{\n"
        ));
        s.push_str(&format!(
            "            Pin {{ port: &*pac::peripherals::{}::PTR, pin, _state: PhantomData }}\n",
            self.periph_mod
        ));
        s.push_str("        }\n\n");

        s.push_str("        pub struct PinConfigurator<'a> {\n");
        s.push_str("            port: &'a ");
        s.push_str(&port_ty);
        s.push_str(",\n");
        s.push_str("            pin: u8,\n");
        s.push_str("            cnf: u32,\n");
        s.push_str("            output: bool,\n");
        s.push_str("        }\n\n");

        s.push_str("        impl<'a> Pin<'a, Unconfigured> {\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn configure(self) -> PinConfigurator<'a> {\n");
        s.push_str(&format!(
            "                let cnf = self.port.{}[self.pin as usize].read();\n",
            self.field_pin_cnf
        ));
        s.push_str("                PinConfigurator { port: self.port, pin: self.pin, cnf, output: false }\n");
        s.push_str("            }\n");
        s.push_str("        }\n\n");

        s.push_str("        impl<'a> PinConfigurator<'a> {\n");

        if let Some((lsb, mask, out_val)) = dir_info {
            s.push_str("            #[inline(always)]\n");
            s.push_str("            pub fn dir(self, v: Dir) -> Self {\n");
            s.push_str(&format!(
                "                let output = (v as u32) == {out_val}u32;\n"
            ));
            if lsb == 0 {
                s.push_str(&format!(
                    "                let cnf = (self.cnf & !0x{mask:X}u32) | (((v as u32) & 0x{mask:X}u32));\n"
                ));
            } else {
                s.push_str(&format!(
                    "                let cnf = (self.cnf & !(0x{mask:X}u32 << {lsb})) | (((v as u32) & 0x{mask:X}u32) << {lsb});\n"
                ));
            }
            s.push_str(
                "                PinConfigurator { port: self.port, pin: self.pin, cnf, output }\n",
            );
            s.push_str("            }\n\n");
        }

        if let Some((lsb, mask)) = pull_info {
            s.push_str("            #[inline(always)]\n");
            s.push_str("            pub fn pull(self, v: Pull) -> Self {\n");
            if lsb == 0 {
                s.push_str(&format!(
                    "                let cnf = (self.cnf & !0x{mask:X}u32) | (((v as u32) & 0x{mask:X}u32));\n"
                ));
            } else {
                s.push_str(&format!(
                    "                let cnf = (self.cnf & !(0x{mask:X}u32 << {lsb})) | (((v as u32) & 0x{mask:X}u32) << {lsb});\n"
                ));
            }
            s.push_str("                PinConfigurator { port: self.port, pin: self.pin, cnf, output: self.output }\n");
            s.push_str("            }\n\n");
        }

        if let Some((lsb, mask)) = drive_info {
            s.push_str("            #[inline(always)]\n");
            s.push_str("            pub fn drive(self, v: Drive) -> Self {\n");
            if lsb == 0 {
                s.push_str(&format!(
                    "                let cnf = (self.cnf & !0x{mask:X}u32) | (((v as u32) & 0x{mask:X}u32));\n"
                ));
            } else {
                s.push_str(&format!(
                    "                let cnf = (self.cnf & !(0x{mask:X}u32 << {lsb})) | (((v as u32) & 0x{mask:X}u32) << {lsb});\n"
                ));
            }
            s.push_str("                PinConfigurator { port: self.port, pin: self.pin, cnf, output: self.output }\n");
            s.push_str("            }\n\n");
        }

        if let Some((lsb, mask)) = sense_info {
            s.push_str("            #[inline(always)]\n");
            s.push_str("            pub fn sense(self, v: Sense) -> Self {\n");
            if lsb == 0 {
                s.push_str(&format!(
                    "                let cnf = (self.cnf & !0x{mask:X}u32) | (((v as u32) & 0x{mask:X}u32));\n"
                ));
            } else {
                s.push_str(&format!(
                    "                let cnf = (self.cnf & !(0x{mask:X}u32 << {lsb})) | (((v as u32) & 0x{mask:X}u32) << {lsb});\n"
                ));
            }
            s.push_str("                PinConfigurator { port: self.port, pin: self.pin, cnf, output: self.output }\n");
            s.push_str("            }\n\n");
        }

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn apply(self) -> PinConfigured<'a> {\n");
        s.push_str(&format!(
            "                self.port.{}[self.pin as usize].write(self.cnf);\n",
            self.field_pin_cnf
        ));
        s.push_str("                if self.output {\n");
        s.push_str("                    PinConfigured::Output(PinOutput { port: self.port, pin: self.pin })\n");
        s.push_str("                } else {\n");
        s.push_str("                    PinConfigured::Input(PinInput { port: self.port, pin: self.pin })\n");
        s.push_str("                }\n");
        s.push_str("            }\n");
        s.push_str("        }\n\n");

        s.push_str("        pub enum PinConfigured<'a> {\n");
        s.push_str("            Input(PinInput<'a>),\n");
        s.push_str("            Output(PinOutput<'a>),\n");
        s.push_str("        }\n\n");

        s.push_str("        pub struct PinInput<'a> {\n");
        s.push_str("            port: &'a ");
        s.push_str(&port_ty);
        s.push_str(",\n");
        s.push_str("            pin: u8,\n");
        s.push_str("        }\n\n");
        s.push_str("        impl<'a> PinInput<'a> {\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn reconfigure(self) -> PinConfigurator<'a> {\n");
        s.push_str(&format!(
            "                let cnf = self.port.{}[self.pin as usize].read();\n",
            self.field_pin_cnf
        ));
        s.push_str("                PinConfigurator { port: self.port, pin: self.pin, cnf, output: false }\n");
        s.push_str("            }\n");
        s.push_str("        }\n\n");

        s.push_str("        pub struct PinOutput<'a> {\n");
        s.push_str("            port: &'a ");
        s.push_str(&port_ty);
        s.push_str(",\n");
        s.push_str("            pin: u8,\n");
        s.push_str("        }\n\n");
        s.push_str("        impl<'a> PinOutput<'a> {\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn reconfigure(self) -> PinConfigurator<'a> {\n");
        s.push_str(&format!(
            "                let cnf = self.port.{}[self.pin as usize].read();\n",
            self.field_pin_cnf
        ));
        s.push_str("                PinConfigurator { port: self.port, pin: self.pin, cnf, output: true }\n");
        s.push_str("            }\n\n");

        if let Some(ref level_enum) = self.level_enum_name {
            s.push_str("            #[inline(always)]\n");
            s.push_str(&format!("            pub fn set_level(&self, level: super::super::pac::peripherals::{}::enums::{level_enum}) {{\n", self.periph_mod));
            s.push_str(&format!(
                "                let mask = 1u32 << (self.pin as u32);\n"
            ));
            s.push_str(&format!(
                "                let new_value = (self.port.{}.read() & !mask) | ((level as u32) << (self.pin as u32));\n",
                self.field_out
            ));
            s.push_str(&format!(
                "                self.port.{}.write(new_value);\n",
                self.field_out
            ));
            s.push_str("            }\n\n");

            s.push_str("            #[inline(always)]\n");
            s.push_str("            pub fn set_high(&self) {\n");
            s.push_str(&format!(
                "                self.set_level(super::super::pac::peripherals::{}::enums::{level_enum}::High);\n",
                self.periph_mod
            ));
            s.push_str("            }\n\n");

            s.push_str("            #[inline(always)]\n");
            s.push_str("            pub fn set_low(&self) {\n");
            s.push_str(&format!(
                "                self.set_level(super::super::pac::peripherals::{}::enums::{level_enum}::Low);\n",
                self.periph_mod
            ));
            s.push_str("            }\n");
        } else {
            s.push_str("            #[inline(always)]\n");
            s.push_str("            pub fn set_high(&self) {\n");
            s.push_str(&format!(
                "                self.port.{}.write(1u32 << (self.pin as u32));\n",
                self.field_outset
            ));
            s.push_str("            }\n\n");
            s.push_str("            #[inline(always)]\n");
            s.push_str("            pub fn set_low(&self) {\n");
            s.push_str(&format!(
                "                self.port.{}.write(1u32 << (self.pin as u32));\n",
                self.field_outclr
            ));
            s.push_str("            }\n");
        }
        s.push_str("        }\n");

        s.push_str("    }\n");
        Ok(s)
    }
}

pub fn collect_gpio_ports(device: &svd::Device) -> Vec<GpioPortInfo> {
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

        out.push(GpioPortInfo {
            periph_name: p.name.clone(),
            periph_mod: sanitize_module_name(&p.name),
            hal_mod: sanitize_field_name(&p.name),
            field_outset: sanitize_field_name(&outset_name),
            field_outclr: sanitize_field_name(&outclr_name),
            field_out: sanitize_field_name(&out_name),
            field_pin_cnf: sanitize_field_name(&pin_cnf_name),
            pin_cnf_reg_path: pin_cnf_reg.name.clone(),
            pin_fields: pin_cnf_reg.field.clone(),
            level_enum_name,
        });
    }
    out
}

fn find_level_enum_for_out_reg(p: &svd::Peripheral, out_reg: &svd::Register) -> Option<String> {
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
        Some(pac_enum_type_name_for_field(
            &p.name,
            &out_reg.name,
            pin0_field,
        )?)
    } else {
        None
    }
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

fn gpio_port_type_name(periph_name: &str) -> String {
    let b = periph_name.as_bytes();
    if b.len() >= 2 && (b[0] == b'P' || b[0] == b'p') && b[1..].iter().all(|c| c.is_ascii_digit()) {
        format!("Port{}", &periph_name[1..])
    } else {
        "Port".to_string()
    }
}

fn pac_enum_type_name_for_field(
    periph_name: &str,
    reg_path: &str,
    f: &svd::Field,
) -> Option<String> {
    let evs = f.enumerated_values.first()?;
    let has_numeric = evs.enumerated_value.iter().any(|v| match &v.spec {
        svd::EnumeratedValueSpec::Value { value } => parse_enum_u64(value).is_some(),
        svd::EnumeratedValueSpec::IsDefault { .. } => false,
    });
    if !has_numeric {
        return None;
    }

    let field_name_for_enum = extract_enum_base_name(&f.name);
    let base = evs
        .header_enum_name
        .as_deref()
        .or(evs.name.as_deref())
        .map(sanitize_type_name)
        .unwrap_or_else(|| {
            sanitize_type_name(&format!(
                "{}_{}_{}",
                periph_name,
                reg_path.replace('.', "_"),
                field_name_for_enum
            ))
        });
    Some(base)
}

fn extract_enum_base_name(field_name: &str) -> String {
    let upper = field_name.to_ascii_uppercase();

    for i in (1..upper.len()).rev() {
        if upper
            .chars()
            .nth(i)
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            if i > 0
                && upper
                    .chars()
                    .nth(i - 1)
                    .map(|c| c.is_ascii_alphabetic())
                    .unwrap_or(false)
            {
                return field_name[..i].to_string();
            }
        }
    }

    field_name.to_string()
}

fn render_field_enum(field_name: &str, f: &svd::Field) -> Option<String> {
    let evs = f.enumerated_values.first()?;
    let mut vars: Vec<(String, u64)> = Vec::new();
    for v in &evs.enumerated_value {
        let Some(val) = (match &v.spec {
            svd::EnumeratedValueSpec::Value { value } => parse_enum_u64(value),
            svd::EnumeratedValueSpec::IsDefault { .. } => None,
        }) else {
            continue;
        };
        vars.push((sanitize_variant_name(&v.name), val));
    }
    if vars.is_empty() {
        return None;
    }

    let ty = sanitize_type_name(field_name);
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

fn infer_output_value(f: &svd::Field) -> Option<u32> {
    let evs = f.enumerated_values.first()?;
    let mut output: Option<u32> = None;
    for v in &evs.enumerated_value {
        let Some(val) = (match &v.spec {
            svd::EnumeratedValueSpec::Value { value } => parse_enum_u64(value),
            svd::EnumeratedValueSpec::IsDefault { .. } => None,
        }) else {
            continue;
        };
        let name = v.name.to_ascii_lowercase();
        if output.is_none()
            && (name.contains("output") || name.contains("output_1") || name.ends_with('1'))
        {
            output = Some(val as u32);
        }
    }
    output
}

pub fn peripheral_register_items<'a>(
    device: &'a svd::Device,
    p: &'a svd::Peripheral,
) -> &'a [svd::RegisterBlockItem] {
    let mut cur = p;
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    loop {
        if let Some(rb) = cur.registers.as_ref() {
            return rb.items.as_slice();
        }
        let Some(df) = cur.derived_from.as_deref() else {
            return &[];
        };
        if !seen.insert(cur.name.clone()) {
            return &[];
        }
        let Some(next) = device.peripherals.iter().find(|pp| pp.name == df) else {
            return &[];
        };
        cur = next;
    }
}

pub fn find_register<'a>(
    items: &'a [svd::RegisterBlockItem],
    needle: &str,
) -> Option<(String, &'a svd::Register)> {
    for it in items {
        match it {
            svd::RegisterBlockItem::Register { register } => {
                if register.name.to_ascii_uppercase().contains(needle) {
                    return Some((register.name.clone(), register));
                }
            }
            svd::RegisterBlockItem::Cluster { cluster } => {
                if let Some(x) = find_register(cluster.items.as_slice(), needle) {
                    return Some(x);
                }
            }
        }
    }
    None
}

pub fn find_register_prefer_exact<'a>(
    items: &'a [svd::RegisterBlockItem],
    needle: &str,
) -> Option<(String, &'a svd::Register)> {
    let needle_upper = needle.to_ascii_uppercase();
    for it in items {
        match it {
            svd::RegisterBlockItem::Register { register } => {
                if register.name.to_ascii_uppercase() == needle_upper {
                    return Some((register.name.clone(), register));
                }
            }
            svd::RegisterBlockItem::Cluster { cluster } => {
                if let Some(x) = find_register_prefer_exact(cluster.items.as_slice(), needle) {
                    return Some(x);
                }
            }
        }
    }
    find_register(items, needle)
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

pub fn parse_enum_u64(s: &str) -> Option<u64> {
    let s = s.trim();
    let s = s.strip_prefix('+').unwrap_or(s);
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        let digits = hex.trim();
        if digits.is_empty() || digits.contains('x') || digits.contains('X') {
            return None;
        }
        u64::from_str_radix(digits, 16).ok()
    } else if let Some(bin) = s.strip_prefix("0b") {
        let digits = bin.trim();
        if digits.is_empty() || digits.contains('x') || digits.contains('X') {
            return None;
        }
        u64::from_str_radix(digits, 2).ok()
    } else if let Some(bin) = s.strip_prefix('#') {
        let digits = bin.trim();
        if digits.is_empty() || digits.contains('x') || digits.contains('X') {
            return None;
        }
        u64::from_str_radix(digits, 2).ok()
    } else if s.chars().all(|c| c.is_ascii_digit()) {
        s.parse::<u64>().ok()
    } else {
        None
    }
}

pub fn sanitize_type_name(s: &str) -> String {
    let s = s.replace("[%s]", "").replace("%s", "");
    let mut out = String::new();
    let mut upper_next = true;
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            if out.is_empty() && ch.is_ascii_digit() {
                out.push('_');
            }
            if upper_next {
                out.push(ch.to_ascii_uppercase());
                upper_next = false;
            } else {
                out.push(ch.to_ascii_lowercase());
            }
        } else {
            upper_next = true;
        }
    }
    if out.is_empty() {
        "Type".to_string()
    } else if is_rust_keyword(&out.to_ascii_lowercase()) {
        format!("{out}_")
    } else {
        out.replace("Ldetect", "LDetect")
            .replace("Dirclr", "DirClr")
            .replace("Dirset", "DirSet")
            .replace("Outclr", "OutClr")
            .replace("Outset", "OutSet")
            .replace("Notlatched", "NotLatched")
            .replace("Pulldown", "PullDown")
            .replace("Pullup", "PullUp")
            .replace("DetectmodeDetectmode", "DetectMode")
            .replace("Detectmode", "DetectMode")
    }
}

pub fn sanitize_variant_name(s: &str) -> String {
    let mut out = String::new();
    let mut upper_next = true;
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            if out.is_empty() && ch.is_ascii_digit() {
                out.push('_');
            }
            if upper_next {
                out.push(ch.to_ascii_uppercase());
                upper_next = false;
            } else {
                out.push(ch.to_ascii_lowercase());
            }
        } else {
            upper_next = true;
        }
    }
    if out.is_empty() {
        "Value".to_string()
    } else if is_rust_keyword(&out.to_ascii_lowercase()) {
        format!("{out}_")
    } else {
        out.replace("Ldetect", "LDetect")
            .replace("Dirclr", "DirClr")
            .replace("Dirset", "DirSet")
            .replace("Outclr", "OutClr")
            .replace("Outset", "OutSet")
            .replace("Notlatched", "NotLatched")
            .replace("Pulldown", "PullDown")
            .replace("Pullup", "PullUp")
    }
}

fn is_rust_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
    )
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

pub fn sanitize_module_name(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        let good = ch.is_ascii_alphanumeric() || ch == '_';
        let ch = if good { ch } else { '_' };
        if i == 0 && ch.is_ascii_digit() {
            out.push('_');
        }
        out.push(ch);
    }
    if out.is_empty() {
        "periph".to_string()
    } else if is_rust_keyword(&out.to_ascii_lowercase()) {
        format!("{out}_")
    } else {
        out.to_ascii_lowercase()
    }
}

pub fn sanitize_field_name(s: &str) -> String {
    let s = s.replace("[%s]", "").replace("%s", "");
    let s_lower = s.to_ascii_lowercase();
    let s = if s_lower.contains("outset") {
        s.replace("OUTSET", "out_set").replace("outset", "out_set")
    } else if s_lower.contains("outclr") {
        s.replace("OUTCLR", "out_clr").replace("outclr", "out_clr")
    } else if s_lower.contains("dirset") {
        s.replace("DIRSET", "dir_set").replace("dirset", "dir_set")
    } else if s_lower.contains("dirclr") {
        s.replace("DIRCLR", "dir_clr").replace("dirclr", "dir_clr")
    } else if s_lower.contains("detectmode") {
        s.replace("DETECTMODE", "detect_mode")
            .replace("detectmode", "detect_mode")
    } else {
        s
    };
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        let good = ch.is_ascii_alphanumeric() || ch == '_';
        let ch = if good { ch } else { '_' };
        if i == 0 && ch.is_ascii_digit() {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    if out.is_empty() {
        "_field".to_string()
    } else if is_rust_keyword(&out) {
        format!("{out}_")
    } else {
        out
    }
}
