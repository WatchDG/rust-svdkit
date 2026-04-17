use std::path::PathBuf;
use svdkit::{Result, pac};

fn main() -> Result<()> {
    let matches = clap::Command::new("svd2rs")
        .about("CMSIS-SVD to Rust code generator")
        .arg(
            clap::Arg::new("svd_file")
                .long("svd_file")
                .value_name("FILE")
                .required(true)
                .help("Input SVD file path"),
        )
        .arg(
            clap::Arg::new("out_dir")
                .long("out_dir")
                .value_name("DIR")
                .required(true)
                .help("Output directory for generated files"),
        )
        .arg(
            clap::Arg::new("pac")
                .long("pac")
                .action(clap::ArgAction::SetTrue)
                .help("Generate PAC files (default: all)"),
        )
        .arg(
            clap::Arg::new("hal")
                .long("hal")
                .action(clap::ArgAction::SetTrue)
                .help("Generate HAL files"),
        )
        .arg(
            clap::Arg::new("rt")
                .long("rt")
                .action(clap::ArgAction::SetTrue)
                .help("Generate runtime files (startup, linker script)"),
        )
        .get_matches();

    let svd_path: PathBuf = matches.get_one::<String>("svd_file").unwrap().into();
    let out_dir: PathBuf = matches.get_one::<String>("out_dir").unwrap().into();

    let generate_all =
        !(matches.get_flag("pac") || matches.get_flag("hal") || matches.get_flag("rt"));
    let gen_pac = generate_all || matches.get_flag("pac");
    let gen_hal = generate_all || matches.get_flag("hal");
    let gen_rt = generate_all || matches.get_flag("rt");

    println!("Parsing SVD file: {}", svd_path.display());
    let device = svdkit::parse_svd_file(&svd_path)?;
    println!("Device: {}", device.name);

    std::fs::create_dir_all(&out_dir)?;

    if gen_pac || gen_rt {
        if gen_rt {
            let files = pac::write_device_files_with_rt(&device, &out_dir)?;
            for p in &files {
                if let Some(os_name) = p.file_name() {
                    if let Some(name) = os_name.to_str() {
                        println!("  Generated: {}", name);
                    }
                }
            }
        } else {
            let path = pac::write_device_file(&device, &out_dir)?;
            if let Some(os_name) = path.file_name() {
                if let Some(name) = os_name.to_str() {
                    println!("  Generated: {}", name);
                }
            }
        }
    }

    if gen_hal {
        let path = svdkit::hal::write_device_hal_file(&device, &out_dir)?;
        if let Some(os_name) = path.file_name() {
            if let Some(name) = os_name.to_str() {
                println!("  Generated: {}", name);
            }
        }
    }

    println!("Output directory: {}", out_dir.display());
    Ok(())
}
