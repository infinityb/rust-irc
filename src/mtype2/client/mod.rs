use std::borrow::{Borrow, ToOwned};
use std::{mem, ops};
use std::io::{self, Write};

use super::FromIrcMsg;
use super::cursor_chk_error;

use ::{IrcMsg, IrcMsgBuf};

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
        writevec(sink, &[
            b"INVITE ",
            nickname,
            b" ",
            channel,
        ]).or_else(cursor_chk_error)
    }

    fn validate(_msg: &IrcMsg) -> Result<(), ()>
    {
        // XXX: unimplemented!();
        Ok(())
    }
}

impl InviteBuf {
    pub fn new(nickname: &[u8], channel: &[u8]) -> Result<InviteBuf, ()>
    {
        let mut wr = io::Cursor::new(Vec::new());
        try!(Invite::construct(&mut wr, nickname, channel));

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(wr.into_inner()).map_err(|_| ()));

        // FIXME: try!(Invite::validate(&message));
        Ok(InviteBuf { inner: message })
    }
}


impl_irc_msg_subtype!(Join);
impl_irc_msg_subtype_buf!(JoinBuf, Join);

impl Join {
    fn construct<W>(sink: &mut W, channel: &[u8]) -> Result<(), ()>
        where W: Write
    {
        writevec(sink, &[b"JOIN ", channel]).or_else(cursor_chk_error)
    }

    /// Create a new `Join` in `storage`.  This does not allocate any storage.
    pub fn new<'a>(storage: &'a mut [u8], channel: &[u8]) -> Result<&'a Join, ()>
    {
        let mut wr = io::Cursor::new(storage);
        try!(Join::construct(&mut wr, channel));
        let end = wr.position() as usize;

        let storage = wr.into_inner();
        Join::parse(&storage[..end])
    }

    fn validate(_msg: &IrcMsg) -> Result<(), ()>
    {
        // XXX: unimplemented!();
        Ok(())
    }
}

impl JoinBuf {
    pub fn new(channel: &[u8]) -> Result<JoinBuf, ()>
    {
        let mut wr = io::Cursor::new(Vec::new());
        try!(Join::construct(&mut wr, channel));

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(wr.into_inner()).map_err(|_| ()));

        // FIXME: try!(Join::validate(&message));
        Ok(JoinBuf { inner: message })
    }
}


impl_irc_msg_subtype!(Nick);
impl_irc_msg_subtype_buf!(NickBuf, Nick);

impl Nick {
    fn construct<W>(sink: &mut W, nick: &[u8]) -> Result<(), ()>
        where W: Write
    {
        writevec(sink, &[b"NICK ", nick]).or_else(cursor_chk_error)
    }

    /// Create a new `Nick` in `storage`.  This does not allocate any storage.
    pub fn new<'a>(storage: &'a mut [u8], nick: &[u8]) -> Result<&'a Nick, ()>
    {
        let mut wr = io::Cursor::new(storage);
        try!(Nick::construct(&mut wr, nick));
        let end = wr.position() as usize;

        let storage = wr.into_inner();
        Nick::parse(&storage[..end])
    }

    fn validate(_msg: &IrcMsg) -> Result<(), ()>
    {
        // XXX: unimplemented!();
        Ok(())
    }
}

impl NickBuf {
    pub fn new(nick: &[u8]) -> Result<NickBuf, ()>
    {
        let mut wr = io::Cursor::new(Vec::new());
        try!(Nick::construct(&mut wr, nick));

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(wr.into_inner()).map_err(|_| ()));

        // FIXME: try!(Nick::validate(&message));
        Ok(NickBuf { inner: message })
    }
}


impl_irc_msg_subtype!(Ping);
impl_irc_msg_subtype_buf!(PingBuf, Ping);

impl Ping {
    fn construct<W>(sink: &mut W, server: &[u8]) -> Result<(), ()>
        where W: Write
    {
        writevec(sink, &[b"PING :", server]).or_else(cursor_chk_error)
    }

    fn validate(_msg: &IrcMsg) -> Result<(), ()> {
        // XXX: unimplemented!();
        Ok(())
    }
}

impl PingBuf {
    pub fn new(server: &[u8]) -> Result<PingBuf, ()>
    {
        let mut wr = io::Cursor::new(Vec::new());
        try!(Ping::construct(&mut wr, server));

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(wr.into_inner()).map_err(|_| ()));

        // FIXME: try!(Ping::validate(&message));
        Ok(PingBuf { inner: message })
    }
}


impl_irc_msg_subtype!(Pong);
impl_irc_msg_subtype_buf!(PongBuf, Pong);

