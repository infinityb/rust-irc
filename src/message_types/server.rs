use std::str;
use std::str::Utf8Error;
use std::cmp::min;

use parse::is_full_prefix;
use parse::IrcMsg;
use irccase::IrcAsciiExt;
use message_types::traits::FromIrcMsg;
use message_types::client;


pub trait IntoIncomingMsg {
    fn into_incoming_msg(self: Self) -> IncomingMsg;
}

fn to_incoming<T: IntoIncomingMsg + FromIrcMsg>(msg: IrcMsg) -> IncomingMsg {
    let intermed: Result<T, IrcMsg> = FromIrcMsg::from_irc_msg(msg);
    match intermed {
        Ok(typed_msg) => typed_msg.into_incoming_msg(),
        Err(msg) => IncomingMsg::Unknown(msg)
    }
}

macro_rules! impl_into_incoming_msg {
    ($id:ident) => {
        impl IntoIncomingMsg for $id {
            fn into_incoming_msg(self) -> IncomingMsg {
                IncomingMsg::$id(self)
            }
        }
    }
}

macro_rules! msg_wrapper_common {
    ($t:ident) => {
        impl $t {
            pub fn into_irc_msg(self) -> IrcMsg {
                let $t(msg) = self;
                msg
            }

            pub fn to_irc_msg<'a>(&'a self) -> &'a IrcMsg {
                let $t(ref msg) = *self;
                msg
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum IncomingMsg {
    Join(Join),
    Kick(Kick),
    Mode(Mode),
    Nick(Nick),
    Notice(Notice),
    Part(Part),
    Ping(Ping),
    Privmsg(Privmsg),
    Quit(Quit),
    Topic(Topic),
    Invite(Invite),
    
    // Others
    Numeric(u16, Numeric),
    Unknown(IrcMsg),
}

impl IncomingMsg {
    pub fn from_msg(msg: IrcMsg) -> IncomingMsg {
        match msg.get_command() {
            "JOIN" => to_incoming::<Join>(msg),
            "KICK" => to_incoming::<Kick>(msg),
            "MODE" => to_incoming::<Mode>(msg),
            "NICK" => to_incoming::<Nick>(msg),
            "NOTICE" => to_incoming::<Notice>(msg),
            "PART" => to_incoming::<Part>(msg),
            "PING" => to_incoming::<Ping>(msg),
            "PRIVMSG" => to_incoming::<Privmsg>(msg),
            "QUIT" => to_incoming::<Quit>(msg),
            "TOPIC" => to_incoming::<Topic>(msg),
            "INVITE" => to_incoming::<Invite>(msg),
            _ => match msg.get_command().parse::<u16>().ok() {
                Some(_) => to_incoming::<Numeric>(msg),
                None => IncomingMsg::Unknown(msg)
            }
        }
    }

    pub fn is_privmsg(&self) -> bool {
        match *self {
            IncomingMsg::Privmsg(_) => true,
            _ => false
        }
    }

    pub fn to_irc_msg<'a>(&'a self) -> &'a IrcMsg {
        match *self {
            IncomingMsg::Join(ref msg) => msg.to_irc_msg(),
            IncomingMsg::Kick(ref msg) => msg.to_irc_msg(),
            IncomingMsg::Mode(ref msg) => msg.to_irc_msg(),
            IncomingMsg::Nick(ref msg) => msg.to_irc_msg(),
            IncomingMsg::Notice(ref msg) => msg.to_irc_msg(),
            IncomingMsg::Part(ref msg) => msg.to_irc_msg(),
            IncomingMsg::Ping(ref msg) => msg.to_irc_msg(),
            IncomingMsg::Privmsg(ref msg) => msg.to_irc_msg(),
            IncomingMsg::Quit(ref msg) => msg.to_irc_msg(),
            IncomingMsg::Topic(ref msg) => msg.to_irc_msg(),
            IncomingMsg::Invite(ref msg) => msg.to_irc_msg(),
            IncomingMsg::Numeric(_, ref msg) => msg.to_irc_msg(),
            IncomingMsg::Unknown(ref msg) => msg,
        }
    }

    pub fn into_irc_msg(self) -> IrcMsg {
        match self {
            IncomingMsg::Join(msg) => msg.into_irc_msg(),
            IncomingMsg::Kick(msg) => msg.into_irc_msg(),
            IncomingMsg::Mode(msg) => msg.into_irc_msg(),
            IncomingMsg::Nick(msg) => msg.into_irc_msg(),
            IncomingMsg::Notice(msg) => msg.into_irc_msg(),
            IncomingMsg::Part(msg) => msg.into_irc_msg(),
            IncomingMsg::Ping(msg) => msg.into_irc_msg(),
            IncomingMsg::Privmsg(msg) => msg.into_irc_msg(),
            IncomingMsg::Quit(msg) => msg.into_irc_msg(),
            IncomingMsg::Topic(msg) => msg.into_irc_msg(),
            IncomingMsg::Invite(msg) => msg.into_irc_msg(),
            IncomingMsg::Numeric(_, msg) => msg.into_irc_msg(),
            IncomingMsg::Unknown(msg) => msg,
        }
    }
}

