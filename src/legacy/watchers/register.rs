use super::super::{IrcMsg, numerics};
use super::super::message_types::server;

pub type RegisterResult = Result<(), RegisterError>;

#[derive(Clone, Debug)]
pub struct RegisterError {
    pub errtype: RegisterErrorType,
    pub message: IrcMsg,
}

impl RegisterError {
    pub fn should_pick_new_nickname(&self) -> bool {
        match server::IncomingMsg::from_msg(self.message.clone()) {
            server::IncomingMsg::Numeric(num, _) => {
                numerics::ERR_NICKNAMEINUSE == (num as i32)
            },
            _ => false
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub enum RegisterErrorType {
    NoNicknameGiven,
    NicknameInUse,
    UnavailableResource,
    ErroneousNickname,
    NicknameCollision,
    Restricted,
}

impl RegisterErrorType {

    pub fn is_known_error(result: i32) -> bool {
        RegisterErrorType::from_ord_known(result).is_some()
    }

    pub fn from_ord_known(result: i32) -> Option<RegisterErrorType> {
        match result {
            numerics::ERR_NONICKNAMEGIVEN => Some(RegisterErrorType::NoNicknameGiven),
            numerics::ERR_NICKNAMEINUSE => Some(RegisterErrorType::NicknameInUse),
            numerics::ERR_UNAVAILRESOURCE => Some(RegisterErrorType::UnavailableResource),
            numerics::ERR_ERRONEUSNICKNAME => Some(RegisterErrorType::ErroneousNickname),
            numerics::ERR_NICKCOLLISION => Some(RegisterErrorType::NicknameCollision),
            numerics::ERR_RESTRICTED => Some(RegisterErrorType::Restricted),
            _ => None
        }
    }
}
