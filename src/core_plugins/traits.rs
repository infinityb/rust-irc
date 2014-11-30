use message::IrcMessage;

/// Simple responder trait
pub trait MessageResponder {
    fn on_message(&mut self, message: &IrcMessage) -> Vec<String>;

    fn finished(&self) -> bool { false }
}
