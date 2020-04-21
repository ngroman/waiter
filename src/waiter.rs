use std::process::Command;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Waiter<'a> {
    pub message: String,
    pub action: Action<'a>,
    pub speak: bool,
    pub start: Instant,
}

#[derive(Debug)]
pub enum Action<'a> {
    Wait(Duration),
    WaitPid(u32),
    RunCommand(Vec<&'a str>),
    Noop,
}

const HOUR_IN_SECS: u64 = 60 * 60;

impl<'a> Waiter<'a> {
    pub fn run(&self) {
        match &self.action {
            Action::Wait(dur) => self.wait(*dur),
            Action::WaitPid(_pid) => eprintln!("Not implemented yet"), // TODO
            Action::RunCommand(cmd) => self.run_command(&cmd),
            Action::Noop => {}
        }
        alert(&self.message);
        if self.speak {
            say(&self.message);
        }
        eprintln!("");
    }

    fn wait(&self, dur: Duration) {
        let end = self.start + dur;
        loop {
            if self.start.elapsed() > dur {
                break;
            }
            self.print_timer(end - Instant::now(), dur);
            thread::sleep(std::cmp::min(
                Duration::from_secs_f32(0.2),
                end - Instant::now(),
            ));
        }
        self.print_timer(Duration::from_secs(0), dur);
    }

    fn run_command(&self, cmd: &[&str]) {
        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        let pair2 = pair.clone();
        let start = self.start;
        let th = thread::spawn(move || {
            let (lock, cvar) = &*pair2;
            let mut done = false;
            while !done {
                eprint!(
                    "  Executing for {}   \r",
                    Waiter::fmt_duration(Instant::now() - start)
                );
                let result = cvar
                    .wait_timeout(lock.lock().unwrap(), Duration::from_secs(1))
                    .unwrap();
                done = *result.0;
            }
        });
        Command::new(cmd[0])
            .args(cmd[1..].iter())
            .spawn()
            .expect("Failed to execute command")
            .wait()
            .expect("Failed to wait on command");
        let (lock, cvar) = &*pair;
        {
            let mut done = lock.lock().unwrap();
            *done = true;
            cvar.notify_one();
        }
        th.join().unwrap();
    }

    fn print_timer(&self, rem: Duration, total: Duration) {
        eprint!(
            "  {} {}    \r",
            Waiter::progress_bar(rem, total),
            Waiter::fmt_duration(rem)
        );
    }

    fn fmt_duration(d: Duration) -> String {
        let secs = d.as_secs();
        if secs > HOUR_IN_SECS {
            return format!(
                "{}:{:02}:{:02}",
                secs / HOUR_IN_SECS,
                secs % HOUR_IN_SECS / 60,
                secs % 60
            );
        }
        format!("{:02}:{:02}", secs / 60, secs % 60)
    }

    fn progress_bar(rem: Duration, total: Duration) -> String {
        let width = 20; // Must match format!
        let bars: usize =
            width - (width as f32 * rem.as_secs_f32() / total.as_secs_f32()).floor() as usize;

        format!(
            "[{}{}]",
            std::iter::repeat('#').take(bars).collect::<String>(),
            std::iter::repeat('-')
                .take(width - bars)
                .collect::<String>()
        )
    }
}

#[cfg(not(target_os = "macos"))]
fn alert(_msg: &str) {}

#[cfg(target_os = "macos")]
fn alert(msg: &str) {
    Command::new("osascript")
        .arg("-e")
        .arg(format!(
            "display notification \"{}\" with title \"waiter\"",
            msg
        ))
        .output()
        .expect("osascript failed");
}

#[cfg(not(target_os = "macos"))]
fn say(_msg: &str) {
    beep();
}

#[cfg(target_os = "macos")]
fn say(msg: &str) {
    let mut child = Command::new("say").arg(msg).spawn().unwrap();
    let start = Instant::now();
    loop {
        if start.elapsed() > Duration::from_secs(5) {
            eprintln!("`say` timeout");
            beep();
            let _ = child.kill();
            break;
        }
        if let Some(_) = child.try_wait().unwrap() {
            break;
        }
        thread::sleep(Duration::from_secs_f32(0.1));
    }
}

fn beep() {
    eprintln!("\x07"); // Bell character
    thread::sleep(Duration::from_secs_f32(0.4));
    eprintln!("\x07");
}
