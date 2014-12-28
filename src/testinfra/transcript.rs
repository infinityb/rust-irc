use std::io::IoResult;

use parse::IrcMsg;

#[deriving(Show)]
pub enum SessionRecord {
    Content(IrcMsg),
    Expectation(String),
    Comment(String),
    Unknown(String),
}

pub fn decode_line(line_res: IoResult<String>) -> Option<SessionRecord> {
    match line_res {
        Ok(ok) => Some(decode_line2(ok)),
        Err(err) => panic!("error reading: {}", err)
    }
}

pub fn decode_line2(line: String) -> SessionRecord {
    let trim_these: &[_] = &['\r', '\n'];
    let slice = line.as_slice().trim_right_chars(trim_these);

    match (slice[0..3], slice[3..].to_string()) {
        (">> ", rest) => match IrcMsg::new(rest.into_bytes()) {
            Ok(irc_msg) => SessionRecord::Content(irc_msg),
            Err(err) => panic!("Error decoding IrcMsg: {}", err)
        },
        ("<< ", rest) => SessionRecord::Expectation(rest),
        ("## ", rest) => SessionRecord::Comment(rest),
        _ => panic!("Bad line in transcript"),
    }
}

pub fn marker_match(rec: &SessionRecord, target: &str) -> bool {
    match *rec {
        SessionRecord::Comment(ref comm) => comm.as_slice() == target,
        _ => false
    }
}

struct Transcript<'a> {
    current_expection: Option<Vec<u8>>,
    buffer: &'a mut (Buffer+'a),
}

impl<'a> Transcript<'a> {
    pub fn read_record(&mut self) -> IoResult<SessionRecord> {
        match self.buffer.read_line() {
            Ok(line) => Ok(decode_line2(line)),
            Err(err) => Err(err),
        }
    }

    // FIXME: impl Writer?
    pub fn write(&self, _buf: &[u8]) -> Result<(), ()> {
        Err(())
    }
}