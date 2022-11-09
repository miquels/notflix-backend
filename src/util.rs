pub trait SystemTimeToUnixTime {
    fn unixtime(&self) -> i64;
    fn unixtime_ms(&self) -> i64;
    fn unixtime_ns(&self) -> i64;
}

impl SystemTimeToUnixTime for std::time::SystemTime {
    fn unixtime(&self) -> i64 {
        match self.duration_since(std::time::SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_secs().try_into().unwrap_or(i64::MAX), 
            Err(t) => t.duration().as_secs().try_into().map(|t: i64| -t).unwrap_or(i64::MIN),
        }
    }

    fn unixtime_ms(&self) -> i64 {
        match self.duration_since(std::time::SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_millis().try_into().unwrap_or(i64::MAX), 
            Err(t) => t.duration().as_millis().try_into().map(|t: i64| -t).unwrap_or(i64::MIN),
        }
    }

    fn unixtime_ns(&self) -> i64 {
        match self.duration_since(std::time::SystemTime::UNIX_EPOCH) {
            Ok(n) => n.as_nanos().try_into().unwrap_or(i64::MAX), 
            Err(t) => t.duration().as_nanos().try_into().map(|t: i64| -t).unwrap_or(i64::MIN),
        }
    }
}
