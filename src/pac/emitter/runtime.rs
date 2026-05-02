use crate::Result;
use crate::pac::ir::*;
use crate::pac::emitter::common::CodeWriter;

use super::common::CodeWriter;

pub fn generate_cortex_m_rs(ir: &PacIr) -> Result<String> {
    let mut out = CodeWriter::new();

    out.writeln("#[allow(non_camel_case_types)]")?;
    out.writeln("use crate::Interrupt;")?;
    out.writeln("")?;

    out.writeln("#[inline(always)]")?;
    out.writeln("pub fn enable_irq(irq: Interrupt) {")?;
    out.indent();
    out.writeln("let n = irq as u32;")?;
    out.writeln("unsafe {")?;
    out.indent();
    out.writeln("let base = 0xE000_E100usize as *mut u32;")?;
    out.writeln("let reg = base.add((n / 32) as usize);")?;
    out.writeln("core::ptr::write_volatile(reg, 1u32 << (n % 32));")?;
    out.dedent();
    out.writeln("}")?;
    out.dedent();
    out.writeln("}")?;
    out.writeln("")?;

    out.writeln("#[inline(always)]")?;
    out.writeln("pub fn disable_irq(irq: Interrupt) {")?;
    out.indent();
    out.writeln("let n = irq as u32;")?;
    out.writeln("unsafe {")?;
    out.indent();
    out.writeln("let base = 0xE000_E180usize as *mut u32;")?;
    out.writeln("let reg = base.add((n / 32) as usize);")?;
    out.writeln("core::ptr::write_volatile(reg, 1u32 << (n % 32));")?;
    out.dedent();
    out.writeln("}")?;
    out.dedent();
    out.writeln("}")?;
    out.writeln("")?;

    out.writeln("#[inline(always)]")?;
    out.writeln("pub fn pending_irq(irq: Interrupt) -> bool {")?;
    out.indent();
    out.writeln("let n = irq as u32;")?;
    out.writeln("unsafe {")?;
    out.indent();
    out.writeln("let base = 0xE000_E200usize as *const u32;")?;
    out.writeln("let reg = base.add((n / 32) as usize);")?;
    out.writeln("(core::ptr::read_volatile(reg) & (1u32 << (n % 32))) != 0")?;
    out.dedent();
    out.writeln("}")?;
    out.dedent();
    out.writeln("}")?;
    out.writeln("")?;

    out.writeln("#[inline(always)]")?;
    out.writeln("pub fn unpending_irq(irq: Interrupt) {")?;
    out.indent();
    out.writeln("let n = irq as u32;")?;
    out.writeln("unsafe {")?;
    out.indent();
    out.writeln("let base = 0xE000_E280usize as *mut u32;")?;
    out.writeln("let reg = base.add((n / 32) as usize);")?;
    out.writeln("core::ptr::write_volatile(reg, 1u32 << (n % 32));")?;
    out.dedent();
    out.writeln("}")?;
    out.dedent();
    out.writeln("}")?;
    out.writeln("")?;

    out.writeln("#[inline(always)]")?;
    out.writeln("pub fn is_active_irq(irq: Interrupt) -> bool {")?;
    out.indent();
    out.writeln("let n = irq as u32;")?;
    out.writeln("unsafe {")?;
    out.indent();
    out.writeln("let active: *const u32 = (0xE000_E300usize as *const u32).add((n as usize / 32) as usize);")?;
    out.writeln("(core::ptr::read_volatile(active) & (1u32 << (n % 32))) != 0")?;
    out.dedent();
    out.writeln("}")?;
    out.dedent();
    out.writeln("}")?;
    out.writeln("")?;

    out.writeln("#[inline(always)]")?;
    out.writeln("pub fn set_priority(irq: Interrupt, prio: u8) {")?;
    out.indent();
    out.writeln("let n = irq as u32;")?;
    out.writeln("unsafe {")?;
    out.indent();
    out.writeln("let base = 0xE000_E400usize as *mut u8;")?;
    out.writeln("let reg = base.add(n as usize);")?;
    out.writeln("core::ptr::write_volatile(reg, prio);")?;
    out.dedent();
    out.writeln("}")?;
    out.dedent();
    out.writeln("}")?;
    out.writeln("")?;

    out.writeln("#[inline(always)]")?;
    out.writeln("pub fn software_trigger(irq: Interrupt) {")?;
    out.indent();
    out.writeln("let n = irq as u32;")?;
    out.writeln("unsafe {")?;
    out.indent();
    out.writeln("let stir: *mut u32 = 0xE000_EF00usize as *mut u32;")?;
    out.writeln("core::ptr::write_volatile(stir, n);")?;
    out.dedent();
    out.writeln("}")?;
    out.dedent();
    out.writeln("}")?;

    Ok(out.s)
}

