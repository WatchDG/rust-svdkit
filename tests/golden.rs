use svdkit::{pac, svd};

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn device_gpio_enum() -> svd::Device {
    svd::Device {
        schema_version: svd::SchemaVersion("1.3.9".to_string()),
        vendor: None,
        vendor_id: None,
        name: "GOLDEN_GPIO_ENUM".to_string(),
        series: None,
        version: "1".to_string(),
        description: "golden test device".to_string(),
        license_text: None,
        header_system_filename: None,
        header_definitions_prefix: None,
        address_unit_bits: 8,
        width: 32,
        default_register_properties: svd::RegisterProperties {
            size: Some(32),
            access: Some(svd::AccessType::ReadWrite),
            protection: None,
            reset_value: Some(0),
            reset_mask: Some(0xFFFF_FFFF),
        },
        cpu: None,
        peripherals: vec![svd::Peripheral {
            derived_from: None,
            name: "GPIO".to_string(),
            version: None,
            description: Some("GPIO peripheral".to_string()),
            alternate_peripheral: None,
            group_name: None,
            prepend_to_name: None,
            append_to_name: None,
            header_struct_name: None,
            disable_condition: None,
            dim: None,
            base_address: 0x5000_0000,
            register_properties: svd::RegisterProperties::default(),
            address_block: vec![],
            interrupt: vec![],
            registers: Some(svd::RegisterBlock {
                items: vec![svd::RegisterBlockItem::Register {
                    register: svd::Register {
                        derived_from: None,
                        name: "OUT".to_string(),
                        display_name: None,
                        description: Some("Output register".to_string()),
                        address_offset: 0,
                        dim: None,
                        properties: svd::RegisterProperties {
                            size: Some(32),
                            access: Some(svd::AccessType::ReadWrite),
                            protection: None,
                            reset_value: Some(0),
                            reset_mask: Some(0xFFFF_FFFF),
                        },
                        alternate: None,
                        data_type: None,
                        modified_write_values: None,
                        write_constraint: None,
                        read_action: None,
                        field: vec![svd::Field {
                            derived_from: None,
                            dim: None,
                            name: "PIN0".to_string(),
                            description: Some("Pin 0".to_string()),
                            bit_range: svd::BitRange::BitOffsetWidth {
                                bit_offset: 0,
                                bit_width: Some(1),
                            },
                            access: None,
                            modified_write_values: None,
                            write_constraint: None,
                            read_action: None,
                            enumerated_values: vec![svd::EnumeratedValues {
                                derived_from: None,
                                name: None,
                                header_enum_name: Some("GpioPinValue".to_string()),
                                usage: None,
                                enumerated_value: vec![
                                    svd::EnumeratedValue {
                                        name: "Low".to_string(),
                                        description: None,
                                        spec: svd::EnumeratedValueSpec::Value {
                                            value: "0".to_string(),
                                        },
                                    },
                                    svd::EnumeratedValue {
                                        name: "High".to_string(),
                                        description: None,
                                        spec: svd::EnumeratedValueSpec::Value {
                                            value: "1".to_string(),
                                        },
                                    },
                                ],
                            }],
                        }],
                    },
                }],
            }),
        }],
        vendor_extensions: None,
    }
}

