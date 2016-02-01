use std::borrow::{Borrow, BorrowMut, ToOwned};
use std::borrow::Cow;
use std::{mem, ops};
use std::io::{self, Write};

use super::FromIrcMsg;
use ::parse::IrcMsg as IrcMsgLegacy;
use ::parse::parse2::{IrcMsg, IrcMsgBuf};
use ::parse_helpers;
use ::cap::NegotiationVersion;


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


impl_irc_msg_subtype!(CapLs);
impl_irc_msg_subtype_buf!(CapLsBuf, CapLs);

impl CapLs {
    fn construct<W>(sink: &mut W, version: NegotiationVersion) -> Result<(), ()>
        where W: Write
    {
        try!(sink.write_all(b"CAP LS ").or_else(cursor_chk_error));
        match version {
            NegotiationVersion::V301 => {
                try!(sink.write_all(b"301").or_else(cursor_chk_error));
            }
            NegotiationVersion::V302 => {
                try!(sink.write_all(b"302").or_else(cursor_chk_error));
            }
        }
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


fn cursor_chk_error(err: io::Error) -> Result<(), ()> {
    match err {
        ref err if err.kind() == io::ErrorKind::WriteZero => Err(()),
        _ => panic!(),
    }
}
