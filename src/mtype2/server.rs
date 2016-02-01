use std::borrow::{Borrow, BorrowMut, ToOwned};
use std::borrow::Cow;
use std::{mem, ops};

use super::FromIrcMsg;
use ::parse::IrcMsg as IrcMsgLegacy;
use ::parse::parse2::{IrcMsg, IrcMsgBuf};
use ::parse_helpers;


macro_rules! impl_irc_msg_subtype {
    ($id:ident) => {
        pub struct $id {
            inner: IrcMsg,
        }

        impl $id {
            /// The following function allows unchecked construction of a irc message
            /// from a u8 slice.  This is unsafe because it does not maintain
            /// the invariant of the message type, nor the invariant of IrcMsg
            pub unsafe fn from_u8_slice_unchecked(s: &[u8]) -> &$id {
                mem::transmute(s)
            }

            pub fn to_irc_msg(&self) -> &IrcMsg {
                &self.inner
            }

            pub fn parse(buffer: &[u8]) -> Result<&$id, ()> {
                // maybe we could skip this check later and turn it into a debug-assert?
                let message = try!(IrcMsg::new(buffer).map_err(|_| ()));
                try!($id::validate(message));

                Ok(unsafe { $id::from_u8_slice_unchecked(buffer) })
            }
        }

        impl ops::Deref for $id {
            type Target = IrcMsg;

            fn deref<'a>(&'a self) -> &'a IrcMsg {
                &self.inner
            }
        }

        impl<'a> FromIrcMsg for &'a $id {
            type Err = ();

            fn from_irc_msg(msg: &IrcMsg) -> Result<&'a $id, ()> {
                try!($id::validate(msg));
                Ok(unsafe {::std::mem::transmute(msg) })
            }
        }
    }
}

macro_rules! impl_irc_msg_subtype_buf {
    ($id:ident, $borrowed:ident) => {
        pub struct $id {
            inner: IrcMsgBuf,
        }

        impl $id {
            fn _borrow(&self) -> &$borrowed {
                unsafe { $borrowed::from_u8_slice_unchecked(self.inner.as_bytes()) }
            }

            pub fn into_inner(self) -> IrcMsgBuf {
                self.inner
            }
        }

        impl AsRef<$borrowed> for $id {
            fn as_ref(&self) -> &$borrowed {
                self._borrow()
            }
        }

        impl ops::Deref for $id {
            type Target = $borrowed;

            fn deref<'a>(&'a self) -> &'a $borrowed {
                self._borrow()
            }
        }

        impl Borrow<$borrowed> for $id {
            fn borrow(&self) -> &$borrowed {
                self._borrow()
            }
        }

        impl ToOwned for $borrowed {
            type Owned = $id;

            fn to_owned(&self) -> $id {
                $id { inner: self.inner.to_owned() }
            }
        }
    }
}

macro_rules! irc_msg_has_source {
    ($id:ident) => {
        impl $id {
            pub fn get_source(&self) -> &[u8] {
                let buf = self.as_bytes();
                let (prefix, _rest) = parse_helpers::split_prefix(buf);
                prefix
            }
        }
    }
}

macro_rules! irc_msg_has_target {
    ($id:ident) => {
        impl $id {
            pub fn get_target(&self) -> &[u8] {
                let buf = self.as_bytes();
                let (_prefix, rest) = parse_helpers::split_prefix(buf);
                let (_command, rest) = parse_helpers::split_command(rest);
                let (target, _rest) = parse_helpers::split_arg(rest);
                target
            }
        }
    }
}

macro_rules! irc_msg_legacy_validator {
    ($on:ident, $from:ident) => {
        impl $on {
            fn validate(msg: &IrcMsg) -> Result<(), ()> {
                use message_types::server;

                let legacy = try!(IrcMsgLegacy::new(msg.as_bytes().to_vec()).map_err(|_| ()));
                match server::IncomingMsg::from_msg(legacy.clone()) {
                    server::IncomingMsg::$from(ref _msg) => (),
                    _ => return Err(()),
                };
                Ok(())
            }
        }
    }
}

