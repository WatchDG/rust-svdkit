#![allow(dead_code)]

use crate::Result;

pub const CDC_ACM_CONFIG_DESCRIPTOR_LEN: usize = 75;

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
            dw_dte_rate: 115200,
            char_format: 0,
            parity_type: 0,
            data_bits: 8,
        }
    }
}

impl LineCoding {
    pub fn new(baud: u32) -> Self {
        Self {
            dw_dte_rate: baud,
            ..Default::default()
        }
    }

    pub fn baud(mut self, baud: u32) -> Self {
        self.dw_dte_rate = baud;
        self
    }

    pub fn data_bits(mut self, bits: u8) -> Self {
        self.data_bits = bits;
        self
    }

    pub fn no_parity(self) -> Self {
        self.parity_type(0)
    }

    pub fn odd_parity(self) -> Self {
        self.parity_type(1)
    }

    pub fn even_parity(self) -> Self {
        self.parity_type(2)
    }

    pub fn parity_type(mut self, parity: u8) -> Self {
        self.parity_type = parity;
        self
    }

    pub fn stop_bits_1(self) -> Self {
        self.char_format(0)
    }

    pub fn stop_bits_1_5(self) -> Self {
        self.char_format(1)
    }

    pub fn stop_bits_2(self) -> Self {
        self.char_format(2)
    }

