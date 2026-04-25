//! PAC code generation from the parsed CMSIS-SVD model.
//!
//! This module intentionally keeps the output minimal and dependency-free:
//! it generates `#[repr(C)]` register blocks with primitive integer fields and
//! `_reservedN: [u8; ..]` paddings to match offsets.
//!
//! The generator is *best-effort*: some CMSIS-SVD features (complex `dim`
//! layouts, unions/alternates, etc.) are emitted as byte arrays with comments
//! when an exact Rust representation would require heavier abstractions.

use crate::{Result, svd};
use std::collections::BTreeMap;
use std::path::Path;

mod helpers;

/// A single generated file.
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    pub file_name: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct GeneratedDir {
    pub path: String,
    pub files: Vec<GeneratedFile>,
}

#[derive(Debug, Clone, Copy)]
pub struct PacOptions {
    pub emit_field_enums: bool,
    pub emit_field_methods: bool,
}

impl PacOptions {
    pub const fn full() -> Self {
        Self {
            emit_field_enums: true,
            emit_field_methods: true,
        }
    }

    pub const fn minimal() -> Self {
        Self {
            emit_field_enums: false,
            emit_field_methods: false,
        }
    }
}

/// Generate PAC output as a directory structure.
/// Each peripheral gets its own directory with a `mod.rs` file.
/// The root contains `{device}_pac/mod.rs` with all peripherals as submodules.
pub fn generate_device_dir(device: &svd::Device) -> Result<GeneratedDir> {
    generate_device_dir_with_options(device, PacOptions::full())
}

pub fn generate_macros_file() -> String {
    r#"macro_rules! impl_ro_register {
    ($name:ident, $ty:ty) => {
        impl $name {
            #[inline(always)]
            pub fn read(&self) -> $ty {
                self.0.read()
            }
        }
    };
}

macro_rules! impl_wo_register {
    ($name:ident, $ty:ty) => {
        impl $name {
            #[inline(always)]
            pub fn write(&self, v: $ty) {
                self.0.write(v)
            }
        }
    };
}

macro_rules! impl_rw_register {
    ($name:ident, $ty:ty) => {
        impl $name {
            #[inline(always)]
            pub fn read(&self) -> $ty {
                self.0.read()
            }
            #[inline(always)]
            pub fn write(&self, v: $ty) {
                self.0.write(v)
            }
        }
    };
}

macro_rules! impl_w1s_register {
    ($name:ident, $ty:ty) => {
        impl $name {
            #[inline(always)]
            pub fn read(&self) -> $ty {
                self.0.read()
            }
            #[inline(always)]
            pub fn write(&self, v: $ty) {
                self.0.write(v)
            }
            #[inline(always)]
            pub fn set_bits(&self, mask: $ty) {
                self.0.set_bits(mask)
            }
        }
    };
}

macro_rules! impl_w1c_register {
    ($name:ident, $ty:ty) => {
        impl $name {
            #[inline(always)]
            pub fn read(&self) -> $ty {
                self.0.read()
            }
            #[inline(always)]
            pub fn write(&self, v: $ty) {
                self.0.write(v)
            }
            #[inline(always)]
            pub fn clear_bits(&self, mask: $ty) {
                self.0.clear_bits(mask)
            }
        }
    };
}

macro_rules! impl_w0s_register {
    ($name:ident, $ty:ty) => {
        impl $name {
            #[inline(always)]
            pub fn read(&self) -> $ty {
                self.0.read()
            }
            #[inline(always)]
            pub fn write(&self, v: $ty) {
                self.0.write(v)
            }
            #[inline(always)]
            pub fn set_bits(&self, mask: $ty) {
                self.0.set_bits(mask)
            }
        }
    };
}

macro_rules! impl_w0c_register {
    ($name:ident, $ty:ty) => {
        impl $name {
            #[inline(always)]
            pub fn read(&self) -> $ty {
                self.0.read()
            }
            #[inline(always)]
            pub fn write(&self, v: $ty) {
                self.0.write(v)
            }
            #[inline(always)]
            pub fn clear_bits(&self, mask: $ty) {
                self.0.clear_bits(mask)
            }
        }
    };
}

macro_rules! impl_wt_register {
    ($name:ident, $ty:ty) => {
        impl $name {
            #[inline(always)]
            pub fn read(&self) -> $ty {
                self.0.read()
            }
            #[inline(always)]
            pub fn write(&self, v: $ty) {
                self.0.write(v)
            }
            #[inline(always)]
            pub fn toggle_bits(&self, mask: $ty) {
                self.0.toggle_bits(mask)
            }
        }
    };
}

macro_rules! define_enum {
    (
        $(#[$doc:meta])*
        $name:ident : $type:ty,
        $(
            $(#[$vdoc:meta])*
            $variant:ident = $value:expr
        ),+ $(,)?
    ) => {
        $(#[$doc])*
        #[repr($type)]
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub enum $name {
            $(
                $(#[$vdoc])*
                $variant = $value,
            )*
        }

        impl $name {
            #[inline(always)]
            pub const fn bits(self) -> $type {
                self as $type
            }

            #[inline(always)]
            pub const fn from_bits(v: $type) -> Option<Self> {
                match v {
                    $(
                        $value => Some(Self::$variant),
                    )*
                    _ => None,
                }
            }
        }

        impl From<$name> for $type {
            #[inline(always)]
            fn from(v: $name) -> $type {
                v.bits()
            }
        }

        impl core::convert::TryFrom<$type> for $name {
            type Error = ();
            #[inline(always)]
            fn try_from(v: $type) -> core::result::Result<Self, ()> {
                Self::from_bits(v).ok_or(())
            }
        }
    };
}
"#
    .to_string()
}

pub fn generate_traits_file() -> String {
    r#"pub trait RegValue: Copy {
    const BITS: u32;
    const MASK: u64;
    fn to_u64(self) -> u64;
    fn from_u64(v: u64) -> Self;
}

impl RegValue for u8 {
    const BITS: u32 = 8;
    const MASK: u64 = 0xFFu64;
    #[inline(always)]
    fn to_u64(self) -> u64 { self as u64 }
    #[inline(always)]
    fn from_u64(v: u64) -> Self { v as u8 }
}

impl RegValue for u16 {
    const BITS: u32 = 16;
    const MASK: u64 = 0xFFFFu64;
    #[inline(always)]
    fn to_u64(self) -> u64 { self as u64 }
    #[inline(always)]
    fn from_u64(v: u64) -> Self { v as u16 }
}

impl RegValue for u32 {
    const BITS: u32 = 32;
    const MASK: u64 = 0xFFFF_FFFFu64;
    #[inline(always)]
    fn to_u64(self) -> u64 { self as u64 }
    #[inline(always)]
    fn from_u64(v: u64) -> Self { v as u32 }
}

impl RegValue for u64 {
    const BITS: u32 = 64;
    const MASK: u64 = 0xFFFF_FFFF_FFFF_FFFFu64;
    #[inline(always)]
    fn to_u64(self) -> u64 { self as u64 }
    #[inline(always)]
    fn from_u64(v: u64) -> Self { v as u64 }
}
"#
    .to_string()
}

pub fn generate_types_file() -> String {
    r#"use core::marker::PhantomData;
use super::traits::RegValue;

#[repr(transparent)]
pub struct RO<T>(core::cell::UnsafeCell<T>);

#[repr(transparent)]
pub struct WO<T>(core::cell::UnsafeCell<T>);

#[repr(transparent)]
pub struct RW<T>(core::cell::UnsafeCell<T>);

#[repr(transparent)]
pub struct W1S<T>(core::cell::UnsafeCell<T>);

#[repr(transparent)]
pub struct W1C<T>(core::cell::UnsafeCell<T>);

#[repr(transparent)]
pub struct W0S<T>(core::cell::UnsafeCell<T>);

#[repr(transparent)]
pub struct W0C<T>(core::cell::UnsafeCell<T>);

#[repr(transparent)]
pub struct WT<T>(core::cell::UnsafeCell<T>);

unsafe impl<T> Send for RO<T> {}
unsafe impl<T> Sync for RO<T> {}
unsafe impl<T> Send for WO<T> {}
unsafe impl<T> Sync for WO<T> {}
unsafe impl<T> Send for RW<T> {}
unsafe impl<T> Sync for RW<T> {}
unsafe impl<T> Send for W1S<T> {}
unsafe impl<T> Sync for W1S<T> {}
unsafe impl<T> Send for W1C<T> {}
unsafe impl<T> Sync for W1C<T> {}
unsafe impl<T> Send for W0S<T> {}
unsafe impl<T> Sync for W0S<T> {}
unsafe impl<T> Send for W0C<T> {}
unsafe impl<T> Sync for W0C<T> {}
unsafe impl<T> Send for WT<T> {}
unsafe impl<T> Sync for WT<T> {}

impl<T: Copy> RO<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(self.0.get()) }
    }
}

impl<T: Copy> WO<T> {
    #[inline(always)]
    pub fn write(&self, v: T) {
        unsafe { core::ptr::write_volatile(self.0.get(), v) }
    }
}

impl<T: Copy> RW<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(self.0.get()) }
    }
    #[inline(always)]
    pub fn write(&self, v: T) {
        unsafe { core::ptr::write_volatile(self.0.get(), v) }
    }
}

impl<T: RegValue> W1S<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(self.0.get()) }
    }
    #[inline(always)]
    pub fn write(&self, v: T) {
        unsafe { core::ptr::write_volatile(self.0.get(), v) }
    }
    #[inline(always)]
    pub fn set_bits(&self, mask: T) {
        self.write(mask)
    }
}

impl<T: RegValue> W1C<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(self.0.get()) }
    }
    #[inline(always)]
    pub fn write(&self, v: T) {
        unsafe { core::ptr::write_volatile(self.0.get(), v) }
    }
    #[inline(always)]
    pub fn clear_bits(&self, mask: T) {
        self.write(mask)
    }
}

impl<T: RegValue> W0S<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(self.0.get()) }
    }
    #[inline(always)]
    pub fn write(&self, v: T) {
        unsafe { core::ptr::write_volatile(self.0.get(), v) }
    }
    #[inline(always)]
    pub fn set_bits(&self, mask: T) {
        let m = mask.to_u64() & T::MASK;
        let v = (!m) & T::MASK;
        self.write(T::from_u64(v));
    }
}

impl<T: RegValue> W0C<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(self.0.get()) }
    }
    #[inline(always)]
    pub fn write(&self, v: T) {
        unsafe { core::ptr::write_volatile(self.0.get(), v) }
    }
    #[inline(always)]
    pub fn clear_bits(&self, mask: T) {
        let m = mask.to_u64() & T::MASK;
        let v = (!m) & T::MASK;
        self.write(T::from_u64(v));
    }
}

impl<T: RegValue> WT<T> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(self.0.get()) }
    }
    #[inline(always)]
    pub fn write(&self, v: T) {
        unsafe { core::ptr::write_volatile(self.0.get(), v) }
    }
    #[inline(always)]
    pub fn toggle_bits(&self, mask: T) {
        self.write(mask)
    }
}

pub struct Unwritten;
pub struct Written;
pub struct WOOnce<T, S> {
    pub base: usize,
    pub offset: usize,
    pub _state: PhantomData<S>,
    pub _t: PhantomData<T>,
}
pub struct RWOnce<T, S> {
    pub base: usize,
    pub offset: usize,
    pub _state: PhantomData<S>,
    pub _t: PhantomData<T>,
}

impl<T: Copy, S> RWOnce<T, S> {
    #[inline(always)]
    pub unsafe fn read(&self) -> T {
        let p = (self.base + self.offset) as *const RW<T>;
        (*p).read()
    }
}

impl<T: Copy> WOOnce<T, Unwritten> {
    #[inline(always)]
    pub unsafe fn write(self, v: T) -> WOOnce<T, Written> {
        let p = (self.base + self.offset) as *const WO<T>;
        (*p).write(v);
        WOOnce {
            base: self.base,
            offset: self.offset,
            _state: PhantomData,
            _t: PhantomData,
        }
    }
}

impl<T: Copy> RWOnce<T, Unwritten> {
    #[inline(always)]
    pub unsafe fn write(self, v: T) -> RWOnce<T, Written> {
        let p = (self.base + self.offset) as *const RW<T>;
        (*p).write(v);
        RWOnce {
            base: self.base,
            offset: self.offset,
            _state: PhantomData,
            _t: PhantomData,
        }
    }
}
"#
    .to_string()
}

pub fn generate_enums_file(device: &svd::Device) -> String {
    let mut out = String::new();
    let (_num_irqs, irqs) = collect_device_interrupts(device);

    if !irqs.is_empty() {
        out.push_str("#[repr(u16)]\n");
        out.push_str("#[derive(Copy, Clone, Debug, PartialEq, Eq)]\n");
        out.push_str("pub enum Interrupt {\n");
        for (n, name, _desc) in &irqs {
            out.push_str(&format!("    {name} = {n},\n"));
        }
        out.push_str("}\n");
    }

    out
}

pub fn generate_constants_file(device: &svd::Device) -> String {
    let (num_irqs, _) = collect_device_interrupts(device);
    let prio_bits = device
        .cpu
        .as_ref()
        .map(|c| c.nvic_prio_bits)
        .unwrap_or(8)
        .min(8);

    format!(
        "pub const DEVICE_NAME: &str = {:?};\npub const DEVICE_DESCRIPTION: &str = {:?};\n\npub const _NUM_IRQS: u32 = {num_irqs}u32;\npub const _PRIO_BITS: u8 = {prio_bits}u8;\n",
        device.name, device.description
    )
}

