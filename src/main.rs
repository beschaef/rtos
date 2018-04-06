//! Code for the RTOS
//!
//! #![no_std] is used to disable the standard library
//! #![no_main] is added to tell the rust compiler that we don't want to use
//! the normal entry point chain. This also requires to remove the main 
//! function, because there's nothing to call the main
//!
//! to build and link the Code on Linux use
//! 'cargo rustc -- -Z pre-link-arg=-nostartfiles'
//!
//! to build and link it under macOS use
//! 'cargo rustc -- -Z pre-link-arg=-lSystem'
//!
//! to build our programm without an underlaying OS use
//! 'xargo build --target x86_64-rtos
//!

#![feature(lang_items)]
#![no_std]
#![no_main]
#![feature(const_fn)]

#[macro_use]
mod vga_buffer;

extern crate rlibc;
extern crate volatile;
#[macro_use]
extern crate lazy_static;
extern crate spin;

#[lang = "panic_fmt"]
#[no_mangle]
pub extern "C" fn rust_begin_panic(
	_msg: core::fmt::Arguments,
	_file: &'static str, 
	_line: u32, 
	_column: u32
) -> ! {
	loop {}
}

// this is the function for the entry point on Linux.
// the "-> !"" means that the function is diverging, i.e. not allowed to ever return.
// 0xb8000 is the address of the VGA buffer
#[no_mangle]
pub extern "C" fn _start() -> ! {
	println!("Hello World{}", "!" );

	loop {}
}

// this is the function for the entry point on macOS.
// the "-> !"" means that the function is diverging, i.e. not allowed to ever return.
#[no_mangle]
pub extern "C" fn main() -> ! {
    loop {}
}


