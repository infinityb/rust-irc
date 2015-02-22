#![warn(dead_code)]
#![deny(unused_variables, unused_mut)]
use std::cmp::max;
use std::default::Default;
use std::collections::{
    hash_map,
    HashMap,
    HashSet,
};
use std::borrow::IntoCow;
use std::ops::Deref;

use message_types::server;
use parse::{IrcMsg, IrcMsgPrefix};
use watchers::{
    JoinSuccess,
    WhoRecord,
    WhoSuccess,
};
use event::IrcEvent;

use self::irc_identifier::IrcIdentifier;
pub use self::MessageEndpoint::{
    KnownUser,
    KnownChannel,
    AnonymousUser,
};

macro_rules! deref_opt_or_return(
    ($inp:expr, $erp:expr, $fr:expr) => (
        match $inp {
            Some(x) => *x,
            _ => {
                println!($erp);
                return $fr;
            }
        }
    );
);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MessageEndpoint {
    KnownUser(UserId),
    KnownChannel(ChannelId),
    Server(String),
    AnonymousUser,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct UserId(u64);


mod irc_identifier {
    use irccase::IrcAsciiExt;

    fn channel_deprefix(target: &str) -> &str {
        match target.find('#') {
            Some(idx) => &target[idx..],
            None => target
        }
    }

    #[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct IrcIdentifier(String);

    impl IrcIdentifier {
        pub fn from_str(mut val: &str) -> IrcIdentifier {
            val = channel_deprefix(val);
            IrcIdentifier(val.to_irc_lower())
        }

        pub fn as_slice(&self) -> &str {
            let IrcIdentifier(ref string) = *self;
            string.as_slice()
        }
    }
}

trait Diff<DiffType> {
    fn diff(&self, other: &Self) -> DiffType;
}

trait Patch<DiffType> {
    fn patch(&self, diff: &DiffType) -> Self;
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct User {
    id: UserId,
    prefix: IrcMsgPrefix<'static>,
    channels: HashSet<ChannelId>
}

impl User {
    fn from_who(id: UserId, who: &WhoRecord) -> User {
        User {
            id: id,
            prefix: who.get_prefix().to_owned(),
            channels: Default::default(),
        }
    }

    fn from_info(user_info: &UserInfo) -> User {
        User {
            id: user_info.id,
            prefix: user_info.prefix.clone(),
            channels: Default::default(),
        }
    }

    pub fn get_nick(&self) -> &str {
        let prefix = self.prefix.as_slice();
        match prefix.find('!') {
            Some(idx) => &prefix[0..idx],
            None => prefix
        }
    }

    fn set_nick(&mut self, nick: &str) {
        self.prefix = self.prefix.with_nick(nick).expect("Need nicked prefix");
    }
}

impl Diff<Vec<UserDiffCmd>> for User {
    fn diff(&self, other: &User) -> Vec<UserDiffCmd> {
        let mut cmds = Vec::new();
        if self.prefix != other.prefix {
            cmds.push(UserDiffCmd::ChangePrefix(other.prefix.as_slice().to_string()));
        }
        for &added_channel in other.channels.difference(&self.channels) {
            cmds.push(UserDiffCmd::AddChannel(added_channel));
        }
        for &removed_channel in self.channels.difference(&other.channels) {
            cmds.push(UserDiffCmd::RemoveChannel(removed_channel));
        }
        cmds
    }
}

impl Patch<Vec<UserDiffCmd>> for User {
    fn patch(&self, diff: &Vec<UserDiffCmd>) -> User {
        let mut other = self.clone();
        for cmd in diff.iter() {
            match *cmd {
                UserDiffCmd::ChangePrefix(ref prefix_str) => {
                    other.prefix = IrcMsgPrefix::new(prefix_str.clone().into_cow());
                },
                UserDiffCmd::AddChannel(chan_id) => {
                    other.channels.insert(chan_id);
                },
                UserDiffCmd::RemoveChannel(chan_id) => {
                    other.channels.remove(&chan_id);
                }
            }
        }
        other
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChannelId(u64);


#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Channel {
    id: ChannelId,
    name: String,
    topic: String,
    users: HashSet<UserId>
}

impl Channel {
    fn from_info(chan_info: &ChannelInfo) -> Channel {
        Channel {
            id: chan_info.id,
            name: chan_info.name.clone(),
            topic: chan_info.topic.clone(),
            users: Default::default(),
        }
    }

    fn set_topic(&mut self, topic: &str) {
        self.topic.clear();
        self.topic.push_str(topic);
    }
}

impl Diff<Vec<ChannelDiffCmd>> for Channel {
    fn diff(&self, other: &Channel) -> Vec<ChannelDiffCmd> {
        let mut cmds = Vec::new();
        if self.topic != other.topic {
            cmds.push(ChannelDiffCmd::ChangeTopic(other.topic.clone()));
        }
        for &added_user in other.users.difference(&self.users) {
            cmds.push(ChannelDiffCmd::AddUser(added_user));
        }
        for &removed_user in self.users.difference(&other.users) {
            cmds.push(ChannelDiffCmd::RemoveUser(removed_user));
        }
        assert_eq!(self.clone().patch(&cmds), *other);
        cmds
    }
}

impl Patch<Vec<ChannelDiffCmd>> for Channel {
    fn patch(&self, diff: &Vec<ChannelDiffCmd>) -> Channel {
        let mut other = self.clone();
        for cmd in diff.iter() {
            match *cmd {
                ChannelDiffCmd::ChangeTopic(ref topic) => {
                    other.topic = topic.clone();
                },
                ChannelDiffCmd::AddUser(user_id) => {
                    other.users.insert(user_id);
                },
                ChannelDiffCmd::RemoveUser(user_id) => {
                    other.users.remove(&user_id);
                }
            }
        }
        other
    }
}

#[derive(Debug)]
struct UserInfo {
    id: UserId,
    prefix: IrcMsgPrefix<'static>,
}

impl UserInfo {
    fn from_internal(user: &User) -> UserInfo {
        UserInfo {
            id: user.id,
            prefix: user.prefix.to_owned(),
        }
    }

    fn get_nick(&self) -> &str {
        let prefix = self.prefix.as_slice();
        match prefix.find('!') {
            Some(idx) => &prefix[0..idx],
            None => prefix
        }
    }
}

#[derive(Debug)]
struct ChannelInfo {
    id: ChannelId,
    name: String,
    topic: String
}

impl ChannelInfo {
    fn from_internal(chan: &Channel) -> ChannelInfo {
        ChannelInfo {
            id: chan.id,
            name: chan.name.clone(),
            topic: chan.topic.clone()
        }
    }

    fn from_join(id: ChannelId, join: &JoinSuccess) -> ChannelInfo {
        let topic = String::from_utf8(match join.topic {
            Some(ref topic) => topic.text.clone(),
            None => Vec::new()
        }).ok().expect("non-string");

        let channel_name = ::std::str::from_utf8(join.channel.as_slice())
            .ok().expect("bad chan").to_string();

        ChannelInfo {
            id: id,
            name: channel_name,
            topic: topic
        }
    }
}

#[derive(Debug)]
pub enum ChannelDiffCmd {
    ChangeTopic(String),
    AddUser(UserId),
    RemoveUser(UserId),
}

#[derive(Debug)]
pub enum UserDiffCmd {
    ChangePrefix(String),
    AddChannel(ChannelId),
    RemoveChannel(ChannelId),
}


#[derive(Debug)]
pub enum StateCommand {
    CreateUser(UserInfo),
    UpdateUser(UserId, Vec<UserDiffCmd>),
    RemoveUser(UserId),

    CreateChannel(ChannelInfo),
    UpdateChannel(ChannelId, Vec<ChannelDiffCmd>),
    RemoveChannel(ChannelId),

    UpdateSelfNick(String),
    SetGeneration(u64),
}

#[derive(Debug)]
pub struct StateDiff {
    from_generation: u64,
    to_generation: u64,
    commands: Vec<StateCommand>
}

pub struct FrozenState(State);

impl Deref for FrozenState {
    type Target = State;

    fn deref<'a>(&'a self) -> &'a State {
        let FrozenState(ref state) = *self;
        state
    }
}

unsafe impl Send for FrozenState {}
unsafe impl Sync for FrozenState {}

#[derive(Debug, Clone)]
pub struct State {
    // Can this be made diffable by using sorted `users`, `channels`,
    // `users[].channels` and `channels[].users`?  TreeSet.
    user_seq: u64,
    channel_seq: u64,

    self_nick: String,
    self_id: UserId,

    user_map: HashMap<IrcIdentifier, UserId>,
    users: HashMap<UserId, User>,

    channel_map: HashMap<IrcIdentifier, ChannelId>,
    channels: HashMap<ChannelId, Channel>,

    generation: u64,
}

impl State {
    pub fn new() -> State {
        State {
            user_seq: 1,
            channel_seq: 0,
            self_nick: String::new(),
            user_map: Default::default(),
            users: Default::default(),
            self_id: UserId(0),
            channel_map: Default::default(),
            channels: Default::default(),
            generation: 0,
        }
    }

    fn on_other_part(&mut self, part: &server::Part) {
        println!("part.channel = {:?}, part.nick = {:?}",
            part.get_channel(), part.get_nick());

        let channel_name = IrcIdentifier::from_str(part.get_channel());
        let user_nick = IrcIdentifier::from_str(part.get_nick());

        let chan_id = deref_opt_or_return!(self.channel_map.get(&channel_name),
            "Got channel without knowing about it.", ());

        let user_id = deref_opt_or_return!(self.user_map.get(&user_nick),
            "Got user without knowing about it.", ());

        self.validate_state_internal_panic();
        self.unlink_user_channel(user_id, chan_id);
        self.validate_state_internal_panic();
    }

    fn on_self_part(&mut self, part: &server::Part) {
        assert!(self.remove_channel_by_name(part.get_channel()).is_some());
    }

    fn on_other_quit(&mut self, quit: &server::Quit) {
        assert!(self.remove_user_by_nick(quit.get_nick()).is_some());
    }

    fn on_other_join(&mut self, join: &server::Join) {
        let channel_name = IrcIdentifier::from_str(join.get_channel());
        let user_nick = IrcIdentifier::from_str(join.get_nick());

        let chan_id = match self.channel_map.get(&channel_name) {
            Some(chan_id) => *chan_id,
            None => panic!("Got message for channel {:?} without knowing about it.", channel_name)
        };

        let (is_create, user_id) = match self.user_map.get(&user_nick) {
            Some(user_id) => {
                (false, *user_id)
            },
            None => {
                let new_user_id = UserId(self.user_seq);
                self.user_seq += 1;
                (true, new_user_id)
            }
        };
        if is_create {
            let user = User {
                id: user_id,
                prefix: join.to_irc_msg().get_prefix().to_owned(),
                channels: HashSet::new(),
            };
            self.users.insert(user_id, user);
            self.user_map.insert(user_nick, user_id);
        }
        self.users.get_mut(&user_id).expect("user not found").channels.insert(chan_id);

        assert!(self.update_channel_by_name(channel_name.as_slice(), |channel| {
            channel.users.insert(user_id);
        }), "Got message for channel {:?} without knowing about it.");
    }

    fn on_self_join(&mut self, join: &JoinSuccess) {
        let channel_name = ::std::str::from_utf8(join.channel.as_slice()).ok().unwrap();
        let channel_name = IrcIdentifier::from_str(channel_name);

        if let Some(_) = self.channel_map.get(&channel_name) {
            warn!("Joining already joined channel {:?}; skipped", join.channel);
            return;
        }
        warn!("users = {:?}", join.nicks);
        let new_chan_id = ChannelId(self.channel_seq);
        self.channel_seq += 1;

        self.channels.insert(new_chan_id, Channel::from_info(
            &ChannelInfo::from_join(new_chan_id, join)));
        self.channel_map.insert(channel_name.clone(), new_chan_id);
    }

    fn validate_state_with_who(&self, who: &WhoSuccess) {
        let channel_name = ::std::str::from_utf8(who.channel.as_slice()).ok().unwrap();
        let channel_name = IrcIdentifier::from_str(channel_name);

        let (_, channel) = match self.get_channel_by_name(channel_name.as_slice()) {
            Some(chan_pair) => chan_pair,
            None => return
        };

        info!("Validating channel state");
        let mut known_users = HashSet::new();
        for user in channel.users.iter() {
            match self.users.get(user) {
                Some(user) => {
                    known_users.insert(user.get_nick().to_string());
                },
                None => panic!("Inconsistent state"),
            }
        }

        let mut valid_users = HashSet::new();
        for rec in who.who_records.iter() {
            valid_users.insert(rec.nick.clone());
        }

        let mut is_valid = true;
        for valid_unknowns in valid_users.difference(&known_users) {
            warn!("Valid but unknown nick: {:?}", valid_unknowns);
            is_valid = false;
        }

        for invalid_knowns in known_users.difference(&valid_users) {
            warn!("Known but invalid nick: {:?}", invalid_knowns);
            is_valid = false;
        }

        if is_valid {
            info!("Channel state has been validated: sychronized");
        } else {
            warn!("Channel state has been validated: desynchronized!");
        }
    }

    fn on_who(&mut self, who: &WhoSuccess) {
        // If we WHO a channel that we aren't in, we aren't changing any
        // state.
        let channel_name = ::std::str::from_utf8(who.channel.as_slice()).ok().unwrap();
        let channel_name = IrcIdentifier::from_str(channel_name);

        let chan_id = match self.get_channel_by_name(channel_name.as_slice()) {
            Some((chan_id, channel)) => {
                if !channel.users.is_empty() {
                    self.validate_state_with_who(who);
                    return;
                }
                chan_id
            }
            None => return
        };

        let mut users = Vec::with_capacity(who.who_records.len());
        let mut user_ids = Vec::with_capacity(who.who_records.len());

        for rec in who.who_records.iter() {
            let nick = IrcIdentifier::from_str(rec.nick.as_slice());
            user_ids.push(match self.user_map.get(&nick) {
                Some(user_id) => *user_id,
                None => {
                    let new_user_id = UserId(self.user_seq);
                    self.user_seq += 1;
                    users.push(User::from_who(new_user_id, rec));
                    new_user_id
                }
            });
        }
        for user in users.into_iter() {
            self.insert_user(user);
        }
        for user_id in user_ids.iter() {
            match self.users.get_mut(user_id) {
                Some(user_state) => {
                    user_state.channels.insert(chan_id);
                },
                None => {
                    if *user_id != self.self_id {
                        panic!("{:?}", user_id);
                    }
                }
            };
        }

        let tmp_chan_name = channel_name.clone();
        assert!(self.update_channel_by_name(channel_name.as_slice(), move |channel| {
            let added = user_ids.len() - channel.users.len();
            info!("Added {:?} users for channel {:?}", added, tmp_chan_name);
            channel.users.extend(user_ids.into_iter());
        }), "Got message for channel {:?} without knowing about it.");
    }

    fn on_topic(&mut self, topic: &server::Topic) {
        assert!(self.update_channel_by_name(topic.get_channel(), |channel| {
            let topic = String::from_utf8_lossy(topic.get_body_raw()).into_owned();
            channel.set_topic(topic.as_slice());
        }));
    }

    fn on_nick(&mut self, nick: &server::Nick) {
        assert!(self.update_user_by_nick(nick.get_nick(), |user| {
            user.set_nick(nick.get_new_nick());
        }))
    }

    //
    fn on_kick(&mut self, kick: &server::Kick) {
        let channel_name = IrcIdentifier::from_str(kick.get_channel());
        let kicked_user_nick = IrcIdentifier::from_str(kick.get_kicked_nick());

        let (chan_id, user_id) = match (
            self.channel_map.get(&channel_name),
            self.user_map.get(&kicked_user_nick)
        ) {
            (Some(chan_id), Some(user_id)) => (*chan_id, *user_id),
            (None, Some(_)) => {
                warn!("Strange: unknown channel {:?}", channel_name);
                return;
            },
            (Some(_), None) => {
                warn!("Strange: unknown nick {:?}", kicked_user_nick);
                return;
            },
            (None, None) => {
                warn!("Strange: unknown chan {:?} and nick {:?}", channel_name, kicked_user_nick);
                return;
            }
        };
        self.unlink_user_channel(user_id, chan_id);
    }

    pub fn is_self_join(&self, msg: &IrcMsg) -> Option<server::Join> {
        use message_types::server::IncomingMsg::Join;

        let is_self = msg.get_prefix().nick().and_then(|nick| {
            Some(nick.as_slice() == self.self_nick.as_slice())
        }).unwrap_or(false);

        if !is_self {
            return None;
        }
        match server::IncomingMsg::from_msg(msg.clone()) {
            Join(join) => Some(join),
            _ => None,
        }
    }

    pub fn on_message(&mut self, msg: &IrcMsg) {
        use message_types::server::IncomingMsg::{Part, Quit, Join, Topic, Kick, Nick};

        let ty_msg = server::IncomingMsg::from_msg(msg.clone());
        let is_self = msg.get_prefix().nick().and_then(|nick| {
            Some(nick.as_slice() == self.self_nick.as_slice())
        }).unwrap_or(false);

        match (&ty_msg, is_self) {
            (&Part(ref part), true) => return self.on_self_part(part),
            (&Part(ref part), false) => return self.on_other_part(part),
            (&Quit(ref quit), false) => return self.on_other_quit(quit),
            // is this JOIN right?
            (&Join(ref join), false) => return self.on_other_join(join),
            (&Topic(ref topic), _) => return self.on_topic(topic),
            (&Nick(ref nick), _) => return self.on_nick(nick),
            (&Kick(ref kick), _) => return self.on_kick(kick),
            (_, _) => ()
        }

        if msg.get_command() == "001" {
            let channel_name = ::std::str::from_utf8(&msg[0]).ok().unwrap();
            self.initialize_self_nick(channel_name);
        }
    }

    pub fn on_event(&mut self, event: &IrcEvent) {
        let () = match *event {
            IrcEvent::IrcMsg(ref message) => self.on_message(message),
            IrcEvent::JoinBundle(Ok(ref join_bun)) => self.on_self_join(join_bun),
            IrcEvent::JoinBundle(Err(_)) => (),
            IrcEvent::WhoBundle(Ok(ref who_bun)) => self.on_who(who_bun),
            IrcEvent::WhoBundle(Err(_)) => (),
        };
    }

    pub fn get_self_nick<'a>(&'a self) -> &'a str {
        self.self_nick.as_slice()
    }

    pub fn set_self_nick(&mut self, new_nick_str: &str) {
        let new_nick = IrcIdentifier::from_str(new_nick_str);
        let old_nick = IrcIdentifier::from_str(self.self_nick.as_slice());
        if self.self_nick.as_slice() != "" {
            let user_id = match self.user_map.remove(&old_nick) {
                Some(user_id) => user_id,
                None => panic!("inconsistent user_map: {:?}[{:?}]",
                    self.user_map, self.self_nick)
            };
            self.user_map.insert(new_nick, user_id);
        }
        self.self_nick = new_nick_str.to_string();
    }

    fn initialize_self_nick(&mut self, new_nick_str: &str) {
        let new_nick = IrcIdentifier::from_str(new_nick_str);
        self.user_map.insert(new_nick, self.self_id);
        self.users.insert(self.self_id, User {
            id: self.self_id,
            // FIXME: hack
            prefix: IrcMsgPrefix::new(format!("{}!someone@somewhere", new_nick_str).into_cow()),
            channels: HashSet::new(),
        });
        self.set_self_nick(new_nick_str);
    }

    fn apply_update_self_nick(&mut self, new_nick_str: &str) {
        let new_nick = IrcIdentifier::from_str(new_nick_str);
        let old_nick = IrcIdentifier::from_str(self.self_nick.as_slice());
        assert!(self.user_map.remove(&old_nick).is_some());
        self.set_self_nick(new_nick_str.as_slice());
        self.user_map.insert(new_nick, self.self_id);
    }

    fn apply_remove_channel(&mut self, id: ChannelId) {
        info!("remove_channel({:?})", id);
        self.remove_channel_by_id(id);
    }

    fn apply_create_chan(&mut self, chan_info: &ChannelInfo) {
        let ChannelId(chan_id) = chan_info.id;
        self.channel_seq = max(self.channel_seq, chan_id);

        self.channels.insert(chan_info.id, Channel::from_info(chan_info));
        let channel_name = IrcIdentifier::from_str(chan_info.name.as_slice());
        self.channel_map.insert(channel_name, chan_info.id);
    }

    fn apply_update_chan(&mut self, id: ChannelId, diff: &Vec<ChannelDiffCmd>) {
        match self.channels.entry(id) {
            hash_map::Entry::Occupied(mut entry) => {
                let channel_state = entry.get().patch(diff);
                entry.insert(channel_state);
            }
            hash_map::Entry::Vacant(_) => warn!("Unknown channel {:?}", id)
        };
    }

    fn apply_create_user(&mut self, user_info: &UserInfo) {
        let UserId(user_id) = user_info.id;
        self.user_seq = max(self.user_seq, user_id);


        self.users.insert(user_info.id, User::from_info(user_info));
        self.user_map.insert(IrcIdentifier::from_str(user_info.get_nick()), user_info.id);
    }

    fn apply_update_user(&mut self, id: UserId, diff: &Vec<UserDiffCmd>) {
        match self.users.entry(id) {
            hash_map::Entry::Occupied(mut entry) => {

                let old_nick = IrcIdentifier::from_str(entry.get().get_nick());
                let new_user = entry.get().patch(diff);
                let new_nick = IrcIdentifier::from_str(new_user.get_nick());

                if old_nick != new_nick {
                    assert_eq!(self.user_map.remove(&old_nick), Some(id));
                    self.user_map.insert(new_nick, id);
                }
                entry.insert(new_user);
            }
            hash_map::Entry::Vacant(_) => warn!("Unknown channel {:?}", id)
        };
    }

    fn apply_remove_user(&mut self, id: UserId) {
        info!("apply_remove_user({:?})", id);
        let user_info = match self.users.remove(&id) {
            Some(user_info) => user_info,
            None => panic!("cannot apply command: {:?} not found.", id)
        };
        let user_nick = IrcIdentifier::from_str(user_info.get_nick());
        match self.user_map.remove(&user_nick) {
            Some(user_id) => assert_eq!(user_id, id),
            None => panic!("inconsistent user_mapm: {:?}[{:?}]",
                self.user_map, user_nick)
        };
    }

    fn apply_command(&mut self, cmd: &StateCommand) {
        match *cmd {
            StateCommand::UpdateSelfNick(ref new_nick) =>
                self.apply_update_self_nick(new_nick.as_slice()),
            StateCommand::SetGeneration(generation) => self.generation = generation,

            StateCommand::CreateUser(ref info) =>
                self.apply_create_user(info),
            StateCommand::UpdateUser(id, ref diff) =>
                self.apply_update_user(id, diff),
            StateCommand::RemoveUser(id) =>
                self.apply_remove_user(id),

            StateCommand::CreateChannel(ref info) =>
                self.apply_create_chan(info),
            StateCommand::UpdateChannel(id, ref diff) =>
                self.apply_update_chan(id, diff),
            StateCommand::RemoveChannel(id) =>
                self.apply_remove_channel(id),
        }
    }

    fn unlink_user_channel(&mut self, uid: UserId, chid: ChannelId) {
        let should_remove = match self.users.entry(uid) {
            hash_map::Entry::Occupied(mut entry) => {
                if entry.get().channels.len() == 1 && entry.get().channels.contains(&chid) {
                    true
                } else {
                    entry.get_mut().channels.remove(&chid);
                    false
                }
            }
            hash_map::Entry::Vacant(_) => panic!("Inconsistent state")
        };
        if should_remove {
            warn!("removing {:?}", uid);
            self.remove_user_by_id(uid);
        }

        let should_remove = match self.channels.entry(chid) {
            hash_map::Entry::Occupied(mut entry) => {
                if entry.get().users.len() == 1 && entry.get().users.contains(&uid) {
                    true
                } else {
                    entry.get_mut().users.remove(&uid);
                    false
                }
            },
            hash_map::Entry::Vacant(_) => panic!("Inconsistent state")
        };
        if should_remove {
            warn!("removing {:?}", chid);
            self.remove_channel_by_id(chid);
        }
    }
    fn update_channel<F>(&mut self, id: ChannelId, modfunc: F) -> bool where
        F: FnOnce(&mut Channel) -> ()
    {
        match self.channels.entry(id) {
            hash_map::Entry::Occupied(mut entry) => {
                // Channel currently has no indexed mutable state
                modfunc(entry.get_mut());
                true
            }
            hash_map::Entry::Vacant(_) => false
        }
    }

    fn update_channel_by_name<F>(&mut self, name: &str, modfunc: F) -> bool where
        F: FnOnce(&mut Channel) -> ()
    {
        let chan_id = deref_opt_or_return!(
            self.channel_map.get(&IrcIdentifier::from_str(name)),
            "Unknown channel name", false);
        let result = self.update_channel(chan_id, modfunc);
        self.validate_state_internal_panic();
        result
    }

    fn remove_channel_by_name(&mut self, name: &str) -> Option<ChannelId> {
        let chan_id = deref_opt_or_return!(
            self.channel_map.get(&IrcIdentifier::from_str(name)),
            "Unknown channel name", None);
        assert!(self.remove_channel_by_id(chan_id));
        self.validate_state_internal_panic();
        Some(chan_id)
    }

    fn remove_channel_by_id(&mut self, id: ChannelId) -> bool {
        let (chan_name, users): (_, Vec<_>) = match self.channels.get(&id) {
            Some(chan_state) => (
                IrcIdentifier::from_str(chan_state.name.as_slice()),
                chan_state.users.iter().map(|x| *x).collect()
            ),
            None => return false
        };
        for user_id in users.into_iter() {
            self.channels.get_mut(&id).unwrap().users.remove(&user_id);
            self.users.get_mut(&user_id).unwrap().channels.remove(&id);
            // self.unlink_user_channel(user_id, id);
        }
        self.channels.remove(&id);
        self.channel_map.remove(&chan_name);
        self.validate_state_internal_panic();
        true
    }

    fn get_channel_by_name(&self, name: &str) -> Option<(ChannelId, &Channel)> {
        let chan_id = match self.channel_map.get(&IrcIdentifier::from_str(name)) {
            Some(chan_id) => *chan_id,
            None => return None
        };
        match self.channels.get(&chan_id) {
            Some(channel) => Some((chan_id, channel)),
            None => panic!("Inconsistent state")
        }
    }

    fn insert_user(&mut self, user: User) {
        let user_id = user.id;
        let nick = IrcIdentifier::from_str(user.prefix.nick().unwrap());
        assert!(self.users.insert(user_id, user).is_none());
        assert!(self.user_map.insert(nick, user_id).is_none());
        self.validate_state_internal_panic();
    }

    fn update_user_by_nick<F>(&mut self, nick: &str, modfunc: F) -> bool where
        F: FnOnce(&mut User) -> ()
    {
        let nick = IrcIdentifier::from_str(nick);
        let user_id = deref_opt_or_return!(self.user_map.get(&nick),
            "Couldn't find user by nick", false);
        let result = self.update_user(user_id, modfunc);

        self.validate_state_internal_panic();
        result
    }

    fn update_user<F>(&mut self, id: UserId, modfunc: F) -> bool where
        F: FnOnce(&mut User) -> ()
    {
        match self.users.entry(id) {
            hash_map::Entry::Occupied(mut entry) => {
                let prev_nick = IrcIdentifier::from_str(entry.get().prefix.nick().unwrap());
                modfunc(entry.get_mut());
                let new_nick = IrcIdentifier::from_str(entry.get().prefix.nick().unwrap());
                warn!("prev_nick != new_nick || {:?} != {:?}", prev_nick, new_nick);
                if prev_nick != new_nick {
                    warn!("self.user_map -- REMOVE {:?}; INSERT {:?}", prev_nick, new_nick);
                    self.user_map.remove(&prev_nick);
                    self.user_map.insert(new_nick, id);
                }
                true
            }
            hash_map::Entry::Vacant(_) => false
        }
    }

    fn remove_user_by_nick(&mut self, name: &str) -> Option<UserId> {
        let user_id = match self.user_map.get(&IrcIdentifier::from_str(name)) {
            Some(user_id) => *user_id,
            None => return None
        };
        match self.remove_user_by_id(user_id) {
            true => Some(user_id),
            false => panic!("Inconsistent state")
        }
    }

    fn remove_user_by_id(&mut self, id: UserId) -> bool {
        if self.self_id == id {
            panic!("Tried to remove self");
        }
        let (nick, channels): (_, Vec<_>) = match self.users.get(&id) {
            Some(user_state) => (
                IrcIdentifier::from_str(user_state.prefix.nick().unwrap()),
                user_state.channels.iter().map(|x| *x).collect(),
            ),
            None => return false
        };
        for chan_id in channels.into_iter() {
            self.channels.get_mut(&chan_id).unwrap().users.remove(&id);
            self.users.get_mut(&id).unwrap().channels.remove(&chan_id);
        }

        self.users.remove(&id).unwrap();
        self.user_map.remove(&nick).unwrap();
        self.validate_state_internal_panic();
        true
    }

    pub fn identify_channel(&self, chan: &str) -> Option<ChannelId> {
        match self.channel_map.get(&IrcIdentifier::from_str(chan)) {
            Some(chan_id) => Some(chan_id.clone()),
            None => None
        }
    }

    pub fn resolve_channel(&self, chid: ChannelId) -> Option<&Channel> {
        self.channels.get(&chid)
    }

    pub fn identify_nick(&self, nick: &str) -> Option<UserId> {
        match self.user_map.get(&IrcIdentifier::from_str(nick)) {
            Some(user_id) => Some(*user_id),
            None => None
        }
    }

    pub fn resolve_user(&self, uid: UserId) -> Option<&User> {
        self.users.get(&uid)
    }

    pub fn clone_frozen(&self) -> FrozenState {
        FrozenState(self.clone())
    }
}