pub fn generate_device_dir_with_options(
    device: &svd::Device,
    options: PacOptions,
) -> Result<GeneratedDir> {
    let dir_name = format!("{}_pac", sanitize_file_stem(&device.name));
    let mut files = Vec::new();

    let mut periphs = device.peripherals.clone();
    periphs.sort_by(|a, b| {
        a.base_address
            .cmp(&b.base_address)
            .then(a.name.cmp(&b.name))
    });

    let mut mod_lines = Vec::new();
    mod_lines.push("#[allow(non_snake_case)]".to_string());
    mod_lines.push("#[allow(non_camel_case_types)]".to_string());
    mod_lines.push("#[allow(dead_code)]".to_string());
    mod_lines.push("#[allow(unused_imports)]".to_string());
    mod_lines.push("#[allow(unsafe_op_in_unsafe_fn)]".to_string());
    mod_lines.push("".to_string());

    mod_lines.push("pub mod traits;".to_string());
    mod_lines.push("pub mod types;".to_string());
    mod_lines.push("pub mod enums;".to_string());
    mod_lines.push("pub mod constants;".to_string());
    mod_lines.push(
        "use types::{RW, RO, WO, W1S, W1C, W0S, W0C, WT, RWOnce, WOOnce, Unwritten, Written};"
            .to_string(),
    );
    mod_lines.push("".to_string());

    mod_lines.push("#[macro_use]".to_string());
    mod_lines.push("pub mod macros;".to_string());
    mod_lines.push("".to_string());

    mod_lines.push("pub mod peripherals;".to_string());
    mod_lines.push("".to_string());

    files.push(GeneratedFile {
        file_name: "mod.rs".to_string(),
        content: mod_lines.join("\n"),
    });

    files.push(GeneratedFile {
        file_name: "macros.rs".to_string(),
        content: generate_macros_file(),
    });

    files.push(GeneratedFile {
        file_name: "traits.rs".to_string(),
        content: generate_traits_file(),
    });

    files.push(GeneratedFile {
        file_name: "types.rs".to_string(),
        content: generate_types_file(),
    });

    files.push(GeneratedFile {
        file_name: "enums.rs".to_string(),
        content: generate_enums_file(device),
    });

    files.push(GeneratedFile {
        file_name: "constants.rs".to_string(),
        content: generate_constants_file(device),
    });

    files.push(GeneratedFile {
        file_name: "peripherals/mod.rs".to_string(),
        content: {
            let mut lines = Vec::new();
            lines.push("#[allow(non_snake_case)]".to_string());
            lines.push("#[allow(non_camel_case_types)]".to_string());
            lines.push("#[allow(dead_code)]".to_string());
            lines.push("#[allow(unused_imports)]".to_string());
            lines.push("#[allow(unsafe_op_in_unsafe_fn)]".to_string());
            lines.push("".to_string());
            for p in &periphs {
                let mod_name = sanitize_module_name(&p.name);
                lines.push(format!("pub mod {mod_name};"));
            }
            lines.join("\n")
        },
    });

    let mut st = GenState::new();
    let mut type_defs = CodeWriter::new();

    for p in &periphs {
        let (mod_content, regs_content, enums_content) =
            generate_peripheral_file_with_enums(device, p, &mut st, &mut type_defs, options)?;
        let mod_name = sanitize_module_name(&p.name);
        files.push(GeneratedFile {
            file_name: format!("peripherals/{}/mod.rs", mod_name),
            content: mod_content,
        });
        files.push(GeneratedFile {
            file_name: format!("peripherals/{}/registers.rs", mod_name),
            content: regs_content,
        });
        if !enums_content.is_empty() {
            files.push(GeneratedFile {
                file_name: format!("peripherals/{}/enums.rs", mod_name),
                content: enums_content,
            });
        }

        let items = peripheral_register_items(device, p);
        let has_clusters = items
            .iter()
            .any(|item| matches!(item, svd::RegisterBlockItem::Cluster { .. }));

        if has_clusters {
            let cluster_files = generate_cluster_dir_for_peripheral(device, p, &mut st, options)?;

            if let Some(mod_file) = files
                .iter_mut()
                .find(|f| f.file_name == format!("peripherals/{}/mod.rs", mod_name))
            {
                mod_file.content = mod_file.content.replace(
                    "pub mod registers;",
                    "pub mod registers;\npub mod clusters;",
                );
            }

            let mut clusters_mod_lines = Vec::new();
            clusters_mod_lines.push("#[allow(non_snake_case)]".to_string());
            clusters_mod_lines.push("#[allow(non_camel_case_types)]".to_string());
            clusters_mod_lines.push("#[allow(dead_code)]".to_string());
            clusters_mod_lines.push("#[allow(unused_imports)]".to_string());
            clusters_mod_lines.push("#[allow(unsafe_op_in_unsafe_fn)]".to_string());
            clusters_mod_lines.push("".to_string());

            for cf in &cluster_files {
                if cf.file_name.ends_with("/mod.rs") {
                    let mod_dir_name = cf.file_name.trim_end_matches("/mod.rs");
                    clusters_mod_lines.push(format!("pub mod {mod_dir_name};"));
                }
            }

            files.push(GeneratedFile {
                file_name: format!("peripherals/{}/clusters/mod.rs", mod_name),
                content: clusters_mod_lines.join("\n"),
            });

            for cf in cluster_files {
                let file_name = format!("peripherals/{}/clusters/{}", mod_name, cf.file_name);
                files.push(GeneratedFile {
                    file_name,
                    content: cf.content,
                });
            }
        }
    }

    Ok(GeneratedDir {
        path: dir_name,
        files,
    })
}

fn generate_peripheral_file(
    device: &svd::Device,
    p: &svd::Peripheral,
    st: &mut GenState,
    type_defs: &mut CodeWriter,
    options: PacOptions,
) -> Result<(String, String, String)> {
    let mut mod_out = CodeWriter::new();
    let mut regs_out = CodeWriter::new();

    regs_out.writeln("use super::super::super::types::{RW, RO, WO, W1S, W1C, W0S, W0C, WT};")?;
    regs_out.writeln("use super::super::super::macros::*;")?;
    regs_out.writeln("")?;

    mod_out.writeln("#[allow(non_snake_case)]")?;
    mod_out.writeln("#[allow(non_camel_case_types)]")?;
    mod_out.writeln("#[allow(dead_code)]")?;
    mod_out.writeln("")?;
    mod_out.writeln(&format!(
        "pub const BASE: usize = 0x{:08X};",
        p.base_address
    ))?;
    mod_out.writeln("")?;

    let items = peripheral_register_items(device, p);
    let ctx = Ctx {
        device,
        periph: Some(p),
        cluster_stack: Vec::new(),
    };

    let register_items: Vec<_> = items
        .iter()
        .filter(|item| matches!(item, svd::RegisterBlockItem::Register { .. }))
        .cloned()
        .collect();

    emit_register_block_items(
        st,
        &mut CodeWriter::new(),
        &mut regs_out,
        type_defs,
        &ctx,
        &register_items,
        0,
        options,
    )?;

    let mut enums_out = CodeWriter::new();
    if options.emit_field_enums {
        emit_peripheral_enums(st, device, p, &mut enums_out)?;
    }
    let has_enums = !enums_out.s.trim().is_empty();

    if has_enums {
        mod_out.writeln("pub mod enums;")?;
        mod_out.writeln("use self::enums as field_enums;")?;
        mod_out.writeln("")?;
    }

    if !type_defs.s.trim().is_empty() {
        regs_out.s.push_str(&type_defs.s);
        regs_out.s.push('\n');
        type_defs.s.clear();
    }

    mod_out.writeln("use core::marker::PhantomData;")?;
    mod_out.writeln("use super::super::macros;")?;
    mod_out.writeln("use super::super::types::{RW, RO, WO, W1S, W1C, W0S, W0C, WT, RWOnce, WOOnce, Unwritten, Written};")?;
    mod_out.writeln("pub mod registers;")?;
    mod_out.writeln("use registers::*;")?;
    mod_out.writeln("")?;

    mod_out.writeln("#[repr(C)]")?;
    mod_out.writeln("pub struct RegisterBlock {")?;
    mod_out.indent();

    emit_register_block_items(
        st,
        &mut mod_out,
        &mut CodeWriter::new(),
        type_defs,
        &ctx,
        &register_items,
        0,
        options,
    )?;

    mod_out.dedent();
    mod_out.writeln("}")?;

    emit_reset_impl_for_struct(
        &mut mod_out,
        st,
        type_defs,
        &ctx,
        &register_items,
        "RegisterBlock",
        options,
    )?;

    mod_out.writeln(&format!(
        "pub const PTR: *const RegisterBlock = BASE as *const RegisterBlock;"
    ))?;
    mod_out.writeln(&format!(
        "pub const PTR_MUT: *mut RegisterBlock = BASE as *mut RegisterBlock;"
    ))?;

    let mut once_regs = Vec::new();
    let mut name_counts = BTreeMap::new();
    collect_once_regs(
        &ctx,
        &register_items,
        0,
        "",
        None,
        &mut name_counts,
        &mut once_regs,
    );
    if !once_regs.is_empty() {
        let once_ty = "Once";
        mod_out.writeln("")?;
        mod_out.writeln("/// Compile-time writeOnce/read-writeOnce tokens (state-machine API).")?;
        mod_out.writeln("///")?;
        mod_out.writeln("/// NOTE: this enforces \"once\" only for code using this token API.")?;
        mod_out.writeln(&format!("pub struct {once_ty} {{"))?;
        mod_out.indent();
        mod_out.writeln("base: usize,")?;
        for r in &once_regs {
            mod_out.writeln(&format!("pub {}: {},", r.field_name, r.token_ty))?;
        }
        mod_out.dedent();
        mod_out.writeln("}")?;
        mod_out.writeln(&format!("impl {once_ty} {{"))?;
        mod_out.indent();
        mod_out.writeln("/// Create tokens for this peripheral.")?;
        mod_out.writeln("///")?;
        mod_out.writeln("/// # Safety")?;
        mod_out.writeln("/// The returned tokens allow volatile access to MMIO.")?;
        mod_out.writeln("pub unsafe fn new() -> Self {")?;
        mod_out.indent();
        mod_out.writeln("let base = BASE;")?;
        mod_out.writeln("Self {")?;
        mod_out.indent();
        mod_out.writeln("base,")?;
        for r in &once_regs {
            mod_out.writeln(&format!("{}: {},", r.field_name, r.init_expr))?;
        }
        mod_out.dedent();
        mod_out.writeln("}")?;
        mod_out.dedent();
        mod_out.writeln("}")?;
        mod_out.dedent();
        mod_out.writeln("}")?;
    }

    Ok((
        mod_out.into_string(),
        regs_out.into_string(),
        enums_out.into_string(),
    ))
}

fn generate_peripheral_file_with_enums(
    device: &svd::Device,
    p: &svd::Peripheral,
    st: &mut GenState,
    type_defs: &mut CodeWriter,
    options: PacOptions,
) -> Result<(String, String, String)> {
    let (mod_out, regs_out, enums_out) =
        generate_peripheral_file(device, p, st, type_defs, options)?;
    let enums_with_import = if enums_out.trim().is_empty() {
        enums_out
    } else {
        enums_out
    };
    Ok((mod_out, regs_out, enums_with_import))
}

pub fn generate_cluster_dir_for_peripheral(
    device: &svd::Device,
    p: &svd::Peripheral,
    st: &mut GenState,
    options: PacOptions,
) -> Result<Vec<GeneratedFile>> {
    let items = peripheral_register_items(device, p);
    let ctx = Ctx {
        device,
        periph: Some(p),
        cluster_stack: Vec::new(),
    };

    collect_clusters(items, &ctx, st, options, 0)
}

fn collect_clusters(
    items: &[svd::RegisterBlockItem],
    ctx: &Ctx<'_>,
    st: &mut GenState,
    options: PacOptions,
    depth: usize,
) -> Result<Vec<GeneratedFile>> {
    let mut files = Vec::new();

    for item in items {
        if let svd::RegisterBlockItem::Cluster { cluster } = item {
            let cluster_files = generate_cluster_files(cluster, ctx, st, options, depth)?;
            files.extend(cluster_files);
        }
    }

    Ok(files)
}

fn generate_cluster_files(
    c: &svd::Cluster,
    ctx: &Ctx<'_>,
    st: &mut GenState,
    options: PacOptions,
    depth: usize,
) -> Result<Vec<GeneratedFile>> {
    let mut files = Vec::new();
    let cluster_mod_name = sanitize_module_name(&c.name);

    let mut mod_out = CodeWriter::new();
    let mut regs_out = CodeWriter::new();
    let mut type_defs = CodeWriter::new();

    let periph_name = ctx
        .periph
        .map(|p| sanitize_module_name(&p.name.to_lowercase()))
        .unwrap_or_default();

    regs_out.writeln(&format!(
        "use crate::nrf52840_pac::peripherals::{periph_name}::enums as field_enums;"
    ))?;
    regs_out.writeln("use crate::nrf52840_pac::types::{RW, RO, WO, W1S, W1C, W0S, W0C, WT};")?;
    regs_out.writeln("use crate::nrf52840_pac::macros::*;")?;
    regs_out.writeln("")?;

    mod_out.writeln("#[allow(non_snake_case)]")?;
    mod_out.writeln("#[allow(non_camel_case_types)]")?;
    mod_out.writeln("#[allow(dead_code)]")?;
    mod_out.writeln("#[allow(unused_imports)]")?;
    mod_out.writeln("#[allow(unsafe_op_in_unsafe_fn)]")?;
    mod_out.writeln("")?;

    let child_ctx = Ctx {
        device: ctx.device,
        periph: ctx.periph,
        cluster_stack: {
            let mut s = ctx.cluster_stack.clone();
            s.push(c);
            s
        },
    };

    emit_register_block_items(
        st,
        &mut CodeWriter::new(),
        &mut regs_out,
        &mut type_defs,
        &child_ctx,
        &c.items,
        0,
        options,
    )?;

    if !type_defs.s.trim().is_empty() {
        regs_out.s.push_str(&type_defs.s);
        regs_out.s.push('\n');
        type_defs.s.clear();
    }

    mod_out.writeln("use crate::nrf52840_pac::types::{RW, RO, WO, W1S, W1C, W0S, W0C, WT, RWOnce, WOOnce, Unwritten, Written};")?;
    mod_out.writeln("use crate::nrf52840_pac::macros;")?;
    mod_out.writeln(&format!(
        "use crate::nrf52840_pac::peripherals::{periph_name}::enums;"
    ))?;
    mod_out.writeln("pub mod registers;")?;
    mod_out.writeln("use registers::*;")?;
    mod_out.writeln("")?;

    mod_out.writeln("#[repr(C)]")?;
    mod_out.writeln(&format!("pub struct {} {{", sanitize_type_name(&c.name)))?;
    mod_out.indent();

    emit_register_block_items(
        st,
        &mut mod_out,
        &mut CodeWriter::new(),
        &mut type_defs,
        &child_ctx,
        &c.items,
        0,
        options,
    )?;

    mod_out.dedent();
    mod_out.writeln("}")?;

    emit_reset_impl_for_struct(
        &mut mod_out,
        st,
        &mut type_defs,
        &child_ctx,
        &c.items,
        &sanitize_type_name(&c.name),
        options,
    )?;

    if !type_defs.s.trim().is_empty() {
        regs_out.s.push_str(&type_defs.s);
        regs_out.s.push('\n');
    }

    let has_nested_clusters = c
        .items
        .iter()
        .any(|it| matches!(it, svd::RegisterBlockItem::Cluster { .. }));
    if has_nested_clusters {
        mod_out.writeln("pub mod clusters;")?;
    }

    files.push(GeneratedFile {
        file_name: format!("{}/mod.rs", cluster_mod_name),
        content: mod_out.into_string(),
    });
    files.push(GeneratedFile {
        file_name: format!("{}/registers.rs", cluster_mod_name),
        content: regs_out.into_string(),
    });

    let nested_files = collect_clusters(&c.items, &child_ctx, st, options, depth + 1)?;
    for f in nested_files {
        files.push(GeneratedFile {
            file_name: format!("{}/{}", cluster_mod_name, f.file_name),
            content: f.content,
        });
    }

    if has_nested_clusters {
        let mut clusters_mod = CodeWriter::new();
        clusters_mod.writeln("#[allow(non_snake_case)]")?;
        clusters_mod.writeln("#[allow(non_camel_case_types)]")?;
        clusters_mod.writeln("#[allow(dead_code)]")?;
        clusters_mod.writeln("#[allow(unused_imports)]")?;
        clusters_mod.writeln("#[allow(unsafe_op_in_unsafe_fn)]")?;
        clusters_mod.writeln("")?;

        for item in &c.items {
            if let svd::RegisterBlockItem::Cluster { cluster } = item {
                let sub_mod_name = sanitize_module_name(&cluster.name);
                clusters_mod.writeln(&format!("pub mod {sub_mod_name};"))?;
            }
        }

        files.push(GeneratedFile {
            file_name: format!("{}/clusters/mod.rs", cluster_mod_name),
            content: clusters_mod.into_string(),
        });
    }

    Ok(files)
}

