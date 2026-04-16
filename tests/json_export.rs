#[cfg(feature = "json")]
mod json_export {
    use svdkit::{svd, device_to_json_pretty};

    #[test]
    fn device_to_json_contains_name() {
        let dev = svd::Device {
            schema_version: svd::SchemaVersion("1.3.9".to_string()),
            vendor: None,
            vendor_id: None,
            name: "JSON_GOLDEN".to_string(),
            series: None,
            version: "1".to_string(),
            description: "desc".to_string(),
            license_text: None,
            header_system_filename: None,
            header_definitions_prefix: None,
            address_unit_bits: 8,
            width: 32,
            default_register_properties: svd::RegisterProperties::default(),
            cpu: None,
            peripherals: vec![],
            vendor_extensions: None,
        };

        let s = device_to_json_pretty(&dev).expect("json");
        assert!(s.contains("\"name\": \"JSON_GOLDEN\""));
    }
}

