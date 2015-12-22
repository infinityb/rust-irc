use std::io::{self, BufReader, Read, Write, BufRead};
// use std::net::TcpStream;

use parse::{IrcMsg, ParseError};
use message_types::{client, server};
use state::State;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Parse(ParseError),
    Empty,
}

pub struct IrcReaderIter<'a, R: IrcRead + 'a> {
    reader: &'a mut R,
}

impl<'a, R> IrcReaderIter<'a, R> where R: IrcRead + 'a {
    pub fn new(reader: &'a mut R) -> IrcReaderIter<'a, R> {
        IrcReaderIter { reader: reader }
    }
}

impl<'a, R> Iterator for IrcReaderIter<'a, R> where R: IrcRead + 'a {
    type Item = Result<IrcMsg, Error>;

    fn next(&mut self) -> Option<Result<IrcMsg, Error>> {
        match self.reader.get_irc_msg() {
            Err(Error::Empty) => None,
            r @ _ => Some(r),
        }
    }
}

pub trait IrcRead {
    fn get_irc_msg(&mut self) -> Result<IrcMsg, Error>;
}

pub struct IrcReader<R>(BufReader<R>);

impl<R> IrcReader<R> where R: Read {
    pub fn new(reader: R) -> IrcReader<R> {
        IrcReader(BufReader::new(reader))
    }
}

impl<R> IrcRead for IrcReader<R> where R: Read {
    fn get_irc_msg(&mut self) -> Result<IrcMsg, Error> {
        let IrcReader(ref mut reader) = *self;

        let mut buf: Vec<u8> = Vec::new();
        if let Err(err) = reader.read_until(b'\n', &mut buf) {
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

}

pub trait IrcWrite {
    fn write_irc_msg(&mut self, msg: &IrcMsg) -> io::Result<()>;
}

pub struct IrcWriter<W>(W);

impl<W> IrcWriter<W> where W: Write {
    pub fn new(writer: W) -> IrcWriter<W> {
        IrcWriter(writer)
    }
}

impl<W> IrcWrite for IrcWriter<W> where W: Write {
    fn write_irc_msg(&mut self, msg: &IrcMsg) -> io::Result<()> {
        let IrcWriter(ref mut writer) = *self;
        let buf = msg.as_bytes();
        assert_eq!(try!(writer.write(buf)), buf.len());
        assert_eq!(try!(writer.write(b"\r\n")), 2);
        try!(writer.flush());
        Ok(())
    }
}

// pub struct IrcStream<S>(S) where S: Read + Write;

// impl<S> IrcStream<S> where S: Read + Write {
//     pub fn new(stream: R) -> IrcStream<R> {
//         IrcStream(BufReader::new(reader))
//     }
// }

// impl<R> IrcRead for IrcStream<R> where R: Read {
//     fn get_irc_msg(&mut self) -> Result<IrcMsg, Error> {
//         let IrcStream(ref mut reader) = *self;

//         let mut buf: Vec<u8> = Vec::new();
//         if let Err(err) = reader.read_until(b'\n', &mut buf) {
//             return Err(Error::Io(err));
//         }
//         if buf.len() == 0 {
//             return Err(Error::Empty);
//         }
//         match IrcMsg::new(buf) {
//             Ok(msg) => Ok(msg),
//             Err(err) => Err(Error::Parse(err))
//         }
//     }
// }


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
            &self.user,
            &format!("{}", mode), "*",
            &self.realname,
        )
    }

    fn get_nick(&self) -> client::Nick {
        client::Nick::new(&self.nick)
    }
}

pub struct IrcConnector<R, W> {
    reader: R,
    writer: W,
    user_sent: bool,
}

impl<R, W> IrcConnector<R, W> where R: IrcRead, W: IrcWrite {
    pub fn from_pair(reader: R, writer: W) -> IrcConnector<R, W> {
        IrcConnector {
            reader: reader,
            writer: writer,
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

        for msg in IrcReaderIter::new(&mut self.reader) {
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

    pub fn split(self) -> (R, W) {
        let IrcConnector { reader: r, writer: w, .. } = self;
        (r, w)
    }
}

impl<T> IrcRead for Box<T> where T: IrcRead {
    fn get_irc_msg(&mut self) -> Result<IrcMsg, Error> {
        (**self).get_irc_msg()
    }
}

impl<T> IrcWrite for Box<T> where T: IrcWrite {
    fn write_irc_msg(&mut self, msg: &IrcMsg) -> io::Result<()> {
        (**self).write_irc_msg(msg)
    }
}

impl IrcRead for Box<IrcRead> {
    fn get_irc_msg(&mut self) -> Result<IrcMsg, Error> {
        (**self).get_irc_msg()
    }
}

impl IrcWrite for Box<IrcWrite> {
    fn write_irc_msg(&mut self, msg: &IrcMsg) -> io::Result<()> {
        (**self).write_irc_msg(msg)
    }
}
