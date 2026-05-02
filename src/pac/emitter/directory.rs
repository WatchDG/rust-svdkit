use crate::Result;
use crate::pac::ir::*;
use crate::pac::config::{GenerationPlan, OutputMode};
use crate::pac::emitter::common::CodeWriter;
use crate::pac::emitter::{registers, enums, clusters, peripherals, runtime, cargo};
use crate::pac::static_files;

use super::GeneratedFile;
use super::GeneratedDir;

pub fn emit_directory(ir: &PacIr, plan: &GenerationPlan) -> Result<crate::pac::GeneratedDir> {
    let dir_name = format!("{}_pac", ir.device_info.file_stem);
    let mut files: Vec<GeneratedFile> = Vec::new();

    let lib_rs_content = {
        let mut lines = Vec::new();
        lines.push("#![no_std]".to_string());
        lines.push("#![allow(unsafe_op_in_unsafe_fn)]".to_string());
        lines.push("".to_string());
        lines.push("pub mod common_traits;".to_string());
        lines.push("pub mod common_types;".to_string());
        lines.push("pub mod common_enums;".to_string());
        lines.push("pub mod common_constants;".to_string());
        lines.push("".to_string());
        lines.push("#[macro_use]".to_string());
        lines.push("pub mod common_macros;".to_string());
        lines.push("".to_string());
        lines.push("pub mod peripherals;".to_string());
        lines.push("".to_string());
        lines.push("pub mod cortex_m;".to_string());
        lines.push("pub mod rt;".to_string());
        lines.join("\n")
    };
    files.push(GeneratedFile {
        file_name: "lib.rs".to_string(),
        content: lib_rs_content,
    });

    files.push(GeneratedFile {
        file_name: "common_macros.rs".to_string(),
        content: static_files::generate_macros_file(),
    });
    files.push(GeneratedFile {
        file_name: "common_traits.rs".to_string(),
        content: static_files::generate_traits_file(),
    });
    files.push(GeneratedFile {
        file_name: "common_types.rs".to_string(),
        content: static_files::generate_types_file(),
    });
    files.push(GeneratedFile {
        file_name: "common_enums.rs".to_string(),
        content: runtime::generate_enums_file(ir),
    });
    files.push(GeneratedFile {
        file_name: "common_constants.rs".to_string(),
        content: runtime::generate_constants_file(ir),
    });

    let module_names: Vec<String> = ir.peripheral_module_names();

    let periph_mod_content = {
        let mut lines = Vec::new();
        lines.push("#[allow(non_snake_case)]".to_string());
        lines.push("#[allow(non_camel_case_types)]".to_string());
        lines.push("#[allow(dead_code)]".to_string());
        lines.push("#[allow(unused_imports)]".to_string());
        lines.push("#[allow(unsafe_op_in_unsafe_fn)]".to_string());
        lines.push("".to_string());
        for name in &module_names {
            lines.push(format!("pub mod {name};"));
        }
        lines.push("".to_string());
        lines.push(peripherals::emit_peripherals_singleton(
            &module_names,
            &ir.device_info.name,
        ));
        lines.join("\n")
    };
    files.push(GeneratedFile {
        file_name: "peripherals/mod.rs".to_string(),
        content: periph_mod_content,
    });

    for p in &ir.peripherals {
        let mod_name = &p.module_name;

        let mod_content = {
            let mut out = CodeWriter::new();
            out.writeln("use super::*;")?;
            out.writeln(&format!(
                "pub const BASE: usize = 0x{:08X};",
                p.base_address
            ))?;
            out.writeln("")?;

            if has_registers(&p.register_block) {
                out.writeln("pub mod registers;")?;
            }
            if !p.field_enums.is_empty() {
                out.writeln("pub mod enums;")?;
            }
            if p.has_clusters {
                out.writeln("pub mod clusters;")?;
            }
            out.s
        };
        files.push(GeneratedFile {
            file_name: format!("peripherals/{}/mod.rs", mod_name),
            content: mod_content,
        });

        let regs_content = emit_register_types(p, plan.options.emit_field_methods)?;
        if !regs_content.trim().is_empty() {
            files.push(GeneratedFile {
                file_name: format!("peripherals/{}/registers.rs", mod_name),
                content: regs_content,
            });
        }

        if !p.field_enums.is_empty() {
            files.push(GeneratedFile {
                file_name: format!("peripherals/{}/enums.rs", mod_name),
                content: enums::generate_enums_for_peripheral(p)?,
            });
        }

        if p.has_clusters {
            let cluster_files = emit_cluster_files(p)?;
            for cf in cluster_files {
                let file_name = format!("peripherals/{}/clusters/{}", mod_name, cf.file_name);
                files.push(GeneratedFile { file_name, content: cf.content });
            }

            let mut clusters_mod_lines = Vec::new();
            clusters_mod_lines.push("#[allow(non_snake_case)]".to_string());
            clusters_mod_lines.push("#[allow(non_camel_case_types)]".to_string());
            clusters_mod_lines.push("#[allow(dead_code)]".to_string());
            clusters_mod_lines.push("#[allow(unused_imports)]".to_string());
            clusters_mod_lines.push("#[allow(unsafe_op_in_unsafe_fn)]".to_string());
            clusters_mod_lines.push("".to_string());

            for item in &p.register_block.items {
                if let RegisterBlockItemIr::Cluster(cluster) = item {
                    clusters_mod_lines.push(format!("pub mod {};", cluster.cluster_type_name.to_lowercase()));
                }
            }

            files.push(GeneratedFile {
                file_name: format!("peripherals/{}/clusters/mod.rs", mod_name),
                content: clusters_mod_lines.join("\n"),
            });
        }
    }

    files.push(GeneratedFile {
        file_name: "Cargo.toml".to_string(),
        content: cargo::generate_cargo_toml(ir, &dir_name)?,
    });

    Ok(GeneratedDir {
        path: dir_name,
        files,
    })
}

