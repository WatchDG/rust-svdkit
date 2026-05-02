use std::path::Path;
use svdkit::pac;

#[test]
fn golden_nrf52840_pac_macros_snapshot() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let svd_path = manifest_dir.join("tests").join("svds").join("nrf52840.svd");

    let svd_content = std::fs::read_to_string(&svd_path).expect("failed to read nrf52840.svd");

    let device = svdkit::parse_svd(&svd_content).expect("failed to parse nrf52840.svd");

    let gen_dir = pac::generate_device_dir(&device).expect("failed to generate PAC");

    let snapshot_path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("nrf52840")
        .join("pac")
        .join("macros.rs");

    let snapshot_content =
        std::fs::read_to_string(&snapshot_path).expect("failed to read snapshot macros.rs");

    let generated_macros = gen_dir
        .files
        .iter()
        .find(|f| f.file_name == "macros.rs")
        .expect("macros.rs not found in generated files")
        .content
        .clone();

    assert_eq!(
        generated_macros, snapshot_content,
        "generated macros.rs does not match snapshot"
    );
}

#[test]
fn golden_nrf52840_pac_p0_registers_snapshot() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let svd_path = manifest_dir.join("tests").join("svds").join("nrf52840.svd");

    let svd_content = std::fs::read_to_string(&svd_path).expect("failed to read nrf52840.svd");

    let device = svdkit::parse_svd(&svd_content).expect("failed to parse nrf52840.svd");

    let gen_dir = pac::generate_device_dir(&device).expect("failed to generate PAC");

    let snapshot_path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("nrf52840")
        .join("pac")
        .join("peripherals")
        .join("p0")
        .join("registers.rs");

    let snapshot_content =
        std::fs::read_to_string(&snapshot_path).expect("failed to read snapshot registers.rs");

    let generated_registers = gen_dir
        .files
        .iter()
        .find(|f| f.file_name == "peripherals/p0/registers.rs")
        .expect("peripherals/p0/registers.rs not found in generated files")
        .content
        .clone();

    assert_eq!(
        generated_registers, snapshot_content,
        "generated p0/registers.rs does not match snapshot"
    );
}

#[test]
fn golden_nrf52840_pac_p0_enums_snapshot() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let svd_path = manifest_dir.join("tests").join("svds").join("nrf52840.svd");

    let svd_content = std::fs::read_to_string(&svd_path).expect("failed to read nrf52840.svd");

    let device = svdkit::parse_svd(&svd_content).expect("failed to parse nrf52840.svd");

    let gen_dir = pac::generate_device_dir(&device).expect("failed to generate PAC");

    let snapshot_path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("nrf52840")
        .join("pac")
        .join("peripherals")
        .join("p0")
        .join("enums.rs");

    let snapshot_content =
        std::fs::read_to_string(&snapshot_path).expect("failed to read snapshot p0/enums.rs");

    let generated_enums = gen_dir
        .files
        .iter()
        .find(|f| f.file_name == "peripherals/p0/enums.rs")
        .expect("peripherals/p0/enums.rs not found in generated files")
        .content
        .clone();

    assert_eq!(
        generated_enums, snapshot_content,
        "generated p0/enums.rs does not match snapshot"
    );
}

#[test]
fn golden_nrf52840_pac_p0_has_singleton() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let svd_path = manifest_dir.join("tests").join("svds").join("nrf52840.svd");

    let svd_content = std::fs::read_to_string(&svd_path).expect("failed to read nrf52840.svd");

    let device = svdkit::parse_svd(&svd_content).expect("failed to parse nrf52840.svd");

    let gen_dir = pac::generate_device_dir(&device).expect("failed to generate PAC");

    let generated_mod = gen_dir
        .files
        .iter()
        .find(|f| f.file_name == "peripherals/p0/mod.rs")
        .expect("peripherals/p0/mod.rs not found in generated files")
        .content
        .clone();

    assert!(
        generated_mod.contains("pub struct Peripherals"),
        "p0/mod.rs should contain Peripherals struct"
    );
    assert!(
        generated_mod.contains("pub unsafe fn steal()"),
        "p0/mod.rs should contain steal() method"
    );
    assert!(
        generated_mod.contains("impl Clone for Peripherals"),
        "p0/mod.rs should contain Clone impl"
    );
    assert!(
        generated_mod.contains("core::ptr::read_volatile(PTR)"),
        "p0/mod.rs should use ptr::read_volatile"
    );
}

