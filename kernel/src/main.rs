#![no_std]
#![no_main]
#![feature(naked_functions)]

extern crate alloc;

mod drivers;
mod filesystem;
mod interrupts;
mod memory;
mod multitasking;
mod shell;
mod syscalls;

use core::arch::asm;
use core::panic::PanicInfo;
use drivers::disk::DISK;
use drivers::pic::PICS;
use interrupts::idt::IDT;
use memory::allocator::Allocator;
use memory::paging::{
		PAGING,
		PageDirectory
	};
use shell::shell::SHELL;
use syscalls::print::PRINTER;
use filesystem::fat::FAT;
use multitasking::task::TASK_MANAGER;

use libfelix;

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

//1MiB. TODO: Get those from linker
const KERNEL_START: u32 = 0x0010_0000;
const KERNEL_SIZE: u32 = 0x0010_0000;
const STACK_SIZE: u32 = 0x0010_0000;

const STACK_START: u32 = KERNEL_START + KERNEL_SIZE + STACK_SIZE;

const VERSION: &str = env!("CARGO_PKG_VERSION");
//const EDITION: &str = env!("CARGO_PKG_EDITION");

//KERNEL ENTRY POINT
#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
pub extern "C" fn _start() -> ! {
    unsafe {
        //setup stack
        asm!("mov esp, {}", in(reg) STACK_START);

        //setup paging
		PageDirectory::identity(&PAGING); 
        PAGING.lock().enable();

        //bochs magic breakpoint
        asm!("xchg bx, bx");

        //setup idt
		let mut idt = IDT.lock();
        idt.init(); //init idt  
        idt.add_exceptions(); //add CPU exceptions to idt 
        idt.add(
            interrupts::timer::TIMER_INT as usize,
            interrupts::timer::timer as u32,
        ); //add timer interrupt to idt     
        idt.add(
            syscalls::handler::SYSCALL_INT as usize,
            syscalls::handler::syscall as u32,
        ); //add system call handler interrupt     
        idt.add(
            drivers::keyboard::KEYBOARD_INT as usize,
            drivers::keyboard::keyboard as u32,
        ); //add keyboard interrupt to idt   
        idt.load(); //load idt

        //init programmable interrupt controllers
        PICS.init();

        //enable ata disk if present
        DISK.lock().check();

        //init filesystem
        if DISK.lock().enabled {
            let mut fat = FAT.lock();
            fat.load_header();
            fat.load_table();
            fat.load_entries();
        }

        //print name, version and copyright
        print_info();

        //init felix shell
        SHELL.lock().init();

        //init multitasking
        TASK_MANAGER.lock().init();

        //bochs magic breakpoint
        asm!("xchg bx, bx");

        //enable hardware interrupts
        asm!("sti");

        loop {}
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    libfelix::println!("PANIC! Info: {}", info);

    loop {}
}

fn print_info() {

    PRINTER.lock().set_colors(0xf, 0);

    libfelix::println!();
    libfelix::println!("FELIX {}", VERSION);
	libfelix::println!("RUST edition {}", 2024);
    libfelix::println!("Copyright (c) 2023-2025 Gianmatteo Palmieri");
    libfelix::println!();
    libfelix::println!("Type \"help\" and press enter to show a list of available commands");
    libfelix::println!();

    PRINTER.lock().reset_colors();
}
