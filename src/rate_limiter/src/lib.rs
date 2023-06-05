// Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#![deny(missing_docs)]

//! # Rate Limiter
//!
//! Provides a rate limiter written in Rust useful for IO operations that need to
//! be throttled.
//!
//! ## Behavior
//!
//! The rate limiter starts off as 'unblocked' with two token buckets configured
//! with the values passed in the `RateLimiter::new()` constructor.
//! All subsequent accounting is done independently for each token bucket based
//! on the `TokenType` used. If any of the buckets runs out of budget, the limiter
//! goes in the 'blocked' state. At this point an internal timer is set up which
//! will later 'wake up' the user in order to retry sending data. The 'wake up'
//! notification will be dispatched as an event on the FD provided by the `AsRawFD`
//! trait implementation.
//!
//! The contract is that the user shall also call the `event_handler()` method on
//! receipt of such an event.
//!
//! The token buckets are replenished when a called `consume()` doesn't find enough
//! tokens in the bucket. The amount of tokens replenished is automatically calculated
//! to respect the `complete_refill_time` configuration parameter provided by the user.
//! The token buckets will never replenish above their respective `size`.
//!
//! Each token bucket can start off with a `one_time_burst` initial extra capacity
//! on top of their `size`. This initial extra credit does not replenish and
//! can be used for an initial burst of data.
//!
//! The granularity for 'wake up' events when the rate limiter is blocked is
//! currently hardcoded to `100 milliseconds`.
//!
//! ## Limitations
//!
//! This rate limiter implementation relies on the *Linux kernel's timerfd* so its
//! usage is limited to Linux systems.
//!
//! Another particularity of this implementation is that it is not self-driving.
//! It is meant to be used in an external event loop and thus implements the `AsRawFd`
//! trait and provides an *event-handler* as part of its API. This *event-handler*
//! needs to be called by the user on every event on the rate limiter's `AsRawFd` FD.
use std::os::unix::io::{AsRawFd, RawFd};
use std::time::{Duration, Instant};
use std::{fmt, io};

use timerfd::{ClockId, SetTimeFlags, TimerFd, TimerState};

pub mod persist;

#[derive(Debug)]
/// Describes the errors that may occur while handling rate limiter events.
pub enum Error {
    /// The event handler was called spuriously.
    SpuriousRateLimiterEvent(&'static str),
}

// Interval at which the refill timer will run when limiter is at capacity.
const REFILL_TIMER_INTERVAL_MS: u64 = 100;
const TIMER_REFILL_STATE: TimerState =
    TimerState::Oneshot(Duration::from_millis(REFILL_TIMER_INTERVAL_MS));

const NANOSEC_IN_ONE_MILLISEC: u64 = 1_000_000;

// Euclid's two-thousand-year-old algorithm for finding the greatest common divisor.
#[tracing::instrument(level = "trace", ret)]
fn gcd(x: u64, y: u64) -> u64 {
    let mut x = x;
    let mut y = y;
    while y != 0 {
        let t = y;
        y = x % y;
        x = t;
    }
    x
}

/// Enum describing the outcomes of a `reduce()` call on a `TokenBucket`.
#[derive(Clone, Debug, PartialEq)]
pub enum BucketReduction {
    /// There are not enough tokens to complete the operation.
    Failure,
    /// A part of the available tokens have been consumed.
    Success,
    /// A number of tokens `inner` times larger than the bucket size have been consumed.
    OverConsumption(f64),
}

/// TokenBucket provides a lower level interface to rate limiting with a
/// configurable capacity, refill-rate and initial burst.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenBucket {
    // Bucket defining traits.
    size: u64,
    // Initial burst size.
    initial_one_time_burst: u64,
    // Complete refill time in milliseconds.
    refill_time: u64,

    // Internal state descriptors.

    // Number of free initial tokens, that can be consumed at no cost.
    one_time_burst: u64,
    // Current token budget.
    budget: u64,
    // Last time this token bucket saw activity.
    last_update: Instant,

    // Fields used for pre-processing optimizations.
    processed_capacity: u64,
    processed_refill_time: u64,
}