    pub fn char_format(mut self, format: u8) -> Self {
        self.char_format = format;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlLineState {
    pub dtr: bool,
    pub rts: bool,
}

impl Default for ControlLineState {
    fn default() -> Self {
        Self {
            dtr: false,
            rts: false,
        }
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

    pub fn rx_carrier(mut self) -> Self {
        self.bits |= 1 << 0;
        self
    }

    pub fn tx_carrier(mut self) -> Self {
        self.bits |= 1 << 1;
        self
    }

    pub fn break_detected(mut self) -> Self {
        self.bits |= 1 << 2;
        self
    }

    pub fn ring_signal(mut self) -> Self {
        self.bits |= 1 << 3;
        self
    }

    pub fn framing_error(mut self) -> Self {
        self.bits |= 1 << 4;
        self
    }

    pub fn parity_error(mut self) -> Self {
        self.bits |= 1 << 5;
        self
    }

    pub fn overrun(mut self) -> Self {
        self.bits |= 1 << 6;
        self
    }

    pub fn to_u16(self) -> u16 {
        self.bits
    }
}

pub struct CdcAcmBuilder {
    vid: u16,
    pid: u16,
    manufacturer: Option<&'static str>,
    product: Option<&'static str>,
    serial: Option<&'static str>,
    max_power_ma: u8,
    self_powered: bool,
    remote_wakeup: bool,
    use_iad: bool,
    data_endpoint_size: u16,
    notify_endpoint_size: u16,
    line_coding: LineCoding,
}

impl CdcAcmBuilder {
    pub fn new() -> Self {
        Self {
            vid: 0x2341,
            pid: 0x0042,
            manufacturer: Some("Vendor"),
            product: Some("USB CDC ACM Device"),
            serial: Some("0001"),
            max_power_ma: 100,
            self_powered: true,
            remote_wakeup: false,
            use_iad: false,
            data_endpoint_size: 64,
            notify_endpoint_size: 8,
            line_coding: LineCoding::default(),
        }
    }

    pub fn vid(mut self, vid: u16) -> Self {
        self.vid = vid;
        self
    }

    pub fn pid(mut self, pid: u16) -> Self {
        self.pid = pid;
        self
    }

    pub fn manufacturer(mut self, m: &'static str) -> Self {
        self.manufacturer = Some(m);
        self
    }

    pub fn product(mut self, p: &'static str) -> Self {
        self.product = Some(p);
        self
    }

    pub fn serial_number(mut self, s: &'static str) -> Self {
        self.serial = Some(s);
        self
    }

    pub fn max_power_ma(mut self, ma: u8) -> Self {
        self.max_power_ma = ma;
        self
    }

    pub fn self_powered(mut self, v: bool) -> Self {
        self.self_powered = v;
        self
    }

    pub fn remote_wakeup(mut self, v: bool) -> Self {
        self.remote_wakeup = v;
        self
    }

    pub fn use_iad(mut self) -> Self {
        self.use_iad = true;
        self
    }

    pub fn data_endpoint_size(mut self, size: u16) -> Self {
        self.data_endpoint_size = size;
        self
    }

    pub fn notify_endpoint_size(mut self, size: u16) -> Self {
        self.notify_endpoint_size = size;
        self
    }

    pub fn line_coding(mut self, lc: LineCoding) -> Self {
        self.line_coding = lc;
        self
    }

    pub fn build_config_descriptor(&self) -> [u8; CDC_ACM_CONFIG_DESCRIPTOR_LEN] {
        let mut desc = [0u8; CDC_ACM_CONFIG_DESCRIPTOR_LEN];
        let mut offset = 0;

        let total_len = CDC_ACM_CONFIG_DESCRIPTOR_LEN as u16;
        let bm_attrs = 0x80u8
            | if self.self_powered { 0x40 } else { 0 }
            | if self.remote_wakeup { 0x20 } else { 0 };
        let max_power = self.max_power_ma / 2;

        desc[offset..offset + 9].copy_from_slice(&[
            0x09,
            0x02,
            (total_len & 0xFF) as u8,
            ((total_len >> 8) & 0xFF) as u8,
            0x02,
            0x01,
            0x00,
            bm_attrs,
            max_power,
        ]);
        offset += 9;

        if self.use_iad {
            desc[offset..offset + 8]
                .copy_from_slice(&[0x08, 0x0B, 0x00, 0x02, 0x02, 0x02, 0x01, 0x00]);
            offset += 8;
        }

        desc[offset..offset + 9]
            .copy_from_slice(&[0x09, 0x04, 0x00, 0x00, 0x01, 0x02, 0x02, 0x00, 0x00]);
        offset += 9;

        desc[offset..offset + 5].copy_from_slice(&[0x05, 0x24, 0x00, 0x10, 0x01]);
        offset += 5;

        desc[offset..offset + 4].copy_from_slice(&[0x04, 0x24, 0x02, 0x02]);
        offset += 4;

        desc[offset..offset + 5].copy_from_slice(&[0x05, 0x24, 0x06, 0x00, 0x01]);
        offset += 5;

        desc[offset..offset + 5].copy_from_slice(&[0x05, 0x24, 0x01, 0x00, 0x01]);
        offset += 5;

        desc[offset..offset + 7].copy_from_slice(&[
            0x07,
            0x05,
            0x81,
            0x03,
            (self.notify_endpoint_size & 0xFF) as u8,
            ((self.notify_endpoint_size >> 8) & 0xFF) as u8,
            0x10,
        ]);
        offset += 7;

        desc[offset..offset + 9]
            .copy_from_slice(&[0x09, 0x04, 0x01, 0x00, 0x02, 0x0A, 0x00, 0x00, 0x00]);
        offset += 9;

        desc[offset..offset + 7].copy_from_slice(&[
            0x07,
            0x05,
            0x82,
            0x02,
            (self.data_endpoint_size & 0xFF) as u8,
            ((self.data_endpoint_size >> 8) & 0xFF) as u8,
            0x00,
        ]);
        offset += 7;

        desc[offset..offset + 7].copy_from_slice(&[
            0x07,
            0x05,
            0x03,
            0x02,
            (self.data_endpoint_size & 0xFF) as u8,
            ((self.data_endpoint_size >> 8) & 0xFF) as u8,
            0x00,
        ]);

        desc
    }

    pub fn build_device_descriptor(&self) -> [u8; 18] {
        let mut desc = [0u8; 18];
        desc[0] = 18;
        desc[1] = 0x01;
        desc[2] = 0x00;
        desc[3] = 0x02;
        desc[4] = if self.use_iad { 0xEF } else { 0x02 };
        desc[5] = if self.use_iad { 0x02 } else { 0x00 };
        desc[6] = if self.use_iad { 0x01 } else { 0x00 };
        desc[7] = 64;
        desc[8] = (self.vid & 0xFF) as u8;
        desc[9] = ((self.vid >> 8) & 0xFF) as u8;
        desc[10] = (self.pid & 0xFF) as u8;
        desc[11] = ((self.pid >> 8) & 0xFF) as u8;
        desc[12] = 0x00;
        desc[13] = 0x01;
        desc[14] = 0x00;
        desc[15] = 0x00;
        desc[16] = 0x00;
        desc[17] = 0x00;
        desc
    }

    pub fn string_index(&self, index: u8) -> Option<&'static str> {
        match index {
            0 => Some(""),
            1 => self.manufacturer,
            2 => self.product,
            3 => self.serial,
            _ => None,
        }
    }
}

impl Default for CdcAcmBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CdcAcmState {
    pub line_coding: LineCoding,
    pub control_line_state: ControlLineState,
    pub serial_state: SerialState,
}

impl Default for CdcAcmState {
    fn default() -> Self {
        Self {
            line_coding: LineCoding::default(),
            control_line_state: ControlLineState::default(),
            serial_state: SerialState::new(),
        }
    }
}

impl CdcAcmState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn serial_state_notification(&self) -> [u8; 10] {
        let mut packet = [0u8; 10];
        packet[0] = 0xA1;
        packet[1] = 0x20;
        packet[6] = 0x02;
        packet[8] = (self.serial_state.bits & 0xFF) as u8;
        packet[9] = ((self.serial_state.bits >> 8) & 0xFF) as u8;
        packet
    }

    pub fn handle_request(
        &mut self,
        b_request: u8,
        w_value: u16,
        data: Option<&mut [u8]>,
    ) -> RequestResult {
        match b_request {
            0x20 => {
                if let Some(buf) = data {
                    if buf.len() >= 7 {
                        self.line_coding = LineCoding {
                            dw_dte_rate: u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]),
                            char_format: buf[4],
                            parity_type: buf[5],
                            data_bits: buf[6],
                        };
                        RequestResult::Ack
                    } else {
                        RequestResult::Stall
                    }
                } else {
                    RequestResult::Stall
                }
            }
            0x21 => {
                if let Some(buf) = data {
                    if buf.len() >= 7 {
                        buf[0..4].copy_from_slice(&self.line_coding.dw_dte_rate.to_le_bytes());
                        buf[4] = self.line_coding.char_format;
                        buf[5] = self.line_coding.parity_type;
                        buf[6] = self.line_coding.data_bits;
                        RequestResult::SendData(7)
                    } else {
                        RequestResult::Stall
                    }
                } else {
                    RequestResult::Stall
                }
            }
            0x22 => {
                self.control_line_state = ControlLineState {
                    dtr: (w_value & 0x01) != 0,
                    rts: (w_value & 0x02) != 0,
                };
                RequestResult::Ack
            }
            0x00 | 0x01 | 0x23 => RequestResult::Ack,
            _ => RequestResult::Stall,
        }
    }
}

