use std::str::CowString;

use parse::IrcMsg;
use message::IrcMessage;
use core_plugins::traits::MessageResponder;
use message_types::{client, server};

static VERSION: &'static str = "rust-irc v0.1.0 https://github.com/infinityb/rust-irc";


/// Responds to CTCP Version requests
pub struct CtcpVersionResponder {
    customized: Option<String>,
    include_rust_irc: bool,
}

impl CtcpVersionResponder {
    pub fn new() -> CtcpVersionResponder {
        CtcpVersionResponder {
            customized: None,
            include_rust_irc: true
        }
    }

    pub fn set_include_rust_irc(&mut self, value: bool) {
        self.include_rust_irc = value;
    }

    pub fn set_version(&mut self, version: &str) {
        self.customized = Some(version.to_string());
    }

    fn get_version(&self) -> CowString {
        match (self.include_rust_irc, &self.customized) {
            (_, &None) => VERSION.into_cow(),
            (true, &Some(ref customized)) => {
                let string = format!("{} ({})", customized[], VERSION);
                string.into_cow()
            },
            (false, &Some(ref customized)) => {
               customized.clone().into_cow()
            }
        }
    }
}


impl MessageResponder for CtcpVersionResponder {
    fn on_message(&mut self, message: &IrcMessage) -> Vec<IrcMsg> {
        let mut out = Vec::new();
        if let server::IncomingMsg::Privmsg(ref msg) = *message.get_typed_message() {
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
