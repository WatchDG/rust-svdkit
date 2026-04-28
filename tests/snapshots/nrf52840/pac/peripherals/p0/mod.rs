pub const BASE: usize = 0x50000000;

pub mod enums;
pub mod registers;

#[repr(C)]
pub struct P0 {
    pub _reserved_0: [u8; 1284 as usize],
    pub out: registers::Out,
    pub out_set: registers::OutSet,
    pub out_clr: registers::OutClr,
    pub in_: registers::In_,
    pub dir: registers::Dir,
    pub dir_set: registers::DirSet,
    pub dir_clr: registers::DirClr,
    pub latch: registers::Latch,
    pub detect_mode: registers::DetectMode,
    pub _reserved_1: [u8; 472 as usize],
    pub pin_cnf: [registers::PinCnf; 32 as usize],
}
impl P0 {
    #[inline(always)]
    pub fn reset(&self) {
        self.out.write(0x00000000u32);
        self.dir.write(0x00000000u32);
        self.latch.write(0x00000000u32);
        self.detect_mode.write(0x00000000u32);
        for r in self.pin_cnf.iter() { r.write(0x00000002u32); }
    }
}

pub const PTR: *const P0 = BASE as *const P0;
pub const PTR_MUT: *mut P0 = BASE as *mut P0;
