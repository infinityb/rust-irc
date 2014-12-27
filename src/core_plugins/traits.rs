use message::IrcMessage;
use parse::IrcMsg;


/// Simple responder trait
pub trait MessageResponder {
    fn on_message(&mut self, message: &IrcMessage) -> Vec<IrcMsg> {
        self.on_irc_msg(message.as_irc_msg())
    }

    fn on_irc_msg(&mut self, message: &IrcMsg) -> Vec<IrcMsg>;

    fn finished(&self) -> bool { false }
}
