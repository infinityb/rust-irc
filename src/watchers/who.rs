use std::fmt;
use std::sync::mpsc::SyncSender;
use std::borrow::Cow;

use irccase::IrcAsciiExt;
use watchers::base::{Bundler, BundlerTrigger, EventWatcher};
use event::IrcEvent;
use parse::{IrcMsg, IrcMsgPrefix};
use message_types::server;

pub type WhoResult = Result<WhoSuccess, WhoError>;

trait ChannelTargeted {
    fn get_channel(&self) -> &[u8];
}

impl ChannelTargeted for WhoResult {
    fn get_channel(&self) -> &[u8] {
        match *self {
            Ok(ref join_succ) => join_succ.channel.as_slice(),
            Err(ref join_err) => join_err.channel.as_slice()
        }
    }
}

#[derive(Clone, Debug)]
pub struct WhoSuccess {
    pub channel: Vec<u8>,
    pub who_records: Vec<WhoRecord>,
}

impl WhoSuccess {
    fn from_bundler(bundler: WhoBundler) -> WhoSuccess {
        WhoSuccess {
            channel: bundler.target_channel,
            who_records: bundler.who_records
        }
    }
}


// Does /WHO even error?
#[derive(Clone, Debug)]
pub struct WhoError {
    pub channel: Vec<u8>,
}


#[derive(Clone, Debug)]
pub struct WhoRecord {
    pub hostname: String,
    pub server: String,
    pub username: String,
    pub nick: String,
    pub rest: String,
}


impl WhoRecord {
    fn new(args: &[&[u8]]) -> Option<WhoRecord> {
        if args.len() != 8 {
            return None;
        }
        let username = args[2];
        let hostname = args[3];
        let server = args[4];
        let nick = args[5];
        let rest = args[7];

        Some(WhoRecord {
            hostname: String::from_utf8_lossy(hostname).into_owned(),
            server: String::from_utf8_lossy(server).into_owned(),
            username: String::from_utf8_lossy(username).into_owned(),
            nick: String::from_utf8_lossy(nick).into_owned(),
            rest: String::from_utf8_lossy(rest).into_owned(),
        })
    }

    pub fn get_prefix_raw(&self) -> String {
        format!("{}!{}@{}", self.nick, self.username, self.hostname)
    }

    pub fn get_prefix(&self) -> IrcMsgPrefix {
        let prefix_str = format!("{}!{}@{}", self.nick, self.username, self.hostname);
        IrcMsgPrefix::new(Cow::Owned(prefix_str))
    }
}

#[derive(Clone, Copy)]
pub struct WhoBundlerTrigger {
    suppress: bool
}


impl WhoBundlerTrigger {
    pub fn new() -> WhoBundlerTrigger {
        WhoBundlerTrigger {
            suppress: false
        }
    }
}


impl BundlerTrigger for WhoBundlerTrigger {
    fn on_irc_msg(&mut self, msg: &IrcMsg) -> Vec<Box<Bundler+Send>> {
        let mut out = Vec::new();
        if msg.get_command() == "315" && self.suppress {
            self.suppress = false;
        }
        if msg.get_command() == "352" && !self.suppress {
            if msg.len() <= 2 {
                return out;
            }
            self.suppress = true;
            let bundler: WhoBundler = WhoBundler::new(&msg[1]);
            let boxed_bundler: Box<Bundler+Send> = Box::new(bundler);
            out.push(boxed_bundler);
        }
        out
    }
}


#[derive(Clone, Debug)]
pub struct WhoBundler {
    target_channel: Vec<u8>,
    who_records: Vec<WhoRecord>,
    finished: bool
}


impl WhoBundler {
    pub fn new(channel: &[u8]) -> WhoBundler {
        WhoBundler {
            target_channel: channel.to_vec(),
            who_records: vec![],
            finished: false
        }
    }

    fn add_record(&mut self, args: &[&[u8]]) {
        match WhoRecord::new(args) {
            Some(who_rec) => {
                self.who_records.push(who_rec);
            },
            None => ()
        }
    }
}


impl Bundler for WhoBundler {
    fn on_irc_msg(&mut self, msg: &IrcMsg) -> Vec<IrcEvent> {
        let args = msg.get_args();
        if args.len() < 2 {
            return Vec::new();
        }

        if !args[1].eq_ignore_irc_case(self.target_channel.as_slice()) {
            return Vec::new();
        }

        match server::IncomingMsg::from_msg(msg.clone()) {
            server::IncomingMsg::Numeric(352, ref message2) => {
                self.add_record(message2.to_irc_msg().get_args().as_slice());
                Vec::new()
            },
            server::IncomingMsg::Numeric(315, ref _message) => {
                self.finished = true;
                let mut out = Vec::new();
                out.push(IrcEvent::WhoBundle(Ok(WhoSuccess::from_bundler(self.clone()))));
                out
            },
            _ => Vec::new()
        }
    }

    fn is_finished(&mut self) -> bool {
        self.finished
    }

    fn get_name(&self) -> &'static str {
        "WhoBundler"
    }
}


/// Waits for target WhoBundleEvent and clones it down the monitor
pub struct WhoEventWatcher {
    channel: Vec<u8>,
    result: Option<WhoResult>,
    monitors: Vec<SyncSender<WhoResult>>,
    finished: bool
}


impl WhoEventWatcher {
    fn dispatch_monitors(&mut self) {
        let result = self.result.clone().unwrap();
        for monitor in self.monitors.iter() {
            match monitor.try_send(result.clone()) {
                Ok(_) => (),
                Err(_) => panic!("sending failed")
            }
        }
        self.monitors = Vec::new();
    }
}

impl fmt::Debug for WhoEventWatcher {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WhoEventWatcher(channel={:?})", self.channel.as_slice())
    }
}

impl EventWatcher for WhoEventWatcher {
    fn on_event(&mut self, event: &IrcEvent) {
        match event {
            &IrcEvent::WhoBundle(ref result) => {
                if result.get_channel().eq_ignore_irc_case(self.channel.as_slice()) {
                    self.result = Some(result.clone());
                    self.dispatch_monitors();
                    self.finished = true;
                }
            },
            _ => ()
        }
    }

    fn is_finished(&self) -> bool {
        self.finished
    }

    fn get_name(&self) -> &'static str {
        "WhoEventWatcher"
    }

    fn display(&self) -> String {
        format!("{:?}", self)
    }
}
