#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(improper_ctypes)]
#![allow(non_snake_case)]
#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod test {
    #[test]
    fn knot_ctl() {
        unsafe {
            let ctl = super::knot_ctl_alloc();
            super::knot_ctl_set_timeout(ctl, 1000);
            super::knot_ctl_free(ctl);
        }
    }
}
