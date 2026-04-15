//! Conversion `xml::Document` -> `svd::*` with strict validation against XSD (approx. 1.3.9).

use crate::error::{Error, Result};
use crate::{svd, xml};

pub fn device_from_document(doc: &xml::Document) -> Result<svd::Device> {
    let root = &doc.root;
    if root.name != "device" {
        return Err(Error::validation(
            root.loc,
            format!("root element must be <device>, got <{}>", root.name),
        ));
    }

    // Allow schemaVersion, xmlns/xsi, etc. (commonly present in real-world SVDs).
    for a in &root.attrs {
        let ok = matches!(
            a.name.as_str(),
            "schemaVersion"
                | "xmlns:xs"
                | "xmlns:xsi"
                // Some SVDs use it without the `xsi:` prefix.
                | "noNamespaceSchemaLocation"
                | "xsi:noNamespaceSchemaLocation"
                | "xs:noNamespaceSchemaLocation"
                | "xmlns"
        );
        if !ok {
            return Err(Error::validation(
                root.loc,
                format!("unexpected attribute on <device>: {}", a.name),
            ));
        }
    }

    let schema_version = root
        .attr("schemaVersion")
        .ok_or_else(|| Error::validation(root.loc, "<device> must have attribute schemaVersion"))?
        .trim()
        .to_string();
    if !schema_version
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.')
        || schema_version.is_empty()
    {
        return Err(Error::validation(
            root.loc,
            format!("invalid schemaVersion: {}", schema_version),
        ));
    }
    let schema_version = svd::SchemaVersion(schema_version);

    let mut vendor = None;
    let mut vendor_id = None;
    let mut name = None;
    let mut series = None;
    let mut version = None;
    let mut description = None;
    let mut license_text = None;
    let mut header_system_filename = None;
    let mut header_definitions_prefix = None;
    let mut address_unit_bits = None;
    let mut width = None;
    let mut default_props = svd::RegisterProperties::default();
    let mut cpu = None;
    let mut peripherals: Option<Vec<svd::Peripheral>> = None;
    let mut vendor_extensions: Option<Vec<svd::AnyXmlElement>> = None;

    for c in root.children_elements() {
        match c.name.as_str() {
            "vendor" => set_once_text(&mut vendor, c, "vendor")?,
            "vendorID" => set_once_text(&mut vendor_id, c, "vendorID")?,
            "name" => set_once_text(&mut name, c, "name")?,
            "series" => set_once_text(&mut series, c, "series")?,
            "version" => set_once_text(&mut version, c, "version")?,
            "description" => set_once_text(&mut description, c, "description")?,
            "licenseText" => set_once_text(&mut license_text, c, "licenseText")?,
            "headerSystemFilename" => {
                set_once_text(&mut header_system_filename, c, "headerSystemFilename")?
            }
            "headerDefinitionsPrefix" => {
                set_once_text(&mut header_definitions_prefix, c, "headerDefinitionsPrefix")?
            }
            "addressUnitBits" => set_once(
                &mut address_unit_bits,
                parse_scaled_u32_text(c, "addressUnitBits")?,
                c,
                "addressUnitBits",
            )?,
            "width" => set_once(&mut width, parse_scaled_u32_text(c, "width")?, c, "width")?,
            // registerPropertiesGroup
            "size" => default_props.size = Some(parse_scaled_u64_text(c, "size")?),
            "access" => default_props.access = Some(parse_access_text(c)?),
            "protection" => default_props.protection = Some(parse_protection_text(c)?),
            "resetValue" => {
                default_props.reset_value = Some(parse_scaled_u64_text(c, "resetValue")?)
            }
            "resetMask" => default_props.reset_mask = Some(parse_scaled_u64_text(c, "resetMask")?),
            "cpu" => set_once(&mut cpu, parse_cpu(c)?, c, "cpu")?,
            "peripherals" => set_once(&mut peripherals, parse_peripherals(c)?, c, "peripherals")?,
            "vendorExtensions" => set_once(
                &mut vendor_extensions,
                parse_vendor_extensions(c)?,
                c,
                "vendorExtensions",
            )?,
            _ => {
                return Err(Error::validation(
                    c.loc,
                    format!("unexpected element in <device>: <{}>", c.name),
                ));
            }
        }
    }

    let name = required(name, root, "name")?;
    let version = required(version, root, "version")?;
    let description = required(description, root, "description")?;
    let address_unit_bits = required(address_unit_bits, root, "addressUnitBits")?;
    let width = required(width, root, "width")?;
    let peripherals = required(peripherals, root, "peripherals")?;

    Ok(svd::Device {
        schema_version,
        vendor,
        vendor_id,
        name,
        series,
        version,
        description,
        license_text,
        header_system_filename,
        header_definitions_prefix,
        address_unit_bits,
        width,
        default_register_properties: default_props,
        cpu,
        peripherals,
        vendor_extensions,
    })
}

fn parse_peripherals(e: &xml::Element) -> Result<Vec<svd::Peripheral>> {
    assert_name(e, "peripherals")?;
    ensure_no_attrs(e, &[])?;
    let mut out = Vec::new();
    for c in e.children_elements() {
        if c.name != "peripheral" {
            return Err(Error::validation(
                c.loc,
                format!(
                    "in <peripherals> only <peripheral> is allowed, got <{}>",
                    c.name
                ),
            ));
        }
        out.push(parse_peripheral(c)?);
    }
    if out.is_empty() {
        return Err(Error::validation(
            e.loc,
            "<peripherals> must contain at least one <peripheral>",
        ));
    }
    Ok(out)
}