#[test]
fn golden_nrf52840_pac_p1_registers_snapshot() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let svd_path = manifest_dir.join("tests").join("svds").join("nrf52840.svd");

    let svd_content = std::fs::read_to_string(&svd_path).expect("failed to read nrf52840.svd");

    let device = svdkit::parse_svd(&svd_content).expect("failed to parse nrf52840.svd");

    let gen_dir = pac::generate_device_dir(&device).expect("failed to generate PAC");

    let snapshot_path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("nrf52840")
        .join("pac")
        .join("peripherals")
        .join("p1")
        .join("registers.rs");

    let snapshot_content =
        std::fs::read_to_string(&snapshot_path).expect("failed to read snapshot p1/registers.rs");

    let generated_registers = gen_dir
        .files
        .iter()
        .find(|f| f.file_name == "peripherals/p1/registers.rs")
        .expect("peripherals/p1/registers.rs not found in generated files")
        .content
        .clone();

    assert_eq!(
        generated_registers, snapshot_content,
        "generated p1/registers.rs does not match snapshot"
    );
}

#[test]
fn golden_nrf52840_pac_p1_enums_snapshot() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let svd_path = manifest_dir.join("tests").join("svds").join("nrf52840.svd");

    let svd_content = std::fs::read_to_string(&svd_path).expect("failed to read nrf52840.svd");

    let device = svdkit::parse_svd(&svd_content).expect("failed to parse nrf52840.svd");

    let gen_dir = pac::generate_device_dir(&device).expect("failed to generate PAC");

    let snapshot_path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("nrf52840")
        .join("pac")
        .join("peripherals")
        .join("p1")
        .join("enums.rs");

    let snapshot_content =
        std::fs::read_to_string(&snapshot_path).expect("failed to read snapshot p1/enums.rs");

    let generated_enums = gen_dir
        .files
        .iter()
        .find(|f| f.file_name == "peripherals/p1/enums.rs")
        .expect("peripherals/p1/enums.rs not found in generated files")
        .content
        .clone();

    assert_eq!(
        generated_enums, snapshot_content,
        "generated p1/enums.rs does not match snapshot"
    );
}

#[test]
fn golden_nrf52840_pac_p1_has_singleton() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let svd_path = manifest_dir.join("tests").join("svds").join("nrf52840.svd");

    let svd_content = std::fs::read_to_string(&svd_path).expect("failed to read nrf52840.svd");

    let device = svdkit::parse_svd(&svd_content).expect("failed to parse nrf52840.svd");

    let gen_dir = pac::generate_device_dir(&device).expect("failed to generate PAC");

    let generated_mod = gen_dir
        .files
        .iter()
        .find(|f| f.file_name == "peripherals/p1/mod.rs")
        .expect("peripherals/p1/mod.rs not found in generated files")
        .content
        .clone();

    assert!(
        generated_mod.contains("pub struct Peripherals"),
        "p1/mod.rs should contain Peripherals struct"
    );
    assert!(
        generated_mod.contains("pub unsafe fn steal()"),
        "p1/mod.rs should contain steal() method"
    );
    assert!(
        generated_mod.contains("impl Clone for Peripherals"),
        "p1/mod.rs should contain Clone impl"
    );
    assert!(
        generated_mod.contains("core::ptr::read_volatile(PTR)"),
        "p1/mod.rs should use ptr::read_volatile"
    );
}

#[test]
fn golden_nrf52840_pac_timer0_registers_snapshot() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let svd_path = manifest_dir.join("tests").join("svds").join("nrf52840.svd");

    let svd_content = std::fs::read_to_string(&svd_path).expect("failed to read nrf52840.svd");

    let device = svdkit::parse_svd(&svd_content).expect("failed to parse nrf52840.svd");

    let gen_dir = pac::generate_device_dir(&device).expect("failed to generate PAC");

    let snapshot_path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("nrf52840")
        .join("pac")
        .join("peripherals")
        .join("timer0")
        .join("registers.rs");

    let snapshot_content = std::fs::read_to_string(&snapshot_path)
        .expect("failed to read snapshot timer0/registers.rs");

    let generated_registers = gen_dir
        .files
        .iter()
        .find(|f| f.file_name == "peripherals/timer0/registers.rs")
        .expect("peripherals/timer0/registers.rs not found in generated files")
        .content
        .clone();

    assert_eq!(
        generated_registers, snapshot_content,
        "generated timer0/registers.rs does not match snapshot"
    );
}

