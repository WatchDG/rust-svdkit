define_enum!(
    #[doc = "nrf52840.DETECTMODE :: field `DETECTMODE`"]
    DetectMode : u8,
    #[doc = "DETECT directly connected to PIN DETECT signals"]
    Default = 0,
    #[doc = "Use the latched LDETECT behavior"]
    LDetect = 1
);

define_enum!(
    #[doc = "nrf52840.DIR :: field `PIN0`"]
    DirPin : u8,
    #[doc = "Pin set as input"]
    Input = 0,
    #[doc = "Pin set as output"]
    Output = 1
);

define_enum!(
    #[doc = "nrf52840.DIRCLR :: field `PIN0`"]
    ReadDirClrPin : u8,
    #[doc = "Read: pin set as input"]
    Input = 0,
    #[doc = "Read: pin set as output"]
    Output = 1
);

define_enum!(
    #[doc = "nrf52840.DIRCLR :: field `PIN0`"]
    WriteDirClrPin : u8,
    #[doc = "Write: a '1' sets pin to input; a '0' has no effect"]
    Clear = 1
);

define_enum!(
    #[doc = "nrf52840.DIRSET :: field `PIN0`"]
    ReadDirSetPin : u8,
    #[doc = "Read: pin set as input"]
    Input = 0,
    #[doc = "Read: pin set as output"]
    Output = 1
);

define_enum!(
    #[doc = "nrf52840.DIRSET :: field `PIN0`"]
    WriteDirSetPin : u8,
    #[doc = "Write: a '1' sets pin to output; a '0' has no effect"]
    Set = 1
);

define_enum!(
    #[doc = "nrf52840.IN :: field `PIN0`"]
    InPin : u8,
    #[doc = "Pin input is low"]
    Low = 0,
    #[doc = "Pin input is high"]
    High = 1
);

define_enum!(
    #[doc = "nrf52840.LATCH :: field `PIN0`"]
    LatchPin : u8,
    #[doc = "Criteria has not been met"]
    NotLatched = 0,
    #[doc = "Criteria has been met"]
    Latched = 1
);

define_enum!(
    #[doc = "nrf52840.OUT :: field `PIN0`"]
    OutPin : u8,
    #[doc = "Pin driver is low"]
    Low = 0,
    #[doc = "Pin driver is high"]
    High = 1
);

define_enum!(
    #[doc = "nrf52840.OUTCLR :: field `PIN0`"]
    ReadOutClrPin : u8,
    #[doc = "Read: pin driver is low"]
    Low = 0,
    #[doc = "Read: pin driver is high"]
    High = 1
);

define_enum!(
    #[doc = "nrf52840.OUTCLR :: field `PIN0`"]
    WriteOutClrPin : u8,
    #[doc = "Write: a '1' sets the pin low; a '0' has no effect"]
    Clear = 1
);

define_enum!(
    #[doc = "nrf52840.OUTSET :: field `PIN0`"]
    ReadOutSetPin : u8,
    #[doc = "Read: pin driver is low"]
    Low = 0,
    #[doc = "Read: pin driver is high"]
    High = 1
);

define_enum!(
    #[doc = "nrf52840.OUTSET :: field `PIN0`"]
    WriteOutSetPin : u8,
    #[doc = "Write: a '1' sets the pin high; a '0' has no effect"]
    Set = 1
);

define_enum!(
    #[doc = "nrf52840.PIN_CNF[%s] :: field `DIR`"]
    PinCnfDir : u8,
    #[doc = "Configure pin as an input pin"]
    Input = 0,
    #[doc = "Configure pin as an output pin"]
    Output = 1
);

define_enum!(
    #[doc = "nrf52840.PIN_CNF[%s] :: field `INPUT`"]
    PinCnfInput : u8,
    #[doc = "Connect input buffer"]
    Connect = 0,
    #[doc = "Disconnect input buffer"]
    Disconnect = 1
);

define_enum!(
    #[doc = "nrf52840.PIN_CNF[%s] :: field `PULL`"]
    PinCnfPull : u8,
    #[doc = "No pull"]
    Disabled = 0,
    #[doc = "Pull down on pin"]
    PullDown = 1,
    #[doc = "Pull up on pin"]
    PullUp = 3
);

define_enum!(
    #[doc = "nrf52840.PIN_CNF[%s] :: field `DRIVE`"]
    PinCnfDrive : u8,
    #[doc = "Standard '0', standard '1'"]
    S0s1 = 0,
    #[doc = "High drive '0', standard '1'"]
    H0s1 = 1,
    #[doc = "Standard '0', high drive '1'"]
    S0h1 = 2,
    #[doc = "High drive '0', high 'drive '1''"]
    H0h1 = 3,
    #[doc = "Disconnect '0' standard '1' (normally used for wired-or connections)"]
    D0s1 = 4,
    #[doc = "Disconnect '0', high drive '1' (normally used for wired-or connections)"]
    D0h1 = 5,
    #[doc = "Standard '0'. disconnect '1' (normally used for wired-and connections)"]
    S0d1 = 6,
    #[doc = "High drive '0', disconnect '1' (normally used for wired-and connections)"]
    H0d1 = 7
);

define_enum!(
    #[doc = "nrf52840.PIN_CNF[%s] :: field `SENSE`"]
    PinCnfSense : u8,
    #[doc = "Disabled"]
    Disabled = 0,
    #[doc = "Sense for high level"]
    High = 2,
    #[doc = "Sense for low level"]
    Low = 3
);
