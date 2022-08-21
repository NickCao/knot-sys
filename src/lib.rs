pub mod bindings;

use crate::bindings::*;
use nom::character::complete::{alpha1, char, u64};
use nom::multi::fold_many1;
use nom::sequence::pair;
use std::collections::HashMap;
use std::ffi::{CStr, CString, NulError};
use std::os::raw::c_int;
use thiserror::Error;

pub struct KnotCtx {
    ctx: *mut knot_ctl_t,
}

pub type KnotResult<T> = Result<T, KnotError>;

#[derive(Error, Debug)]
pub enum KnotError {
    #[error("libknot error")]
    C(&'static CStr),
    #[error("null error")]
    Nul(#[from] NulError),
}

fn knot_result(value: c_int) -> KnotResult<()> {
    match value {
        bindings::knot_error_KNOT_EOK => Ok(()),
        _ => Err(unsafe { KnotError::C(CStr::from_ptr(bindings::knot_strerror(value))) }),
    }
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum KnotCtlType {
    END = knot_ctl_type_t_KNOT_CTL_TYPE_END,
    DATA = knot_ctl_type_t_KNOT_CTL_TYPE_DATA,
    EXTRA = knot_ctl_type_t_KNOT_CTL_TYPE_EXTRA,
    BLOCK = knot_ctl_type_t_KNOT_CTL_TYPE_BLOCK,
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum KnotCtlIdx {
    CMD = knot_ctl_idx_t_KNOT_CTL_IDX_CMD,
    DATA = knot_ctl_idx_t_KNOT_CTL_IDX_DATA,
    ERROR = knot_ctl_idx_t_KNOT_CTL_IDX_ERROR,
    FILTER = knot_ctl_idx_t_KNOT_CTL_IDX_FILTER,
    FLAGS = knot_ctl_idx_t_KNOT_CTL_IDX_FLAGS,
    ID = knot_ctl_idx_t_KNOT_CTL_IDX_ID,
    ITEM = knot_ctl_idx_t_KNOT_CTL_IDX_ITEM,
    OWNER = knot_ctl_idx_t_KNOT_CTL_IDX_OWNER,
    SECTION = knot_ctl_idx_t_KNOT_CTL_IDX_SECTION,
    TTL = knot_ctl_idx_t_KNOT_CTL_IDX_TTL,
    TYPE = knot_ctl_idx_t_KNOT_CTL_IDX_TYPE,
    ZONE = knot_ctl_idx_t_KNOT_CTL_IDX_ZONE,
}

pub type KnotCtlData = HashMap<KnotCtlIdx, CString>;

impl KnotCtx {
    pub fn new() -> Self {
        unsafe {
            Self {
                ctx: knot_ctl_alloc(),
            }
        }
    }
    pub fn connect(&self, path: &str) -> KnotResult<()> {
        unsafe {
            let path = CString::new(path)?;
            knot_result(knot_ctl_connect(self.ctx, path.as_ptr()))
        }
    }
    pub fn close(&self) {
        unsafe {
            knot_ctl_close(self.ctx);
        }
    }
    pub fn send(&self, r#type: KnotCtlType, data: Option<&KnotCtlData>) -> KnotResult<()> {
        let data = data.map(|data| {
            let mut packet = [std::ptr::null(); knot_ctl_idx_t_KNOT_CTL_IDX__COUNT as usize];
            data.iter().for_each(|(&k, v)| {
                packet[k as usize] = v.as_ptr();
            });
            packet
        });
        unsafe {
            knot_result(knot_ctl_send(
                self.ctx,
                r#type as knot_ctl_type_t,
                match data {
                    Some(mut data) => data.as_mut_ptr() as *mut knot_ctl_data_t,
                    None => std::ptr::null_mut::<knot_ctl_data_t>(),
                },
            ))
        }
    }
    pub fn recv(&self) -> KnotResult<(KnotCtlType, KnotCtlData)> {
        let mut r#type = std::mem::MaybeUninit::<knot_ctl_type_t>::uninit();
        let mut data = std::mem::MaybeUninit::<knot_ctl_data_t>::uninit();
        unsafe {
            knot_result(knot_ctl_receive(
                self.ctx,
                r#type.as_mut_ptr(),
                data.as_mut_ptr(),
            ))
        }?;
        unsafe {
            let data = data
                .assume_init_read()
                .iter()
                .enumerate()
                .filter_map(|(i, &ptr)| match ptr as usize {
                    0 => None,
                    _ => Some((
                        std::mem::transmute(i as knot_ctl_idx_t),
                        CStr::from_ptr(ptr).to_owned(),
                    )),
                })
                .collect();
            Ok((std::mem::transmute(r#type), data))
        }
    }
}

impl Drop for KnotCtx {
    fn drop(&mut self) {
        unsafe {
            knot_ctl_free(self.ctx);
        }
    }
}

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
