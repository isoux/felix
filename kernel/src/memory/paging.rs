use core::arch::asm;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
	pub static ref PAGING: Mutex<PageDirectory> = Mutex::new(PageDirectory {
    	//0b010 (supervisor, write, not present)
    	entries: [0x00000002; 1024],
	});
}

lazy_static! {
	pub static ref TABLES: Mutex<[PageTable; 16]> = Mutex::new([NULL_TABLE; 16]);
}

pub static NULL_TABLE: PageTable = PageTable { entries: [0; 1024] };

#[repr(align(4096))]
pub struct PageDirectory {
    pub entries: [u32; 1024],
}


impl PageDirectory {
    pub fn set_table(&mut self, index: usize, table: &PageTable) {
        self.entries[index] = (table as *const PageTable) as u32 | 0b011;
    }

    pub fn enable(&self) {
        unsafe {
            let address = (self as *const PageDirectory) as u32;

            asm!("mov cr3, eax",
                "mov eax, cr0",
                "or eax, 0x80000001",
                "mov cr0, eax",
                in("eax") address);
        }
    }

    //indentity page first 32MiB
    pub fn identity(page_dir: &Mutex<PageDirectory>) {

		let mut tables = TABLES.lock();
		let mut paging = page_dir.lock();
        for i in 0..8 {
			tables[i].set((0x0040_0000 * i) as u32);
			paging.set_table(i, &tables[i]);
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(align(4096))]
pub struct PageTable {
    pub entries: [u32; 1024],
}

impl PageTable {
    pub fn set(&mut self, from: u32) {
        for i in 0..1024 {
            //0b011 (supervisor, write, present)
            self.entries[i] = (((i * 0x1000) + from as usize) | 0b011) as u32;
        }
    }
}
