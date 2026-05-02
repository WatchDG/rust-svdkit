use crate::Result;
use crate::pac::ir::*;
use crate::pac::emitter::common::CodeWriter;

pub fn emit_cluster_type(type_defs: &mut CodeWriter, cluster: &ClusterFieldIr) -> Result<()> {
    if cluster.items.is_empty() {
        return Ok(());
    }

    type_defs.writeln(&format!("/// Cluster `{}`", cluster.name))?;
    type_defs.writeln("#[repr(C)]")?;
    type_defs.writeln(&format!("pub struct {} {{", cluster.cluster_type_name))?;
    type_defs.indent();

    let mut reserved_idx = 0usize;
    let mut cur: u64 = 0;

    for item in &cluster.items {
        match item {
            RegisterBlockItemIr::Reserved { offset, size, .. } => {
                if *offset > cur {
                    let gap = offset - cur;
                    type_defs.writeln(&format!(
                        "pub _reserved_{reserved_idx}: [u8; {gap} as usize],"
                    ))?;
                    reserved_idx += 1;
                    cur = *offset;
                }
            }
            RegisterBlockItemIr::Register(reg) => {
                let off = reg.offset;
                if off > cur {
                    let gap = off - cur;
                    type_defs.writeln(&format!(
                        "pub _reserved_{reserved_idx}: [u8; {gap} as usize],"
                    ))?;
                    reserved_idx += 1;
                    cur = off;
                }
                if let Some(dim) = &reg.dim {
                    if dim.is_typed_array {
                        type_defs.writeln(&format!(
                            "pub {}: [{}; {} as usize],",
                            reg.field_name, reg.reg_type_name, dim.dim
                        ))?;
                    } else {
                        let total = dim.dim_increment * dim.dim;
                        type_defs.writeln(&format!(
                            "pub {}: [u8; {total} as usize],",
                            reg.field_name
                        ))?;
                    }
                } else {
                    type_defs.writeln(&format!(
                        "pub {}: {},",
                        reg.field_name, reg.reg_type_name
                    ))?;
                }
                cur = off + reg.size_bytes;
            }
            RegisterBlockItemIr::Cluster(nested) => {
                let off = nested.offset;
                if off > cur {
                    let gap = off - cur;
                    type_defs.writeln(&format!(
                        "pub _reserved_{reserved_idx}: [u8; {gap} as usize],"
                    ))?;
                    reserved_idx += 1;
                    cur = off;
                }
                if let Some(dim) = &nested.dim {
                    if dim.is_typed_array {
                        type_defs.writeln(&format!(
                            "pub {}: [{}; {} as usize],",
                            nested.field_name, nested.cluster_type_name, dim.dim
                        ))?;
                    } else {
                        let total = dim.dim_increment * dim.dim;
                        type_defs.writeln(&format!(
                            "pub {}: [u8; {total} as usize],",
                            nested.field_name
                        ))?;
                    }
                } else {
                    type_defs.writeln(&format!(
                        "pub {}: {},",
                        nested.field_name, nested.cluster_type_name
                    ))?;
                }
                cur = off + nested.size_bytes;
            }
        }
    }

    type_defs.dedent();
    type_defs.writeln("}")?;
    type_defs.writeln("")?;

    Ok(())
}

pub fn emit_reset_impl(
    out: &mut CodeWriter,
    type_name: &str,
    items: &[RegisterBlockItemIr],
) -> Result<()> {
    out.writeln(&format!("impl {type_name} {{"))?;
    out.indent();
    out.writeln("#[inline(always)]")?;
    out.writeln("pub fn reset(&self) {")?;
    out.indent();

    for item in items {
        match item {
            RegisterBlockItemIr::Register(reg) => {
                let Some((value, mask)) = reg.reset_value else {
                    continue;
                };
                if matches!(reg.access.access, svd::AccessType::ReadOnly) {
                    continue;
                }
                if reg.access.write_model != WriteModel::Normal {
                    continue;
                }
                let val = value & mask;
                let lit = reset_literal(val, &reg.base_type);
                if let Some(dim) = &reg.dim {
                    if dim.is_typed_array {
                        out.writeln(&format!(
                            "for r in self.{}.iter() {{ r.write({lit}); }}",
                            reg.field_name
                        ))?;
                    }
                } else {
                    out.writeln(&format!("self.{}.write({lit});", reg.field_name))?;
                }
            }
            RegisterBlockItemIr::Cluster(cluster) => {
                if let Some(dim) = &cluster.dim {
                    if dim.is_typed_array {
                        out.writeln(&format!(
                            "for c in self.{}.iter() {{ c.reset(); }}",
                            cluster.field_name
                        ))?;
                    }
                } else if !cluster.items.is_empty() {
                    out.writeln(&format!("self.{}.reset();", cluster.field_name))?;
                }
            }
            _ => {}
        }
    }

    out.dedent();
    out.writeln("}")?;
    out.dedent();
    out.writeln("}")?;
    Ok(())
}

fn reset_literal(val: u64, base_ty: &str) -> String {
    match base_ty {
        "u8" => format!("0x{val:02X}u8"),
        "u16" => format!("0x{val:04X}u16"),
        "u32" => format!("0x{val:08X}u32"),
        "u64" => format!("0x{val:016X}u64"),
        _ => format!("0x{val:X}u32"),
    }
}