/// Generate a single Rust file that represents the whole device.
///
/// The resulting file name is derived from `device.name` and ends with `.rs`.
pub fn generate_device_file(device: &svd::Device) -> Result<GeneratedFile> {
    generate_device_file_with_options(device, PacOptions::full())
}

pub fn generate_device_file_with_options(
    device: &svd::Device,
    options: PacOptions,
) -> Result<GeneratedFile> {
    let file_stem = sanitize_file_stem(&device.name);
    let file_name = format!("{file_stem}_pac.rs");
    let content = generate_device_rs_with_options(device, options)?;
    Ok(GeneratedFile { file_name, content })
}

/// Generate Rust code (as a string) for the whole device.
pub fn generate_device_rs(device: &svd::Device) -> Result<String> {
    generate_device_rs_with_options(device, PacOptions::full())
}

pub fn generate_device_rs_with_options(
    device: &svd::Device,
    options: PacOptions,
) -> Result<String> {
    let mut out = CodeWriter::new();
    let mut type_defs = CodeWriter::new();
    let mut st = GenState::new();

    out.writeln("#[allow(non_snake_case)]")?;
    out.writeln("#[allow(non_camel_case_types)]")?;
    out.writeln("#[allow(dead_code)]")?;
    out.writeln("#[allow(unused_imports)]")?;
    out.writeln("#[allow(unsafe_op_in_unsafe_fn)]")?;
    out.writeln("")?;

    out.writeln("")?;
    out.writeln("pub mod traits {")?;
    out.writeln(&generate_traits_file())?;
    out.writeln("}")?;
    out.writeln("")?;
    out.writeln("pub mod types {")?;
    out.writeln(&generate_types_file())?;
    out.writeln("}")?;
    out.writeln("")?;
    out.writeln("pub mod enums {")?;
    out.writeln(&generate_enums_file(device))?;
    out.writeln("}")?;
    out.writeln("")?;
    out.writeln("pub mod constants {")?;
    out.writeln(&generate_constants_file(device))?;
    out.writeln("}")?;
    out.writeln("")?;

    // Enumerations for fields with enumeratedValue blocks.
    // We now emit them within each peripheral module for better organization.
    // Keeping enum_defs for backwards compatibility but not using field_enums module.

    // Keep deterministic order.
    let mut periphs = device.peripherals.clone();
    periphs.sort_by(|a, b| {
        a.base_address
            .cmp(&b.base_address)
            .then(a.name.cmp(&b.name))
    });

    // Address constants.
    for p in &periphs {
        let cname = sanitize_const_name(&p.name);
        out.writeln(&format!(
            "pub const {cname}_BASE: usize = 0x{addr:08X};",
            addr = p.base_address
        ))?;
    }
    out.writeln("")?;

    // Types + pointers.
    for p in &periphs {
        generate_peripheral(&mut st, &mut out, &mut type_defs, device, p, options)?;
        out.writeln("")?;
    }

    // Helper types (clusters, etc.) — append at the end (Rust item order doesn't matter).
    if !type_defs.s.trim().is_empty() {
        out.s.push('\n');
        out.s.push_str(&type_defs.s);
    }

    Ok(out.s)
}

/// Write the generated device file into a directory.
pub fn write_device_file(device: &svd::Device, out_dir: &Path) -> Result<std::path::PathBuf> {
    let generated = generate_device_file(device)?;
    std::fs::create_dir_all(out_dir)?;
    let out_path = out_dir.join(&generated.file_name);
    std::fs::write(&out_path, generated.content)?;
    Ok(out_path)
}

pub fn write_device_dir(device: &svd::Device, out_dir: &Path) -> Result<std::path::PathBuf> {
    let generated = generate_device_dir(device)?;
    let dir_path = out_dir.join(&generated.path);
    std::fs::create_dir_all(&dir_path)?;
    for file in generated.files {
        let file_path = dir_path.join(&file.file_name);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&file_path, file.content)?;
    }
    Ok(dir_path)
}

pub fn write_device_dir_with_options(
    device: &svd::Device,
    out_dir: &Path,
    options: PacOptions,
) -> Result<std::path::PathBuf> {
    let generated = generate_device_dir_with_options(device, options)?;
    let dir_path = out_dir.join(&generated.path);
    std::fs::create_dir_all(&dir_path)?;
    for file in generated.files {
        let file_path = dir_path.join(&file.file_name);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&file_path, file.content)?;
    }
    Ok(dir_path)
}

/// Convenience helper: parse an SVD file and write a generated device `.rs` file.
pub fn generate_from_svd_file(svd_path: &Path, out_dir: &Path) -> Result<std::path::PathBuf> {
    let dev = crate::parse_svd_file(svd_path)?;
    write_device_file(&dev, out_dir)
}

/// Генерирует файл `{device}_cortex_m.rs` с NVIC и enum Interrupt.
///
/// Содержит:
/// - `Interrupt` enum (если есть прерывания)
/// - `nvic` модуль с функциями для работы с прерываниями
pub fn generate_cortex_m_file(device: &svd::Device) -> Result<GeneratedFile> {
    let file_stem = sanitize_file_stem(&device.name);
    let file_name = format!("{file_stem}_cortex_m.rs");
    let content = generate_cortex_m_rs(device)?;
    Ok(GeneratedFile { file_name, content })
}

fn generate_cortex_m_rs(device: &svd::Device) -> Result<String> {
    let (num_irqs, _irqs) = collect_device_interrupts(device);
    let prio_bits = device
        .cpu
        .as_ref()
        .map(|c| c.nvic_prio_bits)
        .unwrap_or(8)
        .min(8);

    let mut out = CodeWriter::new();
    out.writeln("#[allow(non_snake_case)]")?;
    out.writeln("#[allow(non_camel_case_types)]")?;
    out.writeln("#[allow(dead_code)]")?;
    out.writeln("")?;

    out.writeln("pub mod nvic {")?;
    out.indent();

    out.writeln(&format!("pub const PRIO_BITS: u8 = {prio_bits}u8;"))?;
    out.writeln(&format!("pub const NUM_IRQS: u32 = {num_irqs}u32;"))?;
    out.writeln("")?;

    out.writeln("const NVIC_ISER_BASE: usize = 0xE000_E100;")?;
    out.writeln("const NVIC_ICER_BASE: usize = 0xE000_E180;")?;
    out.writeln("const NVIC_ISPR_BASE: usize = 0xE000_E200;")?;
    out.writeln("const NVIC_ICPR_BASE: usize = 0xE000_E280;")?;
    out.writeln("const NVIC_IABR_BASE: usize = 0xE000_E300;")?;
    out.writeln("const NVIC_IPR_BASE: usize = 0xE000_E400;")?;
    out.writeln("const NVIC_STIR: usize = 0xE000_EF00;")?;
    out.writeln("")?;

    out.writeln("#[inline(always)]")?;
    out.writeln("pub unsafe fn enable_irq(irq: u32) {")?;
    out.indent();
    out.writeln("let idx = (irq >> 5) as usize;")?;
    out.writeln("let bit = 1u32 << (irq & 31);")?;
    out.writeln("let p = (NVIC_ISER_BASE + idx * 4) as *mut u32;")?;
    out.writeln("core::ptr::write_volatile(p, bit);")?;
    out.dedent();
    out.writeln("}")?;

    out.writeln("#[inline(always)]")?;
    out.writeln("pub unsafe fn disable_irq(irq: u32) {")?;
    out.indent();
    out.writeln("let idx = (irq >> 5) as usize;")?;
    out.writeln("let bit = 1u32 << (irq & 31);")?;
    out.writeln("let p = (NVIC_ICER_BASE + idx * 4) as *mut u32;")?;
    out.writeln("core::ptr::write_volatile(p, bit);")?;
    out.dedent();
    out.writeln("}")?;

    out.writeln("#[inline(always)]")?;
    out.writeln("pub unsafe fn pending_irq(irq: u32) {")?;
    out.indent();
    out.writeln("let idx = (irq >> 5) as usize;")?;
    out.writeln("let bit = 1u32 << (irq & 31);")?;
    out.writeln("let p = (NVIC_ISPR_BASE + idx * 4) as *mut u32;")?;
    out.writeln("core::ptr::write_volatile(p, bit);")?;
    out.dedent();
    out.writeln("}")?;

    out.writeln("#[inline(always)]")?;
    out.writeln("pub unsafe fn unpending_irq(irq: u32) {")?;
    out.indent();
    out.writeln("let idx = (irq >> 5) as usize;")?;
    out.writeln("let bit = 1u32 << (irq & 31);")?;
    out.writeln("let p = (NVIC_ICPR_BASE + idx * 4) as *mut u32;")?;
    out.writeln("core::ptr::write_volatile(p, bit);")?;
    out.dedent();
    out.writeln("}")?;

    out.writeln("#[inline(always)]")?;
    out.writeln("pub unsafe fn is_active_irq(irq: u32) -> bool {")?;
    out.indent();
    out.writeln("let idx = (irq >> 5) as usize;")?;
    out.writeln("let bit = 1u32 << (irq & 31);")?;
    out.writeln("let p = (NVIC_IABR_BASE + idx * 4) as *const u32;")?;
    out.writeln("(core::ptr::read_volatile(p) & bit) != 0")?;
    out.dedent();
    out.writeln("}")?;

    out.writeln("#[inline(always)]")?;
    out.writeln("pub unsafe fn set_priority(irq: u32, prio: u8) {")?;
    out.indent();
    out.writeln("let prio_bits = PRIO_BITS.min(8);")?;
    out.writeln("let shift = 8u8.saturating_sub(prio_bits);")?;
    out.writeln(
        "let mask = if prio_bits == 8 { 0xFFu8 } else { ((1u16 << prio_bits) - 1) as u8 };",
    )?;
    out.writeln("let v = (prio & mask) << shift;")?;
    out.writeln("let p = (NVIC_IPR_BASE + irq as usize) as *mut u8;")?;
    out.writeln("core::ptr::write_volatile(p, v);")?;
    out.dedent();
    out.writeln("}")?;

    out.writeln("#[inline(always)]")?;
    out.writeln("pub unsafe fn software_trigger(irq: u32) {")?;
    out.indent();
    out.writeln("let p = NVIC_STIR as *mut u32;")?;
    out.writeln("core::ptr::write_volatile(p, irq & 0x1FF);")?;
    out.dedent();
    out.writeln("}")?;

    out.dedent();
    out.writeln("}")?;
    Ok(out.s)
}

/// Генерирует PAC-файл устройства **и** минимальный runtime (startup + linker script).
///
/// Полезно, если вы хотите обойтись без `cortex-m-rt`, но всё равно получить:
/// - таблицу векторов
/// - обработчик `Reset`
/// - инициализацию `.data/.bss`
/// - переход в пользовательский `main()`
///
/// Выходные файлы:
/// - `<device>_pac.rs` (как `generate_device_file`)
/// - `<device>_cortex_m.rs` (nvic модуль)
/// - `<device>_rt.rs` (startup + vector table)
/// - `<device>_link.x` (минимальный linker script; `INCLUDE memory.x`)
pub fn generate_device_files_with_rt(device: &svd::Device) -> Result<Vec<GeneratedFile>> {
    let file_stem = sanitize_file_stem(&device.name);
    let mut out = Vec::new();
    out.push(generate_device_file(device)?);

    out.push(GeneratedFile {
        file_name: format!("{file_stem}_cortex_m.rs"),
        content: generate_cortex_m_rs(device)?,
    });

    out.push(GeneratedFile {
        file_name: format!("{file_stem}_rt.rs"),
        content: generate_rt_rs(device)?,
    });
    out.push(GeneratedFile {
        file_name: format!("{file_stem}_link.x"),
        content: generate_link_x(device)?,
    });
    Ok(out)
}

/// Записывает `<device>_pac.rs` + `<device>_rt.rs` + `<device>_link.x` в `out_dir`.
pub fn write_device_files_with_rt(
    device: &svd::Device,
    out_dir: &Path,
) -> Result<Vec<std::path::PathBuf>> {
    let files = generate_device_files_with_rt(device)?;
    std::fs::create_dir_all(out_dir)?;
    let mut out_paths = Vec::new();
    for f in files {
        let p = out_dir.join(&f.file_name);
        std::fs::write(&p, f.content)?;
        out_paths.push(p);
    }
    Ok(out_paths)
}

struct GenState {
    type_name_counters: BTreeMap<String, usize>,
    emitted_types: std::collections::HashSet<String>,
    type_sizes: BTreeMap<String, u64>,
    /// Dedup: (base type name) -> (layout fingerprint) -> concrete Rust type name.
    cluster_type_by_layout: BTreeMap<String, BTreeMap<String, String>>,
    /// Dedup for enums: (base enum name) -> (enum fingerprint) -> concrete Rust enum type name.
    enum_type_by_layout: BTreeMap<String, BTreeMap<String, String>>,
    emitted_enums: std::collections::HashSet<String>,
    /// Mapping from (peripheral, reg_path, field, enumeratedValues index) to enum Rust type name.
    field_enum_ty_map: BTreeMap<String, String>,
    /// Dedup for register wrapper types: (base) -> (fingerprint) -> type name.
    reg_type_by_layout: BTreeMap<String, BTreeMap<String, String>>,
    emitted_regs: std::collections::HashSet<String>,
}

