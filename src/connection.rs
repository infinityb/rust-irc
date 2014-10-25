use std::collections::{RingBuf, Deque};
use std::task::TaskBuilder;
use std::io::{TcpStream, IoResult, LineBufferedWriter, BufferedReader};
use std::default::Default;

use core_plugins::{
    MessageResponder,
    CtcpVersionResponder,
};

use message::IrcMessage;

use watchers::{
    Bundler,
    BundlerManager,
    RegisterError,
    RegisterEventWatcher,
    JoinBundlerTrigger,
    JoinResult,
    JoinEventWatcher,
    WhoBundlerTrigger,
    WhoBundler,
    WhoResult,
    WhoEventWatcher,
    EventWatcher,
    BundlerTrigger,
    IrcEvent,
    IrcEventMessage
};


pub struct IrcConnection {
    command_queue: SyncSender<IrcConnectionCommand>,
    has_registered: bool
}


struct IrcConnectionInternalState {
    // The output stream towards the user
    event_queue_tx: SyncSender<IrcEvent>,

    // Automatic responders e.g. the PING and CTCP handlers
    responders: RingBuf<Box<MessageResponder+Send>>,

    // manages active bundlers and emits events when they finish
    bundler_man: BundlerManager,

    // Current nickname held by the client
    current_nick: Option<String>
}



impl IrcConnectionInternalState {
    pub fn new(event_queue_tx: SyncSender<IrcEvent>) -> IrcConnectionInternalState {
        IrcConnectionInternalState {
            event_queue_tx: event_queue_tx,
            responders: Default::default(),
            bundler_man: BundlerManager::new(),
            current_nick: Default::default(),
        }
    }

    fn dispatch(&mut self, message: IrcMessage, raw_sender: &SyncSender<String>) {
        if message.command() == "PING" {
            let ping_body: &String = message.get_arg(0);
            raw_sender.send(format!("PONG :{}\n", ping_body));
        }

        if message.command() == "001" {
            let accepted_nick: &String = message.get_arg(0);
            self.current_nick = Some(accepted_nick.clone());
        }

        if message.command() == "NICK" {
            self.current_nick = match (message.source_nick(), self.current_nick.take()) {
                (Some(source_nick), Some(current_nick)) => {
                    if source_nick == current_nick {
                        Some(message.get_arg(0).clone())
                    } else {
                        Some(current_nick)
                    }
                },
                (_, any) => any
            };
        }

        // XXX //
        let outgoing_events = self.bundler_man.dispatch(&message);

        for responder in self.responders.iter_mut() {
            for message in responder.on_message(&message).into_iter() {
                raw_sender.send(message.to_irc());
            }
        }

        for event in outgoing_events.into_iter() {
            self.event_queue_tx.send(event);
        }
    }

    // Do we really need Send here?
    fn add_watcher(&mut self, watcher: Box<EventWatcher+Send>) {
        self.bundler_man.add_watcher(watcher);
    }

    fn add_bundler(&mut self, bundler: Box<Bundler+Send>) {
        self.bundler_man.add_bundler(bundler);
    }
}


pub enum IrcConnectionCommand {
    RawWrite(String),
    AddWatcher(Box<EventWatcher+Send>),
    AddBundler(Box<Bundler+Send>),
}


impl IrcConnection {
    pub fn new(host: &str, port: u16) -> IoResult<(IrcConnection, Receiver<IrcEvent>)> {
        let stream = match TcpStream::connect(host, port) {
            Ok(stream) => stream,
            Err(err) => return Err(err)
        };

        let (command_queue_tx, command_queue_rx) = sync_channel::<IrcConnectionCommand>(10);
        let (event_queue_tx, event_queue_rx) = sync_channel(1024);
        
        let reader = BufferedReader::new(stream.clone());

        let tmp_stream = stream.clone();
        let (raw_writer_tx, raw_writer_rx) = sync_channel::<String>(0);
        let (raw_reader_tx, raw_reader_rx) = sync_channel::<String>(0);


        TaskBuilder::new().named("core-writer").spawn(proc() {
            let mut writer = LineBufferedWriter::new(tmp_stream);
            for message in raw_writer_rx.iter() {
                let mut message = message.clone();
                message.push_str("\n");
                assert!(writer.write_str(message.as_slice()).is_ok());
            }
        });

        TaskBuilder::new().named("core-reader").spawn(proc() {
            let mut reader = reader;
            loop {
                let string = String::from_str(match reader.read_line() {
                        Ok(string) => string,
                        Err(err) => fail!("{}", err)
                    }.as_slice().trim_right());
                raw_reader_tx.send(string);
            }
        });

        TaskBuilder::new().named("core-dispatch").spawn(proc() {
            let mut state = IrcConnectionInternalState::new(event_queue_tx);

            state.bundler_man.add_bundler_trigger(box JoinBundlerTrigger::new());
            state.bundler_man.add_bundler_trigger(box WhoBundlerTrigger::new());
            state.responders.push(box CtcpVersionResponder::new());

            loop {
                select! {
                    command = command_queue_rx.recv() => {
                        match command {
                            RawWrite(value) => raw_writer_tx.send(value),
                            AddWatcher(value) => state.add_watcher(value),
                            AddBundler(value) => state.add_bundler(value),
                        }
                    },
                    string = raw_reader_rx.recv() => {
                        state.dispatch(match IrcMessage::from_str(string.as_slice()) {
                            Ok(message) => message,
                            Err(err) => {
                                println!("Invalid IRC message: {} for {}", err, string);
                                continue;
                            }
                        }, &raw_writer_tx);
                    }
                }

            }
        });

        let conn = IrcConnection {
            command_queue: command_queue_tx,
            has_registered: false
        };
        Ok((conn, event_queue_rx))
    }

    pub fn register(&mut self, nick: &str) -> Result<(), RegisterError> {
        let mut reg_watcher = RegisterEventWatcher::new();        
        let result_rx = reg_watcher.get_monitor();
        let watcher: Box<EventWatcher+Send> = box reg_watcher;
        self.command_queue.send(AddWatcher(watcher));
        self.write_str(format!("NICK {}", nick).as_slice());
        if !self.has_registered {
            self.write_str("USER rustbot 8 *: Rust Bot");
        }
        result_rx.recv()
    }

    pub fn join(&mut self, channel: &str) -> JoinResult {
        let mut join_watcher = JoinEventWatcher::new(channel);
        let result_rx = join_watcher.get_monitor();
        let watcher: Box<EventWatcher+Send> = box join_watcher;
        self.command_queue.send(AddWatcher(watcher));
        self.command_queue.send(RawWrite(format!("JOIN {}", channel.as_slice())));
        result_rx.recv()
    }

    pub fn who(&mut self, target: &str) -> WhoResult {
        let mut who_watcher = WhoEventWatcher::new(target);
        let result_rx = who_watcher.get_monitor();
        let watcher: Box<EventWatcher+Send> = box who_watcher;
        // TODO: we should probably make this a bundle-trigger.  We need to 
        // ensure bundle gets the message that triggers the bundle-trigger
        self.command_queue.send(AddBundler(box WhoBundler::new(target)));
        self.command_queue.send(AddWatcher(watcher));
        self.command_queue.send(RawWrite(format!("WHO {}", target)));
        result_rx.recv()
    }

    pub fn write_str(&mut self, content: &str) {
        self.command_queue.send(RawWrite(String::from_str(content)))
    }

    pub fn get_command_queue(&mut self) -> SyncSender<IrcConnectionCommand> {
        self.command_queue.clone()
    }
}