#[cfg(not(test))]
impl State {
    fn validate_state_internal_panic(&mut self) {
    }
}

#[cfg(test)]
impl State {
    fn validate_state_internal_panic(&mut self) {
        match self.validate_state_internal() {
            Ok(()) => (),
            Err(msg) => panic!("invalid state: {:?}, dump = {:?}", msg, self)
        };
    }


    fn validate_state_internal(&self) -> Result<(), String> {
        for (&id, state) in self.channels.iter() {
            if id != state.id {
                return Err(format!("{:?} at channels[{:?}]", state.id, id));
            }
            for &user_id in state.users.iter() {
                if let Some(user_state) = self.users.get(&user_id) {
                    if !user_state.channels.contains(&id) {
                        return Err(format!("{0:?} ref {1:?} => {1:?} ref {0:?} not holding", id, user_id));
                    }
                } else {
                    return Err(format!("{:?} refs non-existent {:?}", id, user_id));
                }
            }
        }
        for (&id, state) in self.users.iter() {
            if id != state.id {
                return Err(format!("{:?} at users[{:?}]", state.id, id));
            }
            for &chan_id in state.channels.iter() {
                if let Some(chan_state) = self.channels.get(&chan_id) {
                    if !chan_state.users.contains(&id) {
                        return Err(format!("{0:?} ref {1:?} => {1:?} ref {0:?} not holding", id, chan_id));
                    }
                } else {
                    return Err(format!("{:?} refs non-existent {:?}", id, chan_id));
                }
            }
        }
        for (name, &id) in self.channel_map.iter() {
            if let Some(state) = self.channels.get(&id) {
                if *name != IrcIdentifier::from_str(state.name.as_slice()) {
                    return Err(format!("{:?} at channel_map[{:?}]", state.id, name));
                }
            } else {
                return Err(format!("channel map inconsistent"));
            }
        }
        for (name, &id) in self.user_map.iter() {
            if let Some(state) = self.users.get(&id) {
                if *name != IrcIdentifier::from_str(state.get_nick()) {
                    return Err(format!("{:?} at user_map[{:?}]", state.id, name));
                }
            } else {
                return Err(format!(
                    concat!(
                        "user map inconsistent: self.user_map[{:?}] is not None ",
                        "=> self.users[{:?}] is not None"
                    ), name, id));
            }
        }
        Ok(())
    }
}

