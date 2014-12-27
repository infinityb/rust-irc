use parse::IrcMsg;
use watchers::join::JoinResult;
use watchers::who::WhoResult;


/// An event, which is usually generated by reading a line from the server.
#[deriving(Show)]
pub enum IrcEvent {
	/// An IRC message from the server
	IrcMsg(IrcMsg),
    // /// An IRC message from the server
    // Message(IrcMessage),
    /// The bundled result of a JOIN command
    JoinBundle(JoinResult),
    /// The bundled result of a WHO command
    WhoBundle(WhoResult)
}
