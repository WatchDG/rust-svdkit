use crate::Result;
use crate::svd;
use crate::pac::ir::*;
use crate::pac::emitter::common::CodeWriter;

fn base_ty_bits(base_ty: &str) -> u32 {
    match base_ty {
        "u8" => 8,
        "u16" => 16,
        "u32" => 32,
        "u64" => 64,
        _ => 32,
    }
}

pub fn emit_register_type(out: &mut CodeWriter, reg: &RegisterFieldIr, _emit_field_methods: bool) -> Result<()> {
    match &reg.reg_type {
        RegisterTypeIr::Primitive => {
            out.writeln(&format!("/// Register `{}`", reg.name))?;
            Ok(())
        }
        RegisterTypeIr::Wrapper {
            inner,
            has_read,
            has_write,
            write_model,
            is_once,
            once_ctor,
        } => {
            out.writeln(&format!("/// Register `{}`", reg.name))?;
            if let Some(doc) = &reg.doc {
                out.writeln(&format!("/// {}", doc))?;
            }
            out.writeln("#[repr(transparent)]")?;
            out.writeln(&format!("pub struct {}({});", reg.reg_type_name, inner))?;

            if *is_once {
                if let Some(ctor) = once_ctor {
                    let token_inner = format!("{ctor}<{}, Unwritten>", reg.base_type);
                    out.writeln(&format!("impl {} {{", reg.reg_type_name))?;
                    out.indent();
                    if *has_write {
                        out.writeln("#[inline(always)]")?;
                        out.writeln(&format!(
                            "pub(super) fn write(&self, v: {}) {{ self.0.write(v) }}",
                            reg.base_type
                        ))?;
                    }
                    if *has_read {
                        out.writeln("#[inline(always)]")?;
                        out.writeln(&format!(
                            "pub(super) fn read(&self) -> {} {{ self.0.read() }}",
                            reg.base_type
                        ))?;
                    }
                    out.writeln("#[inline(always)]")?;
                    out.writeln(&format!("pub fn once(&self) -> {token_inner} {{"))?;
                    out.indent();
                    out.writeln(&format!(
                        "{ctor} {{ base: super::BASE, offset: 0x{:X}usize, _state: PhantomData, _t: PhantomData }}",
                        reg.offset
                    ))?;
                    out.dedent();
                    out.writeln("}")?;
                    out.dedent();
                    out.writeln("}")?;
                    out.writeln("")?;
                }
            } else {
                let macro_name = match (has_read, has_write, write_model) {
                    (true, false, _) => "impl_ro_register",
                    (false, true, WriteModel::Normal) => "impl_wo_register",
                    (true, true, WriteModel::Normal) => "impl_rw_register",
                    (_, true, WriteModel::W1S) => "impl_w1s_register",
                    (_, true, WriteModel::W1C) => "impl_w1c_register",
                    (_, true, WriteModel::W0S) => "impl_w0s_register",
                    (_, true, WriteModel::W0C) => "impl_w0c_register",
                    (_, true, WriteModel::WT) => "impl_wt_register",
                    _ => "impl_rw_register",
                };
                out.writeln(&format!(
                    "{macro_name}!({}, {});",
                    reg.reg_type_name, reg.base_type
                ))?;
            }

            if let Some(read_action) = &reg.read_action {
                if *has_read {
                    let (method_name, doc) = match read_action {
                        svd::ReadAction::Clear => (
                            "read_and_clear",
                            " Reads the register; hardware clears it to 0 after the read.",
                        ),
                        svd::ReadAction::Set => (
                            "read_and_set",
                            " Reads the register; hardware sets it to the reset/default value after the read.",
                        ),
                        svd::ReadAction::Modify => (
                            "read_side_effect",
                            " Reads the register; the value may be modified by hardware as a side effect.",
                        ),
                        svd::ReadAction::ModifyExternal => (
                            "read_side_effect",
                            " Reads the register; the value may be modified by external hardware as a side effect.",
                        ),
                    };
                    out.writeln(&format!("impl {} {{", reg.reg_type_name))?;
                    out.indent();
                    out.writeln(&format!("///{doc}"))?;
                    out.writeln("#[inline(always)]")?;
                    out.writeln(&format!(
                        "pub fn {method_name}(&self) -> {} {{ self.read() }}",
                        reg.base_type
                    ))?;
                    out.writeln("")?;
                    out.dedent();
                    out.writeln("}")?;
                    out.writeln("")?;
                }
            }

            if _emit_field_methods && !reg.fields.is_empty() {
                emit_field_methods(out, reg)?;
            }

            out.writeln("")?;
            Ok(())
        }
    }
}

