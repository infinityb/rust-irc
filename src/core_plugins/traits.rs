use message::IrcMessage;

/// Simple responder trait
pub trait MessageResponder {
    fn on_message(&mut self, message: &IrcMessage) -> Vec<IrcMessage>;

    fn finished(&self) -> bool { false }
}
