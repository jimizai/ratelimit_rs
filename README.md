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

```
  bucket.take_available(count: u64) -> u64;
```

TakeAvailable takes up to count immediately available tokens from the bucket. It
returns the number of tokens removed, or zero if there are no available tokens.
It does not block.
