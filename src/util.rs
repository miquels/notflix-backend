use std::time::{Duration, SystemTime};
use std::fmt;
use std::ops::Deref;

use rand::Rng;

use crate::sqlx::impl_sqlx_traits_for;

pub trait SystemTimeToUnixTime {
    fn unixtime(&self) -> i64;
    fn unixtime_ms(&self) -> i64;
    fn unixtime_ns(&self) -> i64;
}

impl SystemTimeToUnixTime for SystemTime {
    fn unixtime(&self) -> i64 {
        match self.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs().try_into().unwrap_or(i64::MAX), 
            Err(t) => t.duration().as_secs().try_into().map(|t: i64| -t).unwrap_or(i64::MIN),
        }
    }

    fn unixtime_ms(&self) -> i64 {
        match self.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_millis().try_into().unwrap_or(i64::MAX), 
            Err(t) => t.duration().as_millis().try_into().map(|t: i64| -t).unwrap_or(i64::MIN),
        }
    }

    fn unixtime_ns(&self) -> i64 {
        match self.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_nanos().try_into().unwrap_or(i64::MAX), 
            Err(t) => t.duration().as_nanos().try_into().map(|t: i64| -t).unwrap_or(i64::MIN),
        }
    }
}

// Blech, sqlx re-exports an ancient version of Chrono, and it doesn't
// even export it completely - for example, you can't get at `Duration`.
// So implement our own Rfc3339Time, based on humantime_serde which
// we need anyway.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub struct Rfc3339Time(pub humantime_serde::Serde<SystemTime>);
impl_sqlx_traits_for!(Rfc3339Time, text);

impl Rfc3339Time {
    pub fn new(tm: SystemTime) -> Rfc3339Time {
        // Round to a second.
        let tm = match tm.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(d) => tm - Duration::from_nanos((d.as_nanos() % 1_000_000_000) as u64),
            Err(_) => tm,
        };
        Rfc3339Time(tm.into())
    }

    pub fn as_systemtime(&self) -> SystemTime {
        *self.deref()
    }
}

impl Deref for Rfc3339Time {
    type Target = SystemTime;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Rfc3339Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        serde_plain::to_string(&self.0).map_err(|_| fmt::Error)?.fmt(f)
    }
}

#[derive(serde::Serialize, serde::Deserialize, PartialEq, Clone)]
#[serde(transparent)]
pub struct Id(String);

impl Id {
    /// Generate a base62 encoded unique (random) id of `len` characters.
    /// Maximum length is 22, which can encode 128 bits of data.
    /// Length 20 contains about 125 bits of random data.
    pub fn new_with_len(len: usize) -> Id {
        let mut id = String::new();

        let len = std::cmp::min(len, 22);
        let max = if len < 22 {
            62u128.pow(len as u32)
        } else {
            u128::MAX
        };
        let mut rng = rand::thread_rng();
        let mut n = rng.gen_range(0 ..= max);

        // into base62.
        for _ in 0 .. len {
            let m = (n % 62) as u8;
            n /= 62;
            let c = match m {
                0 ..= 9 => m + b'0',
                10 ..= 35 => m - 10 + b'A',
                36 ..= 61 => m - 36 + b'a',
                _ => unreachable!(),
            };
            id.push(c.into());
        }

        Id(id)
    }

    /// Default length is 20.
    pub fn new() -> Id {
        Id::new_with_len(20)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

impl Deref for Id {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

macro_rules! ok_or_return {
    ($expr:expr, |$var:tt| $($code:tt)*) => {
        match $expr {
            Ok(expr) => expr,
            Err(e) => {
                let $var = e;
                #[allow(unreachable_code)]
                return { $($code)* }
            }
        }
    }
}
pub(crate) use ok_or_return;

macro_rules! some_or_return {
    ($expr:expr, $($code:tt)*) => {
        match $expr {
            Some(expr) => expr,
            None => return $($code)*,
        }
    }
}
pub(crate) use some_or_return;
