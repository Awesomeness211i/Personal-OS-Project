fn main() -> Result<(), std::io::Error> {
	let output_path = std::path::PathBuf::from(env!("OUT_DIR"));
	let uefi_path = env!("IMAGE_PATH");
	let kernel_path = env!("KERNEL_PATH");
	let bootloader_path = env!("BOOTLOADER_PATH");
	let partition_image = output_path.join("part.img");
	// create uefi and partition image
	let _dd1 = std::process::Command::new("dd")
		.arg("if=/dev/zero")
		.arg(format!("of={}", uefi_path))
		.arg(format!("bs={}", 1024)) // block size in bytes
		.arg(format!("count={}", 46875)) // in block size
		.status()?;
	let _dd2 = std::process::Command::new("dd")
		.arg("if=/dev/zero")
		.arg(format!("of={}", partition_image.display()))
		.arg(format!("bs={}", 512)) // block size in bytes
		.arg(format!("count={}", 91669)) // in block size
		.status()?;
	// create gpt headers and efi partition
	let _parted1 = std::process::Command::new("parted")
		.arg(uefi_path)
		.arg("-s") // never prompt
		.arg("-a") // align
		.arg("minimal")
		.arg("mklabel")
		.arg("gpt")
		.status()?;
	let _parted2 = std::process::Command::new("parted")
		.arg(uefi_path)
		.arg("-s") // never prompt
		.arg("-a") // align
		.arg("minimal")
		.arg("mkpart")
		.arg("EFI")
		.arg("FAT32")
		.arg(format!("{}s", 2048))
		.arg(format!("{}s", 93716))
		.status()?;
	let _parted3 = std::process::Command::new("parted")
		.arg(uefi_path)
		.arg("-s") // never prompt
		.arg("-a") // align
		.arg("minimal")
		.arg("toggle")
		.arg("1")
		.arg("boot")
		.status()?;
	// format and bring together our images
	let _mformat = std::process::Command::new("mformat")
		.arg("-i").arg(format!("{}", partition_image.display()))
		.arg("-h").arg(format!("{}", 32))
		.arg("-t").arg(format!("{}", 32))
		.arg("-n").arg(format!("{}", 64))
		.arg("-c").arg(format!("{}", 1))
		.status()?;
	let _mmd = std::process::Command::new("mmd")
		.arg("-i").arg(format!("{}", partition_image.display()))
		.arg("::/efi")
		.arg("::/efi/boot")
		.arg("::/kernel")
		.status()?;
	let _mcopy = std::process::Command::new("mcopy")
		.arg("-i").arg(format!("{}", partition_image.display()))
		.arg(bootloader_path)
		.arg("::/efi/boot/bootx64.efi")
		.status()?;
	let _mcopy2 = std::process::Command::new("mcopy")
		.arg("-i").arg(format!("{}", partition_image.display()))
		.arg(kernel_path)
		.arg("::/kernel/kernel")
		.status()?;
	let _dd3 = std::process::Command::new("dd")
		.arg(format!("if={}", partition_image.display())) // input file
		.arg(format!("of={}", uefi_path)) // output file
		.arg(format!("bs={}", 512)) // in bytes
		.arg(format!("count={}", 91669)) // in block size
		.arg(format!("seek={}", 2048))
		.arg("conv=notrunc")
		.status()?;
	// launch qemu
	#[cfg(target_arch = "x86_64")]
	let exit_status = std::process::Command::new("qemu-system-x86_64")
		.arg("-drive")
		.arg(format!("if=pflash,format=raw,unit=0,file={},readonly=on", std::env::current_dir().unwrap().join("firmware/x64/code.fd").display()))
		.arg("-drive")
		.arg(format!("if=pflash,format=raw,unit=1,file={}", std::env::current_dir().unwrap().join("firmware/x64/vars.fd").display()))
		.arg("-net").arg("none")
		.arg("-drive")
		.arg(format!("format=raw,file={}", uefi_path))
		.status()?;
	#[cfg(target_arch = "aarch64")]
	let exit_status = std::process::Command::new("qemu-system-arm")
		.arg("-drive")
		.arg(format!("format=raw,file={}", uefi_path))
		.arg("-bios")
		.arg(std::env::current_dir().unwrap().join("OVMF.fd"))
		.status()?;
	std::process::exit(exit_status.code().unwrap_or(-1));
}
