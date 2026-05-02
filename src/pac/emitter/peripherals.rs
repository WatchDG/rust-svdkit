use crate::Result;
use crate::pac::ir::*;
use crate::pac::emitter::common::CodeWriter;

use super::clusters;

pub fn emit_peripheral_mod(
    periph: &PeripheralIr,
) -> Result<String> {
    let mut out = CodeWriter::new();
    out.indent();

    out.writeln("use super::*;")?;
    out.writeln(&format!(
        "pub const BASE: usize = 0x{:08X};",
        periph.base_address
    ))?;
    out.writeln("")?;

    if has_registers(&periph.register_block) {
        out.writeln("pub mod registers;")?;
    }
    if !periph.field_enums.is_empty() {
        out.writeln("pub mod enums;")?;
    }
    if periph.has_clusters {
        out.writeln("pub mod clusters;")?;
    }

    out.writeln("")?;
    emit_register_block(&mut out, periph)?;
    out.writeln("")?;

    clusters::emit_reset_impl(
        &mut out,
        &periph.type_name,
        &periph.register_block.items,
    )?;
    out.writeln("")?;

    out.writeln(&format!(
        "pub const PTR: *const {} = BASE as *const {};",
        periph.type_name, periph.type_name
    ))?;
    out.writeln(&format!(
        "pub const PTR_MUT: *mut {} = BASE as *mut {};",
        periph.type_name, periph.type_name
    ))?;
    out.writeln("")?;

    if !periph.once_regs.is_empty() {
        emit_once_struct(&mut out, periph)?;
        out.writeln("")?;
    }

    Ok(out.s)
}

fn has_registers(block: &RegisterBlockIr) -> bool {
    block.items.iter().any(|item| {
        matches!(item, RegisterBlockItemIr::Register(_))
    })
}

fn emit_register_block(out: &mut CodeWriter, periph: &PeripheralIr) -> Result<()> {
    out.writeln("#[repr(C)]")?;
    out.writeln(&format!("pub struct {} {{", periph.type_name))?;
    out.indent();

    let mut reserved_idx = 0usize;
    let mut cur: u64 = 0;

    for item in &periph.register_block.items {
        match item {
            RegisterBlockItemIr::Reserved { offset, size, index } => {
                out.writeln(&format!(
                    "pub _reserved_{index}: [u8; {size} as usize],"
                ))?;
                cur = offset + size;
                let _ = reserved_idx;
            }
            RegisterBlockItemIr::Register(reg) => {
                let off = reg.offset;
                if off > cur {
                    let gap = off - cur;
                    out.writeln(&format!(
                        "pub _reserved_{reserved_idx}: [u8; {gap} as usize],"
                    ))?;
                    reserved_idx += 1;
                }
                if let Some(dim) = &reg.dim {
                    if dim.is_typed_array {
                        out.writeln(&format!(
                            "pub {}: [registers::{}; {} as usize],",
                            reg.field_name, reg.reg_type_name, dim.dim
                        ))?;
                    } else {
                        let total = dim.dim_increment * dim.dim;
                        out.writeln(&format!(
                            "/// NOTE: {} is an array with dimIncrement={}, elementSize={} — emitted as bytes",
                            reg.name, dim.dim_increment, reg.size_bytes
                        ))?;
                        out.writeln(&format!(
                            "pub {}: [u8; {total} as usize],",
                            reg.field_name
                        ))?;
                    }
                } else {
                    out.writeln(&format!(
                        "pub {}: registers::{},",
                        reg.field_name, reg.reg_type_name
                    ))?;
                }
                cur = off + reg.size_bytes;
            }
            RegisterBlockItemIr::Cluster(cluster) => {
                let off = cluster.offset;
                if off > cur {
                    let gap = off - cur;
                    out.writeln(&format!(
                        "pub _reserved_{reserved_idx}: [u8; {gap} as usize],"
                    ))?;
                    reserved_idx += 1;
                }
                if let Some(dim) = &cluster.dim {
                    if dim.is_typed_array {
                        out.writeln(&format!(
                            "pub {}: [{}; {} as usize],",
                            cluster.field_name, cluster.cluster_path, dim.dim
                        ))?;
                    } else {
                        let total = dim.dim_increment * dim.dim;
                        out.writeln(&format!(
                            "pub {}: [u8; {total} as usize],",
                            cluster.field_name
                        ))?;
                    }
                } else {
                    out.writeln(&format!(
                        "pub {}: {},",
                        cluster.field_name, cluster.cluster_path
                    ))?;
                }
                cur = off + cluster.size_bytes;
            }
        }
    }

    out.dedent();
    out.writeln("}")?;
    Ok(())
}

