use super::gpio;
use crate::{Result, svd};

pub const CDC_ACM_DESCRIPTOR_LEN: usize = 75;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineCoding {
    pub dw_dte_rate: u32,
    pub char_format: u8,
    pub parity_type: u8,
    pub data_bits: u8,
}

impl Default for LineCoding {
    fn default() -> Self {
        Self {
            dw_dte_rate: 9600,
            char_format: 0,
            parity_type: 0,
            data_bits: 8,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlLineState {
    pub dtr: bool,
    pub rts: bool,
}

impl Default for ControlLineState {
    fn default() -> Self {
        Self { dtr: false, rts: false }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum SerialStateBit {
    None = 0,
    RxCarrier = 1 << 0,
    TxCarrier = 1 << 1,
    Break = 1 << 2,
    RingSignal = 1 << 3,
    Framing = 1 << 4,
    Parity = 1 << 5,
    OverRun = 1 << 6,
}

impl SerialStateBit {
    pub fn to_u16(self) -> u16 {
        self as u16
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SerialState {
    bits: u16,
}

impl Default for SerialState {
    fn default() -> Self {
        Self { bits: 0 }
    }
}

impl SerialState {
    pub fn new() -> Self {
        Self { bits: 0 }
    }

    pub fn with_rx_carrier(mut self) -> Self {
        self.bits |= SerialStateBit::RxCarrier as u16;
        self
    }

    pub fn with_tx_carrier(mut self) -> Self {
        self.bits |= SerialStateBit::TxCarrier as u16;
        self
    }

    pub fn with_break(mut self) -> Self {
        self.bits |= SerialStateBit::Break as u16;
        self
    }

    pub fn with_ring_signal(mut self) -> Self {
        self.bits |= SerialStateBit::RingSignal as u16;
        self
    }

    pub fn with_framing_error(mut self) -> Self {
        self.bits |= SerialStateBit::Framing as u16;
        self
    }

    pub fn with_parity_error(mut self) -> Self {
        self.bits |= SerialStateBit::Parity as u16;
        self
    }

    pub fn with_overrun(mut self) -> Self {
        self.bits |= SerialStateBit::OverRun as u16;
        self
    }

    pub fn to_u16(self) -> u16 {
        self.bits
    }

    pub fn bits(&self) -> u16 {
        self.bits
    }
}

#[derive(Debug, Clone)]
pub struct CdcAcmConfig {
    pub vid: u16,
    pub pid: u16,
    pub manufacturer: Option<&'static str>,
    pub product: Option<&'static str>,
    pub serial: Option<&'static str>,
    pub max_power_ma: u8,
    pub self_powered: bool,
    pub remote_wakeup: bool,
    pub device_class: u8,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub data_endpoint_size: u16,
    pub notify_endpoint_size: u16,
}

impl Default for CdcAcmConfig {
    fn default() -> Self {
        Self {
            vid: 0x2341,
            pid: 0x0042,
            manufacturer: Some("Vendor"),
            product: Some("USB CDC ACM Device"),
            serial: Some("0001"),
            max_power_ma: 100,
            self_powered: true,
            remote_wakeup: false,
            device_class: 0x02,
            device_subclass: 0x00,
            device_protocol: 0x00,
            data_endpoint_size: 64,
            notify_endpoint_size: 8,
        }
    }
}

pub struct CdcAcmConfigurator<'a> {
    config: CdcAcmConfig,
    line_coding: LineCoding,
    control_line_state: ControlLineState,
    serial_state: SerialState,
    _marker: core::marker::PhantomData<&'a ()>,
}

impl<'a> CdcAcmConfigurator<'a> {
    pub fn new() -> Self {
        Self {
            config: CdcAcmConfig::default(),
            line_coding: LineCoding::default(),
            control_line_state: ControlLineState::default(),
            serial_state: SerialState::new(),
            _marker: core::marker::PhantomData,
        }
    }

    #[inline(always)]
    pub fn vendor_id(mut self, vid: u16) -> Self {
        self.config.vid = vid;
        self
    }

    #[inline(always)]
    pub fn product_id(mut self, pid: u16) -> Self {
        self.config.pid = pid;
        self
    }

    #[inline(always)]
    pub fn manufacturer(mut self, m: &'static str) -> Self {
        self.config.manufacturer = Some(m);
        self
    }

    #[inline(always)]
    pub fn product(mut self, p: &'static str) -> Self {
        self.config.product = Some(p);
        self
    }

    #[inline(always)]
    pub fn serial_number(mut self, s: &'static str) -> Self {
        self.config.serial = Some(s);
        self
    }

    #[inline(always)]
    pub fn max_power_ma(mut self, ma: u8) -> Self {
        self.config.max_power_ma = ma;
        self
    }

    #[inline(always)]
    pub fn self_powered(mut self, v: bool) -> Self {
        self.config.self_powered = v;
        self
    }

    #[inline(always)]
    pub fn remote_wakeup(mut self, v: bool) -> Self {
        self.config.remote_wakeup = v;
        self
    }

    #[inline(always)]
    pub fn device_class(mut self, c: u8) -> Self {
        self.config.device_class = c;
        self
    }

    #[inline(always)]
    pub fn use_iad(mut self) -> Self {
        self.config.device_class = 0xEF;
        self.config.device_subclass = 0x02;
        self.config.device_protocol = 0x01;
        self
    }

    #[inline(always)]
    pub fn data_endpoint_size(mut self, size: u16) -> Self {
        self.config.data_endpoint_size = size;
        self
    }

    #[inline(always)]
    pub fn notify_endpoint_size(mut self, size: u16) -> Self {
        self.config.notify_endpoint_size = size;
        self
    }

    #[inline(always)]
    pub fn line_coding(mut self, lc: LineCoding) -> Self {
        self.line_coding = lc;
        self
    }

    pub fn build_descriptor(&self) -> [u8; CDC_ACM_DESCRIPTOR_LEN] {
        let mut desc = [0u8; CDC_ACM_DESCRIPTOR_LEN];
        let mut offset = 0;

        let total_len = CDC_ACM_DESCRIPTOR_LEN as u16;
        let num_interfaces: u8 = 2;
        let config_val: u8 = 1;
        let mut bm_attrs = 0x80u8;
        if self.config.self_powered {
            bm_attrs |= 0x40;
        }
        if self.config.remote_wakeup {
            bm_attrs |= 0x20;
        }
        let max_power = self.config.max_power_ma / 2;

        desc[offset..offset + 9].copy_from_slice(&[
            0x09,
            0x02,
            (total_len & 0xFF) as u8,
            ((total_len >> 8) & 0xFF) as u8,
            num_interfaces,
            config_val,
            0x00,
            bm_attrs,
            max_power,
        ]);
        offset += 9;

        let iad_first_if = 0u8;
        let iad_if_count = 2u8;
        let iad_func_class = 0x02u8;
        let iad_func_subclass = 0x02u8;
        let iad_func_protocol = 0x01u8;

        desc[offset..offset + 8].copy_from_slice(&[
            0x08,
            0x0B,
            iad_first_if,
            iad_if_count,
            iad_func_class,
            iad_func_subclass,
            iad_func_protocol,
            0x00,
        ]);
        offset += 8;

        let cci_if_num = 0u8;
        let cci_num_eps = 1u8;
        let cci_class = 0x02u8;
        let cci_subclass = 0x02u8;
        let cci_protocol = 0x00u8;

        desc[offset..offset + 9].copy_from_slice(&[
            0x09,
            0x04,
            cci_if_num,
            0x00,
            cci_num_eps,
            cci_class,
            cci_subclass,
            cci_protocol,
            0x00,
        ]);
        offset += 9;

        desc[offset..offset + 5].copy_from_slice(&[
            0x05,
            0x24,
            0x00,
            0x10,
            0x01,
        ]);
        offset += 5;

        let acm_capabilities = 0x02u8;

        desc[offset..offset + 4].copy_from_slice(&[
            0x04,
            0x24,
            0x02,
            acm_capabilities,
        ]);
        offset += 4;

        desc[offset..offset + 5].copy_from_slice(&[
            0x05,
            0x24,
            0x06,
            cci_if_num,
            1u8,
        ]);
        offset += 5;

        let call_mgmt_cap = 0x00u8;

        desc[offset..offset + 5].copy_from_slice(&[
            0x05,
            0x24,
            0x01,
            call_mgmt_cap,
            1u8,
        ]);
        offset += 5;

        let notify_ep_addr = 0x81u8;
        let notify_ep_type = 0x03u8;
        let notify_interval = 0x10u8;

        desc[offset..offset + 7].copy_from_slice(&[
            0x07,
            0x05,
            notify_ep_addr,
            notify_ep_type,
            (self.config.notify_endpoint_size & 0xFF) as u8,
            ((self.config.notify_endpoint_size >> 8) & 0xFF) as u8,
            notify_interval,
        ]);
        offset += 7;

        let dci_if_num = 1u8;
        let dci_num_eps = 2u8;
        let dci_class = 0x0Au8;
        let dci_subclass = 0x00u8;
        let dci_protocol = 0x00u8;

        desc[offset..offset + 9].copy_from_slice(&[
            0x09,
            0x04,
            dci_if_num,
            0x00,
            dci_num_eps,
            dci_class,
            dci_subclass,
            dci_protocol,
            0x00,
        ]);
        offset += 9;

        let bulk_in_ep_addr = 0x82u8;
        let bulk_ep_type = 0x02u8;

        desc[offset..offset + 7].copy_from_slice(&[
            0x07,
            0x05,
            bulk_in_ep_addr,
            bulk_ep_type,
            (self.config.data_endpoint_size & 0xFF) as u8,
            ((self.config.data_endpoint_size >> 8) & 0xFF) as u8,
            0x00,
        ]);
        offset += 7;

        let bulk_out_ep_addr = 0x03u8;

        desc[offset..offset + 7].copy_from_slice(&[
            0x07,
            0x05,
            bulk_out_ep_addr,
            bulk_ep_type,
            (self.config.data_endpoint_size & 0xFF) as u8,
            ((self.config.data_endpoint_size >> 8) & 0xFF) as u8,
            0x00,
        ]);

        desc
    }

    #[inline(always)]
    pub fn get_line_coding(&self) -> LineCoding {
        self.line_coding
    }

    #[inline(always)]
    pub fn set_line_coding(&mut self, lc: LineCoding) {
        self.line_coding = lc;
    }

    #[inline(always)]
    pub fn get_control_line_state(&self) -> ControlLineState {
        self.control_line_state
    }

    #[inline(always)]
    pub fn set_control_line_state(&mut self, state: ControlLineState) {
        self.control_line_state = state;
    }

    #[inline(always)]
    pub fn get_serial_state(&self) -> SerialState {
        self.serial_state
    }

    #[inline(always)]
    pub fn set_serial_state(&mut self, state: SerialState) {
        self.serial_state = state;
    }

    pub fn serial_state_notification_packet(&self, state: SerialState) -> [u8; 10] {
        let mut packet = [0u8; 10];
        packet[0] = 0xA1;
        packet[1] = 0x20;
        packet[2] = 0x00;
        packet[3] = 0x00;
        packet[4] = 0x00;
        packet[5] = 0x00;
        packet[6] = 0x02;
        packet[7] = 0x00;
        packet[8] = (state.bits() & 0xFF) as u8;
        packet[9] = ((state.bits() >> 8) & 0xFF) as u8;
        packet
    }

    pub fn device_descriptor(&self) -> [u8; 18] {
        let mut desc = [0u8; 18];
        desc[0] = 18;
        desc[1] = 0x01;
        desc[2] = 0x00;
        desc[3] = 0x02;
        desc[4] = self.config.device_class;
        desc[5] = self.config.device_subclass;
        desc[6] = self.config.device_protocol;
        desc[7] = 64;
        desc[8] = (self.config.vid & 0xFF) as u8;
        desc[9] = ((self.config.vid >> 8) & 0xFF) as u8;
        desc[10] = (self.config.pid & 0xFF) as u8;
        desc[11] = ((self.config.pid >> 8) & 0xFF) as u8;
        desc[12] = 0x00;
        desc[13] = 0x01;
        desc[14] = 0x00;
        desc[15] = 0x00;
        desc[16] = 0x00;
        desc[17] = 0x00;
        desc
    }

    pub fn string_descriptor_index(&self, index: u8) -> Option<&'static str> {
        match index {
            0 => Some(""),
            1 => self.config.manufacturer,
            2 => self.config.product,
            3 => self.config.serial,
            _ => None,
        }
    }

    pub fn is_class_specific_request(_bm_request_type: u8, b_request: u8) -> bool {
        matches!(
            b_request,
            0x00 | 0x01 | 0x20 | 0x21 | 0x22 | 0x23
        )
    }

    pub fn handle_class_request(
        &mut self,
        _bm_request_type: u8,
        b_request: u8,
        w_value: u16,
        _w_index: u16,
        _w_length: u16,
        data: Option<&mut [u8]>,
    ) -> ClassRequestResult {
        match b_request {
            0x20 => {
                if let Some(buf) = data {
                    if buf.len() >= 7 {
                        let dw_dte_rate = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
                        let char_format = buf[4];
                        let parity_type = buf[5];
                        let data_bits = buf[6];
                        self.line_coding = LineCoding {
                            dw_dte_rate,
                            char_format,
                            parity_type,
                            data_bits,
                        };
                        ClassRequestResult::Ack
                    } else {
                        ClassRequestResult::Stall
                    }
                } else {
                    ClassRequestResult::Stall
                }
            }
            0x21 => {
                if let Some(buf) = data {
                    if buf.len() >= 7 {
                        buf[0..4].copy_from_slice(&self.line_coding.dw_dte_rate.to_le_bytes());
                        buf[4] = self.line_coding.char_format;
                        buf[5] = self.line_coding.parity_type;
                        buf[6] = self.line_coding.data_bits;
                        ClassRequestResult::SendData(7)
                    } else {
                        ClassRequestResult::Stall
                    }
                } else {
                    ClassRequestResult::Stall
                }
            }
            0x22 => {
                let dtr = (w_value & 0x01) != 0;
                let rts = (w_value & 0x02) != 0;
                self.control_line_state = ControlLineState { dtr, rts };
                ClassRequestResult::Ack
            }
            0x00 | 0x01 | 0x23 => {
                ClassRequestResult::Ack
            }
            _ => ClassRequestResult::Stall,
        }
    }
}

pub enum ClassRequestResult {
    Ack,
    Stall,
    SendData(usize),
    ReceiveData(usize),
}

#[derive(Debug, Clone)]
pub struct UsbInfo {
    periph_name: String,
    periph_mod: String,
    hal_mod: String,

    field_enable: String,
    field_usb_pullup: String,
    field_epinen: String,
    field_epouten: String,

    field_tasks_startein: String,
    field_tasks_staroutep: String,

    field_events_ep0setup: String,
    field_events_ep0datadone: String,
    field_events_endepin: String,
    field_events_endepout: String,
    field_events_usbreset: String,
    field_events_usbevent: String,
}

impl UsbInfo {
    pub fn render(&self) -> Result<String> {
        let mut s = String::new();
        let usb_ty = sanitize_type_name(&self.periph_mod);

        s.push_str(&format!("    pub mod {} {{\n", self.hal_mod));
        s.push_str("        use super::pac;\n\n");
        s.push_str(&format!(
            "        pub type {usb_ty} = pac::{}::RegisterBlock;\n\n",
            self.periph_mod,
        ));

        s.push_str("        use core::marker::PhantomData;\n\n");

        s.push_str("        pub trait UsbState {}\n");
        s.push_str("        pub struct Unconfigured;\n");
        s.push_str("        pub struct Disabled;\n");
        s.push_str("        pub struct Enabled;\n");
        s.push_str("        pub struct Addressed;\n");
        s.push_str("        pub struct Configured;\n");
        s.push_str("        pub struct Suspended;\n");
        s.push_str("\n");
        s.push_str("        impl UsbState for Unconfigured {}\n");
        s.push_str("        impl UsbState for Disabled {}\n");
        s.push_str("        impl UsbState for Enabled {}\n");
        s.push_str("        impl UsbState for Addressed {}\n");
        s.push_str("        impl UsbState for Configured {}\n");
        s.push_str("        impl UsbState for Suspended {}\n\n");

        s.push_str("        #[repr(u8)]\n");
        s.push_str("        #[derive(Copy, Clone, Debug, PartialEq, Eq)]\n");
        s.push_str("        pub enum LineState {\n");
        s.push_str("            NoSignal = 0,\n");
        s.push_str("            Dtr = 1,\n");
        s.push_str("            Rts = 2,\n");
        s.push_str("            DtrRts = 3,\n");
        s.push_str("        }\n\n");

        s.push_str("        #[repr(u8)]\n");
        s.push_str("        #[derive(Copy, Clone, Debug, PartialEq, Eq)]\n");
        s.push_str("        pub enum UsbClass {\n");
        s.push_str("            None = 0,\n");
        s.push_str("            Audio = 1,\n");
        s.push_str("            Cdc = 2,\n");
        s.push_str("            Hid = 3,\n");
        s.push_str("            MassStorage = 4,\n");
        s.push_str("            Wireless = 5,\n");
        s.push_str("            SmartCard = 6,\n");
        s.push_str("            ContentSecurity = 7,\n");
        s.push_str("            Video = 8,\n");
        s.push_str("            PersonalHealthcare = 9,\n");
        s.push_str("            AudioVideo = 10,\n");
        s.push_str("            Diagnostic = 11,\n");
        s.push_str("            CdcControl = 12,\n");
        s.push_str("            CdcData = 13,\n");
        s.push_str("            Irda = 16,\n");
        s.push_str("            Ethernet = 17,\n");
        s.push_str("            Hssh = 18,\n");
        s.push_str("            Sync = 19,\n");
        s.push_str("            Wdm = 20,\n");
        s.push_str("        }\n\n");

        s.push_str("        #[repr(u8)]\n");
        s.push_str("        #[derive(Copy, Clone, Debug, PartialEq, Eq)]\n");
        s.push_str("        pub enum UsbSubClass {\n");
        s.push_str("            None = 0,\n");
        s.push_str("            AtFcx = 1,\n");
        s.push_str("            AtCdcDirectLine = 2,\n");
        s.push_str("            EthernetNetworking = 6,\n");
        s.push_str("            WirelessHandset = 7,\n");
        s.push_str("            DeviceManagement = 8,\n");
        s.push_str("            MobileBroadband = 9,\n");
        s.push_str("            Mcap = 10,\n");
        s.push_str("            CdcData = 11,\n");
        s.push_str("            Audio = 12,\n");
        s.push_str("            CdcBridge = 13,\n");
        s.push_str("            CdcAcmodem = 14,\n");
        s.push_str("            VendorSpecific = 255,\n");
        s.push_str("        }\n\n");

        s.push_str("        #[repr(u8)]\n");
        s.push_str("        #[derive(Copy, Clone, Debug, PartialEq, Eq)]\n");
        s.push_str("        pub enum UsbProtocol {\n");
        s.push_str("            None = 0,\n");
        s.push_str("            V250 = 1,\n");
        s.push_str("            Q931 = 2,\n");
        s.push_str("            EuroIsdn = 3,\n");
        s.push_str("            Cmadtr = 4,\n");
        s.push_str("            HostBased = 5,\n");
        s.push_str("            Transparency = 6,\n");
        s.push_str("            UsbIf = 16,\n");
        s.push_str("            VendorSpecific = 255,\n");
        s.push_str("        }\n\n");

        s.push_str(&format!("        pub struct Usb<'a, S: UsbState> {{\n"));
        s.push_str(&format!("            usb: &'a {usb_ty},\n"));
        s.push_str("            _state: PhantomData<S>,\n");
        s.push_str("        }\n\n");

        s.push_str("        impl<'a, S: UsbState> Usb<'a, S> {\n");
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn is_enabled(&self) -> bool {\n");
        s.push_str(&format!(
            "                self.usb.{}.read() == 1\n",
            self.field_enable
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn is_ep0_setup(&self) -> bool {\n");
        s.push_str(&format!(
            "                self.usb.{}.read() != 0\n",
            self.field_events_ep0setup
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn is_ep0_data_done(&self) -> bool {\n");
        s.push_str(&format!(
            "                self.usb.{}.read() != 0\n",
            self.field_events_ep0datadone
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn is_usb_reset(&self) -> bool {\n");
        s.push_str(&format!(
            "                self.usb.{}.read() != 0\n",
            self.field_events_usbreset
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn is_usb_event(&self) -> bool {\n");
        s.push_str(&format!(
            "                self.usb.{}.read() != 0\n",
            self.field_events_usbevent
        ));
        s.push_str("            }\n");
        s.push_str("        }\n\n");

        s.push_str(&format!("        impl<'a> Usb<'a, Unconfigured> {{\n"));
        s.push_str("            #[inline(always)]\n");
        s.push_str(&format!(
            "            pub unsafe fn steal() -> Usb<'static, Unconfigured> {{\n"
        ));
        s.push_str(&format!(
            "                Usb {{ usb: &*pac::{}::PTR, _state: PhantomData }}\n",
            self.periph_mod
        ));
        s.push_str("            }\n\n");

        s.push_str(&format!(
            "            pub fn usb() -> Usb<'static, Unconfigured> {{\n"
        ));
        s.push_str(&format!(
            "                Usb {{ usb: unsafe {{ &*pac::{}::PTR }}, _state: PhantomData }}\n",
            self.periph_mod
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn enable(self) -> Usb<'a, Disabled> {\n");
        s.push_str(&format!(
            "                self.usb.{}.write(1);\n",
            self.field_enable
        ));
        s.push_str(&format!(
            "                Usb {{ usb: self.usb, _state: PhantomData }}\n"
        ));
        s.push_str("            }\n");
        s.push_str("        }\n\n");

        s.push_str(&format!("        impl<'a> Usb<'a, Disabled> {{\n"));
        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn connect_pullup(&self) {\n");
        s.push_str(&format!(
            "                self.usb.{}.write(1);\n",
            self.field_usb_pullup
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn disconnect_pullup(&self) {\n");
        s.push_str(&format!(
            "                self.usb.{}.write(0);\n",
            self.field_usb_pullup
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn enable_ep0(&self) {\n");
        s.push_str(&format!(
            "                let mut epinen = self.usb.{}.read();\n",
            self.field_epinen
        ));
        s.push_str("                epinen |= 1u32;\n");
        s.push_str(&format!(
            "                self.usb.{}.write(epinen);\n",
            self.field_epinen
        ));
        s.push_str(&format!(
            "                let mut epouten = self.usb.{}.read();\n",
            self.field_epouten
        ));
        s.push_str("                epouten |= 1u32;\n");
        s.push_str(&format!(
            "                self.usb.{}.write(epouten);\n",
            self.field_epouten
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str(
            "            pub fn enable_data_endpoints(&self, data_ep: u8, notify_ep: u8) {\n",
        );
        s.push_str("                if data_ep > 7 || notify_ep > 7 {\n");
        s.push_str("                    return;\n");
        s.push_str("                }\n");
        s.push_str(&format!(
            "                let mut epinen = self.usb.{}.read();\n",
            self.field_epinen
        ));
        s.push_str("                epinen |= (1u32 << data_ep) | (1u32 << notify_ep);\n");
        s.push_str(&format!(
            "                self.usb.{}.write(epinen);\n",
            self.field_epinen
        ));
        s.push_str(&format!(
            "                let mut epouten = self.usb.{}.read();\n",
            self.field_epouten
        ));
        s.push_str("                epouten |= 1u32 << data_ep;\n");
        s.push_str(&format!(
            "                self.usb.{}.write(epouten);\n",
            self.field_epouten
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn start_in_endpoint(&self, ep_num: usize) {\n");
        s.push_str("                if ep_num > 7 {\n");
        s.push_str("                    return;\n");
        s.push_str("                }\n");
        s.push_str(&format!(
            "                self.usb.tasks_startepin__s_[ep_num].write(1);\n",
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn start_out_endpoint(&self, ep_num: usize) {\n");
        s.push_str("                if ep_num > 7 {\n");
        s.push_str("                    return;\n");
        s.push_str("                }\n");
        s.push_str(&format!(
            "                self.usb.tasks_startepout__s_[ep_num].write(1);\n",
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn is_endpoint_in_ready(&self, ep_num: usize) -> bool {\n");
        s.push_str("                if ep_num > 7 {\n");
        s.push_str("                    return false;\n");
        s.push_str("                }\n");
        s.push_str(&format!(
            "                self.usb.{}[ep_num].read() != 0\n",
            self.field_events_endepin
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn is_endpoint_out_ready(&self, ep_num: usize) -> bool {\n");
        s.push_str("                if ep_num > 7 {\n");
        s.push_str("                    return false;\n");
        s.push_str("                }\n");
        s.push_str(&format!(
            "                self.usb.{}[ep_num].read() != 0\n",
            self.field_events_endepout
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn clear_endpoint_in_ready(&self, ep_num: usize) {\n");
        s.push_str("                if ep_num > 7 {\n");
        s.push_str("                    return;\n");
        s.push_str("                }\n");
        s.push_str(&format!(
            "                self.usb.{}[ep_num].write(0);\n",
            self.field_events_endepin
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn clear_endpoint_out_ready(&self, ep_num: usize) {\n");
        s.push_str("                if ep_num > 7 {\n");
        s.push_str("                    return;\n");
        s.push_str("                }\n");
        s.push_str(&format!(
            "                self.usb.{}[ep_num].write(0);\n",
            self.field_events_endepout
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn clear_ep0_setup(&self) {\n");
        s.push_str(&format!(
            "                self.usb.{}.write(0);\n",
            self.field_events_ep0setup
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn clear_ep0_data_done(&self) {\n");
        s.push_str(&format!(
            "                self.usb.{}.write(0);\n",
            self.field_events_ep0datadone
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn clear_usb_reset(&self) {\n");
        s.push_str(&format!(
            "                self.usb.{}.write(0);\n",
            self.field_events_usbreset
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn clear_usb_event(&self) {\n");
        s.push_str(&format!(
            "                self.usb.{}.write(0);\n",
            self.field_events_usbevent
        ));
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn ep0_write_data(&self, ptr: *mut u8, maxcnt: usize) {\n");
        s.push_str("                if maxcnt > 64 {\n");
        s.push_str("                    return;\n");
        s.push_str("                }\n");
        s.push_str("                unsafe {\n");
        s.push_str("                    let ep0 = &*(self.usb.epin__s_.as_ptr().add(0 * 20).cast::<pac::Epin>());\n");
        s.push_str("                    ep0.ptr.write(ptr as u32);\n");
        s.push_str("                    ep0.maxcnt.write(maxcnt as u32);\n");
        s.push_str("                }\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn ep0_read_data(&self, ptr: *mut u8, maxcnt: usize) {\n");
        s.push_str("                if maxcnt > 64 {\n");
        s.push_str("                    return;\n");
        s.push_str("                }\n");
        s.push_str("                unsafe {\n");
        s.push_str("                    let ep0 = &*(self.usb.epout__s_.as_ptr().add(0 * 20).cast::<pac::Epout>());\n");
        s.push_str("                    ep0.ptr.write(ptr as u32);\n");
        s.push_str("                    ep0.maxcnt.write(maxcnt as u32);\n");
        s.push_str("                }\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn ep0_get_read_count(&self) -> u32 {\n");
        s.push_str("                unsafe { (&*(self.usb.epout__s_.as_ptr().add(0 * 20).cast::<pac::Epout>())).amount.read() }\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn ep0_get_write_count(&self) -> u32 {\n");
        s.push_str("                unsafe { (&*(self.usb.epin__s_.as_ptr().add(0 * 20).cast::<pac::Epin>())).amount.read() }\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn stall_ep0(&self) {\n");
        s.push_str("                self.usb.tasks_ep0stall.write(1);\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn ep0_receive_out(&self) {\n");
        s.push_str("                self.usb.tasks_ep0rcvout.write(1);\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn ep0_send_status(&self) {\n");
        s.push_str("                self.usb.tasks_ep0status.write(1);\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn get_setup_packet(&self, bm_request_type: &mut u8, b_request: &mut u8, w_value: &mut u16, w_index: &mut u16, w_length: &mut u16) {\n");
        s.push_str("                let wvalueh = self.usb.wvalueh.read();\n");
        s.push_str("                let wvaluel = self.usb.wvaluel.read();\n");
        s.push_str("                let windexh = self.usb.windexh.read();\n");
        s.push_str("                let windexl = self.usb.windexl.read();\n");
        s.push_str("                let wlengthh = self.usb.wlengthh.read();\n");
        s.push_str("                let wlengthl = self.usb.wlengthl.read();\n\n");
        s.push_str("                *bm_request_type = self.usb.bmrequesttype.read() as u8;\n");
        s.push_str("                *b_request = self.usb.brequest.read() as u8;\n");
        s.push_str("                *w_value = ((wvalueh as u16) << 8) | (wvaluel as u16);\n");
        s.push_str("                *w_index = ((windexh as u16) << 8) | (windexl as u16);\n");
        s.push_str("                *w_length = ((wlengthh as u16) << 8) | (wlengthl as u16);\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn is_get_descriptor_request(&self, bm_request_type: u8, b_request: u8, w_value: u16) -> bool {\n");
        s.push_str("                b_request == 0x06 && (bm_request_type & 0x80) != 0 && (w_value >> 8) == 0x03\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn get_descriptor_type(&self, w_value: u16) -> u8 {\n");
        s.push_str("                (w_value >> 8) as u8\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn get_descriptor_index(&self, w_value: u16) -> u8 {\n");
        s.push_str("                (w_value & 0xFF) as u8\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn is_set_configuration_request(&self, bm_request_type: u8, b_request: u8) -> bool {\n");
        s.push_str("                bm_request_type == 0x00 && b_request == 0x09\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn is_get_configuration_request(&self, bm_request_type: u8, b_request: u8) -> bool {\n");
        s.push_str("                bm_request_type == 0x80 && b_request == 0x08\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn is_set_interface_request(&self, bm_request_type: u8, b_request: u8) -> bool {\n");
        s.push_str("                bm_request_type == 0x01 && b_request == 0x0B\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn is_get_interface_request(&self, bm_request_type: u8, b_request: u8) -> bool {\n");
        s.push_str("                bm_request_type == 0x81 && b_request == 0x0A\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn is_cdc_class_request(&self, bm_request_type: u8, b_request: u8) -> bool {\n");
        s.push_str("                let _ = bm_request_type;\n");
        s.push_str("                matches!(b_request, 0x20 | 0x21 | 0x22 | 0x23)\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn write_data_to_endpoint(&self, ep_num: usize, ptr: *mut u8, maxcnt: usize) {\n");
        s.push_str("                if ep_num > 7 || maxcnt > 64 {\n");
        s.push_str("                    return;\n");
        s.push_str("                }\n");
        s.push_str("                unsafe {\n");
        s.push_str("                    let ep = &*(self.usb.epin__s_.as_ptr().add(ep_num as isize * 20).cast::<pac::Epin>());\n");
        s.push_str("                    ep.ptr.write(ptr as u32);\n");
        s.push_str("                    ep.maxcnt.write(maxcnt as u32);\n");
        s.push_str("                }\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn read_data_from_endpoint(&self, ep_num: usize, ptr: *mut u8, maxcnt: usize) {\n");
        s.push_str("                if ep_num > 7 || maxcnt > 64 {\n");
        s.push_str("                    return;\n");
        s.push_str("                }\n");
        s.push_str("                unsafe {\n");
        s.push_str("                    let ep = &*(self.usb.epout__s_.as_ptr().add(ep_num as isize * 20).cast::<pac::Epout>());\n");
        s.push_str("                    ep.ptr.write(ptr as u32);\n");
        s.push_str("                    ep.maxcnt.write(maxcnt as u32);\n");
        s.push_str("                }\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn get_endpoint_in_amount(&self, ep_num: usize) -> u32 {\n");
        s.push_str("                if ep_num > 7 {\n");
        s.push_str("                    return 0;\n");
        s.push_str("                }\n");
        s.push_str("                unsafe {\n");
        s.push_str("                    (&*(self.usb.epin__s_.as_ptr().add(ep_num as isize * 20).cast::<pac::Epin>())).amount.read()\n");
        s.push_str("                }\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn get_endpoint_out_amount(&self, ep_num: usize) -> u32 {\n");
        s.push_str("                if ep_num > 7 {\n");
        s.push_str("                    return 0;\n");
        s.push_str("                }\n");
        s.push_str("                unsafe {\n");
        s.push_str("                    (&*(self.usb.epout__s_.as_ptr().add(ep_num as isize * 20).cast::<pac::Epout>())).amount.read()\n");
        s.push_str("                }\n");
        s.push_str("            }\n");
        s.push_str("        }\n");

        s.push_str("    }\n");
        Ok(s)
    }
}

pub fn collect_usb_devices(device: &svd::Device) -> Vec<UsbInfo> {
    let mut out = Vec::new();
    for p in &device.peripherals {
        if !is_usbd_like(&p.name) {
            continue;
        }
        let items = gpio::peripheral_register_items(device, p);
        if items.is_empty() {
            continue;
        }

        let Some((enable_name, _)) = gpio::find_register(items, "ENABLE") else {
            continue;
        };
        let Some((usb_pullup_name, _)) = gpio::find_register(items, "USBPULLUP") else {
            continue;
        };
        let Some((epinen_name, _)) = gpio::find_register(items, "EPINEN") else {
            continue;
        };
        let Some((epouten_name, _)) = gpio::find_register(items, "EPOUTEN") else {
            continue;
        };
        let Some((tasks_startein_name, _)) = gpio::find_register(items, "TASKS_STARTEPIN") else {
            continue;
        };
        let Some((tasks_staroutep_name, _)) = gpio::find_register(items, "TASKS_STARTEPOUT") else {
            continue;
        };
        let Some((events_ep0setup_name, _)) = gpio::find_register(items, "EVENTS_EP0SETUP") else {
            continue;
        };
        let Some((events_ep0datadone_name, _)) = gpio::find_register(items, "EVENTS_EP0DATADONE")
        else {
            continue;
        };
        let Some((events_endepin_name, _)) = gpio::find_register(items, "EVENTS_ENDEPIN") else {
            continue;
        };
        let Some((events_endepout_name, _)) = gpio::find_register(items, "EVENTS_ENDEPOUT") else {
            continue;
        };
        let Some((events_usbreset_name, _)) = gpio::find_register(items, "EVENTS_USBRESET") else {
            continue;
        };
        let Some((events_usbevent_name, _)) = gpio::find_register(items, "EVENTS_USBEVENT") else {
            continue;
        };

        out.push(UsbInfo {
            periph_name: p.name.clone(),
            periph_mod: gpio::sanitize_module_name(&p.name),
            hal_mod: gpio::sanitize_field_name(&p.name),

            field_enable: gpio::sanitize_field_name(&enable_name),
            field_usb_pullup: gpio::sanitize_field_name(&usb_pullup_name),
            field_epinen: gpio::sanitize_field_name(&epinen_name),
            field_epouten: gpio::sanitize_field_name(&epouten_name),

            field_tasks_startein: gpio::sanitize_field_name(&tasks_startein_name),
            field_tasks_staroutep: gpio::sanitize_field_name(&tasks_staroutep_name),

            field_events_ep0setup: gpio::sanitize_field_name(&events_ep0setup_name),
            field_events_ep0datadone: gpio::sanitize_field_name(&events_ep0datadone_name),
            field_events_endepin: gpio::sanitize_field_name(&events_endepin_name),
            field_events_endepout: gpio::sanitize_field_name(&events_endepout_name),
            field_events_usbreset: gpio::sanitize_field_name(&events_usbreset_name),
            field_events_usbevent: gpio::sanitize_field_name(&events_usbevent_name),
        });
    }
    out
}

fn is_usbd_like(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    upper.starts_with("USBD")
}

fn sanitize_type_name(s: &str) -> String {
    gpio::sanitize_type_name(s)
}