#[test]
fn golden_nrf52840_pac_timer0_enums_snapshot() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let svd_path = manifest_dir.join("tests").join("svds").join("nrf52840.svd");

    let svd_content = std::fs::read_to_string(&svd_path).expect("failed to read nrf52840.svd");

    let device = svdkit::parse_svd(&svd_content).expect("failed to parse nrf52840.svd");

    let gen_dir = pac::generate_device_dir(&device).expect("failed to generate PAC");

    let snapshot_path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("nrf52840")
        .join("pac")
        .join("peripherals")
        .join("timer0")
        .join("enums.rs");

    let snapshot_content =
        std::fs::read_to_string(&snapshot_path).expect("failed to read snapshot timer0/enums.rs");

    let generated_enums = gen_dir
        .files
        .iter()
        .find(|f| f.file_name == "peripherals/timer0/enums.rs")
        .expect("peripherals/timer0/enums.rs not found in generated files")
        .content
        .clone();

    assert_eq!(
        generated_enums, snapshot_content,
        "generated timer0/enums.rs does not match snapshot"
    );
}

#[test]
fn golden_nrf52840_pac_wdt_registers_snapshot() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let svd_path = manifest_dir.join("tests").join("svds").join("nrf52840.svd");

    let svd_content = std::fs::read_to_string(&svd_path).expect("failed to read nrf52840.svd");

    let device = svdkit::parse_svd(&svd_content).expect("failed to parse nrf52840.svd");

    let gen_dir = pac::generate_device_dir(&device).expect("failed to generate PAC");

    let snapshot_path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("nrf52840")
        .join("pac")
        .join("peripherals")
        .join("wdt")
        .join("registers.rs");

    let snapshot_content =
        std::fs::read_to_string(&snapshot_path).expect("failed to read snapshot wdt/registers.rs");

    let generated_registers = gen_dir
        .files
        .iter()
        .find(|f| f.file_name == "peripherals/wdt/registers.rs")
        .expect("peripherals/wdt/registers.rs not found in generated files")
        .content
        .clone();

    assert_eq!(
        generated_registers, snapshot_content,
        "generated wdt/registers.rs does not match snapshot"
    );
}

#[test]
fn golden_nrf52840_pac_traits_snapshot() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let svd_path = manifest_dir.join("tests").join("svds").join("nrf52840.svd");

    let svd_content = std::fs::read_to_string(&svd_path).expect("failed to read nrf52840.svd");

    let device = svdkit::parse_svd(&svd_content).expect("failed to parse nrf52840.svd");

    let gen_dir = pac::generate_device_dir(&device).expect("failed to generate PAC");

    let snapshot_path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("nrf52840")
        .join("pac")
        .join("traits.rs");

    let snapshot_content =
        std::fs::read_to_string(&snapshot_path).expect("failed to read snapshot traits.rs");

    let generated_traits = gen_dir
        .files
        .iter()
        .find(|f| f.file_name == "traits.rs")
        .expect("traits.rs not found in generated files")
        .content
        .clone();

    assert_eq!(
        generated_traits, snapshot_content,
        "generated traits.rs does not match snapshot"
    );
}

#[test]
fn golden_nrf52840_pac_types_snapshot() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let svd_path = manifest_dir.join("tests").join("svds").join("nrf52840.svd");

    let svd_content = std::fs::read_to_string(&svd_path).expect("failed to read nrf52840.svd");

    let device = svdkit::parse_svd(&svd_content).expect("failed to parse nrf52840.svd");

    let gen_dir = pac::generate_device_dir(&device).expect("failed to generate PAC");

    let snapshot_path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("nrf52840")
        .join("pac")
        .join("types.rs");

    let snapshot_content =
        std::fs::read_to_string(&snapshot_path).expect("failed to read snapshot types.rs");

    let generated_types = gen_dir
        .files
        .iter()
        .find(|f| f.file_name == "types.rs")
        .expect("types.rs not found in generated files")
        .content
        .clone();

    assert_eq!(
        generated_types, snapshot_content,
        "generated types.rs does not match snapshot"
    );
}