fn device_cluster_dim() -> svd::Device {
    svd::Device {
        schema_version: svd::SchemaVersion("1.3.9".to_string()),
        vendor: None,
        vendor_id: None,
        name: "GOLDEN_CLUSTER_DIM".to_string(),
        series: None,
        version: "1".to_string(),
        description: "golden test device".to_string(),
        license_text: None,
        header_system_filename: None,
        header_definitions_prefix: None,
        address_unit_bits: 8,
        width: 32,
        default_register_properties: svd::RegisterProperties {
            size: Some(32),
            access: Some(svd::AccessType::ReadWrite),
            protection: None,
            reset_value: Some(0),
            reset_mask: Some(0xFFFF_FFFF),
        },
        cpu: None,
        peripherals: vec![svd::Peripheral {
            derived_from: None,
            name: "TIM".to_string(),
            version: None,
            description: Some("Timer-like peripheral".to_string()),
            alternate_peripheral: None,
            group_name: None,
            prepend_to_name: None,
            append_to_name: None,
            header_struct_name: None,
            disable_condition: None,
            dim: None,
            base_address: 0x4000_0000,
            register_properties: svd::RegisterProperties::default(),
            address_block: vec![],
            interrupt: vec![svd::Interrupt {
                name: "TIM0".to_string(),
                description: None,
                value: 0,
            }],
            registers: Some(svd::RegisterBlock {
                items: vec![svd::RegisterBlockItem::Cluster {
                    cluster: svd::Cluster {
                        derived_from: None,
                        name: "CH".to_string(),
                        description: "Channel cluster".to_string(),
                        alternate_cluster: None,
                        header_struct_name: None,
                        address_offset: 0x100,
                        dim: Some(svd::DimElement {
                            dim: 2,
                            dim_increment: 0x20,
                            dim_index: None,
                            dim_name: None,
                            dim_array_index: None,
                        }),
                        register_properties: svd::RegisterProperties::default(),
                        items: vec![
                            svd::RegisterBlockItem::Register {
                                register: svd::Register {
                                    derived_from: None,
                                    name: "CCR".to_string(),
                                    display_name: None,
                                    description: Some("Capture/Compare".to_string()),
                                    address_offset: 0x00,
                                    dim: None,
                                    properties: svd::RegisterProperties {
                                        size: Some(32),
                                        access: Some(svd::AccessType::ReadWrite),
                                        protection: None,
                                        reset_value: Some(0),
                                        reset_mask: Some(0xFFFF_FFFF),
                                    },
                                    alternate: None,
                                    data_type: None,
                                    modified_write_values: None,
                                    write_constraint: None,
                                    read_action: None,
                                    field: vec![],
                                },
                            },
                            svd::RegisterBlockItem::Register {
                                register: svd::Register {
                                    derived_from: None,
                                    name: "CCER".to_string(),
                                    display_name: None,
                                    description: Some("Enable".to_string()),
                                    address_offset: 0x04,
                                    dim: None,
                                    properties: svd::RegisterProperties {
                                        size: Some(32),
                                        access: Some(svd::AccessType::ReadWrite),
                                        protection: None,
                                        reset_value: Some(0),
                                        reset_mask: Some(0xFFFF_FFFF),
                                    },
                                    alternate: None,
                                    data_type: None,
                                    modified_write_values: None,
                                    write_constraint: None,
                                    read_action: None,
                                    field: vec![],
                                },
                            },
                        ],
                    },
                }],
            }),
        }],
        vendor_extensions: None,
    }
}

#[test]
fn golden_gpio_enum_full_minimal() {
    const EXPECTED_FULL: u64 = 0x02DA90198057482F;
    const EXPECTED_MINIMAL: u64 = 0xAC80C1357C07126C;

    let dev = device_gpio_enum();
    let full =
        pac::generate_device_rs_with_options(&dev, pac::PacOptions::full()).expect("full gen");
    let minimal = pac::generate_device_rs_with_options(&dev, pac::PacOptions::minimal())
        .expect("minimal gen");

    assert!(full.contains("pub mod field_enums {"));
    assert!(!minimal.contains("pub mod field_enums {"));
    assert!(full.contains("fn field_pin0_raw"));
    assert!(!minimal.contains("fn field_pin0_raw"));

    let full_hash = fnv1a64(full.as_bytes());
    let minimal_hash = fnv1a64(minimal.as_bytes());

    assert_eq!(
        full_hash, EXPECTED_FULL,
        "update EXPECTED_FULL to 0x{full_hash:016X}"
    );
    assert_eq!(
        minimal_hash, EXPECTED_MINIMAL,
        "update EXPECTED_MINIMAL to 0x{minimal_hash:016X}"
    );
}

#[test]
fn golden_cluster_dim_full_minimal() {
    const EXPECTED_FULL: u64 = 0x388D2373388FF5DE;
    const EXPECTED_MINIMAL: u64 = 0x6FBD4BBEF43B98A1;

    let dev = device_cluster_dim();
    let full =
        pac::generate_device_rs_with_options(&dev, pac::PacOptions::full()).expect("full gen");
    let minimal = pac::generate_device_rs_with_options(&dev, pac::PacOptions::minimal())
        .expect("minimal gen");

    assert!(full.contains("pub enum Interrupt {"));
    assert!(full.contains("TIM0 = 0"));
    assert!(!minimal.contains("pub mod field_enums {"));

    let full_hash = fnv1a64(full.as_bytes());
    let minimal_hash = fnv1a64(minimal.as_bytes());

    assert_eq!(
        full_hash, EXPECTED_FULL,
        "update EXPECTED_FULL to 0x{full_hash:016X}"
    );
    assert_eq!(
        minimal_hash, EXPECTED_MINIMAL,
        "update EXPECTED_MINIMAL to 0x{minimal_hash:016X}"
    );
}
