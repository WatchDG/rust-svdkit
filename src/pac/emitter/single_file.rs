use crate::Result;
use crate::pac::config::GenerationPlan;
use crate::pac::config::OutputMode;
use crate::pac::emitter::common::CodeWriter;
use crate::pac::emitter::{registers, enums, clusters, peripherals, runtime};
use crate::pac::ir::*;
use crate::pac::static_files;

pub fn emit_single_file(ir: &PacIr, plan: &GenerationPlan) -> Result<String> {
    let mut out = CodeWriter::new();
    let mut type_defs = CodeWriter::new();
    let mut regs_out = CodeWriter::new();

    out.writeln("#[allow(non_snake_case)]")?;
    out.writeln("#[allow(non_camel_case_types)]")?;
    out.writeln("#[allow(dead_code)]")?;
    out.writeln("#[allow(unused_imports)]")?;
    out.writeln("#[allow(unsafe_op_in_unsafe_fn)]")?;
    out.writeln("")?;
    out.writeln("")?;

    out.writeln("pub mod common_traits {")?;
    out.writeln(&static_files::generate_traits_file())?;
    out.writeln("}")?;
    out.writeln("")?;
    out.writeln("pub mod common_types {")?;
    out.writeln(&static_files::generate_types_file())?;
    out.writeln("}")?;
    out.writeln("")?;
    out.writeln("pub mod common_enums {")?;
    out.writeln(&runtime::generate_enums_file(ir))?;
    out.writeln("}")?;
    out.writeln("")?;
    out.writeln("pub mod common_constants {")?;
    out.writeln(&runtime::generate_constants_file(ir))?;
    out.writeln("}")?;
    out.writeln("")?;

    for p in &ir.peripherals {
        let cname = &p.const_name;
        out.writeln(&format!(
            "pub const {cname}_BASE: usize = 0x{addr:08X};",
            addr = p.base_address
        ))?;
    }
    out.writeln("")?;

    let emit_field_methods = plan.options.emit_field_methods;

    for p in &ir.peripherals {
        let mod_content = emit_peripheral_inline(p, emit_field_methods)?;
        out.writeln(&format!("/// Peripheral `{}`", p.name))?;
        if let Some(desc) = &p.description {
            out.writeln(&format!("/// {}", desc))?;
        }
        out.writeln(&format!("pub mod {} {{", p.module_name))?;
        out.s.push_str(&mod_content);
        out.writeln("}")?;
        out.writeln("")?;
    }

    if !type_defs.s.trim().is_empty() {
        out.s.push('\n');
        out.s.push_str(&type_defs.s);
    }

    if !regs_out.s.trim().is_empty() {
        out.s.push('\n');
        out.s.push_str(&regs_out.s);
    }

    Ok(out.s)
}

fn emit_peripheral_inline(periph: &PeripheralIr, emit_field_methods: bool) -> Result<String> {
    let mut out = CodeWriter::new();
    out.indent();
    out.writeln("use super::*;")?;
    out.writeln(&format!(
        "pub const BASE: usize = 0x{:08X};",
        periph.base_address
    ))?;
    out.writeln("")?;

    if has_registers(&periph.register_block) {
        out.writeln("pub mod registers {")?;
        out.indent();
        for item in &periph.register_block.items {
            if let RegisterBlockItemIr::Register(reg) = item {
                if matches!(reg.reg_type, RegisterTypeIr::Primitive) {
                    continue;
                }
                let mut reg_writer = CodeWriter::new();
                registers::emit_register_type(&mut reg_writer, reg, emit_field_methods)?;
                out.s.push_str(&reg_writer.s);
            }
        }
        out.dedent();
        out.writeln("}")?;
        out.writeln("use self::registers::*;")?;
        out.writeln("")?;
    }

    if !periph.field_enums.is_empty() {
        out.writeln("pub mod enums {")?;
        out.writeln(&enums::generate_enums_for_peripheral(periph)?)?;
        out.writeln("}")?;
        out.writeln("use self::enums::*;")?;
        out.writeln("")?;
    }

    if periph.has_clusters {
        out.writeln("pub mod clusters {")?;
        out.indent();
        for item in &periph.register_block.items {
            if let RegisterBlockItemIr::Cluster(cluster) = item {
                clusters::emit_cluster_type(&mut out, cluster)?;
            }
        }
        out.dedent();
        out.writeln("}")?;
        out.writeln("use self::clusters::*;")?;
        out.writeln("")?;
    }

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
                            "pub {}: [{reg_ty}; {n} as usize],",
                            reg.field_name,
                            reg_ty = reg.reg_type_name,
                            n = dim.dim
                        ))?;
                    } else {
                        let total = dim.dim_increment * dim.dim;
                        out.writeln(&format!(
                            "pub {}: [u8; {total} as usize],",
                            reg.field_name
                        ))?;
                    }
                } else {
                    out.writeln(&format!(
                        "pub {}: {},",
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
                            "pub {}: [{cty}; {n} as usize],",
                            cluster.field_name,
                            cty = cluster.cluster_type_name,
                            n = dim.dim
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
                        cluster.field_name, cluster.cluster_type_name
                    ))?;
                }
                cur = off + cluster.size_bytes;
            }
        }
    }

    out.dedent();
    out.writeln("}")?;

    clusters::emit_reset_impl(&mut out, &periph.type_name, &periph.register_block.items)?;

    out.writeln("")?;
    out.writeln(&format!(
        "pub const PTR: *const {} = BASE as *const {};",
        periph.type_name, periph.type_name
    ))?;
    out.writeln(&format!(
        "pub const PTR_MUT: *mut {} = BASE as *mut {};",
        periph.type_name, periph.type_name
    ))?;

    if !periph.once_regs.is_empty() {
        let once_ty = "Once";
        out.writeln("")?;
        out.writeln("pub struct Once {")?;
        out.indent();
        out.writeln("base: usize,")?;
        for r in &periph.once_regs {
            out.writeln(&format!("pub {}: {},", r.field_name, r.token_ty))?;
        }
        out.dedent();
        out.writeln("}")?;
        out.writeln("impl Once {")?;
        out.indent();
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
    }

    Ok(out.s)
}

fn has_registers(block: &RegisterBlockIr) -> bool {
    block.items.iter().any(|item| {
        matches!(item, RegisterBlockItemIr::Register(_))
    })
}
