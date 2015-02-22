use std::borrow::{Cow, IntoCow};

use parse::IrcMsg;
use core_plugins::traits::MessageResponder;
use message_types::{client, server};

static VERSION: &'static str = "rust-irc v0.1.0 https://github.com/infinityb/rust-irc";


/// Responds to CTCP Version requests
pub struct CtcpVersionResponder {
    customized: Option<String>,
    include_rust_irc: bool,
}

impl CtcpVersionResponder {
    fn get_version(&self) -> Cow<'static, str> {
        match (self.include_rust_irc, &self.customized) {
            (_, &None) => VERSION.into_cow(),
            (true, &Some(ref customized)) => {
                let string = format!("{:?} ({:?})", customized.as_slice(), VERSION);
                string.into_cow()
            },
            (false, &Some(ref customized)) => {
               customized.clone().into_cow()
            }
        }
    }
}


impl MessageResponder for CtcpVersionResponder {
    fn on_irc_msg(&mut self, msg: &IrcMsg) -> Vec<IrcMsg> {
        let ty_msg = server::IncomingMsg::from_msg(msg.clone());

        let mut out = Vec::new();
        if let server::IncomingMsg::Privmsg(ref msg) = ty_msg {
            if msg.get_body_raw() == b"\x01VERSION\x01" {
                let mut vec = Vec::new();
                vec.push_all(b"VERSION ");
                vec.push_all(self.get_version().as_bytes());
                let privmsg = client::Privmsg::new_ctcp(msg.get_target(), vec.as_slice());
                out.push(privmsg.into_irc_msg());
            }
        }
        out
    }
}