impl Pong {
    fn construct<W>(sink: &mut W, server: &[u8]) -> Result<(), ()>
        where W: Write
    {
        writevec(sink, &[b"PONG ", server]).or_else(cursor_chk_error)
    }

    fn validate(_msg: &IrcMsg) -> Result<(), ()>
    {
        // XXX: unimplemented!();
        Ok(())
    }
}

impl PongBuf {
    pub fn new(source: &[u8]) -> Result<PongBuf, ()>
    {
        let mut wr = io::Cursor::new(Vec::new());
        try!(Pong::construct(&mut wr, source));

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(wr.into_inner()).map_err(|_| ()));

        // FIXME: try!(Pong::validate(&message));
        Ok(PongBuf { inner: message })
    }
}

impl_irc_msg_subtype!(Privmsg);
impl_irc_msg_subtype_buf!(PrivmsgBuf, Privmsg);

impl Privmsg {
    fn construct<W>(sink: &mut W, target: &[u8], message: &[u8]) -> Result<(), ()>
        where W: Write
    {
        writevec(sink, &[
            b"PRIVMSG ",
            target,
            b" :",
            message,
        ]).or_else(cursor_chk_error)
    }

    fn validate(_msg: &IrcMsg) -> Result<(), ()>
    {
        // XXX: unimplemented!();
        Ok(())
    }
}

impl PrivmsgBuf {
    pub fn new(target: &[u8], message: &[u8]) -> Result<PrivmsgBuf, ()>
    {
        let mut wr = io::Cursor::new(Vec::new());
        try!(Privmsg::construct(&mut wr, target, message));

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(wr.into_inner()).map_err(|_| ()));

        // FIXME: try!(Privmsg::validate(&message));
        Ok(PrivmsgBuf { inner: message })
    }
}


impl_irc_msg_subtype!(Quit);
impl_irc_msg_subtype_buf!(QuitBuf, Quit);

impl Quit {
    fn construct<W>(sink: &mut W, reason: &[u8]) -> Result<(), ()>
        where W: Write
    {
        writevec(sink, &[b"QUIT :", reason]).or_else(cursor_chk_error)
    }

    fn validate(_msg: &IrcMsg) -> Result<(), ()>
    {
        // XXX: unimplemented!();
        Ok(())
    }
}

impl QuitBuf {
    pub fn new(reason: &[u8]) -> Result<QuitBuf, ()>
    {
        let mut wr = io::Cursor::new(Vec::new());
        try!(Quit::construct(&mut wr, reason));

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(wr.into_inner()).map_err(|_| ()));

        // FIXME: try!(Quit::validate(&message));
        Ok(QuitBuf { inner: message })
    }
}


impl_irc_msg_subtype!(User);
impl_irc_msg_subtype_buf!(UserBuf, User);


impl User {
    fn construct<W>(sink: &mut W,
        user: &[u8],
        mode: &[u8],
        unused: &[u8],
        realname: &[u8],
    ) -> Result<(), ()>
        where W: Write
    {
        writevec(sink, &[
            b"USER ",
            user, b" ",
            mode, b" ",
            unused,
            b" :",
            realname
        ]).or_else(cursor_chk_error)
    }

    /// Create a new `User` in `storage`.  This does not allocate any storage.
    /// USER <user> <mode> <unused> <realname>
    pub fn new<'a>(
        storage: &'a mut [u8],
        user: &[u8],
        mode: &[u8],
        unused: &[u8],
        realname: &[u8],
    ) -> Result<&'a User, ()>
    {
        let mut wr = io::Cursor::new(storage);
        try!(User::construct(&mut wr, user, mode, unused, realname));
        let end = wr.position() as usize;

        let storage = wr.into_inner();
        User::parse(&storage[..end])
    }

    fn validate(_msg: &IrcMsg) -> Result<(), ()> {
        // XXX: unimplemented!();
        Ok(())
    }
}

impl UserBuf {
    pub fn new(
        user: &[u8],
        mode: &[u8],
        unused: &[u8],
        realname: &[u8],
    ) -> Result<UserBuf, ()>
    {
        let mut wr = io::Cursor::new(Vec::new());
        try!(User::construct(&mut wr, user, mode, unused, realname));

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(wr.into_inner()).map_err(|_| ()));

        // FIXME: try!(User::validate(&message));
        Ok(UserBuf { inner: message })
    }
}


fn writevec<W: Write>(sink: &mut W, bufs: &[&[u8]]) -> io::Result<()>
{
    for &piece in bufs.iter() {
        try!(sink.write_all(piece));
    }
    Ok(())
}
