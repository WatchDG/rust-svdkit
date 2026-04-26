use crate::svd;

pub fn field_bit_width(f: &svd::Field) -> u32 {
    match f.bit_range {
        svd::BitRange::BitRangeString { msb, lsb } => msb.saturating_sub(lsb) + 1,
        svd::BitRange::LsbMsb { lsb, msb } => msb.saturating_sub(lsb) + 1,
        svd::BitRange::BitOffsetWidth { bit_width, .. } => bit_width.unwrap_or(1),
    }
}

pub fn repr_for_bits(bits: u32) -> &'static str {
    match bits {
        0..=8 => "u8",
        9..=16 => "u16",
        17..=32 => "u32",
        _ => "u64",
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

pub fn sanitize_irq_handler_name(s: &str) -> String {
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
        "IRQ".to_string()
    } else if is_rust_keyword(&out.to_ascii_lowercase()) {
        format!("{out}_")
    } else {
        out
    }
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
