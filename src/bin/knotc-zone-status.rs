#![allow(non_upper_case_globals)]

use knot_sys::*;
use std::collections::HashMap;
use std::ffi::CString;

fn normalize(key: &str) -> String {
    key.to_lowercase()
        .replace(" ", "_")
        .replace("/", "_")
        .replace("-", "_")
}

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
        let (data_type, mut data) = ctx.recv().unwrap();

        match data_type {
            KnotCtlType::BLOCK => break,
            KnotCtlType::DATA | KnotCtlType::EXTRA => {
                let zone = data
                    .remove(&KnotCtlIdx::ZONE)
                    .unwrap()
                    .into_string()
                    .unwrap();
                let label = data
                    .remove(&KnotCtlIdx::TYPE)
                    .unwrap()
                    .into_string()
                    .unwrap();
                let value = data
                    .remove(&KnotCtlIdx::DATA)
                    .unwrap()
                    .into_string()
                    .unwrap();

                let gauge = prometheus::IntGauge::with_opts(prometheus::Opts {
                    namespace: "knot".to_string(),
                    subsystem: "dns".to_string(),
                    name: normalize(&label),
                    help: label,
                    const_labels: HashMap::from([("zone".to_string(), zone)]),
                    variable_labels: vec![],
                })
                .unwrap();
                gauge.set(parse(&value));

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
