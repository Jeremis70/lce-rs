//! `JavaRandom`: a 48-bit linear congruential generator.
//!
//! The generator holds a 48-bit state `s` and advances it as
//! `s ← (s · 0x5_DEEC_E66D + 0xB) mod 2⁴⁸`. Every draw takes the top bits of
//! the new state, and every derived value (bounded integers, floats, longs,
//! Gaussians) is built from those draws in a fixed order, so a seed determines
//! the whole sequence. Two generators with the same seed, driven through the
//! same calls, produce the same values on every platform. The one exception is
//! [`JavaRandom::next_gaussian`], which also depends on the platform's `ln`
//! (see there).
//!
//! There is deliberately no seedless constructor. The simulation must be
//! reproducible, so every generator is created from an explicit seed. Callers
//! that want an unpredictable one choose it themselves (e.g. from the clock)
//! and pass it to [`JavaRandom::new`].

/// Multiplier of the state update.
const MULTIPLIER: u64 = 0x5_DEEC_E66D;

/// Increment of the state update.
const INCREMENT: u64 = 0xB;

/// The state is 48 bits wide.
const STATE_MASK: u64 = (1 << 48) - 1;

/// The state a seed starts from: its low 48 bits XOR the multiplier.
/// Bits above the 48th are discarded.
const fn scramble(seed: i64) -> u64 {
    (seed as u64 ^ MULTIPLIER) & STATE_MASK
}

/// One step of the state update.
const fn step(state: u64) -> u64 {
    state.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT) & STATE_MASK
}

/// A seeded 48-bit linear congruential generator.
///
/// Seeding, the state update and the integer, boolean, `f32`, `f64` and `i64`
/// draws produce the same sequences as `java.util.Random`. [`next_bytes`] and
/// [`next_gaussian`] are specified on their own below.
///
/// [`next_bytes`]: Self::next_bytes
/// [`next_gaussian`]: Self::next_gaussian
///
/// ```
/// use lce_math::random::JavaRandom;
///
/// let mut random = JavaRandom::new(42);
/// assert_eq!(random.next_int_unbounded(), -1_170_105_035);
/// assert_eq!(random.next_int(10), 3);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct JavaRandom {
    /// The 48-bit state; the upper 16 bits are always zero.
    state: u64,
    /// The second value of the last Gaussian pair, until it is handed out.
    next_next_gaussian: Option<f64>,
}

impl JavaRandom {
    /// A generator seeded with `seed`. Only the low 48 bits of `seed` matter.
    #[must_use]
    pub const fn new(seed: i64) -> Self {
        Self {
            state: scramble(seed),
            next_next_gaussian: None,
        }
    }

    /// Restarts the sequence as if newly created with `seed`.
    ///
    /// Also discards a cached Gaussian (see [`next_gaussian`](Self::next_gaussian)).
    pub const fn set_seed(&mut self, seed: i64) {
        self.state = scramble(seed);
        self.next_next_gaussian = None;
    }

    /// Advances the state once and returns its top `BITS` bits.
    ///
    /// The result is the bits as an unsigned number, reinterpreted as `i32`,
    /// so it is non-negative for `BITS < 32` and may be negative for
    /// `BITS == 32`. Every other method here is built on this draw.
    ///
    /// `BITS` must be in `1..=32`; anything else fails to compile:
    ///
    /// ```compile_fail
    /// # use lce_math::random::JavaRandom;
    /// let _ = JavaRandom::new(0).next_bits::<33>();
    /// ```
    pub const fn next_bits<const BITS: u32>(&mut self) -> i32 {
        const { assert!(BITS >= 1 && BITS <= 32, "BITS must be in 1..=32") };
        self.state = step(self.state);
        (self.state >> (48 - BITS)) as u32 as i32
    }

    /// A uniformly distributed `i32` over its whole range: one 32-bit draw.
    pub const fn next_int_unbounded(&mut self) -> i32 {
        self.next_bits::<32>()
    }

