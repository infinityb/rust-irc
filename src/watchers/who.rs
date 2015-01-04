use std::fmt;
use std::sync::Future;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::borrow::IntoCow;

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

#[deriving(Clone, Show)]
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
#[deriving(Clone, Show)]
#[experimental = "Public fields definitely going away"]
pub struct WhoError {
    pub channel: Vec<u8>,
}


#[deriving(Clone, Show)]
#[experimental = "Public fields definitely going away"]
pub struct WhoRecord {
    pub hostname: String,
    pub server: String,
    pub username: String,
    pub nick: String,
    pub rest: String,
}


impl WhoRecord {
    fn new(args: &[&[u8]]) -> Option<WhoRecord> {
        match args {
            [_self_nick, _channel, username,
             hostname, server, nick, _unk1, rest
            ] => {
                let whorec = WhoRecord {
                    hostname: String::from_utf8_lossy(hostname).into_owned(),
                    server: String::from_utf8_lossy(server).into_owned(),
                    username: String::from_utf8_lossy(username).into_owned(),
                    nick: String::from_utf8_lossy(nick).into_owned(),
                    rest: String::from_utf8_lossy(rest).into_owned(),
                };
                Some(whorec)
            },
            _ => {
                None
            }
        }
    }

    #[stable]
    pub fn get_prefix_raw(&self) -> String {
        format!("{}!{}@{}", self.nick, self.username, self.hostname)
    }

    #[stable]
    pub fn get_prefix(&self) -> IrcMsgPrefix {
        let prefix_str = format!("{}!{}@{}", self.nick, self.username, self.hostname);
        IrcMsgPrefix::new(prefix_str.into_cow())
    }
}

#[deriving(Copy)]
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
            let boxed_bundler: Box<Bundler+Send> = box bundler;
            out.push(boxed_bundler);
        }
        out
    }
}


#[deriving(Clone, Show)]
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
    pub fn new(channel: &[u8]) -> WhoEventWatcher {
        WhoEventWatcher {
            channel: channel.to_vec(),
            monitors: Vec::new(),
            result: None,
            finished: false
        }
    }

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

    fn add_monitor(&mut self, monitor: SyncSender<WhoResult>) {
        let result = self.result.clone();

        match result {
            Some(result) => monitor.send(result.clone()).ok().expect("send failure"),
            None => self.monitors.push(monitor)
        }
    }

    pub fn get_monitor(&mut self) -> Receiver<WhoResult> {
        let (tx, rx) = sync_channel(1);
        self.add_monitor(tx);
        rx
    }

    pub fn get_future(&mut self) -> Future<WhoResult> {
        Future::from_receiver(self.get_monitor())
    }
}

impl fmt::Show for WhoEventWatcher {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WhoEventWatcher(channel={})", self.channel.as_slice())
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
        format!("{}", self)
    }
}
