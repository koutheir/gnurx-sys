#![doc = include_str!("../README.md")]
#![cfg(all(
    target_os = "windows",
    target_env = "gnu",
    any(target_arch = "x86", target_arch = "x86_64")
))]
#![doc(html_root_url = "https://docs.rs/gnurx-sys/0.3.8")]
#![warn(unsafe_op_in_unsafe_fn)]
#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/gnurx-sys.rs"));

#[cfg(test)]
mod tests;
