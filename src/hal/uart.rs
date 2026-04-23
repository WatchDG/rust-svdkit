use super::gpio;
use crate::{Result, svd};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct UartInfo {
    periph_name: String,
    periph_mod: String,
    hal_mod: String,

    field_tasks_startrx: String,
    field_tasks_stoprx: String,
    field_tasks_starttx: String,
    field_tasks_stoptx: String,
    field_tasks_flushrx: String,

    field_events_rxrdy: String,
    field_events_txrdy: String,
    field_events_endrx: String,
    field_events_endtx: String,
    field_events_error: String,

    field_intenset: String,

    field_baudrate: String,
    baudrate_field: svd::Field,

    field_config: String,
    config_hwfc_field: Option<svd::Field>,
    config_parity_field: Option<svd::Field>,
}

impl UartInfo {
    pub fn render(&self) -> Result<String> {
        let mut s = String::new();
        let uart_ty = sanitize_type_name(&self.periph_mod);

        s.push_str(&format!("    pub mod {} {{\n", self.hal_mod));
        s.push_str("        use super::pac;\n\n");
        s.push_str(&format!(
            "        pub type {uart_ty} = pac::{}::RegisterBlock;\n\n",
            self.periph_mod,
        ));

        s.push_str("        use core::marker::PhantomData;\n\n");

        s.push_str("        pub trait UartState {}\n");
        s.push_str("        pub struct Unconfigured;\n");
        s.push_str("        pub struct Configured;\n");
        s.push_str("        pub struct Idle;\n");
        s.push_str("        pub struct Receiving;\n");
        s.push_str("        pub struct Transmitting;\n\n");
        s.push_str("        impl UartState for Unconfigured {}\n");
        s.push_str("        impl UartState for Configured {}\n");
        s.push_str("        impl UartState for Idle {}\n");
        s.push_str("        impl UartState for Receiving {}\n");
        s.push_str("        impl UartState for Transmitting {}\n\n");

        s.push_str("        #[inline(always)]\n");
        s.push_str(&format!(
            "        pub unsafe fn steal() -> &'static {uart_ty} {{\n"
        ));
        s.push_str(&format!("            &*pac::{}::PTR\n", self.periph_mod));
        s.push_str("        }\n\n");

        s.push_str(&format!(
            "        pub fn uart() -> Uart<'static, Unconfigured> {{\n"
        ));
        s.push_str(&format!(
            "            Uart {{ u: unsafe {{ &*pac::{}::PTR }}, _state: PhantomData }}\n",
            self.periph_mod
        ));
        s.push_str("        }\n\n");

        let baudrate_alias = sanitize_type_name("BAUDRATE");
        let pac_baudrate_ty =
            pac_enum_type_name_for_field(&self.periph_name, "BAUDRATE", &self.baudrate_field);
        if let Some(ty) = &pac_baudrate_ty {
            s.push_str(&indent_block(
                &format!(
                    "pub use super::super::pac::{}::enums::{ty} as {baudrate_alias};\n",
                    self.periph_mod
                ),
                8,
            ));
            s.push('\n');
        } else if let Some(src) = render_field_enum("BAUDRATE", &self.baudrate_field) {
            s.push_str(&indent_block(&src, 8));
            s.push('\n');
        }

        if let Some(ref hwfc_f) = self.config_hwfc_field {
            let hwfc_alias = sanitize_type_name("Hwfc");
            let pac_hwfc_ty = pac_enum_type_name_for_field(&self.periph_name, "CONFIG", hwfc_f);
            if let Some(ty) = &pac_hwfc_ty {
                s.push_str(&indent_block(
                    &format!(
                        "pub use super::super::pac::{}::enums::{ty} as {hwfc_alias};\n",
                        self.periph_mod
                    ),
                    8,
                ));
                s.push('\n');
            } else if let Some(src) = render_field_enum("HWFC", hwfc_f) {
                s.push_str(&indent_block(&src, 8));
                s.push('\n');
            }
        }

        if let Some(ref parity_f) = self.config_parity_field {
            let parity_alias = sanitize_type_name("Parity");
            let pac_parity_ty = pac_enum_type_name_for_field(&self.periph_name, "CONFIG", parity_f);
            if let Some(ty) = &pac_parity_ty {
                s.push_str(&indent_block(
                    &format!(
                        "pub use super::super::pac::{}::enums::{ty} as {parity_alias};\n",
                        self.periph_mod
                    ),
                    8,
                ));
                s.push('\n');
            } else if let Some(src) = render_field_enum("PARITY", parity_f) {
                s.push_str(&indent_block(&src, 8));
                s.push('\n');
            }
        }

        s.push_str(&format!("        pub struct Uart<'a, S: UartState> {{\n"));
        s.push_str(&format!("            u: &'a {uart_ty},\n"));
        s.push_str("            _state: PhantomData<S>,\n");
        s.push_str("        }\n\n");

        s.push_str(&format!("        impl<'a> Uart<'a, Unconfigured> {{\n"));
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn configure(self) -> UartConfigurator<'a> {\n");
        s.push_str("                UartConfigurator {\n");
        s.push_str("                    u: self.u,\n");
        s.push_str("                    baudrate_val: None,\n");
        s.push_str("                    hwfc_val: None,\n");
        s.push_str("                    parity_val: None,\n");
        s.push_str("                }\n");
        s.push_str("            }\n");
        s.push_str("        }\n\n");

        s.push_str("        pub struct UartConfigurator<'a> {\n");
        s.push_str(&format!("            u: &'a {uart_ty},\n"));
        s.push_str("            baudrate_val: Option<u32>,\n");
        s.push_str("            hwfc_val: Option<u32>,\n");
        s.push_str("            parity_val: Option<u32>,\n");
        s.push_str("        }\n\n");

        s.push_str("        impl<'a> UartConfigurator<'a> {\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str(&format!(
            "            pub fn baudrate(self, v: {baudrate_alias}) -> Self {{\n"
        ));
        s.push_str("                UartConfigurator {\n");
        s.push_str("                    u: self.u,\n");
        s.push_str("                    baudrate_val: Some(v as u32),\n");
        s.push_str("                    hwfc_val: self.hwfc_val,\n");
        s.push_str("                    parity_val: self.parity_val,\n");
        s.push_str("                }\n");
        s.push_str("            }\n\n");

