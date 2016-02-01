use std::borrow::{Borrow, BorrowMut, ToOwned};
use std::borrow::Cow;
use std::{mem, ops};
use std::io::{self, Write};

use super::FromIrcMsg;
use ::parse::IrcMsg as IrcMsgLegacy;
use ::parse::parse2::{IrcMsg, IrcMsgBuf};
use ::parse_helpers;
use ::cap::Capabilities;


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


// The subcommands for CAP are: LS, LIST, REQ, ACK, NAK, and END.

impl_irc_msg_subtype!(CapLs);
impl_irc_msg_subtype_buf!(CapLsBuf, CapLs);


// Client: CAP LS 302
// Server: CAP * LS * :multi-prefix extended-join account-notify batch invite-notify tls
// Server: CAP * LS * :cap-notify server-time example.org/dummy-cap=dummyvalue example.org/second-dummy-cap
// Server: CAP * LS :userhost-in-names sasl=EXTERNAL,DH-AES,DH-BLOWFISH,ECDSA-NIST256P-CHALLENGE,PLAIN

impl CapLs {
    fn validate(_msg: &IrcMsg) -> Result<(), ()> {
        unimplemented!();
    }

    fn construct<W>(sink: &mut W, source: &[u8], caps: &Capabilities) -> Result<(), ()>
        where W: Write
    {
        try!(sink.write_all(b":").or_else(cursor_chk_error));
        try!(sink.write_all(source).or_else(cursor_chk_error));
        try!(sink.write_all(b" CAP * LS :").or_else(cursor_chk_error));

        for cap_phrase in caps.iter_raw() {
            try!(sink.write_all(source).or_else(cursor_chk_error));
            try!(sink.write_all(b" ").or_else(cursor_chk_error));
        }

        Ok(())
    }

    /// True if this is the final line in a multi-line response.
    /// see: [IRCv3.2](http://ircv3.net/specs/core/capability-negotiation-3.2.html)
    pub fn is_final(&self) -> bool {
        unimplemented!();
    }

    pub fn capability_iter(&self) -> CapLsCapabilityIter {
        unimplemented!();
    }
}

impl CapLsBuf {
    pub fn new_unstable(source: &[u8], caps: &Capabilities) -> Result<CapLsBuf, ()> {
        let mut wr = io::Cursor::new(Vec::new());
        try!(CapLs::construct(&mut wr, source, caps));

        let message = try!(IrcMsgBuf::new(wr.into_inner()).map_err(|_| ()));
        try!(CapLs::validate(&message));
        Ok(CapLsBuf { inner: message })
    }
}

pub struct CapLsCapabilityIter;
pub struct CapLsGroupBuf;

impl_irc_msg_subtype!(CapList);
impl_irc_msg_subtype_buf!(CapListBuf, CapList);

// Client: CAP LIST
// Server: CAP modernclient LIST * :example.org/example-cap example.org/second-example-cap account-notify
// Server: CAP modernclient LIST :invite-notify batch example.org/third-example-cap

impl CapList {
    fn validate(_msg: &IrcMsg) -> Result<(), ()> {
        unimplemented!();
    }

    /// True if this is the final line in a multi-line response.
    /// see: [IRCv3.2](http://ircv3.net/specs/core/capability-negotiation-3.2.html)
    pub fn is_final(&self) -> bool {
        unimplemented!();
    }

    pub fn capabilities(&self) -> CapListCapabilityIter {
        unimplemented!();
    }
}

impl CapListBuf {
    //
}

pub struct CapListCapabilityIter;


impl_irc_msg_subtype!(CapReq);
impl_irc_msg_subtype_buf!(CapReqBuf, CapReq);

impl CapReq {
    fn validate(_msg: &IrcMsg) -> Result<(), ()> {
        unimplemented!();
    }
}

impl CapReqBuf {
    //
}

impl_irc_msg_subtype!(CapAck);
impl_irc_msg_subtype_buf!(CapAckBuf, CapAck);

