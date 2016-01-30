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
                use ::legacy::message_types::server;

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