fn emit_once_struct(out: &mut CodeWriter, periph: &PeripheralIr) -> Result<()> {
    let once_ty = "Once";
    out.writeln("/// Compile-time writeOnce/read-writeOnce tokens (state-machine API).")?;
    out.writeln("///")?;
    out.writeln("/// NOTE: this enforces \"once\" only for code using this token API.")?;
    out.writeln(&format!("pub struct {once_ty} {{"))?;
    out.indent();
    out.writeln("base: usize,")?;
    for r in &periph.once_regs {
        out.writeln(&format!("pub {}: {},", r.field_name, r.token_ty))?;
    }
    out.dedent();
    out.writeln("}")?;
    out.writeln(&format!("impl {once_ty} {{"))?;
    out.indent();
    out.writeln("/// Create tokens for this peripheral.")?;
    out.writeln("///")?;
    out.writeln("/// # Safety")?;
    out.writeln("/// The returned tokens allow volatile access to MMIO.")?;
    out.writeln("pub unsafe fn new() -> Self {")?;
    out.indent();
    out.writeln("let base = BASE;")?;
    out.writeln("Self {")?;
    out.indent();
    out.writeln("base,")?;
    for r in &periph.once_regs {
        out.writeln(&format!("{}: {},", r.field_name, r.init_expr))?;
    }
    out.dedent();
    out.writeln("}")?;
    out.dedent();
    out.writeln("}")?;
    out.dedent();
    out.writeln("}")?;
    Ok(())
}

pub fn emit_peripherals_singleton(module_names: &[String], device_name: &str) -> String {
    let mut out = String::new();

    out.push_str("use core::sync::atomic::{AtomicBool, Ordering};\n");
    out.push_str("\n");
    out.push_str("static TAKEN: AtomicBool = AtomicBool::new(false);\n");
    out.push_str("\n");
    out.push_str("#[non_exhaustive]\n");
    out.push_str("pub struct Peripherals {\n");
    for p in module_names {
        out.push_str(&format!("    pub {p}: {p}::RegisterBlock,\n"));
    }
    out.push_str("}\n");
    out.push_str("\n");
    out.push_str("impl Peripherals {\n");
    out.push_str("    #[inline]\n");
    out.push_str("    pub fn take() -> Option<Self> {\n");
    out.push_str("        if TAKEN.compare_exchange(\n");
    out.push_str("            false,\n");
    out.push_str("            true,\n");
    out.push_str("            Ordering::SeqCst,\n");
    out.push_str("            Ordering::SeqCst,\n");
    out.push_str("        )\n");
    out.push_str("        .is_ok()\n");
    out.push_str("        {\n");
    out.push_str("            Some(unsafe { Self::steal() })\n");
    out.push_str("        } else {\n");
    out.push_str("            None\n");
    out.push_str("        }\n");
    out.push_str("    }\n");
    out.push_str("\n");
    out.push_str("    #[inline]\n");
    out.push_str("    pub unsafe fn steal() -> Self {\n");
    out.push_str("        Self {\n");
    for p in module_names {
        out.push_str(&format!(
            "            {p}: {p}::RegisterBlock {{ ..core::mem::zeroed() }},\n"
        ));
    }
    out.push_str("        }\n");
    out.push_str("    }\n");
    out.push_str("}\n");

    out
}