impl_irc_msg_subtype!(Invite);
impl_irc_msg_subtype_buf!(InviteBuf, Invite);
irc_msg_legacy_validator!(Invite, Invite);

impl_irc_msg_subtype!(Join);
impl_irc_msg_subtype_buf!(JoinBuf, Join);
irc_msg_legacy_validator!(Join, Join);
irc_msg_has_source!(Join);
irc_msg_has_target!(Join);

impl Join {
    pub fn get_nick(&self) -> &str {
        let (nick, _, _) = parse_helpers::parse_prefix(self.get_source()).unwrap();
        ::std::str::from_utf8(nick).unwrap()
    }
}

impl JoinBuf {
    pub fn new(channel: &[u8]) -> Result<JoinBuf, ()> {
        let mut out: Vec<u8> = Vec::new();
        out.extend(b"JOIN ");
        out.extend(channel);

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(out).map_err(|_| ()));

        try!(Join::validate(&message));
        Ok(JoinBuf { inner: message })
    }
}


impl_irc_msg_subtype!(Kick);
impl_irc_msg_subtype_buf!(KickBuf, Kick);
irc_msg_legacy_validator!(Kick, Kick);
irc_msg_has_source!(Kick);
irc_msg_has_target!(Kick);


impl KickBuf {
    pub fn new(source: &[u8], channel: &[u8], who: &[u8], reason: Option<&[u8]>) -> Result<KickBuf, ()> {
        let mut out: Vec<u8> = Vec::new();
        out.extend(b":");
        out.extend(source);
        out.extend(b" KICK ");
        out.extend(channel);
        out.extend(b" ");
        out.extend(who);
        if let Some(reason) = reason {
            out.extend(b" :");
            out.extend(reason);
        }

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(out).map_err(|_| ()));

        try!(Kick::validate(&message));
        Ok(KickBuf { inner: message })
    }
}


impl_irc_msg_subtype!(Mode);
impl_irc_msg_subtype_buf!(ModeBuf, Mode);
irc_msg_legacy_validator!(Mode, Mode);
irc_msg_has_source!(Mode);
irc_msg_has_target!(Mode);


impl_irc_msg_subtype!(Nick);
impl_irc_msg_subtype_buf!(NickBuf, Nick);
irc_msg_legacy_validator!(Nick, Nick);


impl_irc_msg_subtype!(Notice);
impl_irc_msg_subtype_buf!(NoticeBuf, Notice);
irc_msg_legacy_validator!(Notice, Notice);
irc_msg_has_source!(Notice);
irc_msg_has_target!(Notice);


impl_irc_msg_subtype!(Part);
impl_irc_msg_subtype_buf!(PartBuf, Part);
irc_msg_legacy_validator!(Part, Part);
irc_msg_has_source!(Part);
irc_msg_has_target!(Part);


impl_irc_msg_subtype!(Ping);
impl_irc_msg_subtype_buf!(PingBuf, Ping);
irc_msg_legacy_validator!(Ping, Ping);


impl_irc_msg_subtype!(Pong);
impl_irc_msg_subtype_buf!(PongBuf, Pong);
irc_msg_legacy_validator!(Pong, Pong);


impl_irc_msg_subtype!(Privmsg);
impl_irc_msg_subtype_buf!(PrivmsgBuf, Privmsg);
irc_msg_legacy_validator!(Privmsg, Privmsg);
irc_msg_has_source!(Privmsg);
irc_msg_has_target!(Privmsg);

impl Privmsg {
    pub fn get_body_raw(&self) -> &[u8] {
        let buf = self.as_bytes();
        let (_prefix, rest) = parse_helpers::split_prefix(buf);
        let (_command, rest) = parse_helpers::split_command(rest);
        let (_target, rest) = parse_helpers::split_arg(rest);
        let (body, must_be_empty) = parse_helpers::split_arg(rest);

        // FIXME: not guaranteed yet???
        assert_eq!(must_be_empty.len(), 0);

        body
    }
}