impl GenState {
    fn new() -> Self {
        Self {
            type_name_counters: BTreeMap::new(),
            emitted_types: std::collections::HashSet::new(),
            type_sizes: BTreeMap::new(),
            cluster_type_by_layout: BTreeMap::new(),
            enum_type_by_layout: BTreeMap::new(),
            emitted_enums: std::collections::HashSet::new(),
            field_enum_ty_map: BTreeMap::new(),
            reg_type_by_layout: BTreeMap::new(),
            emitted_regs: std::collections::HashSet::new(),
        }
    }

    fn alloc_type_name(&mut self, base: String) -> String {
        let n = self.type_name_counters.entry(base.clone()).or_insert(0);
        if *n == 0 {
            *n = 1;
            fix_register_case(&base)
        } else {
            let out = format!("{base}_{}", *n);
            *n += 1;
            out
        }
    }

    /// Returns true if the type has NOT been emitted before and is now marked as emitted.
    fn mark_type_emitted(&mut self, ty: &str) -> bool {
        self.emitted_types.insert(ty.to_string())
    }

    fn set_type_size_bytes(&mut self, ty: &str, sz: u64) {
        self.type_sizes.insert(ty.to_string(), sz);
    }

    fn get_type_size_bytes(&self, ty: &str) -> Option<u64> {
        self.type_sizes.get(ty).copied()
    }

    fn lookup_cluster_type(&self, base_ty: &str, fingerprint: &str) -> Option<String> {
        self.cluster_type_by_layout
            .get(base_ty)
            .and_then(|m| m.get(fingerprint))
            .cloned()
    }

    fn remember_cluster_type(&mut self, base_ty: &str, fingerprint: &str, ty: &str) {
        self.cluster_type_by_layout
            .entry(base_ty.to_string())
            .or_default()
            .insert(fingerprint.to_string(), ty.to_string());
    }

    fn lookup_enum_type(&self, base: &str, fingerprint: &str) -> Option<String> {
        self.enum_type_by_layout
            .get(base)
            .and_then(|m| m.get(fingerprint))
            .cloned()
    }

    fn remember_enum_type(&mut self, base: &str, fingerprint: &str, ty: &str) {
        self.enum_type_by_layout
            .entry(base.to_string())
            .or_default()
            .insert(fingerprint.to_string(), ty.to_string());
    }

    fn mark_enum_emitted(&mut self, ty: &str) -> bool {
        self.emitted_enums.insert(ty.to_string())
    }

    fn field_enum_key(periph: &str, reg_path: &str, field: &str, idx: usize) -> String {
        format!("{periph}|{reg_path}|{field}|{idx}")
    }

    fn remember_field_enum_ty(
        &mut self,
        periph: &str,
        reg_path: &str,
        field: &str,
        idx: usize,
        ty: &str,
    ) {
        let k = Self::field_enum_key(periph, reg_path, field, idx);
        self.field_enum_ty_map.insert(k, ty.to_string());
    }

    fn lookup_field_enum_ty(
        &self,
        periph: &str,
        reg_path: &str,
        field: &str,
        idx: usize,
    ) -> Option<String> {
        let k = Self::field_enum_key(periph, reg_path, field, idx);
        self.field_enum_ty_map.get(&k).cloned()
    }

    fn lookup_reg_type(&self, base: &str, fingerprint: &str) -> Option<String> {
        self.reg_type_by_layout
            .get(base)
            .and_then(|m| m.get(fingerprint))
            .cloned()
    }

    fn remember_reg_type(&mut self, base: &str, fingerprint: &str, ty: &str) {
        self.reg_type_by_layout
            .entry(base.to_string())
            .or_default()
            .insert(fingerprint.to_string(), ty.to_string());
    }

    fn mark_reg_emitted(&mut self, ty: &str) -> bool {
        self.emitted_regs.insert(ty.to_string())
    }
}

// --------------------------- implementation ---------------------------

fn find_peripheral<'a>(device: &'a svd::Device, name: &str) -> Option<&'a svd::Peripheral> {
    device.peripherals.iter().find(|p| p.name == name)
}

fn peripheral_register_items<'a>(
    device: &'a svd::Device,
    p: &'a svd::Peripheral,
) -> &'a [svd::RegisterBlockItem] {
    let mut cur = p;
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    loop {
        if let Some(rb) = cur.registers.as_ref() {
            return rb.items.as_slice();
        }
        let Some(df) = cur.derived_from.as_deref() else {
            return &[];
        };
        if !seen.insert(cur.name.clone()) {
            return &[];
        }
        let Some(next) = find_peripheral(device, df) else {
            return &[];
        };
        cur = next;
    }
}

fn generate_peripheral(
    st: &mut GenState,
    out: &mut CodeWriter,
    type_defs: &mut CodeWriter,
    device: &svd::Device,
    p: &svd::Peripheral,
    options: PacOptions,
) -> Result<()> {
    let mod_name = sanitize_module_name(&p.name);

    let mut mod_out = CodeWriter::new();
    mod_out.indent();
    mod_out.writeln("use super::*;")?;
    mod_out.writeln(&format!(
        "pub const BASE: usize = 0x{:08X};",
        p.base_address
    ))?;
    mod_out.writeln("")?;

    mod_out.writeln("#[repr(C)]")?;
    mod_out.writeln("pub struct RegisterBlock {")?;
    mod_out.indent();

    let items = peripheral_register_items(device, p);

    let ctx = Ctx {
        device,
        periph: Some(p),
        cluster_stack: Vec::new(),
    };

    emit_register_block_items(
        st,
        &mut mod_out,
        &mut CodeWriter::new(),
        type_defs,
        &ctx,
        items,
        0,
        options,
    )?;

    mod_out.dedent();
    mod_out.writeln("}")?;

    // Reset implementation (best-effort, based on SVD resetValue/resetMask).
    emit_reset_impl_for_struct(
        &mut mod_out,
        st,
        type_defs,
        &ctx,
        items,
        "RegisterBlock",
        options,
    )?;

    // Raw pointers.
    mod_out.writeln(&format!(
        "pub const PTR: *const RegisterBlock = BASE as *const RegisterBlock;"
    ))?;
    mod_out.writeln(&format!(
        "pub const PTR_MUT: *mut RegisterBlock = BASE as *mut RegisterBlock;"
    ))?;

    // Field enumerations for this peripheral.
    if options.emit_field_enums {
        emit_peripheral_enums(st, device, p, &mut mod_out)?;
    }

    // Compile-time once-tokens (best-effort): collect WriteOnce/ReadWriteOnce registers by offset.
    let mut once_regs = Vec::new();
    let mut name_counts = BTreeMap::new();
    collect_once_regs(&ctx, items, 0, "", None, &mut name_counts, &mut once_regs);
    if !once_regs.is_empty() {
        let once_ty = "Once";
        mod_out.writeln("")?;
        mod_out.writeln("/// Compile-time writeOnce/read-writeOnce tokens (state-machine API).")?;
        mod_out.writeln("///")?;
        mod_out.writeln("/// NOTE: this enforces \"once\" only for code using this token API.")?;
        mod_out.writeln(&format!("pub struct {once_ty} {{"))?;
        mod_out.indent();
        mod_out.writeln("base: usize,")?;
        for r in &once_regs {
            mod_out.writeln(&format!("pub {}: {},", r.field_name, r.token_ty))?;
        }
        mod_out.dedent();
        mod_out.writeln("}")?;
        mod_out.writeln(&format!("impl {once_ty} {{"))?;
        mod_out.indent();
        mod_out.writeln("/// Create tokens for this peripheral.")?;
        mod_out.writeln("///")?;
        mod_out.writeln("/// # Safety")?;
        mod_out.writeln("/// The returned tokens allow volatile access to MMIO.")?;
        mod_out.writeln("pub unsafe fn new() -> Self {")?;
        mod_out.indent();
        mod_out.writeln("let base = BASE;")?;
        mod_out.writeln("Self {")?;
        mod_out.indent();
        mod_out.writeln("base,")?;
        for r in &once_regs {
            mod_out.writeln(&format!("{}: {},", r.field_name, r.init_expr))?;
        }
        mod_out.dedent();
        mod_out.writeln("}")?;
        mod_out.dedent();
        mod_out.writeln("}")?;
        mod_out.dedent();
        mod_out.writeln("}")?;
    }

    // Wrap the generated peripheral items into their own module.
    out.writeln(&format!("/// Peripheral `{}`", p.name))?;
    if let Some(desc) = &p.description {
        out.writeln(&format!("/// {}", doc_escape(desc)))?;
    }
    out.writeln(&format!("pub mod {mod_name} {{"))?;
    out.s.push_str(&mod_out.s);
    out.writeln("}")?;

    Ok(())
}

/// Context for resolving register properties while descending into clusters.
struct Ctx<'a> {
    device: &'a svd::Device,
    periph: Option<&'a svd::Peripheral>,
    cluster_stack: Vec<&'a svd::Cluster>,
}

fn emit_register_block_items(
    st: &mut GenState,
    out: &mut CodeWriter,
    regs_out: &mut CodeWriter,
    type_defs: &mut CodeWriter,
    ctx: &Ctx<'_>,
    items: &[svd::RegisterBlockItem],
    base_offset: u64,
    options: PacOptions,
) -> Result<u64> {
    // Deterministic and layout-correct order by offset.
    let mut sorted: Vec<&svd::RegisterBlockItem> = items.iter().collect();
    sorted.sort_by_key(|it| match it {
        svd::RegisterBlockItem::Register { register } => register.address_offset,
        svd::RegisterBlockItem::Cluster { cluster } => cluster.address_offset,
    });

    let mut cur = base_offset;
    let mut reserved_idx = 0usize;

    for it in sorted {
        let (name, off, size_bytes, kind_desc) = match it {
            svd::RegisterBlockItem::Register { register } => {
                let (_, elem_sz) = reg_primitive_ty_and_size(ctx, &register.properties);
                let item_sz = reg_item_total_size_bytes(register, elem_sz);
                (
                    sanitize_field_name(&register.name),
                    base_offset + register.address_offset,
                    item_sz,
                    format!("register {}", register.name),
                )
            }
            svd::RegisterBlockItem::Cluster { cluster } => {
                let (_cluster_ty, cluster_sz) =
                    cluster_rust_type_and_size(st, type_defs, ctx, cluster, options)?;
                let item_sz = cluster_item_total_size_bytes(cluster, cluster_sz);
                (
                    sanitize_field_name(&cluster.name),
                    base_offset + cluster.address_offset,
                    item_sz,
                    format!("cluster {}", cluster.name),
                )
            }
        };

        if off < cur {
            // Overlapping / out-of-order: keep layout safe by not emitting negative paddings.
            out.writeln(&format!(
                "// WARNING: {kind_desc} has offset 0x{off:X} which is before current 0x{cur:X}; layout may be incorrect"
            ))?;
        } else if off > cur {
            let gap = off - cur;
            out.writeln(&format!(
                "pub _reserved_{reserved_idx}: [u8; {gap} as usize],"
            ))?;
            reserved_idx += 1;
            cur = off;
        }

        // Emit actual field.
        match it {
            svd::RegisterBlockItem::Register { register } => {
                emit_register_field(st, out, regs_out, type_defs, ctx, &name, register, options)?;
            }
            svd::RegisterBlockItem::Cluster { cluster } => {
                emit_cluster_field(st, out, regs_out, type_defs, ctx, &name, cluster, options)?;
            }
        }

        cur = cur.saturating_add(size_bytes);
    }

    Ok(cur)
}

fn emit_register_field(
    st: &mut GenState,
    out: &mut CodeWriter,
    regs_out: &mut CodeWriter,
    type_defs: &mut CodeWriter,
    ctx: &Ctx<'_>,
    field_name: &str,
    r: &svd::Register,
    options: PacOptions,
) -> Result<()> {
    let (base_ty, size_bytes) = reg_primitive_ty_and_size(ctx, &r.properties);
    let access = resolve_access(ctx, r);
    let reg_ty = register_wrapper_type(st, type_defs, ctx, r, &base_ty, access, options)?;
    if let Some(dim) = &r.dim {
        if dim.dim_increment == size_bytes {
            out.writeln(&format!(
                "pub {field_name}: [{reg_ty}; {n} as usize],",
                n = dim.dim
            ))?;
        } else {
            out.writeln(&format!(
                "/// NOTE: {} is an array with dimIncrement={}, elementSize={} — emitted as bytes",
                r.name, dim.dim_increment, size_bytes
            ))?;
            let total = dim.dim_increment.saturating_mul(dim.dim);
            out.writeln(&format!("pub {field_name}: [u8; {total} as usize],"))?;
        }
    } else {
        out.writeln(&format!("pub {field_name}: {reg_ty},"))?;
    }
    Ok(())
}

fn emit_cluster_field(
    st: &mut GenState,
    out: &mut CodeWriter,
    regs_out: &mut CodeWriter,
    type_defs: &mut CodeWriter,
    ctx: &Ctx<'_>,
    field_name: &str,
    c: &svd::Cluster,
    options: PacOptions,
) -> Result<()> {
    let (cluster_ty, cluster_size_bytes) =
        cluster_rust_type_and_size(st, type_defs, ctx, c, options)?;
    if let Some(dim) = &c.dim {
        if dim.dim_increment == cluster_size_bytes {
            out.writeln(&format!(
                "pub {field_name}: [{cluster_ty}; {n} as usize],",
                n = dim.dim
            ))?;
        } else {
            out.writeln(&format!(
                "/// NOTE: {} is an array cluster with dimIncrement={}, clusterSize={} — emitted as bytes",
                c.name, dim.dim_increment, cluster_size_bytes
            ))?;
            let total = dim.dim_increment.saturating_mul(dim.dim);
            out.writeln(&format!("pub {field_name}: [u8; {total} as usize],"))?;
        }
    } else {
        out.writeln(&format!("pub {field_name}: {cluster_ty},"))?;
    }
    Ok(())
}

fn reg_item_total_size_bytes(r: &svd::Register, elem_size_bytes: u64) -> u64 {
    if let Some(dim) = &r.dim {
        dim.dim_increment.saturating_mul(dim.dim)
    } else {
        elem_size_bytes
    }
}

fn cluster_item_total_size_bytes(c: &svd::Cluster, cluster_size_bytes: u64) -> u64 {
    if let Some(dim) = &c.dim {
        dim.dim_increment.saturating_mul(dim.dim)
    } else {
        cluster_size_bytes
    }
}