        if self.config_hwfc_field.is_some() {
            s.push_str("            #[inline(always)]\n");
            s.push_str("            pub fn hardware_flow_control(self, v: Hwfc) -> Self {\n");
            s.push_str("                UartConfigurator {\n");
            s.push_str("                    u: self.u,\n");
            s.push_str("                    baudrate_val: self.baudrate_val,\n");
            s.push_str("                    hwfc_val: Some(v as u32),\n");
            s.push_str("                    parity_val: self.parity_val,\n");
            s.push_str("                }\n");
            s.push_str("            }\n\n");
        }

        if self.config_parity_field.is_some() {
            s.push_str("            #[inline(always)]\n");
            s.push_str("            pub fn parity(self, v: Parity) -> Self {\n");
            s.push_str("                UartConfigurator {\n");
            s.push_str("                    u: self.u,\n");
            s.push_str("                    baudrate_val: self.baudrate_val,\n");
            s.push_str("                    hwfc_val: self.hwfc_val,\n");
            s.push_str("                    parity_val: Some(v as u32),\n");
            s.push_str("                }\n");
            s.push_str("            }\n\n");
        }

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn apply(self) -> Uart<'a, Idle> {\n");
        s.push_str(&format!(
            "                if let Some(v) = self.baudrate_val {{\n"
        ));
        s.push_str(&format!(
            "                    self.u.{}.write(v);\n",
            self.field_baudrate
        ));
        s.push_str("                }\n\n");
        s.push_str("                {\n");
        s.push_str(&format!(
            "                    let mut config: u32 = self.u.{}.read();\n",
            self.field_config
        ));
        if let Some(ref hwfc_f) = self.config_hwfc_field {
            let (lsb, width) = field_lsb_width(hwfc_f);
            let mask: u32 = ((1u32 << width) - 1) << lsb;
            s.push_str(&format!(
                "                    if let Some(v) = self.hwfc_val {{\n",
            ));
            s.push_str(&format!(
                "                        config = (config & !0x{mask:X}u32) | (v << {lsb});\n",
            ));
            s.push_str("                    }\n");
        }
        if let Some(ref parity_f) = self.config_parity_field {
            let (lsb, width) = field_lsb_width(parity_f);
            let mask: u32 = ((1u32 << width) - 1) << lsb;
            s.push_str(&format!(
                "                    if let Some(v) = self.parity_val {{\n",
            ));
            s.push_str(&format!(
                "                        config = (config & !0x{mask:X}u32) | (v << {lsb});\n",
            ));
            s.push_str("                    }\n");
        }
        s.push_str(&format!(
            "                    self.u.{}.write(config);\n",
            self.field_config
        ));
        s.push_str("                }\n\n");
        s.push_str(&format!(
            "                Uart {{ u: self.u, _state: PhantomData }}\n"
        ));
        s.push_str("            }\n");
        s.push_str("        }\n\n");