fn parse_cpu(e: &xml::Element) -> Result<svd::Cpu> {
    assert_name(e, "cpu")?;
    ensure_no_attrs(e, &[])?;

    let mut name = None;
    let mut revision = None;
    let mut endian = None;

    let mut mpu_present = None;
    let mut fpu_present = None;
    let mut fpu_dp = None;
    let mut dsp_present = None;
    let mut icache_present = None;
    let mut dcache_present = None;
    let mut itcm_present = None;
    let mut dtcm_present = None;
    let mut vtor_present = None;
    let mut nvic_prio_bits = None;
    let mut vendor_systick_config = None;
    let mut device_num_interrupts = None;
    let mut sau_num_regions = None;
    let mut sau_regions_config: Option<svd::SauRegionsConfig> = None;

    for c in e.children_elements() {
        match c.name.as_str() {
            "name" => set_once(&mut name, parse_cpu_name_text(c)?, c, "name")?,
            "revision" => set_once(&mut revision, parse_revision_text(c)?, c, "revision")?,
            "endian" => set_once(&mut endian, parse_endian_text(c)?, c, "endian")?,
            "mpuPresent" => set_once(
                &mut mpu_present,
                parse_bool_text(c, "mpuPresent")?,
                c,
                "mpuPresent",
            )?,
            "fpuPresent" => set_once(
                &mut fpu_present,
                parse_bool_text(c, "fpuPresent")?,
                c,
                "fpuPresent",
            )?,
            "fpuDP" => set_once(&mut fpu_dp, parse_bool_text(c, "fpuDP")?, c, "fpuDP")?,
            "dspPresent" => set_once(
                &mut dsp_present,
                parse_bool_text(c, "dspPresent")?,
                c,
                "dspPresent",
            )?,
            "icachePresent" => set_once(
                &mut icache_present,
                parse_bool_text(c, "icachePresent")?,
                c,
                "icachePresent",
            )?,
            "dcachePresent" => set_once(
                &mut dcache_present,
                parse_bool_text(c, "dcachePresent")?,
                c,
                "dcachePresent",
            )?,
            "itcmPresent" => set_once(
                &mut itcm_present,
                parse_bool_text(c, "itcmPresent")?,
                c,
                "itcmPresent",
            )?,
            "dtcmPresent" => set_once(
                &mut dtcm_present,
                parse_bool_text(c, "dtcmPresent")?,
                c,
                "dtcmPresent",
            )?,
            "vtorPresent" => set_once(
                &mut vtor_present,
                parse_bool_text(c, "vtorPresent")?,
                c,
                "vtorPresent",
            )?,
            "nvicPrioBits" => set_once(
                &mut nvic_prio_bits,
                parse_scaled_u32_text(c, "nvicPrioBits")?,
                c,
                "nvicPrioBits",
            )?,
            "vendorSystickConfig" => set_once(
                &mut vendor_systick_config,
                parse_bool_text(c, "vendorSystickConfig")?,
                c,
                "vendorSystickConfig",
            )?,
            "deviceNumInterrupts" => set_once(
                &mut device_num_interrupts,
                parse_scaled_u32_text(c, "deviceNumInterrupts")?,
                c,
                "deviceNumInterrupts",
            )?,
            // SAU extension (v1.3)
            "sauNumRegions" => set_once(
                &mut sau_num_regions,
                parse_scaled_u32_text(c, "sauNumRegions")?,
                c,
                "sauNumRegions",
            )?,
            "sauRegionsConfig" => set_once(
                &mut sau_regions_config,
                parse_sau_regions_config(c)?,
                c,
                "sauRegionsConfig",
            )?,
            _ => {
                return Err(Error::validation(
                    c.loc,
                    format!("unexpected element in <cpu>: <{}>", c.name),
                ));
            }
        }
    }

    Ok(svd::Cpu {
        name: required(name, e, "name")?,
        revision: required(revision, e, "revision")?,
        endian: required(endian, e, "endian")?,
        mpu_present,
        fpu_present,
        fpu_dp,
        dsp_present,
        icache_present,
        dcache_present,
        itcm_present,
        dtcm_present,
        vtor_present,
        nvic_prio_bits: required(nvic_prio_bits, e, "nvicPrioBits")?,
        vendor_systick_config: required(vendor_systick_config, e, "vendorSystickConfig")?,
        device_num_interrupts,
        sau_num_regions,
        sau_regions_config,
    })
}

fn parse_sau_regions_config(e: &xml::Element) -> Result<svd::SauRegionsConfig> {
    assert_name(e, "sauRegionsConfig")?;
    // XSD: attrs enabled (default=true), protectionWhenDisabled (default="s")
    ensure_only_attrs(e, &["enabled", "protectionWhenDisabled"])?;

    let enabled = e
        .attr("enabled")
        .map(|v| parse_bool_attr(e.loc, "enabled", v))
        .transpose()?
        .unwrap_or(true);
    let protection_when_disabled = e
        .attr("protectionWhenDisabled")
        .map(|v| parse_protection_attr(e.loc, "protectionWhenDisabled", v))
        .transpose()?
        .unwrap_or(svd::Protection::Secure);

    let mut regions = Vec::new();
    for c in e.children_elements() {
        if c.name != "region" {
            return Err(Error::validation(
                c.loc,
                format!(
                    "in <sauRegionsConfig> only <region> is allowed, got <{}>",
                    c.name
                ),
            ));
        }
        regions.push(parse_sau_region(c)?);
    }
    if regions.is_empty() {
        return Err(Error::validation(
            e.loc,
            "<sauRegionsConfig> must contain at least one <region>",
        ));
    }
    Ok(svd::SauRegionsConfig {
        enabled,
        protection_when_disabled,
        regions,
    })
}

fn parse_sau_region(e: &xml::Element) -> Result<svd::SauRegion> {
    assert_name(e, "region")?;
    // XSD: attrs enabled (default=true), name (optional)
    ensure_only_attrs(e, &["enabled", "name"])?;
    let enabled = e
        .attr("enabled")
        .map(|v| parse_bool_attr(e.loc, "enabled", v))
        .transpose()?
        .unwrap_or(true);
    let name = e.attr("name").map(|s| s.to_string());

    let mut base = None;
    let mut limit = None;
    let mut access = None;
    for c in e.children_elements() {
        match c.name.as_str() {
            "base" => set_once(&mut base, parse_scaled_u64_text(c, "base")?, c, "base")?,
            "limit" => set_once(&mut limit, parse_scaled_u64_text(c, "limit")?, c, "limit")?,
            "access" => set_once(&mut access, parse_sau_access_text(c)?, c, "access")?,
            _ => {
                return Err(Error::validation(
                    c.loc,
                    format!("unexpected element in <region>: <{}>", c.name),
                ));
            }
        }
    }
    Ok(svd::SauRegion {
        enabled,
        name,
        base: required(base, e, "base")?,
        limit: required(limit, e, "limit")?,
        access: required(access, e, "access")?,
    })
}