// Client: CAP REQ :account-notify away-notify extended-join multi-prefix sasl
// Client: CAP END
// Server: CAP * ACK :account-notify away-notify extended-join multi-prefix sasl

impl CapAck {
    fn validate(_msg: &IrcMsg) -> Result<(), ()> {
        unimplemented!();
    }
}

impl CapAckBuf {
    //
}

impl_irc_msg_subtype!(CapNak);
impl_irc_msg_subtype_buf!(CapNakBuf, CapNak);

impl CapNak {
    fn validate(_msg: &IrcMsg) -> Result<(), ()> {
        unimplemented!();
    }
}

impl CapNakBuf {
    //
}

impl_irc_msg_subtype!(CapEnd);
impl_irc_msg_subtype_buf!(CapEndBuf, CapEnd);

impl CapEnd {
    fn construct<W>(sink: &mut W, source: &[u8]) -> Result<(), ()>
        where W: Write
    {
        try!(sink.write_all(b":").or_else(cursor_chk_error));
        try!(sink.write_all(source).or_else(cursor_chk_error));
        try!(sink.write_all(b" CAP END").or_else(cursor_chk_error));
        Ok(())
    }

    /// Create a new `CapEnd` in `storage`.  This does not allocate any storage.
    pub fn new<'a>(storage: &'a mut [u8], source: &[u8]) -> Result<&'a CapEnd, ()> {
        let mut wr = io::Cursor::new(storage);
        try!(CapEnd::construct(&mut wr, source));
        let end = wr.position() as usize;

        let storage = wr.into_inner();
        CapEnd::parse(&storage[..end])
    }

    fn validate(_msg: &IrcMsg) -> Result<(), ()> {
        unimplemented!();
    }
}

impl CapEndBuf {
    pub fn new(source: &[u8]) -> Result<CapEndBuf, ()> {
        let mut wr = io::Cursor::new(Vec::new());
        try!(CapEnd::construct(&mut wr, source));

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(wr.into_inner()).map_err(|_| ()));

        try!(CapEnd::validate(&message));
        Ok(CapEndBuf { inner: message })
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
    fn construct<W>(sink: &mut W, source: &[u8], channel: &[u8]) -> Result<(), ()>
        where W: Write
{
        try!(sink.write_all(b":").or_else(cursor_chk_error));
        try!(sink.write_all(source).or_else(cursor_chk_error));
        try!(sink.write_all(b" JOIN ").or_else(cursor_chk_error));
        try!(sink.write_all(channel).or_else(cursor_chk_error));

        Ok(())
    }

    /// Create a new `Join` in `storage`.  This does not allocate any storage.
    pub fn new<'a>(storage: &'a mut [u8], source: &[u8], channel: &[u8]) -> Result<&'a Join, ()> {
        let mut wr = io::Cursor::new(storage);
        try!(Join::construct(&mut wr, source, channel));
        let end = wr.position() as usize;

        let storage = wr.into_inner();
        Join::parse(&storage[..end])
    }

    pub fn get_nick(&self) -> &str {
        let (nick, _, _) = parse_helpers::parse_prefix(self.get_source()).unwrap();
        ::std::str::from_utf8(nick).unwrap()
    }
}

