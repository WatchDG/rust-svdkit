use crate::svd;
use std::collections::BTreeSet;

pub fn field_lsb_width(f: &svd::Field) -> (u32, u32) {
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
            .replace("Notgenerated", "NotGenerated")
            .replace("Pulldown", "PullDown")
            .replace("Pullup", "PullUp")
            .replace("DetectmodeDetectmode", "DetectMode")
            .replace("Detectmode", "DetectMode")
            .replace("BitmodeBitmode", "Bitmode")
            .replace("EventsCompareEventsCompare", "EventsCompare")
            .replace("TasksStopTasksStop", "TasksStop")
            .replace("TasksStartTasksStart", "TasksStart")
            .replace("TasksClearTasksClear", "TasksClear")
            .replace("TasksCaptureTasksCapture", "TasksCapture")
            .replace("TasksShutdownTasksShutdown", "TasksShutdown")
            .replace("TasksCountTasksCount", "TasksCount")
            .replace("Notstarted", "NotStarted")
            .replace("Notdetected", "NotDetected")
            .replace("Nodata", "NoData")
            .replace("Datadone", "DataDone")
            .replace("Notallowed", "NotAllowed")
            .replace("ModeMode", "Mode")
            .replace("ShortsShorts", "Shorts")
            .replace("PrescalerPrescaler", "Prescaler")
            .replace("Lowpowercounter", "LowPowerCounter")
            .replace("EventsTimeoutEventsTimeout", "EventsTimeout")
            .replace("EventsRzEventsRz", "EventsRz")
            .replace("TasksRzTasksRz", "TasksRz")
            .replace("TasksReloadTasksReload", "TasksReload")
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

pub fn sanitize_file_stem(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "device".to_string()
    } else {
        out
    }
}

pub fn indent_block(s: &str, spaces: usize) -> String {
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

pub fn is_rust_keyword(s: &str) -> bool {
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

pub fn extract_enum_base_name(field_name: &str) -> String {
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

pub fn pac_enum_type_name_for_field(
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
                "{}_{}",
                reg_path.replace('.', "_"),
                field_name_for_enum
            ))
        });
    Some(base)
}

pub fn render_field_enum(field_name: &str, f: &svd::Field) -> Option<String> {
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

pub fn infer_output_value(f: &svd::Field) -> Option<u32> {
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
    let mut seen: BTreeSet<String> = BTreeSet::new();
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
