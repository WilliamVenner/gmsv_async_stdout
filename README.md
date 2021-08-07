# gmsv_async_stdout

This is a Garry's Mod server module that moves `-condebug` file I/O out of the main thread, which should significantly improve performance for noisy servers.

**NOTE: This module does nothing if `-condebug` is not enabled in your server startup parameters. If your host doesn't add `-condebug` but still provides a web console (e.g. Pterodactyl panel hosts), you DON'T NEED this module.**

## What?

Many server hosts use the startup option `-condebug` for their "Web Console" feature. Namely, TCAdmin is one server control panel that does this.

I don't believe `-condebug` was ever intended to be used this way. There are plenty of much superior ways of reading the standard output of a program. Anyhow, it's 2021 and there are still hosts and control panels that do this.

What's wrong with that then? Well, what `-condebug` does is, every time a message is printed to the SRCDS console output, it blocks the main thread to append to the `console.log` file and flushes the file. Every. Single. Message!

This module overrides that behaviour and does it in a separate thread instead. This allows the main thread to continue whatever it was doing whilst it was printing a console message and not have to deal with any file I/O.

## Drawbacks

During a server crash, your `console.log` file may be missing some of the messages that occurred before the crash.

In most cases, the messages that occur before a crash are useless anyway. If you don't believe they are, this plugin probably isn't a great idea to use. Maybe consider using a better server host that reads the console output properly instead!

# Installation

The module shouldn't be loaded until the first player joins as you won't get any performance benefit during server startup, really, and you'd probably rather want to see as many messages during a server crash as possible at this point.

1. Go to the [releases](https://github.com/WilliamVenner/gmsv_async_stdout/releases) page and download the right module for your server's operating system and branch.

If you don't know which one that is, run this in your server's console:

```lua
lua_run print("gmsv_async_stdout_"..(system.IsWindows()and"win"or system.IsLinux()and"linux"or"UNSUPPORTED")..(jit.arch=="x64"and"64"or(system.IsLinux()and""or"32"))..".dll")
```

2. Drop the DLL file in `garrysmod/lua/bin/` (if the folder doesn't exist, create it)

If you try and upload the DLL file over FTP and it says access denied, it means your host doesn't let you upload DLL files and you need to ask them to install it themselves.

3. Download [async_stdout.lua](https://raw.githubusercontent.com/WilliamVenner/gmsv_async_stdout/master/src/async_stdout.lua) and drop it in `lua/autorun/server/`
