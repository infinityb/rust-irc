#![feature(plugin, recover, core_intrinsics)]
#![plugin(afl_plugin)]

extern crate afl;

extern crate irc;

use std::io::{self, Read};
use irc::parse::parse2::IrcMsg;
use irc::mtype2::{client, server};

fn run() {
    let mut buf = Vec::new();
    if let Err(err) = io::stdin().read_to_end(&mut buf) {
        panic!("error: failed to read stdin: {:?}", err);
    }

    let msg: &IrcMsg;
    if let Ok(tmp) = IrcMsg::new(&buf) {
        msg = tmp;
    } else {
        return;
    }

    if let Ok(join) = msg.as_tymsg::<&server::Join>() {
        println!("found JOIN:");
        println!("  source = {:?}", join.get_source());
        println!("  target = {:?}", join.get_target());
    }

    if let Ok(kick) = msg.as_tymsg::<&server::Kick>() {
        println!("found KICK:");
    }

    if let Ok(mode) = msg.as_tymsg::<&server::Mode>() {
        println!("found MODE:");
    }

    if let Ok(nick) = msg.as_tymsg::<&server::Nick>() {
        println!("found NICK:");
    }

    if let Ok(notice) = msg.as_tymsg::<&server::Notice>() {
        println!("found NOTICE:");
    }

    if let Ok(part) = msg.as_tymsg::<&server::Part>() {
        println!("found PART:");
    }

    if let Ok(ping) = msg.as_tymsg::<&server::Ping>() {
        println!("found PING:");
    }

    if let Ok(pong) = msg.as_tymsg::<&server::Pong>() {
        println!("found PONG:");
    }

    if let Ok(privmsg) = msg.as_tymsg::<&server::Privmsg>() {
        println!("found PRIVMSG:");
        println!("  source = {:?}", privmsg.get_source());
        println!("  target = {:?}", privmsg.get_target());
        println!("  body_raw = {:?}", privmsg.get_body_raw());
    }

    if let Ok(topic) = msg.as_tymsg::<&server::Topic>() {
        println!("found TOPIC:");
    }

    if let Ok(quit) = msg.as_tymsg::<&server::Quit>() {
        println!("found QUIT:");
    }

    if let Ok(privmsg) = msg.as_tymsg::<&server::Privmsg>() {
        println!("found PRIVMSG:");
    }
}


fn main() {
    if let Err(err) = std::panic::recover(|| run()) {
        unsafe { std::intrinsics::abort(); }
    }
}