Changelog
=========

0.5.0
-----

IrcMsg is being replaced with a new slice-based implementation and
and an associated owned type `IrcMsgBuf`, similar to how `&Path` and
`PathBuf` work.  This will allow for much more flexibility in how
IrcMsgs are passed around.

The old IrcMsg types will remain available for a while at `irc::parse::IrcMsg`.
