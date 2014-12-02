use std::io::net::ip::ToSocketAddr;
use std::collections::RingBuf;
use std::task::TaskBuilder;
use std::io::{
    IoResult, IoError, EndOfFile,
    BufferedReader, TcpStream,
    LineBufferedWriter
};
use std::default::Default;
use std::fmt;

use core_plugins::{
    MessageResponder,
    CtcpVersionResponder,
};

use event::IrcEvent;
use message::IrcMessage;
use message_types::server;
pub use connection::IrcConnectionCommand::{
    RawWrite,
    AddWatcher,
    AddBundler,
};

use watchers::{
    Bundler,
    BundlerManager,
    RegisterError,
    RegisterEventWatcher,
    JoinBundlerTrigger,
    JoinResult,
    JoinEventWatcher,
    WhoBundlerTrigger,
    WhoResult,
    WhoEventWatcher,
    EventWatcher,
    BundlerTrigger,
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

    fn dispatch(&mut self, message: IrcMessage, raw_sender: &SyncSender<Vec<u8>>) {
        if let server::IncomingMsg::Ping(ref msg) = *message.get_typed_message() {
            if let Ok(response) = msg.get_response() {
                raw_sender.send(response.into_bytes());
            }
        }

        if message.command() == "001" {
            let msg = message.get_typed_message().to_irc_msg();
            self.current_nick = Some(String::from_utf8_lossy(&msg[0]).into_owned());
        }

        if let server::IncomingMsg::Nick(ref msg) = *message.get_typed_message() {
            if let Some(current_nick) = self.current_nick.take() {
                if current_nick[] == msg.get_nick() {
                    self.current_nick = Some(msg.get_new_nick().to_string())
                }
            }
        }

        // XXX //
        let outgoing_events = self.bundler_man.on_message(&message);

        for responder in self.responders.iter_mut() {
            for message in responder.on_message(&message).into_iter() {
                raw_sender.send(message.into_bytes());
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
            IrcConnectionCommand::RawWrite(ref s) => write!(f, "RawWrite({})", s),
            IrcConnectionCommand::AddWatcher(ref ew) => write!(f, "AddWatcher({}(...))", ew.get_name()),
            IrcConnectionCommand::AddBundler(ref bun) => write!(f, "AddBundler({}(...))", bun.get_name()),
        }
    }
}


impl IrcConnection {
    fn new_from_rw<R: Reader+Send, W: Writer+Send>(reader: R, writer: W)
            -> IoResult<(IrcConnection, Receiver<IrcEvent>)> {
        let (command_queue_tx, command_queue_rx) = sync_channel::<IrcConnectionCommand>(0);
        let (event_queue_tx, event_queue_rx) = sync_channel(10);
        
        let (raw_writer_tx, raw_writer_rx) = sync_channel::<Vec<u8>>(20);
        let (raw_reader_tx, raw_reader_rx) = sync_channel::<String>(20);

        TaskBuilder::new().named("core-writer").spawn(proc() {
            let mut writer = LineBufferedWriter::new(writer);
            for message in raw_writer_rx.iter() {
                let mut message = message.clone();
                message.push_all(b"\r\n");
                assert!(writer.write(message.as_slice()).is_ok());
            }
            warn!("--!-- core-writer is ending! --!--");
        });

        TaskBuilder::new().named("core-reader").spawn(proc() {
            let trim_these: &[char] = &['\r', '\n'];
            let mut reader = BufferedReader::new(reader);
            loop {
                let line_bin = match reader.read_until('\n' as u8) {
                    Ok(line_bin) => line_bin,
                    Err(IoError{ kind: EndOfFile, .. }) => break,
                    Err(err) => panic!("I/O Error: {}", err)
                };
                let string = String::from_utf8_lossy(line_bin.as_slice());
                let string = string.as_slice().trim_right_chars(trim_these).to_string();
                raw_reader_tx.send(string);
            }
            warn!("--!-- core-reader is ending! --!--");
        });

        TaskBuilder::new().named("core-dispatch").spawn(proc() {
            let mut state = IrcConnectionInternalState::new(event_queue_tx);
            state.bundler_man.add_bundler_trigger(box JoinBundlerTrigger::new());
            state.bundler_man.add_bundler_trigger(box WhoBundlerTrigger::new());
            state.responders.push_back(box CtcpVersionResponder::new());

            loop {
                select! {
                    command = command_queue_rx.recv() => {
                        match command {
                            IrcConnectionCommand::RawWrite(value) => raw_writer_tx.send(value),
                            IrcConnectionCommand::AddWatcher(value) => state.add_watcher(value),
                            IrcConnectionCommand::AddBundler(value) => state.add_bundler(value),
                        }
                    },
                    string = raw_reader_rx.recv() => {
                        state.dispatch(match IrcMessage::from_str(string.as_slice()) {
                            Ok(message) => message,
                            Err(err) => {
                                warn!("Invalid IRC message: {} for {}", err, string);
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

    pub fn new<A: ToSocketAddr>(addr: A) -> IoResult<(IrcConnection, Receiver<IrcEvent>)> {
        let stream = match TcpStream::connect(addr) {
            Ok(stream) => stream,
            Err(err) => return Err(err)
        };
        IrcConnection::new_from_rw(stream.clone(), stream.clone())
    }

    pub fn register(&mut self, nick: &str) -> Result<(), RegisterError> {
        let mut reg_watcher = RegisterEventWatcher::new();        
        let result_rx = reg_watcher.get_monitor();
        let watcher: Box<EventWatcher+Send> = box reg_watcher;
        self.command_queue.send(AddWatcher(watcher));
        self.write_str(format!("NICK {}", nick).as_slice());
        if !self.has_registered {
            self.write_str("USER rustirc 0 *: http://github.com/infinityb/rust-irc");
        }
        let register_result = result_rx.recv();
        register_result
    }

    pub fn join(&mut self, channel: &str) -> JoinResult {
        let mut join_watcher = JoinEventWatcher::new(channel);
        let result_rx = join_watcher.get_monitor();
        let watcher: Box<EventWatcher+Send> = box join_watcher;
        self.command_queue.send(IrcConnectionCommand::AddWatcher(watcher));

        let mut join_msg = Vec::new();
        join_msg.push_all(b"JOIN ");
        join_msg.push_all(channel.as_slice().as_bytes());

        self.command_queue.send(IrcConnectionCommand::RawWrite(join_msg));
        result_rx.recv()
    }

    pub fn who(&mut self, target: &str) -> WhoResult {
        let mut who_watcher = WhoEventWatcher::new(target);
        let result_rx = who_watcher.get_monitor();
        let watcher: Box<EventWatcher+Send> = box who_watcher;
        self.command_queue.send(AddWatcher(watcher));

        let mut who_msg = Vec::new();
        who_msg.push_all(b"WHO ");
        who_msg.push_all(target.as_bytes());

        self.command_queue.send(RawWrite(who_msg));
        result_rx.recv()
    }

    pub fn write_str(&mut self, content: &str) {
        self.write_buf(content.as_bytes());
    }

    pub fn write_buf(&mut self, content: &[u8]) {
        self.command_queue.send(RawWrite(content.into_cow().into_owned()));
    }

    pub fn get_command_queue(&mut self) -> SyncSender<IrcConnectionCommand> {
        self.command_queue.clone()
    }
}
