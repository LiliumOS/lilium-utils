#![no_std]
#![no_main]

use core::ffi::{c_char, c_int, c_void, CStr};

use lilium_sys::result::Error as LiliumError;
use lilium_sys::sys::fs::{
    FileOpenOptions, OpenFile, ACCESS_CREATE, ACCESS_TRUNCATE, ACCESS_WRITE, OP_STREAM_DEFAULT,
};
use lilium_sys::sys::handle::HandlePtr;
use lilium_sys::sys::io::{IOHandle, IORead, IOWrite, __HANDLE_IO_STDIN, __HANDLE_IO_STDOUT};
use lilium_sys::sys::kstr::KStrCPtr;

const BUFFER_SIZE: usize = 4096;

fn write_all(hdl: HandlePtr<IOHandle>, mut buf_ptr: *const c_void, mut len: usize) {
    while len > 0 {
        match unsafe { IOWrite(hdl, buf_ptr, len.try_into().unwrap()) } {
            x @ ..=0 => LiliumError::from_code(x).unwrap(),
            x => {
                len -= usize::try_from(x).unwrap();
                if len > 0 {
                    buf_ptr = unsafe { buf_ptr.add(x.try_into().unwrap()) };
                }
            }
        }
    }
}

#[no_mangle]
extern "C" fn main(argc: c_int, argv: *const *const c_char) {
    // TODO: handle POSIX-style flags correctly
    let args = unsafe { core::slice::from_raw_parts(argv, argc.try_into().unwrap()) };
    let _name = unsafe { CStr::from_ptr(args[0]) };
    let file_name = unsafe { CStr::from_ptr(args[1]) };
    let file_name = KStrCPtr {
        str_ptr: file_name.as_ptr(),
        len: file_name.to_bytes().len().try_into().unwrap(),
    };
    let mut file = HandlePtr::null();
    LiliumError::from_code(unsafe {
        OpenFile(
            &mut file,
            HandlePtr::null(),
            file_name,
            &FileOpenOptions {
                stream_override: KStrCPtr::empty(),
                access_mode: ACCESS_WRITE | ACCESS_CREATE | ACCESS_TRUNCATE,
                op_mode: OP_STREAM_DEFAULT,
                blocking_mode: OP_STREAM_DEFAULT,
                create_acl: HandlePtr::null(),
            },
        )
    })
    .unwrap();
    let stdin = unsafe { __HANDLE_IO_STDIN };
    let stdout = unsafe { __HANDLE_IO_STDOUT };
    let mut buf = [0u8; BUFFER_SIZE];
    let buf_ptr = (&mut buf as *mut u8).cast();
    loop {
        match unsafe { IORead(stdin, buf_ptr, BUFFER_SIZE.try_into().unwrap()) } {
            x @ ..=-1 => LiliumError::from_code(x).unwrap(),
            0 => break,
            x => {
                write_all(stdout, buf_ptr, x.try_into().unwrap());
                write_all(file.cast(), buf_ptr, x.try_into().unwrap());
            }
        }
    }
}
