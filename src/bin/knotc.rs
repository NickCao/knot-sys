#![feature(concat_idents)]
#![allow(non_upper_case_globals)]
use knot_sys::*;
use std::collections::HashMap;
use std::ffi::{CStr, CString};

fn normalize_name(name: &str) -> String {
    name.to_lowercase()
        .replace(" ", "_")
        .replace("/", "_")
        .replace("-", "")
}

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

        let registry = prometheus::Registry::new();

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
                knot_ctl_type_t_KNOT_CTL_TYPE_DATA | knot_ctl_type_t_KNOT_CTL_TYPE_EXTRA => {
                    let zone = data[knot_ctl_idx_t_KNOT_CTL_IDX_ZONE as usize];
                    let label = data[knot_ctl_idx_t_KNOT_CTL_IDX_TYPE as usize];
                    let value = data[knot_ctl_idx_t_KNOT_CTL_IDX_DATA as usize];
                    let zone = CStr::from_ptr(zone).to_str().unwrap().to_owned();
                    let label = CStr::from_ptr(label).to_str().unwrap().to_owned();
                    let value = CStr::from_ptr(value).to_str().unwrap().to_owned();
                    let mut labels = HashMap::new();
                    let value = match label.as_str() {
                        "serial" => value.parse::<f64>().unwrap(),
                        _ => 0.0,
                    };
                    labels.insert("zone".to_string(), zone);
                    let gauge = prometheus::Gauge::with_opts(prometheus::Opts {
                        namespace: "knot".to_string(),
                        subsystem: "knot".to_string(),
                        name: normalize_name(&label),
                        help: label,
                        const_labels: labels,
                        variable_labels: vec![],
                    })
                    .unwrap();
                    gauge.set(value);
                    registry.register(Box::new(gauge)).unwrap();
                }
                _ => unimplemented!(),
            }
        }

        let mut buffer = String::new();
        let encoder = prometheus::TextEncoder::new();
        let metric_families = registry.gather();
        encoder.encode_utf8(&metric_families, &mut buffer).unwrap();
        println!("{}", buffer);

        knot_ctl_close(ctx);
        knot_ctl_free(ctx);
    }
}