impl Eq for State {}

impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        for (nick, id) in self.user_map.iter() {
            if Some(id) != other.user_map.get(nick) {
                return false;
            }
        }
        for (nick, id) in other.user_map.iter() {
            if Some(id) != self.user_map.get(nick) {
                return false;
            }
        }
        for (name, id) in self.channel_map.iter() {
            if Some(id) != other.channel_map.get(name) {
                return false;
            }
        }
        for (name, id) in other.channel_map.iter() {
            if Some(id) != self.channel_map.get(name) {
                return false;
            }
        }
        for (id, self_state) in self.users.iter() {
            if let Some(other_state) = other.users.get(id) {
                if self_state != other_state {
                    return false;
                }
            } else {
                return false;
            }
        }
        for (id, self_state) in self.channels.iter() {
            if let Some(other_state) = other.channels.get(id) {
                if self_state != other_state {
                    return false;
                }
            } else {
                return false;
            }
        }

        if self.user_seq != other.user_seq {
            return false;
        }
        if self.channel_seq != other.channel_seq {
            return false;
        }
        if self.self_nick != other.self_nick {
            return false;
        }
        if self.generation != other.generation {
            return false;
        }
        return true;
    }
}

impl Diff<StateDiff> for State {
    fn diff(&self, other: &State) -> StateDiff {
        let mut commands = Vec::new();
        if self.self_nick != other.self_nick {
            commands.push(StateCommand::UpdateSelfNick(other.self_nick.clone()));
        }

        for (&id, cstate) in other.channels.iter() {
            if let Some(old_channel) = self.channels.get(&id) {
                if cstate != old_channel {
                    commands.push(StateCommand::UpdateChannel(id, old_channel.diff(cstate)));
                }
            } else {
                commands.push(StateCommand::CreateChannel(ChannelInfo::from_internal(cstate)));
                if !cstate.users.is_empty() {
                    let diff: Vec<_> = cstate.users.iter()
                        .map(|&x| ChannelDiffCmd::AddUser(x)).collect();
                    commands.push(StateCommand::UpdateChannel(id, diff));
                }
            }
        }
        for (&id, _) in self.channels.iter() {
            if !other.channels.contains_key(&id) {
                commands.push(StateCommand::RemoveChannel(id));
            }
        }

        for (&id, ustate) in other.users.iter() {
            if let Some(old_user) = self.users.get(&id) {
                if ustate != old_user {
                    commands.push(StateCommand::UpdateUser(id, old_user.diff(ustate)));
                }
            } else {
                commands.push(StateCommand::CreateUser(UserInfo::from_internal(ustate)));
                if !ustate.channels.is_empty() {
                    let diff: Vec<_> = ustate.channels.iter()
                        .map(|&x| UserDiffCmd::AddChannel(x)).collect();
                    commands.push(StateCommand::UpdateUser(id, diff));
                }
            }
        }
        for (&id, _) in self.users.iter() {
            if !other.users.contains_key(&id) {
                commands.push(StateCommand::RemoveUser(id));
            }
        }

        if self.generation != other.generation {
            commands.push(StateCommand::SetGeneration(other.generation));
        }

        StateDiff {
            from_generation: self.generation,
            to_generation: other.generation,
            commands: commands,
        }
    }
}

