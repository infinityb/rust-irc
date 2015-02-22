use std::old_io::net::ip::ToSocketAddr;
use std::collections::VecDeque;
use std::sync::Future;
use std::fmt;
use std::old_io::{
    IoResult, IoError, EndOfFile,
    BufferedReader, TcpStream,
    LineBufferedWriter
};
use std::default::Default;
use std::sync::mpsc::{sync_channel, SyncSender, Receiver};

use core_plugins::{
    MessageResponder,
    CtcpVersionResponder,
};
use parse::IrcMsg;
use event::IrcEvent;
use message_types::{client, server};
pub use connection::IrcConnectionCommand::{
    RawWrite,
    AddWatcher,
    AddBundler,
};

use watchers::{
    Bundler,
    BundlerManager,
    RegisterEventWatcher,
    RegisterResult,
    JoinBundlerTrigger,
    JoinResult,
    JoinEventWatcher,
    WhoBundlerTrigger,
    WhoResult,
    WhoEventWatcher,
    EventWatcher,
    BundlerTrigger,
};

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

impl fmt::Debug for IrcConnectionCommand {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IrcConnectionCommand::RawWrite(ref s) => write!(f, "RawWrite({:?})", s),
            IrcConnectionCommand::AddWatcher(ref ew) => write!(f, "AddWatcher({:?}(...))", ew.get_name()),
            IrcConnectionCommand::AddBundler(ref bun) => write!(f, "AddBundler({:?}(...))", bun.get_name()),
        }
    }
}

pub struct IrcConnectionBuf {
    // Lines coming from the server
    incoming_lines: VecDeque<Vec<u8>>,

    // Lines going out to the server
    outgoing_msgs: VecDeque<Vec<u8>>,

    // outgoing event queue
    event_queue: VecDeque<IrcEvent>,

    // Internal command queue
    command_queue: VecDeque<IrcConnectionCommand>,

    // Automatic responders e.g. the PING and CTCP handlers
    responders: VecDeque<Box<MessageResponder+Send>>,

    // manages active bundlers and emits events when they finish
    bundler_man: BundlerManager,

    // Current nickname held by the client
    current_nick: Option<String>,
}


impl IrcConnectionBuf {
    pub fn new() -> IrcConnectionBuf {
        let mut out = IrcConnectionBuf {
            incoming_lines: VecDeque::new(),
            outgoing_msgs: VecDeque::new(),
            event_queue: VecDeque::new(),
            command_queue: VecDeque::new(),
            responders: VecDeque::new(),
            bundler_man: BundlerManager::new(),
            current_nick: None,
        };
        out.responders.push_back(Box::new(CtcpVersionResponder::new()));
        out.bundler_man.add_bundler_trigger(Box::new(JoinBundlerTrigger::new()));
        out.bundler_man.add_bundler_trigger(Box::new(WhoBundlerTrigger::new()));
        out
    }

    pub fn push_line(&mut self, msg: Vec<u8>) {
        self.incoming_lines.push_back(msg);
    }

    pub fn pop_line(&mut self) -> Option<Vec<u8>> {
        match self.outgoing_msgs.pop_front() {
            Some(mut outgoing_line) => {
                outgoing_line.push_all(b"\r\n");
                Some(outgoing_line)
            },
            None => None
        }
    }

    fn add_watcher(&mut self, watcher: Box<EventWatcher+Send>) {
        self.bundler_man.add_watcher(watcher);
    }

    fn add_bundler(&mut self, bundler: Box<Bundler+Send>) {
        self.bundler_man.add_bundler(bundler);
    }

    pub fn dispatch(&mut self) {
        while let Some(command) = self.command_queue.pop_front() {
            use self::IrcConnectionCommand::{RawWrite, AddWatcher, AddBundler};
            match command {
                RawWrite(value) => self.outgoing_msgs.push_back(value),
                AddWatcher(value) => self.add_watcher(value),
                AddBundler(value) => self.add_bundler(value),
            }
        }

        while let Some(incoming) = self.incoming_lines.pop_front() {
            let incoming_copy = incoming.clone();

            let msg = match IrcMsg::new(incoming) {
                Ok(msg) => msg,
                Err(reason) => {
                    panic!("Invalid msg: {:?} for {:?}", reason, incoming_copy);
                }
            };

            if msg.get_command() == "001" {
                self.current_nick = Some(String::from_utf8_lossy(&msg[0]).into_owned());
            }

            let msg = {
                let tymsg = server::IncomingMsg::from_msg(msg);

                if let server::IncomingMsg::Ping(ref msg) = tymsg {
                    if let Ok(response) = msg.get_response() {
                        self.outgoing_msgs.push_back(response.into_bytes());
                    }
                }

                if let server::IncomingMsg::Nick(ref msg) = tymsg {
                    if let Some(current_nick) = self.current_nick.clone() {
                        if current_nick.as_slice() == msg.get_nick() {
                            self.current_nick = Some(msg.get_new_nick().to_string())
                        }
                    }
                }

                tymsg.into_irc_msg()
            };

            self.event_queue.extend(self.bundler_man.on_irc_msg(&msg).into_iter());
            for responder in self.responders.iter_mut() {
                for resp_msg in responder.on_irc_msg(&msg).into_iter() {
                    self.outgoing_msgs.push_back(resp_msg.into_bytes());
                }
            }
        }
    }

