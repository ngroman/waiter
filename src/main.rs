use std::time::Instant;

extern crate clap;
use clap::{App, AppSettings, Arg};

mod times;
mod waiter;

fn main() {
    let start = Instant::now();
    let matches = App::new("Waiter")
        .version("0.1")
        .author("Nathaniel Roman <ngroman@gmail.com>")
        .setting(AppSettings::TrailingVarArg)
        .about("Run a binary or example of the local package")
        .arg(Arg::with_name("duration").validator(is_dur))
        .arg(Arg::with_name("message").default_value("Done"))
        .arg(Arg::with_name("command").last(true).multiple(true))
        .arg(
            Arg::with_name("speak")
                .short("s")
                .long("speak")
                .help("Annouce audibly"),
        )
        .arg(
            Arg::with_name("pid")
                .short("p")
                .long("pid")
                .conflicts_with("command")
                .help("Wait for PID")
                .validator(is_pid)
                .takes_value(true),
        )
        .get_matches();

    let action = if let Some(dur_s) = matches.value_of("duration") {
        match times::parse_dur(dur_s) {
            Ok(dur) => waiter::Action::Wait(dur),
            Err(err) => panic!(err), // TODO
        }
    } else if let Some(pid) = matches.value_of("pid") {
        waiter::Action::WaitPid(pid.parse::<u32>().unwrap())
    } else if let Some(cmd) = matches.values_of("command") {
        waiter::Action::RunCommand(cmd.collect())
    } else {
        waiter::Action::Noop
    };
    let waiter = waiter::Waiter {
        message: String::from(matches.value_of("message").expect("no message provided")),
        action,
        speak: matches.is_present("speak"),
        start,
    };
    waiter.run();
}

fn is_pid(val: String) -> Result<(), String> {
    match val.parse::<u32>() {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("'{}' is not a valid pid", val)),
    }
}

fn is_dur(val: String) -> Result<(), String> {
    match times::parse_dur(&val) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("'{}' is not a valid duration ({})", val, err)),
    }
}