fn parse_peripheral(e: &xml::Element) -> Result<svd::Peripheral> {
    assert_name(e, "peripheral")?;
    let derived_from = e.attr("derivedFrom").map(|s| s.to_string());
    ensure_only_attrs(e, &["derivedFrom"])?;

    let mut name = None;
    let mut version = None;
    let mut description = None;
    let mut alternate_peripheral = None;
    let mut group_name = None;
    let mut prepend_to_name = None;
    let mut append_to_name = None;
    let mut header_struct_name = None;
    let mut disable_condition = None;
    let mut dim: Option<svd::DimElement> = None;
    let mut base_address = None;
    let mut props = svd::RegisterProperties::default();
    let mut address_block = Vec::new();
    let mut interrupt = Vec::new();
    let mut registers: Option<svd::RegisterBlock> = None;

    for c in e.children_elements() {
        match c.name.as_str() {
            "name" => set_once_text(&mut name, c, "name")?,
            "version" => set_once_text(&mut version, c, "version")?,
            "description" => set_once_text(&mut description, c, "description")?,
            "alternatePeripheral" => {
                set_once_text(&mut alternate_peripheral, c, "alternatePeripheral")?
            }
            "groupName" => set_once_text(&mut group_name, c, "groupName")?,
            "prependToName" => set_once_text(&mut prepend_to_name, c, "prependToName")?,
            "appendToName" => set_once_text(&mut append_to_name, c, "appendToName")?,
            "headerStructName" => set_once_text(&mut header_struct_name, c, "headerStructName")?,
            "disableCondition" => set_once_text(&mut disable_condition, c, "disableCondition")?,
            // dimElementGroup
            "dim" | "dimIncrement" | "dimIndex" | "dimName" | "dimArrayIndex" => {
                dim = Some(parse_dim_group(e)?);
            }
            "baseAddress" => set_once(
                &mut base_address,
                parse_scaled_u64_text(c, "baseAddress")?,
                c,
                "baseAddress",
            )?,
            // registerPropertiesGroup
            "size" => props.size = Some(parse_scaled_u64_text(c, "size")?),
            "access" => props.access = Some(parse_access_text(c)?),
            "protection" => props.protection = Some(parse_protection_text(c)?),
            "resetValue" => props.reset_value = Some(parse_scaled_u64_text(c, "resetValue")?),
            "resetMask" => props.reset_mask = Some(parse_scaled_u64_text(c, "resetMask")?),
            "addressBlock" => address_block.push(parse_address_block(c)?),
            "interrupt" => interrupt.push(parse_interrupt(c)?),
            "registers" => set_once(&mut registers, parse_registers(c)?, c, "registers")?,
            _ => {
                return Err(Error::validation(
                    c.loc,
                    format!("unexpected element in <peripheral>: <{}>", c.name),
                ));
            }
        }
    }

    Ok(svd::Peripheral {
        derived_from,
        name: required(name, e, "name")?,
        version,
        description,
        alternate_peripheral,
        group_name,
        prepend_to_name,
        append_to_name,
        header_struct_name,
        disable_condition,
        dim,
        base_address: required(base_address, e, "baseAddress")?,
        register_properties: props,
        address_block,
        interrupt,
        registers,
    })
}

fn parse_address_block(e: &xml::Element) -> Result<svd::AddressBlock> {
    assert_name(e, "addressBlock")?;
    ensure_no_attrs(e, &[])?;
    let mut offset = None;
    let mut size = None;
    let mut usage = None;
    let mut protection = None;
    for c in e.children_elements() {
        match c.name.as_str() {
            "offset" => set_once(
                &mut offset,
                parse_scaled_u64_text(c, "offset")?,
                c,
                "offset",
            )?,
            "size" => set_once(&mut size, parse_scaled_u64_text(c, "size")?, c, "size")?,
            "usage" => set_once(&mut usage, parse_addr_usage_text(c)?, c, "usage")?,
            "protection" => set_once(&mut protection, parse_protection_text(c)?, c, "protection")?,
            _ => {
                return Err(Error::validation(
                    c.loc,
                    format!("unexpected element in <addressBlock>: <{}>", c.name),
                ));
            }
        }
    }
    Ok(svd::AddressBlock {
        offset: required(offset, e, "offset")?,
        size: required(size, e, "size")?,
        usage: required(usage, e, "usage")?,
        protection,
    })
}

fn parse_interrupt(e: &xml::Element) -> Result<svd::Interrupt> {
    assert_name(e, "interrupt")?;
    ensure_no_attrs(e, &[])?;
    let mut name = None;
    let mut description = None;
    let mut value = None;
    for c in e.children_elements() {
        match c.name.as_str() {
            "name" => set_once_text(&mut name, c, "name")?,
            "description" => set_once_text(&mut description, c, "description")?,
            "value" => set_once(&mut value, parse_i64_text(c, "value")?, c, "value")?,
            _ => {
                return Err(Error::validation(
                    c.loc,
                    format!("unexpected element in <interrupt>: <{}>", c.name),
                ));
            }
        }
    }
    Ok(svd::Interrupt {
        name: required(name, e, "name")?,
        description,
        value: required(value, e, "value")?,
    })
}

fn parse_registers(e: &xml::Element) -> Result<svd::RegisterBlock> {
    assert_name(e, "registers")?;
    ensure_no_attrs(e, &[])?;
    let mut items = Vec::new();
    for c in e.children_elements() {
        match c.name.as_str() {
            "cluster" => items.push(svd::RegisterBlockItem::Cluster {
                cluster: parse_cluster(c)?,
            }),
            "register" => items.push(svd::RegisterBlockItem::Register {
                register: parse_register(c)?,
            }),
            _ => {
                return Err(Error::validation(
                    c.loc,
                    format!("unexpected element in <registers>: <{}>", c.name),
                ));
            }
        }
    }
    if items.is_empty() {
        return Err(Error::validation(
            e.loc,
            "<registers> must contain at least one <cluster> or <register>",
        ));
    }
    Ok(svd::RegisterBlock { items })
}

