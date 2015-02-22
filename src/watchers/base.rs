use std::collections::VecDeque;

use parse::IrcMsg;
use event::IrcEvent;

pub trait MessageWatcher {
    fn on_irc_msg(&mut self, message: &IrcMsg);

    /// If true, the `MessageWatcher` should be removed from the watcher set
    fn finished(&self) -> bool;
}


pub trait EventWatcher {
    fn on_event(&mut self, message: &IrcEvent);

    /// If true, the `EventWatcher` should be removed from the watcher set
    fn is_finished(&self) -> bool;

    fn get_name(&self) -> &'static str;

    fn display(&self) -> String;
}


/// Emits IrcEvents when certain messages are detected
pub trait Bundler {
    fn on_irc_msg(&mut self, message: &IrcMsg) -> Vec<IrcEvent>;

    /// If true, the `Bundler` should be removed from the bundler set
    fn is_finished(&mut self) -> bool;

    fn get_name(&self) -> &'static str;
}

/// Emits Bundlers when certain messages are detected
pub trait BundlerTrigger {
    fn on_irc_msg(&mut self, message: &IrcMsg) -> Vec<Box<Bundler+Send+'static>>;
}

/// Controls the lifecycle of EventWatchers, Bundlers, and BundlerTriggers
pub struct BundlerManager {
    /// Unfinished watchers currently attached to the stream
    event_watchers: VecDeque<Box<EventWatcher+Send+'static>>,

    /// Active event bundlers.
    event_bundlers: VecDeque<Box<Bundler+Send+'static>>,

    /// Bundler triggers.  They create Bundlers.
    bundler_triggers: Vec<Box<BundlerTrigger+Send+'static>>,
}

impl BundlerManager {
    pub fn new() -> BundlerManager {
        BundlerManager {
            event_watchers: VecDeque::new(),
            event_bundlers: VecDeque::new(),
            bundler_triggers: Vec::new(),
        }
    }

    /// Initialise a BundlerManager with JoinBundlerTrigger and
    /// WhoBundlerTrigger
    pub fn with_defaults() -> BundlerManager {
        let mut manager = BundlerManager::new();
        manager.add_bundler_trigger(Box::new(super::JoinBundlerTrigger::new()));
        manager.add_bundler_trigger(Box::new(super::WhoBundlerTrigger::new()));
        manager
    }

    // Do we really need +Send here?
    pub fn add_watcher(&mut self, watcher: Box<EventWatcher+Send+'static>) {
        self.event_watchers.push_back(watcher);
    }

    pub fn add_bundler(&mut self, bundler: Box<Bundler+Send+'static>) {
        self.event_bundlers.push_back(bundler);
    }

    pub fn add_bundler_trigger(&mut self, bundler: Box<BundlerTrigger+Send+'static>) {
        self.bundler_triggers.push(bundler);
    }

    pub fn on_irc_msg(&mut self, msg: &IrcMsg) -> Vec<IrcEvent> {
        let mut outgoing_events: Vec<IrcEvent> = Vec::new();

        for new_bundler in bundler_trigger_impl(&mut self.bundler_triggers, msg).into_iter() {
            self.event_bundlers.push_back(new_bundler);
        }

        for event in bundler_accept_impl(&mut self.event_bundlers, msg).into_iter() {
            outgoing_events.push(event);
        }

        outgoing_events.push(IrcEvent::IrcMsg(msg.clone()));

        for event in outgoing_events.iter() {
            for watcher in watcher_accept_impl(&mut self.event_watchers, event).into_iter() {
                drop(watcher);
            }
        }

        outgoing_events
    }
}

fn bundler_trigger_impl(triggers: &mut Vec<Box<BundlerTrigger+Send+'static>>,
                        msg: &IrcMsg
                       ) -> Vec<Box<Bundler+Send>> {

    let mut activating: Vec<Box<Bundler+Send>> = Vec::new();
    for trigger in triggers.iter_mut() {
        let new_bundlers = trigger.on_irc_msg(msg);
        activating.reserve(new_bundlers.len());
        for bundler in new_bundlers.into_iter() {
            activating.push(bundler);
        }
    }
    activating
}


fn watcher_accept_impl(buf: &mut VecDeque<Box<EventWatcher+Send+'static>>,
                       event: &IrcEvent
                      ) -> Vec<Box<EventWatcher+Send+'static>> {
    let mut keep_watchers: VecDeque<Box<EventWatcher+Send>> = VecDeque::new();
    let mut finished_watchers: Vec<Box<EventWatcher+Send>> = Vec::new();

    loop {
        match buf.pop_front() {
            Some(mut watcher) => {
                watcher.on_event(event);
                if watcher.is_finished() {
                    finished_watchers.push(watcher);
                } else {
                    keep_watchers.push_back(watcher);
                }
            },
            None => break
        }
    }
    loop {
        match keep_watchers.pop_front() {
            Some(watcher) => buf.push_back(watcher),
            None => break
        }
    }
    finished_watchers
}


fn bundler_accept_impl(buf: &mut VecDeque<Box<Bundler+Send+'static>>,
                       msg: &IrcMsg
                      ) -> Vec<IrcEvent> {

    let mut keep_bundlers: VecDeque<Box<Bundler+Send>> = VecDeque::new();
    let mut emit_events: Vec<IrcEvent> = Vec::new();

    loop {
        match buf.pop_front() {
            Some(mut bundler) => {
                for event in bundler.on_irc_msg(msg).into_iter() {
                    emit_events.push(event);
                }
                if !bundler.is_finished() {
                    keep_bundlers.push_back(bundler);
                }
            },
            None => break
        }
    }
    loop {
        match keep_bundlers.pop_front() {
            Some(watcher) => buf.push_back(watcher),
            None => break
        }
    }
    emit_events
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::old_io::{IoResult, BufReader};

    use parse::IrcMsg;
    use watchers::{
        WhoBundlerTrigger,
        JoinBundlerTrigger,
    };
    use event::IrcEvent::{
        JoinBundle,
        WhoBundle,
    };

    const TEST_DATA: &'static [u8] = include_bytes!("../../testdata/watcher.txt");

    fn unsafe_to_irc_message(line_res: IoResult<String>) -> IrcMsg {
        let line = match line_res {
            Ok(line) => line,
            Err(err) => panic!("err: {:?}", err)
        };
        let totrim: &[_] = &['\n', '\r'];
        match IrcMsg::new(line.as_slice().trim_right_matches(totrim).to_string().into_bytes()) {
            Ok(message) => message,
            Err(err) => panic!("err: {:?}", err)
        }
    }

    #[test]
    fn test_bundle_watcher() {
        let mut reader = BufReader::new(TEST_DATA);
        let mut bunman = BundlerManager::new();
        bunman.add_bundler_trigger(Box::new(JoinBundlerTrigger::new()));
        bunman.add_bundler_trigger(Box::new(WhoBundlerTrigger::new()));
        let mut events = Vec::new();

        for msg in reader.lines().map(unsafe_to_irc_message) {
            events.extend(bunman.on_irc_msg(&msg).into_iter());
        }

        let mut join_bundles = 0;
        let mut who_bundles = 0;

        for event in events.into_iter() {
            if let JoinBundle(_) = event {
                join_bundles += 1;
            }
            if let WhoBundle(_) = event {
                who_bundles += 1
            }
        }
        assert_eq!(join_bundles, 1);
        assert_eq!(who_bundles, 1);
    }
}
