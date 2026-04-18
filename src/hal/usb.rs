use super::gpio;
use crate::{Result, svd};

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
        s.push_str("                    let ep0 = &*(self.usb.epin__s_.as_ptr().add(0 * 20).cast::<pac::EpinS>());\n");
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
        s.push_str("                    let ep0 = &*(self.usb.epout__s_.as_ptr().add(0 * 20).cast::<pac::EpoutS>());\n");
        s.push_str("                    ep0.ptr.write(ptr as u32);\n");
        s.push_str("                    ep0.maxcnt.write(maxcnt as u32);\n");
        s.push_str("                }\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn ep0_get_read_count(&self) -> u32 {\n");
        s.push_str("                unsafe { (&*(self.usb.epout__s_.as_ptr().add(0 * 20).cast::<pac::EpoutS>())).amount.read() }\n");
        s.push_str("            }\n\n");

        s.push_str("            #[inline(always)]\n");
        s.push_str("            pub fn ep0_get_write_count(&self) -> u32 {\n");
        s.push_str("                unsafe { (&*(self.usb.epin__s_.as_ptr().add(0 * 20).cast::<pac::EpinS>())).amount.read() }\n");
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
