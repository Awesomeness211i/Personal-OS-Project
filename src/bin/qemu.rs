fn main() -> Result<(), std::io::Error> {
	let uefi_path = env!("IMAGE_PATH");
	#[cfg(target_arch = "x86_64")]
	let exit_status = std::process::Command::new("qemu-system-x86_64")
		.arg("-drive")
		.arg(format!(
			"if=pflash,format=raw,unit=0,file={},readonly=on",
			std::env::current_dir().unwrap().join("firmware/x64/code.fd").display()
		))
		.arg("-drive")
		.arg(format!("if=pflash,format=raw,unit=1,file={}", std::env::current_dir().unwrap().join("firmware/x64/vars.fd").display()))
		.arg("-net")
		.arg("none")
		.arg("-drive")
		.arg(format!("format=raw,file={uefi_path}"))
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