    pub fn pop_event(&mut self) -> Option<IrcEvent> {
        self.event_queue.pop_front()
    }

    pub fn register(&mut self, nick: &str) -> Future<RegisterResult> {
        use self::IrcConnectionCommand::AddWatcher;
        let mut reg_watcher = RegisterEventWatcher::new();
        let result_future = reg_watcher.get_future();
        let watcher: Box<EventWatcher+Send> = Box::new(reg_watcher);
        self.command_queue.push_back(AddWatcher(watcher));
        self.outgoing_msgs.push_back(client::Nick::new(nick).into_irc_msg().into_bytes());
        self.outgoing_msgs.push_back(client::User::new(
            "rustirc", "0", "*", "http://github.com/infinityb/rust-irc"
        ).into_irc_msg().into_bytes());
        result_future
    }

    pub fn join(&mut self, channel: &str) -> Future<JoinResult> {
        use self::IrcConnectionCommand::AddWatcher;
        let mut join_watcher = JoinEventWatcher::new(channel.as_bytes());
        let result_future = join_watcher.get_future();
        let watcher: Box<EventWatcher+Send> = Box::new(join_watcher);
        self.command_queue.push_back(AddWatcher(watcher));
        self.outgoing_msgs.push_back(client::Join::new(channel).into_irc_msg().into_bytes());
        result_future
    }

    pub fn who(&mut self, target: &str) -> Future<WhoResult> {
        use self::IrcConnectionCommand::AddWatcher;
        let mut who_watcher = WhoEventWatcher::new(target.as_bytes());
        let result_future = who_watcher.get_future();
        let watcher: Box<EventWatcher+Send> = Box::new(who_watcher);
        self.command_queue.push_back(AddWatcher(watcher));
        self.outgoing_msgs.push_back(client::Who::new(target).into_irc_msg().into_bytes());
        result_future
    }
}

#[cfg(test)]
mod tests {
    use super::IrcConnectionBuf;

    #[test]
    fn test_pingpong() {
        let mut conn = IrcConnectionBuf::new();

        assert_eq!(conn.pop_line(), None);
        conn.push_line(b"PING pretend-server\r\n".to_vec());
        assert_eq!(conn.pop_line(), None);

        conn.dispatch();
        assert!(conn.pop_event().is_some());
        assert!(conn.pop_event().is_none());
        assert_eq!(conn.pop_line(), Some(b"PONG pretend-server\r\n".to_vec()));
    }
}


pub struct IrcConnection {
    command_queue: SyncSender<IrcConnectionCommand>,
    has_registered: bool
}


struct IrcConnectionInternalState {
    // The output stream towards the user
    event_queue_tx: SyncSender<IrcEvent>,

    // Automatic responders e.g. the PING and CTCP handlers
    responders: VecDeque<Box<MessageResponder+Send>>,

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

