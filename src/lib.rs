#![doc(html_root_url = "https://docs.rs/gnurx-sys/0.2.1")]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

/*!
# `gnurx-sys`: Unsafe Rust bindings for `libgnurx`

This is the regex functionality from `glibc` extracted into a separate library,
for Win32.
See the [`README`](libgnurx/README) of the C library.

[`regcomp()`], [`regexec()`], [`regerror()`] and [`regfree()`] are POSIX regex
functions.
They conform to `POSIX.1-2001` and `POSIX.1-2008` standards.
They are defined as follows:

```ignore
extern "C" {
    pub fn regcomp(
        preg: *mut regex_t,
        pattern: *const c_char,
        cflags: c_int,
    ) -> c_int;

    pub fn regexec(
        preg: *const regex_t,
        string: *const c_char,
        nmatch: usize,
        pmatch: *mut regmatch_t,
        eflags: c_int,
    ) -> c_int;

    pub fn regerror(
        errcode: c_int,
        preg: *const regex_t,
        errbuf: *mut c_char,
        errbuf_size: usize,
    ) -> usize;

    pub fn regfree(preg: *mut regex_t);
}
```

[`regcomp()`]: fn.regcomp.html
[`regexec()`]: fn.regexec.html
[`regerror()`]: fn.regerror.html
[`regfree()`]: fn.regfree.html

## POSIX regex compiling

`regcomp()` is used to compile a regular expression into a form that is suitable
for subsequent `regexec()` searches.

`regcomp()` is supplied with `preg`, a pointer to a pattern buffer storage area;
`pattern`, a pointer to the null-terminated string and `cflags`, flags used to
determine the type of compilation.

All regular expression searching must be done via a compiled pattern buffer,
thus `regexec()` must always be supplied with the address of a `regcomp()`
initialized pattern buffer.

`cflags` may be the bitwise-or of zero or more of the following:

- `REG_EXTENDED`:
  Use POSIX Extended Regular Expression syntax when interpreting regex.
  If not set, POSIX Basic Regular Expression syntax is used.

- `REG_ICASE`:
  Do not differentiate case.
  Subsequent `regexec()` searches using this pattern buffer will be case
  insensitive.

- `REG_NOSUB`:
  Do not report position of matches.
  The nmatch and pmatch arguments to `regexec()` are ignored if the pattern
  buffer supplied was compiled with this flag set.

- `REG_NEWLINE`: Match-any-character operators don't match a newline.
  - A nonmatching list (`[^...]`) not containing a newline does not match
    a newline.
  - Match-beginning-of-line operator (`^`) matches the empty string immediately
    after a newline, regardless of whether `eflags`, the execution flags
    of `regexec()`, contains `REG_NOTBOL`.
  - Match-end-of-line operator (`$`) matches the empty string immediately before
    a newline, regardless of whether `eflags` contains `REG_NOTEOL`.

## POSIX regex matching

`regexec()` is used to match a null-terminated string against the precompiled
pattern buffer, `preg`.
`nmatch` and `pmatch` are used to provide information regarding the location
of any matches.
`eflags` may be the bitwise-or of one or both of `REG_NOTBOL` and `REG_NOTEOL`
which cause changes in matching behavior described below:

- `REG_NOTBOL`:
  The match-beginning-of-line operator always fails to match (but see the
  compilation flag `REG_NEWLINE` above).
  This flag may be used when different portions of a string are passed
  to `regexec()` and the beginning of the string should not be interpreted
  as the beginning of the line.

- `REG_NOTEOL`:
  The match-end-of-line operator always fails to match (but see the compilation
  flag `REG_NEWLINE` above).

## Byte offsets

Unless `REG_NOSUB` was set for the compilation of the pattern buffer,
it is possible to obtain match addressing information.

`pmatch` must be dimensioned to have at least `nmatch` elements.
These are filled in by `regexec()` with substring match addresses.
The offsets of the subexpression starting at the `i`th open parenthesis
are stored in `pmatch[i]`.

The entire regular expression's match addresses are stored in `pmatch[0]`.
(Note that to return the offsets of `N` subexpression matches, `nmatch`
must be at least `N + 1`.)

Any unused structure elements will contain the value `-1`.

The [`regmatch_t`] structure which is the type of `pmatch` is defined as:

```ignore
pub struct regmatch_t {
    pub rm_so: regoff_t,
    pub rm_eo: regoff_t,
}
```

Each `rm_so` element that is not `-1` indicates the start offset of the next
largest substring match within the string.

The relative `rm_eo` element indicates the end offset of the match, which is
the offset of the first character after the matching text.

[`regmatch_t`]: struct.regmatch_t.html

## POSIX error reporting

`regerror()` is used to turn the error codes that can be returned by both
`regcomp()` and `regexec()` into error message strings.

`regerror()` is passed the error code, `errcode`, the pattern buffer, `preg`,
a pointer to a character string buffer, `errbuf`, and the size of the string
buffer, `errbuf_size`.

It returns the size of the `errbuf` required to contain the null-terminated
error message string.
If both `errbuf` and `errbuf_size` are nonzero, `errbuf` is filled in with
the first `errbuf_size - 1` characters of the error message and a terminating
null byte (`\0`).

## POSIX pattern buffer freeing

Supplying `regfree()` with a precompiled pattern buffer, `preg` will free the
memory allocated to the pattern buffer by the compiling process, `regcomp()`.

## Return values and errors

`regcomp()` returns zero for a successful compilation.
On failure, it returns one of the following errors (see [`reg_errcode_t`]):

- `REG_BADBR`: Invalid use of back reference operator.
- `REG_BADPAT`: Invalid use of pattern operators such as group or list.
- `REG_BADRPT`: Invalid use of repetition operators such as using `*` as
  the first character.
- `REG_EBRACE`: Un-matched brace interval operators.
- `REG_EBRACK`: Un-matched bracket list operators.
- `REG_ECOLLATE`: Invalid collating element.
- `REG_ECTYPE`: Unknown character class name.
- `REG_EEND`: Nonspecific error. This is not defined by POSIX.2.
- `REG_EESCAPE`: Trailing backslash.
- `REG_EPAREN`: Un-matched parenthesis group operators.
- `REG_ERANGE`: Invalid use of the range operator; for example, the ending
  point of the range occurs prior to the starting point.
- `REG_ESIZE`: Compiled regular expression requires a pattern buffer larger
  than 64Kb. This is not defined by POSIX.2.
- `REG_ESPACE`: The regex routines ran out of memory.
- `REG_ESUBREG`: Invalid back reference to a subexpression.

`regexec()` returns zero for a successful match or `REG_NOMATCH` for failure.

[`reg_errcode_t`]: reg_errcode_t/index.html

## Thread safety

- `regcomp()` and `regexec()` are thread-safe only if the process locale
  is not modified during the call.
- `regerror()` is thread-safe only if the process environment is not modified
  during the call.
- `regfree()` is thread-safe.

## Supported environment variables

This crate depends on some environment variables, and *variants* of those.
For each environment variable (e.g., `CC`), the following are the accepted
variants of it:
- `<var>_<target>`, e.g., `CC_x86_64-pc-windows-gnu`.
- `<var>_<target-with-underscores>`, e.g., `CC_x86_64_pc_windows_gnu`.
- `TARGET_<var>`, e.g., `TARGET_CC`.
- `<var>`, e.g., `CC`.

The following environment variables (and their variants) affect how this crate
is built:
- `GNURX_LIB_DIR_PREFIX`
- `CC`
- `CFLAGS`
- `AR`
- `ARFLAGS`

## Linking options

By default, this crate builds `libgnurx` from sources and links statically
against it.

In order to change this behavior, and instruct this crate to dynamically link
against an externally built `libgnurx-0.dll` library, please define
the environment variable `GNURX_LIB_DIR_PREFIX` (or any of its variants)
when building.
The value of `GNURX_LIB_DIR_PREFIX` needs to be the absolute prefix path where
the library is installed.
The `libgnurx` header files are expected to reside
in `<GNURX_LIB_DIR_PREFIX>/include/`, and the shared library should reside
in `<GNURX_LIB_DIR_PREFIX>/bin/`.

## Depending on this crate

This crate provides the following variables to other crates that depend on it:
- `DEP_GNURX_INCLUDE`: Path of the directory where library C header files reside.
- `DEP_GNURX_LIB`: Path of the directory where the library binary resides.

## Platform-specific notes

This crate supports only the following target platforms:
- `x86_64-pc-windows-gnu`.
- `i686-pc-windows-gnu`.

This is due to the nature of the `libgnurx` library.

## Versioning

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
The `CHANGELOG.md` file details notable changes over time.

## License

Copyright (c) 2020 Koutheir Attouchi.

See the `LICENSE.md` file at the top-level directory of this distribution.

Licensed under the **LGPL version 2.1 license, or any later version thereof**.
This file may not be copied, modified, or distributed except according to those terms.
*/

#[cfg(all(target_family = "windows", target_env = "gnu"))]
include!(concat!(env!("OUT_DIR"), "/gnurx-sys.rs"));

#[cfg(all(test, target_family = "windows", target_env = "gnu"))]
mod tests;