        s.push_str("        impl<'a> Uart<'a, Idle> {\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn enable_rx_ready_interrupt(&self) {\n");
        s.push_str(&format!(
            "                let cur = self.u.{}.read();\n",
            self.field_intenset
        ));
        s.push_str(&format!(
            "                self.u.{}.write(cur | (1u32 << 2));\n",
            self.field_intenset
        ));
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn enable_tx_ready_interrupt(&self) {\n");
        s.push_str(&format!(
            "                let cur = self.u.{}.read();\n",
            self.field_intenset
        ));
        s.push_str(&format!(
            "                self.u.{}.write(cur | (1u32 << 7));\n",
            self.field_intenset
        ));
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn enable_end_rx_interrupt(&self) {\n");
        s.push_str(&format!(
            "                let cur = self.u.{}.read();\n",
            self.field_intenset
        ));
        s.push_str(&format!(
            "                self.u.{}.write(cur | (1u32 << 4));\n",
            self.field_intenset
        ));
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn enable_end_tx_interrupt(&self) {\n");
        s.push_str(&format!(
            "                let cur = self.u.{}.read();\n",
            self.field_intenset
        ));
        s.push_str(&format!(
            "                self.u.{}.write(cur | (1u32 << 8));\n",
            self.field_intenset
        ));
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn enable_error_interrupt(&self) {\n");
        s.push_str(&format!(
            "                let cur = self.u.{}.read();\n",
            self.field_intenset
        ));
        s.push_str(&format!(
            "                self.u.{}.write(cur | (1u32 << 9));\n",
            self.field_intenset
        ));
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn start_rx(self) -> UartRx<'a> {\n");
        s.push_str(&format!(
            "                self.u.{}.write(1);\n",
            self.field_tasks_startrx
        ));
        s.push_str(&format!(
            "                UartRx {{ u: self.u, _state: PhantomData }}\n"
        ));
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn start_tx(self) -> UartTx<'a> {\n");
        s.push_str(&format!(
            "                self.u.{}.write(1);\n",
            self.field_tasks_starttx
        ));
        s.push_str(&format!(
            "                UartTx {{ u: self.u, _state: PhantomData }}\n"
        ));
        s.push_str("            }\n");
        s.push_str("        }\n\n");

        s.push_str(&format!("        pub struct UartRx<'a> {{\n"));
        s.push_str(&format!("            u: &'a {uart_ty},\n"));
        s.push_str("            _state: PhantomData<Receiving>,\n");
        s.push_str("        }\n\n");

        s.push_str("        impl<'a> UartRx<'a> {\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn is_rx_ready(&self) -> bool {\n");
        s.push_str(&format!(
            "                self.u.{}.read() != 0\n",
            self.field_events_rxrdy
        ));
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn is_rx_complete(&self) -> bool {\n");
        s.push_str(&format!(
            "                self.u.{}.read() != 0\n",
            self.field_events_endrx
        ));
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn clear_rx_ready(&self) {\n");
        s.push_str(&format!(
            "                self.u.{}.write(0);\n",
            self.field_events_rxrdy
        ));
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn clear_rx_complete(&self) {\n");
        s.push_str(&format!(
            "                self.u.{}.write(0);\n",
            self.field_events_endrx
        ));
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn stop(self) -> Uart<'a, Idle> {\n");
        s.push_str(&format!(
            "                self.u.{}.write(1);\n",
            self.field_tasks_stoprx
        ));
        s.push_str(&format!(
            "                Uart {{ u: self.u, _state: PhantomData }}\n"
        ));
        s.push_str("            }\n");
        s.push_str("        }\n\n");

        s.push_str(&format!("        pub struct UartTx<'a> {{\n"));
        s.push_str(&format!("            u: &'a {uart_ty},\n"));
        s.push_str("            _state: PhantomData<Transmitting>,\n");
        s.push_str("        }\n\n");

        s.push_str("        impl<'a> UartTx<'a> {\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn is_tx_ready(&self) -> bool {\n");
        s.push_str(&format!(
            "                self.u.{}.read() != 0\n",
            self.field_events_txrdy
        ));
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn is_tx_complete(&self) -> bool {\n");
        s.push_str(&format!(
            "                self.u.{}.read() != 0\n",
            self.field_events_endtx
        ));
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn clear_tx_ready(&self) {\n");
        s.push_str(&format!(
            "                self.u.{}.write(0);\n",
            self.field_events_txrdy
        ));
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn clear_tx_complete(&self) {\n");
        s.push_str(&format!(
            "                self.u.{}.write(0);\n",
            self.field_events_endtx
        ));
        s.push_str("            }\n\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn stop(self) -> Uart<'a, Idle> {\n");
        s.push_str(&format!(
            "                self.u.{}.write(1);\n",
            self.field_tasks_stoptx
        ));
        s.push_str(&format!(
            "                Uart {{ u: self.u, _state: PhantomData }}\n"
        ));
        s.push_str("            }\n");
        s.push_str("        }\n");

        s.push_str("    }\n");
        Ok(s)
    }
}

pub fn collect_uarts(device: &svd::Device) -> Vec<UartInfo> {
    let mut out = Vec::new();
    for p in &device.peripherals {
        if !is_uart_like(&p.name) {
            continue;
        }
        let items = gpio::peripheral_register_items(device, p);
        if items.is_empty() {
            continue;
        }

        let Some((tasks_startrx_name, _)) = gpio::find_register(items, "TASKS_STARTRX") else {
            continue;
        };
        let Some((tasks_stoprx_name, _)) = gpio::find_register(items, "TASKS_STOPRX") else {
            continue;
        };
        let Some((tasks_starttx_name, _)) = gpio::find_register(items, "TASKS_STARTTX") else {
            continue;
        };
        let Some((tasks_stoptx_name, _)) = gpio::find_register(items, "TASKS_STOPTX") else {
            continue;
        };
        let Some((tasks_flushrx_name, _)) = gpio::find_register(items, "TASKS_FLUSHRX") else {
            continue;
        };

        let Some((events_rxrdy_name, _)) = gpio::find_register(items, "EVENTS_RXDRDY") else {
            continue;
        };
        let Some((events_txrdy_name, _)) = gpio::find_register(items, "EVENTS_TXDRDY") else {
            continue;
        };
        let Some((events_endrx_name, _)) = gpio::find_register(items, "EVENTS_ENDRX") else {
            continue;
        };
        let Some((events_endtx_name, _)) = gpio::find_register(items, "EVENTS_ENDTX") else {
            continue;
        };
        let Some((events_error_name, _)) = gpio::find_register(items, "EVENTS_ERROR") else {
            continue;
        };

        let Some((intenset_name, _intenset_reg)) = gpio::find_register(items, "INTENSET") else {
            continue;
        };

        let Some((baudrate_name, baudrate_reg)) = gpio::find_register(items, "BAUDRATE") else {
            continue;
        };
        let baudrate_field = baudrate_reg
            .field
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case("BAUDRATE"))
            .cloned()
            .or_else(|| baudrate_reg.field.first().cloned());
        let Some(baudrate_field) = baudrate_field else {
            continue;
        };

        let Some((config_name, config_reg)) = gpio::find_register(items, "CONFIG") else {
            continue;
        };
        let config_hwfc_field = config_reg
            .field
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case("HWFC"))
            .cloned();
        let config_parity_field = config_reg
            .field
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case("PARITY"))
            .cloned();

        out.push(UartInfo {
            periph_name: p.name.clone(),
            periph_mod: gpio::sanitize_module_name(&p.name),
            hal_mod: gpio::sanitize_field_name(&p.name),

            field_tasks_startrx: gpio::sanitize_field_name(&tasks_startrx_name),
            field_tasks_stoprx: gpio::sanitize_field_name(&tasks_stoprx_name),
            field_tasks_starttx: gpio::sanitize_field_name(&tasks_starttx_name),
            field_tasks_stoptx: gpio::sanitize_field_name(&tasks_stoptx_name),
            field_tasks_flushrx: gpio::sanitize_field_name(&tasks_flushrx_name),

            field_events_rxrdy: gpio::sanitize_field_name(&events_rxrdy_name),
            field_events_txrdy: gpio::sanitize_field_name(&events_txrdy_name),
            field_events_endrx: gpio::sanitize_field_name(&events_endrx_name),
            field_events_endtx: gpio::sanitize_field_name(&events_endtx_name),
            field_events_error: gpio::sanitize_field_name(&events_error_name),

            field_intenset: gpio::sanitize_field_name(&intenset_name),

            field_baudrate: gpio::sanitize_field_name(&baudrate_name),
            baudrate_field,

            field_config: gpio::sanitize_field_name(&config_name),
            config_hwfc_field,
            config_parity_field,
        });
    }
    out
}

fn is_uart_like(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    upper.starts_with("UART") || upper.starts_with("UARTE")
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