fn emit_field_methods(out: &mut CodeWriter, reg: &RegisterFieldIr) -> Result<()> {
    let base_ty = &reg.base_type;
    let reg_bits = base_ty_bits(base_ty) as u64;
    let has_read = reg.reg_type.has_read();
    let has_write = reg.reg_type.has_write();
    let write_model = reg.reg_type.write_model();
    let access = reg.access.access;
    let mut has_field_methods = false;

    for f in &reg.fields {
        if f.enum_bindings.is_empty() {
            continue;
        }

        let lsb = f.lsb;
        let width = f.width;
        let mask = f.mask;

        let read_pick = f.enum_bindings.iter().find(|b| b.is_read_pick);
        let write_pick = f.enum_bindings.iter().find(|b| b.is_write_pick);

        let method_base = format!("field_{}", f.name);

        if let Some(binding) = read_pick {
            if has_read {
                if !has_field_methods {
                    has_field_methods = true;
                    out.writeln(&format!("impl {} {{", reg.reg_type_name))?;
                    out.indent();
                }
                out.writeln("")?;
                out.writeln(&format!("/// Field `{}`", f.name))?;
                if let Some(d) = &f.description {
                    out.writeln(&format!("/// {}", d))?;
                }
                out.writeln("#[inline(always)]")?;
                if lsb == 0 {
                    out.writeln(&format!(
                        "pub fn {method_base}_raw(&self) -> u64 {{ (self.read() as u64) & 0x{mask:X}u64 }}"
                    ))?;
                } else {
                    out.writeln(&format!(
                        "pub fn {method_base}_raw(&self) -> u64 {{ (self.read() as u64) >> {lsb} & 0x{mask:X}u64 }}"
                    ))?;
                }
                out.writeln("#[inline(always)]")?;
                let repr = repr_for_bits(width);
                let enum_ty = &binding.enum_type_name;
                out.writeln(&format!(
                    "pub fn {method_base}(&self) -> Option<enums::{enum_ty}> {{ enums::{enum_ty}::from_bits(self.{method_base}_raw() as {repr}) }}"
                ))?;
            }
        }

        if let Some(binding) = write_pick {
            let field_access = f.access;
            let writable = !matches!(field_access, svd::AccessType::ReadOnly);
            if !writable {
                continue;
            }

            let enum_ty = &binding.enum_type_name;

            if matches!(access, svd::AccessType::ReadWrite | svd::AccessType::ReadWriteOnce)
                && write_model == WriteModel::Normal
            {
                if !has_field_methods {
                    has_field_methods = true;
                    out.writeln(&format!("impl {} {{", reg.reg_type_name))?;
                    out.indent();
                }
                let set_name = if read_pick.map(|b| &b.enum_type_name) == Some(enum_ty) {
                    format!("set_{method_base}")
                } else {
                    format!("set_{method_base}_write")
                };
                out.writeln("")?;
                if f.has_write_constraint {
                    out.writeln("/// Write value must be one of the enumerated values.")?;
                }
                out.writeln("#[inline(always)]")?;
                out.writeln(&format!(
                    "pub fn {set_name}(&self, v: enums::{enum_ty}) {{"
                ))?;
                out.indent();
                out.writeln(&format!("let cur = self.read() as u64;"))?;
                out.writeln(&format!("let v = (v.bits() as u64) & 0x{mask:X}u64;"))?;
                if lsb == 0 {
                    out.writeln(&format!("let new = (cur & !0x{mask:X}u64) | v;"))?;
                } else {
                    out.writeln(&format!(
                        "let new = (cur & !(0x{mask:X}u64 << {lsb})) | (v << {lsb});"
                    ))?;
                }
                out.writeln(&format!("self.write(new as {base_ty});"))?;
                out.dedent();
                out.writeln("}")?;
            } else if has_write {
                let method_name = if matches!(access, svd::AccessType::ReadWrite | svd::AccessType::ReadWriteOnce) {
                    if read_pick.map(|b| &b.enum_type_name) == Some(enum_ty) {
                        format!("set_{method_base}")
                    } else {
                        format!("set_{method_base}_write")
                    }
                } else {
                    format!("write_{method_base}")
                };

                let call = match (access, write_model) {
                    (_, WriteModel::W1S) | (_, WriteModel::W0S) => "set_bits",
                    (_, WriteModel::W1C) | (_, WriteModel::W0C) => "clear_bits",
                    (_, WriteModel::WT) => "toggle_bits",
                    (svd::AccessType::WriteOnly | svd::AccessType::WriteOnce, WriteModel::Normal) => "write",
                    _ => continue,
                };

                if !has_field_methods {
                    has_field_methods = true;
                    out.writeln(&format!("impl {} {{", reg.reg_type_name))?;
                    out.indent();
                }
                out.writeln("")?;
                if f.has_write_constraint {
                    out.writeln("/// Write value must be one of the enumerated values.")?;
                }
                out.writeln("#[inline(always)]")?;
                out.writeln(&format!(
                    "pub fn {method_name}(&self, v: enums::{enum_ty}) {{"
                ))?;
                out.indent();
                if call == "write" && lsb == 0 && (width as u64) == reg_bits {
                    out.writeln(&format!("self.write(v.bits() as {base_ty});"))?;
                } else if lsb == 0 {
                    out.writeln(&format!("self.{call}(v.bits() as {base_ty});"))?;
                } else {
                    out.writeln(&format!("let v = (v.bits() as u64) & 0x{mask:X}u64;"))?;
                    out.writeln(&format!("self.{call}((v << {lsb}) as {base_ty});"))?;
                }
                out.dedent();
                out.writeln("}")?;
            }
        }
    }

    if has_field_methods {
        out.dedent();
        out.writeln("}")?;
    }

    Ok(())
}

fn repr_for_bits(bits: u32) -> &'static str {
    match bits {
        0..=8 => "u8",
        9..=16 => "u16",
        17..=32 => "u32",
        _ => "u64",
    }
}
