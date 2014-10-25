use std::collections::{RingBuf, Deque};

use message::IrcMessage;
use watchers::event::{IrcEvent, IrcEventMessage};

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

    pub fn dispatch(&mut self, message: &IrcMessage) -> Vec<IrcEvent> {
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


#[test]
fn test_bundle_watcher() {
    //
}

// """
// :botnick!rustbot@hostname JOIN :#
// :server 332 botnick # :# - EBOLA EBOLA JUST LIVIN IN THE EBOLA | RIP LoleBola 2006 - 2014 | [21:16:06] <plus> Ebola sure takes a long time to compile | Take the # Fall Ebola Challenge | free EbolA, contact IB | EBOLA 2014 IS UPON US | much of # is relocating to flee the ebola menace | eBOWLa 2014 - WHO WILL EMERGE VICTORIOUS? | T
// :server 333 botnick # owls!owl@miyu.godless-internets.org 1414115720
// :server 353 botnick = # :+sell +nagisapls flsp +aibi botnick stick plus ngy|casper pem +ildverden +owls Leafa usagi Faux pasv +tanasinn +theLORD mr_flea Yukirin cthuljew betabot miyu tmpy 
// :server 366 botnick # :End of /NAMES list.
// """