    /// A uniformly distributed integer in `0..bound`, for `bound > 0`.
    ///
    /// When `bound` is a power of two, the result is the top bits of one
    /// 31-bit draw: `(draw · bound) >> 31`.
    ///
    /// Otherwise the result is `draw % bound`, but a draw is rejected and
    /// another taken if it falls in the incomplete last block of `bound`
    /// values below `2³¹`. The test is `draw - result + (bound - 1) < 0` with
    /// wrapping `i32` arithmetic, i.e. the block starting at `draw - result`
    /// would run past `i32::MAX`. The number of draws consumed therefore
    /// varies; it is fewer than 2 on average for any `bound`.
    ///
    /// A `bound` of zero passes the power-of-two test and so returns `0`
    /// after one draw. Negative bounds are treated the same way: `0` after one
    /// 31-bit draw. This never panics.
    pub const fn next_int(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            self.next_bits::<31>();
            return 0;
        }
        if bound & bound.wrapping_neg() == bound {
            return ((self.next_bits::<31>() as i64 * bound as i64) >> 31) as i32;
        }
        loop {
            let draw = self.next_bits::<31>();
            let result = draw % bound;
            if draw.wrapping_sub(result).wrapping_add(bound - 1) >= 0 {
                return result;
            }
        }
    }

    /// A uniformly distributed `i64`: a 32-bit draw shifted up by 32 bits,
    /// plus a second 32-bit draw.
    ///
    /// The draws are taken in that order. The second is added as a signed
    /// value, so when it is negative it borrows from the first (wrapping).
    /// Not every `i64` can occur, since the result depends on only 48 bits of
    /// state.
    pub const fn next_long(&mut self) -> i64 {
        let high = (self.next_bits::<32>() as i64) << 32;
        high.wrapping_add(self.next_bits::<32>() as i64)
    }

    /// `true` or `false` with equal probability: the top bit of one draw.
    pub const fn next_boolean(&mut self) -> bool {
        self.next_bits::<1>() != 0
    }

    /// A uniformly distributed `f32` in `[0, 1)`: one 24-bit draw divided by
    /// `2²⁴` in single precision, exact.
    pub const fn next_float(&mut self) -> f32 {
        self.next_bits::<24>() as f32 / (1u32 << 24) as f32
    }

    /// A uniformly distributed `f64` in `[0, 1)`: a 26-bit draw shifted up
    /// by 27 bits, plus a 27-bit draw, divided by `2⁵³`, exact.
    ///
    /// The draws are taken in that order.
    pub const fn next_double(&mut self) -> f64 {
        let high = (self.next_bits::<26>() as i64) << 27;
        let low = self.next_bits::<27>() as i64;
        (high + low) as f64 / (1u64 << 53) as f64
    }

    /// Fills `bytes` in order, each byte from its own 8-bit draw, so filling
    /// `n` bytes consumes exactly `n` draws.
    pub const fn next_bytes(&mut self, bytes: &mut [u8]) {
        let mut i = 0;
        while i < bytes.len() {
            bytes[i] = self.next_bits::<8>() as u8;
            i += 1;
        }
    }

    /// A normally distributed `f64` with mean 0 and standard deviation 1,
    /// by the polar method.
    ///
    /// Values come in pairs. On a call with no value cached, pairs of
    /// [`next_double`](Self::next_double) draws are turned into
    /// `v1 = 2·d1 − 1` and `v2 = 2·d2 − 1` until `s = v1² + v2²` lies strictly
    /// between 0 and 1. With `m = sqrt(−2·ln(s) / s)` the call returns `v1·m`
    /// and caches `v2·m`. The next call returns the cached value without
    /// drawing, however many other draws came in between, unless
    /// [`set_seed`](Self::set_seed) discarded it.
    ///
    /// Every step is IEEE-754 double arithmetic, evaluated exactly as written
    /// (`-2 * ln(s)`, then `/ s`), except `ln`, which is the platform's
    /// [`f64::ln`]. That is not guaranteed to be correctly rounded, so
    /// results can differ in the last bits between platforms. This is the only
    /// method here that is not bit-for-bit platform-independent.
    pub fn next_gaussian(&mut self) -> f64 {
        if let Some(cached) = self.next_next_gaussian.take() {
            return cached;
        }
        loop {
            let v1 = 2.0 * self.next_double() - 1.0;
            let v2 = 2.0 * self.next_double() - 1.0;
            let s = v1 * v1 + v2 * v2;
            if s < 1.0 && s != 0.0 {
                let multiplier = (-2.0 * s.ln() / s).sqrt();
                self.next_next_gaussian = Some(v2 * multiplier);
                return v1 * multiplier;
            }
        }
    }
}
