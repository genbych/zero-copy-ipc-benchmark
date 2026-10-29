use std::path::Path;
use xshell::{cmd, Shell};

static KERNEL_PATH: &str = "target/x86_64-unknown-none/release/kernel";
static ISO_DIR: &str = "target/iso_root";
static ISO_PATH: &str = "target/baremetal.iso";
static LIMINE_DIR: &str = "target/limine";
static TARBALL_PATH: &str = "target/limine.tar.gz";
static MANIFEST: &str = env!("CARGO_MANIFEST_DIR");

macro_rules! log {
    ($msg:expr) => {
        println!("\x1b[1;32m[xtask]\x1b[0m {}", $msg);
    };

    ($fmt:expr, $($arg:tt)*) => {
        print!("\x1b[1;32m[xtask]\x1b[0m ");
        println!($fmt, $($arg)*);
    };
}
macro_rules! sub_log {
    ($fmt:expr, $($arg:tt)*) => {
        print!("{:>10}↳ ", "");
        println!($fmt, $($arg)*);
    };
    ($msg:expr) => {
        println!("{:>10}↳ {}", "", $msg);
    };
}

fn main() -> Result<(), xshell::Error> {
    let sh = Shell::new()?;

    sh.change_dir(Path::new(MANIFEST).parent().unwrap());

    log!("Current workspace root: {:?}", sh.current_dir());
    log!("Building Kernel.");
    cmd!(
        sh,
        "cargo build --package kernel --target x86_64-unknown-none --release"
    )
    .run()?;

    log!("Preparing ISO directory");
    sub_log!("Creating directories");

    let _ = sh.remove_path(ISO_DIR);
    sh.create_dir(format!("{}/boot", ISO_DIR))?;
    sh.create_dir(format!("{}/EFI/BOOT", ISO_DIR))?;

    sub_log!("Copying files");
    sh.copy_file(KERNEL_PATH, format!("{}/boot/kernel.elf", ISO_DIR))?;
    sh.copy_file("limine.conf", format!("{}/boot/limine.conf", ISO_DIR))?;

    if !Path::new(LIMINE_DIR).join("limine-bios.sys").exists() {
        sub_log!("Limine binaries not found. Downloading");

        sh.create_dir(LIMINE_DIR)?;
        cmd!(sh, "curl -L -o {TARBALL_PATH} https://github.com/limine-bootloader/limine/releases/latest/download/limine-binary.tar.gz").run()?;

        sub_log!("Extracting Limine archive");
        cmd!(
            sh,
            "tar -xzf {TARBALL_PATH} -C {LIMINE_DIR} --strip-components=1"
        )
        .run()?;

        let _ = sh.remove_path(TARBALL_PATH);

        log!("Building Limine binary");
        cmd!(sh, "make -C {LIMINE_DIR}").run()?;
    }

    sh.copy_file(
        format!("{}/limine-bios.sys", LIMINE_DIR),
        format!("{}/boot/limine-bios.sys", ISO_DIR),
    )?;
    sh.copy_file(
        format!("{}/limine-bios-cd.bin", LIMINE_DIR),
        format!("{}/boot/limine-bios-cd.bin", ISO_DIR),
    )?;
    sh.copy_file(
        format!("{}/limine-uefi-cd.bin", LIMINE_DIR),
        format!("{}/boot/limine-uefi-cd.bin", ISO_DIR),
    )?;
    sh.copy_file(
        format!("{}/BOOTX64.EFI", LIMINE_DIR),
        format!("{}/EFI/BOOT/BOOTX64.EFI", ISO_DIR),
    )?;

    log!("ISO directory prepared at {}", ISO_DIR);
    log!("Generating bootable image");

    cmd!(
        sh,
        "xorriso -as mkisofs
        -b boot/limine-bios-cd.bin
        -no-emul-boot -boot-load-size 4 -boot-info-table
        --efi-boot boot/limine-uefi-cd.bin
        -efi-boot-part --efi-boot-image --protective-msdos-label
        {ISO_DIR} -o {ISO_PATH}"
    )
    .quiet()
    .run()?;

    log!("Embedding the BIOS bootloader into the image");

    cmd!(sh, "target/limine/limine bios-install {ISO_PATH}")
        .quiet()
        .run()?;
    log!("Image created at {}!", ISO_PATH);

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "run" {
        log!("Running QEMU");
        cmd!(sh, "qemu-system-x86_64 -cdrom {ISO_PATH} -serial stdio")
            .quiet()
            .run()?;
    }

    Ok(())
}
