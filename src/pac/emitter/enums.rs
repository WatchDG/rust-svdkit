use crate::Result;
use crate::pac::ir::*;
use crate::pac::emitter::common::CodeWriter;

pub fn generate_enums_for_peripheral(periph: &PeripheralIr) -> Result<String> {
    let mut out = CodeWriter::new();
    for edef in &periph.field_enums {
        emit_define_enum(&mut out, edef)?;
        out.writeln("")?;
    }
    Ok(out.s)
}

fn emit_define_enum(out: &mut CodeWriter, edef: &EnumDef) -> Result<()> {
    out.writeln(&format!(
        r#"    #[doc = "{}"]"#,
        edef.doc.replace('\"', "\\\"")
    ))?;

    let repr = &edef.repr;
    let ty = &edef.type_name;

    let mut variants_str = String::new();
    variants_str.push_str(&format!("\n    {ty} : {repr},"));

    let last_idx = edef.variants.len().saturating_sub(1);
    let mut any_numeric = false;

    for (i, v) in edef.variants.iter().enumerate() {
        if let Some(d) = &v.description {
            variants_str.push_str(&format!(
                "\n    #[doc = \"{}\"]",
                d.replace('\"', "\\\"")
            ));
        }
        if let Some(val) = v.value {
            any_numeric = true;
            let comma = if i < last_idx { "," } else { "" };
            variants_str.push_str(&format!("\n    {} = {val}{comma}", v.name));
        } else {
            variants_str.push_str(&format!("\n    // {} = <non-const value>,", v.name));
        }
    }

    if !any_numeric {
        variants_str.push_str(
            r#"
    #[doc = "No fully-constant values in SVD; placeholder."]"#,
        );
        variants_str.push_str("\n    __Reserved = 0");
    }

    out.writeln(&format!("define_enum!(\n{variants_str}\n);"))?;
    Ok(())
}