impl Patch<StateDiff> for State {
    fn patch(&self, diff: &StateDiff) -> State {
        let mut new = self.clone();
        assert_eq!(self.generation, diff.from_generation);
        for command in diff.commands.iter() {
            new.apply_command(command);
        }
        assert_eq!(self.generation, diff.from_generation);
        new
    }
}


// #[cfg(test)]
// mod tests {
//     use std::old_io::BufReader;

//     use super::{State, UserId};
//     use super::irc_identifier::IrcIdentifier;

//     use connection::IrcConnectionBuf;
//     use testinfra::transcript::{
//         SessionRecord,
//         decode_line,
//         marker_match,
//     };

//     const TEST_SESSION_STATETRACKER: &'static [u8] =
//         include_bytes!("../testdata/statetracker.txt");

//     #[test]
//     fn test_state_tracking() {
//         let mut reader = BufReader::new(TEST_SESSION_STATETRACKER);
//         let mut iterator = reader.lines().filter_map(decode_line);

//         let mut connection = IrcConnectionBuf::new();
//         let mut state = State::new();

//         let mut it = |target: &str, statefunc: &mut FnMut(&mut State)| {
//             if target != "" {
//                 for rec in iterator {
//                     println!("Processing message: {:?}", rec);
//                     if marker_match(&rec, target) {
//                         break;
//                     }
//                     if let SessionRecord::Content(ref content) = rec {
//                         connection.push_line(content.as_bytes().to_vec());
//                         connection.dispatch();
//                         while let Some(event) = connection.pop_event() {
//                             println!("event: {:?}", event);
//                             state.on_event(&event);
//                             state.validate_state_internal_panic();
//                         }
//                     }
//                 }
//             }
//             statefunc(&mut state);
//         };

