rust-irc
========
[![Build Status](https://travis-ci.org/infinityb/rust-irc.svg?branch=master)](https://travis-ci.org/infinityb/rust-irc)

Parsing IRC messages and maybe a bit more!

This code-base is in the process of being split into two projects, the
IRC protocol and the IRC bot.  The IRC bot repository has moved [Here](https://github.com/infinityb/rust-irc-bot).

Current State
=============
Work in progress. Lots of unsupported stuff.

Not really ready for the real world.  Usable in controlled circumstances where
the server won't send you broken messages.  Broken messages will most likely
result in fail!() at present.  This is considered a bug.
