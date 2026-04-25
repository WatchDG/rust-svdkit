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
fn golden_nrf52840_pac_enums_snapshot() {
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
        .join("enums.rs");

    let snapshot_content =
        std::fs::read_to_string(&snapshot_path).expect("failed to read snapshot enums.rs");

    let generated_enums = gen_dir
        .files
        .iter()
        .find(|f| f.file_name == "enums.rs")
        .expect("enums.rs not found in generated files")
        .content
        .clone();

    assert_eq!(
        generated_enums, snapshot_content,
        "generated enums.rs does not match snapshot"
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
