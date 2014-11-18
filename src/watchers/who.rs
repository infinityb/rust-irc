use std::fmt;
use std::ascii::AsciiExt;

use watchers::base::{Bundler, BundlerTrigger, EventWatcher};
use event::IrcEvent;

use message::{
    IrcMessage,
    IrcProtocolMessage,
};
use parse::IrcMsgPrefix;
use util::{StringSlicer, OptionalStringSlicer};

pub type WhoResult = Result<WhoSuccess, WhoError>;

trait ChannelTargeted {
    fn get_channel(&self) -> &str;
}

impl ChannelTargeted for WhoResult {
    fn get_channel(&self) -> &str {
        match *self {
            Ok(ref join_succ) => join_succ.channel.as_slice(),
            Err(ref join_err) => join_err.channel.as_slice()
        }
    }
}

#[deriving(Clone, Show)]
pub struct WhoSuccess {
    pub channel: String,
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
    pub channel: String
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
    fn new(args: &Vec<String>) -> Option<WhoRecord> {
        match args.as_slice() {
            [ref _self_nick, ref _channel, ref username,
             ref hostname, ref server, ref nick, ref _unk1, ref rest
            ] => {
                Some(WhoRecord {
                    hostname: hostname.clone(),
                    server: server.clone(),
                    username: username.clone(),
                    nick: nick.clone(),
                    rest: rest.clone()
                })
            },
            _ => None
        }
    }

    #[stable]
    pub fn get_prefix_raw(&self) -> String {
        format!("{}!{}@{}", self.nick, self.username, self.hostname)
    }

    #[stable]
    pub fn get_prefix(&self) -> IrcMsgPrefix {
        let prefix_str = format!("{}!{}@{}", self.nick, self.username, self.hostname);
        IrcMsgPrefix::new(prefix_str.into_maybe_owned())
    }
}

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
    fn on_message(&mut self, message: &IrcMessage) -> Vec<Box<Bundler+Send>> {
        let mut out = Vec::new();
        if message.command() == "315" && self.suppress {
            self.suppress = false;
        }
        if message.command() == "352" && !self.suppress {
            let args = message.get_args();
            if args.len() <= 2 {
                return out;
            }
            self.suppress = true;
            let bundler: WhoBundler = WhoBundler::new(args[1]);
            let boxed_bundler: Box<Bundler+Send> = box bundler;
            out.push(boxed_bundler);
        }
        out
    }
}


#[deriving(Clone, Show)]
pub struct WhoBundler {
    target_channel: String,
    who_records: Vec<WhoRecord>,
    finished: bool
}


impl WhoBundler {
    pub fn new(channel: &str) -> WhoBundler {
        WhoBundler {
            target_channel: String::from_str(channel),
            who_records: vec![],
            finished: false
        }
    }

    fn add_record(&mut self, args: &Vec<String>) {
        match WhoRecord::new(args) {
            Some(who_rec) => {
                self.who_records.push(who_rec);
            },
            None => ()
        }
    }
}


impl Bundler for WhoBundler {
    fn on_message(&mut self, message: &IrcMessage) -> Vec<IrcEvent> {
        let args = message.get_args();
        if args.len() < 2 {
            return Vec::new();
        }

        if !args[1].eq_ignore_ascii_case(self.target_channel.as_slice()) {
            return Vec::new();
        }
        info!("WhoBundler on_message({})", message);
        match *message.get_message() {
            IrcProtocolMessage::Numeric(352, ref message) => {
                self.add_record(message);
                Vec::new()
            },
            IrcProtocolMessage::Numeric(315, ref _message) => {
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
    channel: String,
    result: Option<WhoResult>,
    monitors: Vec<SyncSender<WhoResult>>,
    finished: bool
}


impl WhoEventWatcher {
    pub fn new(channel: &str) -> WhoEventWatcher {
        WhoEventWatcher {
            channel: String::from_str(channel),
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
            Some(result) => monitor.send(result.clone()),
            None => self.monitors.push(monitor)
        }
    }

    pub fn get_monitor(&mut self) -> Receiver<WhoResult> {
        let (tx, rx) = sync_channel(1);
        self.add_monitor(tx);
        rx
    }
}

impl fmt::Show for WhoEventWatcher {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WhoEventWatcher(channel={})", self.channel.as_slice())
    }
}

impl EventWatcher for WhoEventWatcher {
    fn on_event(&mut self, message: &IrcEvent) {
        match message {
            &IrcEvent::WhoBundle(ref result) => {
                if result.get_channel().eq_ignore_ascii_case(self.channel.as_slice()) {
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


pub struct IrcUser {
    hostmask: String,
    nick_idx_pair: StringSlicer,
    username_idx_pair: OptionalStringSlicer,
    hostname_idx_pair: OptionalStringSlicer,
}

impl IrcUser {
    #[inline]
    pub fn new(hostmask: &str) -> Option<IrcUser> {
        let idx_pair = match hostmask.find('!') {
            Some(exc_idx) => match hostmask[exc_idx+1..].find('@') {
                Some(at_idx) => Some((exc_idx, exc_idx + at_idx + 1)),
                None => None
            },
            None => None
        };

        let hostmask_str = hostmask.to_string();
        Some(match idx_pair {
            Some((exc_idx, at_idx)) => IrcUser {
                hostmask: hostmask_str,
                nick_idx_pair: StringSlicer::new(0, exc_idx),
                username_idx_pair: OptionalStringSlicer::new_some(exc_idx + 1, at_idx),
                hostname_idx_pair: OptionalStringSlicer::new_some(at_idx + 1, hostmask.len())
            },
            None => IrcUser {
                hostmask: hostmask_str,
                nick_idx_pair: StringSlicer::new(0, hostmask.len()),
                username_idx_pair: OptionalStringSlicer::new_none(),
                hostname_idx_pair: OptionalStringSlicer::new_none()
            }
        })
    }

    #[inline]
    pub fn nick<'a>(&'a self) -> &'a str {
        self.nick_idx_pair.slice_on(self.hostmask[])
    }

    #[inline]
    pub fn username<'a>(&'a self) -> Option<&'a str> {
        self.username_idx_pair.slice_on(self.hostmask[])
    }

    #[inline]
    pub fn hostname<'a>(&'a self) -> Option<&'a str> {
        self.hostname_idx_pair.slice_on(self.hostmask[])
    }
}

#[test]
fn test_irc_user() {
    let user = IrcUser::new("sell!q@127.0.0.1").unwrap();
    assert_eq!(user.nick(), "sell");
    assert_eq!(user.username(), Some("q"));
    assert_eq!(user.hostname(), Some("127.0.0.1"));

    let user = IrcUser::new("sell").unwrap();
    assert_eq!(user.nick(), "sell");
    assert_eq!(user.username(), None);
    assert_eq!(user.hostname(), None);
}
