use std::thread;
use std::time::{Duration, Instant};

const ZERO_TIME: Duration = Duration::from_secs(0);

/// `Bucket` represents a token bucket that fills at a predetermined rate. Methods on
/// `Bucket` may be called concurrently.
#[derive(Debug)]
pub struct Bucket {
    /// capacity holds the overall capacity of the bucket.
    pub capacity: u64,
    /// availableTokens holds the number of available
    /// tokens as of the associated latestTick.
    /// It will be negative when there are consumers
    /// waiting for tokens.
    pub available_tokens: u64,
    /// quantum holds how many tokens are added on
    /// each tick.
    pub quantum: u64,
    /// fillInterval holds the interval between each tick.
    pub fill_interval: Duration,
    /// latestTick holds the latest tick for which
    /// we know the number of tokens in the bucket.
    pub latest_tick: Instant,
}

impl Bucket {
    pub fn new(
        fill_interval: Duration,
        capacity: u64,
        quantum: u64,
        available_tokens: u64,
    ) -> Self {
        Self {
            capacity,
            available_tokens,
            latest_tick: Instant::now(),
            quantum,
            fill_interval,
        }
    }

    fn current_tick(&self) -> f64 {
        (self.latest_tick.elapsed().as_millis() as f64) / (self.fill_interval.as_millis() as f64)
    }

    fn adjust_available_tokens(&mut self, tick: f64) {
        self.latest_tick = Instant::now();
        if self.available_tokens >= self.capacity {
            self.available_tokens = self.capacity;
            return;
        }
        self.available_tokens += (tick * self.quantum as f64) as u64;
        if self.available_tokens >= self.capacity {
            self.available_tokens = self.capacity;
        }
    }

    /// TakeAvailable takes up to count immediately available tokens from the bucket. It
    /// returns the number of tokens removed, or zero if there are no available tokens.
    /// It does not block.
    pub fn take_available(&mut self, count: u64) -> u64 {
        if count == 0 {
            return 0;
        }
        self.adjust_available_tokens(self.current_tick());
        if self.available_tokens == 0 {
            return 0;
        }
        let mut tokens = count;
        if count > self.available_tokens {
            tokens = self.available_tokens
        }
        self.available_tokens -= tokens;
        tokens
    }

    /// TakeOneAvailable takes up a token from the bucket.
    pub fn take_one_available(&mut self) -> u64 {
        self.take_available(1)
    }

    // take is the internal version of Take - it takes the current time as
    // an argument to enable easy testing.
    fn take(&mut self, count: u64, max_wait: Duration) -> (Duration, bool) {
        if count == 0 {
            return (ZERO_TIME, true);
        }
        let tick = self.current_tick();
        self.adjust_available_tokens(tick);
        let avail = (self.available_tokens as i64) - (count as i64);
        if avail >= 0 {
            self.available_tokens = avail as u64;
            return (ZERO_TIME, true);
        }
        let end_tick = (-avail as f64) / self.quantum as f64;
        let wait_time = (self.fill_interval.as_millis() as f64) * end_tick;
        if wait_time > max_wait.as_millis() as f64 {
            return (ZERO_TIME, false);
        }
        (Duration::from_millis(wait_time as u64), true)
    }

    /// TakeMaxDuration is take, except that it will only take tokens from the
    /// bucket if the wait time for the tokens is no greater than maxWait.

    /// If it would take longer than maxWait for the tokens to become available, it does
    /// nothing and reports false, otherwise it returns the time that the caller should
    /// wait until the tokens are actually available, and reports true.
    pub fn take_max_duration(&mut self, count: u64, max_wait: Duration) -> (Duration, bool) {
        self.take(count, max_wait)
    }

    /// WaitMaxDuration is like Wait except that it will only take tokens from the
    /// bucket if it needs to wait for no greater than maxWait. It reports whether any
    /// tokens have been removed from the bucket If no tokens have been removed, it
    /// returns immediately.
    pub fn wait_max_duration(&mut self, count: u64, max_wait: Duration) -> bool {
        let (sleep_time, ok) = self.take(count, max_wait);
        if sleep_time.as_millis() > 0 {
            thread::sleep(sleep_time);
        }
        ok
    }
}

#[cfg(test)]
mod tests {
    use crate::Bucket;
    use std::thread;
    use std::time::Duration;
    #[test]
    fn take_avaliable_works() {
        let mut bucket = Bucket::new(Duration::from_secs(3), 100, 100, 100);
        let count = bucket.take_available(200);
        assert_eq!(count, 100);
        let count = bucket.take_available(100);
        assert_eq!(count, 0);
        thread::sleep(Duration::from_secs(3));
        let count = bucket.take_available(100);
        assert_eq!(count, 100);
        thread::sleep(Duration::from_secs(2));
        let count = bucket.take_available(100);
        assert_eq!(66, count);
        thread::sleep(Duration::from_secs(3));
        let count = bucket.take_available(200);
        assert_eq!(100, count);
    }

    #[test]
    fn take_max_duration_works() {
        let mut bucket = Bucket::new(Duration::from_secs(3), 100, 100, 100);
        bucket.take_available(100);
        let (time, ok) = bucket.take_max_duration(100, Duration::from_secs(4));
        assert_eq!(time.as_millis(), 3000);
        assert_eq!(ok, true);
        let (time, ok) = bucket.take_max_duration(100, Duration::from_secs(1));
        assert_eq!(time.as_millis(), 0);
        assert_eq!(ok, false);
        thread::sleep(Duration::from_secs(1));
        let (time, ok) = bucket.take_max_duration(100, Duration::from_secs(7));
        assert_eq!(time.as_secs(), 2);
        assert_eq!(ok, true);
    }
}
