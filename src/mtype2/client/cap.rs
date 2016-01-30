use std::borrow::{Borrow, BorrowMut, ToOwned};
use std::borrow::Cow;
use std::{mem, ops};
use std::io::{self, Write};

use super::super::FromIrcMsg;
use super::super::cursor_chk_error;

use ::legacy::IrcMsg as IrcMsgLegacy;
use ::{IrcMsg, IrcMsgBuf};
use ::parse_helpers;
use ::cap::{Capabilities, NegotiationVersion};

impl_irc_msg_subtype!(CapLs);
impl_irc_msg_subtype_buf!(CapLsBuf, CapLs);

#[inline]
fn neg_ver_str(version: NegotiationVersion) -> &'static str {
    use self::NegotiationVersion::*;
    match NegotiationVersion {
        V301 => "301",
        V302 => "302",
    }
}

impl CapLs {
    fn construct<W>(sink: &mut W, version: NegotiationVersion) -> Result<(), ()>
        where W: Write
    {
        try!(sink.write_all(b"CAP LS ").or_else(cursor_chk_error));
        let vers_name = neg_ver_str(version).as_bytes();
        try!(sink.write_all(vers_name).or_else(cursor_chk_error));
        Ok(())
    }

    fn validate(_msg: &IrcMsg) -> Result<(), ()> {
        unimplemented!();
    }

    pub fn get_version(&self) -> Option<NegotiationVersion> {
        unimplemented!();
    }
}

impl CapLsBuf {
    pub fn new(version: NegotiationVersion) -> CapLsBuf {
        let mut wr = io::Cursor::new(Vec::new());
        CapLs::construct(&mut wr, version).unwrap();

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = IrcMsgBuf::new(wr.into_inner()).unwrap();
        CapLs::validate(&message).unwrap();
        CapLsBuf { inner: message }
    }
}
