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
        knot_ctl_close(ctx);
        knot_ctl_free(ctx);
    }
}
