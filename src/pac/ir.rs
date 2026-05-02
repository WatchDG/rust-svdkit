use crate::svd;

#[derive(Debug, Clone)]
pub struct PacIr {
    pub device_info: DeviceInfo,
    pub interrupts: InterruptTable,
    pub memory_regions: Vec<MemoryRegion>,
    pub peripherals: Vec<PeripheralIr>,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub name: String,
    pub file_stem: String,
    pub description: String,
    pub cpu: Option<svd::Cpu>,
    pub width: u32,
    pub address_unit_bits: u32,
    pub constants: Vec<(String, u64)>,
}

#[derive(Debug, Clone)]
pub struct InterruptTable {
    pub num_irqs: u32,
    pub irqs: Vec<(u32, String, Option<String>)>,
}

#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub name: String,
    pub base: u64,
    pub size: u64,
    pub kind: MemoryKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryKind {
    Flash,
    Ram,
    Peripheral,
    External,
}

#[derive(Debug, Clone)]
pub struct PeripheralIr {
    pub name: String,
    pub base_address: u64,
    pub module_name: String,
    pub type_name: String,
    pub const_name: String,
    pub description: Option<String>,
    pub register_block: RegisterBlockIr,
    pub has_clusters: bool,
    pub field_enums: Vec<EnumDef>,
    pub once_regs: Vec<OnceRegIr>,
}

#[derive(Debug, Clone)]
pub struct RegisterBlockIr {
    pub items: Vec<RegisterBlockItemIr>,
    pub total_size: u64,
}

#[derive(Debug, Clone)]
pub enum RegisterBlockItemIr {
    Register(RegisterFieldIr),
    Cluster(ClusterFieldIr),
    Reserved {
        offset: u64,
        size: u64,
        index: usize,
    },
}

#[derive(Debug, Clone)]
pub struct RegisterFieldIr {
    pub name: String,
    pub field_name: String,
    pub offset: u64,
    pub size_bytes: u64,
    pub base_type: String,
    pub access: ResolvedAccessIr,
    pub reg_type_name: String,
    pub reg_type: RegisterTypeIr,
    pub reset_value: Option<(u64, u64)>,
    pub read_action: Option<svd::ReadAction>,
    pub write_constraint: Option<WriteConstraintIr>,
    pub fields: Vec<FieldIr>,
    pub is_once: bool,
    pub dim: Option<DimInfo>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClusterFieldIr {
    pub name: String,
    pub field_name: String,
    pub offset: u64,
    pub size_bytes: u64,
    pub cluster_type_name: String,
    pub cluster_path: String,
    pub items: Vec<RegisterBlockItemIr>,
    pub dim: Option<DimInfo>,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FieldIr {
    pub name: String,
    pub lsb: u32,
    pub width: u32,
    pub mask: u64,
    pub access: svd::AccessType,
    pub description: Option<String>,
    pub enum_bindings: Vec<FieldEnumBindingIr>,
    pub has_write_constraint: bool,
}

#[derive(Debug, Clone)]
pub struct FieldEnumBindingIr {
    pub enum_type_name: String,
    pub usage: svd::EnumUsage,
    pub is_read_pick: bool,
    pub is_write_pick: bool,
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub type_name: String,
    pub repr: String,
    pub doc: String,
    pub variants: Vec<EnumVariant>,
    pub dedup_key: String,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<u64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub enum RegisterTypeIr {
    Primitive,
    Wrapper {
        inner: String,
        has_read: bool,
        has_write: bool,
        write_model: WriteModel,
        is_once: bool,
        once_ctor: Option<String>,
    },
}

impl RegisterTypeIr {
    pub fn has_read(&self) -> bool {
        match self {
            Self::Wrapper { has_read, .. } => *has_read,
            Self::Primitive => false,
        }
    }

    pub fn has_write(&self) -> bool {
        match self {
            Self::Wrapper { has_write, .. } => *has_write,
            Self::Primitive => false,
        }
    }

    pub fn write_model(&self) -> WriteModel {
        match self {
            Self::Wrapper { write_model, .. } => *write_model,
            Self::Primitive => WriteModel::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteModel {
    Normal,
    W1S,
    W1C,
    W0S,
    W0C,
    WT,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedAccessIr {
    pub access: svd::AccessType,
    pub write_model: WriteModel,
}

#[derive(Debug, Clone)]
pub enum WriteConstraintIr {
    WriteAsRead,
    UseEnumeratedValues,
    Range { min: u64, max: u64 },
}

#[derive(Debug, Clone)]
pub struct DimInfo {
    pub dim: u64,
    pub dim_increment: u64,
    pub is_typed_array: bool,
}

#[derive(Debug, Clone)]
pub struct OnceRegIr {
    pub field_name: String,
    pub token_ty: String,
    pub init_expr: String,
}

impl PacIr {
    pub fn peripheral_module_names(&self) -> Vec<String> {
        self.peripherals
            .iter()
            .map(|p| p.module_name.clone())
            .collect()
    }
}
