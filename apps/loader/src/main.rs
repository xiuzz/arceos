#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]
#![feature(asm_const)]



#[cfg(feature = "axstd")]
use axstd::println;
#[cfg(feature = "axstd")]
use axstd::process::exit;

const PLASH_START: usize = 0x22000000;
const SYS_HELLO: usize = 1;
const SYS_PUTCHAR: usize = 2;
const SYS_TERMINATE: usize = 3;
static mut ABI_TABLE: [usize; 16] = [0; 16];

fn register_abi(num: usize, handle: usize) {
    unsafe { ABI_TABLE[num] = handle; }
}

fn abi_hello() {
    println!("[ABI:Hello] Hello, Apps!");
}

fn abi_putchar(c: char) {
    println!("[ABI:Print] {c}");
}

fn abi_terminate(exit_code: i32) {
    println!("[ABI:terminate] exit the terminal!");
    exit(exit_code);
}


struct ImageHeader{
    ptr_len: usize
}

struct AppHeader {
    start: usize,
    size: usize,
    content: &'static [u8],
}

impl AppHeader {
    pub fn new(start: usize, size: usize, content: &'static [u8]) -> Self {
        Self {
            start,
            size,
            content,
        }
    }
}

impl ImageHeader {

    pub fn new(ptr_len: usize) -> Self {
        Self {
            ptr_len
        }
    }

    #[inline]
    pub fn load_app_nums(&self, image_start:usize) -> usize {
        let app_size = self.read_bytes(image_start, self.ptr_len);
        self.bytes_to_usize(app_size)
    }

    #[inline]
    pub fn load_app(&self, mut app_start :usize) -> AppHeader{
        let tmp = self.read_bytes(app_start, self.ptr_len);
        let app_size = self.bytes_to_usize(tmp);
        app_start += self.ptr_len;
        AppHeader::new(app_start, app_size, self.read_bytes(app_start, app_size))
    }

    #[inline]
    fn read_bytes(&self, ptr: usize, ptr_len: usize) -> &'static [u8] {
        unsafe { core::slice::from_raw_parts(ptr as *const u8, ptr_len) }
    }

    #[inline]
    fn bytes_to_usize(&self, binary: &[u8]) -> usize {  
        let high_byte = binary[0] as usize;
        let low_byte = binary[1] as usize;
        (high_byte << 8) | low_byte
    }
}


#[cfg_attr(feature = "axstd", no_mangle)]
fn main() {
    //image header
    let ptr_len = 2;
    let image_header  = ImageHeader::new(ptr_len);
    
    let app_nums: usize= image_header.load_app_nums(PLASH_START);

    println!("the app nums : {}", app_nums);

    // switch aspace from kernel to app
    unsafe { init_app_page_table(); }
    unsafe { switch_app_aspace(); }
    //app x
    const RUN_START : usize = 0x4010_0000;
    let mut app_start = PLASH_START + ptr_len;
    for i in 0..app_nums {
        let app_header = image_header.load_app(app_start);
        println!("start:{:#}",app_header.start);
        println!("size:{}", app_header.size);
        // println!("context:{:?}",app_header.content);
        app_start += app_header.size + ptr_len;
        let run_code = unsafe {
            core::slice::from_raw_parts_mut(RUN_START as *mut u8, app_header.size)
        };
        run_code.copy_from_slice(app_header.content);
        // println!("run code {:?}; address [{:?}]", run_code, run_code.as_ptr());

        println!("App:{}",i);

        register_abi(SYS_HELLO, abi_hello as usize);
        register_abi(SYS_PUTCHAR, abi_putchar as usize);
        register_abi(SYS_TERMINATE, abi_terminate as usize);
    
        let arg0: u8 = b'A';
        // execute app
        unsafe { core::arch::asm!("
            la      a7, {abi_table}
            li      t2, {run_start}
            jalr    t2",
            run_start = const RUN_START,
            abi_table = sym ABI_TABLE,
        )}
        println!("App {} fininshed..........",i);
        println!("......................................");
    }    
}

//
// App aspace
//
#[link_section = ".data.app_page_table"]
static mut APP_PT_SV39: [u64; 512] = [0; 512];

unsafe fn init_app_page_table() {
    // 0x8000_0000..0xc000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[2] = (0x80000 << 10) | 0xef;
    // 0xffff_ffc0_8000_0000..0xffff_ffc0_c000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[0x102] = (0x80000 << 10) | 0xef;

    // 0x0000_0000..0x4000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[0] = (0x00000 << 10) | 0xef;

    // For App aspace!
    // 0x4000_0000..0x8000_0000, VRWX_GAD, 1G block
    APP_PT_SV39[1] = (0x80000 << 10) | 0xef;
}

unsafe fn switch_app_aspace() {
    use riscv::register::satp;
    let page_table_root = APP_PT_SV39.as_ptr() as usize - axconfig::PHYS_VIRT_OFFSET;
    satp::set(satp::Mode::Sv39, 0, page_table_root >> 12);
    riscv::asm::sfence_vma_all();
}
