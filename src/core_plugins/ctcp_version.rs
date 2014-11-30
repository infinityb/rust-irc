use std::str::{MaybeOwned, IntoMaybeOwned};

use message::IrcMessage;
use core_plugins::traits::MessageResponder;


static VERSION: &'static str = "rust-irc v0.1.0 https://github.com/infinityb/rust-irc";

type OwnedOrStatic = MaybeOwned<'static>;


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

    fn get_version(&self) -> OwnedOrStatic {
        match (self.include_rust_irc, &self.customized) {
            (_, &None) => VERSION.into_maybe_owned(),
            (true, &Some(ref customized)) => {
                let string = format!("{} ({})", customized[], VERSION);
                string.into_maybe_owned()
            },
            (false, &Some(ref customized)) => {
               customized.clone().into_maybe_owned()
            }
        }
    }
}


impl MessageResponder for CtcpVersionResponder {
    fn on_message(&mut self, message: &IrcMessage) -> Vec<String> {
        let mut out = Vec::new();
        let args = message.get_args();
        if message.command() == "PRIVMSG" && args.len() >= 2 {
            match (args[1], message.source_nick()) {
                ("\x01VERSION\x01", Some(source_nick)) => {
                    let body = format!(
                        "\x01VERSION {}\x01",
                        self.get_version().as_slice());
                    out.push(format!("PRIVMSG {} :{}", source_nick[], body[]));
                },
                _ => ()
            }
        }
        out
    }
}
