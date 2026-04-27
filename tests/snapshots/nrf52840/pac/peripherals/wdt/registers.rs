use super::super::super::macros::*;
use super::super::super::types::{RW, RO, WO};

/// Register `TASKS_START`
/// Start the watchdog
#[repr(transparent)]
pub struct WdtTasksStart(WO<u32>);
impl_wo_register!(WdtTasksStart, u32);

/// Register `EVENTS_TIMEOUT`
/// Watchdog timeout
#[repr(transparent)]
pub struct WdtEventsTimeout(RW<u32>);
impl_rw_register!(WdtEventsTimeout, u32);

/// Register `INTENSET`
/// Enable interrupt
#[repr(transparent)]
pub struct WdtIntEnSet(RW<u32>);
impl_rw_register!(WdtIntEnSet, u32);

/// Register `INTENCLR`
/// Disable interrupt
#[repr(transparent)]
pub struct WdtIntEnClr(RW<u32>);
impl_rw_register!(WdtIntEnClr, u32);

/// Register `RUNSTATUS`
/// Run status
#[repr(transparent)]
pub struct WdtRunStatus(RO<u32>);
impl_ro_register!(WdtRunStatus, u32);

/// Register `REQSTATUS`
/// Request status
#[repr(transparent)]
pub struct WdtReqStatus(RO<u32>);
impl_ro_register!(WdtReqStatus, u32);

/// Register `CRV`
/// Counter reload value
#[repr(transparent)]
pub struct WdtCrv(RW<u32>);
impl_rw_register!(WdtCrv, u32);

/// Register `RREN`
/// Enable register for reload request registers
#[repr(transparent)]
pub struct WdtRren(RW<u32>);
impl_rw_register!(WdtRren, u32);

/// Register `CONFIG`
/// Configuration register
#[repr(transparent)]
pub struct WdtConfig(RW<u32>);
impl_rw_register!(WdtConfig, u32);

/// Register `RR[%s]`
/// Description collection: Reload request n
#[repr(transparent)]
pub struct WdtRr(WO<u32>);
impl_wo_register!(WdtRr, u32);
