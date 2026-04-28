define_enum!(
    #[doc = "nrf52840.BITMODE :: field `BITMODE`"]
    Bitmode : u8,
    #[doc = "16 bit timer bit width"]
    _16bit = 0,
    #[doc = "8 bit timer bit width"]
    _08bit = 1,
    #[doc = "24 bit timer bit width"]
    _24bit = 2,
    #[doc = "32 bit timer bit width"]
    _32bit = 3
);

define_enum!(
    #[doc = "nrf52840.EVENTS_COMPARE[%s] :: field `EVENTS_COMPARE`"]
    EventsCompare : u8,
    #[doc = "Event not generated"]
    NotGenerated = 0,
    #[doc = "Event generated"]
    Generated = 1
);

define_enum!(
    #[doc = "nrf52840.INTENCLR :: field `COMPARE0`"]
    ReadIntEnClrCompare : u8,
    #[doc = "Read: Disabled"]
    Disabled = 0,
    #[doc = "Read: Enabled"]
    Enabled = 1
);

define_enum!(
    #[doc = "nrf52840.INTENCLR :: field `COMPARE0`"]
    WriteIntEnClrCompare : u8,
    #[doc = "Disable"]
    Clear = 1
);

define_enum!(
    #[doc = "nrf52840.INTENSET :: field `COMPARE0`"]
    ReadIntEnSetCompare : u8,
    #[doc = "Read: Disabled"]
    Disabled = 0,
    #[doc = "Read: Enabled"]
    Enabled = 1
);

define_enum!(
    #[doc = "nrf52840.INTENSET :: field `COMPARE0`"]
    WriteIntEnSetCompare : u8,
    #[doc = "Enable"]
    Set = 1
);

define_enum!(
    #[doc = "nrf52840.MODE :: field `MODE`"]
    Mode : u8,
    #[doc = "Select Timer mode"]
    Timer = 0,
    #[doc = "Deprecated enumerator -  Select Counter mode"]
    Counter = 1,
    #[doc = "Select Low Power Counter mode"]
    LowPowerCounter = 2
);

define_enum!(
    #[doc = "nrf52840.SHORTS :: field `COMPARE0_CLEAR`"]
    ShortsCompare : u8,
    #[doc = "Disable shortcut"]
    Disabled = 0,
    #[doc = "Enable shortcut"]
    Enabled = 1
);

define_enum!(
    #[doc = "nrf52840.TASKS_CAPTURE[%s] :: field `TASKS_CAPTURE`"]
    TasksCapture : u8,
    #[doc = "Trigger task"]
    Trigger = 1
);

define_enum!(
    #[doc = "nrf52840.TASKS_CLEAR :: field `TASKS_CLEAR`"]
    TasksClear : u8,
    #[doc = "Trigger task"]
    Trigger = 1
);

define_enum!(
    #[doc = "nrf52840.TASKS_COUNT :: field `TASKS_COUNT`"]
    TasksCount : u8,
    #[doc = "Trigger task"]
    Trigger = 1
);

define_enum!(
    #[doc = "nrf52840.TASKS_SHUTDOWN :: field `TASKS_SHUTDOWN`"]
    TasksShutdown : u8,
    #[doc = "Trigger task"]
    Trigger = 1
);

define_enum!(
    #[doc = "nrf52840.TASKS_START :: field `TASKS_START`"]
    TasksStart : u8,
    #[doc = "Trigger task"]
    Trigger = 1
);

define_enum!(
    #[doc = "nrf52840.TASKS_STOP :: field `TASKS_STOP`"]
    TasksStop : u8,
    #[doc = "Trigger task"]
    Trigger = 1
);
