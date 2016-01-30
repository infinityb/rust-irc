use std::borrow::{Borrow, BorrowMut, ToOwned};
use std::borrow::Cow;
use std::{mem, ops};
use std::io::{self, Write};

use super::super::FromIrcMsg;
use super::super::cursor_chk_error;

use ::legacy::IrcMsg as IrcMsgLegacy;
use ::{IrcMsg, IrcMsgBuf};
use ::parse_helpers;

use ::cap::Capabilities;

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

    fn construct<'a, W, I>(sink: &mut W, source: &[u8], caps: &mut I) -> Result<(), ()>
        where
            W: Write,
            I: Iterator<Item=&'a str>,
    {
        try!(sink.write_all(b":").or_else(cursor_chk_error));
        try!(sink.write_all(source).or_else(cursor_chk_error));
        try!(sink.write_all(b" CAP * LS :").or_else(cursor_chk_error));

        for cap_phrase in caps.iter_raw() {
            try!(sink.write_all(cap_phrase.as_bytes()).or_else(cursor_chk_error));
            try!(sink.write_all(b" ").or_else(cursor_chk_error));
        }

        Ok(())
    }

    fn construct_partial<'a, W, I>(sink: &mut W, source: &[u8], caps: &mut I) -> Result<bool, ()>
        where
            W: Write,
            I: Peekable<Iterator<Item=&'a str>>,
    {
        const MAX_MESSAGE: usize = 256; // really 512 but let's play it safe.
        const PREFIX_STARTER: &[u8] = b":";
        const CAP_COMMAND: &[u8] = b" CAP * LS :";

        let base_message = PREFIX_STARTER.len() + source.len() + CAP_COMMAND.len();
        try!(sink.write_all(PREFIX_STARTER).or_else(cursor_chk_error));
        try!(sink.write_all(source).or_else(cursor_chk_error));
        try!(sink.write_all(CAP_COMMAND).or_else(cursor_chk_error));


        let mut is_finished = true;

        while let Some(&cap) = caps.peek() {
            if MAX_MESSAGE <= bytes_written + cap.len() {
                is_finished = false;
                break;
            }
            if MAX_MESSAGE <= base_message + cap.len() {
                // We'll never be able to emit this message.
                return Err(());
            }

            let cap_bytes = caps.next().unwrap().as_bytes();
            bytes_written += cap_bytes.len() + 1;

            try!(sink.write_all(cap_bytes).or_else(cursor_chk_error));
            try!(sink.write_all(b" ").or_else(cursor_chk_error));
        }

        Ok(is_finished)
    }

    /// True if this is the final line in a multi-line response.
    /// see: [IRCv3.2](http://ircv3.net/specs/core/capability-negotiation-3.2.html)
    pub fn is_final(&self) -> bool {
        unimplemented!();
    }

    pub fn capabilities(&self) -> Capabilities {
        unimplemented!();
    }

    pub fn capability_raw_iter(&self) -> CapLsCapabilityRawIter {
        unimplemented!();
    }
}

impl CapLsBuf {
    fn _new(source: &[u8], caps: &Capabilities) -> Result<CapLsBuf, ()> {
    }

    #[cfg(feature = "unstable")]
    pub fn new_unstable(source: &[u8], caps: &Capabilities) -> Result<CapLsBuf, ()> {
        let mut wr = io::Cursor::new(Vec::new());
        try!(CapLs::construct(&mut wr, source, caps));

        let message = try!(IrcMsgBuf::new(wr.into_inner()).map_err(|_| ()));
        try!(CapLs::validate(&message));
        Ok(CapLsBuf { inner: message })
    }
}

pub struct CapLsCapabilityRawIter;

pub struct CapLsGroupBuilder<'a> {
    source: &'a [u8],
    caps: &'a Capabilities,
}

impl CapLsGroupBuilder {
    pub fn build(&self) -> Result<CapLsGroupBuf, ()> {
        let wr = io::Cursor::new(Vec::new());

        let caps = self.caps.iter().peekable();
        loop {
            let finished = try!(CapLs::construct_partial(&mut wr, self.source, &mut caps));
            try!(wr.write_all(b"\r\n").or_else(cursor_chk_error));
            if finished {
                break;
            }
        }

        CapLsGroupBuf { data: wr.into_inner() }
    }
}

pub struct CapLsGroupBuf {
    data: Vec<u8>,
}


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
