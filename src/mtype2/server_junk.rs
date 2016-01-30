impl_irc_msg_subtype!(Join);

impl Join {
    pub fn from_irc_msg(msg: &IrcMsg) -> Result<&Join, ()> {
        unimplemented!(); // must validate

        Ok(FromIrcMsgTransmute::transmute(msg))
    }

    pub fn to_irc_msg(&self) -> &IrcMsg {
        &self.inner
    }

    pub fn get_channel(&self) -> &[u8] {
        let buf = self.inner.as_bytes();
        let (_prefix, rest) = parse_helpers::split_prefix(buf);
        let (_command, rest) = parse_helpers::split_command(rest);
        let (channel, _rest) = parse_helpers::split_arg(rest);

        channel
    }

    pub fn get_nick(&self) -> &str {
        let buf = self.inner.as_bytes();
        let (prefix, _rest) = parse_helpers::split_prefix(buf);
        let (nick, _, _) = parse_helpers::parse_prefix(prefix).unwrap();
        unsafe { ::std::str::from_utf8_unchecked(nick) }
    }
}

// struct JoinBuf {
//     inner: IrcMsgBuf
// }

impl_irc_msg_subtype!(Kick);

impl Kick {
    pub fn from_irc_msg(msg: &IrcMsg) -> Result<&Kick, ()> {
        unimplemented!(); // must validate

        Ok(FromIrcMsgTransmute::transmute(msg))
    }

    pub fn to_irc_msg(&self) -> &IrcMsg {
        &self.inner
    }
}

// struct KickBuf {
//     inner: IrcMsgBuf
// }

impl_irc_msg_subtype!(Mode);

impl Mode {
    pub fn from_irc_msg(msg: &IrcMsg) -> Result<&Mode, ()> {
        unimplemented!(); // must validate

        Ok(FromIrcMsgTransmute::transmute(msg))
    }

    pub fn to_irc_msg(&self) -> &IrcMsg {
        &self.inner
    }
}

// struct ModeBuf {
//     inner: IrcMsgBuf
// }

impl_irc_msg_subtype!(Nick);

impl Nick {
    pub fn from_irc_msg(msg: &IrcMsg) -> Result<&Nick, ()> {
        unimplemented!(); // must validate

        Ok(FromIrcMsgTransmute::transmute(msg))
    }

    pub fn to_irc_msg(&self) -> &IrcMsg {
        &self.inner
    }
}

// struct NickBuf {
//     inner: IrcMsgBuf
// }