impl TokenBucket {
    /// Creates a `TokenBucket` wrapped in an `Option`.
    ///
    /// TokenBucket created is of `size` total capacity and takes `complete_refill_time_ms`
    /// milliseconds to go from zero tokens to total capacity. The `one_time_burst` is initial
    /// extra credit on top of total capacity, that does not replenish and which can be used
    /// for an initial burst of data.
    ///
    /// If the `size` or the `complete refill time` are zero, then `None` is returned.
    #[tracing::instrument(level = "trace", ret)]
    pub fn new(size: u64, one_time_burst: u64, complete_refill_time_ms: u64) -> Option<Self> {
        // If either token bucket capacity or refill time is 0, disable limiting.
        if size == 0 || complete_refill_time_ms == 0 {
            return None;
        }
        // Formula for computing current refill amount:
        // refill_token_count = (delta_time * size) / (complete_refill_time_ms * 1_000_000)
        // In order to avoid overflows, simplify the fractions by computing greatest common divisor.

        let complete_refill_time_ns =
            complete_refill_time_ms.checked_mul(NANOSEC_IN_ONE_MILLISEC)?;
        // Get the greatest common factor between `size` and `complete_refill_time_ns`.
        let common_factor = gcd(size, complete_refill_time_ns);
        // The division will be exact since `common_factor` is a factor of `size`.
        let processed_capacity: u64 = size / common_factor;
        // The division will be exact since `common_factor` is a factor of
        // `complete_refill_time_ns`.
        let processed_refill_time: u64 = complete_refill_time_ns / common_factor;

        Some(TokenBucket {
            size,
            one_time_burst,
            initial_one_time_burst: one_time_burst,
            refill_time: complete_refill_time_ms,
            // Start off full.
            budget: size,
            // Last updated is now.
            last_update: Instant::now(),
            processed_capacity,
            processed_refill_time,
        })
    }

    // Replenishes token bucket based on elapsed time. Should only be called internally by `Self`.
    fn auto_replenish(&mut self) {
        // Compute time passed since last refill/update.
        let now = Instant::now();
        let time_delta = (now - self.last_update).as_nanos();

        if time_delta >= u128::from(self.refill_time * NANOSEC_IN_ONE_MILLISEC) {
            self.budget = self.size;
            self.last_update = now;
        } else {
            // At each 'time_delta' nanoseconds the bucket should refill with:
            // refill_amount = (time_delta * size) / (complete_refill_time_ms * 1_000_000)
            // `processed_capacity` and `processed_refill_time` are the result of simplifying above
            // fraction formula with their greatest-common-factor.

            // In the constructor, we assured that (self.refill_time * NANOSEC_IN_ONE_MILLISEC)
            // fits into a u64 That means, at this point we know that time_delta <
            // u64::MAX. Since all other values here are u64, this assures that u128
            // multiplication cannot overflow.
            let processed_capacity = u128::from(self.processed_capacity);
            let processed_refill_time = u128::from(self.processed_refill_time);

            let tokens = (time_delta * processed_capacity) / processed_refill_time;

            // We increment `self.last_update` by the minimum time required to generate `tokens`, in
            // the case where we have the time to generate `1.8` tokens but only
            // generate `x` tokens due to integer arithmetic this will carry the time
            // required to generate 0.8th of a token over to the next call, such that if
            // the next call where to generate `2.3` tokens it would instead
            // generate `3.1` tokens. This minimizes dropping tokens at high frequencies.
            // We want the integer division here to round up instead of down (as if we round down,
            // we would allow some fraction of a nano second to be used twice, allowing
            // for the generation of one extra token in extreme circumstances).
            let mut time_adjustment = tokens * processed_refill_time / processed_capacity;
            if tokens * processed_refill_time % processed_capacity != 0 {
                time_adjustment += 1;
            }

            // Ensure that we always generate as many tokens as we can: assert that the "unused"
            // part of time_delta is less than the time it would take to generate a
            // single token (= processed_refill_time / processed_capacity)
            debug_assert!(time_adjustment <= time_delta);
            debug_assert!(
                (time_delta - time_adjustment) * processed_capacity <= processed_refill_time
            );

            // time_adjustment is at most time_delta, and since time_delta <= u64::MAX, this cast is
            // fine
            self.last_update += Duration::from_nanos(time_adjustment as u64);
            self.budget = std::cmp::min(self.budget.saturating_add(tokens as u64), self.size);
        }
    }

    /// Attempts to consume `tokens` from the bucket and returns whether the action succeeded.
    #[tracing::instrument(level = "trace", ret)]
    pub fn reduce(&mut self, mut tokens: u64) -> BucketReduction {
        // First things first: consume the one-time-burst budget.
        if self.one_time_burst > 0 {
            // We still have burst budget for *all* tokens requests.
            if self.one_time_burst >= tokens {
                self.one_time_burst -= tokens;
                self.last_update = Instant::now();
                // No need to continue to the refill process, we still have burst budget to consume
                // from.
                return BucketReduction::Success;
            } else {
                // We still have burst budget for *some* of the tokens requests.
                // The tokens left unfulfilled will be consumed from current `self.budget`.
                tokens -= self.one_time_burst;
                self.one_time_burst = 0;
            }
        }

        if tokens > self.budget {
            // Hit the bucket bottom, let's auto-replenish and try again.
            self.auto_replenish();

            // This operation requests a bandwidth higher than the bucket size
            if tokens > self.size {
                tracing::error!(
                    "Consumed {} tokens from bucket of size {}",
                    tokens,
                    self.size
                );
                // Empty the bucket and report an overconsumption of
                // (remaining tokens / size) times larger than the bucket size
                tokens -= self.budget;
                self.budget = 0;
                return BucketReduction::OverConsumption(tokens as f64 / self.size as f64);
            }

            if tokens > self.budget {
                // Still not enough tokens, consume() fails, return false.
                return BucketReduction::Failure;
            }
        }

        self.budget -= tokens;
        BucketReduction::Success
    }

