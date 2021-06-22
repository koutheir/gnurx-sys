#![cfg(all(
    test,
    target_os = "windows",
    target_env = "gnu",
    any(target_arch = "x86", target_arch = "x86_64")
))]

use std::ffi::CString;
use std::os::raw::c_int;
use std::{mem, ptr};

use super::{REG_EXTENDED, REG_ICASE, REG_NEWLINE, REG_NOSUB};

fn new_regex_t() -> super::regex_t {
    unsafe { mem::zeroed() }
}

fn regcomp(preg: &mut super::regex_t, pattern: &str, flags: c_int) -> c_int {
    let c_pattern = CString::new(pattern).unwrap();
    unsafe { super::regcomp(preg, c_pattern.as_c_str().as_ptr(), flags) }
}

fn regfree(preg: &mut super::regex_t) {
    unsafe { super::regfree(preg) }
}

fn regexec(
    preg: &super::regex_t,
    string: &str,
    pmatch: &mut [super::regmatch_t],
    flags: c_int,
) -> c_int {
    let c_string = CString::new(string).unwrap();

    unsafe {
        super::regexec(
            preg,
            c_string.as_c_str().as_ptr(),
            pmatch.len(),
            pmatch.as_mut_ptr(),
            flags,
        )
    }
}

fn regerror(errcode: c_int, preg: &super::regex_t) -> String {
    let buffer_size = unsafe { super::regerror(errcode, preg, ptr::null_mut(), 0) };
    if buffer_size <= 1 {
        return String::default();
    }

    let mut buf = vec![0_u8; buffer_size];
    let r = unsafe { super::regerror(errcode, preg, buf.as_mut_ptr().cast(), buf.capacity()) };

    assert_eq!(r, buf.capacity());
    assert_eq!(buf.pop(), Some(0_u8));
    String::from_utf8(buf).unwrap()
}

#[test]
fn regerror_error() {
    let mut re_state = new_regex_t();

    let r = regcomp(&mut re_state, r#"ab\"#, REG_NOSUB | REG_NEWLINE);
    assert_eq!(r, super::reg_errcode_t::REG_EESCAPE as c_int);

    assert!(!regerror(r, &re_state).is_empty());
}

#[test]
fn regcomp_invalid() {
    let mut re_state = new_regex_t();

    let params = [
        (r#"ab\"#, super::reg_errcode_t::REG_EESCAPE),
        (r#"a[9-6]b"#, super::reg_errcode_t::REG_ERANGE),
        (r#"a[0-b"#, super::reg_errcode_t::REG_EBRACK),
        (r#"a\(bb"#, super::reg_errcode_t::REG_EPAREN),
        (r#"ab\{2c"#, super::reg_errcode_t::REG_EBRACE),
        (r#"a\3b"#, super::reg_errcode_t::REG_ESUBREG),
    ];

    for (pattern, err_code) in &params {
        let r = regcomp(&mut re_state, pattern, REG_NOSUB | REG_NEWLINE);
        assert_eq!(r, (*err_code) as c_int);
    }
}

#[test]
fn regcomp_ok() {
    let mut re_state = new_regex_t();

    let flags = REG_NOSUB | REG_NEWLINE;
    assert_eq!(0, regcomp(&mut re_state, r#"a[0-9]b"#, flags));

    regfree(&mut re_state);
}

#[test]
fn regexec_no_match() {
    let mut re_state = new_regex_t();

    let flags = REG_NOSUB | REG_NEWLINE;
    assert_eq!(0, regcomp(&mut re_state, r#"a[5-9]b"#, flags));

    for text in &["a5c", "a4b", "A5B"] {
        let r = regexec(&re_state, text, &mut [], 0);
        assert_eq!(r, super::reg_errcode_t::REG_NOMATCH as c_int);
    }

    regfree(&mut re_state);
}

#[test]
fn regexec_match() {
    let mut re_state = new_regex_t();

    let r = regcomp(&mut re_state, r#"a[5-9]b"#, REG_NOSUB | REG_NEWLINE);
    assert_eq!(r, 0);

    assert_eq!(0, regexec(&re_state, "a7b", &mut [], 0));

    regfree(&mut re_state);
}

#[test]
fn regexec_match_case_insensitive() {
    let mut re_state = new_regex_t();

    let flags = REG_NOSUB | REG_NEWLINE | REG_ICASE;
    assert_eq!(0, regcomp(&mut re_state, r#"a[5-9]b"#, flags));

    assert_eq!(0, regexec(&re_state, "A7b", &mut [], 0));

    regfree(&mut re_state);
}

#[test]
fn regexec_match_extended() {
    let mut re_state = new_regex_t();

    let flags = REG_NOSUB | REG_NEWLINE | REG_EXTENDED;
    assert_eq!(0, regcomp(&mut re_state, r#"a(x|y)b"#, flags));

    assert_eq!(0, regexec(&re_state, "axb", &mut [], 0));
    assert_eq!(0, regexec(&re_state, "ayb", &mut [], 0));

    let r = regexec(&re_state, "azb", &mut [], 0);
    assert_eq!(r, super::reg_errcode_t::REG_NOMATCH as c_int);

    regfree(&mut re_state);
}

#[test]
fn regexec_multiple_matches() {
    let mut re_state = new_regex_t();

    let pattern = r#"a\([0-9]\)-\([a-z]\)-\([a-z]\)\?b"#;
    assert_eq!(0, regcomp(&mut re_state, pattern, REG_NEWLINE));

    let mut matches: [super::regmatch_t; 4] = [unsafe { mem::zeroed() }; 4];
    assert_eq!(0, regexec(&re_state, "a7-m-bxxx", &mut matches, 0));
    assert_eq!(matches[0].rm_so, 0);
    assert_eq!(matches[0].rm_eo, 6);
    assert_eq!(matches[1].rm_so, 1);
    assert_eq!(matches[1].rm_eo, 2);
    assert_eq!(matches[2].rm_so, 3);
    assert_eq!(matches[2].rm_eo, 4);
    assert_eq!(matches[3].rm_so, -1);

    let mut matches: [super::regmatch_t; 4] = [unsafe { mem::zeroed() }; 4];
    assert_eq!(0, regexec(&re_state, "ra7-m-yb", &mut matches, 0));
    assert_eq!(matches[0].rm_so, 1);
    assert_eq!(matches[0].rm_eo, 8);
    assert_eq!(matches[1].rm_so, 2);
    assert_eq!(matches[1].rm_eo, 3);
    assert_eq!(matches[2].rm_so, 4);
    assert_eq!(matches[2].rm_eo, 5);
    assert_eq!(matches[3].rm_so, 6);
    assert_eq!(matches[3].rm_eo, 7);

    regfree(&mut re_state);
}
