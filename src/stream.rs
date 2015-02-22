use std::io::{self, Write, BufRead};
use parse::{IrcMsg, ParseError};

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
	reader: Box<BufRead+'static>,
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
	writer: Box<Write+'static>
}

impl IrcWriter {
	pub fn write_irc_msg(&mut self, msg: &IrcMsg) -> io::Result<()> {
		let buf = msg.as_bytes();
		assert_eq!(try!(self.writer.write(buf)), buf.len());
		try!(self.writer.flush());
		Ok(())
	}
}

pub enum RegisterError {
	InvalidUser,
	InvalidNick,
	Io(io::Error),
}

pub struct IrcConnector {
	reader: IrcReader,
	writer: IrcWriter,
}

impl IrcConnector {
	pub fn from_pair(reader: Box<BufRead+'static>, writer: Box<Write+'static>) -> IrcConnector {
		IrcConnector {
			reader: IrcReader { reader: reader },
			writer: IrcWriter { writer: writer },
		}
	}

	pub fn register() -> Result<(), RegisterError> {
		unimplemented!();
	}

	pub fn split(self) -> (IrcReader, IrcWriter) {
		let IrcConnector { reader: r, writer: w } = self;
		(r, w)
	}
}