    /// "Manually" adds tokens to bucket.
    #[tracing::instrument(level = "trace", ret)]
    pub fn force_replenish(&mut self, tokens: u64) {
        // This means we are still during the burst interval.
        // Of course there is a very small chance  that the last reduce() also used up burst
        // budget which should now be replenished, but for performance and code-complexity
        // reasons we're just gonna let that slide since it's practically inconsequential.
        if self.one_time_burst > 0 {
            self.one_time_burst = std::cmp::min(
                self.one_time_burst.saturating_add(tokens),
                self.initial_one_time_burst,
            );
            return;
        }
        self.budget = std::cmp::min(self.budget.saturating_add(tokens), self.size);
    }

    /// Returns the capacity of the token bucket.
    #[tracing::instrument(level = "trace", ret)]
    pub fn capacity(&self) -> u64 {
        self.size
    }

    /// Returns the remaining one time burst budget.
    #[tracing::instrument(level = "trace", ret)]
    pub fn one_time_burst(&self) -> u64 {
        self.one_time_burst
    }

    /// Returns the time in milliseconds required to to completely fill the bucket.
    #[tracing::instrument(level = "trace", ret)]
    pub fn refill_time_ms(&self) -> u64 {
        self.refill_time
    }

    /// Returns the current budget (one time burst allowance notwithstanding).
    #[tracing::instrument(level = "trace", ret)]
    pub fn budget(&self) -> u64 {
        self.budget
    }

    /// Returns the initially configured one time burst budget.
    #[tracing::instrument(level = "trace", ret)]
    pub fn initial_one_time_burst(&self) -> u64 {
        self.initial_one_time_burst
    }
}

/// Enum that describes the type of token used.
#[derive(Debug)]
pub enum TokenType {
    /// Token type used for bandwidth limiting.
    Bytes,
    /// Token type used for operations/second limiting.
    Ops,
}

/// Enum that describes the type of token bucket update.
#[derive(Debug)]
pub enum BucketUpdate {
    /// No Update - same as before.
    None,
    /// Rate Limiting is disabled on this bucket.
    Disabled,
    /// Rate Limiting enabled with updated bucket.
    Update(TokenBucket),
}

/// Rate Limiter that works on both bandwidth and ops/s limiting.
///
/// Bandwidth (bytes/s) and ops/s limiting can be used at the same time or individually.
///
/// Implementation uses a single timer through TimerFd to refresh either or
/// both token buckets.
///
/// Its internal buckets are 'passively' replenished as they're being used (as
/// part of `consume()` operations).
/// A timer is enabled and used to 'actively' replenish the token buckets when
/// limiting is in effect and `consume()` operations are disabled.
///
/// RateLimiters will generate events on the FDs provided by their `AsRawFd` trait
/// implementation. These events are meant to be consumed by the user of this struct.
/// On each such event, the user must call the `event_handler()` method.
pub struct RateLimiter {
    bandwidth: Option<TokenBucket>,
    ops: Option<TokenBucket>,

    timer_fd: TimerFd,
    // Internal flag that quickly determines timer state.
    timer_active: bool,
}

impl PartialEq for RateLimiter {
    fn eq(&self, other: &RateLimiter) -> bool {
        self.bandwidth == other.bandwidth && self.ops == other.ops
    }
}

impl fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "RateLimiter {{ bandwidth: {:?}, ops: {:?} }}",
            self.bandwidth, self.ops
        )
    }
}