#[test]
fn test_incoming() {
    let mut msg_raw = Vec::new();
    msg_raw.push_all(b":person!user@host JOIN #foo");
    let msg = IrcMsg::new(msg_raw).unwrap();

    match IncomingMsg::from_msg(msg.clone()) {
        IncomingMsg::Join(ref join) => {
            assert_eq!(join.get_nick(), "person");
            assert_eq!(join.get_channel(), "#foo");
        },
        _ => {
            panic!("Wrong IncomingMsg enum value: expected JOIN, got {:?}",
                IncomingMsg::from_msg(msg)
            )
        }
    }
}


#[derive(Clone, Debug)]
pub struct Join(IrcMsg);
msg_wrapper_common!(Join);
impl_into_incoming_msg!(Join);

impl Join {
    pub fn get_channel(&self) -> &str {
        let Join(ref msg) = *self;
        unsafe { str::from_utf8_unchecked(&msg[0]) }
    }

    pub fn get_nick<'a>(&'a self) -> &'a str {
        let Join(ref msg) = *self;
        let prefix = msg.get_prefix_str();
        &prefix[..prefix.find('!').unwrap()]
    }
}

impl FromIrcMsg for Join {
    fn from_irc_msg(msg: IrcMsg) -> Result<Join, IrcMsg> {
        if !msg.get_command().eq_ignore_irc_case("JOIN") {
            return Err(msg);
        }
        if msg.len() == 0 {
            warn!("Invalid JOIN: Not enough arguments {}", msg.len());
            return Err(msg);
        }
        if !is_full_prefix(msg.get_prefix_str()) {
            warn!("Invalid JOIN: Insufficient prefix `{}`", msg.get_prefix_str());
            return Err(msg);
        }
        if !str::from_utf8(&msg[0]).is_ok() {
            return Err(msg);
        }
        Ok(Join(msg))
    }
}

#[test]
fn test_join_basics() {
    let valid_messages: &[(&[u8], &str, &str)] = &[
        // Standard messages
        (b":person!user@host JOIN #foobar", "person", "#foobar"),
        (b":person!user@host JOIN #foobar\n", "person", "#foobar"),
        (b":person!user@host JOIN #foobar\r\n", "person", "#foobar"),
    ];

    for &(raw, nick, channel) in valid_messages.iter() {
        let mut raw_owned = Vec::with_capacity(raw.len());
        raw_owned.push_all(raw);

        let msg = IrcMsg::new(raw_owned).unwrap();
        let join_msg: Join = FromIrcMsg::from_irc_msg(msg).ok().unwrap();
        assert_eq!(join_msg.get_nick(), nick);
        assert_eq!(join_msg.get_channel(), channel);
    }
}


#[derive(Clone, Debug)]
pub struct Kick(IrcMsg);
impl_into_incoming_msg!(Kick);
msg_wrapper_common!(Kick);

impl Kick {
    pub fn get_channel(&self) -> &str {
        let Kick(ref msg) = *self;
        unsafe { str::from_utf8_unchecked(&msg[0]) }
    }

