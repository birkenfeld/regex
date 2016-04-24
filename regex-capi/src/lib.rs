#![allow(dead_code, unused_variables)]

extern crate libc;
extern crate regex;

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::fmt;
use std::io::{self, Write};
use std::ops::Deref;
use std::process;
use std::ptr;
use std::slice;
use std::str;

use libc::c_char;
use regex::bytes;

pub struct Options(());

#[derive(Debug)]
pub struct Error {
    message: Option<CString>,
    kind: ErrorKind,
}

#[derive(Debug)]
enum ErrorKind {
    None,
    Str(str::Utf8Error),
    Regex(regex::Error),
}

impl Error {
    fn new(kind: ErrorKind) -> Error {
        Error {
            message: None,
            kind: kind,
        }
    }

    fn is_err(&self) -> bool {
        match self.kind {
            ErrorKind::None => false,
            ErrorKind::Str(_) | ErrorKind::Regex(_) => true,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.kind {
            ErrorKind::None => write!(f, "no error"),
            ErrorKind::Str(ref e) => e.fmt(f),
            ErrorKind::Regex(ref e) => e.fmt(f),
        }
    }
}

#[no_mangle]
pub extern fn rure_error_new() -> *mut Error {
    Box::into_raw(Box::new(Error::new(ErrorKind::None)))
}

#[no_mangle]
pub extern fn rure_error_free(err: *const Error) {
    unsafe { Box::from_raw(err as *mut Error); }
}

#[no_mangle]
pub extern fn rure_error_message(err: *mut Error) -> *const c_char {
    let err = unsafe { &mut *err };
    let cmsg = match CString::new(format!("{}", err)) {
        Ok(msg) => msg,
        Err(err) => {
            // I guess this can probably happen if the regex itself has a NUL,
            // and that NUL re-occurs in the context presented by the error
            // message. In this case, just show as much as we can.
            let nul = err.nul_position();
            let msg = err.into_vec();
            CString::new(msg[0..nul].to_owned()).unwrap()
        }
    };
    let p = cmsg.as_ptr();
    err.message = Some(cmsg);
    p
}

#[repr(C)]
pub struct rure_match {
    pub start: usize,
    pub end: usize,
}

pub struct Captures(Vec<Option<usize>>);

#[no_mangle]
pub extern fn rure_captures_new(re: *const Regex) -> *mut Captures {
    let re = unsafe { &*re };
    let captures = Captures(vec![None; 2 * re.captures_len()]);
    Box::into_raw(Box::new(captures))
}

#[no_mangle]
pub extern fn rure_captures_free(captures: *const Captures) {
    unsafe { Box::from_raw(captures as *mut Captures); }
}

#[no_mangle]
pub extern fn rure_captures_at(
    captures: *const Captures,
    i: usize,
    match_info: *mut rure_match,
) -> bool {
    let captures = unsafe { &(*captures).0 };
    match (captures[i * 2], captures[i * 2 + 1]) {
        (Some(start), Some(end)) => {
            if !match_info.is_null() {
                unsafe {
                    (*match_info).start = start;
                    (*match_info).end = end;
                }
            }
            true
        }
        _ => false
    }
}

#[no_mangle]
pub extern fn rure_captures_len(captures: *const Captures) -> usize {
    unsafe { (*captures).0.len() / 2 }
}

pub struct Regex {
    re: bytes::Regex,
    capture_names: HashMap<String, i32>,
}

impl Deref for Regex {
    type Target = bytes::Regex;
    fn deref(&self) -> &bytes::Regex { &self.re }
}

#[no_mangle]
pub extern fn rure_compile(
    pattern: *const c_char,
    error: *mut Error,
) -> *const Regex {
    let len = unsafe { CStr::from_ptr(pattern).to_bytes().len() };
    let pat = pattern as *const u8;
    rure_compile_options(pat, len, ptr::null(), error)
}

#[no_mangle]
pub extern fn rure_compile_must(
    pattern: *const c_char,
) -> *const Regex {
    let mut err = Error::new(ErrorKind::None);
    let re = rure_compile(pattern, &mut err);
    if err.is_err() {
        let _ = writeln!(&mut io::stderr(), "{}", err);
        let _ = writeln!(&mut io::stderr(), "aborting from rure_compile_must");
        process::exit(1);
    }
    re
}

#[no_mangle]
pub extern fn rure_compile_options(
    pattern: *const u8,
    length: usize,
    options: *const Options,
    error: *mut Error,
) -> *const Regex {
    let pat = unsafe { slice::from_raw_parts(pattern, length) };
    let pat = match str::from_utf8(pat) {
        Ok(pat) => pat,
        Err(err) => {
            unsafe {
                if !error.is_null() {
                    *error = Error::new(ErrorKind::Str(err));
                }
                return ptr::null();
            }
        }
    };
    match bytes::Regex::new(pat) {
        Ok(re) => {
            let mut capture_names = HashMap::new();
            for (i, name) in re.capture_names().enumerate() {
                if let Some(name) = name {
                    capture_names.insert(name.to_owned(), i as i32);
                }
            }
            let re = Regex {
                re: re,
                capture_names: capture_names,
            };
            Box::into_raw(Box::new(re))
        }
        Err(err) => {
            unsafe {
                if !error.is_null() {
                    *error = Error::new(ErrorKind::Regex(err));
                }
                ptr::null()
            }
        }
    }
}

#[no_mangle]
pub extern fn rure_free(re: *const Regex) {
    unsafe { Box::from_raw(re as *mut Regex); }
}

#[no_mangle]
pub extern fn rure_capture_name_index(
    re: *const Regex,
    name: *const c_char,
) -> i32 {
    let re = unsafe { &*re };
    let name = unsafe { CStr::from_ptr(name) };
    let name = match name.to_str() {
        Err(_) => return -1,
        Ok(name) => name,
    };
    re.capture_names.get(name).map(|&i|i).unwrap_or(-1)
}

#[no_mangle]
pub extern fn rure_is_match(
    re: *const Regex,
    haystack: *const u8,
    len: usize,
    start: usize,
) -> bool {
    let re = unsafe { &*re };
    let haystack = unsafe { slice::from_raw_parts(haystack, len) };
    re.is_match_at(haystack, start)
}

#[no_mangle]
pub extern fn rure_find(
    re: *const Regex,
    haystack: *const u8,
    len: usize,
    start: usize,
    match_info: *mut rure_match,
) -> bool {
    let re = unsafe { &*re };
    let haystack = unsafe { slice::from_raw_parts(haystack, len) };
    re.find_at(haystack, start).map(|(s, e)| unsafe {
        if !match_info.is_null() {
            (*match_info).start = s;
            (*match_info).end = e;
        }
    }).is_some()
}

#[no_mangle]
pub extern fn rure_find_captures(
    re: *const Regex,
    haystack: *const u8,
    len: usize,
    start: usize,
    captures: *mut Captures,
) -> bool {
    let re = unsafe { &*re };
    let haystack = unsafe { slice::from_raw_parts(haystack, len) };
    let slots = unsafe { &mut (*captures).0 };
    re.read_captures_at(slots, haystack, start).is_some()
}

pub struct Iter {
    re: *const Regex,
    haystack: *const u8,
    len: usize,
    last_end: usize,
    last_match: Option<usize>,
}

#[no_mangle]
pub extern fn rure_iter_new(
    re: *const Regex,
    haystack: *const u8,
    len: usize,
) -> *mut Iter {
    Box::into_raw(Box::new(Iter {
        re: re,
        haystack: haystack,
        len: len,
        last_end: 0,
        last_match: None,
    }))
}

#[no_mangle]
pub extern fn rure_iter_free(it: *mut Iter) {
    unsafe { Box::from_raw(it); }
}

#[no_mangle]
pub extern fn rure_iter_next(
    it: *mut Iter,
    match_info: *mut rure_match,
) -> bool {
    let it = unsafe { &mut *it };
    let re = unsafe { &*it.re };
    let text = unsafe { slice::from_raw_parts(it.haystack, it.len) };
    if it.last_end > text.len() {
        return false;
    }
    let (s, e) = match re.find_at(text, it.last_end) {
        None => return false,
        Some((s, e)) => (s, e),
    };
    if s == e {
        // This is an empty match. To ensure we make progress, start
        // the next search at the smallest possible starting position
        // of the next match following this one.
        it.last_end += 1;
        // Don't accept empty matches immediately following a match.
        // Just move on to the next match.
        if Some(e) == it.last_match {
            return rure_iter_next(it, match_info);
        }
    } else {
        it.last_end = e;
    }
    it.last_match = Some(e);
    if !match_info.is_null() {
        unsafe {
            (*match_info).start = s;
            (*match_info).end = e;
        }
    }
    true
}

#[no_mangle]
pub extern fn rure_iter_next_captures(
    it: *mut Iter,
    captures: *mut Captures,
) -> bool {
    let it = unsafe { &mut *it };
    let re = unsafe { &*it.re };
    let slots = unsafe { &mut (*captures).0 };
    let text = unsafe { slice::from_raw_parts(it.haystack, it.len) };
    if it.last_end > text.len() {
        return false;
    }
    let (s, e) = match re.read_captures_at(slots, text, it.last_end) {
        None => return false,
        Some((s, e)) => (s, e),
    };
    if s == e {
        // This is an empty match. To ensure we make progress, start
        // the next search at the smallest possible starting position
        // of the next match following this one.
        it.last_end += 1;
        // Don't accept empty matches immediately following a match.
        // Just move on to the next match.
        if Some(e) == it.last_match {
            return rure_iter_next_captures(it, captures);
        }
    } else {
        it.last_end = e;
    }
    it.last_match = Some(e);
    true
}
