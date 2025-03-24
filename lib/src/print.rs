use core::arch::asm;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

pub struct Printer {}

lazy_static! {
	pub static ref PRINTER: Mutex<Printer> = Mutex::new(Printer {});
}

//core lib needs to know how to print a string to implement its print formatted func
impl fmt::Write for Printer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.prints(s);
        Ok(())
    }
}

impl Printer {
    pub fn prints(&self, s: &str) {
        unsafe {
            let ptr = s.as_ptr();
            let len = s.len();

            asm!("push eax", "push ebx","push ecx", "int 0x80", "pop ecx", "pop ebx", "pop eax", in("eax") 0, in("ebx") ptr as u32, in("ecx") len as u32);
        }
    }
}

//macro for print!
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*)));
}

//macro for println!
#[macro_export]
macro_rules! println {
    () => {
        unsafe {
            $crate::print::PRINTER.lock().prints("\n");
        }
    };


    ($($arg:tt)*) => {
        $crate::print!("{}", format_args!($($arg)*));
        unsafe {
            $crate::print::PRINTER.lock().prints("\n");
        }
    };
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    PRINTER.lock().write_fmt(args).unwrap();
}