    /// nick of the user being kicked
    pub fn get_kicked_nick<'a>(&'a self) -> &'a str {
        let Kick(ref msg) = *self;
        unsafe { str::from_utf8_unchecked(&msg[1]) }
    }

    /// nick of the user doing the kicking
    pub fn get_nick<'a>(&'a self) -> &'a str {
        let Kick(ref msg) = *self;
        let prefix = msg.get_prefix_str();
        &prefix[..prefix.find('!').unwrap()]
    }

    pub fn get_body_raw<'a>(&'a self) -> &'a [u8] {
        let Kick(ref msg) = *self;
        &msg[1]
    }
}

impl FromIrcMsg for Kick {
    fn from_irc_msg(msg: IrcMsg) -> Result<Kick, IrcMsg> {
        if !msg.get_command().eq_ignore_irc_case("KICK") {
            return Err(msg);
        }
        if msg.len() < 3 {
            warn!("Invalid KICK: Not enough arguments {}", msg.len());
            return Err(msg);
        }
        if !is_full_prefix(msg.get_prefix_str()) {
            warn!("Invalid KICK: Insufficient prefix `{}`", msg.get_prefix_str());
            return Err(msg);
        }
        // msg[0] is channel, msg[1] is kicked nick
        if !str::from_utf8(&msg[0]).is_ok() || !str::from_utf8(&msg[1]).is_ok() {
            return Err(msg);
        }
        Ok(Kick(msg))
    }
}


#[derive(Clone, Debug)]
pub struct Mode(IrcMsg);
impl_into_incoming_msg!(Mode);
msg_wrapper_common!(Mode);

impl Mode {
    /// Target of the MODE command, channel or user
    pub fn get_target<'a>(&'a self) -> &'a str {
        let Mode(ref msg) = *self;
        unsafe { str::from_utf8_unchecked(&msg[0]) }
    }
}

impl FromIrcMsg for Mode {
    fn from_irc_msg(msg: IrcMsg) -> Result<Mode, IrcMsg> {
        if !msg.get_command().eq_ignore_irc_case("MODE") {
            return Err(msg);
        }
        if msg.len() < 2 {
            warn!("Invalid MODE: Not enough arguments {}", msg.len());
            return Err(msg);
        }
        if !is_full_prefix(msg.get_prefix_str()) {
            warn!("Invalid MODE: Insufficient prefix `{}`", msg.get_prefix_str());
            return Err(msg);
        }
        if !str::from_utf8(&msg[0]).is_ok() {
            return Err(msg);
        }
        Ok(Mode(msg))
    }
}


#[derive(Clone, Debug)]
pub struct Nick(IrcMsg);
impl_into_incoming_msg!(Nick);
msg_wrapper_common!(Nick);

impl Nick {
    /// The previous nick of the user
    pub fn get_nick<'a>(&'a self) -> &'a str {
        let Nick(ref msg) = *self;
        let prefix = msg.get_prefix_str();
        &prefix[..prefix.find('!').unwrap()]
    }

    /// The new nick of the user
    pub fn get_new_nick<'a>(&'a self) -> &'a str {
        let Nick(ref msg) = *self;
        unsafe { str::from_utf8_unchecked(&msg[0]) }
    }
}

impl FromIrcMsg for Nick {
    fn from_irc_msg(msg: IrcMsg) -> Result<Nick, IrcMsg> {
        if !msg.get_command().eq_ignore_irc_case("NICK") {
            return Err(msg);
        }
        if msg.len() < 1 {
            warn!("Invalid NICK: Not enough arguments {}", msg.len());
            return Err(msg);
        }
        if !is_full_prefix(msg.get_prefix_str()) {
            warn!("Invalid NICK: Insufficient prefix `{}`", msg.get_prefix_str());
            return Err(msg);
        }
        // msg[0] is channel, msg[1] is kicked nick
        if !str::from_utf8(&msg[0]).is_ok() || !str::from_utf8(&msg[1]).is_ok() {
            return Err(msg);
        }
        Ok(Nick(msg))
    }
}


