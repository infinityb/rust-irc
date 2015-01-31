use std::old_io::IoResult;

use parse::IrcMsg;

#[derive(Debug)]
pub enum SessionRecord {
    Content(IrcMsg),
    Expectation(String),
    Comment(String),
    Unknown(String),
}

pub fn decode_line(line_res: IoResult<String>) -> Option<SessionRecord> {
    match line_res {
        Ok(ok) => Some(decode_line2(ok)),
        Err(err) => panic!("error reading: {:?}", err)
    }
}

pub fn decode_line2(line: String) -> SessionRecord {
    let trim_these: &[_] = &['\r', '\n'];
    let slice = line.as_slice().trim_right_matches(trim_these);

    match (&slice[0..3], (&slice[3..]).to_string()) {
        (">> ", rest) => match IrcMsg::new(rest.into_bytes()) {
            Ok(irc_msg) => SessionRecord::Content(irc_msg),
            Err(err) => panic!("Error decoding IrcMsg: {:?}", err)
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