//         let mut random_user_id_hist = Vec::new();
//         let mut chan_test_id_hist = Vec::new();

//         it("should have a channel `#test` with 7 users", &mut |state| {
//             let test_channel = IrcIdentifier::from_str("#test");
//             let channel_id = match state.channel_map.get(&test_channel) {
//                 Some(channel_id) => *channel_id,
//                 None => panic!("channel `#test` not found.")
//             };
//             chan_test_id_hist.push(channel_id);

//             let channel_state = match state.channels.get(&channel_id) {
//                 Some(channel) => channel.clone(),
//                 None => panic!("channel `#test` had Id but no state")
//             };
//             assert_eq!(channel_state.users.len(), 7);
//         });

//         it("topic of `#test` should be `irc is bad.`", &mut |state| {
//             let msg = format!("state.identify_channel failed on line {}", 1 + line!());
//             let chan_id = state.identify_channel("#test").expect(msg.as_slice());
//             let msg = format!("state.channels.find failed on line {}", 1 + line!());
//             let channel = state.channels.get(&chan_id).expect(msg.as_slice());
//             assert_eq!(channel.topic.as_slice(), "irc is bad.");
//         });

//         it("should have a user `randomuser` after JOIN", &mut |state| {
//             let msg = format!("state.identify_nick failed on line {}", 1 + line!());
//             let randomuser_id = state.identify_nick("randomuser").expect(msg.as_slice());
//             if random_user_id_hist.contains(&randomuser_id) {
//                 assert!(false, "nick `randomuser` UserId must change between losses in view");
//             }
//             random_user_id_hist.push(randomuser_id);
//             match state.users.get(&randomuser_id) {
//                 Some(randomuser) => {
//                     assert_eq!(
//                         randomuser.prefix.as_slice(),
//                         "randomuser!rustbot@coolhost");
//                 },
//                 None => panic!("inconsistent state. state = {:?}", state)
//             }
//         });