#[derive(Clone, Debug)]
pub struct Notice(IrcMsg);
impl_into_incoming_msg!(Notice);
msg_wrapper_common!(Notice);

impl Notice {
    /// Target of the MODE command, channel or user
    pub fn get_target<'a>(&'a self) -> &'a str {
        let Notice(ref msg) = *self;
        unsafe { str::from_utf8_unchecked(&msg[0]) }
    }
}

impl FromIrcMsg for Notice {
    fn from_irc_msg(msg: IrcMsg) -> Result<Notice, IrcMsg> {
        if !msg.get_command().eq_ignore_irc_case("MODE") {
            return Err(msg);
        }
        if msg.len() < 2 {
            warn!("Invalid MODE: Not enough arguments {}", msg.len());
            return Err(msg);
        }
        if !is_full_prefix(msg.get_prefix_str()) {
            warn!("Invalid MODE: Insufficient prefix `{}`", msg.get_prefix_str());
            return Err(msg);
        }
        if !str::from_utf8(&msg[0]).is_ok() {
            return Err(msg);
        }
        Ok(Notice(msg))
    }
}


#[derive(Clone, Debug)]
pub struct Part(IrcMsg);
impl_into_incoming_msg!(Part);
msg_wrapper_common!(Part);

impl Part {
    pub fn get_nick<'a>(&'a self) -> &'a str {
        let Part(ref msg) = *self;
        let prefix = msg.get_prefix_str();
        &prefix[..prefix.find('!').unwrap()]
    }

    pub fn get_channel<'a>(&'a self) -> &'a str {
        let Part(ref msg) = *self;
        unsafe { str::from_utf8_unchecked(&msg[0]) }
    }
}

impl FromIrcMsg for Part {
    fn from_irc_msg(msg: IrcMsg) -> Result<Part, IrcMsg> {
        if !msg.get_command().eq_ignore_irc_case("PART") {
            return Err(msg);
        }
        if msg.len() < 1 {
            warn!("Invalid PART: Not enough arguments {}", msg.len());
            return Err(msg);
        }
        if !is_full_prefix(msg.get_prefix_str()) {
            warn!("Invalid PART: Insufficient prefix `{}`", msg.get_prefix_str());
            return Err(msg);
        }
        if !str::from_utf8(&msg[0]).is_ok() {
            return Err(msg);
        }
        Ok(Part(msg))
    }
}

#[derive(Clone, Debug)]
pub struct Ping(IrcMsg);
impl_into_incoming_msg!(Ping);
msg_wrapper_common!(Ping);

impl Ping {
    pub fn get_server1(&self) -> &str {
        let Ping(ref msg) = *self;
        unsafe { str::from_utf8_unchecked(&msg[0]) }
    }

    pub fn get_server2(&self) -> Option<&str> {
        let Ping(ref msg) = *self;
        if msg.len() > 1 {
            unsafe { Some(str::from_utf8_unchecked(&msg[1])) }
        } else {
            None
        }
    }

    pub fn get_response(&self) -> Result<client::Pong, ()> {
        let Ping(ref msg) = *self;
        match msg.len() {
            1 => Ok(client::Pong::new(self.get_server1())),
            _ => Err(())
        }
    }
}

impl FromIrcMsg for Ping {
    fn from_irc_msg(msg: IrcMsg) -> Result<Ping, IrcMsg> {
        if !msg.get_command().eq_ignore_irc_case("PING") {
            return Err(msg);
        }
        if msg.len() < 1 {
            warn!("Invalid PING: Not enough arguments {}", msg.len());
            return Err(msg);
        }
        for idx in 0..min(2, msg.len()) {
            if !str::from_utf8(&msg[idx]).is_ok() {
                return Err(msg);
            }
        }
        Ok(Ping(msg))
    }
}

#[test]
fn test_ping_basics() {
    let mut raw_owned = Vec::new();
    raw_owned.push_all(b":person!user@host NOTPING server1 :server2");
    let msg = IrcMsg::new(raw_owned).unwrap();
    let ping: Result<Ping, _> = FromIrcMsg::from_irc_msg(msg);
    assert!(ping.is_err());
}


