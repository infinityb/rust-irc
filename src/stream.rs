use std::io::{self, Write, BufRead};
use parse::{IrcMsg, ParseError};
use message_types::{client, server};
use state::State;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Parse(ParseError),
    Empty,
}

pub struct IrcReaderIter<'a> {
    reader: &'a mut IrcReader,
}

impl<'a> Iterator for IrcReaderIter<'a> {
    type Item = Result<IrcMsg, Error>;

    fn next(&mut self) -> Option<Result<IrcMsg, Error>> {
        match self.reader.get_irc_msg() {
            Err(Error::Empty) => None,
            r @ _ => Some(r),
        }
    }
}

pub struct IrcReader {
    reader: Box<BufRead+Send+'static>,
}

impl IrcReader {
    pub fn get_irc_msg(&mut self) -> Result<IrcMsg, Error> {
        let mut buf: Vec<u8> = Vec::new();
        if let Err(err) = self.reader.read_until(b'\n', &mut buf) {
            return Err(Error::Io(err));
        }
        if buf.len() == 0 {
            return Err(Error::Empty);
        }
        match IrcMsg::new(buf) {
            Ok(msg) => Ok(msg),
            Err(err) => Err(Error::Parse(err))
        }
    }

    pub fn iter<'a>(&'a mut self) -> IrcReaderIter<'a> {
        IrcReaderIter { reader: self }
    }
}

pub struct IrcWriter {
    writer: Box<Write+Send+'static>
}

impl IrcWriter {
    pub fn write_irc_msg(&mut self, msg: &IrcMsg) -> io::Result<()> {
        let buf = msg.as_bytes();
        assert_eq!(try!(self.writer.write(buf)), buf.len());
        assert_eq!(try!(self.writer.write(b"\r\n")), 2);
        try!(self.writer.flush());
        Ok(())
    }
}

#[derive(Debug)]
pub enum RegisterError {
    Stream(Error),
    InvalidUser,
    InvalidNick,
    NickInUse,
}

pub struct RegisterReqBuilder {
    nick: Option<String>,
    user: Option<String>,
    realname: String,

    wallops: bool,
    invisible: bool,
}

impl RegisterReqBuilder {
    pub fn new() -> RegisterReqBuilder {
        RegisterReqBuilder {
            nick: None,
            user: None,
            realname: "http://github.com/infinityb/rust-irc".to_string(),
            wallops: false,
            invisible: false,
        }
    }

    pub fn nick(&mut self, val: &str) -> &mut RegisterReqBuilder {
        self.nick = Some(val.to_string());
        self
    }

    pub fn user(&mut self, val: &str) -> &mut RegisterReqBuilder {
        self.user = Some(val.to_string());
        self
    }

    pub fn realname(&mut self, val: &str) -> &mut RegisterReqBuilder {
        self.realname = val.to_string();
        self
    }

    pub fn mode_invisible(&mut self, val: bool) -> &mut RegisterReqBuilder {
        self.invisible = val;
        self
    }

    pub fn mode_wallops(&mut self, val: bool) -> &mut RegisterReqBuilder {
        self.invisible = val;
        self
    }

    pub fn build(&mut self) -> Result<RegisterRequest, &'static str> {
        let nick = match self.nick {
            Some(ref x) if x.len() == 0 => return Err("nick must be non-empty"),
            None => return Err("nick must be set"),
            Some(ref nick) => nick.clone(),
        };
        let user = match self.user {
            Some(ref x) if x.len() == 0 => return Err("user must be non-empty"),
            None => return Err("user must be set"),
            Some(ref user) => user.clone(),
        };
        if self.realname.len() == 0 {
            return Err("realname must be non-empty")
        }
        Ok(RegisterRequest {
            nick: nick,
            user: user,
            realname: self.realname.clone(),
            wallops: self.wallops,
            invisible: self.invisible,
        })
    }
}

pub struct RegisterRequest {
    nick: String,
    user: String,
    realname: String,

    wallops: bool,
    invisible: bool,
}

impl RegisterRequest {
    pub fn get_mut_nick(&mut self) -> &mut String {
        &mut self.nick
    }

    fn get_user(&self) -> client::User {
        let mut mode = 0;
        if self.wallops {
            mode += 1 << 2;
        }
        if self.invisible {
            mode += 1 << 3;
        }

        client::User::new(
            self.user.as_slice(),
            &format!("{}", mode), "*",
            self.realname.as_slice(),
        )
    }

    fn get_nick(&self) -> client::Nick {
        client::Nick::new(self.nick.as_slice())
    }
}

pub struct IrcConnector {
    reader: IrcReader,
    writer: IrcWriter,
    user_sent: bool,
}

impl IrcConnector {
    pub fn from_pair(reader: Box<BufRead+Send+'static>, writer: Box<Write+Send+'static>) -> IrcConnector {
        IrcConnector {
            reader: IrcReader { reader: reader },
            writer: IrcWriter { writer: writer },
            user_sent: false,
        }
    }

    pub fn register(&mut self, req: &RegisterRequest) -> Result<State, RegisterError> {
        if !self.user_sent {
            if let Err(err) = self.writer.write_irc_msg(&req.get_user().into_irc_msg()) {
                return Err(RegisterError::Stream(Error::Io(err)))
            }
            self.user_sent = true;
        }
        if let Err(err) = self.writer.write_irc_msg(&req.get_nick().into_irc_msg()) {
            return Err(RegisterError::Stream(Error::Io(err)))
        }
        
        let mut state = State::new();

        for msg in self.reader.iter() {
            let msg = match msg {
                Ok(msg) => msg,
                Err(err) => return Err(RegisterError::Stream(err))
            };
            state.on_message(&msg);
            let tymsg = server::IncomingMsg::from_msg(msg);
            if let server::IncomingMsg::Numeric(432, _) = tymsg {
                return Err(RegisterError::InvalidNick);
            }
            if let server::IncomingMsg::Numeric(433, _) = tymsg {
                return Err(RegisterError::NickInUse);
            }
            if let server::IncomingMsg::Numeric(1, _) = tymsg {
                return Ok(state);
            }
        }
        unreachable!();
    }

    pub fn split(self) -> (IrcReader, IrcWriter) {
        let IrcConnector { reader: r, writer: w, .. } = self;
        (r, w)
    }
}
