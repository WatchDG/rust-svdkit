use crate::Result;
use crate::hal::config::{HalGenerationPlan, HalModulePlan, HalOutputMode};
use crate::hal::ir::*;
use crate::hal::common::*;
use crate::hal::{GeneratedFile, GeneratedDir};

mod gpio;
mod timer;
mod clock;
mod power;
mod usb;

pub fn emit(ir: &HalIr, plan: &HalGenerationPlan) -> Result<GeneratedDir> {
    match plan.output_mode {
        HalOutputMode::Directory => emit_directory(ir, plan),
        HalOutputMode::SingleFile => emit_single_file(ir, plan),
    }
}

fn emit_directory(ir: &HalIr, plan: &HalGenerationPlan) -> Result<GeneratedDir> {
    let dir_name = &plan.dir_name;
    let mut files: Vec<GeneratedFile> = Vec::new();

    let mut lib_lines = Vec::new();
    lib_lines.push("#![no_std]".to_string());
    lib_lines.push("#![allow(unsafe_op_in_unsafe_fn)]".to_string());
    lib_lines.push("".to_string());
    lib_lines.push(format!("use {} as pac;", plan.pac_crate_name));
    lib_lines.push("".to_string());

    for m in &plan.enabled_modules {
        match m {
            HalModulePlan::Gpio => lib_lines.push("pub mod gpio;".to_string()),
            HalModulePlan::Timer => lib_lines.push("pub mod timer;".to_string()),
            HalModulePlan::Clock => lib_lines.push("pub mod clock;".to_string()),
            HalModulePlan::Power => lib_lines.push("pub mod power;".to_string()),
            HalModulePlan::Usb => lib_lines.push("pub mod usb;".to_string()),
            _ => {}
        }
    }
    lib_lines.push("".to_string());

    files.push(GeneratedFile {
        file_name: "lib.rs".to_string(),
        content: lib_lines.join("\n"),
    });

    let cargo_toml = format!(
        "[package]\nname = {:?}\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[lib]\nname = {:?}\npath = \"lib.rs\"\n\n[dependencies]\n{} = {{ path = \"../{}\" }}\n",
        dir_name, dir_name, plan.pac_crate_name, plan.pac_crate_name
    );
    files.push(GeneratedFile {
        file_name: "Cargo.toml".to_string(),
        content: cargo_toml,
    });

    for m in &plan.enabled_modules {
        match m {
            HalModulePlan::Gpio => {
                let content = gpio::emit_gpio_file(&ir.gpio_ports, &plan.pac_crate_name)?;
                files.push(GeneratedFile {
                    file_name: "gpio/mod.rs".to_string(),
                    content,
                });
            }
            HalModulePlan::Timer => {
                let content = timer::emit_timer_file(&ir.timers, &plan.pac_crate_name)?;
                files.push(GeneratedFile {
                    file_name: "timer/mod.rs".to_string(),
                    content,
                });
            }
            HalModulePlan::Clock => {
                let content = clock::emit_clock_file(&ir.clocks, &plan.pac_crate_name)?;
                files.push(GeneratedFile {
                    file_name: "clock/mod.rs".to_string(),
                    content,
                });
            }
            HalModulePlan::Power => {
                let content = power::emit_power_file(&ir.power_devices, &plan.pac_crate_name)?;
                files.push(GeneratedFile {
                    file_name: "power/mod.rs".to_string(),
                    content,
                });
            }
            HalModulePlan::Usb => {
                let include_cdc = plan.options.emit_usb_cdc_acm;
                let content = usb::emit_usb_file(&ir.usb_devices, &plan.pac_crate_name, include_cdc)?;
                files.push(GeneratedFile {
                    file_name: "usb/mod.rs".to_string(),
                    content,
                });
            }
            _ => {}
        }
    }

    Ok(GeneratedDir {
        path: dir_name.clone(),
        files,
    })
}

fn emit_single_file(ir: &HalIr, plan: &HalGenerationPlan) -> Result<GeneratedDir> {
    let dir_name = &plan.dir_name;
    let mut files: Vec<GeneratedFile> = Vec::new();

    let mut out = String::new();
    out.push_str("#[allow(dead_code)]\n");
    out.push_str("#[allow(non_snake_case)]\n");
    out.push_str("#[allow(clippy::eq_op)]\n");
    out.push_str("#[allow(clippy::erasing_op)]\n");
    out.push_str("#[allow(clippy::identity_op)]\n");
    out.push_str("#[allow(unsafe_op_in_unsafe_fn)]\n\n");
    out.push_str(&format!("use {} as pac;\n\n", plan.pac_crate_name));

    for m in &plan.enabled_modules {
        match m {
            HalModulePlan::Gpio => {
                out.push_str("pub mod gpio {\n    use super::pac;\n\n");
                for port in &ir.gpio_ports {
                    out.push_str(&gpio::emit_port(port, &plan.pac_crate_name));
                    out.push('\n');
                }
                out.push_str("}\n");
            }
            HalModulePlan::Timer => {
                out.push_str("pub mod timer {\n    use super::pac;\n\n");
                for t in &ir.timers {
                    out.push_str(&timer::emit_timer(t, &plan.pac_crate_name));
                    out.push('\n');
                }
                out.push_str("}\n");
            }
            HalModulePlan::Clock => {
                out.push_str("pub mod clock {\n    use super::pac;\n\n");
                for c in &ir.clocks {
                    out.push_str(&clock::emit_clock(c, &plan.pac_crate_name));
                    out.push('\n');
                }
                out.push_str("}\n");
            }
            HalModulePlan::Power => {
                out.push_str("pub mod power {\n    use super::pac;\n\n");
                for p in &ir.power_devices {
                    out.push_str(&power::emit_power(p, &plan.pac_crate_name));
                    out.push('\n');
                }
                out.push_str("}\n");
            }
            HalModulePlan::Usb => {
                out.push_str("pub mod usb {\n    use super::pac;\n\n");
                for u in &ir.usb_devices {
                    out.push_str(&usb::emit_usb(u, &plan.pac_crate_name, plan.options.emit_usb_cdc_acm));
                    out.push('\n');
                }
                out.push_str("}\n");
            }
            _ => {}
        }
    }

    let file_name = format!("{}_hal.rs", sanitize_file_stem(&plan.pac_crate_name.replace("_pac", "")));
    files.push(GeneratedFile {
        file_name,
        content: out,
    });

    Ok(GeneratedDir {
        path: dir_name.clone(),
        files,
    })
}
