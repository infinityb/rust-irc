use std::fmt;
use std::sync::Future;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::str::from_str;
use irccase::IrcAsciiExt;

use parse::IrcMsg;
use watchers::base::{Bundler, BundlerTrigger, EventWatcher};
use event::IrcEvent;


pub type JoinResult = Result<JoinSuccess, JoinError>;

trait ChannelTargeted {
    fn get_channel(&self) -> &[u8];
}

impl ChannelTargeted for JoinResult {
    fn get_channel(&self) -> &[u8] {
        match self {
            &Ok(ref join_succ) => join_succ.channel.as_slice(),
            &Err(ref join_err) => join_err.channel.as_slice()
        }
    }
}

#[derive(Clone, Show)]
pub struct JoinSuccess {
    pub channel: Vec<u8>,
    pub nicks: Vec<String>,
    pub topic: Option<TopicMeta>,
}


#[derive(Clone, Show)]
pub struct TopicMeta {
    pub text: Vec<u8>,
    pub set_at: u64,
    pub set_by: String,
}


impl TopicMeta {
    fn new(topic: &Vec<u8>, other: &BundlerTopicMeta) -> TopicMeta {
        TopicMeta {
            text: topic.clone(),
            set_at: other.set_at,
            set_by: other.set_by.clone(),
        }
    }
}


#[derive(Clone, Show)]
pub struct JoinError {
    pub channel: Vec<u8>,
    pub errcode: i16,
    pub message: String
}

#[derive(Copy)]
enum JoinBundlerTriggerState {
    Unregistered,
    Running
}


pub struct JoinBundlerTrigger {
    state: JoinBundlerTriggerState,
    current_nick: Vec<u8>,
}


impl JoinBundlerTrigger {
    pub fn new() -> JoinBundlerTrigger {
        JoinBundlerTrigger {
            state: JoinBundlerTriggerState::Unregistered,
            current_nick: Vec::new()
        }
    }

    fn on_nick(&mut self, msg: &IrcMsg) {
        let is_self_nick = msg.get_prefix().nick()
            .and_then(|: nick| Some(nick.as_bytes() == self.current_nick.as_slice()))
            .unwrap_or(false);

        if is_self_nick {
            info!("{} detected nick change {} -> {}",
                self, self.current_nick, &msg[0]);
            self.current_nick = msg[0].to_vec();
        }
    }

    fn is_self_join(&self, msg: &IrcMsg) -> bool {
        msg.get_prefix().nick()
            .and_then(|: nick| Some(nick.as_bytes() == self.current_nick.as_slice()))
            .unwrap_or(false)
    }
}