    fn dispatch(&mut self, msg: IrcMsg, raw_sender: &SyncSender<Vec<u8>>) {
        let tymsg = server::IncomingMsg::from_msg(msg);
        if let server::IncomingMsg::Ping(ref msg) = tymsg {
            if let Ok(response) = msg.get_response() {
                raw_sender.send(response.into_bytes()).unwrap();
            }
        }
        if let server::IncomingMsg::Numeric(1, ref numeric) = tymsg {
            let msg = numeric.to_irc_msg();
            self.current_nick = Some(String::from_utf8_lossy(&msg[0]).into_owned());
        }

        if let server::IncomingMsg::Nick(ref msg) = tymsg {
            if let Some(current_nick) = self.current_nick.take() {
                if current_nick.as_slice() == msg.get_nick() {
                    self.current_nick = Some(msg.get_new_nick().to_string())
                }
            }
        }

        let outgoing_events = self.bundler_man.on_irc_msg(tymsg.to_irc_msg());

        for responder in self.responders.iter_mut() {
            for msg in responder.on_irc_msg(tymsg.to_irc_msg()).into_iter() {
                raw_sender.send(msg.into_bytes()).unwrap();
            }
        }

        for event in outgoing_events.into_iter() {
            self.event_queue_tx.send(event).unwrap();
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

impl IrcConnection {
    fn new_from_rw<R, W>(reader: R, writer: W)
            -> IoResult<(IrcConnection, Receiver<IrcEvent>)>
        where
           R: Reader+Send+'static,
           W: Writer+Send+'static {
        let (command_queue_tx, command_queue_rx) = sync_channel::<IrcConnectionCommand>(0);
        let (event_queue_tx, event_queue_rx) = sync_channel(10);
        
        let (raw_writer_tx, raw_writer_rx) = sync_channel::<Vec<u8>>(20);
        let (raw_reader_tx, raw_reader_rx) = sync_channel::<Vec<u8>>(20);

        let _ = ::std::thread::Builder::new().name("core-writer".to_string()).spawn(move || {
            let mut writer = LineBufferedWriter::new(writer);
            for message in raw_writer_rx.iter() {
                let mut message = message.clone();
                message.push_all(b"\r\n");
                assert!(writer.write_all(message.as_slice()).is_ok());
            }
            warn!("--!-- core-writer is ending! --!--");
        });

        let _ = ::std::thread::Builder::new().name("core-reader".to_string()).spawn(move || {
            let mut reader = BufferedReader::new(reader);
            loop {
                let line_bin = match reader.read_until('\n' as u8) {
                    Ok(line_bin) => deline(line_bin),
                    Err(IoError{ kind: EndOfFile, .. }) => break,
                    Err(err) => panic!("I/O Error: {}", err)
                };
                raw_reader_tx.send(line_bin).unwrap();
            }
            warn!("--!-- core-reader is ending! --!--");
        });

        let _ = ::std::thread::Builder::new().name("core-dispatch".to_string()).spawn(move || {
            let mut state = IrcConnectionInternalState::new(event_queue_tx);
            state.bundler_man.add_bundler_trigger(Box::new(JoinBundlerTrigger::new()));
            state.bundler_man.add_bundler_trigger(Box::new(WhoBundlerTrigger::new()));
            state.responders.push_back(Box::new(CtcpVersionResponder::new()));

            loop {
                select! {
                    command = command_queue_rx.recv() => {
                        match command.unwrap() {
                            IrcConnectionCommand::RawWrite(value) => {
                                raw_writer_tx.send(value).unwrap();
                            }
                            IrcConnectionCommand::AddWatcher(value) => state.add_watcher(value),
                            IrcConnectionCommand::AddBundler(value) => state.add_bundler(value),
                        }
                    },
                    string = raw_reader_rx.recv() => {
                        let string = string.unwrap();
                        state.dispatch(match IrcMsg::new(string) {
                            Ok(message) => message,
                            Err(err) => {
                                warn!("Invalid IRC message: {:?}", err);
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

    pub fn register(&mut self, nick: &str) -> RegisterResult {
        let mut reg_watcher = RegisterEventWatcher::new();        
        let result_rx = reg_watcher.get_monitor();
        let watcher: Box<EventWatcher+Send> = Box::new(reg_watcher);
        self.command_queue.send(AddWatcher(watcher)).unwrap();
        self.write_str(format!("NICK {}", nick).as_slice());
        if !self.has_registered {
            self.write_str("USER rustirc 0 *: http://github.com/infinityb/rust-irc");
        }
        let register_result = result_rx.recv();
        register_result.unwrap()
    }

    pub fn join(&mut self, channel: &str) -> JoinResult {
        let mut join_watcher = JoinEventWatcher::new(channel.as_bytes());
        let result_rx = join_watcher.get_monitor();
        let watcher: Box<EventWatcher+Send> = Box::new(join_watcher);
        self.command_queue.send(IrcConnectionCommand::AddWatcher(watcher)).unwrap();

        let mut join_msg = Vec::new();
        join_msg.push_all(b"JOIN ");
        join_msg.push_all(channel.as_slice().as_bytes());

        self.command_queue.send(IrcConnectionCommand::RawWrite(join_msg)).unwrap();
        result_rx.recv().unwrap()
    }

    pub fn who(&mut self, target: &str) -> WhoResult {
        let mut who_watcher = WhoEventWatcher::new(target.as_bytes());
        let result_rx = who_watcher.get_monitor();
        let watcher: Box<EventWatcher+Send> = Box::new(who_watcher);
        self.command_queue.send(AddWatcher(watcher)).unwrap();

        let mut who_msg = Vec::new();
        who_msg.push_all(b"WHO ");
        who_msg.push_all(target.as_bytes());

        self.command_queue.send(RawWrite(who_msg)).unwrap();
        result_rx.recv().unwrap()
    }

    pub fn write_str(&mut self, content: &str) {
        self.write_buf(content.as_bytes());
    }

    pub fn write_buf(&mut self, content: &[u8]) {
        self.command_queue.send(RawWrite(content.to_vec())).unwrap();
    }

    pub fn get_command_queue(&mut self) -> SyncSender<IrcConnectionCommand> {
        self.command_queue.clone()
    }
}

/// Removes newline characters from a line
fn deline(mut line: Vec<u8>) -> Vec<u8> {
    if Some(&b'\n') != line.iter().last() {
        return line;
    }
    line.pop();
    if Some(&b'\r') != line.iter().last() {
        return line;
    }
    line.pop();
    line
}