//         it("should not have a user `randomuser` after PART", &mut |state| {
//             assert_eq!(state.identify_nick("randomuser"), None);
//         });

//         it("should not have a user `randomuser` after KICK", &mut |state| {
//             assert_eq!(state.identify_nick("randomuser"), None);
//         });

//         it("should not have a user `randomuser` after QUIT", &mut |state| {
//             assert_eq!(state.identify_nick("randomuser"), None);
//         });

//         it("topic of `#test` should be `setting a cool topic`", &mut |&: state| {
//             let msg = format!("state.identify_channel failed on line {}", 1 + line!());
//             let chan_id = state.identify_channel("#test").expect(msg.as_slice());
//             let msg = format!("state.channels.find failed on line {}", 1 + line!());
//             let channel = state.channels.get(&chan_id).expect(msg.as_slice());
//             assert_eq!(channel.topic.as_slice(), "setting a cool topic");
//         });

//         let mut randomuser_id: Option<UserId> = None;
//         it("store randomuser's UserID here", &mut |state| {
//             randomuser_id = state.identify_nick("randomuser")
//         });
//         let randomuser_id = randomuser_id.expect("bad randomuser");

//         it("ensure randomuser's UserID has not changed after a nick change", &mut |state| {
//             assert_eq!(state.identify_nick("resumodnar"), Some(randomuser_id));
//         });

