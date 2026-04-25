use super::super::super::types::{RW, RO, WO, W1S, W1C, W0S, W0C, WT};
use super::super::super::macros::*;

/// Register `TASKS_START`
/// Start Timer
#[repr(transparent)]
pub struct Timer0TasksStart(WO<u32>);
impl_wo_register!(Timer0TasksStart, u32);

/// Register `TASKS_STOP`
/// Stop Timer
#[repr(transparent)]
pub struct Timer0TasksStop(WO<u32>);
impl_wo_register!(Timer0TasksStop, u32);

/// Register `TASKS_COUNT`
/// Increment Timer (Counter mode only)
#[repr(transparent)]
pub struct Timer0TasksCount(WO<u32>);
impl_wo_register!(Timer0TasksCount, u32);

/// Register `TASKS_CLEAR`
/// Clear time
#[repr(transparent)]
pub struct Timer0TasksClear(WO<u32>);
impl_wo_register!(Timer0TasksClear, u32);

/// Register `TASKS_SHUTDOWN`
/// Deprecated register - Shut down timer
#[repr(transparent)]
pub struct Timer0TasksShutdown(WO<u32>);
impl_wo_register!(Timer0TasksShutdown, u32);

/// Register `TASKS_CAPTURE[%s]`
/// Description collection: Capture Timer value to CC[n] register
#[repr(transparent)]
pub struct Timer0TasksCapture(WO<u32>);
impl_wo_register!(Timer0TasksCapture, u32);

/// Register `EVENTS_COMPARE[%s]`
/// Description collection: Compare event on CC[n] match
#[repr(transparent)]
pub struct Timer0EventsCompare(RW<u32>);
impl_rw_register!(Timer0EventsCompare, u32);

/// Register `SHORTS`
/// Shortcuts between local events and tasks
#[repr(transparent)]
pub struct Timer0Shorts(RW<u32>);
impl_rw_register!(Timer0Shorts, u32);

/// Register `INTENSET`
/// Enable interrupt
#[repr(transparent)]
pub struct Timer0IntEnSet(RW<u32>);
impl_rw_register!(Timer0IntEnSet, u32);

/// Register `INTENCLR`
/// Disable interrupt
#[repr(transparent)]
pub struct Timer0IntEnClr(RW<u32>);
impl_rw_register!(Timer0IntEnClr, u32);

/// Register `MODE`
/// Timer mode selection
#[repr(transparent)]
pub struct Timer0Mode(RW<u32>);
impl_rw_register!(Timer0Mode, u32);

/// Register `BITMODE`
/// Configure the number of bits used by the TIMER
#[repr(transparent)]
pub struct Timer0Bitmode(RW<u32>);
impl_rw_register!(Timer0Bitmode, u32);

/// Register `PRESCALER`
/// Timer prescaler register
#[repr(transparent)]
pub struct Timer0Prescaler(RW<u32>);
impl_rw_register!(Timer0Prescaler, u32);

/// Register `CC[%s]`
/// Description collection: Capture/Compare register n
#[repr(transparent)]
pub struct Timer0Cc(RW<u32>);
impl_rw_register!(Timer0Cc, u32);