fn cluster_rust_type_and_size(
    st: &mut GenState,
    type_defs: &mut CodeWriter,
    ctx: &Ctx<'_>,
    c: &svd::Cluster,
    options: PacOptions,
) -> Result<(String, u64)> {
    // Dedup strategy:
    // - Prefer reusing the same Rust type for clusters with the same (name, layout).
    // - Only disambiguate (suffix _1, _2...) when the same name appears with a different layout.
    let base_ty = sanitize_type_name(&c.name);

    // Compute cluster byte size by looking at its members.
    let child_ctx = Ctx {
        device: ctx.device,
        periph: ctx.periph,
        cluster_stack: {
            let mut s = ctx.cluster_stack.clone();
            s.push(c);
            s
        },
    };

    let fingerprint = cluster_layout_fingerprint(&child_ctx, c);
    let periph_name = ctx.periph.map(|p| p.name.as_str()).unwrap_or("");
    let full_base_ty = format!("{}_{}", periph_name, base_ty);
    let ty = if let Some(existing) = st.lookup_cluster_type(&full_base_ty, &fingerprint) {
        existing
    } else {
        let new_ty = st.alloc_type_name(full_base_ty.clone());
        st.remember_cluster_type(&full_base_ty, &fingerprint, &new_ty);
        new_ty
    };

    // We need the layout, so we generate the type definition only once.
    if st.mark_type_emitted(&ty) {
        // Not emitted yet — emit now.
        //
        // Important: we can't pass `type_defs` as both the "field output" and the
        // "type definitions output" at the same time. So we generate the struct body
        // into a temporary writer, while nested cluster type definitions still go
        // into `type_defs`.
        let mut body = CodeWriter::new();
        body.indent();
        let end = emit_register_block_items(
            st,
            &mut body,
            &mut CodeWriter::new(),
            type_defs,
            &child_ctx,
            &c.items,
            0,
            options,
        )?;

        type_defs.writeln(&format!("/// Cluster `{}`", c.name))?;
        type_defs.writeln("#[repr(C)]")?;
        type_defs.writeln(&format!("pub struct {ty} {{"))?;
        type_defs.s.push_str(&body.s);
        type_defs.writeln("}")?;
        type_defs.writeln("")?;

        let mut impl_out = CodeWriter::new();
        emit_reset_impl_for_struct(
            &mut impl_out,
            st,
            type_defs,
            &child_ctx,
            &c.items,
            &ty,
            options,
        )?;
        type_defs.s.push_str(&impl_out.s);
        type_defs.s.push('\n');

        // Save computed size for later users.
        st.set_type_size_bytes(&ty, end);
    }

    let size = st.get_type_size_bytes(&ty).unwrap_or(0);
    Ok((ty, size))
}

fn cluster_layout_fingerprint(ctx: &Ctx<'_>, c: &svd::Cluster) -> String {
    // Goal: stable, reasonably unique signature of the generated struct layout.
    // We include:
    // - cluster dim (if any)
    // - ordered items by address_offset
    // - for registers: offset, effective primitive type, effective access, dim params
    // - for nested clusters: offset, their own (name, fingerprint), dim params
    let mut parts: Vec<String> = Vec::new();

    if let Some(dim) = &c.dim {
        parts.push(format!(
            "DIM:{}:{}:{}:{}",
            dim.dim,
            dim.dim_increment,
            dim.dim_index.as_deref().unwrap_or(""),
            dim.dim_name.as_deref().unwrap_or("")
        ));
    }

    let mut sorted: Vec<&svd::RegisterBlockItem> = c.items.iter().collect();
    sorted.sort_by_key(|it| match it {
        svd::RegisterBlockItem::Register { register } => register.address_offset,
        svd::RegisterBlockItem::Cluster { cluster } => cluster.address_offset,
    });

    for it in sorted {
        match it {
            svd::RegisterBlockItem::Register { register } => {
                let access = resolve_access_type(ctx, &register.properties);
                let (_base_ty, bytes) = reg_primitive_ty_and_size(ctx, &register.properties);
                let dim_sig = register
                    .dim
                    .as_ref()
                    .map(|d| format!("dim={} inc={}", d.dim, d.dim_increment))
                    .unwrap_or_else(|| "scalar".to_string());
                parts.push(format!(
                    "R:{}:{}:{}:{:?}:{dim_sig}",
                    register.name, register.address_offset, bytes, access
                ));
            }
            svd::RegisterBlockItem::Cluster { cluster } => {
                // Recurse with extended context.
                let child_ctx = Ctx {
                    device: ctx.device,
                    periph: ctx.periph,
                    cluster_stack: {
                        let mut s = ctx.cluster_stack.clone();
                        s.push(cluster);
                        s
                    },
                };
                let dim_sig = cluster
                    .dim
                    .as_ref()
                    .map(|d| format!("dim={} inc={}", d.dim, d.dim_increment))
                    .unwrap_or_else(|| "scalar".to_string());
                let nested_fp = cluster_layout_fingerprint(&child_ctx, cluster);
                parts.push(format!(
                    "C:{}:{}:{dim_sig}:({})",
                    cluster.name, cluster.address_offset, nested_fp
                ));
            }
        }
    }

    parts.join("|")
}

fn resolve_size_bits(ctx: &Ctx<'_>, leaf_props: &svd::RegisterProperties) -> u64 {
    // Priority (spec-ish): leaf -> nearest cluster -> peripheral -> device defaults -> device.width.
    if let Some(v) = leaf_props.size {
        return v;
    }
    for c in ctx.cluster_stack.iter().rev() {
        if let Some(v) = c.register_properties.size {
            return v;
        }
    }
    if let Some(mut p) = ctx.periph {
        let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        loop {
            if let Some(v) = p.register_properties.size {
                return v;
            }
            let Some(df) = p.derived_from.as_deref() else {
                break;
            };
            if !seen.insert(p.name.clone()) {
                break;
            }
            let Some(next) = find_peripheral(ctx.device, df) else {
                break;
            };
            p = next;
        }
    }
    if let Some(v) = ctx.device.default_register_properties.size {
        return v;
    }
    ctx.device.width as u64
}

fn reg_primitive_ty_and_size(ctx: &Ctx<'_>, leaf_props: &svd::RegisterProperties) -> (String, u64) {
    let bits = resolve_size_bits(ctx, leaf_props);
    let bytes = ((bits + 7) / 8).max(1);
    let base_ty = match bytes {
        1 => "u8",
        2 => "u16",
        4 => "u32",
        8 => "u64",
        _ => "u32",
    };
    (base_ty.to_string(), bytes)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegWriteModel {
    Normal,
    W1S,
    W1C,
    W0S,
    W0C,
    WT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ResolvedAccess {
    access: svd::AccessType,
    write_model: RegWriteModel,
}

fn resolve_access_type(ctx: &Ctx<'_>, leaf_props: &svd::RegisterProperties) -> svd::AccessType {
    // Priority (spec-ish): leaf -> nearest cluster -> peripheral -> device defaults -> ReadWrite.
    if let Some(v) = leaf_props.access {
        return v;
    }
    for c in ctx.cluster_stack.iter().rev() {
        if let Some(v) = c.register_properties.access {
            return v;
        }
    }
    if let Some(mut p) = ctx.periph {
        let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        loop {
            if let Some(v) = p.register_properties.access {
                return v;
            }
            let Some(df) = p.derived_from.as_deref() else {
                break;
            };
            if !seen.insert(p.name.clone()) {
                break;
            }
            let Some(next) = find_peripheral(ctx.device, df) else {
                break;
            };
            p = next;
        }
    }
    if let Some(v) = ctx.device.default_register_properties.access {
        return v;
    }
    svd::AccessType::ReadWrite
}

fn resolve_access(ctx: &Ctx<'_>, r: &svd::Register) -> ResolvedAccess {
    let access = resolve_access_type(ctx, &r.properties);
    if matches!(access, svd::AccessType::ReadOnly) {
        return ResolvedAccess {
            access,
            write_model: RegWriteModel::Normal,
        };
    }

    if resolve_write_as_read(ctx, r) {
        return ResolvedAccess {
            access,
            write_model: RegWriteModel::Normal,
        };
    }

    let mwv = resolve_modified_write_values(ctx, r);
    let write_model = match mwv {
        Some(svd::ModifiedWriteValues::OneToSet) => RegWriteModel::W1S,
        Some(svd::ModifiedWriteValues::OneToClear) => RegWriteModel::W1C,
        Some(svd::ModifiedWriteValues::ZeroToSet) => RegWriteModel::W0S,
        Some(svd::ModifiedWriteValues::ZeroToClear) => RegWriteModel::W0C,
        Some(svd::ModifiedWriteValues::OneToToggle) => RegWriteModel::WT,
        _ => RegWriteModel::Normal,
    };

    ResolvedAccess {
        access,
        write_model,
    }
}

fn resolve_write_as_read(ctx: &Ctx<'_>, r: &svd::Register) -> bool {
    let mut cur = r;
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    loop {
        if matches!(
            cur.write_constraint,
            Some(svd::WriteConstraint::WriteAsRead {
                write_as_read: true
            })
        ) {
            return true;
        }
        if cur.field.iter().any(|f| {
            matches!(
                f.write_constraint,
                Some(svd::WriteConstraint::WriteAsRead {
                    write_as_read: true
                })
            )
        }) {
            return true;
        }

        let Some(df) = cur.derived_from.as_deref() else {
            return false;
        };
        let cur_key = ctx_reg_path(ctx, &cur.name);
        if !seen.insert(cur_key) {
            return false;
        }
        let Some(next) = find_register_in_ctx(ctx, df) else {
            return false;
        };
        cur = next;
    }
}

fn resolve_modified_write_values(
    ctx: &Ctx<'_>,
    r: &svd::Register,
) -> Option<svd::ModifiedWriteValues> {
    let mut cur = r;
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    loop {
        if let Some(mwv) = cur.modified_write_values {
            if mwv != svd::ModifiedWriteValues::Modify {
                return Some(mwv);
            }
        }

        let mut picked: Option<svd::ModifiedWriteValues> = None;
        let mut conflict = false;
        for f in &cur.field {
            let Some(mwv) = f.modified_write_values else {
                continue;
            };
            if mwv == svd::ModifiedWriteValues::Modify {
                continue;
            }
            match picked {
                None => picked = Some(mwv),
                Some(p) if p == mwv => {}
                Some(_) => {
                    conflict = true;
                    break;
                }
            }
        }
        if !conflict {
            if let Some(p) = picked {
                return Some(p);
            }
        }

        let Some(df) = cur.derived_from.as_deref() else {
            return None;
        };
        let cur_key = ctx_reg_path(ctx, &cur.name);
        if !seen.insert(cur_key) {
            return None;
        }
        let Some(next) = find_register_in_ctx(ctx, df) else {
            return None;
        };
        cur = next;
    }
}

fn find_register_in_ctx<'a>(ctx: &Ctx<'a>, derived_from: &str) -> Option<&'a svd::Register> {
    let Some(p) = ctx.periph else {
        return None;
    };
    let items = peripheral_register_items(ctx.device, p);

    let mut prefix = String::new();
    for (i, c) in ctx.cluster_stack.iter().enumerate() {
        if i > 0 {
            prefix.push('.');
        }
        prefix.push_str(&c.name);
    }

    let mut candidates: Vec<String> = Vec::new();
    if derived_from.contains('.') {
        candidates.push(derived_from.to_string());
    } else {
        if !prefix.is_empty() {
            candidates.push(format!("{prefix}.{derived_from}"));
        }
        candidates.push(derived_from.to_string());
    }

    for c in candidates {
        if let Some(r) = find_register_by_path(items, &c, "") {
            return Some(r);
        }
    }
    None
}

fn find_register_by_path<'a>(
    items: &'a [svd::RegisterBlockItem],
    target: &str,
    prefix: &str,
) -> Option<&'a svd::Register> {
    for it in items {
        match it {
            svd::RegisterBlockItem::Register { register } => {
                let p = if prefix.is_empty() {
                    register.name.clone()
                } else {
                    format!("{prefix}.{}", register.name)
                };
                if p == target {
                    return Some(register);
                }
            }
            svd::RegisterBlockItem::Cluster { cluster } => {
                let pfx = if prefix.is_empty() {
                    cluster.name.clone()
                } else {
                    format!("{prefix}.{}", cluster.name)
                };
                if let Some(r) = find_register_by_path(cluster.items.as_slice(), target, &pfx) {
                    return Some(r);
                }
            }
        }
    }
    None
}

