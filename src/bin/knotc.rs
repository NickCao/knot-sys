#![feature(concat_idents)]
#![allow(non_upper_case_globals)]
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
        let cmd = CString::new("zone-status").unwrap();
        data[knot_ctl_idx_t_KNOT_CTL_IDX_CMD as usize] = cmd.as_ptr();
        let flags = CString::new("F").unwrap();
        data[knot_ctl_idx_t_KNOT_CTL_IDX_FLAGS as usize] = flags.as_ptr();
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

        loop {
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
            match data_type {
                knot_ctl_type_t_KNOT_CTL_TYPE_BLOCK => break,
                knot_ctl_type_t_KNOT_CTL_TYPE_EXTRA => {
                    let r#type = CStr::from_ptr(data[knot_ctl_idx_t_KNOT_CTL_IDX_TYPE as usize])
                        .to_str()
                        .unwrap();
                    let data = CStr::from_ptr(data[knot_ctl_idx_t_KNOT_CTL_IDX_DATA as usize])
                        .to_str()
                        .unwrap();
                    print!(" | {type}: {data}");
                }
                knot_ctl_type_t_KNOT_CTL_TYPE_DATA => {
                    let zone = CStr::from_ptr(data[knot_ctl_idx_t_KNOT_CTL_IDX_ZONE as usize])
                        .to_str()
                        .unwrap();
                    let r#type = CStr::from_ptr(data[knot_ctl_idx_t_KNOT_CTL_IDX_TYPE as usize])
                        .to_str()
                        .unwrap();
                    let data = CStr::from_ptr(data[knot_ctl_idx_t_KNOT_CTL_IDX_DATA as usize])
                        .to_str()
                        .unwrap();
                    print!("\n[{zone}] {type}: {data}");
                }
                _ => unimplemented!(),
            }
        }

        knot_ctl_close(ctx);
        knot_ctl_free(ctx);
    }
}
