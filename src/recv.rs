use std::collections::VecDeque;
use std::convert::From;
use ::parse::{IrcMsg, ParseError};

pub enum PushError {
	Full,
}

pub enum RecvError {
	MoreData,
	Parse(ParseError),
}

impl From<ParseError> for RecvError {
	fn from(e: ParseError) -> RecvError {
		RecvError::Parse(e)
	}
}

pub struct IrcMsgBuffer {
	capacity: usize,
	data: VecDeque<u8>,
}

impl IrcMsgBuffer {
	// Later, we'll use a doubly-mapped circular buffer
	pub fn new(capacity: usize) -> IrcMsgBuffer {
		IrcMsgBuffer {
			capacity: capacity,
			data: VecDeque::with_capacity(capacity),
		}
	}

	pub fn push(&mut self, buffer: &[u8]) -> Result<(), PushError> {
		if self.data.len() + buffer.len() > self.capacity {
			return Err(PushError::Full);
		}
		self.data.extend(buffer.iter().cloned());
		Ok(())
	}

	pub fn recv(&mut self) -> Result<IrcMsg, RecvError> {
		let mut bytes = 0;
		let mut found_eol = false;

		for &byte in self.data.iter() {
			if byte == b'\n' {
				found_eol = true;
				break;
			}
			bytes += 1;
		}

		if !found_eol {
			return Err(RecvError::MoreData);
		}

		let buffer: Vec<u8> = self.data.iter().take(bytes).cloned().collect();
		for _ in 0..bytes {
			self.data.pop_front();
		}

		Ok(try!(IrcMsg::new(buffer)))
	}
}