pub enum RequestResult {
    Ack,
    Stall,
    SendData(usize),
}

pub fn generate_cdc_acm_content(_device: &crate::svd::Device) -> crate::Result<String> {
    let mut s = String::new();
    s.push_str("//! USB CDC ACM helpers for embedded USB devices.\n\n");
    s.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
    s.push_str("pub struct LineCoding {\n");
    s.push_str("    pub dw_dte_rate: u32,\n");
    s.push_str("    pub char_format: u8,\n");
    s.push_str("    pub parity_type: u8,\n");
    s.push_str("    pub data_bits: u8,\n");
    s.push_str("}\n\n");
    s.push_str("impl Default for LineCoding {\n");
    s.push_str("    fn default() -> Self {\n");
    s.push_str("        Self {\n");
    s.push_str("            dw_dte_rate: 115200,\n");
    s.push_str("            char_format: 0,\n");
    s.push_str("            parity_type: 0,\n");
    s.push_str("            data_bits: 8,\n");
    s.push_str("        }\n");
    s.push_str("    }\n");
    s.push_str("}\n\n");
    s.push_str("impl LineCoding {\n");
    s.push_str("    pub fn new(baud: u32) -> Self {\n");
    s.push_str("        Self { dw_dte_rate: baud, ..Self::default() }\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn baud(mut self, baud: u32) -> Self {\n");
    s.push_str("        self.dw_dte_rate = baud;\n");
    s.push_str("        self\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn data_bits(mut self, bits: u8) -> Self {\n");
    s.push_str("        self.data_bits = bits;\n");
    s.push_str("        self\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn no_parity(self) -> Self {\n");
    s.push_str("        self.parity_type(0)\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn odd_parity(self) -> Self {\n");
    s.push_str("        self.parity_type(1)\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn even_parity(self) -> Self {\n");
    s.push_str("        self.parity_type(2)\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn parity_type(mut self, parity: u8) -> Self {\n");
    s.push_str("        self.parity_type = parity;\n");
    s.push_str("        self\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn stop_bits_1(self) -> Self {\n");
    s.push_str("        self.char_format(0)\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn stop_bits_2(self) -> Self {\n");
    s.push_str("        self.char_format(2)\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn char_format(mut self, format: u8) -> Self {\n");
    s.push_str("        self.char_format = format;\n");
    s.push_str("        self\n");
    s.push_str("    }\n");
    s.push_str("}\n\n");
    s.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
    s.push_str("pub struct ControlLineState {\n");
    s.push_str("    pub dtr: bool,\n");
    s.push_str("    pub rts: bool,\n");
    s.push_str("}\n\n");
    s.push_str("impl Default for ControlLineState {\n");
    s.push_str("    fn default() -> Self {\n");
    s.push_str("        Self { dtr: false, rts: false }\n");
    s.push_str("    }\n");
    s.push_str("}\n\n");
    s.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
    s.push_str("pub struct SerialState {\n");
    s.push_str("    bits: u16,\n");
    s.push_str("}\n\n");
    s.push_str("impl Default for SerialState {\n");
    s.push_str("    fn default() -> Self {\n");
    s.push_str("        Self { bits: 0 }\n");
    s.push_str("    }\n");
    s.push_str("}\n\n");
    s.push_str("impl SerialState {\n");
    s.push_str("    pub fn new() -> Self {\n");
    s.push_str("        Self { bits: 0 }\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn rx_carrier(mut self) -> Self {\n");
    s.push_str("        self.bits |= 1;\n");
    s.push_str("        self\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn tx_carrier(mut self) -> Self {\n");
    s.push_str("        self.bits |= 2;\n");
    s.push_str("        self\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn break_detected(mut self) -> Self {\n");
    s.push_str("        self.bits |= 4;\n");
    s.push_str("        self\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn framing_error(mut self) -> Self {\n");
    s.push_str("        self.bits |= 16;\n");
    s.push_str("        self\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn parity_error(mut self) -> Self {\n");
    s.push_str("        self.bits |= 32;\n");
    s.push_str("        self\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn overrun(mut self) -> Self {\n");
    s.push_str("        self.bits |= 64;\n");
    s.push_str("        self\n");
    s.push_str("    }\n");
    s.push_str("}\n\n");
    s.push_str("pub struct CdcAcmBuilder {\n");
    s.push_str("    vid: u16,\n");
    s.push_str("    pid: u16,\n");
    s.push_str("    manufacturer: Option<&'static str>,\n");
    s.push_str("    product: Option<&'static str>,\n");
    s.push_str("    serial: Option<&'static str>,\n");
    s.push_str("    max_power_ma: u8,\n");
    s.push_str("    self_powered: bool,\n");
    s.push_str("    use_iad: bool,\n");
    s.push_str("    data_endpoint_size: u16,\n");
    s.push_str("    notify_endpoint_size: u16,\n");
    s.push_str("    line_coding: LineCoding,\n");
    s.push_str("}\n\n");
    s.push_str("impl CdcAcmBuilder {\n");
    s.push_str("    pub fn new() -> Self {\n");
    s.push_str("        Self {\n");
    s.push_str("            vid: 0x2341,\n");
    s.push_str("            pid: 0x0042,\n");
    s.push_str("            manufacturer: Some(\"Vendor\"),\n");
    s.push_str("            product: Some(\"USB CDC ACM Device\"),\n");
    s.push_str("            serial: Some(\"0001\"),\n");
    s.push_str("            max_power_ma: 100,\n");
    s.push_str("            self_powered: true,\n");
    s.push_str("            use_iad: false,\n");
    s.push_str("            data_endpoint_size: 64,\n");
    s.push_str("            notify_endpoint_size: 8,\n");
    s.push_str("            line_coding: LineCoding::default(),\n");
    s.push_str("        }\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn vid(mut self, vid: u16) -> Self { self.vid = vid; self }\n");
    s.push_str("    pub fn pid(mut self, pid: u16) -> Self { self.pid = pid; self }\n");
    s.push_str("    pub fn manufacturer(mut self, m: &'static str) -> Self { self.manufacturer = Some(m); self }\n");
    s.push_str(
        "    pub fn product(mut self, p: &'static str) -> Self { self.product = Some(p); self }\n",
    );
    s.push_str("    pub fn serial_number(mut self, s: &'static str) -> Self { self.serial = Some(s); self }\n");
    s.push_str(
        "    pub fn max_power_ma(mut self, ma: u8) -> Self { self.max_power_ma = ma; self }\n",
    );
    s.push_str(
        "    pub fn self_powered(mut self, v: bool) -> Self { self.self_powered = v; self }\n",
    );
    s.push_str("    pub fn use_iad(mut self) -> Self { self.use_iad = true; self }\n");
    s.push_str("    pub fn data_endpoint_size(mut self, size: u16) -> Self { self.data_endpoint_size = size; self }\n");
    s.push_str("    pub fn notify_endpoint_size(mut self, size: u16) -> Self { self.notify_endpoint_size = size; self }\n");
    s.push_str("    pub fn line_coding(mut self, lc: LineCoding) -> Self { self.line_coding = lc; self }\n\n");
    s.push_str("    pub fn build_config_descriptor(&self) -> [u8; 75] {\n");
    s.push_str("        let mut desc = [0u8; 75];\n");
    s.push_str("        let mut offset = 0;\n");
    s.push_str("        desc[offset..offset + 9].copy_from_slice(&[0x09, 0x02, 0x4B, 0x00, 0x02, 0x01, 0x00, 0xC0 | if self.self_powered { 0x40 } else { 0 }, self.max_power_ma / 2]);\n");
    s.push_str("        offset += 9;\n");
    s.push_str("        if self.use_iad {\n");
    s.push_str("            desc[offset..offset + 8].copy_from_slice(&[0x08, 0x0B, 0x00, 0x02, 0x02, 0x02, 0x01, 0x00]);\n");
    s.push_str("            offset += 8;\n");
    s.push_str("        }\n");
    s.push_str("        desc[offset..offset + 9].copy_from_slice(&[0x09, 0x04, 0x00, 0x00, 0x01, 0x02, 0x02, 0x00, 0x00]);\n");
    s.push_str("        offset += 9;\n");
    s.push_str(
        "        desc[offset..offset + 5].copy_from_slice(&[0x05, 0x24, 0x00, 0x10, 0x01]);\n",
    );
    s.push_str("        offset += 5;\n");
    s.push_str("        desc[offset..offset + 4].copy_from_slice(&[0x04, 0x24, 0x02, 0x02]);\n");
    s.push_str("        offset += 4;\n");
    s.push_str(
        "        desc[offset..offset + 5].copy_from_slice(&[0x05, 0x24, 0x06, 0x00, 0x01]);\n",
    );
    s.push_str("        offset += 5;\n");
    s.push_str(
        "        desc[offset..offset + 5].copy_from_slice(&[0x05, 0x24, 0x01, 0x00, 0x01]);\n",
    );
    s.push_str("        offset += 5;\n");
    s.push_str("        desc[offset..offset + 7].copy_from_slice(&[0x07, 0x05, 0x81, 0x03, (self.notify_endpoint_size & 0xFF) as u8, (self.notify_endpoint_size >> 8) as u8, 0x10]);\n");
    s.push_str("        offset += 7;\n");
    s.push_str("        desc[offset..offset + 9].copy_from_slice(&[0x09, 0x04, 0x01, 0x00, 0x02, 0x0A, 0x00, 0x00, 0x00]);\n");
    s.push_str("        offset += 9;\n");
    s.push_str("        desc[offset..offset + 7].copy_from_slice(&[0x07, 0x05, 0x82, 0x02, (self.data_endpoint_size & 0xFF) as u8, (self.data_endpoint_size >> 8) as u8, 0x00]);\n");
    s.push_str("        offset += 7;\n");
    s.push_str("        desc[offset..offset + 7].copy_from_slice(&[0x07, 0x05, 0x03, 0x02, (self.data_endpoint_size & 0xFF) as u8, (self.data_endpoint_size >> 8) as u8, 0x00]);\n");
    s.push_str("        desc\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn build_device_descriptor(&self) -> [u8; 18] {\n");
    s.push_str("        let mut desc = [0u8; 18];\n");
    s.push_str("        desc[0] = 18; desc[1] = 0x01;\n");
    s.push_str("        desc[4] = if self.use_iad { 0xEF } else { 0x02 };\n");
    s.push_str("        desc[5] = if self.use_iad { 0x02 } else { 0x00 };\n");
    s.push_str("        desc[6] = if self.use_iad { 0x01 } else { 0x00 };\n");
    s.push_str("        desc[7] = 64;\n");
    s.push_str("        desc[8] = (self.vid & 0xFF) as u8; desc[9] = (self.vid >> 8) as u8;\n");
    s.push_str("        desc[10] = (self.pid & 0xFF) as u8; desc[11] = (self.pid >> 8) as u8;\n");
    s.push_str("        desc[13] = 0x01;\n");
    s.push_str("        desc\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn string_index(&self, index: u8) -> Option<&'static str> {\n");
    s.push_str("        match index {\n");
    s.push_str("            0 => Some(\"\"),\n");
    s.push_str("            1 => self.manufacturer,\n");
    s.push_str("            2 => self.product,\n");
    s.push_str("            3 => self.serial,\n");
    s.push_str("            _ => None,\n");
    s.push_str("        }\n");
    s.push_str("    }\n");
    s.push_str("}\n\n");
    s.push_str("impl Default for CdcAcmBuilder {\n");
    s.push_str("    fn default() -> Self {\n");
    s.push_str("        Self::new()\n");
    s.push_str("    }\n");
    s.push_str("}\n\n");
    s.push_str("pub struct CdcAcmState {\n");
    s.push_str("    pub line_coding: LineCoding,\n");
    s.push_str("    pub control_line_state: ControlLineState,\n");
    s.push_str("    pub serial_state: SerialState,\n");
    s.push_str("}\n\n");
    s.push_str("impl Default for CdcAcmState {\n");
    s.push_str("    fn default() -> Self {\n");
    s.push_str("        Self {\n");
    s.push_str("            line_coding: LineCoding::default(),\n");
    s.push_str("            control_line_state: ControlLineState::default(),\n");
    s.push_str("            serial_state: SerialState::new(),\n");
    s.push_str("        }\n");
    s.push_str("    }\n");
    s.push_str("}\n\n");
    s.push_str("impl CdcAcmState {\n");
    s.push_str("    pub fn new() -> Self {\n");
    s.push_str("        Self::default()\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn serial_state_notification(&self) -> [u8; 10] {\n");
    s.push_str("        let mut packet = [0u8; 10];\n");
    s.push_str("        packet[0] = 0xA1;\n");
    s.push_str("        packet[1] = 0x20;\n");
    s.push_str("        packet[6] = 0x02;\n");
    s.push_str("        packet[8] = self.serial_state.bits as u8;\n");
    s.push_str("        packet[9] = (self.serial_state.bits >> 8) as u8;\n");
    s.push_str("        packet\n");
    s.push_str("    }\n\n");
    s.push_str("    pub fn handle_request(&mut self, b_request: u8, w_value: u16, data: Option<&mut [u8]>) -> RequestResult {\n");
    s.push_str("        match b_request {\n");
    s.push_str("            0x20 => {\n");
    s.push_str("                if let Some(buf) = data {\n");
    s.push_str("                    if buf.len() >= 7 {\n");
    s.push_str("                        self.line_coding = LineCoding {\n");
    s.push_str("                            dw_dte_rate: u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]),\n");
    s.push_str("                            char_format: buf[4],\n");
    s.push_str("                            parity_type: buf[5],\n");
    s.push_str("                            data_bits: buf[6],\n");
    s.push_str("                        };\n");
    s.push_str("                        RequestResult::Ack\n");
    s.push_str("                    } else {\n");
    s.push_str("                        RequestResult::Stall\n");
    s.push_str("                    }\n");
    s.push_str("                } else {\n");
    s.push_str("                    RequestResult::Stall\n");
    s.push_str("                }\n");
    s.push_str("            }\n");
    s.push_str("            0x21 => {\n");
    s.push_str("                if let Some(buf) = data {\n");
    s.push_str("                    if buf.len() >= 7 {\n");
    s.push_str("                        buf[0..4].copy_from_slice(&self.line_coding.dw_dte_rate.to_le_bytes());\n");
    s.push_str("                        buf[4] = self.line_coding.char_format;\n");
    s.push_str("                        buf[5] = self.line_coding.parity_type;\n");
    s.push_str("                        buf[6] = self.line_coding.data_bits;\n");
    s.push_str("                        RequestResult::SendData(7)\n");
    s.push_str("                    } else {\n");
    s.push_str("                        RequestResult::Stall\n");
    s.push_str("                    }\n");
    s.push_str("                } else {\n");
    s.push_str("                    RequestResult::Stall\n");
    s.push_str("                }\n");
    s.push_str("            }\n");
    s.push_str("            0x22 => {\n");
    s.push_str("                self.control_line_state = ControlLineState {\n");
    s.push_str("                    dtr: (w_value & 0x01) != 0,\n");
    s.push_str("                    rts: (w_value & 0x02) != 0,\n");
    s.push_str("                };\n");
    s.push_str("                RequestResult::Ack\n");
    s.push_str("            }\n");
    s.push_str("            0x00 | 0x01 | 0x23 => RequestResult::Ack,\n");
    s.push_str("            _ => RequestResult::Stall,\n");
    s.push_str("        }\n");
    s.push_str("    }\n");
    s.push_str("}\n\n");
    s.push_str("pub enum RequestResult {\n");
    s.push_str("    Ack,\n");
    s.push_str("    Stall,\n");
    s.push_str("    SendData(usize),\n");
    s.push_str("}\n");
    Ok(s)
}