pub fn generate_rt_rs(ir: &PacIr) -> String {
    let num_irqs = ir.interrupts.num_irqs;
    let irqs = &ir.interrupts.irqs;

    let mut out = CodeWriter::new();
    let _ = out.writeln("#[allow(non_snake_case)]");
    let _ = out.writeln("#[allow(dead_code)]");
    let _ = out.writeln("");
    let _ = out.writeln("use core::ptr;");
    let _ = out.writeln("");

    let _ = out.writeln("#[inline(always)]");
    let _ = out.writeln("pub unsafe fn enable_interrupts() {");
    out.indent();
    let _ = out.writeln("#[cfg(target_arch = \"arm\")]");
    let _ = out.writeln("core::arch::asm!(\"cpsie i\", options(nomem, nostack, preserves_flags));");
    out.dedent();
    let _ = out.writeln("}");
    let _ = out.writeln("#[inline(always)]");
    let _ = out.writeln("pub unsafe fn disable_interrupts() {");
    out.indent();
    let _ = out.writeln("#[cfg(target_arch = \"arm\")]");
    let _ = out.writeln("core::arch::asm!(\"cpsid i\", options(nomem, nostack, preserves_flags));");
    out.dedent();
    let _ = out.writeln("}");
    let _ = out.writeln("#[inline(always)]");
    let _ = out.writeln("pub unsafe fn wait_for_interrupt() {");
    out.indent();
    let _ = out.writeln("#[cfg(target_arch = \"arm\")]");
    let _ = out.writeln("core::arch::asm!(\"wfi\", options(nomem, nostack, preserves_flags));");
    out.dedent();
    let _ = out.writeln("}");
    let _ = out.writeln("");
    let _ = out.writeln("#[inline(always)]");
    let _ = out.writeln("pub unsafe fn nop() {");
    out.indent();
    let _ = out.writeln("#[cfg(target_arch = \"arm\")]");
    let _ = out.writeln("core::arch::asm!(\"nop\", options(nomem, nostack, preserves_flags));");
    out.dedent();
    let _ = out.writeln("}");
    let _ = out.writeln("");

    let _ = out.writeln("#[repr(C)]");
    let _ = out.writeln("pub union Vector {");
    out.indent();
    let _ = out.writeln("pub handler: unsafe extern \"C\" fn(),");
    let _ = out.writeln("pub reserved: usize,");
    out.dedent();
    let _ = out.writeln("}");
    let _ = out.writeln("");

    let _ = out.writeln("unsafe extern \"C\" {");
    out.indent();
    let _ = out.writeln("fn main() -> !;");
    out.dedent();
    let _ = out.writeln("}");
    let _ = out.writeln("");

    let _ = out.writeln("#[unsafe(no_mangle)]");
    let _ = out.writeln("pub unsafe extern \"C\" fn DefaultHandler() {");
    out.indent();
    let _ = out.writeln("loop {");
    out.indent();
    let _ = out.writeln("core::hint::spin_loop();");
    out.dedent();
    let _ = out.writeln("}");
    out.dedent();
    let _ = out.writeln("}");
    let _ = out.writeln("");

    let exception_names = [
        "NMI", "HardFault", "MemManage", "BusFault", "UsageFault", "SVCall",
        "DebugMonitor", "PendSV", "SysTick",
    ];
    for name in &exception_names {
        let _ = out.writeln("#[unsafe(no_mangle)]");
        let _ = out.writeln(&format!("pub unsafe extern \"C\" fn {name}() {{"));
        let _ = out.writeln("    DefaultHandler()");
        let _ = out.writeln("}");
        let _ = out.writeln("");
    }

    let irq_names: Vec<String> = irqs.iter().map(|(_, name, _)| name.clone()).collect();
    for name in &irq_names {
        let _ = out.writeln("#[unsafe(no_mangle)]");
        let _ = out.writeln(&format!("pub unsafe extern \"C\" fn {name}() {{"));
        let _ = out.writeln("    DefaultHandler()");
        let _ = out.writeln("}");
        let _ = out.writeln("");
    }

    let _ = out.writeln("#[unsafe(no_mangle)]");
    let _ = out.writeln("pub unsafe extern \"C\" fn Reset() {");
    out.indent();
    let _ = out.writeln("main()");
    out.dedent();
    let _ = out.writeln("}");
    let _ = out.writeln("");

    let _ = out.writeln("#[used]");
    let _ = out.writeln("#[unsafe(link_section = \".vector_table.reset_vector\")]");
    let _ = out.writeln("pub static __RESET_VECTOR: [Vector; 1] = [");
    out.indent();
    let _ = out.writeln("Vector { handler: Reset },");
    out.dedent();
    let _ = out.writeln("];");
    let _ = out.writeln("");

    let _ = out.writeln("#[used]");
    let _ = out.writeln("#[unsafe(link_section = \".vector_table.exceptions\")]");
    let _ = out.writeln("pub static __EXCEPTIONS: [Vector; 14] = [");
    out.indent();
    let _ = out.writeln("Vector { handler: NMI },");
    let _ = out.writeln("Vector { handler: HardFault },");
    let _ = out.writeln("Vector { handler: MemManage },");
    let _ = out.writeln("Vector { handler: BusFault },");
    let _ = out.writeln("Vector { handler: UsageFault },");
    let _ = out.writeln("Vector { reserved: 0 },");
    let _ = out.writeln("Vector { reserved: 0 },");
    let _ = out.writeln("Vector { reserved: 0 },");
    let _ = out.writeln("Vector { reserved: 0 },");
    let _ = out.writeln("Vector { handler: SVCall },");
    let _ = out.writeln("Vector { handler: DebugMonitor },");
    let _ = out.writeln("Vector { reserved: 0 },");
    let _ = out.writeln("Vector { handler: PendSV },");
    let _ = out.writeln("Vector { handler: SysTick },");
    out.dedent();
    let _ = out.writeln("];");
    let _ = out.writeln("");

    let mut irq_slots: Vec<String> = vec!["DefaultHandler".to_string(); num_irqs as usize];
    for (n, name, _desc) in irqs {
        if (*n as usize) < irq_slots.len() {
            irq_slots[*n as usize] = name.clone();
        }
    }

    let _ = out.writeln("#[unsafe(link_section = \".vector_table.interrupts\")]");
    let _ = out.writeln(&format!("pub static __INTERRUPTS: [Vector; {}usize] = [", num_irqs));
    out.indent();
    for h in &irq_slots {
        let _ = out.writeln(&format!("Vector {{ handler: {h} }},"));
    }
    out.dedent();
    let _ = out.writeln("];");

    out.s
}

