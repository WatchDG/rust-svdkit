//! Static content generators for PAC: macros, traits, types.
//! These do not depend on the SVD device content.

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
use super::common_traits::RegValue;

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
