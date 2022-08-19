#![feature(concat_idents)]
use knot_sys::*;
use std::ffi::{CStr, CString};

fn main() {
    unsafe {
        let ctx = knot_ctl_alloc();

        let path = CString::new("/run/knot/knot.sock").unwrap();
        let code = knot_ctl_connect(ctx, path.as_ptr());
        if code < 0 {
            eprintln!("{:?}", CStr::from_ptr(knot_strerror(code)));
        }

        let mut data: knot_ctl_data_t = std::mem::zeroed();
        let cmd = CString::new("stats").unwrap();
        data[knot_ctl_idx_t_KNOT_CTL_IDX_CMD as usize] = cmd.as_ptr();
        let code = knot_ctl_send(
            ctx,
            knot_ctl_type_t_KNOT_CTL_TYPE_DATA,
            &mut data as &mut knot_ctl_data_t,
        );
        if code < 0 {
            eprintln!("1: {:?}", CStr::from_ptr(knot_strerror(code)));
        }

        let code = knot_ctl_send(
            ctx,
            knot_ctl_type_t_KNOT_CTL_TYPE_BLOCK,
            0 as *mut knot_ctl_data_t,
        );
        if code < 0 {
            eprintln!("2: {:?}", CStr::from_ptr(knot_strerror(code)));
        }

        let mut data: knot_ctl_data_t = std::mem::zeroed();
        let mut data_type: knot_ctl_type_t = std::mem::zeroed();
        let code = knot_ctl_receive(
            ctx,
            &mut data_type,
            data.as_mut_ptr() as *mut knot_ctl_data_t,
        );
        if code < 0 {
            eprintln!("3: {:?}", CStr::from_ptr(knot_strerror(code)));
        }

        knot_ctl_close(ctx);
        knot_ctl_free(ctx);
    }
}