fn resolve_reset(ctx: &Ctx<'_>, leaf_props: &svd::RegisterProperties) -> Option<(u64, u64)> {
    // Priority: leaf -> nearest cluster -> peripheral -> device defaults.
    // If reset_value is missing everywhere, we can't implement reset.
    let mut value: Option<u64> = leaf_props.reset_value;
    let mut mask: Option<u64> = leaf_props.reset_mask;

    if value.is_none() || mask.is_none() {
        for c in ctx.cluster_stack.iter().rev() {
            if value.is_none() {
                value = c.register_properties.reset_value;
            }
            if mask.is_none() {
                mask = c.register_properties.reset_mask;
            }
            if value.is_some() && mask.is_some() {
                break;
            }
        }
    }
    if (value.is_none() || mask.is_none()) && ctx.periph.is_some() {
        let mut p = ctx.periph.unwrap();
        let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        loop {
            if value.is_none() {
                value = p.register_properties.reset_value;
            }
            if mask.is_none() {
                mask = p.register_properties.reset_mask;
            }
            if value.is_some() && mask.is_some() {
                break;
            }
            let Some(df) = p.derived_from.as_deref() else {
                break;
            };
            if !seen.insert(p.name.clone()) {
                break;
            }
            let Some(next) = find_peripheral(ctx.device, df) else {
                break;
            };
            p = next;
        }
    }
    if value.is_none() {
        value = ctx.device.default_register_properties.reset_value;
    }
    if mask.is_none() {
        mask = ctx.device.default_register_properties.reset_mask;
    }

    let value = value?;
    let mask = mask.unwrap_or(u64::MAX);
    Some((value, mask))
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

fn emit_reset_impl_for_struct(
    out: &mut CodeWriter,
    st: &mut GenState,
    type_defs: &mut CodeWriter,
    ctx: &Ctx<'_>,
    items: &[svd::RegisterBlockItem],
    ty: &str,
    options: PacOptions,
) -> Result<()> {
    // Generate reset statements; if none, still emit empty reset() so callers can recurse.
    out.writeln(&format!("impl {ty} {{"))?;
    out.indent();
    out.writeln("#[inline(always)]")?;
    out.writeln("pub fn reset(&self) {")?;
    out.indent();

    emit_reset_stmts_for_items(out, st, type_defs, ctx, items, 0, options)?;

    out.dedent();
    out.writeln("}")?;
    out.dedent();
    out.writeln("}")?;
    Ok(())
}

fn emit_reset_stmts_for_items(
    out: &mut CodeWriter,
    st: &mut GenState,
    type_defs: &mut CodeWriter,
    ctx: &Ctx<'_>,
    items: &[svd::RegisterBlockItem],
    base_offset: u64,
    options: PacOptions,
) -> Result<()> {
    // Deterministic order by offset.
    let mut sorted: Vec<&svd::RegisterBlockItem> = items.iter().collect();
    sorted.sort_by_key(|it| match it {
        svd::RegisterBlockItem::Register { register } => register.address_offset,
        svd::RegisterBlockItem::Cluster { cluster } => cluster.address_offset,
    });

    for it in sorted {
        match it {
            svd::RegisterBlockItem::Register { register } => {
                // Only registers with known reset value can be reset.
                let Some((rv, rm)) = resolve_reset(ctx, &register.properties) else {
                    continue;
                };
                // Only if it is write-capable.
                let access = resolve_access(ctx, register);
                if matches!(access.access, svd::AccessType::ReadOnly) {
                    continue;
                }
                if access.write_model != RegWriteModel::Normal {
                    continue;
                }

                // Only if we emitted a typed field (not raw bytes) for dim arrays.
                let field_name = sanitize_field_name(&register.name);
                let (base_ty, elem_sz) = reg_primitive_ty_and_size(ctx, &register.properties);

                // Determine if this field is an array of typed regs or scalar.
                if let Some(dim) = &register.dim {
                    // We only emitted `[T; dim]` when increment == element size.
                    if dim.dim_increment != elem_sz {
                        continue;
                    }
                    let val = rv & rm;
                    let lit = reset_literal(val, &base_ty);
                    out.writeln(&format!(
                        "for r in self.{field_name}.iter() {{ r.write({lit}); }}"
                    ))?;
                } else {
                    let val = rv & rm;
                    let lit = reset_literal(val, &base_ty);
                    out.writeln(&format!("self.{field_name}.write({lit});"))?;
                }
            }
            svd::RegisterBlockItem::Cluster { cluster } => {
                let field_name = sanitize_field_name(&cluster.name);
                let (_cluster_ty, cluster_size_bytes) =
                    cluster_rust_type_and_size(st, type_defs, ctx, cluster, options)?;
                // Only if this cluster field was emitted as a typed struct/array.
                if let Some(dim) = &cluster.dim {
                    if dim.dim_increment != cluster_size_bytes {
                        continue;
                    }
                    out.writeln(&format!(
                        "for c in self.{field_name}.iter() {{ c.reset(); }}"
                    ))?;
                } else {
                    out.writeln(&format!("self.{field_name}.reset();"))?;
                }
            }
        }
        let _ = base_offset; // currently unused; kept for symmetry with layout traversal.
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct OnceRegInfo {
    field_name: String,
    /// Field type inside the token struct (e.g. `WOOnce<u32, Unwritten>`).
    token_ty: String,
    /// Field initializer expression inside `new()` (e.g. `WOOnce{..}` or `core::array::from_fn(..)`).
    init_expr: String,
}

fn collect_once_regs(
    ctx: &Ctx<'_>,
    items: &[svd::RegisterBlockItem],
    base_offset: u64,
    prefix: &str,
    array_ctx: Option<(u64, u64)>, // (len, stride)
    name_counts: &mut BTreeMap<String, usize>,
    out: &mut Vec<OnceRegInfo>,
) {
    // Sort by offset for stable output.
    let mut sorted: Vec<&svd::RegisterBlockItem> = items.iter().collect();
    sorted.sort_by_key(|it| match it {
        svd::RegisterBlockItem::Register { register } => register.address_offset,
        svd::RegisterBlockItem::Cluster { cluster } => cluster.address_offset,
    });

    for it in sorted {
        match it {
            svd::RegisterBlockItem::Register { register } => {
                let access = resolve_access(ctx, register).access;
                let (base_ty, _) = reg_primitive_ty_and_size(ctx, &register.properties);
                let (token_ctor, token_single_ty) = match access {
                    svd::AccessType::WriteOnce => (
                        "WOOnce",
                        format!("{token_ctor}<{base_ty}, Unwritten>", token_ctor = "WOOnce"),
                    ),
                    svd::AccessType::ReadWriteOnce => (
                        "RWOnce",
                        format!("{token_ctor}<{base_ty}, Unwritten>", token_ctor = "RWOnce"),
                    ),
                    _ => continue,
                };

                let mut fname = if prefix.is_empty() {
                    sanitize_field_name(&register.name)
                } else {
                    format!("{prefix}_{}", sanitize_field_name(&register.name))
                };
                let n = name_counts.entry(fname.clone()).or_insert(0);
                if *n > 0 {
                    fname = format!("{fname}_{}", *n);
                }
                *n += 1;

                let reg_base_off = base_offset + register.address_offset;

                // Determine whether this register becomes a scalar token or an array of tokens.
                // Rules:
                // - If we are inside an "array context" (dim cluster), we emit `[token; len]` for every leaf once-register.
                // - Else, if the register itself has dim, we emit `[token; dim]` with stride=dimIncrement.
                // - Nested dim inside dim-cluster is skipped (would become 2D).
                let (token_ty, init_expr) = match (array_ctx, &register.dim) {
                    (Some((len, stride)), None) => {
                        let token_ty = format!("[{token_single_ty}; {len}usize]");
                        let init_expr = format!(
                            "core::array::from_fn(|i| {token_ctor} {{ base, offset: 0x{reg_base_off:X}usize + (i as usize) * 0x{stride:X}usize, _state: PhantomData, _t: PhantomData }})"
                        );
                        (token_ty, init_expr)
                    }
                    (Some(_), Some(_)) => {
                        // 2D tokens: not generated (best-effort).
                        continue;
                    }
                    (None, Some(dim)) => {
                        let len = dim.dim;
                        let stride = dim.dim_increment;
                        let token_ty = format!("[{token_single_ty}; {len}usize]");
                        let init_expr = format!(
                            "core::array::from_fn(|i| {token_ctor} {{ base, offset: 0x{reg_base_off:X}usize + (i as usize) * 0x{stride:X}usize, _state: PhantomData, _t: PhantomData }})"
                        );
                        (token_ty, init_expr)
                    }
                    (None, None) => {
                        let token_ty = token_single_ty;
                        let init_expr = format!(
                            "{token_ctor} {{ base, offset: 0x{reg_base_off:X}usize, _state: PhantomData, _t: PhantomData }}"
                        );
                        (token_ty, init_expr)
                    }
                };

                out.push(OnceRegInfo {
                    field_name: fname,
                    token_ty,
                    init_expr,
                });
            }
            svd::RegisterBlockItem::Cluster { cluster } => {
                let child_ctx = Ctx {
                    device: ctx.device,
                    periph: ctx.periph,
                    cluster_stack: {
                        let mut s = ctx.cluster_stack.clone();
                        s.push(cluster);
                        s
                    },
                };
                let child_prefix = if prefix.is_empty() {
                    sanitize_field_name(&cluster.name)
                } else {
                    format!("{prefix}_{}", sanitize_field_name(&cluster.name))
                };

                if let Some(dim) = &cluster.dim {
                    // Cluster array: flatten into token arrays for each nested once-register.
                    // Note: nested dim registers/clusters inside this dim cluster are skipped (2D or more).
                    collect_once_regs(
                        &child_ctx,
                        &cluster.items,
                        base_offset + cluster.address_offset,
                        &child_prefix,
                        Some((dim.dim, dim.dim_increment)),
                        name_counts,
                        out,
                    );
                } else {
                    collect_once_regs(
                        &child_ctx,
                        &cluster.items,
                        base_offset + cluster.address_offset,
                        &child_prefix,
                        array_ctx,
                        name_counts,
                        out,
                    );
                }
            }
        }
    }
}

// --------------------------- writer + naming ---------------------------

struct CodeWriter {
    s: String,
    indent: usize,
}

impl Clone for CodeWriter {
    fn clone(&self) -> Self {
        Self {
            s: self.s.clone(),
            indent: self.indent,
        }
    }
}

impl Default for CodeWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeWriter {
    fn new() -> Self {
        Self {
            s: String::new(),
            indent: 0,
        }
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent = self.indent.saturating_sub(1);
    }

    fn writeln(&mut self, line: &str) -> Result<()> {
        for _ in 0..self.indent {
            self.s.push_str("    ");
        }
        self.s.push_str(line);
        self.s.push('\n');
        Ok(())
    }

    fn into_string(self) -> String {
        format!("{}\n", self.s.trim_end())
    }
}

fn sanitize_file_stem(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "device".to_string()
    } else {
        out
    }
}

fn sanitize_const_name(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_uppercase());
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "PERIPH".to_string()
    } else {
        out
    }
}

fn sanitize_field_name(s: &str) -> String {
    let s = s.replace("[%s]", "").replace("%s", "");
    let s_lower = s.to_ascii_lowercase();
    let s = if s_lower.contains("outset") {
        s.replace("OUTSET", "out_set").replace("outset", "out_set")
    } else if s_lower.contains("outclr") {
        s.replace("OUTCLR", "out_clr").replace("outclr", "out_clr")
    } else if s_lower.contains("dirset") {
        s.replace("DIRSET", "dir_set").replace("dirset", "dir_set")
    } else if s_lower.contains("dirclr") {
        s.replace("DIRCLR", "dir_clr").replace("dirclr", "dir_clr")
    } else if s_lower.contains("detectmode") {
        s.replace("DETECTMODE", "detect_mode")
            .replace("detectmode", "detect_mode")
    } else {
        s
    };
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        let good = ch.is_ascii_alphanumeric() || ch == '_';
        let ch = if good { ch } else { '_' };
        if i == 0 && ch.is_ascii_digit() {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    if out.is_empty() {
        "_field".to_string()
    } else if helpers::is_rust_keyword(&out) {
        format!("{out}_")
    } else {
        out
    }
}

fn sanitize_module_name(s: &str) -> String {
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        let good = ch.is_ascii_alphanumeric() || ch == '_';
        let ch = if good { ch } else { '_' };
        if i == 0 && ch.is_ascii_digit() {
            out.push('_');
        }
        out.push(ch);
    }
    if out.is_empty() {
        "periph".to_string()
    } else if helpers::is_rust_keyword(&out.to_ascii_lowercase()) {
        format!("{out}_")
    } else {
        out.to_ascii_lowercase()
    }
}

fn sanitize_type_name(s: &str) -> String {
    // CamelCase from tokens separated by non-alnum.
    // Remove [%s] and %s patterns used as array index placeholders in SVD.
    let s = s.replace("[%s]", "").replace("%s", "");
    let mut out = String::new();
    let mut upper_next = true;
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            if out.is_empty() && ch.is_ascii_digit() {
                out.push('_');
            }
            if upper_next {
                out.push(ch.to_ascii_uppercase());
                upper_next = false;
            } else {
                out.push(ch.to_ascii_lowercase());
            }
        } else {
            upper_next = true;
        }
    }
    if out.is_empty() {
        "Type".to_string()
    } else if helpers::is_rust_keyword(&out.to_ascii_lowercase()) {
        format!("{out}_")
    } else {
        out
    }
}

fn doc_escape(s: &str) -> String {
    s.replace('\n', " ").replace('\r', " ")
}

fn fix_register_case(name: &str) -> String {
    let name = if name.ends_with("Outset") {
        name.replacen("Outset", "OutSet", 1)
    } else if name.ends_with("Outclr") {
        name.replacen("Outclr", "OutClr", 1)
    } else if name.ends_with("Dirset") {
        name.replacen("Dirset", "DirSet", 1)
    } else if name.ends_with("Dirclr") {
        name.replacen("Dirclr", "DirClr", 1)
    } else if name.ends_with("Detectmode") {
        name.replacen("Detectmode", "DetectMode", 1)
    } else {
        name.to_string()
    };
    name
}

fn ctx_reg_path(ctx: &Ctx<'_>, reg_name: &str) -> String {
    if ctx.cluster_stack.is_empty() {
        reg_name.to_string()
    } else {
        let mut s = String::new();
        for (i, c) in ctx.cluster_stack.iter().enumerate() {
            if i > 0 {
                s.push('.');
            }
            s.push_str(&c.name);
        }
        s.push('.');
        s.push_str(reg_name);
        s
    }
}

