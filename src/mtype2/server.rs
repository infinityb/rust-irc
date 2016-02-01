use std::borrow::{Borrow, BorrowMut, ToOwned};
use std::borrow::Cow;
use std::{mem, ops};

use super::FromIrcMsg;
use ::parse::IrcMsg as IrcMsgLegacy;
use ::parse::parse2::{IrcMsg, IrcMsgBuf};
use ::parse_helpers;

#[derive(Copy, Clone)]
pub enum IncomingMsg<'a> {
    Invite(&'a Invite),
    Join(&'a Join),
    Kick(&'a Kick),
    Mode(&'a Mode),
    Nick(&'a Nick),
    Notice(&'a Notice),
    Part(&'a Part),
    Ping(&'a Ping),
    Pong(&'a Pong),
    Privmsg(&'a Privmsg),
    Quit(&'a Quit),
    Topic(&'a Topic),

    // Others
    Unknown(&'a IrcMsg),
}


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


macro_rules! irc_msg_legacy_validator {
    ($on:ident, $from:ident) => {
        impl $on {
            fn validate(msg: &IrcMsg) -> Result<(), ()> {
                use message_types::server;

                let legacy = IrcMsgLegacy::new(msg.as_bytes().to_vec()).unwrap();
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

impl Join {
    pub fn get_channel(&self) -> &[u8] {
        let buf = self.inner.as_bytes();
        let (_prefix, rest) = parse_helpers::split_prefix(buf);
        let (_command, rest) = parse_helpers::split_command(rest);
        let (channel, _rest) = parse_helpers::split_arg(rest);

        channel
    }

    pub fn get_nick(&self) -> &str {
        let buf = self.inner.as_bytes();
        let (prefix, _rest) = parse_helpers::split_prefix(buf);
        let (nick, _, _) = parse_helpers::parse_prefix(prefix).unwrap();
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

impl KickBuf {
    pub fn new(channel: &[u8], who: &[u8], reason: Option<&[u8]>) -> Result<KickBuf, ()> {
        let mut out: Vec<u8> = Vec::new();
        out.extend(b"KICK ");
        out.extend(channel);
        out.extend(b" ");
        out.extend(who);
        if let Some(reason) = reason {
            out.extend(b" :");
            out.extend(reason);
        }

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(out).map_err(|_| ()));

        // try!(Kick::validate(&message));
        Ok(KickBuf { inner: message })
    }
}


impl_irc_msg_subtype!(Mode);
impl_irc_msg_subtype_buf!(ModeBuf, Mode);
irc_msg_legacy_validator!(Mode, Mode);


impl_irc_msg_subtype!(Nick);
impl_irc_msg_subtype_buf!(NickBuf, Nick);
irc_msg_legacy_validator!(Nick, Nick);


impl_irc_msg_subtype!(Notice);
impl_irc_msg_subtype_buf!(NoticeBuf, Notice);
irc_msg_legacy_validator!(Notice, Notice);


impl_irc_msg_subtype!(Part);
impl_irc_msg_subtype_buf!(PartBuf, Part);
irc_msg_legacy_validator!(Part, Part);


impl_irc_msg_subtype!(Ping);
impl_irc_msg_subtype_buf!(PingBuf, Ping);
irc_msg_legacy_validator!(Ping, Ping);


impl_irc_msg_subtype!(Pong);
impl_irc_msg_subtype_buf!(PongBuf, Pong);
irc_msg_legacy_validator!(Pong, Pong);


impl_irc_msg_subtype!(Privmsg);
impl_irc_msg_subtype_buf!(PrivmsgBuf, Privmsg);
irc_msg_legacy_validator!(Privmsg, Privmsg);

impl Privmsg {
    pub fn get_target(&self) -> &[u8] {
        let buf = self.as_bytes();
        let (_prefix, rest) = parse_helpers::split_prefix(buf);
        let (_command, rest) = parse_helpers::split_command(rest);
        let (target, _rest) = parse_helpers::split_arg(rest);
        target
    }

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
    pub fn new(target: &[u8], body: &[u8]) -> Result<PrivmsgBuf, ()> {
        let mut out: Vec<u8> = Vec::new();
        out.extend(b"PRIVMSG ");
        out.extend(target);
        out.extend(b" :");
        out.extend(body);

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(out).map_err(|_| ()));

        // try!(Kick::validate(&message));
        Ok(PrivmsgBuf { inner: message })
    }
}

#[test]
fn privmsg_create_and_check() {
    let privmsg_buf = PrivmsgBuf::new(b"#mychannel", b"Hello!").unwrap();
    assert_eq!(privmsg_buf.get_target(), b"#mychannel");
    assert_eq!(privmsg_buf.get_body_raw(), b"Hello!");
}

impl_irc_msg_subtype!(Topic);
impl_irc_msg_subtype_buf!(TopicBuf, Topic);
irc_msg_legacy_validator!(Topic, Topic);


impl_irc_msg_subtype!(Quit);
impl_irc_msg_subtype_buf!(QuitBuf, Quit);
irc_msg_legacy_validator!(Quit, Quit);


// impl_irc_msg_subtype!(Numeric);

// impl Numeric {
//     fn validate(msg: &IrcMsg) -> Result<(), ()> {
//         use message_types::server;

//         let legacy = IrcMsgLegacy::new(msg.as_bytes().to_vec()).unwrap();
//         match server::IncomingMsg::from_msg(legacy.clone()) {
//             server::IncomingMsg::Numeric(num, ref _msg) => (),
//             _ => return Err(()),
//         };
//         Ok(())
//     }
// }

#[test]
fn kick_asrefs() {
    fn kick_acceptor(_: &Kick) {}
    fn ircmsg_acceptor(_: &IrcMsg) {}

    let kick = KickBuf::new(b"#foo", b"you", Some(b"BREAKIN DA RULEZ")).unwrap();
    kick_acceptor(&kick);
    ircmsg_acceptor(&kick);
}

#[test]
fn kick_derefs() {
    let kick = KickBuf::new(b"#foo", b"you", Some(b"BREAKIN DA RULEZ")).unwrap();
    assert_eq!("KICK", kick.get_command());
}

#[test]
fn kickbufs_from_borrowed() {
    let kick = KickBuf::new(b"#foo", b"you", Some(b"BREAKIN DA RULEZ")).unwrap();
    let kick_ref: &Kick = &kick;

    let new_kick: KickBuf = kick_ref.to_owned();
    assert_eq!("KICK", new_kick.get_command());
}