pub fn generate_link_x(ir: &PacIr) -> Result<String> {
    let num_irqs = ir.interrupts.num_irqs;
    let irqs = &ir.interrupts.irqs;
    let mut out = CodeWriter::new();

    out.writeln("/* AUTO-GENERATED BY svdkit::pac (linker script) */")?;
    out.writeln("INCLUDE memory.x")?;
    out.writeln("")?;
    out.writeln("ENTRY(Reset)")?;
    out.writeln("")?;
    out.writeln("__stack_top = ORIGIN(RAM) + LENGTH(RAM);")?;
    out.writeln("")?;

    out.writeln("PROVIDE(_sidata = 0);")?;
    out.writeln("PROVIDE(_sdata = 0);")?;
    out.writeln("PROVIDE(_edata = 0);")?;
    out.writeln("PROVIDE(_sbss = 0);")?;
    out.writeln("PROVIDE(_ebss = 0);")?;
    out.writeln("")?;

    out.writeln("PROVIDE(NMI = DefaultHandler);")?;
    out.writeln("PROVIDE(HardFault = DefaultHandler);")?;
    out.writeln("PROVIDE(MemManage = DefaultHandler);")?;
    out.writeln("PROVIDE(BusFault = DefaultHandler);")?;
    out.writeln("PROVIDE(UsageFault = DefaultHandler);")?;
    out.writeln("PROVIDE(SVCall = DefaultHandler);")?;
    out.writeln("PROVIDE(DebugMonitor = DefaultHandler);")?;
    out.writeln("PROVIDE(PendSV = DefaultHandler);")?;
    out.writeln("PROVIDE(SysTick = DefaultHandler);")?;

    let mut slots: Vec<&str> = vec![""; num_irqs as usize];
    for (n, name, _) in irqs {
        if (*n as usize) < slots.len() {
            slots[*n as usize] = name.as_str();
        }
    }
    for name in &slots {
        if !name.is_empty() {
            out.writeln(&format!("PROVIDE({name} = DefaultHandler);"))?;
        }
    }

    Ok(out.s)
}