#[derive(Clone, Debug)]
pub struct Privmsg(IrcMsg);
impl_into_incoming_msg!(Privmsg);
msg_wrapper_common!(Privmsg);

impl Privmsg {
    pub fn get_target(&self) -> &str {
        let Privmsg(ref msg) = *self;
        unsafe { str::from_utf8_unchecked(&msg[0]) }
    }

    pub fn get_nick<'a>(&'a self) -> &'a str {
        let Privmsg(ref msg) = *self;
        let prefix = msg.get_prefix_str();
        &prefix[..prefix.find('!').unwrap()]
    }

    pub fn get_body_unicode<'a>(&'a self) -> Result<&'a str, Utf8Error> {
        ::std::str::from_utf8(self.get_body_raw())
    }

    pub fn get_body_raw<'a>(&'a self) -> &'a [u8] {
        let Privmsg(ref msg) = *self;
        &msg[1]
    }
}

impl FromIrcMsg for Privmsg {
    fn from_irc_msg(msg: IrcMsg) -> Result<Privmsg, IrcMsg> {
        if !msg.get_command().eq_ignore_irc_case("PRIVMSG") {
            return Err(msg);
        }
        if msg.len() < 2 {
            warn!("Invalid PRIVMSG: Not enough arguments {}", msg.len());
            return Err(msg);
        }
        if !is_full_prefix(msg.get_prefix_str()) {
            warn!("Invalid PRIVMSG: Insufficient prefix `{}`", msg.get_prefix_str());
            return Err(msg);
        }
        if !str::from_utf8(&msg[0]).is_ok() {
            return Err(msg);
        }
        Ok(Privmsg(msg))
    }
}

#[test]
fn test_privmsg_basics() {
    let mut raw_owned = Vec::new();
    raw_owned.push_all(b":person!user@host NOTPRIVMSG server1 :server2");
    let privmsg: Result<Privmsg, _> = FromIrcMsg::from_irc_msg(IrcMsg::new(raw_owned).unwrap());
    assert!(privmsg.is_err());

    let valid_messages: &[(&[u8], &str, &str, &[u8])] = &[
        // Standard messages
        (b":person!user@host PRIVMSG #foobar :foobarbaz",
            "person", "#foobar", b"foobarbaz"),
        (b":person!user@host PRIVMSG #foobar :foobar\r\n",
            "person", "#foobar", b"foobar"),
        (b":person!user@host PRIVMSG #foobar :foobar\r\n",
            "person", "#foobar", b"foobar"),

        // Invalid UTF-8 in message, but we ignore messages
        (b":person!user@host PRIVMSG #foobar :\xe3\x81",
            "person", "#foobar", b"\xe3\x81"),
    ];

    for &(raw, nick, channel, body) in valid_messages.iter() {
        let mut raw_owned = Vec::with_capacity(raw.len());
        raw_owned.push_all(raw);

        let msg = IrcMsg::new(raw_owned).unwrap();
        let priv_msg: Privmsg = FromIrcMsg::from_irc_msg(msg).ok().unwrap();
        assert_eq!(priv_msg.get_nick(), nick);
        assert_eq!(priv_msg.get_target(), channel);
        assert_eq!(priv_msg.get_body_raw(), body);
    }
}

#[derive(Clone, Debug)]
pub struct Quit(IrcMsg);
impl_into_incoming_msg!(Quit);
msg_wrapper_common!(Quit);

impl Quit {
    pub fn get_nick<'a>(&'a self) -> &'a str {
        let Quit(ref msg) = *self;
        let prefix = msg.get_prefix_str();
        &prefix[..prefix.find('!').unwrap()]
    }

    pub fn get_channel(&self) -> &str {
        let Quit(ref msg) = *self;
        unsafe { str::from_utf8_unchecked(&msg[0]) }
    }

    pub fn get_body_raw<'a>(&'a self) -> &'a [u8] {
        let Quit(ref msg) = *self;
        &msg[1]
    }
}

