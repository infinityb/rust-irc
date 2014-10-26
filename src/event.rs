use message::IrcMessage;

use watchers::join::JoinResult;
use watchers::who::WhoResult;


pub enum IrcEvent {
    IrcEventMessage(IrcMessage),
    IrcEventJoinBundle(JoinResult),
    IrcEventWhoBundle(WhoResult)
}
