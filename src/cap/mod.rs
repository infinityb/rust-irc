use std::slice;
use std::any::Any;
use std::collections::HashMap;

/// The available IRCv3 capability negotiation versions.
pub enum NegotiationVersion {
    /// [IRCv3.1](http://ircv3.net/specs/core/capability-negotiation-3.1.html)
    V301,
    /// [IRCv3.2](http://ircv3.net/specs/core/capability-negotiation-3.2.html)
    V302,
}


pub struct Capabilities {
    items: HashMap<String, Vec<u8>>
}

impl Capabilities {
    pub fn new() -> Capabilities {
        Capabilities {
            items: HashMap::new(),
        }
    }

    pub fn set<C: Capability + CapabilityFormat>(&mut self, value: C) {
        unimplemented!();
    }

    pub fn get<C: Capability + CapabilityFormat>(&self) -> Option<&C> {
        unimplemented!();
    }

    pub fn iter_raw(&self) -> CapabilitiesRawIter {
        unimplemented!();
    }
}

pub struct CapabilitiesRawIter<'a> {
    _marker: ::std::marker::PhantomData<&'a ()>,
}

impl<'a> Iterator for CapabilitiesRawIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        unimplemented!();
    }
}


pub trait Capability: Clone + Any { // Send + Sync?
    fn capability_name() -> &'static str;
    fn parse_capability(cap: &[u8]) -> Result<Self, ()>;
}

pub trait CapabilityFormat: Clone + Any { // Send + Sync?
}

#[derive(Clone)]
pub struct MultiPrefix;

impl Capability for MultiPrefix {
    fn capability_name() -> &'static str {
        "multi-prefix"
    }

    fn parse_capability(cap: &[u8]) -> Result<MultiPrefix, ()> {
        if cap == MultiPrefix::capability_name().as_bytes() {
            return Ok(MultiPrefix)
        }
        return Err(())
    }
}


#[derive(Clone)]
pub struct ExtendedJoin;

impl Capability for ExtendedJoin {
    fn capability_name() -> &'static str {
        "extended-join"
    }

    fn parse_capability(cap: &[u8]) -> Result<ExtendedJoin, ()> {
        if cap == ExtendedJoin::capability_name().as_bytes() {
            return Ok(ExtendedJoin)
        }
        return Err(())
    }
}


#[derive(Clone)]
pub struct AccountNotify;

impl Capability for AccountNotify {
    fn capability_name() -> &'static str {
        "account-notify"
    }

    fn parse_capability(cap: &[u8]) -> Result<AccountNotify, ()> {
        if cap == AccountNotify::capability_name().as_bytes() {
            return Ok(AccountNotify)
        }
        return Err(())
    }
}


#[derive(Clone)]
pub struct Batch;

impl Capability for Batch {
    fn capability_name() -> &'static str {
        "batch"
    }

    fn parse_capability(cap: &[u8]) -> Result<Batch, ()> {
        if cap == Batch::capability_name().as_bytes() {
            return Ok(Batch)
        }
        return Err(())
    }
}

#[derive(Clone)]
pub struct InviteNotify;

impl Capability for InviteNotify {
    fn capability_name() -> &'static str {
        "invite-notify"
    }

    fn parse_capability(cap: &[u8]) -> Result<InviteNotify, ()> {
        if cap == InviteNotify::capability_name().as_bytes() {
            return Ok(InviteNotify)
        }
        return Err(())
    }
}


#[derive(Clone)]
pub struct Tls;

impl Capability for Tls {
    fn capability_name() -> &'static str {
        "tls"
    }

    fn parse_capability(cap: &[u8]) -> Result<Tls, ()> {
        if cap == Tls::capability_name().as_bytes() {
            return Ok(Tls)
        }
        return Err(())
    }
}


#[derive(Clone)]
pub struct CapNotify;

impl Capability for CapNotify {
    fn capability_name() -> &'static str {
        "cap-notify"
    }

    fn parse_capability(cap: &[u8]) -> Result<CapNotify, ()> {
        if cap == CapNotify::capability_name().as_bytes() {
            return Ok(CapNotify)
        }
        return Err(())
    }
}

#[derive(Clone)]
pub struct ServerTime;

impl Capability for ServerTime {
    fn capability_name() -> &'static str {
        "server-time"
    }

    fn parse_capability(cap: &[u8]) -> Result<ServerTime, ()> {
        if cap == ServerTime::capability_name().as_bytes() {
            return Ok(ServerTime)
        }
        return Err(())
    }
}


#[derive(Clone)]
pub struct UserhostInNames;

impl Capability for UserhostInNames {
    fn capability_name() -> &'static str {
        "userhost-in-names"
    }

    fn parse_capability(cap: &[u8]) -> Result<UserhostInNames, ()> {
        if cap == UserhostInNames::capability_name().as_bytes() {
            return Ok(UserhostInNames)
        }
        return Err(())
    }
}

#[derive(Clone)]
pub struct Sasl {
    args: Vec<u8>
}

impl Capability for Sasl {
    fn capability_name() -> &'static str {
        "sasl"
    }

    fn parse_capability(cap: &[u8]) -> Result<Sasl, ()> {
        if cap == Sasl::capability_name().as_bytes() {
            return Ok(Sasl { args: Vec::new() });
        }

        if cap.starts_with(b"sasl=") {
            return Ok(Sasl { args: cap[5..].to_vec() });
        }

        return Err(())
    }
}

impl Sasl {
    /// returns an iterator that yields the supported protocols, if any
    pub fn protocols(&self) -> SaslProtocolIter {
        SaslProtocolIter {
            inner: self.args.split(is_comma)
        }
    }
}

fn is_comma(by: &u8) -> bool {
    *by == b','
}

pub struct SaslProtocolIter<'a> {
    inner: slice::Split<'a, u8, fn(&u8) -> bool>
}

impl<'a> Iterator for SaslProtocolIter<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<&'a [u8]> {
        self.inner.next()
    }
}
