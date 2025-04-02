#![no_std]
#![no_main]

#[unsafe(no_mangle)]
fn _start(data: kernel::KernelData) -> ! {
	// match system_table.exit_boot_services(image_handle, memmap.mapkey) {
	// 	Ok(runtime) => {
	// 	},
	// 	Err((table, e)) => {
	// 		let (pool, _, _, _) = memmap.data();
	// 		table.boot_services().free_pool(pool)?;
	// 		kernelfile.close()?;
	// 		rootfs.close()?;
	// 		Err(e)
	// 	},
	// }
	// let system_table = match data.systemtable.exit_boot_services(data.imagehandle, data.memorymap.mapkey) {
	// 	Ok(runtime) => runtime,
	// 	Err(_) => panic!(),
	// };
	let buffer = unsafe { core::slice::from_raw_parts_mut(data.graphics, data.graphicslen) };
	let mask = uefi::protocols::graphics::PixelBitmask::new(0x000000FF, 0x0000FF00, 0x00FF0000, 0xFF000000);
	let white = uefi::protocols::graphics::GraphicsOutputProtocol::grapics_color(0xFFFFFFFF, &mask);
	for pixel in buffer {
		*pixel = white;
	}
	loop {}
}

#[panic_handler]
#[allow(unused_variables)]
fn panic(info: &core::panic::PanicInfo) -> ! {
	loop {}
}
