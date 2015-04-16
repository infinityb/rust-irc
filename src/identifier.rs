use std::cmp::PartialEq;
use std::default::Default;
use std::hash::{Hash, Hasher};
use std::str::{from_utf8, Utf8Error};

use irccase::CaseMapping;

// nickname   =  ( letter / special ) *8( letter / digit / special / "-" )
// special    =  %x5B-60 / %x7B-7D ; "[", "]", "\", "`", "_", "^", "{", "|", "}"
// letter     =  %x41-5A / %x61-7A       ; A-Z / a-z
// digit      =  %x30-39                 ; 0-9

static SPECIAL: &'static [u8] = &[b'[', b']', b'\\', b'`', b'_', b'^', b'{', b'|', b'}'];

static LETTER: &'static [u8] = &[
          b'a', b'b', b'c', b'd', b'e', b'f', b'g',
    b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o',
    b'p', b'q', b'r', b's', b't', b'u', b'v', b'w',
    b'x', b'y', b'z',

          b'A', b'B', b'C', b'D', b'E', b'F', b'G',
    b'H', b'I', b'J', b'K', b'L', b'M', b'N', b'O',
    b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W',
    b'X', b'Y', b'Z',
];

static DIGIT: &'static [u8] = &[b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];


#[derive(Clone, Eq, Debug)]
pub struct Channel<CM: CaseMapping>(CM, Vec<u8>);

pub enum ChannelError {
    InvalidByte(usize),
}

#[inline]
fn channel_is_valid_byte(target: u8) -> bool {
    match target {
        // NUL, BEL, LF, CR, space, comma, colon
        0x00 | 0x07 | 0x0A | 0x0D | 0x20 | 0x2C | 0x3A => false,
        _ => true
    }
}

#[inline]
fn channel_validate_buf(buf: &[u8]) -> Result<(), ChannelError> {
    for (idx, &byte) in buf.iter().enumerate() {
        if !channel_is_valid_byte(byte) {
            return Err(ChannelError::InvalidByte(idx));
        }
    }
    Ok(())
}

impl<CM: CaseMapping> Channel<CM> {
    #[inline]
    pub fn from_str(channel: &str) -> Result<Channel<CM>, ChannelError> {
        Channel::from_bytes(channel.as_bytes())
    }

    /// Safe to call if you know channel does not contain any invalid characters
    #[inline]
    #[deprecated]
    pub fn from_str_panic(channel: &str) -> Channel<CM> {
        Channel::from_str(channel).ok().expect("Illegal character in channel name")
    }

    #[inline]
    pub fn from_bytes<Q: AsRef<[u8]>+?Sized>(name: &Q) -> Result<Channel<CM>, ChannelError> {
        match channel_validate_buf(name.as_ref()) {
            Ok(()) => Ok(Channel(Default::default(), name.as_ref().to_vec())),
            Err(err) => Err(err),
        }
    }
}

impl<CM: CaseMapping> PartialEq for Channel<CM> {
    #[inline]
    fn eq(&self, other: &Channel<CM>) -> bool {
        let Channel(ref cm0, ref s_data) = *self;
        let Channel(ref cm1, ref o_data) = *other;
        cm0 == cm1 && cm0.eq_ignore_case(s_data, o_data)
    }

    #[inline]
    fn ne(&self, other: &Channel<CM>) -> bool {
        let Channel(ref cm0, ref s_data) = *self;
        let Channel(ref cm1, ref o_data) = *other;
        cm0 != cm1 || !cm0.eq_ignore_case(s_data, o_data)
    }
}

impl<CM: CaseMapping> Hash for Channel<CM> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Channel(ref case_mapping, ref data) = *self;
        data.len().hash(state);
        case_mapping.hash_ignore_case(data, state);
    }
}


#[derive(Clone, Eq, Debug)]
pub struct Nickname<CM: CaseMapping>(CM, Vec<u8>);

pub enum NicknameError {
    InvalidByte(usize),
    Utf8Error(Utf8Error),
}

#[inline]
fn nickname_is_valid_byte(target: u8) -> bool {
    for &byte in LETTER.iter() {
        if target == byte {
            return true;
        }
    }
    for &byte in DIGIT.iter() {
        if target == byte {
            return true;
        }
    }
    for &byte in SPECIAL.iter() {
        if target == byte {
            return true;
        }
    }
    return false;
}

#[inline]
fn nickname_is_valid_first_byte(target: u8) -> bool {
    for &byte in LETTER.iter() {
        if target == byte {
            return true;
        }
    }
    for &byte in SPECIAL.iter() {
        if target == byte {
            return true;
        }
    }
    return false;
}

#[inline]
fn nickname_validate_buf(buf: &[u8]) -> Result<(), NicknameError> {
    if let Err(err) = from_utf8(buf) {
        return Err(NicknameError::Utf8Error(err));
    }
    for (idx, &byte) in buf.iter().enumerate() {
        if idx == 0 {
            if !nickname_is_valid_first_byte(byte) {
                return Err(NicknameError::InvalidByte(idx));
            }
        } else {
            if !nickname_is_valid_byte(byte) {
                return Err(NicknameError::InvalidByte(idx));
            }
        }
    }
    Ok(())
}

impl<CM: CaseMapping> Nickname<CM> {
    #[inline]
    pub fn from_str(nick: &str) -> Result<Nickname<CM>, NicknameError> {
        Nickname::from_bytes(nick.as_bytes())
    }

    /// Safe to call if you know nickname does not contain any invalid characters
    #[inline]
    #[deprecated]
    pub fn from_str_panic(nick: &str) -> Nickname<CM> {
        Nickname::from_bytes(nick.as_bytes()).ok().expect("Illegal character in nickname")
    }

    #[inline]
    pub fn from_bytes<Q: AsRef<[u8]>+?Sized>(name: &Q) -> Result<Nickname<CM>, NicknameError> {
        match nickname_validate_buf(name.as_ref()) {
            Ok(()) => Ok(Nickname(Default::default(), name.as_ref().to_vec())),
            Err(err) => Err(err),
        }
    }

    pub fn as_str(&self) -> &str {
        let Nickname(_, ref data) = *self;
        match from_utf8(data.as_ref()) {
            Ok(str_ref) => str_ref,
            // Error condition should never happen. the UTF-8 invariant should
            // not be violated at any point.
            Err(err) => panic!("Illegal byte sequence in nickname: {:?}", err)
        }
    }

    /// Return the underlying byte buffer, encoded as UTF-8.
    pub fn into_bytes(self) -> Vec<u8> {
        let Nickname(_, data) = self;
        data
    }

    pub fn into_string(self) -> String {
        match String::from_utf8(self.into_bytes()) {
            Ok(string) => string,
            // Error condition should never happen. the UTF-8 invariant should
            // not be violated at any point.
            Err(err) => panic!("Illegal byte sequence in nickname: {:?}", err.into_bytes())
        }
    }
}

impl<CM: CaseMapping> PartialEq for Nickname<CM> {
    #[inline]
    fn eq(&self, other: &Nickname<CM>) -> bool {
        let Nickname(ref cm0, ref s_data) = *self;
        let Nickname(ref cm1, ref o_data) = *other;
        cm0 == cm1 && cm0.eq_ignore_case(s_data, o_data)
    }

    #[inline]
    fn ne(&self, other: &Nickname<CM>) -> bool {
        let Nickname(ref cm0, ref s_data) = *self;
        let Nickname(ref cm1, ref o_data) = *other;
        cm0 != cm1 || !cm0.eq_ignore_case(s_data, o_data)
    }
}

impl<CM: CaseMapping> Hash for Nickname<CM> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Nickname(ref case_mapping, ref data) = *self;
        data.len().hash(state);
        case_mapping.hash_ignore_case(data, state);
    }
}
