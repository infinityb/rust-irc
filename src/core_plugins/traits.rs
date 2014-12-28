use parse::IrcMsg;

/// Simple responder trait
pub trait MessageResponder {
    fn on_irc_msg(&mut self, message: &IrcMsg) -> Vec<IrcMsg>;

    fn finished(&self) -> bool { false }
}
