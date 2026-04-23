use super::{RW, RO, WO, W1S, W1C, W0S, W0C, WT};
use super::macros::*;

/// Register `OUT`
/// Write GPIO port
#[repr(transparent)]
pub struct P0Out(RW<u32>);
impl_rw_register!(P0Out, u32);

/// Register `OUTSET`
/// Set individual bits in GPIO port
#[repr(transparent)]
pub struct P0OutSet(W1S<u32>);
impl_w1s_register!(P0OutSet, u32);

/// Register `OUTCLR`
/// Clear individual bits in GPIO port
#[repr(transparent)]
pub struct P0OutClr(W1C<u32>);
impl_w1c_register!(P0OutClr, u32);

/// Register `IN`
/// Read GPIO port
#[repr(transparent)]
pub struct P0In(RO<u32>);
impl_ro_register!(P0In, u32);

/// Register `DIR`
/// Direction of GPIO pins
#[repr(transparent)]
pub struct P0Dir(RW<u32>);
impl_rw_register!(P0Dir, u32);

/// Register `DIRSET`
/// DIR set register
#[repr(transparent)]
pub struct P0DirSet(W1S<u32>);
impl_w1s_register!(P0DirSet, u32);

/// Register `DIRCLR`
/// DIR clear register
#[repr(transparent)]
pub struct P0DirClr(W1C<u32>);
impl_w1c_register!(P0DirClr, u32);

/// Register `LATCH`
/// Latch register indicating what GPIO pins that have met the criteria set in the PIN_CNF[n].SENSE registers
#[repr(transparent)]
pub struct P0Latch(RW<u32>);
impl_rw_register!(P0Latch, u32);

/// Register `DETECTMODE`
/// Select between default DETECT signal behavior and LDETECT mode
#[repr(transparent)]
pub struct P0DetectMode(RW<u32>);
impl_rw_register!(P0DetectMode, u32);

/// Register `PIN_CNF[%s]`
/// Description collection: Configuration of GPIO pins
#[repr(transparent)]
pub struct P0PinCnf(RW<u32>);
impl_rw_register!(P0PinCnf, u32);
