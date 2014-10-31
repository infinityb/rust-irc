use std::collections::{RingBuf, Deque};

use message::IrcMessage;
use event::{
    IrcEvent,
    IrcEventMessage,
};


pub trait MessageResponder {
    fn on_message(&mut self, message: &IrcMessage) -> Vec<IrcMessage>;

    fn finished(&self) -> bool { false }
}

pub trait MessageWatcher {
    fn on_message(&mut self, message: &IrcMessage);

    /// If true, the `MessageWatcher` should be removed from the watcher set
    fn finished(&self) -> bool;
}


pub trait EventWatcher{
    fn on_event(&mut self, message: &IrcEvent);

    /// If true, the `EventWatcher` should be removed from the watcher set
    fn is_finished(&self) -> bool;

    fn get_name(&self) -> &'static str;
}


pub trait Bundler {
    fn on_message(&mut self, message: &IrcMessage) -> Vec<IrcEvent>;

    /// If true, the `Bundler` should be removed from the bundler set
    fn is_finished(&mut self) -> bool;
}


pub trait BundlerTrigger {
	fn on_message(&mut self, message: &IrcMessage) -> Vec<Box<Bundler+Send>>;
}

pub struct BundlerManager {
    // Unfinished watchers currently attached to the stream
    event_watchers: RingBuf<Box<EventWatcher+Send>>,

    // Active event bundlers.
    event_bundlers: RingBuf<Box<Bundler+Send>>,

    // Bundler triggers.  They create Bundlers.
    bundler_triggers: Vec<Box<BundlerTrigger+Send>>,
}

impl BundlerManager {
    pub fn new() -> BundlerManager {
        BundlerManager {
            event_watchers: RingBuf::new(),
            event_bundlers: RingBuf::new(),
            bundler_triggers: Vec::new(),
        }
    }
    
    // Do we really need +Send here?
    pub fn add_watcher(&mut self, watcher: Box<EventWatcher+Send>) {
        self.event_watchers.push(watcher);
    }

    pub fn add_bundler(&mut self, bundler: Box<Bundler+Send>) {
        self.event_bundlers.push(bundler);
    }

    pub fn add_bundler_trigger(&mut self, bundler: Box<BundlerTrigger+Send>) {
        self.bundler_triggers.push(bundler);
    }

    pub fn on_message(&mut self, message: &IrcMessage) -> Vec<IrcEvent> {
        let mut outgoing_events: Vec<IrcEvent> = Vec::new();

        for new_bundler in bundler_trigger_impl(&mut self.bundler_triggers, message).into_iter() {
            self.event_bundlers.push(new_bundler);
        }

        for event in bundler_accept_impl(&mut self.event_bundlers, message).into_iter() {
            outgoing_events.push(event);
        }

        outgoing_events.push(IrcEventMessage(message.clone()));

        for event in outgoing_events.iter() {
            for watcher in watcher_accept_impl(&mut self.event_watchers, event).into_iter() {
                drop(watcher);
            }
        }

        outgoing_events
    }
}

fn bundler_trigger_impl(triggers: &mut Vec<Box<BundlerTrigger+Send>>,
                       message: &IrcMessage
                      ) -> Vec<Box<Bundler+Send>> {

    let mut activating: Vec<Box<Bundler+Send>> = Vec::new();
    for trigger in triggers.iter_mut() {
        let new_bundlers = trigger.on_message(message);
        activating.reserve_additional(new_bundlers.len());
        for bundler in new_bundlers.into_iter() {
            activating.push(bundler);
        }
    }
    activating
}


fn watcher_accept_impl(buf: &mut RingBuf<Box<EventWatcher+Send>>,
                       event: &IrcEvent
                      ) -> Vec<Box<EventWatcher+Send>> {
    let mut keep_watchers: RingBuf<Box<EventWatcher+Send>> = RingBuf::new();
    let mut finished_watchers: Vec<Box<EventWatcher+Send>> = Vec::new();

    loop {
        match buf.pop_front() {
            Some(mut watcher) => {
                watcher.on_event(event);
                if watcher.is_finished() {
                    finished_watchers.push(watcher);
                } else {
                    keep_watchers.push(watcher);
                }
            },
            None => break
        }
    }
    loop {
        match keep_watchers.pop_front() {
            Some(watcher) => buf.push(watcher),
            None => break
        }
    }
    finished_watchers
}


fn bundler_accept_impl(buf: &mut RingBuf<Box<Bundler+Send>>,
                       message: &IrcMessage
                      ) -> Vec<IrcEvent> {

    let mut keep_bundlers: RingBuf<Box<Bundler+Send>> = RingBuf::new();
    let mut emit_events: Vec<IrcEvent> = Vec::new();

    loop {
        match buf.pop_front() {
            Some(mut bundler) => {
                for event in bundler.on_message(message).into_iter() {
                    emit_events.push(event);
                }
                if !bundler.is_finished() {
                    keep_bundlers.push(bundler);
                }
            },
            None => break
        }
    }
    loop {
        match keep_bundlers.pop_front() {
            Some(watcher) => buf.push(watcher),
            None => break
        }
    }
    emit_events
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{IoResult, BufReader};

    use message::IrcMessage;
    use watchers::{
        WhoBundlerTrigger,
        JoinBundlerTrigger,
    };
    use event::{
        IrcEventJoinBundle,
        IrcEventWhoBundle,
    };

    const TEST_DATA: &'static [u8] = include_bin!("../../testdata/watcher.txt");

    fn unsafe_to_irc_message(line_res: IoResult<String>) -> IrcMessage {
        let line = match line_res {
            Ok(line) => line,
            Err(err) => panic!("err: {}", err)
        };
        let totrim: &[_] = &['\n', '\r'];
        match IrcMessage::from_str(line.as_slice().trim_right_chars(totrim)) {
            Ok(message) => message,
            Err(err) => panic!("err: {}", err)
        }
    }

    #[test]
    fn test_bundle_watcher() {
        let mut reader = BufReader::new(TEST_DATA);
        let mut bunman = BundlerManager::new();
        bunman.add_bundler_trigger(box JoinBundlerTrigger::new());
        bunman.add_bundler_trigger(box WhoBundlerTrigger::new());
        let mut events = Vec::new();

        for msg in reader.lines().map(unsafe_to_irc_message) {
            events.extend(bunman.on_message(&msg).into_iter());
        }

        let mut join_bundles = 0u;
        let mut who_bundles = 0u;

        for event in events.into_iter() {
            if let IrcEventJoinBundle(_) = event {
                join_bundles += 1;
            }
            if let IrcEventWhoBundle(_) = event {
                who_bundles += 1
            }
        }
        assert_eq!(join_bundles, 1);
        assert_eq!(who_bundles, 1);
    }
}
