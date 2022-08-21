#![allow(non_upper_case_globals)]
use knot_sys::bindings::*;
use knot_sys::*;
use std::collections::HashMap;
use std::ffi::{CStr, CString};

fn parse_bool(value: &str) -> i64 {
    match knot_bool_parse(value) {
        Some(true) => 1,
        Some(false) => 0,
        None => -1,
    }
}

fn parse_event(value: &str) -> i64 {
    match knot_time_parse(value) {
        Some(time) => time as i64,
        None => -1,
    }
}

fn main() {
    unsafe {
        let ctx = KnotCtx::new();
        ctx.connect("/run/knot/knot.sock").unwrap();

        let mut data: knot_ctl_data_t = std::mem::zeroed();
        let cmd = CString::new("zone-status").unwrap();
        data[knot_ctl_idx_t_KNOT_CTL_IDX_CMD as usize] = cmd.as_ptr();
        ctx.send(knot_ctl_type_t_KNOT_CTL_TYPE_DATA, &mut data)
            .unwrap();
        ctx.send(
            knot_ctl_type_t_KNOT_CTL_TYPE_BLOCK,
            0 as *mut knot_ctl_data_t,
        )
        .unwrap();

        let registry = prometheus::Registry::new();

        loop {
            let mut data: knot_ctl_data_t = std::mem::zeroed();
            let mut data_type: knot_ctl_type_t = std::mem::zeroed();
            ctx.recv(&mut data_type, &mut data).unwrap();

            match data_type {
                knot_ctl_type_t_KNOT_CTL_TYPE_BLOCK => break,
                knot_ctl_type_t_KNOT_CTL_TYPE_DATA | knot_ctl_type_t_KNOT_CTL_TYPE_EXTRA => {
                    let zone = data[knot_ctl_idx_t_KNOT_CTL_IDX_ZONE as usize];
                    let label = data[knot_ctl_idx_t_KNOT_CTL_IDX_TYPE as usize];
                    let value = data[knot_ctl_idx_t_KNOT_CTL_IDX_DATA as usize];
                    let zone = CStr::from_ptr(zone).to_str().unwrap().to_owned();
                    let label = CStr::from_ptr(label).to_str().unwrap().to_owned();
                    let value = CStr::from_ptr(value).to_str().unwrap().to_owned();

                    let (name, value) = match label.as_str() {
                        "serial" => ("serial", value.parse::<i64>().unwrap()),
                        "refresh" => ("refresh", parse_event(&value)),
                        "load" => ("load", parse_event(&value)),
                        "update" => ("update", parse_event(&value)),
                        "notify" => ("notify", parse_event(&value)),
                        "journal flush" => ("journal_flush", parse_event(&value)),
                        "DNSSEC re-sign" => ("dnssec_resign", parse_event(&value)),
                        "backup/restore" => ("backup_restore", parse_event(&value)),
                        "expiration" => ("expiration", parse_event(&value)),
                        "NSEC3 resalt" => ("nsec3_resalt", parse_event(&value)),
                        "DS check" => ("ds_check", parse_event(&value)),
                        "DS push" => ("ds_push", parse_event(&value)),
                        "XFR freeze" => ("xfr_freeze", parse_bool(&value)),
                        "freeze" => ("freeze", parse_bool(&value)),
                        "transaction" => ("transaction", parse_bool(&value)),
                        _ => continue,
                    };

                    let mut labels = HashMap::new();
                    labels.insert("zone".to_string(), zone);

                    let gauge = prometheus::IntGauge::with_opts(prometheus::Opts {
                        namespace: "knot".to_string(),
                        subsystem: "dns".to_string(),
                        name: name.to_string(),
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
    }
}
