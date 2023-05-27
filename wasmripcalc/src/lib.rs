mod io_interop;


use libripcalc::cmds::derange::derange;
use libripcalc::cmds::enumerate::enumerate;
use libripcalc::cmds::minimize::minimize;
use libripcalc::cmds::resize::resize;
use libripcalc::cmds::show_net::show_net;

use crate::io_interop::{HtmlWasmStdout, WasmStderr, write_to_error};


static mut BUFFER_SIZE: usize = 0;
static mut U16_BUFFER: [u16; 1024] = [0; 1024];


#[no_mangle]
pub extern "C" fn ripcalc_get_buffer_size_offset() -> *mut usize {
    unsafe { (&mut BUFFER_SIZE) as *mut usize }
}
#[no_mangle]
pub extern "C" fn ripcalc_get_buffer_offset() -> *mut u16 {
    unsafe { (&mut U16_BUFFER[0]) as *mut u16 }
}


fn get_buffer_slice() -> &'static [u16] {
    unsafe { &U16_BUFFER[0..BUFFER_SIZE] }
}


#[no_mangle]
pub extern "C" fn ripcalc_show_net() {
    let net_utf16_slice = get_buffer_slice();
    let net_str = match String::from_utf16(net_utf16_slice) {
        Ok(ns) => ns,
        Err(_) => {
            write_to_error("Failed to decode network.");
            return;
        },
    };

    let mut stdout = HtmlWasmStdout;
    let mut stderr = WasmStderr;

    show_net(&["ripcalc", &net_str], &mut stdout, &mut stderr);
}


#[no_mangle]
pub extern "C" fn ripcalc_minimize() {
    let nets_utf16_slice = get_buffer_slice();
    let nets_str = match String::from_utf16(nets_utf16_slice) {
        Ok(ns) => ns,
        Err(_) => {
            write_to_error("Failed to decode networks.");
            return;
        },
    };

    // prepend a few fixed arguments
    let args: Vec<&str> = std::iter::once("ripcalc")
        .chain(std::iter::once("--minimize"))
        .chain(nets_str.split("\n"))
        .collect();

    let mut stdout = HtmlWasmStdout;
    let mut stderr = WasmStderr;

    minimize(&args, &mut stdout, &mut stderr);
}


#[no_mangle]
pub extern "C" fn ripcalc_derange() {
    let range_utf16_slice = get_buffer_slice();
    let range_str = match String::from_utf16(range_utf16_slice) {
        Ok(ns) => ns,
        Err(_) => {
            write_to_error("Failed to decode range.");
            return;
        },
    };

    let (range_start, range_end) = range_str.split_once(' ').unwrap();

    let mut stdout = HtmlWasmStdout;
    let mut stderr = WasmStderr;

    derange(&["ripcalc", "--derange", range_start, range_end],  &mut stdout, &mut stderr);
}


#[no_mangle]
pub extern "C" fn ripcalc_resize() {
    let input_utf16_slice = get_buffer_slice();
    let input_str = match String::from_utf16(input_utf16_slice) {
        Ok(ns) => ns,
        Err(_) => {
            write_to_error("Failed to decode range.");
            return;
        },
    };

    let (existing_net, subnet) = input_str.split_once(' ').unwrap();

    let mut stdout = HtmlWasmStdout;
    let mut stderr = WasmStderr;
    resize(&["ripcalc", "--resize", existing_net, subnet], &mut stdout, &mut stderr);
}

#[no_mangle]
pub extern "C" fn ripcalc_enumerate() {
    let input_utf16_slice = get_buffer_slice();
    let input_str = match String::from_utf16(input_utf16_slice) {
        Ok(ns) => ns,
        Err(_) => {
            write_to_error("Failed to decode range.");
            return;
        },
    };

    let mut stdout = HtmlWasmStdout;
    let mut stderr = WasmStderr;
    enumerate(&["ripcalc", "--enumerate", &input_str], &mut stdout, &mut stderr);
}