//         it("should not have a channel `#test` anymore", &mut |state| {
//             assert!(
//                 state.identify_channel("#test").is_none(),
//                 "#test was not cleaned up correctly");
//         });

//         it("should have the channel `#test` once again", &mut |state| {
//             let test_id = state.identify_channel("#test").unwrap();
//             if chan_test_id_hist.contains(&test_id) {
//                 assert!(false, "channel `#test` ChannelId must change between losses in view");
//             }
//             chan_test_id_hist.push(test_id);
//         });

//         let mut randomuser_id: Option<UserId> = None;

//         it("should have a channel `#test2` with 2 users", &mut |state| {
//             assert!(state.identify_channel("#test2").is_some());
//             randomuser_id = state.identify_nick("randomuser");
//             assert!(randomuser_id.is_some(), "randomuser wasn't found!");
//         });

//         it("randomuser should have the same ID as before", &mut |state| {
//             assert!(state.identify_channel("#test2").is_some());
//             assert_eq!(
//                 state.identify_nick("randomuser").unwrap(),
//                 randomuser_id.unwrap());
//         });

//         it("randomuser should have been forgotten", &mut |state| {
//             assert_eq!(state.identify_nick("randomuser"), None);
//         });

//         it("randomuser should not have the same ID as before", &mut |state| {
//             assert!(state.identify_channel("#test2").is_some());
//             if state.identify_nick("randomuser") == randomuser_id {
//                 assert!(false, "randomuser should be different now");
//             }
//         });
//     }
// }
