use message::IrcMessage;
use parse::IrcMsg;


/// Simple responder trait
pub trait MessageResponder {
    fn on_message(&mut self, message: &IrcMessage) -> Vec<IrcMsg>;

    fn finished(&self) -> bool { false }
}