fn has_registers(block: &RegisterBlockIr) -> bool {
    block.items.iter().any(|item| {
        matches!(item, RegisterBlockItemIr::Register(_))
    })
}

fn emit_register_types(periph: &PeripheralIr, emit_field_methods: bool) -> Result<String> {
    let mut out = CodeWriter::new();
    for item in &periph.register_block.items {
        if let RegisterBlockItemIr::Register(reg) = item {
            if matches!(reg.reg_type, RegisterTypeIr::Primitive) {
                continue;
            }
            registers::emit_register_type(&mut out, reg, emit_field_methods)?;
        }
    }
    Ok(out.s)
}

fn emit_cluster_files(periph: &PeripheralIr) -> Result<Vec<GeneratedFile>> {
    let mut files = Vec::new();

    for item in &periph.register_block.items {
        if let RegisterBlockItemIr::Cluster(cluster) = item {
            let mod_name = cluster.cluster_type_name.to_lowercase();

            let mod_content = {
                let mut cw = CodeWriter::new();
                if cluster.items.iter().any(|i| matches!(i, RegisterBlockItemIr::Register(_))) {
                    cw.writeln("pub mod registers;")?;
                    cw.writeln("pub use self::registers::*;")?;
                }
                cw.writeln("")?;
                cw.s
            };
            files.push(GeneratedFile {
                file_name: format!("{}/mod.rs", mod_name),
                content: mod_content,
            });

            let regs = emit_cluster_register_types(cluster)?;
            if !regs.trim().is_empty() {
                files.push(GeneratedFile {
                    file_name: format!("{}/registers.rs", mod_name),
                    content: regs,
                });
            }
        }
    }

    Ok(files)
}

fn emit_cluster_register_types(cluster: &ClusterFieldIr) -> Result<String> {
    let mut out = CodeWriter::new();
    for item in &cluster.items {
        if let RegisterBlockItemIr::Register(reg) = item {
            if matches!(reg.reg_type, RegisterTypeIr::Primitive) {
                continue;
            }
            registers::emit_register_type(&mut out, reg, false)?;
        }
    }
    Ok(out.s)
}
