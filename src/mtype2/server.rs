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
irc_msg_legacy_validator!(Invite, Invite);

impl_irc_msg_subtype!(Join);
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
        unsafe { ::std::str::from_utf8_unchecked(nick) }
    }
}


impl_irc_msg_subtype!(Kick);
irc_msg_legacy_validator!(Kick, Kick);


impl_irc_msg_subtype!(Mode);
irc_msg_legacy_validator!(Mode, Mode);


impl_irc_msg_subtype!(Nick);
irc_msg_legacy_validator!(Nick, Nick);


impl_irc_msg_subtype!(Notice);
irc_msg_legacy_validator!(Notice, Notice);


impl_irc_msg_subtype!(Part);
irc_msg_legacy_validator!(Part, Part);


impl_irc_msg_subtype!(Ping);
irc_msg_legacy_validator!(Ping, Ping);


impl_irc_msg_subtype!(Pong);
irc_msg_legacy_validator!(Pong, Pong);


impl_irc_msg_subtype!(Privmsg);
irc_msg_legacy_validator!(Privmsg, Privmsg);


impl_irc_msg_subtype!(Topic);
irc_msg_legacy_validator!(Topic, Topic);


impl_irc_msg_subtype!(Quit);
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