impl PrivmsgBuf {
    pub fn new(source: &[u8], target: &[u8], body: &[u8]) -> Result<PrivmsgBuf, ()> {
        let mut out: Vec<u8> = Vec::new();
        out.extend(b":");
        out.extend(source);
        out.extend(b" PRIVMSG ");
        out.extend(target);
        out.extend(b" :");
        out.extend(body);

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(out).map_err(|_| ()));
        try!(Privmsg::validate(&message));
        Ok(PrivmsgBuf { inner: message })
    }
}

#[test]
fn privmsg_create_and_check() {
    let privmsg_buf = PrivmsgBuf::new(b"n!u@h", b"#mychannel", b"Hello!").unwrap();
    assert_eq!(privmsg_buf.get_target(), b"#mychannel");
    assert_eq!(privmsg_buf.get_body_raw(), b"Hello!");
}

impl_irc_msg_subtype!(Topic);
impl_irc_msg_subtype_buf!(TopicBuf, Topic);
irc_msg_legacy_validator!(Topic, Topic);
irc_msg_has_source!(Topic);
irc_msg_has_target!(Topic);

impl Topic {
}

impl_irc_msg_subtype!(Quit);
impl_irc_msg_subtype_buf!(QuitBuf, Quit);
irc_msg_legacy_validator!(Quit, Quit);


#[test]
fn kick_asrefs() {
    fn kick_acceptor(_: &Kick) {}
    fn ircmsg_acceptor(_: &IrcMsg) {}

    let kick = KickBuf::new(b"n!u@h", b"#foo", b"you", Some(b"BREAKIN DA RULEZ")).unwrap();
    kick_acceptor(&kick);
    ircmsg_acceptor(&kick);
}

#[test]
fn kick_derefs() {
    let kick = KickBuf::new(b"n!u@h", b"#foo", b"you", Some(b"BREAKIN DA RULEZ")).unwrap();
    assert_eq!("KICK", kick.get_command());
}

#[test]
fn kickbufs_from_borrowed() {
    let kick = KickBuf::new(b"n!u@h", b"#foo", b"you", Some(b"BREAKIN DA RULEZ")).unwrap();
    let kick_ref: &Kick = &kick;

    let new_kick: KickBuf = kick_ref.to_owned();
    assert_eq!("KICK", new_kick.get_command());
}

#[test]
fn privmsg_buf_validity() {
    let _privmsg = PrivmsgBuf::new(b"n!u@h", b"#foo", b"swagever").unwrap();
}

#[test]
fn privmsg_cons_with_bad_arguments() {
    let msg = PrivmsgBuf::new(b"n!u\xFF@h", b"#foo", b"BREAKIN DA RULEZ");

    assert_eq!(msg.is_err(), true);
}

#[test]
fn rustbot_excerpt_001() {
    let msg = PrivmsgBuf::new(b"n!u@h", b"#foo", b"BREAKIN DA RULEZ").unwrap();

    let privmsg: &Privmsg;
    if let Ok(p) = msg.as_tymsg::<&Privmsg>() {
        privmsg = p;
    } else {
        unreachable!();
    }

    if privmsg.get_target().starts_with(b"#") {
        let out = privmsg.get_body_raw().to_vec();

        let response = PrivmsgBuf::new(b"n!u@h", privmsg.get_target(), &out[..]).unwrap();
        let _: &IrcMsg = &response;
    } else {
        unreachable!();
    }
}

#[test]
fn privmsg_too_many_args() {
    if let Ok(privmsg) = Privmsg::parse(b":n!u@h PRIVMSG #target wtfisthis :body") {
        panic!("privmsg = {:?}", privmsg.as_bytes());
    }
}