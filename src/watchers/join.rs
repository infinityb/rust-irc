use std::fmt;
use std::from_str::from_str;
use std::ascii::AsciiExt;

use message::IrcMessage;
use watchers::base::{Bundler, BundlerTrigger, EventWatcher};
use event::{
    IrcEvent,
    IrcEventJoinBundle
};


pub type JoinResult = Result<JoinSuccess, JoinError>;

trait ChannelTargeted {
    fn get_channel(&self) -> &str;
}

impl ChannelTargeted for JoinResult {
    fn get_channel(&self) -> &str {
        match self {
            &Ok(ref join_succ) => join_succ.channel.as_slice(),
            &Err(ref join_err) => join_err.channel.as_slice()
        }
    }
}

#[deriving(Clone, Show)]
pub struct JoinSuccess {
    pub channel: String,
    pub nicks: Vec<String>,
    pub topic: Option<TopicMeta>,
}


#[deriving(Clone, Show)]
pub struct TopicMeta {
    pub text: String,
    pub set_at: u64,
    pub set_by: String,
}


impl TopicMeta {
    fn new(topic: &String, other: &BundlerTopicMeta) -> TopicMeta {
        TopicMeta {
            text: topic.clone(),
            set_at: other.set_at,
            set_by: other.set_by.clone(),
        }
    }
}


#[deriving(Clone, Show)]
pub struct JoinError {
    pub channel: String,
    pub errcode: i16,
    pub message: String
}

enum JoinBundlerTriggerState {
    Unregistered,
    Running
}

pub struct JoinBundlerTrigger {
    state: JoinBundlerTriggerState,
    current_nick: String
}


impl JoinBundlerTrigger {
    pub fn new() -> JoinBundlerTrigger {
        JoinBundlerTrigger {
            state: Unregistered,
            current_nick: String::new()
        }
    }

    fn on_nick(&mut self, message: &IrcMessage) {
        if let Some(ref pref) = message.get_prefix() {
            if pref.nick() == Some(self.current_nick.as_slice()) {
                let new_nick = message.get_args()[0].to_string();
                info!("{} detected nick change {} -> {}",
                    self, self.current_nick, new_nick);
                self.current_nick = new_nick;
            }
        }
    }

    fn is_self_join(&self, message: &IrcMessage) -> bool {
        if let Some(ref pref) = message.get_prefix() {
            pref.nick() == Some(self.current_nick.as_slice())
        } else {
            false
        }
    }
}


impl BundlerTrigger for JoinBundlerTrigger {
    fn on_message(&mut self, message: &IrcMessage) -> Vec<Box<Bundler+Send>> {
        match (self.state, message.command()) {
            (Unregistered, "001") => {
                self.state = Running;
                self.current_nick = message.get_args()[0].to_string();
                Vec::new()
            },
            (Unregistered, _) => Vec::new(),
            (Running, "JOIN") => {
                let mut out = Vec::new();
                if self.is_self_join(message) {
                    let channel = message.get_args()[0];
                    let bundler: Box<Bundler+Send> = box JoinBundler::new(channel);
                    out.push(bundler);
                }
                out
            },
            (Running, "NICK") => {
                // potentially our nick is changing
                self.on_nick(message);
                Vec::new()
            }
            (Running, _) => Vec::new()
        }
    }
}

impl fmt::Show for JoinBundlerTrigger {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "JoinBundlerTrigger(current_nick={})", self.current_nick.as_slice())
    }
}


struct BundlerTopicMeta {
    set_by: String,
    set_at: u64,
}

impl BundlerTopicMeta {
    fn from_msg(msg: &IrcMessage) -> Option<BundlerTopicMeta> {
        let args = msg.get_args();
        match from_str(args[3]) {
            Some(set_at) => {
                Some(BundlerTopicMeta {
                    set_by: args[2].to_string(),
                    set_at: set_at,
                })
            },
            None => None
        }
    }
}


pub struct JoinBundler {
    channel: String,
    topic: Option<String>,
    topic_meta: Option<BundlerTopicMeta>,
    nicks: Option<Vec<String>>,
    state: JoinBundlerState,
    result: Option<JoinResult>,
}

enum JoinBundlerState {
    PreJoin,
    Joining,
    Joined,
    JoinFail
}


impl JoinBundler {
    pub fn new(channel: &str) -> JoinBundler {
        JoinBundler {
            channel: String::from_str(channel),
            topic: None,
            topic_meta: None,
            nicks: Some(Vec::new()),
            state: PreJoin,
            result: None
        }
    }

    fn accept_state_prejoin(&mut self, message: &IrcMessage) -> Option<JoinBundlerState> {
        let success = match message.command() {
            "JOIN" => {
                if !message.get_args()[0].eq_ignore_ascii_case(self.channel.as_slice()) {
                    return None;
                }
                true
            },
            "475" => {
                if message.get_args()[1].eq_ignore_ascii_case(self.channel.as_slice()) {
                    return None;
                }
                false
            },
            _ => return None
        };

        if !success {
            self.result = Some(Err(JoinError {
                channel: String::from_str(self.channel.as_slice()),
                errcode: 0,
                message: String::from_str("")
            }));
        }
        Some(if success { Joining } else { JoinFail })
    }