impl RateLimiter {
    /// Creates a new Rate Limiter that can limit on both bytes/s and ops/s.
    ///
    /// # Arguments
    ///
    /// * `bytes_total_capacity` - the total capacity of the `TokenType::Bytes` token bucket.
    /// * `bytes_one_time_burst` - initial extra credit on top of `bytes_total_capacity`,
    /// that does not replenish and which can be used for an initial burst of data.
    /// * `bytes_complete_refill_time_ms` - number of milliseconds for the `TokenType::Bytes`
    /// token bucket to go from zero Bytes to `bytes_total_capacity` Bytes.
    /// * `ops_total_capacity` - the total capacity of the `TokenType::Ops` token bucket.
    /// * `ops_one_time_burst` - initial extra credit on top of `ops_total_capacity`,
    /// that does not replenish and which can be used for an initial burst of data.
    /// * `ops_complete_refill_time_ms` - number of milliseconds for the `TokenType::Ops` token
    /// bucket to go from zero Ops to `ops_total_capacity` Ops.
    ///
    /// If either bytes/ops *size* or *refill_time* are **zero**, the limiter
    /// is **disabled** for that respective token type.
    ///
    /// # Errors
    ///
    /// If the timerfd creation fails, an error is returned.
    #[tracing::instrument(level = "trace", ret)]
    pub fn new(
        bytes_total_capacity: u64,
        bytes_one_time_burst: u64,
        bytes_complete_refill_time_ms: u64,
        ops_total_capacity: u64,
        ops_one_time_burst: u64,
        ops_complete_refill_time_ms: u64,
    ) -> io::Result<Self> {
        let bytes_token_bucket = TokenBucket::new(
            bytes_total_capacity,
            bytes_one_time_burst,
            bytes_complete_refill_time_ms,
        );

        let ops_token_bucket = TokenBucket::new(
            ops_total_capacity,
            ops_one_time_burst,
            ops_complete_refill_time_ms,
        );

        // We'll need a timer_fd, even if our current config effectively disables rate limiting,
        // because `Self::update_buckets()` might re-enable it later, and we might be
        // seccomp-blocked from creating the timer_fd at that time.
        let timer_fd = TimerFd::new_custom(ClockId::Monotonic, true, true)?;

        Ok(RateLimiter {
            bandwidth: bytes_token_bucket,
            ops: ops_token_bucket,
            timer_fd,
            timer_active: false,
        })
    }

    // Arm the timer of the rate limiter with the provided `TimerState`.
    fn activate_timer(&mut self, timer_state: TimerState) {
        // Register the timer; don't care about its previous state
        self.timer_fd.set_state(timer_state, SetTimeFlags::Default);
        self.timer_active = true;
    }

    /// Attempts to consume tokens and returns whether that is possible.
    ///
    /// If rate limiting is disabled on provided `token_type`, this function will always succeed.
    #[tracing::instrument(level = "trace", ret)]
    pub fn consume(&mut self, tokens: u64, token_type: TokenType) -> bool {
        // If the timer is active, we can't consume tokens from any bucket and the function fails.
        if self.timer_active {
            return false;
        }

        // Identify the required token bucket.
        let token_bucket = match token_type {
            TokenType::Bytes => self.bandwidth.as_mut(),
            TokenType::Ops => self.ops.as_mut(),
        };
        // Try to consume from the token bucket.
        if let Some(bucket) = token_bucket {
            let refill_time = bucket.refill_time_ms();
            match bucket.reduce(tokens) {
                // When we report budget is over, there will be no further calls here,
                // register a timer to replenish the bucket and resume processing;
                // make sure there is only one running timer for this limiter.
                BucketReduction::Failure => {
                    if !self.timer_active {
                        self.activate_timer(TIMER_REFILL_STATE);
                    }
                    false
                }
                // The operation succeeded and further calls can be made.
                BucketReduction::Success => true,
                // The operation succeeded as the tokens have been consumed
                // but the timer still needs to be armed.
                BucketReduction::OverConsumption(ratio) => {
                    // The operation "borrowed" a number of tokens `ratio` times
                    // greater than the size of the bucket, and since it takes
                    // `refill_time` milliseconds to fill an empty bucket, in
                    // order to enforce the bandwidth limit we need to prevent
                    // further calls to the rate limiter for
                    // `ratio * refill_time` milliseconds.
                    #[allow(clippy::cast_sign_loss)] // ratio is always positive
                    self.activate_timer(TimerState::Oneshot(Duration::from_millis(
                        (ratio * refill_time as f64) as u64,
                    )));
                    true
                }
            }
        } else {
            // If bucket is not present rate limiting is disabled on token type,
            // consume() will always succeed.
            true
        }
    }

    /// Adds tokens of `token_type` to their respective bucket.
    ///
    /// Can be used to *manually* add tokens to a bucket. Useful for reverting a
    /// `consume()` if needed.
    #[tracing::instrument(level = "trace", ret)]
    pub fn manual_replenish(&mut self, tokens: u64, token_type: TokenType) {
        // Identify the required token bucket.
        let token_bucket = match token_type {
            TokenType::Bytes => self.bandwidth.as_mut(),
            TokenType::Ops => self.ops.as_mut(),
        };
        // Add tokens to the token bucket.
        if let Some(bucket) = token_bucket {
            bucket.force_replenish(tokens);
        }
    }

    /// Returns whether this rate limiter is blocked.
    ///
    /// The limiter 'blocks' when a `consume()` operation fails because there was not enough
    /// budget for it.
    /// An event will be generated on the exported FD when the limiter 'unblocks'.
    #[tracing::instrument(level = "trace", ret)]
    pub fn is_blocked(&self) -> bool {
        self.timer_active
    }

