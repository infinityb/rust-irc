use std::{slice, fmt, str};
use std::any::{Any, TypeId};
use std::collections::{HashMap, hash_map};
use std::borrow::{Cow, ToOwned};

use unicase::UniCase;

/// The available IRCv3 capability negotiation versions.
pub enum NegotiationVersion {
    /// [IRCv3.1](http://ircv3.net/specs/core/capability-negotiation-3.1.html)
    V301,
    /// [IRCv3.2](http://ircv3.net/specs/core/capability-negotiation-3.2.html)
    V302,
}

type CapabilityName = UniCase<Cow<'static, str>>;

pub struct Capabilities {
    items: HashMap<CapabilityName, String>,
}

impl Capabilities {
    pub fn new() -> Capabilities {
        Capabilities {
            items: HashMap::new(),
        }
    }

    pub fn set<C: Capability + CapabilityFormat>(&mut self, value: C) {
        let key = UniCase(Cow::Borrowed(C::capability_name()));
        let item = value.serialize_capability();
        self.items.insert(key, item);
    }

    pub fn get<C: Capability + CapabilityFormat>(&self) -> Option<C> {
        let key = UniCase(Cow::Borrowed(C::capability_name()));

        self.items.get(&key).and_then(|buf| {
            Capability::parse_capability(buf.as_bytes()).ok()
        })
    }

    pub fn iter_raw(&self) -> CapabilitiesRawIter {
        CapabilitiesRawIter { piter: self.items.values() }
    }
}

pub struct CapabilitiesRawIter<'a> {
    piter: hash_map::Values<'a, CapabilityName, String>,
}

impl<'a> Iterator for CapabilitiesRawIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        self.piter.next().map(|ser| &ser[..])
    }
}


pub trait Capability: Clone + Any { // Send + Sync?
    fn capability_name() -> &'static str;

    fn serialize_capability(&self) -> String {
        Self::capability_name().to_string()
    }

    fn parse_capability(cap: &[u8]) -> Result<Self, ()>;
}

pub trait CapabilityFormat: Clone + Any { // Send + Sync?
    fn fmt_capability(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

#[derive(Clone)]
pub struct MultiPrefix;

impl Capability for MultiPrefix {
    fn capability_name() -> &'static str {
        "multi-prefix"
    }

    fn parse_capability(cap: &[u8]) -> Result<Self, ()> {
        if cap == Self::capability_name().as_bytes() {
            return Ok(MultiPrefix);
        }
        Err(())
    }
}

impl CapabilityFormat for MultiPrefix {
    fn fmt_capability(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", ExtendedJoin::capability_name())
    }
}

impl fmt::Display for MultiPrefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_capability(f)
    }
}


#[derive(Clone)]
pub struct ExtendedJoin;

impl Capability for ExtendedJoin {
    fn capability_name() -> &'static str {
        "extended-join"
    }

    fn parse_capability(cap: &[u8]) -> Result<Self, ()> {
        if cap == Self::capability_name().as_bytes() {
            return Ok(ExtendedJoin);
        }
        Err(())
    }
}

impl CapabilityFormat for ExtendedJoin {
    fn fmt_capability(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", ExtendedJoin::capability_name())
    }
}

impl fmt::Display for ExtendedJoin {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_capability(f)
    }
}



#[derive(Clone)]
pub struct AccountNotify;

impl Capability for AccountNotify {
    fn capability_name() -> &'static str {
        "account-notify"
    }

    fn parse_capability(cap: &[u8]) -> Result<Self, ()> {
        if cap == Self::capability_name().as_bytes() {
            return Ok(AccountNotify);
        }
        Err(())
    }
}

impl CapabilityFormat for AccountNotify {
    fn fmt_capability(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", AccountNotify::capability_name())
    }
}

impl fmt::Display for AccountNotify {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_capability(f)
    }
}


#[derive(Clone)]
pub struct Batch;

impl Capability for Batch {
    fn capability_name() -> &'static str {
        "batch"
    }

    fn parse_capability(cap: &[u8]) -> Result<Self, ()> {
        if cap == Self::capability_name().as_bytes() {
            return Ok(Batch);
        }
        Err(())
    }
}

impl CapabilityFormat for Batch {
    fn fmt_capability(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Batch::capability_name())
    }
}

impl fmt::Display for Batch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_capability(f)
    }
}


#[derive(Clone)]
pub struct InviteNotify;

impl Capability for InviteNotify {
    fn capability_name() -> &'static str {
        "invite-notify"
    }

    fn parse_capability(cap: &[u8]) -> Result<Self, ()> {
        if cap == Self::capability_name().as_bytes() {
            return Ok(InviteNotify);
        }
        Err(())
    }
}

