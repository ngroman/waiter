use std::time::Instant;

extern crate clap;
use clap::{App, AppSettings, Arg};

mod times;
mod waiter;

fn main() {
    let start = Instant::now();
    let app = App::new("Waiter")
        .version("0.2")
        .author("Nathaniel Roman <ngroman@gmail.com>")
        .setting(AppSettings::TrailingVarArg)
        .about("Utility to provide notifications after long commands")
        .arg(
            Arg::with_name("duration")
                .validator(is_dur)
                .help("Duration to wait in seconds or human-readable units (e.g. '6h 10m3s')"),
        )
        .arg(
            Arg::with_name("message")
                .short("m")
                .long("message")
                .help("Message to say when complete")
                .default_value("Done"),
        )
        .arg(Arg::with_name("command").last(true).multiple(true))
        .arg(
            Arg::with_name("speak")
                .short("s")
                .long("speak")
                .help("Audibly announce that the command is complete with terminal beep or `say` command (Mac only)"),
        );
    let app = pid_arg(app);
    let matches = app.get_matches();

    let action = if let Some(dur_s) = matches.value_of("duration") {
        waiter::Action::Wait(times::parse_dur(dur_s).unwrap())
    } else if let Some(pid) = matches.value_of("pid") {
        waiter::Action::WaitPid(pid.parse::<i32>().unwrap())
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

fn pid_arg<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    if cfg!(unix) {
        return app.arg(
            Arg::with_name("pid")
                .short("p")
                .long("pid")
                .conflicts_with("command")
                .help("Wait for PID")
                .validator(is_pid)
                .takes_value(true),
        );
    }
    app
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
