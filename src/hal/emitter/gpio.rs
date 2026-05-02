use crate::Result;
use crate::hal::ir::*;
use crate::hal::common::*;

pub fn emit_gpio_file(ports: &[GpioPortIr], pac_crate: &str) -> Result<String> {
    let mut s = String::new();
    s.push_str("#[allow(dead_code)]\n");
    s.push_str("#[allow(non_snake_case)]\n\n");
    s.push_str(&format!("use super::pac;\n\n"));
    for port in ports {
        s.push_str(&emit_port(port, pac_crate));
        s.push('\n');
    }
    Ok(s)
}

pub fn emit_port(port: &GpioPortIr, _pac_crate: &str) -> String {
    let mut s = String::new();
    let port_ty = &port.port_type_name;
    let mod_name = &port.hal_mod;

    s.push_str(&format!("pub mod {mod_name} {{\n"));
    s.push_str("    #[allow(dead_code)]\n");
    s.push_str("    #[allow(non_snake_case)]\n\n");
    s.push_str(&format!("    pub type {port_ty} = crate::pac::peripherals::{}::{port_ty};\n\n", port.periph_mod));
    s.push_str("    use core::marker::PhantomData;\n\n");
    s.push_str("    pub trait PinState {}\n");
    s.push_str("    pub struct Unconfigured;\n\n");
    s.push_str("    impl PinState for Unconfigured {}\n\n");
    s.push_str("    #[repr(u8)]\n");
    s.push_str("    #[derive(Copy, Clone, Debug, PartialEq, Eq)]\n");
    s.push_str("    pub enum Level {\n        Low = 0,\n        High = 1,\n    }\n\n");
    s.push_str("    #[inline(always)]\n");
    s.push_str(&format!("    pub unsafe fn steal() -> &'static {port_ty} {{\n        &*crate::pac::peripherals::{}::PTR\n    }}\n\n", port.periph_mod));

    let mut generated_any = false;
    for field in &port.pin_fields {
        if let Some(ref pac_name) = field.pac_enum_binding {
            s.push_str(&format!(
                "    pub use crate::pac::peripherals::{}::enums::{} as {};\n",
                port.periph_mod, pac_name, field.alias
            ));
            generated_any = true;
        } else if let Some(ref def) = field.local_enum_def {
            s.push_str(&indent_block(def, 8));
            generated_any = true;
        }
    }
    if generated_any {
        s.push('\n');
    }

    s.push_str(&format!("    pub struct Pin<'a, S: PinState> {{\n        port: &'a {port_ty},\n        pin: u8,\n        _state: PhantomData<S>,\n    }}\n\n"));
    s.push_str("    #[inline(always)]\n");
    s.push_str(&format!("    pub unsafe fn pin(pin: u8) -> Pin<'static, Unconfigured> {{\n        Pin {{ port: &*crate::pac::peripherals::{}::PTR, pin, _state: PhantomData }}\n    }}\n\n", port.periph_mod));
    s.push_str(&format!("    pub struct PinConfigurator<'a> {{\n        port: &'a {port_ty},\n        pin: u8,\n        cnf: u32,\n        output: bool,\n    }}\n\n"));
    s.push_str("    impl<'a> Pin<'a, Unconfigured> {\n        #[inline(always)]\n        pub fn configure(self) -> PinConfigurator<'a> {\n");
    s.push_str(&format!("            let cnf = self.port.{}[self.pin as usize].read();\n", port.field_pin_cnf));
    s.push_str("            PinConfigurator { port: self.port, pin: self.pin, cnf, output: false }\n        }\n    }\n\n");

    s.push_str("    impl<'a> PinConfigurator<'a> {\n");
    for field in &port.pin_fields {
        if field.name == "DIR" {
            if let Some(out_val) = field.output_dir_value {
                s.push_str("        #[inline(always)]\n");
                s.push_str(&format!("        pub fn dir(self, v: {}) -> Self {{\n", field.alias));
                s.push_str(&format!("            let output = (v as u32) == {out_val}u32;\n"));
                if field.lsb == 0 {
                    s.push_str(&format!("            let cnf = (self.cnf & !0x{mask:X}u32) | (((v as u32) & 0x{mask:X}u32));\n", mask = field.mask));
                } else {
                    s.push_str(&format!("            let cnf = (self.cnf & !(0x{mask:X}u32 << {lsb})) | (((v as u32) & 0x{mask:X}u32) << {lsb});\n", mask = field.mask, lsb = field.lsb));
                }
                s.push_str("            PinConfigurator { port: self.port, pin: self.pin, cnf, output }\n        }\n\n");
            }
        } else {
            s.push_str("        #[inline(always)]\n");
            s.push_str(&format!("        pub fn {}(self, v: {}) -> Self {{\n", field.name.to_lowercase(), field.alias));
            if field.lsb == 0 {
                s.push_str(&format!("            let cnf = (self.cnf & !0x{mask:X}u32) | (((v as u32) & 0x{mask:X}u32));\n", mask = field.mask));
            } else {
                s.push_str(&format!("            let cnf = (self.cnf & !(0x{mask:X}u32 << {lsb})) | (((v as u32) & 0x{mask:X}u32) << {lsb});\n", mask = field.mask, lsb = field.lsb));
            }
            s.push_str("            PinConfigurator { port: self.port, pin: self.pin, cnf, output: self.output }\n        }\n\n");
        }
    }

    s.push_str("        #[inline(always)]\n        pub fn apply(self) -> PinConfigured<'a> {\n");
    s.push_str(&format!("            self.port.{}[self.pin as usize].write(self.cnf);\n", port.field_pin_cnf));
    s.push_str("            if self.output {\n                PinConfigured::Output(PinOutput { port: self.port, pin: self.pin })\n            } else {\n                PinConfigured::Input(PinInput { port: self.port, pin: self.pin })\n            }\n        }\n    }\n\n");

    s.push_str("    pub enum PinConfigured<'a> {\n        Input(PinInput<'a>),\n        Output(PinOutput<'a>),\n    }\n\n");
    s.push_str(&format!("    pub struct PinInput<'a> {{\n        port: &'a {port_ty},\n        pin: u8,\n    }}\n\n"));
    s.push_str("    impl<'a> PinInput<'a> {\n        #[inline(always)]\n        pub fn reconfigure(self) -> PinConfigurator<'a> {\n");
    s.push_str(&format!("            let cnf = self.port.{}[self.pin as usize].read();\n", port.field_pin_cnf));
    s.push_str("            PinConfigurator { port: self.port, pin: self.pin, cnf, output: false }\n        }\n    }\n\n");

    s.push_str(&format!("    pub struct PinOutput<'a> {{\n        port: &'a {port_ty},\n        pin: u8,\n    }}\n\n"));
    s.push_str("    impl<'a> PinOutput<'a> {\n        #[inline(always)]\n        pub fn reconfigure(self) -> PinConfigurator<'a> {\n");
    s.push_str(&format!("            let cnf = self.port.{}[self.pin as usize].read();\n", port.field_pin_cnf));
    s.push_str("            PinConfigurator { port: self.port, pin: self.pin, cnf, output: true }\n        }\n\n");

    if let Some(ref level_enum) = port.level_enum_name {
        s.push_str("        #[inline(always)]\n");
        s.push_str(&format!("        pub fn set_level(&self, level: crate::pac::peripherals::{}::enums::{level_enum}) {{\n", port.periph_mod));
        s.push_str("            let mask = 1u32 << (self.pin as u32);\n");
        s.push_str(&format!("            let new_value = (self.port.{}.read() & !mask) | ((level as u32) << (self.pin as u32));\n", port.field_out));
        s.push_str(&format!("            self.port.{}.write(new_value);\n", port.field_out));
        s.push_str("        }\n\n");
        s.push_str("        #[inline(always)]\n        pub fn set_high(&self) {\n");
        s.push_str(&format!("            self.set_level(crate::pac::peripherals::{}::enums::{level_enum}::High);\n", port.periph_mod));
        s.push_str("        }\n\n        #[inline(always)]\n        pub fn set_low(&self) {\n");
        s.push_str(&format!("            self.set_level(crate::pac::peripherals::{}::enums::{level_enum}::Low);\n", port.periph_mod));
        s.push_str("        }\n");
    } else {
        s.push_str("        #[inline(always)]\n        pub fn set_high(&self) {\n");
        s.push_str(&format!("            self.port.{}.write(1u32 << (self.pin as u32));\n", port.field_outset));
        s.push_str("        }\n\n        #[inline(always)]\n        pub fn set_low(&self) {\n");
        s.push_str(&format!("            self.port.{}.write(1u32 << (self.pin as u32));\n", port.field_outclr));
        s.push_str("        }\n");
    }
    s.push_str("    }\n}\n");

    s
}
