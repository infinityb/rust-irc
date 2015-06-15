rust-irc
========
[![Build Status](https://travis-ci.org/infinityb/rust-irc.svg?branch=master)](https://travis-ci.org/infinityb/rust-irc)

[Documentation](http://infinityb.github.io/rust-irc)

Safe API for parsing and creating IRC commands/messages.

## Server Message types
* JOIN
* KICK
* MODE
* NICK
* NOTICE
* PART
* PING
* PONG
* PRIVMSG
* QUIT
* TOPIC
* INVITE

## Client Message types
* JOIN
* NICK
* PING
* PONG
* PRIVMSG
* USER
* WHO
* QUIT

Current State
=============
Work in progress. Lots of unsupported message types.

See [infinityb/rust-irc-mio](https://github.com/infinityb/rust-irc-mio) for integration with [carllerche/mio](https://github.com/carllerche/mio).

Event-loop specific implementations will most likely be implemented in separate crates eventually.

Periodically fuzzed with AFL using [kmcallister/afl.rs](https://github.com/kmcallister/afl.rs).

License
=======
This library is distributed under similar terms to Rust: dual licensed under
the MIT license and the Apache license (version 2.0).

See LICENSE-APACHE, LICENSE-MIT, and COPYRIGHT for details.
