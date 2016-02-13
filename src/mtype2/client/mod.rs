use std::borrow::{Borrow, BorrowMut, ToOwned};
use std::borrow::Cow;
use std::{mem, ops};
use std::io::{self, Write};

use super::FromIrcMsg;
use super::cursor_chk_error;

use ::parse::old_parse::IrcMsg as IrcMsgLegacy;
use ::{IrcMsg, IrcMsgBuf};
use ::parse_helpers;

#[cfg(feature = "unstable")] mod cap;
#[cfg(feature = "unstable")] pub use self::cap::{
    CapLs, CapLsBuf,
};

impl_irc_msg_subtype!(Invite);
impl_irc_msg_subtype_buf!(InviteBuf, Invite);

impl Invite {
    fn construct<W>(sink: &mut W, nickname: &[u8], channel: &[u8]) -> Result<(), ()>
        where W: Write
    {
        try!(sink.write_all(b"INVITE ").or_else(cursor_chk_error));
        try!(sink.write_all(nickname).or_else(cursor_chk_error));
        try!(sink.write_all(b" ").or_else(cursor_chk_error));
        try!(sink.write_all(channel).or_else(cursor_chk_error));
        Ok(())
    }

    fn validate(_msg: &IrcMsg) -> Result<(), ()> {
        unimplemented!();
    }
}

impl InviteBuf {
    pub fn new(nickname: &[u8], channel: &[u8]) -> Result<InviteBuf, ()> {
        let mut wr = io::Cursor::new(Vec::new());
        try!(Invite::construct(&mut wr, nickname, channel));

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(wr.into_inner()).map_err(|_| ()));

        try!(Invite::validate(&message));
        Ok(InviteBuf { inner: message })
    }
}

impl_irc_msg_subtype!(Join);
impl_irc_msg_subtype_buf!(JoinBuf, Join);

impl Join {
    fn construct<W>(sink: &mut W, channel: &[u8]) -> Result<(), ()>
        where W: Write
    {
        try!(sink.write_all(b"JOIN ").or_else(cursor_chk_error));
        try!(sink.write_all(channel).or_else(cursor_chk_error));
        Ok(())
    }

    fn validate(_msg: &IrcMsg) -> Result<(), ()> {
        unimplemented!();
    }
}

impl JoinBuf {
    pub fn new(channel: &[u8]) -> Result<JoinBuf, ()> {
        let mut wr = io::Cursor::new(Vec::new());
        try!(Join::construct(&mut wr, channel));

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(wr.into_inner()).map_err(|_| ()));

        try!(Join::validate(&message));
        Ok(JoinBuf { inner: message })
    }
}


impl_irc_msg_subtype!(Ping);
impl_irc_msg_subtype_buf!(PingBuf, Ping);

impl Ping {
    fn construct<W>(sink: &mut W, server: &[u8]) -> Result<(), ()>
        where W: Write
    {
        try!(sink.write_all(b"PING :").or_else(cursor_chk_error));
        try!(sink.write_all(server).or_else(cursor_chk_error));
        Ok(())
    }

    fn validate(_msg: &IrcMsg) -> Result<(), ()> {
        unimplemented!();
    }
}

impl PingBuf {
    pub fn new(server: &[u8]) -> Result<PingBuf, ()> {
        let mut wr = io::Cursor::new(Vec::new());
        try!(Ping::construct(&mut wr, server));

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(wr.into_inner()).map_err(|_| ()));

        try!(Ping::validate(&message));
        Ok(PingBuf { inner: message })
    }
}


impl_irc_msg_subtype!(Pong);
impl_irc_msg_subtype_buf!(PongBuf, Pong);

impl Pong {
    fn construct<W>(sink: &mut W, server: &[u8]) -> Result<(), ()>
        where W: Write
    {
        try!(sink.write_all(b"PONG ").or_else(cursor_chk_error));
        try!(sink.write_all(server).or_else(cursor_chk_error));
        Ok(())
    }

    fn validate(_msg: &IrcMsg) -> Result<(), ()> {
        unimplemented!();
    }
}

impl PongBuf {
    pub fn new(source: &[u8]) -> Result<PongBuf, ()> {
        let mut wr = io::Cursor::new(Vec::new());
        try!(Pong::construct(&mut wr, source));

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(wr.into_inner()).map_err(|_| ()));

        try!(Pong::validate(&message));
        Ok(PongBuf { inner: message })
    }
}
