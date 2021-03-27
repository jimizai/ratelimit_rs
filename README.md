# ratelimit_rs

--
use ratelimit::Bucket;

The ratelimit package provides an efficient token bucket implementation. See
http://en.wikipedia.org/wiki/Token_bucket.

## Usage

#### struct Bucket

Bucket represents a token bucket that fills at a predetermined rate. Methods on
Bucket may be called concurrently.

```rust
  let bucket = Bucket::new(fill_interval: Duration, capacity: u64, quantum: u64, available_tokens: u64);
```

Bucket::new returns a new token bucket that fills at the rate of one token every
fillInterval, up to the given maximum capacity. Both arguments must be positive.
The bucket is initially full.

#### fn take_available

```rust
  bucket.take_available(count: u64) -> u64;
```

TakeAvailable takes up to count immediately available tokens from the bucket. It
returns the number of tokens removed, or zero if there are no available tokens.
It does not block.

#### fn take_max_duration

```rust
  bucket.take_max_duration(count: u64, max_wait: Duration) -> (Duration, bool);
```

TakeMaxDuration is take, except that it will only take tokens from the
bucket if the wait time for the tokens is no greater than maxWait.

If it would take longer than maxWait for the tokens to become available, it does
nothing and reports false, otherwise it returns the time that the caller should
wait until the tokens are actually available, and reports true.

#### fn wait_max_duration

```rust
  bucket.wait_max_duration(count: u64, max_wait: Duration) -> bool;
```

WaitMaxDuration is like Wait except that it will only take tokens from the
bucket if it needs to wait for no greater than maxWait. It reports whether any
tokens have been removed from the bucket If no tokens have been removed, it
returns immediately.
