# waiter
Audible alerts after commands

## Basic Usage
Audibly say "done" (on a mac, terminal beep otherwise) after running long command:
`./scripts/my-super-long-command; waiter -s`

## Install
`cargo install --path path/to/waiter`


### Developer's note
I used this as a first project to learn the basic of Rust. Because of this certain parts are over-engineered while others are under-engineered.


## Help
```
Utility to provide notifications after long commands

USAGE:
    waiter [FLAGS] [OPTIONS] [duration] [-- <command>...]

FLAGS:
    -h, --help       Prints help information
    -s, --speak      Audibly announce that the command is complete with terminal beep or `say` command (Mac only)
    -V, --version    Prints version information

OPTIONS:
    -m, --message <message>    Message to say when complete [default: Done]
    -p, --pid <pid>            Wait for PID

ARGS:
    <duration>      Duration to wait in seconds or human-readable units (e.g. '6h 10m3s')
    <command>...
```
