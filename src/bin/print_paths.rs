fn main() {
	println!("bootloader at {}", env!("BOOTLOADER_PATH"));
	println!("kernel at {}", env!("KERNEL_PATH"));
	println!("UEFI disk image at {}", env!("IMAGE_PATH"));
}
