use super::super::super::macros::*;
use super::super::super::types::{RW, RO, W1S, W1C};

/// Register `OUT`
/// Write GPIO port
#[repr(transparent)]
pub struct Out(RW<u32>);
impl_rw_register!(Out, u32);

/// Register `OUTSET`
/// Set individual bits in GPIO port
#[repr(transparent)]
pub struct OutSet(W1S<u32>);
impl_w1s_register!(OutSet, u32);

/// Register `OUTCLR`
/// Clear individual bits in GPIO port
#[repr(transparent)]
pub struct OutClr(W1C<u32>);
impl_w1c_register!(OutClr, u32);

/// Register `IN`
/// Read GPIO port
#[repr(transparent)]
pub struct In_(RO<u32>);
impl_ro_register!(In_, u32);

/// Register `DIR`
/// Direction of GPIO pins
#[repr(transparent)]
pub struct Dir(RW<u32>);
impl_rw_register!(Dir, u32);

/// Register `DIRSET`
/// DIR set register
#[repr(transparent)]
pub struct DirSet(W1S<u32>);
impl_w1s_register!(DirSet, u32);

/// Register `DIRCLR`
/// DIR clear register
#[repr(transparent)]
pub struct DirClr(W1C<u32>);
impl_w1c_register!(DirClr, u32);

/// Register `LATCH`
/// Latch register indicating what GPIO pins that have met the criteria set in the PIN_CNF[n].SENSE registers
#[repr(transparent)]
pub struct Latch(RW<u32>);
impl_rw_register!(Latch, u32);

/// Register `DETECTMODE`
/// Select between default DETECT signal behavior and LDETECT mode
#[repr(transparent)]
pub struct DetectMode(RW<u32>);
impl_rw_register!(DetectMode, u32);

/// Register `PIN_CNF[%s]`
/// Description collection: Configuration of GPIO pins
#[repr(transparent)]
pub struct PinCnf(RW<u32>);
impl_rw_register!(PinCnf, u32);