    /// This function needs to be called every time there is an event on the
    /// FD provided by this object's `AsRawFd` trait implementation.
    ///
    /// # Errors
    ///
    /// If the rate limiter is disabled or is not blocked, an error is returned.
    #[tracing::instrument(level = "trace", ret)]
    pub fn event_handler(&mut self) -> Result<(), Error> {
        match self.timer_fd.read() {
            0 => Err(Error::SpuriousRateLimiterEvent(
                "Rate limiter event handler called without a present timer",
            )),
            _ => {
                self.timer_active = false;
                Ok(())
            }
        }
    }

    /// Updates the parameters of the token buckets associated with this RateLimiter.
    // TODO: Please note that, right now, the buckets become full after being updated.
    #[tracing::instrument(level = "trace", ret)]
    pub fn update_buckets(&mut self, bytes: BucketUpdate, ops: BucketUpdate) {
        match bytes {
            BucketUpdate::Disabled => self.bandwidth = None,
            BucketUpdate::Update(tb) => self.bandwidth = Some(tb),
            BucketUpdate::None => (),
        };
        match ops {
            BucketUpdate::Disabled => self.ops = None,
            BucketUpdate::Update(tb) => self.ops = Some(tb),
            BucketUpdate::None => (),
        };
    }

    /// Returns an immutable view of the inner bandwidth token bucket.
    #[tracing::instrument(level = "trace", ret)]
    pub fn bandwidth(&self) -> Option<&TokenBucket> {
        self.bandwidth.as_ref()
    }

    /// Returns an immutable view of the inner ops token bucket.
    #[tracing::instrument(level = "trace", ret)]
    pub fn ops(&self) -> Option<&TokenBucket> {
        self.ops.as_ref()
    }
}

impl AsRawFd for RateLimiter {
    /// Provides a FD which needs to be monitored for POLLIN events.
    ///
    /// This object's `event_handler()` method must be called on such events.
    ///
    /// Will return a negative value if rate limiting is disabled on both
    /// token types.
    fn as_raw_fd(&self) -> RawFd {
        self.timer_fd.as_raw_fd()
    }
}