fn parse_cluster(e: &xml::Element) -> Result<svd::Cluster> {
    assert_name(e, "cluster")?;
    let derived_from = e.attr("derivedFrom").map(|s| s.to_string());
    ensure_only_attrs(e, &["derivedFrom"])?;

    let mut name = None;
    let mut description = None;
    let mut alternate_cluster = None;
    let mut header_struct_name = None;
    let mut address_offset = None;
    let mut dim: Option<svd::DimElement> = None;
    let mut props = svd::RegisterProperties::default();
    let mut items: Vec<svd::RegisterBlockItem> = Vec::new();

    for c in e.children_elements() {
        match c.name.as_str() {
            "name" => set_once_text(&mut name, c, "name")?,
            "description" => set_once_text(&mut description, c, "description")?,
            "alternateCluster" => set_once_text(&mut alternate_cluster, c, "alternateCluster")?,
            "headerStructName" => set_once_text(&mut header_struct_name, c, "headerStructName")?,
            "addressOffset" => set_once(
                &mut address_offset,
                parse_scaled_u64_text(c, "addressOffset")?,
                c,
                "addressOffset",
            )?,
            "dim" | "dimIncrement" | "dimIndex" | "dimName" | "dimArrayIndex" => {
                dim = Some(parse_dim_group(e)?);
            }
            "size" => props.size = Some(parse_scaled_u64_text(c, "size")?),
            "access" => props.access = Some(parse_access_text(c)?),
            "protection" => props.protection = Some(parse_protection_text(c)?),
            "resetValue" => props.reset_value = Some(parse_scaled_u64_text(c, "resetValue")?),
            "resetMask" => props.reset_mask = Some(parse_scaled_u64_text(c, "resetMask")?),
            "cluster" => items.push(svd::RegisterBlockItem::Cluster {
                cluster: parse_cluster(c)?,
            }),
            "register" => items.push(svd::RegisterBlockItem::Register {
                register: parse_register(c)?,
            }),
            _ => {
                return Err(Error::validation(
                    c.loc,
                    format!("unexpected element in <cluster>: <{}>", c.name),
                ));
            }
        }
    }

    if items.is_empty() {
        return Err(Error::validation(
            e.loc,
            "<cluster> must contain at least one nested <cluster> or <register>",
        ));
    }

    Ok(svd::Cluster {
        derived_from,
        name: required(name, e, "name")?,
        description: required(description, e, "description")?,
        alternate_cluster,
        header_struct_name,
        address_offset: required(address_offset, e, "addressOffset")?,
        dim,
        register_properties: props,
        items,
    })
}

fn parse_register(e: &xml::Element) -> Result<svd::Register> {
    assert_name(e, "register")?;
    let derived_from = e.attr("derivedFrom").map(|s| s.to_string());
    ensure_only_attrs(e, &["derivedFrom"])?;

    let mut name = None;
    let mut display_name = None;
    let mut description = None;
    let mut alternate_group = None;
    let mut alternate_register = None;
    let mut address_offset = None;
    let mut dim: Option<svd::DimElement> = None;
    let mut props = svd::RegisterProperties::default();
    let mut data_type: Option<svd::DataType> = None;
    let mut modified_write_values = None;
    let mut write_constraint = None;
    let mut read_action = None;
    let mut field = Vec::new();

    for c in e.children_elements() {
        match c.name.as_str() {
            "name" => set_once_text(&mut name, c, "name")?,
            "displayName" => set_once_text(&mut display_name, c, "displayName")?,
            "description" => set_once_text(&mut description, c, "description")?,
            "alternateGroup" => set_once_text(&mut alternate_group, c, "alternateGroup")?,
            "alternateRegister" => set_once_text(&mut alternate_register, c, "alternateRegister")?,
            "addressOffset" => set_once(
                &mut address_offset,
                parse_scaled_u64_text(c, "addressOffset")?,
                c,
                "addressOffset",
            )?,
            "dim" | "dimIncrement" | "dimIndex" | "dimName" | "dimArrayIndex" => {
                dim = Some(parse_dim_group(e)?);
            }
            "size" => props.size = Some(parse_scaled_u64_text(c, "size")?),
            "access" => props.access = Some(parse_access_text(c)?),
            "protection" => props.protection = Some(parse_protection_text(c)?),
            "resetValue" => props.reset_value = Some(parse_scaled_u64_text(c, "resetValue")?),
            "resetMask" => props.reset_mask = Some(parse_scaled_u64_text(c, "resetMask")?),
            "dataType" => set_once(&mut data_type, parse_data_type_text(c)?, c, "dataType")?,
            "modifiedWriteValues" => set_once(
                &mut modified_write_values,
                parse_mwv_text(c)?,
                c,
                "modifiedWriteValues",
            )?,
            "writeConstraint" => set_once(
                &mut write_constraint,
                parse_write_constraint(c)?,
                c,
                "writeConstraint",
            )?,
            "readAction" => set_once(
                &mut read_action,
                parse_read_action_text(c)?,
                c,
                "readAction",
            )?,
            "fields" => {
                // <fields><field>...</field>...</fields>
                for f in c.children_elements() {
                    if f.name != "field" {
                        return Err(Error::validation(
                            f.loc,
                            format!("in <fields> only <field> is allowed, got <{}>", f.name),
                        ));
                    }
                    field.push(parse_field(f)?);
                }
            }
            _ => {
                return Err(Error::validation(
                    c.loc,
                    format!("unexpected element in <register>: <{}>", c.name),
                ));
            }
        }
    }

    let alternate = match (alternate_group, alternate_register) {
        (Some(g), None) => Some(svd::RegisterAlternate::AlternateGroup { alternate_group: g }),
        (None, Some(r)) => Some(svd::RegisterAlternate::AlternateRegister {
            alternate_register: r,
        }),
        (None, None) => None,
        (Some(_), Some(_)) => {
            return Err(Error::validation(
                e.loc,
                "<register>: alternateGroup and alternateRegister are mutually exclusive",
            ));
        }
    };

    Ok(svd::Register {
        derived_from,
        name: required(name, e, "name")?,
        display_name,
        description,
        address_offset: required(address_offset, e, "addressOffset")?,
        dim,
        properties: props,
        alternate,
        data_type,
        modified_write_values,
        write_constraint,
        read_action,
        field,
    })
}

