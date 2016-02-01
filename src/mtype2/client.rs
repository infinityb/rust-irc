use ::parse::IrcMsg as IrcMsgLegacy;
use ::parse::parse2::IrcMsg;
use ::parse_helpers;


macro_rules! impl_irc_msg_subtype {
    ($id:ident) => {
        pub struct $id {
            inner: IrcMsg,
        }

        impl $id {
            pub fn from_irc_msg(msg: &IrcMsg) -> Result<&Self, ()> {
                try!($id::validate(msg));
                Ok(unsafe {::std::mem::transmute(msg) })
            }

            pub fn to_irc_msg(&self) -> &IrcMsg {
                &self.inner
            }
        }
    }
}