impl CapabilityFormat for InviteNotify {
    fn fmt_capability(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", InviteNotify::capability_name())
    }
}

impl fmt::Display for InviteNotify {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_capability(f)
    }
}



#[derive(Clone)]
pub struct Tls;

impl Capability for Tls {
    fn capability_name() -> &'static str {
        "tls"
    }

    fn parse_capability(cap: &[u8]) -> Result<Self, ()> {
        if cap == Self::capability_name().as_bytes() {
            return Ok(Tls);
        }
        Err(())
    }
}

impl CapabilityFormat for Tls {
    fn fmt_capability(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Tls::capability_name())
    }
}

impl fmt::Display for Tls {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_capability(f)
    }
}


#[derive(Clone)]
pub struct CapNotify;

impl Capability for CapNotify {
    fn capability_name() -> &'static str {
        "cap-notify"
    }

    fn parse_capability(cap: &[u8]) -> Result<Self, ()> {
        if cap == Self::capability_name().as_bytes() {
            return Ok(CapNotify);
        }
        Err(())
    }
}

impl CapabilityFormat for CapNotify {
    fn fmt_capability(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", CapNotify::capability_name())
    }
}

impl fmt::Display for CapNotify {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_capability(f)
    }
}


#[derive(Clone)]
pub struct ServerTime;

impl Capability for ServerTime {
    fn capability_name() -> &'static str {
        "server-time"
    }

    fn parse_capability(cap: &[u8]) -> Result<Self, ()> {
        if cap == Self::capability_name().as_bytes() {
            return Ok(ServerTime);
        }
        Err(())
    }
}

impl CapabilityFormat for ServerTime {
    fn fmt_capability(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", ServerTime::capability_name())
    }
}

impl fmt::Display for ServerTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_capability(f)
    }
}


#[derive(Clone)]
pub struct UserhostInNames;

impl Capability for UserhostInNames {
    fn capability_name() -> &'static str {
        "userhost-in-names"
    }

    fn parse_capability(cap: &[u8]) -> Result<Self, ()> {
        if cap == Self::capability_name().as_bytes() {
            return Ok(UserhostInNames);
        }
        Err(())
    }
}

impl CapabilityFormat for UserhostInNames {
    fn fmt_capability(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", UserhostInNames::capability_name())
    }
}

impl fmt::Display for UserhostInNames {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_capability(f)
    }
}


#[derive(Clone)]
pub struct Sasl {
    args: String,
}

impl Capability for Sasl {
    fn capability_name() -> &'static str {
        "sasl"
    }

    fn serialize_capability(&self) -> String {
        if self.args.len() == 0 {
            format!("{}", Sasl::capability_name())
        } else {
            format!("{}={}", Sasl::capability_name(), self.args)
        }
    }

    fn parse_capability(cap: &[u8]) -> Result<Sasl, ()> {
        if cap == Sasl::capability_name().as_bytes() {
            return Ok(Sasl { args: String::new() });
        }

        if cap.starts_with(b"sasl=") {
            let args = try!(str::from_utf8(&cap[5..]).map_err(|_| ()));
            return Ok(Sasl { args: args.to_string() });
        }

        return Err(())
    }
}

impl CapabilityFormat for Sasl {
    fn fmt_capability(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.args.len() == 0 {
            write!(f, "{}", Sasl::capability_name())
        } else {
            write!(f, "{}={}", Sasl::capability_name(), self.args)
        }
    }
}

impl fmt::Display for Sasl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt_capability(f)
    }
}

impl Sasl {
    pub fn new(args: &str) -> Sasl {
        Sasl { args: args.to_string() }
    }

    /// returns an iterator that yields the supported protocols, if any
    pub fn protocols(&self) -> SaslProtocolIter {
        SaslProtocolIter {
            inner: self.args.split(is_comma)
        }
    }
}

fn is_comma(ch: char) -> bool {
    ch == ','
}

pub struct SaslProtocolIter<'a> {
    inner: str::Split<'a, fn(char) -> bool>
}

impl<'a> Iterator for SaslProtocolIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        self.inner.next()
    }
}

#[test]
fn swag() {
    let mut caps = Capabilities::new();
    caps.set(UserhostInNames);
    caps.set(Sasl::new("EXTERNAL,DH-AES,DH-BLOWFISH,ECDSA-NIST256P-CHALLENGE,PLAIN"));

    let _uhost: UserhostInNames = caps.get().unwrap();

    let mut out = String::new();
    for cap_phrase in caps.iter_raw() {
        out.push_str(cap_phrase);
        out.push_str(" ");
    }
}