pub fn generate_enums_file(ir: &PacIr) -> String {
    let mut out = String::new();
    let irqs = &ir.interrupts.irqs;

    if !irqs.is_empty() {
        out.push_str("#[repr(u16)]\n");
        out.push_str("#[derive(Copy, Clone, Debug, PartialEq, Eq)]\n");
        out.push_str("pub enum Interrupt {\n");
        for (n, name, _desc) in irqs {
            out.push_str(&format!("    {name} = {n},\n"));
        }
        out.push_str("}\n");

        out.push_str("\nimpl Interrupt {\n");
        out.push_str("    #[inline]\n");
        out.push_str("    pub const fn bits(self) -> u16 {\n");
        out.push_str("        self as u16\n");
        out.push_str("    }\n\n");
        out.push_str("    #[inline]\n");
        out.push_str("    pub const fn from_bits(bits: u16) -> Option<Self> {\n");
        out.push_str("        match bits {\n");
        for (n, name, _desc) in irqs {
            out.push_str(&format!("            {n} => Some(Self::{name}),\n"));
        }
        out.push_str("            _ => None,\n");
        out.push_str("        }\n");
        out.push_str("    }\n");
        out.push_str("}\n");

        out.push_str("\nimpl From<Interrupt> for u16 {\n");
        out.push_str("    #[inline]\n");
        out.push_str("    fn from(intr: Interrupt) -> u16 {\n");
        out.push_str("        intr.bits()\n");
        out.push_str("    }\n");
        out.push_str("}\n");

        out.push_str("\nimpl From<Interrupt> for u32 {\n");
        out.push_str("    #[inline]\n");
        out.push_str("    fn from(intr: Interrupt) -> u32 {\n");
        out.push_str("        intr.bits() as u32\n");
        out.push_str("    }\n");
        out.push_str("}\n");

        out.push_str("\nimpl TryFrom<u16> for Interrupt {\n");
        out.push_str("    type Error = ();\n");
        out.push_str("    #[inline]\n");
        out.push_str("    fn try_from(bits: u16) -> core::result::Result<Self, ()> {\n");
        out.push_str("        Self::from_bits(bits).ok_or(())\n");
        out.push_str("    }\n");
        out.push_str("}\n");

        out.push_str("\nimpl TryFrom<u32> for Interrupt {\n");
        out.push_str("    type Error = ();\n");
        out.push_str("    #[inline]\n");
        out.push_str("    fn try_from(bits: u32) -> core::result::Result<Self, ()> {\n");
        out.push_str("        Self::from_bits(bits as u16).ok_or(())\n");
        out.push_str("    }\n");
        out.push_str("}\n");
    }

    out
}

pub fn generate_constants_file(ir: &PacIr) -> String {
    let num_irqs = ir.interrupts.num_irqs;
    let prio_bits = ir
        .device_info
        .cpu
        .as_ref()
        .map(|c| c.nvic_prio_bits)
        .unwrap_or(8)
        .min(8);

    format!(
        "pub const DEVICE_NAME: &str = {:?};\npub const DEVICE_DESCRIPTION: &str = {:?};\n\npub const _NUM_IRQS: u32 = {num_irqs}u32;\npub const _PRIO_BITS: u8 = {prio_bits}u8;\n",
        ir.device_info.name, ir.device_info.description
    )
}

pub fn generate_memory_x(ir: &PacIr) -> Result<String> {
    let mut out = CodeWriter::new();
    out.writeln("MEMORY")?;
    out.writeln("{")?;
    out.indent();

    let flash_regions: Vec<_> = ir
        .memory_regions
        .iter()
        .filter(|r| r.kind == MemoryKind::Flash)
        .collect();
    let ram_regions: Vec<_> = ir
        .memory_regions
        .iter()
        .filter(|r| r.kind == MemoryKind::Ram)
        .collect();

    if flash_regions.is_empty() {
        out.writeln("FLASH (rx) : ORIGIN = 0x00000000, LENGTH = 256K")?;
    } else {
        for r in &flash_regions {
            out.writeln(&format!(
                "FLASH (rx) : ORIGIN = 0x{base:08X}, LENGTH = {size}K",
                base = r.base,
                size = r.size / 1024
            ))?;
        }
    }

    if ram_regions.is_empty() {
        out.writeln("RAM (rwx)  : ORIGIN = 0x20000000, LENGTH = 64K")?;
    } else {
        for r in &ram_regions {
            out.writeln(&format!(
                "RAM (rwx)  : ORIGIN = 0x{base:08X}, LENGTH = {size}K",
                base = r.base,
                size = r.size / 1024
            ))?;
        }
    }

    out.dedent();
    out.writeln("}")?;
    Ok(out.s)
}
