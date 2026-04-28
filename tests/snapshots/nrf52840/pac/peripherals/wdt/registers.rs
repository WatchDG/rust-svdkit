use super::super::super::macros::*;
use super::super::super::types::{RW, RO, WO};

/// Register `TASKS_START`
/// Start the watchdog
#[repr(transparent)]
pub struct TasksStart(WO<u32>);
impl_wo_register!(TasksStart, u32);

/// Register `EVENTS_TIMEOUT`
/// Watchdog timeout
#[repr(transparent)]
pub struct EventsTimeout(RW<u32>);
impl_rw_register!(EventsTimeout, u32);

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

/// Register `RUNSTATUS`
/// Run status
#[repr(transparent)]
pub struct RunStatus(RO<u32>);
impl_ro_register!(RunStatus, u32);

/// Register `REQSTATUS`
/// Request status
#[repr(transparent)]
pub struct ReqStatus(RO<u32>);
impl_ro_register!(ReqStatus, u32);

/// Register `CRV`
/// Counter reload value
#[repr(transparent)]
pub struct Crv(RW<u32>);
impl_rw_register!(Crv, u32);

/// Register `RREN`
/// Enable register for reload request registers
#[repr(transparent)]
pub struct Rren(RW<u32>);
impl_rw_register!(Rren, u32);

/// Register `CONFIG`
/// Configuration register
#[repr(transparent)]
pub struct Config(RW<u32>);
impl_rw_register!(Config, u32);

/// Register `RR[%s]`
/// Description collection: Reload request n
#[repr(transparent)]
pub struct Rr(WO<u32>);
impl_wo_register!(Rr, u32);