impl FromIrcMsg for Quit {
    fn from_irc_msg(msg: IrcMsg) -> Result<Quit, IrcMsg> {
        if !msg.get_command().eq_ignore_irc_case("QUIT") {
            return Err(msg);
        }
        if msg.len() < 1 {
            warn!("Invalid QUIT: Not enough arguments {}", msg.len());
            return Err(msg);
        }
        if !str::from_utf8(&msg[0]).is_ok() {
            return Err(msg);
        }
        Ok(Quit(msg))
    }
}

#[test]
fn test_quit_basics() {
    let msg = IrcMsg::new(b":person!user@host NOTQUIT :server2".to_vec()).unwrap();
    let ping: Result<Ping, _> = FromIrcMsg::from_irc_msg(msg);
    assert!(ping.is_err());
}

#[derive(Clone, Debug)]
pub struct Topic(IrcMsg);
impl_into_incoming_msg!(Topic);
msg_wrapper_common!(Topic);

impl Topic {
    pub fn get_channel(&self) -> &str {
        let Topic(ref msg) = *self;
        unsafe { str::from_utf8_unchecked(&msg[0]) }
    }

    pub fn get_nick<'a>(&'a self) -> &'a str {
        let Topic(ref msg) = *self;
        let prefix = msg.get_prefix_str();
        &prefix[..prefix.find('!').unwrap()]
    }

    pub fn get_body_raw<'a>(&'a self) -> &'a [u8] {
        let Topic(ref msg) = *self;
        &msg[1]
    }
}

impl FromIrcMsg for Topic {
    fn from_irc_msg(msg: IrcMsg) -> Result<Topic, IrcMsg> {
        if !msg.get_command().eq_ignore_irc_case("TOPIC") {
            return Err(msg);
        }
        if msg.len() < 2 {
            warn!("Invalid TOPIC: Not enough arguments {}", msg.len());
            return Err(msg);
        }
        if !is_full_prefix(msg.get_prefix_str()) {
            warn!("Invalid TOPIC: Insufficient prefix `{}`", msg.get_prefix_str());
            return Err(msg);
        }
        if !str::from_utf8(&msg[0]).is_ok() {
            return Err(msg);
        }
        Ok(Topic(msg))
    }
}

#[derive(Clone, Debug)]
pub struct Invite(IrcMsg);
impl_into_incoming_msg!(Invite);
msg_wrapper_common!(Invite);

impl Invite {
    pub fn get_target(&self) -> &str {
        let Invite(ref msg) = *self;
        unsafe { str::from_utf8_unchecked(&msg[1]) }
    }
}

impl FromIrcMsg for Invite {
    fn from_irc_msg(msg: IrcMsg) -> Result<Invite, IrcMsg> {
        if !msg.get_command().eq_ignore_irc_case("INVITE") {
            return Err(msg);
        }
        if msg.len() < 2 {
            warn!("Invalid TOPIC: Not enough arguments {}", msg.len());
            return Err(msg);
        }
        if !is_full_prefix(msg.get_prefix_str()) {
            warn!("Invalid TOPIC: Insufficient prefix `{}`", msg.get_prefix_str());
            return Err(msg);
        }
        if !str::from_utf8(&msg[0]).is_ok() {
            return Err(msg);
        }
        Ok(Invite(msg))
    }
}

#[derive(Clone, Debug)]
pub struct Numeric(IrcMsg);
msg_wrapper_common!(Numeric);

impl Numeric {
    pub fn get_code(&self) -> u16 {
        let Numeric(ref msg) = *self;
        msg.get_command().parse::<u16>().unwrap()
    }
}

impl IntoIncomingMsg for Numeric {
    fn into_incoming_msg(self) -> IncomingMsg {
        let numeric_num = self.get_code();
        IncomingMsg::Numeric(numeric_num, self) 
    }
}

impl FromIrcMsg for Numeric {
    fn from_irc_msg(msg: IrcMsg) -> Result<Numeric, IrcMsg>  {
        match msg.get_command().parse::<u16>().ok() {
            Some(_) => Ok(Numeric(msg)),
            None => Err(msg)
        }
    }
}
