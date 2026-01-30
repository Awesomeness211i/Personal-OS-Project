fn main() -> Result<(), std::io::Error> {
	let uefi_path = env!("IMAGE_PATH");
	let current_directory = std::env::current_dir()?;

	#[cfg(target_arch = "x86_64")]
	let exit_status = std::process::Command::new("qemu-system-x86_64")
		.arg("-drive")
		.arg(format!("if=pflash,format=raw,unit=0,file={},readonly=on", current_directory.join("FV/code.fd").display()))
		.arg("-drive")
		.arg(format!("if=pflash,format=raw,unit=1,file={}", current_directory.join("FV/OVMF_VARS.fd").display()))
		.arg("-net")
		.arg("none")
		.arg("-drive")
		.arg(format!("format=raw,file={uefi_path}"))
		.arg("-serial")
		.arg("stdio")
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