impl Default for RateLimiter {
    /// Default RateLimiter is a no-op limiter with infinite budget.
    fn default() -> Self {
        // Safe to unwrap since this will not attempt to create timer_fd.
        RateLimiter::new(0, 0, 0, 0, 0, 0).expect("Failed to build default RateLimiter")
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::thread;
    use std::time::Duration;

    use super::*;

    impl TokenBucket {
        // Resets the token bucket: budget set to max capacity and last-updated set to now.
        fn reset(&mut self) {
            self.budget = self.size;
            self.last_update = Instant::now();
        }

        fn get_last_update(&self) -> &Instant {
            &self.last_update
        }

        fn get_processed_capacity(&self) -> u64 {
            self.processed_capacity
        }

        fn get_processed_refill_time(&self) -> u64 {
            self.processed_refill_time
        }

        // After a restore, we cannot be certain that the last_update field has the same value.
        #[tracing::instrument(level = "trace", ret)]
        pub fn partial_eq(&self, other: &TokenBucket) -> bool {
            (other.capacity() == self.capacity())
                && (other.one_time_burst() == self.one_time_burst())
                && (other.refill_time_ms() == self.refill_time_ms())
                && (other.budget() == self.budget())
        }
    }

    impl RateLimiter {
        fn get_token_bucket(&self, token_type: TokenType) -> Option<&TokenBucket> {
            match token_type {
                TokenType::Bytes => self.bandwidth.as_ref(),
                TokenType::Ops => self.ops.as_ref(),
            }
        }
    }

    #[test]
    fn test_token_bucket_auto_replenish_one() {
        // These values will give 1 token every 100 milliseconds
        const SIZE: u64 = 10;
        const TIME: u64 = 1000;
        let mut tb = TokenBucket::new(SIZE, 0, TIME).unwrap();
        tb.reduce(SIZE);
        assert_eq!(tb.budget(), 0);

        // Auto-replenishing after 10 milliseconds should not yield any tokens
        thread::sleep(Duration::from_millis(10));
        tb.auto_replenish();
        assert_eq!(tb.budget(), 0);

        // Neither after 20.
        thread::sleep(Duration::from_millis(10));
        tb.auto_replenish();
        assert_eq!(tb.budget(), 0);

        // We should get 1 token after 100 millis
        thread::sleep(Duration::from_millis(80));
        tb.auto_replenish();
        assert_eq!(tb.budget(), 1);

        // So, 5 after 500 millis
        thread::sleep(Duration::from_millis(400));
        tb.auto_replenish();
        assert_eq!(tb.budget(), 5);

        // And be fully replenished after 1 second.
        // Wait more here to make sure we do not overshoot
        thread::sleep(Duration::from_millis(1000));
        tb.auto_replenish();
        assert_eq!(tb.budget(), 10);
    }

    #[test]
    fn test_token_bucket_auto_replenish_two() {
        const SIZE: u64 = 1000;
        const TIME: u64 = 1000;
        let time = Duration::from_millis(TIME);

        let mut tb = TokenBucket::new(SIZE, 0, TIME).unwrap();
        tb.reduce(SIZE);
        assert_eq!(tb.budget(), 0);

        let now = Instant::now();
        while now.elapsed() < time {
            tb.auto_replenish();
        }
        tb.auto_replenish();
        assert_eq!(tb.budget(), SIZE);
    }

    #[test]
    fn test_token_bucket_create() {
        let before = Instant::now();
        let tb = TokenBucket::new(1000, 0, 1000).unwrap();
        assert_eq!(tb.capacity(), 1000);
        assert_eq!(tb.budget(), 1000);
        assert!(*tb.get_last_update() >= before);
        let after = Instant::now();
        assert!(*tb.get_last_update() <= after);
        assert_eq!(tb.get_processed_capacity(), 1);
        assert_eq!(tb.get_processed_refill_time(), 1_000_000);

        // Verify invalid bucket configurations result in `None`.
        assert!(TokenBucket::new(0, 1234, 1000).is_none());
        assert!(TokenBucket::new(100, 1234, 0).is_none());
        assert!(TokenBucket::new(0, 1234, 0).is_none());
    }

    #[test]
    fn test_token_bucket_preprocess() {
        let tb = TokenBucket::new(1000, 0, 1000).unwrap();
        assert_eq!(tb.get_processed_capacity(), 1);
        assert_eq!(tb.get_processed_refill_time(), NANOSEC_IN_ONE_MILLISEC);

        let thousand = 1000;
        let tb = TokenBucket::new(3 * 7 * 11 * 19 * thousand, 0, 7 * 11 * 13 * 17).unwrap();
        assert_eq!(tb.get_processed_capacity(), 3 * 19);
        assert_eq!(
            tb.get_processed_refill_time(),
            13 * 17 * (NANOSEC_IN_ONE_MILLISEC / thousand)
        );
    }

    #[test]
    fn test_token_bucket_reduce() {
        // token bucket with capacity 1000 and refill time of 1000 milliseconds
        // allowing rate of 1 token/ms.
        let capacity = 1000;
        let refill_ms = 1000;
        let mut tb = TokenBucket::new(capacity, 0, refill_ms).unwrap();

        assert_eq!(tb.reduce(123), BucketReduction::Success);
        assert_eq!(tb.budget(), capacity - 123);
        assert_eq!(tb.reduce(capacity), BucketReduction::Failure);

        // token bucket with capacity 1000 and refill time of 1000 milliseconds
        let mut tb = TokenBucket::new(1000, 1100, 1000).unwrap();
        // safely assuming the thread can run these 3 commands in less than 500ms
        assert_eq!(tb.reduce(1000), BucketReduction::Success);
        assert_eq!(tb.one_time_burst(), 100);
        assert_eq!(tb.reduce(500), BucketReduction::Success);
        assert_eq!(tb.one_time_burst(), 0);
        assert_eq!(tb.reduce(500), BucketReduction::Success);
        assert_eq!(tb.reduce(500), BucketReduction::Failure);
        thread::sleep(Duration::from_millis(500));
        assert_eq!(tb.reduce(500), BucketReduction::Success);
        thread::sleep(Duration::from_millis(1000));
        assert_eq!(tb.reduce(2500), BucketReduction::OverConsumption(1.5));

        let before = Instant::now();
        tb.reset();
        assert_eq!(tb.capacity(), 1000);
        assert_eq!(tb.budget(), 1000);
        assert!(*tb.get_last_update() >= before);
        let after = Instant::now();
        assert!(*tb.get_last_update() <= after);
    }

    #[test]
    fn test_rate_limiter_default() {
        let mut l = RateLimiter::default();

        // limiter should not be blocked
        assert!(!l.is_blocked());
        // limiter should be disabled so consume(whatever) should work
        assert!(l.consume(u64::max_value(), TokenType::Ops));
        assert!(l.consume(u64::max_value(), TokenType::Bytes));
        // calling the handler without there having been an event should error
        assert!(l.event_handler().is_err());
        assert_eq!(
            format!("{:?}", l.event_handler().err().unwrap()),
            "SpuriousRateLimiterEvent(\"Rate limiter event handler called without a present \
             timer\")"
        );
    }

    #[test]
    fn test_rate_limiter_new() {
        let l = RateLimiter::new(1000, 1001, 1002, 1003, 1004, 1005).unwrap();

        let bw = l.bandwidth.unwrap();
        assert_eq!(bw.capacity(), 1000);
        assert_eq!(bw.one_time_burst(), 1001);
        assert_eq!(bw.refill_time_ms(), 1002);
        assert_eq!(bw.budget(), 1000);

        let ops = l.ops.unwrap();
        assert_eq!(ops.capacity(), 1003);
        assert_eq!(ops.one_time_burst(), 1004);
        assert_eq!(ops.refill_time_ms(), 1005);
        assert_eq!(ops.budget(), 1003);
    }

    #[test]
    fn test_rate_limiter_manual_replenish() {
        // rate limiter with limit of 1000 bytes/s and 1000 ops/s
        let mut l = RateLimiter::new(1000, 0, 1000, 1000, 0, 1000).unwrap();

        // consume 123 bytes
        assert!(l.consume(123, TokenType::Bytes));
        l.manual_replenish(23, TokenType::Bytes);
        {
            let bytes_tb = l.get_token_bucket(TokenType::Bytes).unwrap();
            assert_eq!(bytes_tb.budget(), 900);
        }
        // consume 123 ops
        assert!(l.consume(123, TokenType::Ops));
        l.manual_replenish(23, TokenType::Ops);
        {
            let bytes_tb = l.get_token_bucket(TokenType::Ops).unwrap();
            assert_eq!(bytes_tb.budget(), 900);
        }
    }

    #[test]
    fn test_rate_limiter_bandwidth() {
        // rate limiter with limit of 1000 bytes/s
        let mut l = RateLimiter::new(1000, 0, 1000, 0, 0, 0).unwrap();

        // limiter should not be blocked
        assert!(!l.is_blocked());
        // raw FD for this disabled should be valid
        assert!(l.as_raw_fd() > 0);

        // ops/s limiter should be disabled so consume(whatever) should work
        assert!(l.consume(u64::max_value(), TokenType::Ops));

        // do full 1000 bytes
        assert!(l.consume(1000, TokenType::Bytes));
        // try and fail on another 100
        assert!(!l.consume(100, TokenType::Bytes));
        // since consume failed, limiter should be blocked now
        assert!(l.is_blocked());
        // wait half the timer period
        thread::sleep(Duration::from_millis(REFILL_TIMER_INTERVAL_MS / 2));
        // limiter should still be blocked
        assert!(l.is_blocked());
        // wait the other half of the timer period
        thread::sleep(Duration::from_millis(REFILL_TIMER_INTERVAL_MS / 2));
        // the timer_fd should have an event on it by now
        assert!(l.event_handler().is_ok());
        // limiter should now be unblocked
        assert!(!l.is_blocked());
        // try and succeed on another 100 bytes this time
        assert!(l.consume(100, TokenType::Bytes));
    }

    #[test]
    fn test_rate_limiter_ops() {
        // rate limiter with limit of 1000 ops/s
        let mut l = RateLimiter::new(0, 0, 0, 1000, 0, 1000).unwrap();

        // limiter should not be blocked
        assert!(!l.is_blocked());
        // raw FD for this disabled should be valid
        assert!(l.as_raw_fd() > 0);

        // bytes/s limiter should be disabled so consume(whatever) should work
        assert!(l.consume(u64::max_value(), TokenType::Bytes));

        // do full 1000 ops
        assert!(l.consume(1000, TokenType::Ops));
        // try and fail on another 100
        assert!(!l.consume(100, TokenType::Ops));
        // since consume failed, limiter should be blocked now
        assert!(l.is_blocked());
        // wait half the timer period
        thread::sleep(Duration::from_millis(REFILL_TIMER_INTERVAL_MS / 2));
        // limiter should still be blocked
        assert!(l.is_blocked());
        // wait the other half of the timer period
        thread::sleep(Duration::from_millis(REFILL_TIMER_INTERVAL_MS / 2));
        // the timer_fd should have an event on it by now
        assert!(l.event_handler().is_ok());
        // limiter should now be unblocked
        assert!(!l.is_blocked());
        // try and succeed on another 100 ops this time
        assert!(l.consume(100, TokenType::Ops));
    }

    #[test]
    fn test_rate_limiter_full() {
        // rate limiter with limit of 1000 bytes/s and 1000 ops/s
        let mut l = RateLimiter::new(1000, 0, 1000, 1000, 0, 1000).unwrap();

        // limiter should not be blocked
        assert!(!l.is_blocked());
        // raw FD for this disabled should be valid
        assert!(l.as_raw_fd() > 0);

        // do full 1000 bytes
        assert!(l.consume(1000, TokenType::Ops));
        // do full 1000 bytes
        assert!(l.consume(1000, TokenType::Bytes));
        // try and fail on another 100 ops
        assert!(!l.consume(100, TokenType::Ops));
        // try and fail on another 100 bytes
        assert!(!l.consume(100, TokenType::Bytes));
        // since consume failed, limiter should be blocked now
        assert!(l.is_blocked());
        // wait half the timer period
        thread::sleep(Duration::from_millis(REFILL_TIMER_INTERVAL_MS / 2));
        // limiter should still be blocked
        assert!(l.is_blocked());
        // wait the other half of the timer period
        thread::sleep(Duration::from_millis(REFILL_TIMER_INTERVAL_MS / 2));
        // the timer_fd should have an event on it by now
        assert!(l.event_handler().is_ok());
        // limiter should now be unblocked
        assert!(!l.is_blocked());
        // try and succeed on another 100 ops this time
        assert!(l.consume(100, TokenType::Ops));
        // try and succeed on another 100 bytes this time
        assert!(l.consume(100, TokenType::Bytes));
    }

    #[test]
    fn test_rate_limiter_overconsumption() {
        // initialize the rate limiter
        let mut l = RateLimiter::new(1000, 0, 1000, 1000, 0, 1000).unwrap();
        // try to consume 2.5x the bucket size
        // we are "borrowing" 1.5x the bucket size in tokens since
        // the bucket is full
        assert!(l.consume(2500, TokenType::Bytes));

        // check that even after a whole second passes, the rate limiter
        // is still blocked
        thread::sleep(Duration::from_millis(1000));
        assert!(l.event_handler().is_err());
        assert!(l.is_blocked());

        // after 1.5x the replenish time has passed, the rate limiter
        // is available again
        thread::sleep(Duration::from_millis(500));
        assert!(l.event_handler().is_ok());
        assert!(!l.is_blocked());

        // reset the rate limiter
        let mut l = RateLimiter::new(1000, 0, 1000, 1000, 0, 1000).unwrap();
        // try to consume 1.5x the bucket size
        // we are "borrowing" 1.5x the bucket size in tokens since
        // the bucket is full, should arm the timer to 0.5x replenish
        // time, which is 500 ms
        assert!(l.consume(1500, TokenType::Bytes));

        // check that after more than the minimum refill time,
        // the rate limiter is still blocked
        thread::sleep(Duration::from_millis(200));
        assert!(l.event_handler().is_err());
        assert!(l.is_blocked());

        // try to consume some tokens, which should fail as the timer
        // is still active
        assert!(!l.consume(100, TokenType::Bytes));
        assert!(l.event_handler().is_err());
        assert!(l.is_blocked());

        // check that after the minimum refill time, the timer was not
        // overwritten and the rate limiter is still blocked from the
        // borrowing we performed earlier
        thread::sleep(Duration::from_millis(100));
        assert!(l.event_handler().is_err());
        assert!(l.is_blocked());
        assert!(!l.consume(100, TokenType::Bytes));

        // after waiting out the full duration, rate limiter should be
        // availale again
        thread::sleep(Duration::from_millis(200));
        assert!(l.event_handler().is_ok());
        assert!(!l.is_blocked());
        assert!(l.consume(100, TokenType::Bytes));
    }

    #[test]
    fn test_update_buckets() {
        let mut x = RateLimiter::new(1000, 2000, 1000, 10, 20, 1000).unwrap();

        let initial_bw = x.bandwidth.clone();
        let initial_ops = x.ops.clone();

        x.update_buckets(BucketUpdate::None, BucketUpdate::None);
        assert_eq!(x.bandwidth, initial_bw);
        assert_eq!(x.ops, initial_ops);

        let new_bw = TokenBucket::new(123, 0, 57).unwrap();
        let new_ops = TokenBucket::new(321, 12346, 89).unwrap();
        x.update_buckets(
            BucketUpdate::Update(new_bw.clone()),
            BucketUpdate::Update(new_ops.clone()),
        );

        // We have manually adjust the last_update field, because it changes when update_buckets()
        // constructs new buckets (and thus gets a different value for last_update). We do this so
        // it makes sense to test the following assertions.
        x.bandwidth.as_mut().unwrap().last_update = new_bw.last_update;
        x.ops.as_mut().unwrap().last_update = new_ops.last_update;

        assert_eq!(x.bandwidth, Some(new_bw));
        assert_eq!(x.ops, Some(new_ops));

        x.update_buckets(BucketUpdate::Disabled, BucketUpdate::Disabled);
        assert_eq!(x.bandwidth, None);
        assert_eq!(x.ops, None);
    }

    #[test]
    fn test_rate_limiter_debug() {
        let l = RateLimiter::new(1, 2, 3, 4, 5, 6).unwrap();
        assert_eq!(
            format!("{:?}", l),
            format!(
                "RateLimiter {{ bandwidth: {:?}, ops: {:?} }}",
                l.bandwidth(),
                l.ops()
            ),
        );
    }
}
