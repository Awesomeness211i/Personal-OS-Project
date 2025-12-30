fn main() {
	let out_dir = std::env::var("OUT_DIR").expect("Couldn't find OUT_DIR variable?");
	let output_path = std::path::PathBuf::from(out_dir);
	let kernel_path = std::path::PathBuf::from(std::env::var("CARGO_BIN_FILE_KERNEL").unwrap());
	let bootloader_path = std::path::PathBuf::from(std::env::var("CARGO_BIN_FILE_BOOTLOADER").unwrap());

	let uefi_path = output_path.join("uefi.img");
	let _dd1 = std::process::Command::new("dd")
		.arg("if=/dev/zero")
		.arg(format!("of={}", uefi_path.display()))
		.arg(format!("bs={}", 1024)) // block size in bytes
		.arg(format!("count={}", 46875)) // in block size
		.status()
		.unwrap();

	let partition_image = output_path.join("part.img");
	let _dd2 = std::process::Command::new("dd")
		.arg("if=/dev/zero")
		.arg(format!("of={}", partition_image.display()))
		.arg(format!("bs={}", 512)) // block size in bytes
		.arg(format!("count={}", 91669)) // in block size
		.status()
		.unwrap();

	// create gpt headers and efi partition
	let _parted1 = std::process::Command::new("parted")
		.arg(uefi_path.as_path())
		.arg("-s") // never prompt
		.arg("-a")
		.arg("minimal") // align minimal
		.arg("mklabel")
		.arg("gpt") // make gpt partition table
		.status()
		.unwrap();

	let _parted2 = std::process::Command::new("parted")
		.arg(uefi_path.as_path())
		.arg("-s") // never prompt
		.arg("-a")
		.arg("minimal") // align minimal
		.arg("mkpart")
		.arg("EFI")
		.arg("FAT32")
		.arg(format!("{}s", 2048)) // start
		.arg(format!("{}s", 93716)) // end
		.status()
		.unwrap();

	let _parted3 = std::process::Command::new("parted")
		.arg(uefi_path.as_path())
		.arg("-s") // never prompt
		.arg("-a")
		.arg("minimal") // align minimal
		.arg("toggle")
		.arg("1")
		.arg("boot")
		.status()
		.unwrap();

	// format and bring together our images
	let _mformat = std::process::Command::new("mformat")
		.arg("-i")
		.arg(partition_image.as_path())
		.arg("-h")
		.arg(format!("{}", 32)) // head number
		.arg("-t")
		.arg(format!("{}", 32)) // track number
		.arg("-n")
		.arg(format!("{}", 64))
		.arg("-c")
		.arg(format!("{}", 1)) // cluster size in sectors
		.status()
		.unwrap();

	// Create directories in the image
	let _mmd = std::process::Command::new("mmd")
		.arg("-i")
		.arg(partition_image.as_path())
		.arg("::/efi")
		.arg("::/efi/boot")
		.arg("::/kernel")
		.status()
		.unwrap();

	let _mcopy = std::process::Command::new("mcopy")
		.arg("-i")
		.arg(partition_image.as_path())
		.arg(bootloader_path.as_path())
		.arg("::/efi/boot/bootx64.efi")
		.status()
		.unwrap();

	let _mcopy2 = std::process::Command::new("mcopy")
		.arg("-i")
		.arg(partition_image.as_path())
		.arg(kernel_path.as_path())
		.arg("::/kernel/kernel")
		.status()
		.unwrap();

	let _dd3 = std::process::Command::new("dd")
		.arg(format!("if={}", partition_image.display())) // input file
		.arg(format!("of={}", uefi_path.display())) // output file
		.arg(format!("bs={}", 512)) // in bytes
		.arg(format!("count={}", 91669)) // in block size
		.arg(format!("seek={}", 2048)) // start
		.arg("conv=notrunc")
		.status()
		.unwrap();

	println!("cargo::rustc-env=IMAGE_PATH={}", uefi_path.display());
}
