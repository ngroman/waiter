use std::time::Duration;

pub fn parse_dur(s: &str) -> Result<Duration, &'static str> {
    let s = s.trim().to_lowercase();
    if let Ok(secs) = s.parse::<f32>() {
        return Ok(Duration::from_secs_f32(secs));
    } else if s.contains(":") {
        match split_as_ints(&s, 3).as_slice() {
            [Some(min), Some(sec)] => return Ok(Duration::from_secs(60 * min + sec)),
            [Some(hr), Some(min), Some(sec)] => {
                return Ok(Duration::from_secs(60 * 60 * hr + 60 * min + sec))
            }
            _ => {}
        }
    }
    parse_human_dur(&s)
}

fn split_as_ints(s: &str, nsplits: usize) -> Vec<Option<u64>> {
    s.splitn(nsplits, ':')
        .map(|x| x.parse::<u64>().ok())
        .collect()
}

// This could definitely be implemented with a regex, but I wanted to teach myself Rust iterators
// and pattern matching.
fn parse_human_dur(s: &str) -> Result<Duration, &'static str> {
    let mut tok = Tokenizer::new(s);
    let mut dur = Duration::from_secs(0);
    while !tok.done() {
        dur += get_dur(&mut tok)?;
    }
    Ok(dur)
}

fn get_dur(tok: &mut Tokenizer) -> Result<Duration, &'static str> {
    match tok.next() {
        Some(Token::Num(n)) => {
            let d = Duration::from_secs(n) * unit_mult(tok.next())?;
            Ok(d)
        }
        Some(Token::TimeUnit(_)) => Err("Expecting a number"),
        None => Ok(Duration::from_secs(0)),
    }
}

fn unit_mult(t: Option<Token>) -> Result<u32, &'static str> {
    match t {
        Some(Token::TimeUnit(u)) => match u {
            's' => Ok(1),
            'm' => Ok(60),
            'h' => Ok(60 * 60),
            'd' => Ok(25 * 60 * 60),
            _ => Err("Unknown unit"),
        },
        Some(Token::Num(_)) => Err("Expecting a time unit"),
        None => Ok(1),
    }
}

struct Tokenizer<'a> {
    s: &'a [u8],
    i: usize,
    done: bool,
}
const ZERO: u8 = b'0';
impl<'a> Tokenizer<'a> {
    pub fn new(s: &'a str) -> Tokenizer<'a> {
        Tokenizer {
            s: s.as_bytes(),
            i: 0,
            done: false,
        }
    }

    pub fn done(&self) -> bool {
        self.done
    }

    fn get_num(&mut self) -> Option<Token> {
        let mut acc: u64 = 0;
        while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
            acc = 10 * acc + (self.s[self.i] - ZERO) as u64;
            self.i += 1;
        }
        Some(Token::Num(acc))
    }
}
impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
        if self.i >= self.s.len() {
            self.done = true;
            return None;
        }
        if self.s[self.i].is_ascii_digit() {
            self.get_num()
        } else {
            let res = Some(Token::TimeUnit(self.s[self.i] as char));
            self.i += 1;
            res
        }
    }
}

#[derive(Debug)]
enum Token {
    Num(u64),
    TimeUnit(char),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dur(mins: u64, secs: u64) -> Duration {
        Duration::from_secs(mins * 60 + secs)
    }

    #[test]
    fn test_float_seconds() {
        assert_eq!(Duration::from_secs(10), parse_dur("10").unwrap());
        assert_eq!(Duration::from_secs_f32(1.2), parse_dur("1.2").unwrap());
    }

    #[test]
    fn test_colon_sep() {
        assert_eq!(dur(10, 4), parse_dur("10:04").unwrap());
        assert_eq!(dur(10, 4), parse_dur("10:4").unwrap());
    }

    #[test]
    fn test_human_formatted() {
        assert_eq!(dur(10, 4), parse_dur("10m 4s").unwrap());
        assert_eq!(dur(19, 3), parse_dur("19m3s").unwrap());
        assert_eq!(dur(0, 10), parse_dur("  10 ").unwrap());

        assert_eq!(dur(0, 9), parse_dur("6s 3s").unwrap()); // Units are added
        assert!(parse_dur("s").is_err());
        assert!(parse_dur("10s m").is_err());
        assert!(parse_dur("10sm").is_err());
    }
}