    fn on_topic(&mut self, message: &IrcMessage) -> Option<JoinBundlerState> {
        self.topic = Some(message.get_args()[2].to_string());
        None
    }

    fn on_topic_meta(&mut self, message: &IrcMessage) -> Option<JoinBundlerState> {
        self.topic_meta = BundlerTopicMeta::from_msg(message);
        None
    }

    fn on_names(&mut self, message: &IrcMessage) -> Option<JoinBundlerState> {
        if let Some(nicks) = self.nicks.as_mut() {
            for nick in message.get_args()[3].split(' ') {
                nicks.push(nick.to_string());
            }
        }
        None
    }

    fn on_names_end(&mut self, _: &IrcMessage) -> Option<JoinBundlerState> {
        let topic = match (self.topic.as_ref(), self.topic_meta.as_ref()) {
            (Some(topic), Some(topic_meta)) => {
                Some(TopicMeta::new(topic, topic_meta))
            },
            _ => None
        };
        self.result = Some(Ok(JoinSuccess {
            channel: self.channel.clone(),
            nicks: self.nicks.take().unwrap(),
            topic: topic
        }));
        Some(Joined)
    }

    fn accept_state_joining(&mut self, message: &IrcMessage) -> Option<JoinBundlerState> {
        if message.command() == "332" {
            if message.get_args()[1].eq_ignore_ascii_case(self.channel.as_slice()) {
                return self.on_topic(message);
            }
            return None;
        }
        if message.command() == "333" {
            if message.get_args()[1].eq_ignore_ascii_case(self.channel.as_slice()) {
                return self.on_topic_meta(message);
            }
            return None;
        }
        if message.command() == "353" {
            assert!(match message.get_args()[1] {
                "=" => true,
                "*" => true,
                "@" => true,
                _ => false
            });
            if message.get_args()[2].eq_ignore_ascii_case(self.channel.as_slice()) {
                return self.on_names(message);
            }
            return None;
        }
        if message.command() == "366" {
            if message.get_args()[1].eq_ignore_ascii_case(self.channel.as_slice()) {
                return self.on_names_end(message);
            }
            return None;
        }
        None
    }
}


impl Bundler for JoinBundler {
    fn on_message(&mut self, message: &IrcMessage) -> Vec<IrcEvent> {
        let new_state = match self.state {
            PreJoin => self.accept_state_prejoin(message),
            Joining => self.accept_state_joining(message),
            _ => None
        };
        match new_state {
            Some(new_state) => {
                self.state = new_state;
            },
            None => ()
        }
        match self.result.take() {
            Some(result) => {
                vec![IrcEventJoinBundle(result)]
            },
            None => vec![]
        }
    }

    fn is_finished(&mut self) -> bool {
        match self.state {
            JoinFail | Joined => true,
            _ => false
        }
    }

    fn get_name(&self) -> &'static str {
        "JoinBundler"
    }
}


impl fmt::Show for JoinBundler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "JoinBundler({})", self.channel.as_slice())
    }
}


/// Waits for target JoinBundleEvent and clones it down the monitor
pub struct JoinEventWatcher {
    channel: String,
    result: Option<JoinResult>,
    monitors: Vec<SyncSender<JoinResult>>,
}


impl JoinEventWatcher {
    pub fn new(channel: &str) -> JoinEventWatcher {
        JoinEventWatcher {
            channel: String::from_str(channel),
            monitors: Vec::new(),
            result: None,
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

    fn add_monitor(&mut self, monitor: SyncSender<JoinResult>) {
        let result = self.result.clone();

        match result {
            Some(result) => monitor.send(result.clone()),
            None => self.monitors.push(monitor)
        }
    }

    pub fn get_monitor(&mut self) -> Receiver<JoinResult> {
        let (tx, rx) = sync_channel(1);
        self.add_monitor(tx);
        rx
    }
}


impl fmt::Show for JoinEventWatcher {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "JoinEventWatcher(channel={})", self.channel.as_slice())
    }
}


impl EventWatcher for JoinEventWatcher {
    fn on_event(&mut self, message: &IrcEvent) {
        match *message {
            IrcEventJoinBundle(ref result) => {
                if result.get_channel().eq_ignore_ascii_case(self.channel.as_slice()) {
                    self.result = Some(result.clone());
                    self.dispatch_monitors();
                }
            },
            _ => ()
        }
    }

    fn is_finished(&self) -> bool {
        self.result.is_some()
    }

    fn get_name(&self) -> &'static str {
        "JoinEventWatcher"
    }

    fn display(&self) -> String {
        format!("{}", self)
    }
}