fn parse_field(e: &xml::Element) -> Result<svd::Field> {
    assert_name(e, "field")?;
    let derived_from = e.attr("derivedFrom").map(|s| s.to_string());
    ensure_only_attrs(e, &["derivedFrom"])?;
    let mut dim: Option<svd::DimElement> = None;
    let mut name = None;
    let mut description = None;
    let mut bit_range: Option<svd::BitRange> = None;
    let mut access = None;
    let mut modified_write_values = None;
    let mut write_constraint = None;
    let mut read_action = None;
    let mut enumerated_values = Vec::new();

    // bit range variants: bitRange OR (lsb+msb) OR (bitOffset+bitWidth)
    let mut lsb = None;
    let mut msb = None;
    let mut bit_offset = None;
    let mut bit_width = None;

    for c in e.children_elements() {
        match c.name.as_str() {
            "dim" | "dimIncrement" | "dimIndex" | "dimName" | "dimArrayIndex" => {
                dim = Some(parse_dim_group(e)?);
            }
            "name" => set_once_text(&mut name, c, "name")?,
            "description" => set_once_text(&mut description, c, "description")?,
            "bitRange" => set_once(&mut bit_range, parse_bit_range_string(c)?, c, "bitRange")?,
            "lsb" => set_once(&mut lsb, parse_u32_scaled(c, "lsb")?, c, "lsb")?,
            "msb" => set_once(&mut msb, parse_u32_scaled(c, "msb")?, c, "msb")?,
            "bitOffset" => set_once(
                &mut bit_offset,
                parse_u32_scaled(c, "bitOffset")?,
                c,
                "bitOffset",
            )?,
            "bitWidth" => set_once(
                &mut bit_width,
                parse_u32_scaled(c, "bitWidth")?,
                c,
                "bitWidth",
            )?,
            "access" => set_once(&mut access, parse_access_text(c)?, c, "access")?,
            "modifiedWriteValues" => set_once(
                &mut modified_write_values,
                parse_mwv_text(c)?,
                c,
                "modifiedWriteValues",
            )?,
            "writeConstraint" => set_once(
                &mut write_constraint,
                parse_write_constraint(c)?,
                c,
                "writeConstraint",
            )?,
            "readAction" => set_once(
                &mut read_action,
                parse_read_action_text(c)?,
                c,
                "readAction",
            )?,
            "enumeratedValues" => enumerated_values.push(parse_enumerated_values(c)?),
            _ => {
                return Err(Error::validation(
                    c.loc,
                    format!("unexpected element in <field>: <{}>", c.name),
                ));
            }
        }
    }

    let bit_range = if let Some(br) = bit_range {
        br
    } else if lsb.is_some() || msb.is_some() {
        svd::BitRange::LsbMsb {
            lsb: required(lsb, e, "lsb")?,
            msb: required(msb, e, "msb")?,
        }
    } else if bit_offset.is_some() || bit_width.is_some() {
        let bit_offset = required(bit_offset, e, "bitOffset")?;
        svd::BitRange::BitOffsetWidth {
            bit_offset,
            bit_width,
        }
    } else {
        return Err(Error::validation(
            e.loc,
            "<field> requires bitRange, or (lsb+msb), or (bitOffset+bitWidth)",
        ));
    };

    if enumerated_values.len() > 2 {
        return Err(Error::validation(
            e.loc,
            "<field>: at most 2 <enumeratedValues> blocks are allowed (XSD maxOccurs=2)",
        ));
    }

    Ok(svd::Field {
        derived_from,
        dim,
        name: required(name, e, "name")?,
        description,
        bit_range,
        access,
        modified_write_values,
        write_constraint,
        read_action,
        enumerated_values,
    })
}

fn parse_enumerated_values(e: &xml::Element) -> Result<svd::EnumeratedValues> {
    assert_name(e, "enumeratedValues")?;
    let derived_from = e.attr("derivedFrom").map(|s| s.to_string());
    ensure_only_attrs(e, &["derivedFrom"])?;

    let mut name = None;
    let mut header_enum_name = None;
    let mut usage = None;
    let mut enumerated_value = Vec::new();

    for c in e.children_elements() {
        match c.name.as_str() {
            "name" => set_once_text(&mut name, c, "name")?,
            "headerEnumName" => set_once_text(&mut header_enum_name, c, "headerEnumName")?,
            "usage" => set_once(&mut usage, parse_enum_usage_text(c)?, c, "usage")?,
            "enumeratedValue" => enumerated_value.push(parse_enumerated_value(c)?),
            _ => {
                return Err(Error::validation(
                    c.loc,
                    format!("unexpected element in <enumeratedValues>: <{}>", c.name),
                ));
            }
        }
    }
    if enumerated_value.is_empty() {
        return Err(Error::validation(
            e.loc,
            "<enumeratedValues> must contain at least one <enumeratedValue>",
        ));
    }
    Ok(svd::EnumeratedValues {
        derived_from,
        name,
        header_enum_name,
        usage,
        enumerated_value,
    })
}

fn parse_enumerated_value(e: &xml::Element) -> Result<svd::EnumeratedValue> {
    assert_name(e, "enumeratedValue")?;
    ensure_no_attrs(e, &[])?;
    let mut name = None;
    let mut description = None;
    let mut value = None;
    let mut is_default = None;
    for c in e.children_elements() {
        match c.name.as_str() {
            "name" => set_once_text(&mut name, c, "name")?,
            "description" => set_once_text(&mut description, c, "description")?,
            "value" => set_once_text(&mut value, c, "value")?,
            "isDefault" => set_once(
                &mut is_default,
                parse_bool_text(c, "isDefault")?,
                c,
                "isDefault",
            )?,
            _ => {
                return Err(Error::validation(
                    c.loc,
                    format!("unexpected element in <enumeratedValue>: <{}>", c.name),
                ));
            }
        }
    }
    let spec = match (value, is_default) {
        (Some(v), None) => {
            ensure_enumerated_value_pattern(e.loc, &v)?;
            svd::EnumeratedValueSpec::Value { value: v }
        }
        (None, Some(b)) => svd::EnumeratedValueSpec::IsDefault { is_default: b },
        (None, None) => {
            return Err(Error::validation(
                e.loc,
                "<enumeratedValue> must contain either <value> or <isDefault>",
            ));
        }
        (Some(_), Some(_)) => {
            return Err(Error::validation(
                e.loc,
                "<enumeratedValue>: <value> and <isDefault> are mutually exclusive (XSD choice)",
            ));
        }
    };
    Ok(svd::EnumeratedValue {
        name: required(name, e, "name")?,
        description,
        spec,
    })
}

fn parse_write_constraint(e: &xml::Element) -> Result<svd::WriteConstraint> {
    assert_name(e, "writeConstraint")?;
    ensure_no_attrs(e, &[])?;
    let mut found = None;
    for c in e.children_elements() {
        if found.is_some() {
            return Err(Error::validation(
                c.loc,
                "<writeConstraint> must contain exactly one constraint variant",
            ));
        }
        found = Some(match c.name.as_str() {
            "writeAsRead" => svd::WriteConstraint::WriteAsRead {
                write_as_read: parse_bool_text(c, "writeAsRead")?,
            },
            "useEnumeratedValues" => svd::WriteConstraint::UseEnumeratedValues {
                use_enumerated_values: parse_bool_text(c, "useEnumeratedValues")?,
            },
            "range" => {
                let (min, max) = parse_range(c)?;
                svd::WriteConstraint::Range {
                    minimum: min,
                    maximum: max,
                }
            }
            _ => {
                return Err(Error::validation(
                    c.loc,
                    format!("unexpected element in <writeConstraint>: <{}>", c.name),
                ));
            }
        });
    }
    found.ok_or_else(|| Error::validation(e.loc, "<writeConstraint> is empty"))
}

