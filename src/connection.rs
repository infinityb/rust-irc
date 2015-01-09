use std::collections::RingBuf;
use std::sync::Future;
use std::fmt;

use core_plugins::{
    MessageResponder,
    CtcpVersionResponder,
};

use parse::IrcMsg;
use event::IrcEvent;
use message_types::{client, server};
pub use connection::IrcConnectionCommand::{
    RawWrite,
    AddWatcher,
    AddBundler,
};

use watchers::{
    Bundler,
    BundlerManager,
    RegisterEventWatcher,
    RegisterResult,
    JoinBundlerTrigger,
    JoinResult,
    JoinEventWatcher,
    WhoBundlerTrigger,
    WhoResult,
    WhoEventWatcher,
    EventWatcher,
    BundlerTrigger,
};

pub enum IrcConnectionCommand {
    RawWrite(Vec<u8>),
    AddWatcher(Box<EventWatcher+Send>),
    AddBundler(Box<Bundler+Send>),
}

impl IrcConnectionCommand {
    pub fn raw_write(message: Vec<u8>) -> IrcConnectionCommand {
        IrcConnectionCommand::RawWrite(message)
    }
}

impl fmt::Show for IrcConnectionCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IrcConnectionCommand::RawWrite(ref s) => write!(f, "RawWrite({:?})", s),
            IrcConnectionCommand::AddWatcher(ref ew) => write!(f, "AddWatcher({:?}(...))", ew.get_name()),
            IrcConnectionCommand::AddBundler(ref bun) => write!(f, "AddBundler({:?}(...))", bun.get_name()),
        }
    }
}

pub struct IrcConnectionBuf {
    // Lines coming from the server
    incoming_lines: RingBuf<Vec<u8>>,

    // Lines going out to the server
    outgoing_msgs: RingBuf<Vec<u8>>,

    // Internal command queue
    command_queue: RingBuf<IrcConnectionCommand>,

    // Automatic responders e.g. the PING and CTCP handlers
    responders: RingBuf<Box<MessageResponder+Send>>,

    // manages active bundlers and emits events when they finish
    bundler_man: BundlerManager,

    // Current nickname held by the client
    current_nick: Option<String>,
}


impl IrcConnectionBuf {
    pub fn new() -> IrcConnectionBuf {
        let mut out = IrcConnectionBuf {
            incoming_lines: RingBuf::new(),
            outgoing_msgs: RingBuf::new(),
            command_queue: RingBuf::new(),
            responders: RingBuf::new(),
            bundler_man: BundlerManager::new(),
            current_nick: None,
        };
        out.responders.push_back(Box::new(CtcpVersionResponder::new()));
        out.bundler_man.add_bundler_trigger(Box::new(JoinBundlerTrigger::new()));
        out.bundler_man.add_bundler_trigger(Box::new(WhoBundlerTrigger::new()));
        out
    }

    pub fn push_line(&mut self, msg: Vec<u8>) {
        self.incoming_lines.push_back(msg);
    }

    pub fn pop_line(&mut self) -> Option<Vec<u8>> {
        match self.outgoing_msgs.pop_front() {
            Some(mut outgoing_line) => {
                outgoing_line.push_all(b"\r\n");
                Some(outgoing_line)
            },
            None => None
        }
    }

    fn add_watcher(&mut self, watcher: Box<EventWatcher+Send>) {
        self.bundler_man.add_watcher(watcher);
    }

    fn add_bundler(&mut self, bundler: Box<Bundler+Send>) {
        self.bundler_man.add_bundler(bundler);
    }

    pub fn dispatch(&mut self) -> Vec<IrcEvent> {
        while let Some(command) = self.command_queue.pop_front() {
            use self::IrcConnectionCommand::{RawWrite, AddWatcher, AddBundler};
            match command {
                RawWrite(value) => self.outgoing_msgs.push_back(value),
                AddWatcher(value) => self.add_watcher(value),
                AddBundler(value) => self.add_bundler(value),
            }
        }

        let mut outgoing_events = Vec::new();

        while let Some(incoming) = self.incoming_lines.pop_front() {
            let incoming_copy = incoming.clone();

            let msg = match IrcMsg::new(incoming) {
                Ok(msg) => msg,
                Err(reason) => {
                    panic!("Invalid msg: {:?} for {:?}", reason, incoming_copy);
                }
            };

            if msg.get_command() == "001" {
                self.current_nick = Some(String::from_utf8_lossy(&msg[0]).into_owned());
            }

            let msg = {
                let tymsg = server::IncomingMsg::from_msg(msg);

                if let server::IncomingMsg::Ping(ref msg) = tymsg {
                    if let Ok(response) = msg.get_response() {
                        self.outgoing_msgs.push_back(response.into_bytes());
                    }
                }

                if let server::IncomingMsg::Nick(ref msg) = tymsg {
                    if let Some(current_nick) = self.current_nick.clone() {
                        if current_nick.as_slice() == msg.get_nick() {
                            self.current_nick = Some(msg.get_new_nick().to_string())
                        }
                    }
                }

                tymsg.into_irc_msg()
            };

            outgoing_events.extend(self.bundler_man.on_irc_msg(&msg).into_iter());
            for responder in self.responders.iter_mut() {
                for resp_msg in responder.on_irc_msg(&msg).into_iter() {
                    self.outgoing_msgs.push_back(resp_msg.into_bytes());
                }
            }
        }

        outgoing_events
    }

    pub fn register(&mut self, nick: &str) -> Future<RegisterResult> {
        use self::IrcConnectionCommand::AddWatcher;

        let mut reg_watcher = RegisterEventWatcher::new();
        let result_future = reg_watcher.get_future();
        let watcher: Box<EventWatcher+Send> = Box::new(reg_watcher);

        self.command_queue.push_back(AddWatcher(watcher));
        self.outgoing_msgs.push_back(client::Nick::new(nick).into_irc_msg().into_bytes());
        self.outgoing_msgs.push_back(client::User::new(
            "rustirc", "0", "*", "http://github.com/infinityb/rust-irc"
        ).into_irc_msg().into_bytes());

        result_future
    }

    pub fn join(&mut self, channel: &str) -> Future<JoinResult> {
        use self::IrcConnectionCommand::AddWatcher;

        let mut join_watcher = JoinEventWatcher::new(channel.as_bytes());
        let result_future = join_watcher.get_future();
        let watcher: Box<EventWatcher+Send> = Box::new(join_watcher);

        self.command_queue.push_back(AddWatcher(watcher));
        self.outgoing_msgs.push_back(client::Join::new(channel).into_irc_msg().into_bytes());
        result_future

    }

    pub fn who(&mut self, target: &str) -> Future<WhoResult> {
        use self::IrcConnectionCommand::AddWatcher;

        let mut who_watcher = WhoEventWatcher::new(target.as_bytes());
        let result_future = who_watcher.get_future();
        let watcher: Box<EventWatcher+Send> = Box::new(who_watcher);

        self.command_queue.push_back(AddWatcher(watcher));
        self.outgoing_msgs.push_back(client::Who::new(target).into_irc_msg().into_bytes());
        result_future
    }
}

#[cfg(test)]
mod tests {
    use super::IrcConnectionBuf;

    #[test]
    fn test_pingpong() {
        let mut conn = IrcConnectionBuf::new();

        assert_eq!(conn.pop_line(), None);
        conn.push_line(b"PING pretend-server\r\n".to_vec());
        assert_eq!(conn.pop_line(), None);

        assert_eq!(conn.dispatch().len(), 1);
        assert_eq!(conn.pop_line(), Some(b"PONG pretend-server\r\n".to_vec()));
    }
}
