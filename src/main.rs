#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        hlt();
    }
}

use bootloader_api::config::Mapping;
use writer::FrameBufferWriter;
use x86_64::instructions::{hlt, interrupts};
use spin::Mutex;
use core::arch::asm;
mod interruptsa;
// Use the entry_point macro to register the entry point function: bootloader_api::entry_point!(kernel_main)

// Optionally pass a custom config

pub static BOOTLOADER_CONFIG: bootloader_api::BootloaderConfig = {
    let mut config = bootloader_api::BootloaderConfig::new_default();

    config.mappings.physical_memory = Some(Mapping::Dynamic);

    config.kernel_stack_size = 100 * 1024; // 100 KiB

    config
};

use core::fmt::{Arguments, Write};
mod writer;

bootloader_api::entry_point!(my_entry_point, config = &BOOTLOADER_CONFIG);

static FRAME_BUFFER_WRITER: Mutex<Option<FrameBufferWriter>> = Mutex::new(None);

fn my_entry_point(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    let frame_buffer_info = boot_info.framebuffer.as_mut().unwrap().info();
    let buffer = boot_info.framebuffer.as_mut().unwrap().buffer_mut();

    let mut frame_buffer_writer = FrameBufferWriter::new(buffer, frame_buffer_info);

    // Set the cursor position to the top-left corner
    frame_buffer_writer.set_cursor(1, 3);
    interruptsa::init();
    *FRAME_BUFFER_WRITER.lock() = Some(frame_buffer_writer);
     print!("The print macro is working corrrectly in the defined position");
    
    loop {

        hlt(); // Stop x86_64 from being unnecessarily busy while looping
    }
}

#[doc(hidden)]
pub fn printx(args: Arguments) {
    use core::fmt::Write;
    if let Some(writer) = &mut *FRAME_BUFFER_WRITER.lock() {
        writer.write_fmt(args).unwrap();
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::printx(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! input_char {
    () => {{
        let result: u8;
        unsafe {
            asm!(
                "xor eax, eax",
                "in al, 0x60",
                lateout("al") result,
            );
        }
        result
    }};
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ({
        $crate::print!("{}\n", core::format_args!($($arg)*));
    })
}