fn parse_range(e: &xml::Element) -> Result<(u64, u64)> {
    assert_name(e, "range")?;
    ensure_no_attrs(e, &[])?;
    let mut min = None;
    let mut max = None;
    for c in e.children_elements() {
        match c.name.as_str() {
            "minimum" => set_once(&mut min, parse_scaled_u64_text(c, "minimum")?, c, "minimum")?,
            "maximum" => set_once(&mut max, parse_scaled_u64_text(c, "maximum")?, c, "maximum")?,
            _ => {
                return Err(Error::validation(
                    c.loc,
                    format!("unexpected element in <range>: <{}>", c.name),
                ));
            }
        }
    }
    Ok((required(min, e, "minimum")?, required(max, e, "maximum")?))
}

// ---------------- helpers ----------------

fn assert_name(e: &xml::Element, expected: &str) -> Result<()> {
    if e.name != expected {
        return Err(Error::validation(
            e.loc,
            format!("expected <{}>, got <{}>", expected, e.name),
        ));
    }
    Ok(())
}

fn ensure_no_attrs(e: &xml::Element, allowed: &[&str]) -> Result<()> {
    ensure_only_attrs(e, allowed)
}

fn ensure_only_attrs(e: &xml::Element, allowed: &[&str]) -> Result<()> {
    for a in &e.attrs {
        if !allowed.iter().any(|x| *x == a.name) {
            return Err(Error::validation(
                e.loc,
                format!("unexpected attribute on <{}>: {}", e.name, a.name),
            ));
        }
    }
    Ok(())
}

fn required<T>(v: Option<T>, ctx: &xml::Element, field: &str) -> Result<T> {
    v.ok_or_else(|| {
        Error::validation(
            ctx.loc,
            format!("required field <{}> is missing in <{}>", field, ctx.name),
        )
    })
}

fn set_once<T>(dst: &mut Option<T>, v: T, ctx: &xml::Element, field: &str) -> Result<()> {
    if dst.is_some() {
        return Err(Error::validation(
            ctx.loc,
            format!("duplicate field <{}> in <{}>", field, ctx.name),
        ));
    }
    *dst = Some(v);
    Ok(())
}

fn set_once_text(dst: &mut Option<String>, e: &xml::Element, field: &str) -> Result<()> {
    let t = text_required(e, field)?;
    set_once(dst, t, e, field)
}

fn text_required(e: &xml::Element, field: &str) -> Result<String> {
    e.text()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::validation(e.loc, format!("element <{}> is empty", field)))
}

fn parse_bool_text(e: &xml::Element, field: &str) -> Result<bool> {
    let t = text_required(e, field)?;
    match t.as_str() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        _ => Err(Error::validation(
            e.loc,
            format!("invalid boolean for <{}>: {}", field, t),
        )),
    }
}

fn parse_u32_text(e: &xml::Element, field: &str) -> Result<u32> {
    let t = text_required(e, field)?;
    t.parse::<u32>().map_err(|_| {
        Error::validation(
            e.loc,
            format!("invalid integer (u32) for <{}>: {}", field, t),
        )
    })
}

fn parse_i64_text(e: &xml::Element, field: &str) -> Result<i64> {
    let t = text_required(e, field)?;
    t.parse::<i64>().map_err(|_| {
        Error::validation(
            e.loc,
            format!("invalid integer (i64) for <{}>: {}", field, t),
        )
    })
}

fn parse_revision_text(e: &xml::Element) -> Result<String> {
    let t = text_required(e, "revision")?;
    // XSD: r[0-9]*p[0-9]*
    let ok = t.starts_with('r')
        && t.contains('p')
        && t.split_once('p').is_some_and(|(l, r)| {
            l[1..].chars().all(|c| c.is_ascii_digit()) && r.chars().all(|c| c.is_ascii_digit())
        });
    if !ok {
        return Err(Error::validation(
            e.loc,
            format!("invalid revisionType (expected rNpM): {}", t),
        ));
    }
    Ok(t)
}

fn parse_scaled_u64_text(e: &xml::Element, field: &str) -> Result<u64> {
    let t = text_required(e, field)?;
    parse_scaled_u64(e.loc, &t)
        .map_err(|m| Error::validation(e.loc, format!("{} ({}): {}", field, t, m)))
}

fn parse_scaled_u32_text(e: &xml::Element, field: &str) -> Result<u32> {
    let v = parse_scaled_u64_text(e, field)?;
    u32::try_from(v).map_err(|_| {
        Error::validation(
            e.loc,
            format!("value <{}> does not fit into u32: {}", field, v),
        )
    })
}

fn parse_u32_scaled(e: &xml::Element, field: &str) -> Result<u32> {
    let v = parse_scaled_u64_text(e, field)?;
    u32::try_from(v).map_err(|_| {
        Error::validation(
            e.loc,
            format!("value <{}> does not fit into u32: {}", field, v),
        )
    })
}

fn parse_scaled_u64(loc: xml::Location, s: &str) -> core::result::Result<u64, String> {
    let mut s = s.trim();
    if let Some(rest) = s.strip_prefix('+') {
        s = rest;
    }
    let (mul, core) = match s.chars().last() {
        Some('k') | Some('K') => (1024u64, &s[..s.len() - 1]),
        Some('m') | Some('M') => (1024u64.pow(2), &s[..s.len() - 1]),
        Some('g') | Some('G') => (1024u64.pow(3), &s[..s.len() - 1]),
        Some('t') | Some('T') => (1024u64.pow(4), &s[..s.len() - 1]),
        _ => (1u64, s),
    };

    let core = core.trim();
    let (radix, digits) =
        if let Some(rest) = core.strip_prefix("0x").or_else(|| core.strip_prefix("0X")) {
            (16, rest)
        } else if let Some(rest) = core.strip_prefix('#') {
            (16, rest)
        } else if core.chars().any(|c| matches!(c, 'a'..='f' | 'A'..='F')) {
            (16, core)
        } else {
            (10, core)
        };
    if digits.is_empty() {
        return Err("empty value".to_string());
    }
    let v = u64::from_str_radix(digits, radix).map_err(|_| "invalid digits".to_string())?;
    v.checked_mul(mul)
        .ok_or_else(|| format!("overflow while applying scale (mul={})", mul))
        .map_err(|e| format!("{e} @ {loc}"))
}

fn parse_bool_attr(loc: xml::Location, name: &str, v: &str) -> Result<bool> {
    match v.trim() {
        "true" | "1" => Ok(true),
        "false" | "0" => Ok(false),
        x => Err(Error::validation(
            loc,
            format!("invalid boolean attribute {}: {}", name, x),
        )),
    }
}

