#![cfg_attr(feature = "axstd", no_std)]
#![cfg_attr(feature = "axstd", no_main)]

#[cfg(feature = "axstd")]
use axstd::println;

const PLASH_START: usize = 0x22000000;

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

    //app x

    let mut app_start = PLASH_START + ptr_len;
    for _ in 0..app_nums {
        let app_header = image_header.load_app(app_start);
        println!("start:{:#}",app_header.start);
        println!("size:{}", app_header.size);
        println!("context:{:?}",app_header.content);
        println!("......................................");
        app_start += app_header.size + ptr_len;
    }
}

