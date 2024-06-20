extern crate libc;

use libc::{c_int, c_void, size_t};
use std::ffi::CString;
use std::ptr;
use tokio::net::UdpSocket;
use std::slice;

#[link(name = "pcre2-8")]
extern "C" {
    fn pcre2_compile_8(
        pattern: *const u8,
        length: size_t,
        options: u32,
        errorcode: *mut c_int,
        erroroffset: *mut size_t,
        context: *mut c_void,
    ) -> *mut c_void;
    
    fn pcre2_match_data_create_8(length: size_t, context: *mut c_void) -> *mut c_void;
    
    fn pcre2_match_8(
        code: *const c_void,
        subject: *const u8,
        length: size_t,
        startoffset: size_t,
        options: u32,
        match_data: *mut c_void,
        context: *mut c_void,
    ) -> c_int;
    
    fn pcre2_get_ovector_pointer_8(match_data: *mut c_void) -> *const size_t;
    fn pcre2_match_data_free_8(match_data: *mut c_void);
    fn pcre2_code_free_8(code: *mut c_void);
}

#[tokio::main]
async fn main() {
    let pattern = CString::new(r"(?<=\d{4})[^\d\s]{3,11}(?=\S)").unwrap();
    //to check multi matches
    //let subject = CString::new("a;jhgoqoghqoj0329 u0tyu10hg0h9Y0Y9827342482y(Y0y(G)_)lajf;lqjfgqhgpqjopjqa=)*(^!@#$%^&*())9999999abcde").unwrap();
    //original string
    let subject = CString::new("a;jhgoqoghqoj0329 u0tyu10hg0h9Y0Y9827342482y(Y0y(G)_)lajf;lqjfgqhgpqjopjqa=)*(^!@#$%^&*())9999999").unwrap();
    
    let mut errorcode = 0;
    let mut erroroffset = 0;
    
    unsafe {
        let code = pcre2_compile_8(
            pattern.as_ptr() as *const u8,
            pattern.as_bytes().len(),
            0,
            &mut errorcode,
            &mut erroroffset,
            ptr::null_mut(),
        );
        
        if code.is_null() {
            println!("PCRE2 compilation failed with error code: {}", errorcode);
            return;
        }
        
        let match_data = pcre2_match_data_create_8(10, ptr::null_mut());
        let mut startoffset = 0;
        let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        
        loop {
            let rc = pcre2_match_8(
                code,
                subject.as_ptr() as *const u8,
                subject.as_bytes().len(),
                startoffset,
                0,
                match_data,
                ptr::null_mut(),
            );
            
            if rc <= 0 {
                break;
            }

            let ovector = pcre2_get_ovector_pointer_8(match_data);
            let ovector = slice::from_raw_parts(ovector, rc as usize * 2);
            let start = ovector[0] as usize;
            let end = ovector[1] as usize;
            let result = &subject.to_bytes()[start..end];
            
            socket.send_to(result, "127.0.0.1:12345").await.unwrap();
            println!("Match found: {:?}", String::from_utf8_lossy(result));
            
            startoffset = end;
        }
        
        pcre2_match_data_free_8(match_data);
        pcre2_code_free_8(code);
    }
}