fn parse_protection_attr(loc: xml::Location, name: &str, v: &str) -> Result<svd::Protection> {
    match v.trim() {
        "s" => Ok(svd::Protection::Secure),
        "n" => Ok(svd::Protection::NonSecure),
        "p" => Ok(svd::Protection::Privileged),
        x => Err(Error::validation(
            loc,
            format!("invalid attribute {} (protectionStringType): {}", name, x),
        )),
    }
}

fn parse_access_text(e: &xml::Element) -> Result<svd::AccessType> {
    let t = text_required(e, "access")?;
    // XSD: read-only|write-only|read-write|writeOnce|read-writeOnce.
    // In practice, other casing variants exist (e.g. read-writeonce).
    let norm = t.trim();
    let lower = norm.to_ascii_lowercase();
    match lower.as_str() {
        "read-only" => Ok(svd::AccessType::ReadOnly),
        "write-only" => Ok(svd::AccessType::WriteOnly),
        "read-write" => Ok(svd::AccessType::ReadWrite),
        "writeonce" | "write-once" => Ok(svd::AccessType::WriteOnce),
        "read-writeonce" | "read-write-once" => Ok(svd::AccessType::ReadWriteOnce),
        _ => Err(Error::validation(
            e.loc,
            format!("unknown accessType: {}", norm),
        )),
    }
}

fn parse_protection_text(e: &xml::Element) -> Result<svd::Protection> {
    let t = text_required(e, "protection")?;
    match t.as_str() {
        "s" => Ok(svd::Protection::Secure),
        "n" => Ok(svd::Protection::NonSecure),
        "p" => Ok(svd::Protection::Privileged),
        _ => Err(Error::validation(
            e.loc,
            format!("unknown protectionStringType: {}", t),
        )),
    }
}

fn parse_mwv_text(e: &xml::Element) -> Result<svd::ModifiedWriteValues> {
    let t = text_required(e, "modifiedWriteValues")?;
    use svd::ModifiedWriteValues as M;
    match t.as_str() {
        "oneToClear" => Ok(M::OneToClear),
        "oneToSet" => Ok(M::OneToSet),
        "oneToToggle" => Ok(M::OneToToggle),
        "zeroToClear" => Ok(M::ZeroToClear),
        "zeroToSet" => Ok(M::ZeroToSet),
        "zeroToToggle" => Ok(M::ZeroToToggle),
        "clear" => Ok(M::Clear),
        "set" => Ok(M::Set),
        "modify" => Ok(M::Modify),
        _ => Err(Error::validation(
            e.loc,
            format!("unknown modifiedWriteValuesType: {}", t),
        )),
    }
}

fn parse_data_type_text(e: &xml::Element) -> Result<svd::DataType> {
    let t = text_required(e, "dataType")?;
    use svd::DataType as D;
    match t.as_str() {
        "uint8_t" => Ok(D::UInt8),
        "uint16_t" => Ok(D::UInt16),
        "uint32_t" => Ok(D::UInt32),
        "uint64_t" => Ok(D::UInt64),
        "int8_t" => Ok(D::Int8),
        "int16_t" => Ok(D::Int16),
        "int32_t" => Ok(D::Int32),
        "int64_t" => Ok(D::Int64),
        "uint8_t *" => Ok(D::UInt8Ptr),
        "uint16_t *" => Ok(D::UInt16Ptr),
        "uint32_t *" => Ok(D::UInt32Ptr),
        "uint64_t *" => Ok(D::UInt64Ptr),
        "int8_t *" => Ok(D::Int8Ptr),
        "int16_t *" => Ok(D::Int16Ptr),
        "int32_t *" => Ok(D::Int32Ptr),
        "int64_t *" => Ok(D::Int64Ptr),
        _ => Err(Error::validation(
            e.loc,
            format!("unknown dataTypeType: {}", t),
        )),
    }
}

fn parse_read_action_text(e: &xml::Element) -> Result<svd::ReadAction> {
    let t = text_required(e, "readAction")?;
    use svd::ReadAction as R;
    match t.as_str() {
        "clear" => Ok(R::Clear),
        "set" => Ok(R::Set),
        "modify" => Ok(R::Modify),
        "modifyExternal" => Ok(R::ModifyExternal),
        _ => Err(Error::validation(
            e.loc,
            format!("unknown readActionType: {}", t),
        )),
    }
}

fn parse_enum_usage_text(e: &xml::Element) -> Result<svd::EnumUsage> {
    let t = text_required(e, "usage")?;
    match t.as_str() {
        "read" => Ok(svd::EnumUsage::Read),
        "write" => Ok(svd::EnumUsage::Write),
        "read-write" => Ok(svd::EnumUsage::ReadWrite),
        _ => Err(Error::validation(
            e.loc,
            format!("unknown enumUsageType: {}", t),
        )),
    }
}

fn parse_endian_text(e: &xml::Element) -> Result<svd::Endian> {
    let t = text_required(e, "endian")?;
    match t.as_str() {
        "little" => Ok(svd::Endian::Little),
        "big" => Ok(svd::Endian::Big),
        "selectable" => Ok(svd::Endian::Selectable),
        "other" => Ok(svd::Endian::Other),
        _ => Err(Error::validation(
            e.loc,
            format!("unknown endianType: {}", t),
        )),
    }
}

fn parse_cpu_name_text(e: &xml::Element) -> Result<svd::CpuName> {
    let t = text_required(e, "name")?;
    use svd::CpuName as C;
    match t.as_str() {
        "CM0" => Ok(C::CM0),
        "CM0PLUS" => Ok(C::CM0PLUS),
        "CM0+" => Ok(C::CM0Plus),
        "CM1" => Ok(C::CM1),
        "CM3" => Ok(C::CM3),
        "CM4" => Ok(C::CM4),
        "CM7" => Ok(C::CM7),
        "CM23" => Ok(C::CM23),
        "CM33" => Ok(C::CM33),
        "CM35P" => Ok(C::CM35P),
        "CM55" => Ok(C::CM55),
        "CM85" => Ok(C::CM85),
        "SC000" => Ok(C::SC000),
        "SC300" => Ok(C::SC300),
        "ARMV8MML" => Ok(C::ARMV8MML),
        "ARMV8MBL" => Ok(C::ARMV8MBL),
        "ARMV81MML" => Ok(C::ARMV81MML),
        "CA5" => Ok(C::CA5),
        "CA7" => Ok(C::CA7),
        "CA8" => Ok(C::CA8),
        "CA9" => Ok(C::CA9),
        "CA15" => Ok(C::CA15),
        "CA17" => Ok(C::CA17),
        "CA53" => Ok(C::CA53),
        "CA57" => Ok(C::CA57),
        "CA72" => Ok(C::CA72),
        "SMC1" => Ok(C::SMC1),
        "other" => Ok(C::Other),
        _ => Err(Error::validation(
            e.loc,
            format!("unknown cpuNameType: {}", t),
        )),
    }
}

