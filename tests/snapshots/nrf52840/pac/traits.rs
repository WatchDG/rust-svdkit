pub trait RegValue: Copy {
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
