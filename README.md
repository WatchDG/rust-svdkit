# svdkit

CMSIS-SVD code generator for Rust PAC/HAL.

## CLI

Generate PAC and HAL code from SVD files:

```bash
cargo run --bin svd2rs -- --svd_file tests/svds/nrf52840.svd --out_dir generated/nrf52840
```

### Options

- `--svd_file <FILE>` — Input SVD file path (required)
- `--out_dir <DIR>` — Output directory for generated files (required)
- `--pac` — Generate only PAC files
- `--hal` — Generate only HAL files
- `--rt` — Generate only runtime files (startup, linker script)

### Output Files

By default generates:

- `<device>_pac.rs` — Peripheral Access Crate registers
- `<device>_cortex_m.rs` — NVIC module
- `<device>_rt.rs` — Startup and vector table
- `<device>_link.x` — Linker script
- `<device>_hal.rs` — HAL helpers
