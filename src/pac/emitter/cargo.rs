use crate::Result;
use crate::pac::ir::*;
use crate::pac::emitter::common::CodeWriter;

pub fn generate_cargo_toml(ir: &PacIr, dir_name: &str) -> Result<String> {
    let mut lines = Vec::new();
    lines.push("[package]".to_string());
    lines.push(format!("name = {:?}", dir_name));
    lines.push("version = \"0.1.0\"".to_string());
    lines.push("edition = \"2024\"".to_string());
    lines.push(format!("description = \"PAC for {}\"", ir.device_info.name));
    lines.push("".to_string());
    lines.push("[lib]".to_string());
    lines.push(format!("name = {:?}", dir_name));
    lines.push("path = \"lib.rs\"".to_string());
    lines.push("".to_string());
    lines.push("[dependencies]".to_string());
    Ok(lines.join("\n"))
}
