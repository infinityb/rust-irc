// This file is heavily derived from Rust's stdlib, and therefore
// retains the copyright notice below

// Copyright 2013-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::string::{mod, String};

pub static IRC_ASCII_LOWER_MAP: [u8, ..256] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
    0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
    b' ', b'!', b'"', b'#', b'$', b'%', b'&', b'\'',
    b'(', b')', b'*', b'+', b',', b'-', b'.', b'/',
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7',
    b'8', b'9', b':', b';', b'<', b'=', b'>', b'?',
    b'@',

          b'a', b'b', b'c', b'd', b'e', b'f', b'g',
    b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o',
    b'p', b'q', b'r', b's', b't', b'u', b'v', b'w',
    b'x', b'y', b'z',

                      b'{', b'|', b'}', b'^', b'_',
    b'`', b'a', b'b', b'c', b'd', b'e', b'f', b'g',
    b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o',
    b'p', b'q', b'r', b's', b't', b'u', b'v', b'w',
    b'x', b'y', b'z', b'{', b'|', b'}', b'^', 0x7f,
    0x80, 0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
    0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f,
    0x90, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97,
    0x98, 0x99, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f,
    0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7,
    0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf,
    0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7,
    0xb8, 0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf,
    0xc0, 0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7,
    0xc8, 0xc9, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf,
    0xd0, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7,
    0xd8, 0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf,
    0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7,
    0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef,
    0xf0, 0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7,
    0xf8, 0xf9, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
];


pub trait IrcAsciiExt<T> for Sized? {
    /// Makes a copy of the string in IRC ASCII lower case:
    /// ASCII letters 'A' to 'Z' are mapped to 'a' to 'z',
    /// and "[]\~" are mapped to "{}|^" respectively,
    /// but all other characters are unchanged.
    fn to_irc_lower(&self) -> T;

    /// Check that two strings are an ASCII case-insensitive match.
    /// Same as `to_irc_lower(a) == to_irc_lower(b)`,
    /// but without allocating and copying temporary strings.
    fn eq_ignore_irc_case(&self, other: &Self) -> bool;
}

pub trait OwnedIrcAsciiExt {
    fn into_irc_lower(self) -> Self;
}

impl IrcAsciiExt<Vec<u8>> for [u8] {
    #[inline]
    fn to_irc_lower(&self) -> Vec<u8> {
        self.iter().map(|&byte| IRC_ASCII_LOWER_MAP[byte as uint]).collect()
    }

    #[inline]
    fn eq_ignore_irc_case(&self, other: &[u8]) -> bool {
        self.len() == other.len() &&
            self.iter().zip(other.iter()).all(
            |(byte_self, byte_other)| {
                IRC_ASCII_LOWER_MAP[*byte_self as uint] ==
                    IRC_ASCII_LOWER_MAP[*byte_other as uint]
            })
    }
}

impl OwnedIrcAsciiExt for Vec<u8> {
    #[inline]
    fn into_irc_lower(mut self) -> Vec<u8> {
        for byte in self.iter_mut() {
            *byte = IRC_ASCII_LOWER_MAP[*byte as uint];
        }
        self
    }
}

impl IrcAsciiExt<String> for str {
    #[inline]
    fn to_irc_lower(&self) -> String {
        // Vec<u8>::to_irc_lower() preserves the UTF-8 invariant.
        unsafe { string::raw::from_utf8(self.as_bytes().to_irc_lower()) }
    }

    #[inline]
    fn eq_ignore_irc_case(&self, other: &str) -> bool {
        self.as_bytes().eq_ignore_irc_case(other.as_bytes())
    }
}

impl OwnedIrcAsciiExt for String {
    #[inline]
    fn into_irc_lower(self) -> String {
        // Vec<u8>::into_irc_lower() preserves the UTF-8 invariant.
        unsafe { string::raw::from_utf8(self.into_bytes().into_irc_lower()) }
    }
}

#[test]
fn test_basics() {
    assert!("[".eq_ignore_irc_case("{"));
    assert!("]".eq_ignore_irc_case("}"));
    assert!("\\".eq_ignore_irc_case("|"));
    assert!("~".eq_ignore_irc_case("^"));

    assert_eq!("[".to_irc_lower()[], "{");
    assert_eq!("]".to_irc_lower()[], "}");
    assert_eq!("\\".to_irc_lower()[], "|");
    assert_eq!("~".to_irc_lower()[], "^");
}