fn parse_sau_access_text(e: &xml::Element) -> Result<svd::SauAccess> {
    let t = text_required(e, "access")?;
    match t.as_str() {
        "c" => Ok(svd::SauAccess::CallableOrSecure),
        "n" => Ok(svd::SauAccess::NonSecure),
        _ => Err(Error::validation(
            e.loc,
            format!("unknown sauAccessType: {}", t),
        )),
    }
}

fn parse_addr_usage_text(e: &xml::Element) -> Result<svd::AddressBlockUsage> {
    let t = text_required(e, "usage")?;
    match t.as_str() {
        "registers" => Ok(svd::AddressBlockUsage::Registers),
        "buffer" => Ok(svd::AddressBlockUsage::Buffer),
        "reserved" => Ok(svd::AddressBlockUsage::Reserved),
        _ => Err(Error::validation(
            e.loc,
            format!("unknown addressBlock usage: {}", t),
        )),
    }
}

fn parse_bit_range_string(e: &xml::Element) -> Result<svd::BitRange> {
    let t = text_required(e, "bitRange")?;
    let s = t.trim();
    if !(s.starts_with('[') && s.ends_with(']')) {
        return Err(Error::validation(
            e.loc,
            format!("bitRange must be in form [msb:lsb], got: {}", s),
        ));
    }
    let inner = &s[1..s.len() - 1];
    let (msb_s, lsb_s) = inner
        .split_once(':')
        .ok_or_else(|| Error::validation(e.loc, format!("bitRange must contain ':' ({})", s)))?;
    let msb = msb_s
        .trim()
        .parse::<u32>()
        .map_err(|_| Error::validation(e.loc, format!("invalid msb in bitRange: {}", msb_s)))?;
    let lsb = lsb_s
        .trim()
        .parse::<u32>()
        .map_err(|_| Error::validation(e.loc, format!("invalid lsb in bitRange: {}", lsb_s)))?;
    Ok(svd::BitRange::BitRangeString { msb, lsb })
}

fn parse_vendor_extensions(e: &xml::Element) -> Result<Vec<svd::AnyXmlElement>> {
    assert_name(e, "vendorExtensions")?;
    ensure_no_attrs(e, &[])?;
    let mut out = Vec::new();
    for n in &e.children {
        match n {
            xml::Node::Element(el) => out.push(any_xml_from_element(el)),
            xml::Node::Text(t) => {
                return Err(Error::validation(
                    e.loc,
                    format!(
                        "<vendorExtensions> must not contain text nodes per XSD (got: {:?})",
                        t
                    ),
                ));
            }
        }
    }
    Ok(out)
}

fn any_xml_from_element(e: &xml::Element) -> svd::AnyXmlElement {
    let attrs = e
        .attrs
        .iter()
        .map(|a| svd::AnyXmlAttr {
            name: a.name.clone(),
            value: a.value.clone(),
        })
        .collect();
    let mut children = Vec::new();
    for n in &e.children {
        match n {
            xml::Node::Element(el) => children.push(svd::AnyXmlNode::Element {
                element: any_xml_from_element(el),
            }),
            xml::Node::Text(t) => children.push(svd::AnyXmlNode::Text { text: t.clone() }),
        }
    }
    svd::AnyXmlElement {
        name: e.name.clone(),
        attrs,
        children,
    }
}

fn ensure_enumerated_value_pattern(loc: xml::Location, s: &str) -> Result<()> {
    let s = s.trim();
    let ok = {
        let s = s.strip_prefix('+').unwrap_or(s);
        if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
            !hex.is_empty() && hex.chars().all(|c| c.is_ascii_hexdigit())
        } else if s.chars().all(|c| c.is_ascii_digit()) {
            true
        } else if let Some(bin) = s.strip_prefix('#').or_else(|| s.strip_prefix("0b")) {
            !bin.is_empty() && bin.chars().all(|c| matches!(c, '0' | '1' | 'x' | 'X'))
        } else {
            false
        }
    };
    if ok {
        Ok(())
    } else {
        Err(Error::validation(
            loc,
            format!("enumeratedValue does not match XSD pattern: {}", s),
        ))
    }
}

fn parse_dim_group(parent: &xml::Element) -> Result<svd::DimElement> {
    // dimElementGroup is "flat" inside `parent`, so we collect it from parent's children.
    let mut dim = None;
    let mut dim_increment = None;
    let mut dim_index = None;
    let mut dim_name = None;
    let mut dim_array_index = None;
    for c in parent.children_elements() {
        match c.name.as_str() {
            "dim" => dim = Some(parse_scaled_u64_text(c, "dim")?),
            "dimIncrement" => dim_increment = Some(parse_scaled_u64_text(c, "dimIncrement")?),
            "dimIndex" => dim_index = Some(text_required(c, "dimIndex")?),
            "dimName" => dim_name = Some(text_required(c, "dimName")?),
            "dimArrayIndex" => dim_array_index = Some(parse_dim_array_index(c)?),
            _ => {}
        }
    }
    if dim.is_none()
        && dim_increment.is_none()
        && dim_index.is_none()
        && dim_name.is_none()
        && dim_array_index.is_none()
    {
        return Err(Error::validation(
            parent.loc,
            "dimElementGroup: no elements found",
        ));
    }
    Ok(svd::DimElement {
        dim: required(dim, parent, "dim")?,
        dim_increment: required(dim_increment, parent, "dimIncrement")?,
        dim_index,
        dim_name,
        dim_array_index,
    })
}

fn parse_dim_array_index(e: &xml::Element) -> Result<svd::DimArrayIndex> {
    assert_name(e, "dimArrayIndex")?;
    ensure_no_attrs(e, &[])?;
    let mut header_enum_name = None;
    let mut enumerated_value = Vec::new();
    for c in e.children_elements() {
        match c.name.as_str() {
            "headerEnumName" => set_once_text(&mut header_enum_name, c, "headerEnumName")?,
            "enumeratedValue" => enumerated_value.push(parse_enumerated_value(c)?),
            _ => {
                return Err(Error::validation(
                    c.loc,
                    format!("unexpected element in <dimArrayIndex>: <{}>", c.name),
                ));
            }
        }
    }
    if enumerated_value.is_empty() {
        return Err(Error::validation(
            e.loc,
            "<dimArrayIndex> must contain at least one <enumeratedValue>",
        ));
    }
    Ok(svd::DimArrayIndex {
        header_enum_name,
        enumerated_value,
    })
}
