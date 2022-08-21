#![allow(non_upper_case_globals)]

use knot_sys::*;
use std::collections::HashMap;
use std::ffi::CString;

fn parse(value: &str) -> i64 {
    if let Ok(value) = value.parse() {
        return value;
    }
    if let Some(value) = knot_bool_parse(value) {
        return if value { 1 } else { 0 };
    }
    if let Some(value) = knot_time_parse(value) {
        return value as i64;
    }
    -1
}

fn main() {
    let ctx = KnotCtx::new();
    ctx.connect("/run/knot/knot.sock").unwrap();
    ctx.send(
        KnotCtlType::DATA,
        Some(&KnotCtlData::from([(
            KnotCtlIdx::CMD,
            CString::new("zone-status").unwrap(),
        )])),
    )
    .unwrap();
    ctx.send(KnotCtlType::BLOCK, None).unwrap();

    let registry = prometheus::Registry::new();

    loop {
        let (data_type, data) = ctx.recv().unwrap();

        match data_type {
            KnotCtlType::BLOCK => break,
            KnotCtlType::DATA | KnotCtlType::EXTRA => {
                let zone = data
                    .get(&KnotCtlIdx::ZONE)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
                let label = data
                    .get(&KnotCtlIdx::TYPE)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
                let value = data
                    .get(&KnotCtlIdx::DATA)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();

                let (name, value) = match label.as_str() {
                    "serial" => ("serial", parse(&value)),
                    "refresh" => ("refresh", parse(&value)),
                    "load" => ("load", parse(&value)),
                    "update" => ("update", parse(&value)),
                    "notify" => ("notify", parse(&value)),
                    "journal flush" => ("journal_flush", parse(&value)),
                    "DNSSEC re-sign" => ("dnssec_resign", parse(&value)),
                    "backup/restore" => ("backup_restore", parse(&value)),
                    "expiration" => ("expiration", parse(&value)),
                    "NSEC3 resalt" => ("nsec3_resalt", parse(&value)),
                    "DS check" => ("ds_check", parse(&value)),
                    "DS push" => ("ds_push", parse(&value)),
                    "XFR freeze" => ("xfr_freeze", parse(&value)),
                    "freeze" => ("freeze", parse(&value)),
                    "transaction" => ("transaction", parse(&value)),
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
