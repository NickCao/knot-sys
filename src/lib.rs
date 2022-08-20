#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(improper_ctypes)]
#![allow(non_snake_case)]
#![allow(dead_code)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use nom::character::complete::{alpha1, char, u64};
use nom::multi::fold_many1;
use nom::sequence::pair;

pub fn knot_bool_parse(value: &str) -> Option<bool> {
    match value {
        "yes" | "freezing" | "open" => Some(true),
        "no" | "thawing" | "none" => Some(false),
        _ => None,
    }
}

pub fn knot_time_parse(value: &str) -> Option<u64> {
    match value {
        "running" | "not scheduled" | "frozen" | "pending" => None,
        "0" => Some(0),
        _ => {
            let res = pair::<_, _, _, (), _, _>(
                char('+'), // consider events in the past as invalid
                fold_many1(
                    pair(u64, alpha1),
                    || 0u64,
                    |acc, item| {
                        let scale = match item.1 {
                            "Y" => 3600 * 24 * 365,
                            "M" => 3600 * 24 * 30,
                            "D" => 3600 * 24,
                            "h" => 3600,
                            "m" => 60,
                            "s" => 1,
                            _ => 0,
                        };
                        acc + scale * item.0
                    },
                ),
            )(value);
            match res {
                Ok(res) => Some(res.1 .1),
                Err(_) => None,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::knot_time_parse;

    #[test]
    fn knot_ctl() {
        unsafe {
            let ctl = super::knot_ctl_alloc();
            super::knot_ctl_set_timeout(ctl, 1000);
            super::knot_ctl_free(ctl);
        }
    }

    #[test]
    fn knot_time() {
        assert_eq!(knot_time_parse("0"), Some(0));
        assert_eq!(knot_time_parse("+23h57m29s"), Some(86249));
        assert_eq!(knot_time_parse("+6D23h37m28s"), Some(603448));
        assert_eq!(knot_time_parse("+68Y1M5D2h51m34s"), Some(2147482294));
    }
}
