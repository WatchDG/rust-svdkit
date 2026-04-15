//! Strongly typed CMSIS-SVD model (based on XSD revision 1.3.9).
//!
//! These structures intentionally do not try to be a 1:1 XML mapping:
//! XML → `xml::Document` → strict validation/conversion → this model.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVersion(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AccessType {
    ReadOnly,
    WriteOnly,
    ReadWrite,
    WriteOnce,
    ReadWriteOnce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModifiedWriteValues {
    OneToClear,
    OneToSet,
    OneToToggle,
    ZeroToClear,
    ZeroToSet,
    ZeroToToggle,
    Clear,
    Set,
    Modify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReadAction {
    Clear,
    Set,
    Modify,
    ModifyExternal,
}

/// s = Secure, n = Non-secure, p = Privileged.
///
/// Note: some tools may treat it as a combination, but XSD pattern `[snp]`
/// allows exactly one character, so we model it as a single enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Protection {
    #[serde(rename = "s")]
    Secure,
    #[serde(rename = "n")]
    NonSecure,
    #[serde(rename = "p")]
    Privileged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterProperties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access: Option<AccessType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protection: Option<Protection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_value: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_mask: Option<u64>,
}

impl Default for RegisterProperties {
    fn default() -> Self {
        Self {
            size: None,
            access: None,
            protection: None,
            reset_value: None,
            reset_mask: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DimElement {
    pub dim: u64,
    pub dim_increment: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim_index: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim_array_index: Option<DimArrayIndex>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DimArrayIndex {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_enum_name: Option<String>,
    pub enumerated_value: Vec<EnumeratedValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    /// `device/@schemaVersion` (XSD: required decimal; in practice a string like "1.3" / "1.3.9")
    pub schema_version: SchemaVersion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_id: Option<String>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,
    /// XSD: required
    pub version: String,
    /// XSD: required
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license_text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_system_filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_definitions_prefix: Option<String>,

    pub address_unit_bits: u32,
    pub width: u32,

    #[serde(flatten)]
    pub default_register_properties: RegisterProperties,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<Cpu>,

    pub peripherals: Vec<Peripheral>,

    /// `<vendorExtensions>` allows `xs:any*` (arbitrary XML). We store it as a
    /// serializable subtree.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor_extensions: Option<Vec<AnyXmlElement>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Endian {
    Little,
    Big,
    Selectable,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CpuName {
    CM0,
    CM0PLUS,
    #[serde(rename = "CM0+")]
    CM0Plus,
    CM1,
    CM3,
    CM4,
    CM7,
    CM23,
    CM33,
    CM35P,
    CM55,
    CM85,
    SC000,
    SC300,
    ARMV8MML,
    ARMV8MBL,
    ARMV81MML,
    CA5,
    CA7,
    CA8,
    CA9,
    CA15,
    CA17,
    CA53,
    CA57,
    CA72,
    SMC1,
    #[serde(rename = "other")]
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cpu {
    pub name: CpuName,
    pub revision: String,
    pub endian: Endian,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mpu_present: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fpu_present: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fpu_dp: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dsp_present: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icache_present: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dcache_present: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub itcm_present: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dtcm_present: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vtor_present: Option<bool>,
    /// XSD: required
    pub nvic_prio_bits: u32,
    /// XSD: required
    pub vendor_systick_config: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_num_interrupts: Option<u32>,

    // SAU (Secure Attribution Unit) extension (v1.3)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sau_num_regions: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sau_regions_config: Option<SauRegionsConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SauRegionsConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_protection_when_disabled")]
    pub protection_when_disabled: Protection,
    #[serde(default)]
    pub regions: Vec<SauRegion>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SauRegion {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub base: u64,
    pub limit: u64,
    pub access: SauAccess,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SauAccess {
    #[serde(rename = "c")]
    CallableOrSecure,
    #[serde(rename = "n")]
    NonSecure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Peripheral {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derived_from: Option<String>,

    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternate_peripheral: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepend_to_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub append_to_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_struct_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_condition: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim: Option<DimElement>,

    pub base_address: u64,

    #[serde(flatten)]
    pub register_properties: RegisterProperties,

    #[serde(default)]
    pub address_block: Vec<AddressBlock>,
    #[serde(default)]
    pub interrupt: Vec<Interrupt>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub registers: Option<RegisterBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressBlock {
    pub offset: u64,
    pub size: u64,
    pub usage: AddressBlockUsage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protection: Option<Protection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AddressBlockUsage {
    Registers,
    Buffer,
    Reserved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Interrupt {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub value: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterBlock {
    /// XSD: `choice (cluster|register) maxOccurs=unbounded` — preserves order.
    #[serde(default)]
    pub items: Vec<RegisterBlockItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RegisterBlockItem {
    Cluster { cluster: Cluster },
    Register { register: Register },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cluster {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derived_from: Option<String>,

    pub name: String,
    /// XSD: required
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternate_cluster: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_struct_name: Option<String>,
    pub address_offset: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim: Option<DimElement>,

    #[serde(flatten)]
    pub register_properties: RegisterProperties,

    /// XSD: `choice (register|cluster) maxOccurs=unbounded` — preserves order.
    #[serde(default)]
    pub items: Vec<RegisterBlockItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Register {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derived_from: Option<String>,

    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub address_offset: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim: Option<DimElement>,

    #[serde(flatten)]
    pub properties: RegisterProperties,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternate: Option<RegisterAlternate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<DataType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_write_values: Option<ModifiedWriteValues>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_constraint: Option<WriteConstraint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_action: Option<ReadAction>,

    #[serde(default)]
    pub field: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derived_from: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub dim: Option<DimElement>,

    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    pub bit_range: BitRange,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub access: Option<AccessType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_write_values: Option<ModifiedWriteValues>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write_constraint: Option<WriteConstraint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_action: Option<ReadAction>,

    #[serde(default)]
    pub enumerated_values: Vec<EnumeratedValues>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "style", rename_all = "camelCase")]
pub enum BitRange {
    /// `<bitRange>[msb:lsb]</bitRange>`
    BitRangeString { msb: u32, lsb: u32 },
    /// `<lsb>..</lsb><msb>..</msb>`
    LsbMsb { lsb: u32, msb: u32 },
    /// `<bitOffset>..</bitOffset><bitWidth>..</bitWidth>`
    BitOffsetWidth {
        bit_offset: u32,
        /// XSD: `bitWidth` is optional in the group.
        #[serde(skip_serializing_if = "Option::is_none")]
        bit_width: Option<u32>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumeratedValues {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derived_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_enum_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<EnumUsage>,
    pub enumerated_value: Vec<EnumeratedValue>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EnumUsage {
    Read,
    Write,
    ReadWrite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnumeratedValue {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub spec: EnumeratedValueSpec,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum EnumeratedValueSpec {
    /// Value as a raw XML string (hex/dec/binary/#/0b + X).
    Value { value: String },
    /// XSD: alternative to `value`
    IsDefault { is_default: bool },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum WriteConstraint {
    WriteAsRead { write_as_read: bool },
    UseEnumeratedValues { use_enumerated_values: bool },
    Range { minimum: u64, maximum: u64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    #[serde(rename = "uint8_t")]
    UInt8,
    #[serde(rename = "uint16_t")]
    UInt16,
    #[serde(rename = "uint32_t")]
    UInt32,
    #[serde(rename = "uint64_t")]
    UInt64,
    #[serde(rename = "int8_t")]
    Int8,
    #[serde(rename = "int16_t")]
    Int16,
    #[serde(rename = "int32_t")]
    Int32,
    #[serde(rename = "int64_t")]
    Int64,
    #[serde(rename = "uint8_t *")]
    UInt8Ptr,
    #[serde(rename = "uint16_t *")]
    UInt16Ptr,
    #[serde(rename = "uint32_t *")]
    UInt32Ptr,
    #[serde(rename = "uint64_t *")]
    UInt64Ptr,
    #[serde(rename = "int8_t *")]
    Int8Ptr,
    #[serde(rename = "int16_t *")]
    Int16Ptr,
    #[serde(rename = "int32_t *")]
    Int32Ptr,
    #[serde(rename = "int64_t *")]
    Int64Ptr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RegisterAlternate {
    AlternateGroup { alternate_group: String },
    AlternateRegister { alternate_register: String },
}

// ------------- generic XML for vendorExtensions -------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnyXmlElement {
    pub name: String,
    #[serde(default)]
    pub attrs: Vec<AnyXmlAttr>,
    #[serde(default)]
    pub children: Vec<AnyXmlNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnyXmlAttr {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AnyXmlNode {
    Element { element: AnyXmlElement },
    Text { text: String },
}

fn default_true() -> bool {
    true
}

fn default_protection_when_disabled() -> Protection {
    Protection::Secure
}
