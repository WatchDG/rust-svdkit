use super::super::super::macros::*;
use super::super::super::types::{RW, WO};

/// Register `TASKS_START`
/// Start Timer
#[repr(transparent)]
pub struct TasksStart(WO<u32>);
impl_wo_register!(TasksStart, u32);

/// Register `TASKS_STOP`
/// Stop Timer
#[repr(transparent)]
pub struct TasksStop(WO<u32>);
impl_wo_register!(TasksStop, u32);

/// Register `TASKS_COUNT`
/// Increment Timer (Counter mode only)
#[repr(transparent)]
pub struct TasksCount(WO<u32>);
impl_wo_register!(TasksCount, u32);

/// Register `TASKS_CLEAR`
/// Clear time
#[repr(transparent)]
pub struct TasksClear(WO<u32>);
impl_wo_register!(TasksClear, u32);

/// Register `TASKS_SHUTDOWN`
/// Deprecated register - Shut down timer
#[repr(transparent)]
pub struct TasksShutdown(WO<u32>);
impl_wo_register!(TasksShutdown, u32);

/// Register `TASKS_CAPTURE[%s]`
/// Description collection: Capture Timer value to CC[n] register
#[repr(transparent)]
pub struct TasksCapture(WO<u32>);
impl_wo_register!(TasksCapture, u32);

/// Register `EVENTS_COMPARE[%s]`
/// Description collection: Compare event on CC[n] match
#[repr(transparent)]
pub struct EventsCompare(RW<u32>);
impl_rw_register!(EventsCompare, u32);

/// Register `SHORTS`
/// Shortcuts between local events and tasks
#[repr(transparent)]
pub struct Shorts(RW<u32>);
impl_rw_register!(Shorts, u32);

/// Register `INTENSET`
/// Enable interrupt
#[repr(transparent)]
pub struct IntEnSet(RW<u32>);
impl_rw_register!(IntEnSet, u32);

/// Register `INTENCLR`
/// Disable interrupt
#[repr(transparent)]
pub struct IntEnClr(RW<u32>);
impl_rw_register!(IntEnClr, u32);

/// Register `MODE`
/// Timer mode selection
#[repr(transparent)]
pub struct Mode(RW<u32>);
impl_rw_register!(Mode, u32);

/// Register `BITMODE`
/// Configure the number of bits used by the TIMER
#[repr(transparent)]
pub struct Bitmode(RW<u32>);
impl_rw_register!(Bitmode, u32);

/// Register `PRESCALER`
/// Timer prescaler register
#[repr(transparent)]
pub struct Prescaler(RW<u32>);
impl_rw_register!(Prescaler, u32);

/// Register `CC[%s]`
/// Description collection: Capture/Compare register n
#[repr(transparent)]
pub struct Cc(RW<u32>);
impl_rw_register!(Cc, u32);