impl JoinBuf {
    /// Create a new `JoinBuf`.  Allocates storage.
    pub fn new(source: &[u8], channel: &[u8]) -> Result<JoinBuf, ()> {
        let mut wr = io::Cursor::new(Vec::new());
        try!(Join::construct(&mut wr, source, channel));

        // maybe we could skip this check later and turn it into a debug-assert?
        let message = try!(IrcMsgBuf::new(wr.into_inner()).map_err(|_| ()));

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
irc_msg_has_source!(Privmsg);
irc_msg_has_target!(Privmsg);

impl Privmsg {
    pub fn validate(msg: &IrcMsg) -> Result<(), ()> {
        use std::ascii::AsciiExt;

        let buf = msg.as_bytes();

        let (prefix, rest) = parse_helpers::split_prefix(buf);
        if prefix.len() == 0 {
            return Err(());
        }
        if !parse_helpers::is_valid_prefix(prefix) {
            return Err(());
        }

        let (command, rest) = parse_helpers::split_command(rest);
        if !AsciiExt::eq_ignore_ascii_case(command, b"PRIVMSG") {
            return Err(());
        }

        let (target, rest) = parse_helpers::split_arg(rest);
        if target.len() < 1 {
            return Err(());
        }

        let (_body, must_be_empty) = parse_helpers::split_arg(rest);
        if must_be_empty.len() != 0 {
            return Err(());
        }

        Ok(())
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


fn cursor_chk_error(err: io::Error) -> Result<(), ()> {
    match err {
        ref err if err.kind() == io::ErrorKind::WriteZero => Err(()),
        _ => panic!(),
    }
}



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
fn privmsg_create_and_check() {
    let privmsg_buf = PrivmsgBuf::new(b"n!u@h", b"#mychannel", b"Hello!").unwrap();
    assert_eq!(privmsg_buf.get_target(), b"#mychannel");
    assert_eq!(privmsg_buf.get_body_raw(), b"Hello!");
}

#[test]
fn privmsg_buf_validity() {
    let _privmsg = PrivmsgBuf::new(b"n!u@h", b"#foo", b"swagever").unwrap();
}

#[test]
fn privmsg_cons_with_bad_arguments() {
    let msg = PrivmsgBuf::new(b"n!u @h", b"#foo", b"BREAKIN DA RULEZ");
    println!("msg.is_err() = {:?}", msg.is_err());
    assert_eq!(msg.is_err(), true);

    let msg = PrivmsgBuf::new(b"n!u\n@h", b"#foo", b"BREAKIN DA RULEZ");
    println!("msg.is_err() = {:?}", msg.is_err());
    assert_eq!(msg.is_err(), true);

    let msg = PrivmsgBuf::new(b"n!u\x00@h", b"#foo", b"BREAKIN DA RULEZ");
    println!("msg.is_err() = {:?}", msg.is_err());
    assert_eq!(msg.is_err(), true);

    let msg = PrivmsgBuf::new(b"n!u\r@h", b"#foo", b"BREAKIN DA RULEZ");
    println!("msg.is_err() = {:?}", msg.is_err());
    assert_eq!(msg.is_err(), true);

    let msg = PrivmsgBuf::new(b"n!u@h", b"# foo", b"BREAKIN DA RULEZ");
    println!("msg.is_err() = {:?}", msg.is_err());
    assert_eq!(msg.is_err(), true);

    let msg = PrivmsgBuf::new(b"n!u@h", b"#\nfoo", b"BREAKIN DA RULEZ");
    println!("msg.is_err() = {:?}", msg.is_err());
    assert_eq!(msg.is_err(), true);

    let msg = PrivmsgBuf::new(b"n!u@h", b"#\x00foo", b"BREAKIN DA RULEZ");
    println!("msg.is_err() = {:?}", msg.is_err());
    assert_eq!(msg.is_err(), true);

    let msg = PrivmsgBuf::new(b"n!u@h", b"#\rfoo", b"BREAKIN DA RULEZ");
    println!("msg.is_err() = {:?}", msg.is_err());
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

#[test]
fn privmsg_lowercase() {
    let _privmsg = Privmsg::parse(b":n!u@h privmsg #target :body").unwrap();
}

#[test]
fn construct_join_on_stack() {
    let mut my_mem = [0; 1024];
    let len = Join::new(&mut my_mem, b"n!u@h", b"#somewhere").unwrap().as_bytes().len();

    assert_eq!(&my_mem[..len], b":n!u@h JOIN #somewhere");
    for &byte in my_mem[len..].iter() {
        assert_eq!(byte, 0);
    }
}


#[test]
fn construct_join_on_stack_too_small() {
    let mut my_mem = [0; 20];
    if let Ok(_join) = Join::new(&mut my_mem, b"n!u@h", b"#somewhere") {
        panic!("created JOIN in insufficient buffer: {:?}", _join.as_bytes());
    }
}