impl BundlerTrigger for JoinBundlerTrigger {
    fn on_irc_msg(&mut self, msg: &IrcMsg) -> Vec<Box<Bundler+Send>> {
        match (self.state, msg.get_command()) {
            (JoinBundlerTriggerState::Unregistered, "001") => {
                self.state = JoinBundlerTriggerState::Running;
                self.current_nick = msg[0].to_vec();
                Vec::new()
            },
            (JoinBundlerTriggerState::Unregistered, _) => Vec::new(),
            (JoinBundlerTriggerState::Running, "JOIN") => {
                let mut out = Vec::new();
                if self.is_self_join(msg) {
                    let channel = &msg[0];
                    let bundler: Box<Bundler+Send> = box JoinBundler::new(channel);
                    out.push(bundler);
                }
                out
            },
            (JoinBundlerTriggerState::Running, "NICK") => {
                // potentially our nick is changing
                self.on_nick(msg);
                Vec::new()
            }
            (JoinBundlerTriggerState::Running, _) => Vec::new()
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
    fn from_msg(msg: &IrcMsg) -> Option<BundlerTopicMeta> {
        let args2 = match ::std::str::from_utf8(&msg[2]) {
            Ok(args3) => args3,
            Err(_) => return None,
        };
        let args3 = match ::std::str::from_utf8(&msg[3]) {
            Ok(args3) => args3,
            Err(_) => return None,
        };
        match args3.parse() {
            Some(set_at) => {
                Some(BundlerTopicMeta {
                    set_by: args2.to_string(),
                    set_at: set_at,
                })
            },
            None => None
        }
    }
}


pub struct JoinBundler {
    channel: Vec<u8>,
    topic: Option<Vec<u8>>,
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
    pub fn new(channel: &[u8]) -> JoinBundler {
        JoinBundler {
            channel: channel.to_vec(),
            topic: None,
            topic_meta: None,
            nicks: Some(Vec::new()),
            state: JoinBundlerState::PreJoin,
            result: None
        }
    }

    fn accept_state_prejoin(&mut self, msg: &IrcMsg) -> Option<JoinBundlerState> {
        let success = match msg.get_command() {
            "JOIN" => {
                if !msg[0].eq_ignore_irc_case(self.channel.as_slice()) {
                    return None;
                }
                true
            },
            "475" => {
                if !msg[1].eq_ignore_irc_case(self.channel.as_slice()) {
                    return None;
                }
                false
            },
            _ => return None
        };

        if !success {
            self.result = Some(Err(JoinError {
                channel: self.channel.as_slice().to_vec(),
                errcode: 0,
                message: String::from_str("")
            }));
        }
        Some(if success { JoinBundlerState::Joining } else { JoinBundlerState::JoinFail })
    }

    fn on_topic(&mut self, msg: &IrcMsg) -> Option<JoinBundlerState> {
        self.topic = Some(msg[2].to_vec());
        None
    }

    fn on_topic_meta(&mut self, msg: &IrcMsg) -> Option<JoinBundlerState> {
        self.topic_meta = BundlerTopicMeta::from_msg(msg);
        None
    }

    fn on_names(&mut self, msg: &IrcMsg) -> Option<JoinBundlerState> {
        // FIXME
        let nicks_data = String::from_utf8_lossy(&msg[3]);

        if let Some(nicks) = self.nicks.as_mut() {
            for nick in nicks_data.split(' ') {
                if nick.len() > 0 {
                    nicks.push(nick.to_string());
                }
            }
        }
        None
    }

    fn on_names_end(&mut self, _: &IrcMsg) -> Option<JoinBundlerState> {
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
        Some(JoinBundlerState::Joined)
    }

    fn accept_state_joining(&mut self, msg: &IrcMsg) -> Option<JoinBundlerState> {
        if msg.get_command() == "332" {
            if msg[1].eq_ignore_irc_case(self.channel.as_slice()) {
                return self.on_topic(msg);
            }
            return None;
        }
        if msg.get_command() == "333" {
            if msg[1].eq_ignore_irc_case(self.channel.as_slice()) {
                return self.on_topic_meta(msg);
            }
            return None;
        }
        if msg.get_command() == "353" {
            assert!(match &msg[1] {
                b"=" | b"*" | b"@" => true,
                _ => false
            });
            if msg[2].eq_ignore_irc_case(self.channel.as_slice()) {
                return self.on_names(msg);
            }
            return None;
        }
        if msg.get_command() == "366" {
            if msg[1].eq_ignore_irc_case(self.channel.as_slice()) {
                return self.on_names_end(msg);
            }
            return None;
        }
        None
    }
}


impl Bundler for JoinBundler {
    fn on_irc_msg(&mut self, msg: &IrcMsg) -> Vec<IrcEvent> {
        let new_state = match self.state {
            JoinBundlerState::PreJoin => self.accept_state_prejoin(msg),
            JoinBundlerState::Joining => self.accept_state_joining(msg),
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
                vec![IrcEvent::JoinBundle(result)]
            },
            None => vec![]
        }
    }

    fn is_finished(&mut self) -> bool {
        match self.state {
            JoinBundlerState::JoinFail | JoinBundlerState::Joined => true,
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
    channel: Vec<u8>,
    result: Option<JoinResult>,
    monitors: Vec<SyncSender<JoinResult>>,
}


impl JoinEventWatcher {
    pub fn new(channel: &[u8]) -> JoinEventWatcher {
        JoinEventWatcher {
            channel: channel.to_vec(),
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
            Some(result) => monitor.send(result.clone()).ok().expect("send failure"),
            None => self.monitors.push(monitor)
        };
    }

    pub fn get_monitor(&mut self) -> Receiver<JoinResult> {
        let (tx, rx) = sync_channel(1);
        self.add_monitor(tx);
        rx
    }

    pub fn get_future(&mut self) -> Future<JoinResult> {
        Future::from_receiver(self.get_monitor())
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
            IrcEvent::JoinBundle(ref result) => {
                if result.get_channel().eq_ignore_irc_case(self.channel.as_slice()) {
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
