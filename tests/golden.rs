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