#[test]
fn golden_nrf52840_pac_enums_has_interrupt_impls() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let svd_path = manifest_dir.join("tests").join("svds").join("nrf52840.svd");

    let svd_content = std::fs::read_to_string(&svd_path).expect("failed to read nrf52840.svd");

    let device = svdkit::parse_svd(&svd_content).expect("failed to parse nrf52840.svd");

    let gen_dir = pac::generate_device_dir(&device).expect("failed to generate PAC");

    let generated_enums = gen_dir
        .files
        .iter()
        .find(|f| f.file_name == "enums.rs")
        .expect("enums.rs not found in generated files")
        .content
        .clone();

    assert!(
        generated_enums.contains("pub enum Interrupt"),
        "enums.rs should contain Interrupt enum"
    );
    assert!(
        generated_enums.contains("impl Interrupt {"),
        "enums.rs should contain Interrupt impl block"
    );
    assert!(
        generated_enums.contains("pub const fn bits(self)"),
        "enums.rs should contain bits() method"
    );
    assert!(
        generated_enums.contains("pub const fn from_bits"),
        "enums.rs should contain from_bits() method"
    );
    assert!(
        generated_enums.contains("impl From<Interrupt> for u16"),
        "enums.rs should contain From<Interrupt> for u16 impl"
    );
    assert!(
        generated_enums.contains("impl From<Interrupt> for u32"),
        "enums.rs should contain From<Interrupt> for u32 impl"
    );
    assert!(
        generated_enums.contains("impl TryFrom<u16> for Interrupt"),
        "enums.rs should contain TryFrom<u16> for Interrupt impl"
    );
    assert!(
        generated_enums.contains("impl TryFrom<u32> for Interrupt"),
        "enums.rs should contain TryFrom<u32> for Interrupt impl"
    );
}

#[test]
fn golden_nrf52840_pac_constants_snapshot() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let svd_path = manifest_dir.join("tests").join("svds").join("nrf52840.svd");

    let svd_content = std::fs::read_to_string(&svd_path).expect("failed to read nrf52840.svd");

    let device = svdkit::parse_svd(&svd_content).expect("failed to parse nrf52840.svd");

    let gen_dir = pac::generate_device_dir(&device).expect("failed to generate PAC");

    let snapshot_path = manifest_dir
        .join("tests")
        .join("snapshots")
        .join("nrf52840")
        .join("pac")
        .join("constants.rs");

    let snapshot_content =
        std::fs::read_to_string(&snapshot_path).expect("failed to read snapshot constants.rs");

    let generated_constants = gen_dir
        .files
        .iter()
        .find(|f| f.file_name == "constants.rs")
        .expect("constants.rs not found in generated files")
        .content
        .clone();

    assert_eq!(
        generated_constants, snapshot_content,
        "generated constants.rs does not match snapshot"
    );
}

#[test]
fn golden_nrf52840_pac_peripherals_mod_has_singleton() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let svd_path = manifest_dir.join("tests").join("svds").join("nrf52840.svd");

    let svd_content = std::fs::read_to_string(&svd_path).expect("failed to read nrf52840.svd");

    let device = svdkit::parse_svd(&svd_content).expect("failed to parse nrf52840.svd");

    let gen_dir = pac::generate_device_dir(&device).expect("failed to generate PAC");

    let generated_peripherals_mod = gen_dir
        .files
        .iter()
        .find(|f| f.file_name == "peripherals/mod.rs")
        .expect("peripherals/mod.rs not found in generated files")
        .content
        .clone();

    assert!(
        generated_peripherals_mod.contains("pub struct Peripherals"),
        "peripherals/mod.rs should contain Peripherals struct"
    );
    assert!(
        generated_peripherals_mod.contains("pub fn take()"),
        "peripherals/mod.rs should contain take() method"
    );
    assert!(
        generated_peripherals_mod.contains("pub unsafe fn steal()"),
        "peripherals/mod.rs should contain steal() method"
    );
    assert!(
        generated_peripherals_mod.contains("static TAKEN: AtomicBool"),
        "peripherals/mod.rs should contain atomic TAKEN flag"
    );
}

#[test]
fn peripherals_singleton_contains_all_peripherals() {
    use svdkit::pac::generate_peripherals_singleton;

    let peripherals = vec!["clock", "usbd", "p0", "p1"];
    let generated = generate_peripherals_singleton(&peripherals, "test_device");

    assert!(
        generated.contains("static TAKEN: AtomicBool"),
        "Should contain atomic TAKEN flag"
    );
    assert!(
        generated.contains("pub struct Peripherals"),
        "Should contain Peripherals struct"
    );
    assert!(
        generated.contains("pub fn take()"),
        "Should contain take() method"
    );
    assert!(
        generated.contains("pub unsafe fn steal()"),
        "Should contain steal() method"
    );

    for p in &peripherals {
        assert!(
            generated.contains(&format!("pub {p}:")),
            "Should contain field for {}",
            p
        );
        assert!(
            generated.contains(&format!("{p}::Peripherals::steal()")),
            "Should call steal() for {}",
            p
        );
    }
}