fn register_wrapper_type(
    st: &mut GenState,
    out: &mut CodeWriter,
    ctx: &Ctx<'_>,
    r: &svd::Register,
    base_ty: &str,
    access: ResolvedAccess,
    options: PacOptions,
) -> Result<String> {
    // Base name: <PERIPH>_<CLUSTERS>_<REG>
    let periph = ctx.periph.map(|p| p.name.as_str()).unwrap_or("PERIPH");
    let reg_path = ctx_reg_path(ctx, &r.name);
    let base = sanitize_type_name(&format!("{}_{}", periph, reg_path.replace('.', "_")));

    // Fingerprint: access + base type + field layout + enum bindings.
    let mut fp = format!(
        "A={:?}|WM={:?}|T={base_ty}|",
        access.access, access.write_model
    );
    for (i, f) in r.field.iter().enumerate() {
        let (lsb, width) = field_lsb_width(f);
        fp.push_str(&format!("F{i}:{}:{}:{}|", f.name, lsb, width));
        for (j, evs) in f.enumerated_values.iter().enumerate() {
            fp.push_str(&format!(
                "E{j}:{}:{}:{};",
                evs.name.as_deref().unwrap_or(""),
                evs.header_enum_name.as_deref().unwrap_or(""),
                evs.usage.map(|u| format!("{u:?}")).unwrap_or_default()
            ));
        }
        fp.push('|');
    }

    let ty = if let Some(existing) = st.lookup_reg_type(&base, &fp) {
        existing
    } else {
        let new_ty = st.alloc_type_name(base.clone());
        st.remember_reg_type(&base, &fp, &new_ty);
        new_ty
    };

    if !st.mark_reg_emitted(&ty) {
        return Ok(ty);
    }

    // Underlying storage type (layout-critical).
    let inner = match (access.access, access.write_model) {
        (svd::AccessType::ReadOnly, _) => format!("RO<{base_ty}>"),
        (svd::AccessType::WriteOnly | svd::AccessType::WriteOnce, RegWriteModel::W1S) => {
            format!("W1S<{base_ty}>")
        }
        (svd::AccessType::WriteOnly | svd::AccessType::WriteOnce, RegWriteModel::W1C) => {
            format!("W1C<{base_ty}>")
        }
        (svd::AccessType::WriteOnly | svd::AccessType::WriteOnce, RegWriteModel::W0S) => {
            format!("W0S<{base_ty}>")
        }
        (svd::AccessType::WriteOnly | svd::AccessType::WriteOnce, RegWriteModel::W0C) => {
            format!("W0C<{base_ty}>")
        }
        (svd::AccessType::WriteOnly | svd::AccessType::WriteOnce, RegWriteModel::WT) => {
            format!("WT<{base_ty}>")
        }
        (svd::AccessType::WriteOnly | svd::AccessType::WriteOnce, _) => format!("WO<{base_ty}>"),
        (svd::AccessType::ReadWrite | svd::AccessType::ReadWriteOnce, RegWriteModel::W1S) => {
            format!("W1S<{base_ty}>")
        }
        (svd::AccessType::ReadWrite | svd::AccessType::ReadWriteOnce, RegWriteModel::W1C) => {
            format!("W1C<{base_ty}>")
        }
        (svd::AccessType::ReadWrite | svd::AccessType::ReadWriteOnce, RegWriteModel::W0S) => {
            format!("W0S<{base_ty}>")
        }
        (svd::AccessType::ReadWrite | svd::AccessType::ReadWriteOnce, RegWriteModel::W0C) => {
            format!("W0C<{base_ty}>")
        }
        (svd::AccessType::ReadWrite | svd::AccessType::ReadWriteOnce, RegWriteModel::WT) => {
            format!("WT<{base_ty}>")
        }
        (svd::AccessType::ReadWrite | svd::AccessType::ReadWriteOnce, _) => {
            format!("RW<{base_ty}>")
        }
    };

    out.writeln(&format!("/// Register `{}`", reg_path))?;
    if let Some(desc) = &r.description {
        out.writeln(&format!("/// {}", doc_escape(desc)))?;
    }
    out.writeln("#[repr(transparent)]")?;
    out.writeln(&format!("pub struct {ty}({inner});"))?;

    let has_read = matches!(
        access.access,
        svd::AccessType::ReadOnly | svd::AccessType::ReadWrite | svd::AccessType::ReadWriteOnce
    );
    let has_write = matches!(
        access.access,
        svd::AccessType::WriteOnly
            | svd::AccessType::WriteOnce
            | svd::AccessType::ReadWrite
            | svd::AccessType::ReadWriteOnce
    );
    let write_model = access.write_model;

    let macro_name = match (has_read, has_write, write_model) {
        (true, false, _) => "impl_ro_register",
        (false, true, RegWriteModel::Normal) => "impl_wo_register",
        (true, true, RegWriteModel::Normal) => "impl_rw_register",
        (_, true, RegWriteModel::W1S) => "impl_w1s_register",
        (_, true, RegWriteModel::W1C) => "impl_w1c_register",
        (_, true, RegWriteModel::W0S) => "impl_w0s_register",
        (_, true, RegWriteModel::W0C) => "impl_w0c_register",
        (_, true, RegWriteModel::WT) => "impl_wt_register",
        _ => "impl_rw_register",
    };

    out.writeln(&format!("{macro_name}!({ty}, {base_ty});"))?;

    // Field enum helpers.
    let mut has_field_methods = false;
    if options.emit_field_enums && options.emit_field_methods && !r.field.is_empty() {
        let reg_bits = (base_ty_bits(base_ty) as u64).max(1);
        let reg_path_key = ctx_reg_path(ctx, &r.name);
        let mut used_method: BTreeMap<String, usize> = BTreeMap::new();

        for f in &r.field {
            if f.enumerated_values.is_empty() {
                continue;
            }
            // Pick one enum for read and one for write (by usage).
            let mut read_pick: Option<(usize, String)> = None;
            let mut write_pick: Option<(usize, String)> = None;
            for (idx, evs) in f.enumerated_values.iter().enumerate() {
                let ty_opt = st.lookup_field_enum_ty(periph, &reg_path_key, &f.name, idx);
                let Some(enum_ty) = ty_opt else { continue };
                match evs.usage {
                    None | Some(svd::EnumUsage::ReadWrite) => {
                        if read_pick.is_none() {
                            read_pick = Some((idx, enum_ty.clone()));
                        }
                        if write_pick.is_none() {
                            write_pick = Some((idx, enum_ty));
                        }
                    }
                    Some(svd::EnumUsage::Read) => {
                        if read_pick.is_none() {
                            read_pick = Some((idx, enum_ty));
                        }
                    }
                    Some(svd::EnumUsage::Write) => {
                        if write_pick.is_none() {
                            write_pick = Some((idx, enum_ty));
                        }
                    }
                }
            }

            let (lsb, width) = field_lsb_width(f);
            let mask: u64 = if width >= 64 {
                u64::MAX
            } else {
                (1u64 << width) - 1
            };

            // Prefix field-related helpers with `field_` to avoid collisions with
            // register-level methods like `read()` / `write(..)` when a field is
            // named READ/WRITE/etc.
            let fname = sanitize_field_name(&f.name);
            let n = used_method.entry(fname.clone()).or_insert(0);
            let method_suffix = if *n == 0 {
                fname.clone()
            } else {
                format!("{fname}_{}", *n)
            };
            *n += 1;
            let method_base = format!("field_{method_suffix}");

            // Read helpers.
            if let Some((_idx, enum_ty)) = &read_pick {
                // Require readable register.
                if matches!(
                    access.access,
                    svd::AccessType::ReadOnly
                        | svd::AccessType::ReadWrite
                        | svd::AccessType::ReadWriteOnce
                ) {
                    if !has_field_methods {
                        has_field_methods = true;
                        out.writeln(&format!("impl {ty} {{"))?;
                        out.indent();
                    }
                    out.writeln("")?;
                    out.writeln(&format!("/// Field `{}`", f.name))?;
                    if let Some(d) = &f.description {
                        out.writeln(&format!("/// {}", doc_escape(d)))?;
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
                    out.writeln(&format!(
                        "pub fn {method_base}(&self) -> Option<field_enums::{enum_ty}> {{ field_enums::{enum_ty}::from_bits(self.{method_base}_raw() as {repr}) }}",
                        repr = helpers::repr_for_bits(width)
                    ))?;
                }
            }

            // Write helpers.
            if let Some((_idx, enum_ty)) = &write_pick {
                // Field access can restrict writing.
                let field_access = f.access.unwrap_or(access.access);
                let writable = !matches!(field_access, svd::AccessType::ReadOnly);
                if !writable {
                    continue;
                }

                // Prefer RMW only for readable+writeable (RW).
                if matches!(
                    access.access,
                    svd::AccessType::ReadWrite | svd::AccessType::ReadWriteOnce
                ) && access.write_model == RegWriteModel::Normal
                {
                    if !has_field_methods {
                        has_field_methods = true;
                        out.writeln(&format!("impl {ty} {{"))?;
                        out.indent();
                    }
                    let set_name = if read_pick.as_ref().map(|x| &x.1) == Some(enum_ty) {
                        format!("set_{method_base}")
                    } else {
                        format!("set_{method_base}_write")
                    };
                    out.writeln("")?;
                    out.writeln("#[inline(always)]")?;
                    out.writeln(&format!(
                        "pub fn {set_name}(&self, v: field_enums::{enum_ty}) {{"
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
                } else {
                    let method_name = if matches!(
                        access.access,
                        svd::AccessType::ReadWrite | svd::AccessType::ReadWriteOnce
                    ) {
                        if read_pick.as_ref().map(|x| &x.1) == Some(enum_ty) {
                            format!("set_{method_base}")
                        } else {
                            format!("set_{method_base}_write")
                        }
                    } else {
                        format!("write_{method_base}")
                    };

                    let call = match (access.access, access.write_model) {
                        (
                            svd::AccessType::ReadWrite | svd::AccessType::ReadWriteOnce,
                            RegWriteModel::W1S,
                        )
                        | (
                            svd::AccessType::WriteOnly | svd::AccessType::WriteOnce,
                            RegWriteModel::W1S,
                        ) => "set_bits",
                        (
                            svd::AccessType::ReadWrite | svd::AccessType::ReadWriteOnce,
                            RegWriteModel::W0S,
                        )
                        | (
                            svd::AccessType::WriteOnly | svd::AccessType::WriteOnce,
                            RegWriteModel::W0S,
                        ) => "set_bits",
                        (
                            svd::AccessType::ReadWrite | svd::AccessType::ReadWriteOnce,
                            RegWriteModel::W1C,
                        )
                        | (
                            svd::AccessType::WriteOnly | svd::AccessType::WriteOnce,
                            RegWriteModel::W1C,
                        ) => "clear_bits",
                        (
                            svd::AccessType::ReadWrite | svd::AccessType::ReadWriteOnce,
                            RegWriteModel::W0C,
                        )
                        | (
                            svd::AccessType::WriteOnly | svd::AccessType::WriteOnce,
                            RegWriteModel::W0C,
                        ) => "clear_bits",
                        (
                            svd::AccessType::ReadWrite | svd::AccessType::ReadWriteOnce,
                            RegWriteModel::WT,
                        )
                        | (
                            svd::AccessType::WriteOnly | svd::AccessType::WriteOnce,
                            RegWriteModel::WT,
                        ) => "toggle_bits",
                        (
                            svd::AccessType::WriteOnly | svd::AccessType::WriteOnce,
                            RegWriteModel::Normal,
                        ) => "write",
                        _ => {
                            continue;
                        }
                    };

                    if !has_field_methods {
                        has_field_methods = true;
                        out.writeln(&format!("impl {ty} {{"))?;
                        out.indent();
                    }
                    out.writeln("")?;
                    out.writeln("#[inline(always)]")?;
                    out.writeln(&format!(
                        "pub fn {method_name}(&self, v: field_enums::{enum_ty}) {{"
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
    }

    if has_field_methods {
        out.dedent();
        out.writeln("}")?;
    }
    out.writeln("")?;

    Ok(ty)
}

fn base_ty_bits(base_ty: &str) -> u32 {
    match base_ty {
        "u8" => 8,
        "u16" => 16,
        "u32" => 32,
        "u64" => 64,
        _ => 32,
    }
}

fn field_lsb_width(f: &svd::Field) -> (u32, u32) {
    match f.bit_range {
        svd::BitRange::BitRangeString { msb, lsb } => (lsb, msb.saturating_sub(lsb) + 1),
        svd::BitRange::LsbMsb { lsb, msb } => (lsb, msb.saturating_sub(lsb) + 1),
        svd::BitRange::BitOffsetWidth {
            bit_offset,
            bit_width,
        } => (bit_offset, bit_width.unwrap_or(1)),
    }
}

// --------------------------- enums (enumeratedValues) ---------------------------

fn emit_peripheral_enums(
    st: &mut GenState,
    device: &svd::Device,
    p: &svd::Peripheral,
    out: &mut CodeWriter,
) -> Result<()> {
    let items = peripheral_register_items(device, p);
    let mut regs: Vec<(&svd::Register, String)> = Vec::new();
    collect_registers(items, "", &mut regs);
    regs.sort_by(|a, b| a.1.cmp(&b.1));

    let mut has_enums = false;
    for (r, path) in &regs {
        for f in &r.field {
            for evs in &f.enumerated_values {
                if !has_enums {
                    has_enums = true;
                }
                emit_enum_for_field(device, p, r, f, evs, st, out, path)?;
            }
        }
    }

    Ok(())
}

fn collect_registers<'a>(
    items: &'a [svd::RegisterBlockItem],
    prefix: &str,
    out: &mut Vec<(&'a svd::Register, String)>,
) {
    for it in items {
        match it {
            svd::RegisterBlockItem::Register { register } => {
                let name = if prefix.is_empty() {
                    register.name.clone()
                } else {
                    format!("{prefix}.{}", register.name)
                };
                out.push((register, name));
            }
            svd::RegisterBlockItem::Cluster { cluster } => {
                let pfx = if prefix.is_empty() {
                    cluster.name.clone()
                } else {
                    format!("{prefix}.{}", cluster.name)
                };
                collect_registers(cluster.items.as_slice(), &pfx, out);
            }
        }
    }
}

fn emit_enum_for_field(
    device: &svd::Device,
    p: &svd::Peripheral,
    r: &svd::Register,
    f: &svd::Field,
    evs: &svd::EnumeratedValues,
    st: &mut GenState,
    out: &mut CodeWriter,
    reg_path: &str,
) -> Result<()> {
    // Base enum name: prefer headerEnumName/name; otherwise derive from path.
    // For fields like PIN0, PIN1, PIN2, extract common base (PIN) to enable deduplication.
    let field_name_for_enum = helpers::extract_enum_base_name(&f.name);
    let (usage_prefix, cap_first) = match evs.usage {
        Some(svd::EnumUsage::Read) => ("Read", true),
        Some(svd::EnumUsage::Write) => ("Write", true),
        _ => ("", false),
    };
    let base = evs
        .header_enum_name
        .as_deref()
        .or(evs.name.as_deref())
        .map(sanitize_type_name)
        .unwrap_or_else(|| {
            let name = sanitize_type_name(&format!(
                "{}_{}_{}",
                p.name,
                reg_path.replace('.', "_"),
                field_name_for_enum
            ));
            if cap_first {
                format!("{}{}", usage_prefix, name)
            } else {
                name
            }
        });

    let bit_width = helpers::field_bit_width(f);
    let repr = helpers::repr_for_bits(bit_width);

    let mut variants: Vec<(String, Option<u64>, Option<String>)> = Vec::new();
    for ev in &evs.enumerated_value {
        let vname = helpers::sanitize_variant_name(&ev.name);
        let val = match &ev.spec {
            svd::EnumeratedValueSpec::Value { value } => helpers::parse_enum_u64(value),
            svd::EnumeratedValueSpec::IsDefault { .. } => None,
        };
        variants.push((vname, val, ev.description.clone()));
    }

    // Fingerprint for dedup (repr + list of (name, value/none)).
    let mut fp = String::new();
    fp.push_str(repr);
    fp.push('|');
    for (n, v, _) in &variants {
        fp.push_str(n);
        fp.push('=');
        if let Some(v) = v {
            fp.push_str(&format!("{v}"));
        } else {
            fp.push_str("?");
        }
        fp.push(';');
    }

    let ty = if let Some(existing) = st.lookup_enum_type(&base, &fp) {
        existing
    } else {
        let new_ty = st.alloc_type_name(base.clone());
        st.remember_enum_type(&base, &fp, &new_ty);
        new_ty
    };

    // Remember mapping for later use in register wrappers.
    let evs_idx = f
        .enumerated_values
        .iter()
        .position(|x| core::ptr::eq(x, evs))
        .unwrap_or(0);
    st.remember_field_enum_ty(&p.name, reg_path, &f.name, evs_idx, &ty);

    if !st.mark_enum_emitted(&ty) {
        return Ok(());
    }

    // Ensure variant identifiers are unique.
    let mut used: BTreeMap<String, usize> = BTreeMap::new();
    for (name, _, _) in variants.iter_mut() {
        let n = used.entry(name.clone()).or_insert(0);
        if *n > 0 {
            *name = format!("{name}_{}", *n);
        }
        *n += 1;
    }

    let mut any_numeric = false;
    let mut enum_body = String::new();

    enum_body.push_str(&format!(
        r#"    #[doc = "{}.{} :: field `{}`"]"#,
        device.name, reg_path, f.name
    ));

    let variants_start = enum_body.len();
    enum_body.push_str(&format!("\n    {ty} : {repr},"));

    let last_idx = variants.len().saturating_sub(1);
    for (i, (name, val, desc)) in variants.iter().enumerate() {
        if let Some(d) = desc {
            enum_body.push_str(&format!("\n    #[doc = \"{}\"]", doc_escape(d)));
        }
        if let Some(v) = val {
            let max = if bit_width >= 64 {
                u64::MAX
            } else {
                (1u64 << bit_width) - 1
            };
            if *v > max {
                enum_body.push_str(&format!(
                    "\n    // {name} = {v}, // value does not fit into {bit_width} bits"
                ));
                continue;
            }
            any_numeric = true;
            let comma = if i < last_idx { "," } else { "" };
            enum_body.push_str(&format!("\n    {name} = {v}{comma}"));
        } else {
            enum_body.push_str(&format!("\n    // {name} = <non-const value>,"));
        }
    }

    if !any_numeric {
        enum_body.push_str(
            r#"
    #[doc = "No fully-constant values in SVD; placeholder."]"#,
        );
        enum_body.push_str(&format!("\n    __Reserved = 0"));
    }

    out.writeln(&format!("define_enum!(\n{enum_body}\n);"))?;
    out.writeln("")?;

    Ok(())
}
fn collect_device_interrupts(device: &svd::Device) -> (u32, Vec<(u32, String, Option<String>)>) {
    // Возвращает (num_irqs, список (irqn, name, desc)).
    //
    // В SVD interrupt.value — это номер IRQ начиная с 0 (первое внешнее прерывание).
    // Дедуп по номеру IRQ; если несколько имён у одного value — берём первое.
    let mut by_num: BTreeMap<u32, (String, Option<String>)> = BTreeMap::new();
    for p in &device.peripherals {
        for irq in &p.interrupt {
            if irq.value < 0 {
                continue;
            }
            let n = irq.value as u32;
            by_num.entry(n).or_insert_with(|| {
                (
                    helpers::sanitize_irq_handler_name(&irq.name),
                    irq.description.clone(),
                )
            });
        }
    }

    let max_seen = by_num.keys().next_back().copied().unwrap_or(0);
    let mut num_irqs = device
        .cpu
        .as_ref()
        .and_then(|c| c.device_num_interrupts)
        .unwrap_or(max_seen.saturating_add(1));
    num_irqs = num_irqs.max(max_seen.saturating_add(1));

    let list = by_num
        .into_iter()
        .map(|(n, (name, desc))| (n, name, desc))
        .collect::<Vec<_>>();
    (num_irqs, list)
}

fn generate_rt_rs(device: &svd::Device) -> Result<String> {
    let (num_irqs, irqs) = collect_device_interrupts(device);

    let mut out = CodeWriter::new();
    out.writeln("#[allow(non_snake_case)]")?;
    out.writeln("#[allow(dead_code)]")?;
    out.writeln("")?;
    out.writeln("use core::ptr;")?;
    out.writeln("")?;

    out.writeln("#[inline(always)]")?;
    out.writeln("pub unsafe fn enable_interrupts() {")?;
    out.indent();
    out.writeln("#[cfg(target_arch = \"arm\")]")?;
    out.writeln("core::arch::asm!(\"cpsie i\", options(nomem, nostack, preserves_flags));")?;
    out.dedent();
    out.writeln("}")?;
    out.writeln("#[inline(always)]")?;
    out.writeln("pub unsafe fn disable_interrupts() {")?;
    out.indent();
    out.writeln("#[cfg(target_arch = \"arm\")]")?;
    out.writeln("core::arch::asm!(\"cpsid i\", options(nomem, nostack, preserves_flags));")?;
    out.dedent();
    out.writeln("}")?;
    out.writeln("#[inline(always)]")?;
    out.writeln("pub unsafe fn wait_for_interrupt() {")?;
    out.indent();
    out.writeln("#[cfg(target_arch = \"arm\")]")?;
    out.writeln("core::arch::asm!(\"wfi\", options(nomem, nostack, preserves_flags));")?;
    out.dedent();
    out.writeln("}")?;
    out.writeln("")?;
    out.writeln("#[inline(always)]")?;
    out.writeln("pub unsafe fn nop() {")?;
    out.indent();
    out.writeln("#[cfg(target_arch = \"arm\")]")?;
    out.writeln("core::arch::asm!(\"nop\", options(nomem, nostack, preserves_flags));")?;
    out.dedent();
    out.writeln("}")?;
    out.writeln("")?;

    out.writeln("#[repr(C)]")?;
    out.writeln("pub union Vector {")?;
    out.indent();
    out.writeln("pub handler: unsafe extern \"C\" fn(),")?;
    out.writeln("pub reserved: usize,")?;
    out.dedent();
    out.writeln("}")?;
    out.writeln("")?;

    out.writeln("extern \"C\" {")?;
    out.indent();
    out.writeln("static mut __vector_table: u32;")?;
    out.writeln("static mut _sidata: u32;")?;
    out.writeln("static mut _sdata: u32;")?;
    out.writeln("static mut _edata: u32;")?;
    out.writeln("static mut _sbss: u32;")?;
    out.writeln("static mut _ebss: u32;")?;
    out.writeln("")?;
    out.writeln("fn main() -> !;")?;
    out.writeln("")?;
    // Core exception handlers (weak-provided by link.x).
    out.writeln("fn NMI();")?;
    out.writeln("fn HardFault();")?;
    out.writeln("fn MemManage();")?;
    out.writeln("fn BusFault();")?;
    out.writeln("fn UsageFault();")?;
    out.writeln("fn SVCall();")?;
    out.writeln("fn DebugMonitor();")?;
    out.writeln("fn PendSV();")?;
    out.writeln("fn SysTick();")?;
    out.writeln("")?;
    // IRQ handlers (weak-provided by link.x).
    for (_n, name, _desc) in &irqs {
        out.writeln(&format!("fn {name}();"))?;
    }
    out.dedent();
    out.writeln("}")?;
    out.writeln("")?;

    // Default handler (used by linker PROVIDE(..) fallbacks).
    //
    // Note: we intentionally use `fn()` (not `fn() -> !`) so user IRQ handlers can
    // naturally be written as `unsafe extern \"C\" fn NAME() { .. }`.
    out.writeln("#[unsafe(no_mangle)]")?;
    out.writeln("pub unsafe extern \"C\" fn DefaultHandler() {")?;
    out.indent();
    out.writeln("loop {")?;
    out.indent();
    out.writeln("core::hint::spin_loop();")?;
    out.dedent();
    out.writeln("}")?;
    out.dedent();
    out.writeln("}")?;
    out.writeln("")?;

    // Reset handler: init memory, optionally set VTOR, then call main().
    out.writeln("#[unsafe(no_mangle)]")?;
    out.writeln("pub unsafe extern \"C\" fn Reset() {")?;
    out.indent();
    out.writeln("let vtor = 0xE000_ED08usize as *mut u32;")?;
    out.writeln("ptr::write_volatile(vtor, ptr::addr_of!(__vector_table) as u32);")?;
    out.writeln("")?;
    out.writeln("let mut src = ptr::addr_of!(_sidata) as *const u32;")?;
    out.writeln("let mut dst = ptr::addr_of_mut!(_sdata) as *mut u32;")?;
    out.writeln("let end = ptr::addr_of_mut!(_edata) as *mut u32;")?;
    out.writeln("while (dst as usize) < (end as usize) {")?;
    out.indent();
    out.writeln("ptr::write(dst, ptr::read(src));")?;
    out.writeln("dst = dst.add(1);")?;
    out.writeln("src = src.add(1);")?;
    out.dedent();
    out.writeln("}")?;
    out.writeln("")?;
    out.writeln("let mut bss = ptr::addr_of_mut!(_sbss) as *mut u32;")?;
    out.writeln("let bss_end = ptr::addr_of_mut!(_ebss) as *mut u32;")?;
    out.writeln("while (bss as usize) < (bss_end as usize) {")?;
    out.indent();
    out.writeln("ptr::write(bss, 0);")?;
    out.writeln("bss = bss.add(1);")?;
    out.dedent();
    out.writeln("}")?;
    out.writeln("")?;
    out.writeln("main()")?;
    out.dedent();
    out.writeln("}")?;
    out.writeln("")?;

    out.writeln("#[used]")?;
    out.writeln("#[unsafe(link_section = \".vector_table.reset_vector\")]")?;
    out.writeln("pub static __RESET_VECTOR: [Vector; 1] = [")?;
    out.indent();
    out.writeln("Vector { handler: Reset },")?;
    out.dedent();
    out.writeln("];")?;
    out.writeln("")?;

    out.writeln("#[used]")?;
    out.writeln("#[unsafe(link_section = \".vector_table.exceptions\")]")?;
    out.writeln("pub static __EXCEPTIONS: [Vector; 14] = [")?;
    out.indent();
    out.writeln("Vector { handler: NMI },")?;
    out.writeln("Vector { handler: HardFault },")?;
    out.writeln("Vector { handler: MemManage },")?;
    out.writeln("Vector { handler: BusFault },")?;
    out.writeln("Vector { handler: UsageFault },")?;
    out.writeln("Vector { reserved: 0 },")?;
    out.writeln("Vector { reserved: 0 },")?;
    out.writeln("Vector { reserved: 0 },")?;
    out.writeln("Vector { reserved: 0 },")?;
    out.writeln("Vector { handler: SVCall },")?;
    out.writeln("Vector { handler: DebugMonitor },")?;
    out.writeln("Vector { reserved: 0 },")?;
    out.writeln("Vector { handler: PendSV },")?;
    out.writeln("Vector { handler: SysTick },")?;
    out.dedent();
    out.writeln("];")?;
    out.writeln("")?;

    // Interrupt table: dense array [0..num_irqs).
    let mut irq_slots: Vec<String> = vec!["DefaultHandler".to_string(); num_irqs as usize];
    for (n, name, _desc) in &irqs {
        if (*n as usize) < irq_slots.len() {
            irq_slots[*n as usize] = name.clone();
        }
    }

    out.writeln("#[used]")?;
    out.writeln("#[unsafe(link_section = \".vector_table.interrupts\")]")?;
    out.writeln(&format!(
        "pub static __INTERRUPTS: [Vector; {num}usize] = [",
        num = num_irqs
    ))?;
    out.indent();
    for h in &irq_slots {
        out.writeln(&format!("Vector {{ handler: {h} }},",))?;
    }
    out.dedent();
    out.writeln("];")?;
    out.writeln("")?;

    Ok(out.s)
}

fn generate_link_x(device: &svd::Device) -> Result<String> {
    let (num_irqs, irqs) = collect_device_interrupts(device);
    let mut out = CodeWriter::new();

    out.writeln("/* AUTO-GENERATED BY svdkit::pac (linker script) */")?;
    out.writeln("INCLUDE memory.x")?;
    out.writeln("")?;
    out.writeln("ENTRY(Reset)")?;
    out.writeln("")?;
    out.writeln("__stack_top = ORIGIN(RAM) + LENGTH(RAM);")?;
    out.writeln("")?;

    // Weak defaults for exception + irq handlers.
    out.writeln("PROVIDE(NMI = DefaultHandler);")?;
    out.writeln("PROVIDE(HardFault = DefaultHandler);")?;
    out.writeln("PROVIDE(MemManage = DefaultHandler);")?;
    out.writeln("PROVIDE(BusFault = DefaultHandler);")?;
    out.writeln("PROVIDE(UsageFault = DefaultHandler);")?;
    out.writeln("PROVIDE(SVCall = DefaultHandler);")?;
    out.writeln("PROVIDE(DebugMonitor = DefaultHandler);")?;
    out.writeln("PROVIDE(PendSV = DefaultHandler);")?;
    out.writeln("PROVIDE(SysTick = DefaultHandler);")?;
    for (_n, name, _desc) in &irqs {
        out.writeln(&format!("PROVIDE({name} = DefaultHandler);"))?;
    }
    out.writeln("")?;

    out.writeln("SECTIONS")?;
    out.writeln("{")?;
    out.indent();

    out.writeln(".vector_table ORIGIN(FLASH) :")?;
    out.writeln("{")?;
    out.indent();
    out.writeln("__vector_table = .;")?;
    out.writeln("LONG(__stack_top);")?;
    out.writeln("KEEP(*(.vector_table.reset_vector))")?;
    out.writeln("KEEP(*(.vector_table.exceptions))")?;
    out.writeln("KEEP(*(.vector_table.interrupts))")?;
    out.dedent();
    out.writeln("} > FLASH")?;
    out.writeln("")?;

    out.writeln(".text :")?;
    out.writeln("{")?;
    out.indent();
    out.writeln("*(.text .text.*)")?;
    out.writeln("*(.rodata .rodata.*)")?;
    out.writeln("*(.glue_7 .glue_7t)")?;
    out.writeln("*(.eh_frame)")?;
    out.dedent();
    out.writeln("} > FLASH")?;
    out.writeln("")?;

    out.writeln(".data : ALIGN(4)")?;
    out.writeln("{")?;
    out.indent();
    out.writeln("_sdata = .;")?;
    out.writeln("*(.data .data.*)")?;
    out.writeln("*(.got .got.*)")?;
    out.writeln("*(.got.plt)")?;
    out.writeln("_edata = .;")?;
    out.dedent();
    out.writeln("} > RAM AT > FLASH")?;
    out.writeln("_sidata = LOADADDR(.data);")?;
    out.writeln("")?;

    out.writeln(".bss (NOLOAD) : ALIGN(4)")?;
    out.writeln("{")?;
    out.indent();
    out.writeln("_sbss = .;")?;
    out.writeln("*(.bss .bss.*)")?;
    out.writeln("*(.sbss .sbss.*)")?;
    out.writeln("*(COMMON)")?;
    out.writeln("_ebss = .;")?;
    out.dedent();
    out.writeln("} > RAM")?;
    out.writeln("")?;

    out.writeln("/DISCARD/ : { *(.note .note.*) *(.comment) }")?;
    out.dedent();
    out.writeln("}")?;
    out.writeln("")?;

    out.writeln("/* Sanity: expected IRQ count */")?;
    out.writeln(&format!("/* device_num_interrupts = {num_irqs} */"))?;

    Ok(out.s)
}
