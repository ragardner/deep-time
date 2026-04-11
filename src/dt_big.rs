use core::fmt::{self, Binary, Display, LowerHex, Octal, UpperHex};
use core::iter::{Product, Sum};
use core::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};
use core::str::FromStr;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "js", derive(tsify::Tsify))]
#[repr(transparent)]
pub struct DtBig {
    /// Raw 320-bit two's-complement integer stored as 5 little-endian `u64` parts.
    /// The sign bit is the highest bit (bit 63) of `num[4]`.
    num: [u64; 5],
}

impl DtBig {
    /// The zero value.
    pub const ZERO: Self = Self { num: [0; 5] };

    /// The value `1`.
    pub const ONE: Self = Self {
        num: [1, 0, 0, 0, 0],
    };

    pub const NEG_ONE: Self = Self { num: [u64::MAX; 5] };

    /// The value `10`.
    pub const TEN: Self = Self {
        num: [10, 0, 0, 0, 0],
    };

    /// The minimum representable value (`-2^319`).
    pub const MIN: Self = Self {
        num: [0, 0, 0, 0, 1u64 << 63],
    };

    /// The maximum representable value (`2^319 - 1`).
    pub const MAX: Self = Self {
        num: [u64::MAX, u64::MAX, u64::MAX, u64::MAX, (1u64 << 63) - 1],
    };

    /// Creates a new `DtBig` from its raw little-endian representation.
    pub const fn from_num(num: [u64; 5]) -> Self {
        Self { num }
    }

    /// Returns a reference to the raw little-endian representation.
    pub const fn num(&self) -> &[u64; 5] {
        &self.num
    }

    /// Mutable access to the raw representation (internal use only).
    pub(crate) const fn num_mut(&mut self) -> &mut [u64; 5] {
        &mut self.num
    }

    /// Const-friendly equality check (use until `PartialEq` becomes `const`).
    #[inline]
    pub const fn eq(self, other: Self) -> bool {
        let mut i = 0usize;
        while i < 5 {
            if self.num[i] != other.num[i] {
                return false;
            }
            i += 1;
        }
        true
    }

    /// Returns `true` if this value is zero.
    #[inline(always)]
    pub const fn is_zero(self) -> bool {
        self.eq(Self::ZERO)
    }

    /// Returns `true` if this value is exactly `1`.
    #[inline(always)]
    pub const fn is_one(self) -> bool {
        self.eq(Self::ONE)
    }

    /// Returns `true` if this value is exactly `-1`.
    #[inline(always)]
    pub const fn is_neg_one(self) -> bool {
        self.eq(Self::NEG_ONE)
    }

    /// Returns `true` if this value is exactly `MIN` (`-2^319`).
    #[inline(always)]
    pub const fn is_min(self) -> bool {
        self.eq(Self::MIN)
    }

    /// Returns `true` if `self` is even (divisible by 2).
    #[inline(always)]
    pub const fn is_even(self) -> bool {
        (self.num[0] & 1) == 0
    }

    /// Returns `true` if `self` is odd (not divisible by 2).
    #[inline(always)]
    pub const fn is_odd(self) -> bool {
        (self.num[0] & 1) != 0
    }

    pub const fn is_multiple_of(self, rhs: Self) -> bool {
        if rhs.is_zero() {
            return self.is_zero(); // only 0 is a multiple of 0
        }
        if self.is_zero() {
            return true; // 0 is a multiple of any non-zero number
        }
        if rhs.is_neg_one() || rhs.eq(Self::ONE) {
            return true; // any number is a multiple of ±1
        }
        // General case: remainder must be zero
        self.wrapping_rem(rhs).is_zero()
    }
}

impl Default for DtBig {
    fn default() -> Self {
        Self::ZERO
    }
}

// ──────────────────────────────────────────────────────────────
// Debug
// ──────────────────────────────────────────────────────────────

impl core::fmt::Debug for DtBig {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("DtBig").field(&self.num).finish()
    }
}

// ──────────────────────────────────────────────────────────────
// Negation
// ──────────────────────────────────────────────────────────────

impl core::ops::Neg for DtBig {
    type Output = Self;

    fn neg(self) -> Self {
        Self::neg(self)
    }
}

impl DtBig {
    /// Negation (two's complement).
    #[inline]
    pub const fn neg(self) -> Self {
        let mut result = self;
        let parts = result.num_mut();
        let mut carry = 1u64;
        let mut i = 0;
        while i < 5 {
            let (sum, overflow) = (!parts[i]).overflowing_add(carry);
            parts[i] = sum;
            carry = if overflow { 1 } else { 0 };
            i += 1;
        }

        result
    }

    /// Wrapping negation (two's complement).
    #[inline(always)]
    pub const fn wrapping_neg(self) -> Self {
        self.neg()
    }

    /// Saturating negation (MIN → MAX, everything else normal).
    #[inline(always)]
    pub const fn saturating_neg(self) -> Self {
        if self.is_min() { Self::MAX } else { self.neg() }
    }

    /// Checked negation. Returns `None` on `MIN`.
    #[inline(always)]
    pub const fn checked_neg(self) -> Option<Self> {
        if self.is_min() {
            None
        } else {
            Some(self.neg())
        }
    }

    /// Returns the negated value and whether overflow occurred (`MIN`).
    #[inline(always)]
    pub const fn overflowing_neg(self) -> (Self, bool) {
        if self.is_min() {
            (Self::MAX, true)
        } else {
            (self.neg(), false)
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Addition
// ──────────────────────────────────────────────────────────────

impl Add for DtBig {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }
}

impl Add<&DtBig> for DtBig {
    type Output = Self;
    #[inline]
    fn add(self, rhs: &DtBig) -> Self {
        self + *rhs
    }
}

impl Add<DtBig> for &DtBig {
    type Output = DtBig;
    #[inline]
    fn add(self, rhs: DtBig) -> DtBig {
        *self + rhs
    }
}

impl Add<&DtBig> for &DtBig {
    type Output = DtBig;
    #[inline]
    fn add(self, rhs: &DtBig) -> DtBig {
        *self + *rhs
    }
}

impl AddAssign for DtBig {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl AddAssign<&DtBig> for DtBig {
    #[inline]
    fn add_assign(&mut self, rhs: &DtBig) {
        *self = *self + rhs;
    }
}

impl DtBig {
    /// Wrapping addition (modulo 2^320).
    #[inline]
    pub const fn wrapping_add(self, rhs: Self) -> Self {
        let mut result = Self { num: [0; 5] };
        let mut carry = false;
        let mut i = 0;
        while i < 5 {
            let (sum1, c1) = self.num[i].overflowing_add(rhs.num[i]);
            let (sum2, c2) = sum1.overflowing_add(carry as u64);
            result.num[i] = sum2;
            carry = c1 || c2;
            i += 1;
        }
        result
    }

    /// Saturating addition (clamps to `MIN` or `MAX` on overflow).
    #[inline]
    pub const fn saturating_add(self, rhs: Self) -> Self {
        let sum = self.wrapping_add(rhs);
        let self_sign = (self.num[4] & (1u64 << 63)) != 0;
        let rhs_sign = (rhs.num[4] & (1u64 << 63)) != 0;
        let sum_sign = (sum.num[4] & (1u64 << 63)) != 0;
        if self_sign == rhs_sign && self_sign != sum_sign {
            if self_sign { Self::MIN } else { Self::MAX }
        } else {
            sum
        }
    }

    /// Checked addition. Returns `None` on overflow.
    #[inline]
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        let sum = self.wrapping_add(rhs);
        let self_sign = (self.num[4] & (1u64 << 63)) != 0;
        let rhs_sign = (rhs.num[4] & (1u64 << 63)) != 0;
        let sum_sign = (sum.num[4] & (1u64 << 63)) != 0;
        if self_sign == rhs_sign && self_sign != sum_sign {
            None
        } else {
            Some(sum)
        }
    }

    /// Returns the sum and a boolean indicating whether overflow occurred.
    #[inline]
    pub const fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        let sum = self.wrapping_add(rhs);
        let self_sign = (self.num[4] & (1u64 << 63)) != 0;
        let rhs_sign = (rhs.num[4] & (1u64 << 63)) != 0;
        let sum_sign = (sum.num[4] & (1u64 << 63)) != 0;
        let overflow = self_sign == rhs_sign && self_sign != sum_sign;
        (sum, overflow)
    }
}

macro_rules! impl_add_small {
    ($($ty:ty => $from_fn:ident),* $(,)?) => {
        $(
            // DtBig + small_int
            impl Add<$ty> for DtBig {
                type Output = Self;

                #[inline(always)]
                fn add(self, rhs: $ty) -> Self {
                    self.wrapping_add(DtBig::$from_fn(rhs))
                }
            }

            impl AddAssign<$ty> for DtBig {
                #[inline(always)]
                fn add_assign(&mut self, rhs: $ty) {
                    *self = *self + rhs;
                }
            }

            // small_int + DtBig
            impl Add<DtBig> for $ty {
                type Output = DtBig;

                #[inline(always)]
                fn add(self, rhs: DtBig) -> DtBig {
                    rhs.wrapping_add(DtBig::$from_fn(self))
                }
            }

            impl Add<$ty> for &DtBig {
                type Output = DtBig;
                fn add(self, rhs: $ty) -> DtBig {
                    *self + rhs
                }
            }

            impl Add<&DtBig> for $ty {
                type Output = DtBig;
                fn add(self, rhs: &DtBig) -> DtBig {
                    self + *rhs
                }
            }
        )*
    };
}

impl_add_small! {
    i8  => from_i8,
    i16 => from_i16,
    i32 => from_i32,
    i64 => from_i64,
    u8  => from_u8,
    u16 => from_u16,
    u32 => from_u32,
    u64 => from_u64,
}

// ──────────────────────────────────────────────────────────────
// Signed ordering
// ──────────────────────────────────────────────────────────────

impl PartialOrd for DtBig {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DtBig {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // Compare highest part as signed i64 first (contains sign bit)
        match (self.num[4] as i64).cmp(&(other.num[4] as i64)) {
            core::cmp::Ordering::Equal => {
                // Same sign → compare remaining parts from most to least significant
                for i in (0..4).rev() {
                    match self.num[i].cmp(&other.num[i]) {
                        core::cmp::Ordering::Equal => continue,
                        ord => return ord,
                    }
                }
                core::cmp::Ordering::Equal
            }
            ord => ord,
        }
    }
}

impl DtBig {
    /// Const-friendly three-way comparison (exact equivalent of your `Ord::cmp`).
    ///
    /// This version is fully `const fn` on stable Rust:
    /// - No reliance on `Ord::cmp` (still not stably `const`).
    /// - No iterator / `for` loop (the `Rev<Range>` iterator is not const-friendly yet).
    /// - Everything is expanded manually with primitive `>`, `<`, `==` (these *are* stable in `const fn`).
    pub const fn cmp(self, other: Self) -> core::cmp::Ordering {
        let a = self.num();
        let b = other.num();

        // 1. Highest limb first – treat as signed (sign bit is in the high bit of a[4]/b[4])
        let a_high = a[4] as i64;
        let b_high = b[4] as i64;

        if a_high > b_high {
            return core::cmp::Ordering::Greater;
        }
        if a_high < b_high {
            return core::cmp::Ordering::Less;
        }

        // 2. High limbs are equal → compare the remaining limbs from MSB to LSB as unsigned.
        if a[3] > b[3] {
            return core::cmp::Ordering::Greater;
        }
        if a[3] < b[3] {
            return core::cmp::Ordering::Less;
        }

        if a[2] > b[2] {
            return core::cmp::Ordering::Greater;
        }
        if a[2] < b[2] {
            return core::cmp::Ordering::Less;
        }

        if a[1] > b[1] {
            return core::cmp::Ordering::Greater;
        }
        if a[1] < b[1] {
            return core::cmp::Ordering::Less;
        }

        if a[0] > b[0] {
            return core::cmp::Ordering::Greater;
        }
        if a[0] < b[0] {
            return core::cmp::Ordering::Less;
        }

        core::cmp::Ordering::Equal
    }

    /// Const-friendly `self >= other`
    pub const fn ge(self, other: Self) -> bool {
        match self.cmp(other) {
            core::cmp::Ordering::Greater | core::cmp::Ordering::Equal => true,
            _ => false,
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Subtraction
// ──────────────────────────────────────────────────────────────

impl Sub for DtBig {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self {
        self + Self::neg(rhs)
    }
}

impl Sub<&DtBig> for DtBig {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: &DtBig) -> Self {
        self - *rhs
    }
}

impl Sub<DtBig> for &DtBig {
    type Output = DtBig;

    #[inline(always)]
    fn sub(self, rhs: DtBig) -> DtBig {
        *self - rhs
    }
}

impl Sub<&DtBig> for &DtBig {
    type Output = DtBig;

    #[inline(always)]
    fn sub(self, rhs: &DtBig) -> DtBig {
        *self - *rhs
    }
}

impl SubAssign for DtBig {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl SubAssign<&DtBig> for DtBig {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &DtBig) {
        *self = *self - rhs;
    }
}

impl DtBig {
    /// Wrapping subtraction (modulo 2^320).
    #[inline(always)]
    pub const fn wrapping_sub(self, rhs: Self) -> Self {
        self.wrapping_add(Self::neg(rhs))
    }

    /// Saturating subtraction (clamps to `MIN` or `MAX` on overflow).
    #[inline]
    pub const fn saturating_sub(self, rhs: Self) -> Self {
        let diff = self.wrapping_sub(rhs);

        let self_sign = (self.num[4] & (1u64 << 63)) != 0;
        let rhs_sign = (rhs.num[4] & (1u64 << 63)) != 0;
        let diff_sign = (diff.num[4] & (1u64 << 63)) != 0;

        if self_sign != rhs_sign && self_sign != diff_sign {
            if self_sign { Self::MIN } else { Self::MAX }
        } else {
            diff
        }
    }

    /// Checked subtraction. Returns `None` on overflow.
    #[inline]
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        let diff = self.wrapping_sub(rhs);
        let self_sign = (self.num[4] & (1u64 << 63)) != 0;
        let rhs_sign = (rhs.num[4] & (1u64 << 63)) != 0;
        let diff_sign = (diff.num[4] & (1u64 << 63)) != 0;

        if self_sign != rhs_sign && self_sign != diff_sign {
            None
        } else {
            Some(diff)
        }
    }

    /// Returns the difference and a boolean indicating whether overflow occurred.
    #[inline]
    pub const fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
        let diff = self.wrapping_sub(rhs);
        let self_sign = (self.num[4] & (1u64 << 63)) != 0;
        let rhs_sign = (rhs.num[4] & (1u64 << 63)) != 0;
        let diff_sign = (diff.num[4] & (1u64 << 63)) != 0;

        let overflow = self_sign != rhs_sign && self_sign != diff_sign;
        (diff, overflow)
    }
}

macro_rules! impl_sub_small {
    ($($ty:ty => $from_fn:ident),* $(,)?) => {
        $(
            // DtBig - small_int
            impl Sub<$ty> for DtBig {
                type Output = Self;

                #[inline(always)]
                fn sub(self, rhs: $ty) -> Self {
                    self.wrapping_sub(DtBig::$from_fn(rhs))
                }
            }

            impl SubAssign<$ty> for DtBig {
                #[inline(always)]
                fn sub_assign(&mut self, rhs: $ty) {
                    *self = *self - rhs;
                }
            }

            // small_int - DtBig
            impl Sub<DtBig> for $ty {
                type Output = DtBig;

                #[inline(always)]
                fn sub(self, rhs: DtBig) -> DtBig {
                    DtBig::$from_fn(self).wrapping_sub(rhs)
                }
            }

            // Reference versions for full consistency with the rest of the crate
            impl Sub<$ty> for &DtBig {
                type Output = DtBig;

                #[inline(always)]
                fn sub(self, rhs: $ty) -> DtBig {
                    *self - rhs
                }
            }

            impl Sub<&DtBig> for $ty {
                type Output = DtBig;

                #[inline(always)]
                fn sub(self, rhs: &DtBig) -> DtBig {
                    self - *rhs
                }
            }
        )*
    };
}

impl_sub_small! {
    i8  => from_i8,
    i16 => from_i16,
    i32 => from_i32,
    i64 => from_i64,
    u8  => from_u8,
    u16 => from_u16,
    u32 => from_u32,
    u64 => from_u64,
}

// ──────────────────────────────────────────────────────────────
// Multiplication
// ──────────────────────────────────────────────────────────────

impl Mul for DtBig {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self {
        self.wrapping_mul(rhs)
    }
}

impl Mul<&DtBig> for DtBig {
    type Output = Self;

    #[inline(always)]
    fn mul(self, rhs: &DtBig) -> Self {
        self * *rhs
    }
}

impl Mul<DtBig> for &DtBig {
    type Output = DtBig;

    #[inline(always)]
    fn mul(self, rhs: DtBig) -> DtBig {
        *self * rhs
    }
}

impl Mul<&DtBig> for &DtBig {
    type Output = DtBig;

    #[inline(always)]
    fn mul(self, rhs: &DtBig) -> DtBig {
        *self * *rhs
    }
}

impl MulAssign for DtBig {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl MulAssign<&DtBig> for DtBig {
    #[inline(always)]
    fn mul_assign(&mut self, rhs: &DtBig) {
        *self = *self * rhs;
    }
}

impl DtBig {
    /// Computes the full 640-bit unsigned product of the absolute values.
    /// Returns `(low_320_bits, high_320_bits_nonzero)`.
    const fn unsigned_mul_with_overflow_info(self, rhs: Self) -> (Self, bool) {
        let mut product = [0u64; 10];

        let mut i = 0;
        while i < 5 {
            let mut carry = 0u128;
            let mut j = 0;
            while j < 5 {
                let a = self.num[i] as u128;
                let b = rhs.num[j] as u128;
                let temp = a * b + carry + product[i + j] as u128;

                product[i + j] = temp as u64;
                carry = temp >> 64;
                j += 1;
            }

            let mut k = i + 5;
            while carry > 0 && k < 10 {
                let temp = product[k] as u128 + carry;
                product[k] = temp as u64;
                carry = temp >> 64;
                k += 1;
            }
            i += 1;
        }

        // Extract low 320 bits
        let mut low = [0u64; 5];
        let mut i = 0;
        while i < 5 {
            low[i] = product[i];
            i += 1;
        }

        // Check if any high 320 bits are nonzero
        let mut high_nonzero = false;
        let mut i = 5;
        while i < 10 {
            if product[i] != 0 {
                high_nonzero = true;
                break;
            }
            i += 1;
        }

        (Self { num: low }, high_nonzero)
    }

    /// Wrapping multiplication (returns the low 320 bits of the full product).
    #[inline(always)]
    pub const fn wrapping_mul(self, rhs: Self) -> Self {
        // Direct unsigned multiplication of the raw bit patterns is
        // bitwise-identical to the signed wrapping product.
        self.unsigned_wrapping_mul(rhs)
    }

    /// Unsigned wrapping multiplication of the absolute values (low 320 bits only).
    #[inline(always)]
    const fn unsigned_wrapping_mul(self, rhs: Self) -> Self {
        let (low, _) = self.unsigned_mul_with_overflow_info(rhs);
        low
    }

    /// Saturating multiplication (clamps to `MIN` or `MAX` on overflow).
    #[inline]
    pub const fn saturating_mul(self, rhs: Self) -> Self {
        let (product, overflow) = self.overflowing_mul(rhs);
        if overflow {
            let result_neg = ((self.num[4] ^ rhs.num[4]) & (1u64 << 63)) != 0;
            if result_neg { Self::MIN } else { Self::MAX }
        } else {
            product
        }
    }

    /// Checked multiplication. Returns `None` on overflow.
    #[inline(always)]
    pub const fn checked_mul(self, rhs: Self) -> Option<Self> {
        let (product, overflow) = self.overflowing_mul(rhs);
        if overflow { None } else { Some(product) }
    }

    /// Returns the product and a boolean indicating whether overflow occurred.
    pub const fn overflowing_mul(self, rhs: Self) -> (Self, bool) {
        if self.is_zero() || rhs.is_zero() {
            return (Self::ZERO, false);
        }

        let self_neg = (self.num[4] & (1u64 << 63)) != 0;
        let rhs_neg = (rhs.num[4] & (1u64 << 63)) != 0;
        let result_neg = self_neg ^ rhs_neg;

        let abs_a = self.wrapping_abs();
        let abs_b = rhs.wrapping_abs();

        let (low_abs, high_nonzero) = abs_a.unsigned_mul_with_overflow_info(abs_b);

        let overflow = if high_nonzero {
            true
        } else if result_neg {
            // Negative result: allowed up to 2^319 (i.e. MIN)
            // Overflow if bit 319 is set AND the value is not exactly 2^319
            (low_abs.num[4] & (1u64 << 63)) != 0 && !low_abs.is_min()
        } else {
            // Positive result: allowed up to 2^319 - 1 (bit 319 must be clear)
            (low_abs.num[4] & (1u64 << 63)) != 0
        };

        let product = if result_neg {
            Self::neg(low_abs)
        } else {
            low_abs
        };

        (product, overflow)
    }
}

macro_rules! impl_mul_small {
    ($($ty:ty => $from_fn:ident),* $(,)?) => {
        $(
            // DtBig * small_int
            impl Mul<$ty> for DtBig {
                type Output = Self;

                #[inline(always)]
                fn mul(self, rhs: $ty) -> Self {
                    self.wrapping_mul(DtBig::$from_fn(rhs))
                }
            }

            impl MulAssign<$ty> for DtBig {
                #[inline(always)]
                fn mul_assign(&mut self, rhs: $ty) {
                    *self = *self * rhs;
                }
            }

            // small_int * DtBig
            impl Mul<DtBig> for $ty {
                type Output = DtBig;

                #[inline(always)]
                fn mul(self, rhs: DtBig) -> DtBig {
                    rhs.wrapping_mul(DtBig::$from_fn(self))
                }
            }

            // &DtBig versions for full consistency with the rest of the crate
            impl Mul<$ty> for &DtBig {
                type Output = DtBig;

                #[inline(always)]
                fn mul(self, rhs: $ty) -> DtBig {
                    *self * rhs
                }
            }

            impl Mul<&DtBig> for $ty {
                type Output = DtBig;

                #[inline(always)]
                fn mul(self, rhs: &DtBig) -> DtBig {
                    self * *rhs
                }
            }
        )*
    };
}

impl_mul_small! {
    i8  => from_i8,
    i16 => from_i16,
    i32 => from_i32,
    i64 => from_i64,
    u8  => from_u8,
    u16 => from_u16,
    u32 => from_u32,
    u64 => from_u64,
}

// ──────────────────────────────────────────────────────────────
// Division
// ──────────────────────────────────────────────────────────────

impl Div for DtBig {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            panic!("attempt to divide by zero")
        }
        self.wrapping_div(rhs)
    }
}

impl Div<&DtBig> for DtBig {
    type Output = Self;

    #[inline(always)]
    fn div(self, rhs: &DtBig) -> Self {
        self / *rhs
    }
}

impl Div<DtBig> for &DtBig {
    type Output = DtBig;

    #[inline(always)]
    fn div(self, rhs: DtBig) -> DtBig {
        *self / rhs
    }
}

impl Div<&DtBig> for &DtBig {
    type Output = DtBig;

    #[inline(always)]
    fn div(self, rhs: &DtBig) -> DtBig {
        *self / *rhs
    }
}

impl DivAssign for DtBig {
    #[inline(always)]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl DivAssign<&DtBig> for DtBig {
    #[inline(always)]
    fn div_assign(&mut self, rhs: &DtBig) {
        *self = *self / rhs;
    }
}

impl DtBig {
    /// Returns `Some(k)` if `self` is exactly `2^k` (`0 ≤ k < 320`), otherwise `None`.
    #[inline(always)]
    const fn is_power_of_two(self) -> Option<u32> {
        if self.is_zero() || self.count_ones() != 1 {
            None
        } else {
            Some(self.trailing_zeros())
        }
    }

    /// Fast division by a divisor that fits in a single `u64` limb.
    ///
    /// Uses native `u128` arithmetic. Covers the majority of real-world calls
    /// (e.g. repeated division by 10 inside `Display`).
    const fn div_by_single_limb(self, rhs: u64) -> Self {
        if rhs == 0 {
            return Self::ZERO;
        }
        if rhs == 1 {
            return self;
        }

        let mut quotient = Self::ZERO;
        let mut remainder = 0u64;
        let mut i = 4usize;
        while i != usize::MAX {
            let temp = ((remainder as u128) << 64) | (self.num[i] as u128);
            quotient.num[i] = (temp / rhs as u128) as u64;
            remainder = (temp % rhs as u128) as u64;
            if i == 0 {
                break;
            }
            i -= 1;
        }
        quotient
    }

    /// Unsigned wrapping division (internal helper).
    ///
    /// Dispatches to fast paths for the most common cases before falling back
    /// to the original bitwise long-division algorithm.
    pub const fn unsigned_wrapping_div(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            return Self::ZERO;
        }

        // Fast path: dividend < divisor (unsigned)
        if self.unsigned_less_than(rhs) {
            return Self::ZERO;
        }

        // Fast path: power-of-two divisor → simple shift
        if let Some(shift) = rhs.is_power_of_two() {
            return self.wrapping_shr(shift);
        }

        // Easy win #1: single-limb divisor
        // (replaces the original 4-iteration while loop with a single OR)
        let is_single_limb = (rhs.num[1] | rhs.num[2] | rhs.num[3] | rhs.num[4]) == 0;
        if is_single_limb {
            return self.div_by_single_limb(rhs.num[0]);
        }

        // General case: original proven-correct bitwise algorithm
        // (with easy win #2: compute word/mask only once per iteration)
        let mut quotient = Self::ZERO;
        let mut remainder = Self::ZERO;
        let mut bit = 319usize;
        while bit != usize::MAX {
            remainder = remainder.wrapping_shl(1);

            let word = bit / 64;
            let mask = 1u64 << (bit % 64);
            if (self.num[word] & mask) != 0 {
                remainder = remainder.wrapping_add(Self::ONE);
            }

            if !remainder.unsigned_less_than(rhs) {
                remainder = remainder.wrapping_sub(rhs);

                quotient.num[word] |= mask;
            }

            if bit == 0 {
                break;
            }
            bit -= 1;
        }
        quotient
    }

    /// Logical left shift (zero-filled).
    pub const fn wrapping_shl(self, rhs: u32) -> Self {
        let mut result = Self { num: [0; 5] };
        let shift = rhs as usize;
        if shift >= 320 {
            return Self::ZERO;
        }

        let word_shift = shift / 64;
        let bit_shift = shift % 64;

        let mut i = word_shift;
        while i < 5 {
            let src_idx = i - word_shift;
            let low = self.num[src_idx] << bit_shift;
            let high = if bit_shift != 0 && src_idx > 0 {
                self.num[src_idx - 1] >> (64 - bit_shift)
            } else {
                0
            };
            result.num[i] = low | high;
            i += 1;
        }
        result
    }

    /// Arithmetic right shift (sign-extending).
    pub const fn wrapping_shr(self, rhs: u32) -> Self {
        let mut result = Self { num: [0; 5] };
        let shift = rhs as usize;
        if shift >= 320 {
            return if (self.num[4] & (1u64 << 63)) != 0 {
                Self { num: [u64::MAX; 5] }
            } else {
                Self::ZERO
            };
        }

        let word_shift = shift / 64;
        let bit_shift = shift % 64;
        let sign_extend = if (self.num[4] & (1u64 << 63)) != 0 {
            u64::MAX
        } else {
            0
        };

        let mut i = 0usize;
        while i < 5 {
            let src_idx = i + word_shift;
            if src_idx >= 5 {
                result.num[i] = sign_extend;
            } else {
                let low = self.num[src_idx] >> bit_shift;
                let high = if bit_shift != 0 {
                    if src_idx + 1 < 5 {
                        self.num[src_idx + 1] << (64 - bit_shift)
                    } else {
                        sign_extend << (64 - bit_shift)
                    }
                } else {
                    0
                };
                result.num[i] = low | high;
            }
            i += 1;
        }
        result
    }

    /// Wrapping division (truncates toward zero).
    #[inline]
    pub const fn wrapping_div(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            return Self::ZERO;
        }
        let a_neg = (self.num[4] & (1u64 << 63)) != 0;
        let b_neg = (rhs.num[4] & (1u64 << 63)) != 0;
        let a = if a_neg { Self::neg(self) } else { self };
        let b = if b_neg { Self::neg(rhs) } else { rhs };
        let q = a.unsigned_wrapping_div(b);
        let result_neg = a_neg ^ b_neg;
        if result_neg { Self::neg(q) } else { q }
    }

    /// Saturating division (clamps to `MIN`/`MAX` on divide-by-zero or overflow).
    #[inline]
    pub const fn saturating_div(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            return if self.is_negative() {
                Self::MIN
            } else {
                Self::MAX
            };
        }
        // MIN / -1 is the only signed overflow case
        if self.is_min() && rhs.is_neg_one() {
            return Self::MAX;
        }
        self.wrapping_div(rhs)
    }

    /// Checked division. Returns `None` on divide-by-zero or MIN/-1 overflow.
    #[inline]
    pub const fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.is_zero() || (self.is_min() && rhs.is_neg_one()) {
            None
        } else {
            Some(self.wrapping_div(rhs))
        }
    }

    /// Returns the quotient and a boolean indicating whether an error occurred.
    #[inline]
    pub const fn overflowing_div(self, rhs: Self) -> (Self, bool) {
        if rhs.is_zero() {
            (Self::ZERO, true)
        } else if self.is_min() && rhs.is_neg_one() {
            (Self::MIN, true)
        } else {
            (self.wrapping_div(rhs), false)
        }
    }

    /// Helper for unsigned comparison (used in division).
    #[inline]
    const fn unsigned_less_than(self, other: Self) -> bool {
        let mut i = 4;
        while i != usize::MAX {
            if self.num[i] != other.num[i] {
                return self.num[i] < other.num[i];
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
        false
    }
}

macro_rules! impl_div_small {
    ($($ty:ty => $from_fn:ident),* $(,)?) => {
        $(
            // DtBig / small_int
            impl Div<$ty> for DtBig {
                type Output = Self;

                #[inline(always)]
                fn div(self, rhs: $ty) -> Self {
                    self.wrapping_div(DtBig::$from_fn(rhs))
                }
            }

            impl DivAssign<$ty> for DtBig {
                #[inline(always)]
                fn div_assign(&mut self, rhs: $ty) {
                    *self = *self / rhs;
                }
            }

            // small_int / DtBig
            impl Div<DtBig> for $ty {
                type Output = DtBig;

                #[inline(always)]
                fn div(self, rhs: DtBig) -> DtBig {
                    DtBig::$from_fn(self).wrapping_div(rhs)
                }
            }

            // Reference versions for full consistency with the rest of the crate
            impl Div<$ty> for &DtBig {
                type Output = DtBig;

                #[inline(always)]
                fn div(self, rhs: $ty) -> DtBig {
                    *self / rhs
                }
            }

            impl Div<&DtBig> for $ty {
                type Output = DtBig;

                #[inline(always)]
                fn div(self, rhs: &DtBig) -> DtBig {
                    self / *rhs
                }
            }
        )*
    };
}

impl_div_small! {
    i8  => from_i8,
    i16 => from_i16,
    i32 => from_i32,
    i64 => from_i64,
    u8  => from_u8,
    u16 => from_u16,
    u32 => from_u32,
    u64 => from_u64,
}

// ──────────────────────────────────────────────────────────────
// Remainder
// ──────────────────────────────────────────────────────────────

impl Rem for DtBig {
    type Output = Self;

    #[inline(always)]
    fn rem(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            panic!("attempt to divide by zero")
        }
        self.wrapping_rem(rhs)
    }
}

impl Rem<&DtBig> for DtBig {
    type Output = Self;

    #[inline(always)]
    fn rem(self, rhs: &DtBig) -> Self {
        self % *rhs
    }
}

impl Rem<DtBig> for &DtBig {
    type Output = DtBig;

    #[inline(always)]
    fn rem(self, rhs: DtBig) -> DtBig {
        *self % rhs
    }
}

impl Rem<&DtBig> for &DtBig {
    type Output = DtBig;

    #[inline(always)]
    fn rem(self, rhs: &DtBig) -> DtBig {
        *self % *rhs
    }
}

impl RemAssign for DtBig {
    #[inline(always)]
    fn rem_assign(&mut self, rhs: Self) {
        *self = *self % rhs;
    }
}

impl RemAssign<&DtBig> for DtBig {
    #[inline(always)]
    fn rem_assign(&mut self, rhs: &DtBig) {
        *self = *self % rhs;
    }
}

impl DtBig {
    /// Wrapping remainder (result has the same sign as the dividend).
    #[inline(always)]
    pub const fn wrapping_rem(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            return Self::ZERO;
        }
        // Use wrapping_sub instead of the `-` operator (required in const fn)
        self.wrapping_sub(self.wrapping_div(rhs).wrapping_mul(rhs))
    }

    /// Saturating remainder.
    #[inline(always)]
    pub const fn saturating_rem(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            return Self::ZERO;
        }
        self.wrapping_rem(rhs)
    }

    /// Checked remainder. Returns `None` on divide-by-zero.
    #[inline(always)]
    pub const fn checked_rem(self, rhs: Self) -> Option<Self> {
        if rhs.is_zero() {
            None
        } else {
            Some(self.wrapping_rem(rhs))
        }
    }

    /// Returns the remainder and a boolean indicating whether an error
    /// (divide-by-zero) occurred.
    #[inline(always)]
    pub const fn overflowing_rem(self, rhs: Self) -> (Self, bool) {
        if rhs.is_zero() {
            (Self::ZERO, true)
        } else {
            (self.wrapping_rem(rhs), false)
        }
    }
}

macro_rules! impl_rem_small {
    ($($ty:ty => $from_fn:ident),* $(,)?) => {
        $(
            // DtBig % small_int
            impl Rem<$ty> for DtBig {
                type Output = Self;

                #[inline(always)]
                fn rem(self, rhs: $ty) -> Self {
                    self.wrapping_rem(DtBig::$from_fn(rhs))
                }
            }

            impl RemAssign<$ty> for DtBig {
                #[inline(always)]
                fn rem_assign(&mut self, rhs: $ty) {
                    *self = *self % rhs;
                }
            }

            // small_int % DtBig
            impl Rem<DtBig> for $ty {
                type Output = DtBig;

                #[inline(always)]
                fn rem(self, rhs: DtBig) -> DtBig {
                    DtBig::$from_fn(self).wrapping_rem(rhs)
                }
            }

            // Reference versions for full consistency with the rest of the crate
            impl Rem<$ty> for &DtBig {
                type Output = DtBig;

                #[inline(always)]
                fn rem(self, rhs: $ty) -> DtBig {
                    *self % rhs
                }
            }

            impl Rem<&DtBig> for $ty {
                type Output = DtBig;

                #[inline(always)]
                fn rem(self, rhs: &DtBig) -> DtBig {
                    self % *rhs
                }
            }
        )*
    };
}

impl_rem_small! {
    i8  => from_i8,
    i16 => from_i16,
    i32 => from_i32,
    i64 => from_i64,
    u8  => from_u8,
    u16 => from_u16,
    u32 => from_u32,
    u64 => from_u64,
}

// ──────────────────────────────────────────────────────────────
// Shifts
// ──────────────────────────────────────────────────────────────

impl Shl<u32> for DtBig {
    type Output = Self;

    #[inline(always)]
    fn shl(self, rhs: u32) -> Self {
        self.wrapping_shl(rhs)
    }
}

impl ShlAssign<u32> for DtBig {
    #[inline(always)]
    fn shl_assign(&mut self, rhs: u32) {
        *self = *self << rhs;
    }
}

impl Shr<u32> for DtBig {
    type Output = Self;

    #[inline(always)]
    fn shr(self, rhs: u32) -> Self {
        self.wrapping_shr(rhs)
    }
}

impl ShrAssign<u32> for DtBig {
    #[inline(always)]
    fn shr_assign(&mut self, rhs: u32) {
        *self = *self >> rhs;
    }
}

// ──────────────────────────────────────────────────────────────
// Conversions from smaller integer and floating-point types
// ──────────────────────────────────────────────────────────────

impl From<i8> for DtBig {
    #[inline(always)]
    fn from(value: i8) -> Self {
        Self::from_i8(value)
    }
}

impl From<i16> for DtBig {
    #[inline(always)]
    fn from(value: i16) -> Self {
        Self::from_i16(value)
    }
}

impl From<i32> for DtBig {
    #[inline(always)]
    fn from(value: i32) -> Self {
        Self::from_i32(value)
    }
}

impl From<i64> for DtBig {
    #[inline(always)]
    fn from(value: i64) -> Self {
        Self::from_i64(value)
    }
}

impl From<u8> for DtBig {
    #[inline(always)]
    fn from(value: u8) -> Self {
        Self::from_u8(value)
    }
}

impl From<u16> for DtBig {
    #[inline(always)]
    fn from(value: u16) -> Self {
        Self::from_u16(value)
    }
}

impl From<u32> for DtBig {
    #[inline(always)]
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl From<u64> for DtBig {
    #[inline(always)]
    fn from(value: u64) -> Self {
        Self::from_u64(value)
    }
}

impl From<i128> for DtBig {
    #[inline(always)]
    fn from(value: i128) -> Self {
        Self::from_i128(value)
    }
}

impl From<u128> for DtBig {
    #[inline(always)]
    fn from(value: u128) -> Self {
        Self::from_u128(value)
    }
}

impl From<[u64; 5]> for DtBig {
    #[inline(always)]
    fn from(num: [u64; 5]) -> Self {
        Self::from_num(num)
    }
}

impl From<DtBig> for [u64; 5] {
    #[inline(always)]
    fn from(dt: DtBig) -> Self {
        dt.num
    }
}

impl From<usize> for DtBig {
    #[inline(always)]
    fn from(value: usize) -> Self {
        Self::from_usize(value)
    }
}

impl From<isize> for DtBig {
    #[inline(always)]
    fn from(value: isize) -> Self {
        Self::from_isize(value)
    }
}

impl DtBig {
    /// Creates a new `DtBig` from an `i8` (const fn).
    #[inline(always)]
    pub const fn from_i8(value: i8) -> Self {
        let sign = if value < 0 { u64::MAX } else { 0 };
        let mut num = [sign; 5];
        num[0] = value as u64;
        Self { num }
    }

    /// Creates a new `DtBig` from an `i16` (const fn).
    #[inline(always)]
    pub const fn from_i16(value: i16) -> Self {
        let sign = if value < 0 { u64::MAX } else { 0 };
        let mut num = [sign; 5];
        num[0] = value as u64;
        Self { num }
    }

    /// Creates a new `DtBig` from an `i32` (const fn).
    #[inline(always)]
    pub const fn from_i32(value: i32) -> Self {
        let sign = if value < 0 { u64::MAX } else { 0 };
        let mut num = [sign; 5];
        num[0] = value as u64;
        Self { num }
    }

    /// Creates a new `DtBig` from an `i64` (const fn).
    #[inline(always)]
    pub const fn from_i64(value: i64) -> Self {
        let sign = if value < 0 { u64::MAX } else { 0 };
        let mut num = [sign; 5];
        num[0] = value as u64;
        Self { num }
    }

    /// Creates a new `DtBig` from a `u8` (const fn).
    #[inline(always)]
    pub const fn from_u8(value: u8) -> Self {
        let mut num = [0u64; 5];
        num[0] = value as u64;
        Self { num }
    }

    /// Creates a new `DtBig` from a `u16` (const fn).
    #[inline(always)]
    pub const fn from_u16(value: u16) -> Self {
        let mut num = [0u64; 5];
        num[0] = value as u64;
        Self { num }
    }

    /// Creates a new `DtBig` from a `u32` (const fn).
    #[inline(always)]
    pub const fn from_u32(value: u32) -> Self {
        let mut num = [0u64; 5];
        num[0] = value as u64;
        Self { num }
    }

    /// Creates a new `DtBig` from a `u64` (const fn) — the canonical fast path
    /// used by `From<f64>` and everywhere else.
    #[inline(always)]
    pub const fn from_u64(value: u64) -> Self {
        let mut num = [0u64; 5];
        num[0] = value;
        Self { num }
    }

    /// const-friendly conversion from `i128` (correct two’s-complement sign extension).
    #[inline(always)]
    pub const fn from_i128(value: i128) -> Self {
        let sign = if value < 0 { u64::MAX } else { 0 };
        let mut num = [sign; 5];
        num[0] = value as u64;
        num[1] = (value >> 64) as u64;
        Self { num }
    }

    /// const-friendly conversion from `u128`.
    #[inline(always)]
    pub const fn from_u128(value: u128) -> Self {
        let mut num = [0u64; 5];
        num[0] = value as u64;
        num[1] = (value >> 64) as u64;
        Self { num }
    }

    /// Creates a new `DtBig` from a `usize` (const fn).
    ///
    /// Delegates to the canonical `from_u64` path (works on both 32-bit and 64-bit targets).
    #[inline(always)]
    pub const fn from_usize(value: usize) -> Self {
        Self::from_u64(value as u64)
    }

    /// Creates a new `DtBig` from an `isize` (const fn).
    ///
    /// Delegates to the canonical `from_i64` path (correct sign extension on both 32-bit and 64-bit targets).
    #[inline(always)]
    pub const fn from_isize(value: isize) -> Self {
        Self::from_i64(value as i64)
    }

    /// Saturating conversion to `i128` (clamps to `i128::MIN` / `i128::MAX`).
    /// Fully `const fn`, direct limb inspection.
    #[inline]
    pub const fn to_i128_saturating(self) -> i128 {
        let limbs = self.num;
        let is_negative = (limbs[4] as i64) < 0;

        if is_negative {
            if limbs[2] == u64::MAX && limbs[3] == u64::MAX && limbs[4] == u64::MAX {
                let low = limbs[0] as u128;
                let high = limbs[1] as u128;
                let val = ((high << 64) | low) as i128;
                if val >= i128::MIN { val } else { i128::MIN }
            } else {
                i128::MIN
            }
        } else {
            if limbs[2] == 0 && limbs[3] == 0 && limbs[4] == 0 {
                let low = limbs[0] as u128;
                let high = limbs[1] as u128;
                let val = ((high << 64) | low) as i128;
                if val <= i128::MAX { val } else { i128::MAX }
            } else {
                i128::MAX
            }
        }
    }

    /// Saturating conversion to `u128` (clamps to `u128::MAX`; negative → 0).
    /// Fully `const fn`, direct limb inspection.
    #[inline]
    pub const fn to_u128_saturating(self) -> u128 {
        if self.is_negative() {
            0
        } else {
            let limbs = self.num;
            if limbs[2] == 0 && limbs[3] == 0 && limbs[4] == 0 {
                let low = limbs[0] as u128;
                let high = limbs[1] as u128;
                (high << 64) | low
            } else {
                u128::MAX
            }
        }
    }
}

// ──────────────────────────────────────────────────────────────
// TryFrom / TryInto support (DtBig → primitive integers)
// ──────────────────────────────────────────────────────────────

/// Error returned when a `DtBig` value cannot be represented by the target
/// primitive integer type (i.e. it is out of range).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TryFromDtBigError;

impl core::fmt::Display for TryFromDtBigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("out of range integral type conversion attempted")
    }
}

#[cfg(feature = "std")]
impl core::error::Error for TryFromDtBigError {}

impl TryFrom<DtBig> for i8 {
    type Error = TryFromDtBigError;

    #[inline(always)]
    fn try_from(v: DtBig) -> Result<Self, Self::Error> {
        v.try_to_i8()
    }
}
impl TryFrom<DtBig> for i16 {
    type Error = TryFromDtBigError;

    #[inline(always)]
    fn try_from(v: DtBig) -> Result<Self, Self::Error> {
        v.try_to_i16()
    }
}
impl TryFrom<DtBig> for i32 {
    type Error = TryFromDtBigError;

    #[inline(always)]
    fn try_from(v: DtBig) -> Result<Self, Self::Error> {
        v.try_to_i32()
    }
}
impl TryFrom<DtBig> for i64 {
    type Error = TryFromDtBigError;

    #[inline(always)]
    fn try_from(v: DtBig) -> Result<Self, Self::Error> {
        v.try_to_i64()
    }
}

impl TryFrom<DtBig> for u8 {
    type Error = TryFromDtBigError;

    #[inline(always)]
    fn try_from(v: DtBig) -> Result<Self, Self::Error> {
        v.try_to_u8()
    }
}
impl TryFrom<DtBig> for u16 {
    type Error = TryFromDtBigError;

    #[inline(always)]
    fn try_from(v: DtBig) -> Result<Self, Self::Error> {
        v.try_to_u16()
    }
}
impl TryFrom<DtBig> for u32 {
    type Error = TryFromDtBigError;

    #[inline(always)]
    fn try_from(v: DtBig) -> Result<Self, Self::Error> {
        v.try_to_u32()
    }
}
impl TryFrom<DtBig> for u64 {
    type Error = TryFromDtBigError;

    #[inline(always)]
    fn try_from(v: DtBig) -> Result<Self, Self::Error> {
        v.try_to_u64()
    }
}

impl TryFrom<DtBig> for i128 {
    type Error = TryFromDtBigError;

    #[inline(always)]
    fn try_from(v: DtBig) -> Result<Self, Self::Error> {
        v.try_to_i128()
    }
}
impl TryFrom<DtBig> for u128 {
    type Error = TryFromDtBigError;

    #[inline(always)]
    fn try_from(v: DtBig) -> Result<Self, Self::Error> {
        v.try_to_u128()
    }
}

impl TryFrom<DtBig> for usize {
    type Error = TryFromDtBigError;

    #[inline(always)]
    fn try_from(v: DtBig) -> Result<Self, Self::Error> {
        v.try_to_usize()
    }
}

impl TryFrom<DtBig> for isize {
    type Error = TryFromDtBigError;

    #[inline(always)]
    fn try_from(v: DtBig) -> Result<Self, Self::Error> {
        v.try_to_isize()
    }
}

impl DtBig {
    /// Tries to convert this `DtBig` to `i8`. Returns `Err(TryFromDtBigError)` if out of range.
    #[inline]
    pub const fn try_to_i8(self) -> Result<i8, TryFromDtBigError> {
        let min = Self::from_i8(i8::MIN);
        let max = Self::from_i8(i8::MAX);
        match self.cmp(min) {
            core::cmp::Ordering::Less => Err(TryFromDtBigError),
            _ => match self.cmp(max) {
                core::cmp::Ordering::Greater => Err(TryFromDtBigError),
                _ => Ok(self.num[0] as i8),
            },
        }
    }

    #[inline]
    pub const fn try_to_i16(self) -> Result<i16, TryFromDtBigError> {
        let min = Self::from_i16(i16::MIN);
        let max = Self::from_i16(i16::MAX);
        match self.cmp(min) {
            core::cmp::Ordering::Less => Err(TryFromDtBigError),
            _ => match self.cmp(max) {
                core::cmp::Ordering::Greater => Err(TryFromDtBigError),
                _ => Ok(self.num[0] as i16),
            },
        }
    }

    #[inline]
    pub const fn try_to_i32(self) -> Result<i32, TryFromDtBigError> {
        let min = Self::from_i32(i32::MIN);
        let max = Self::from_i32(i32::MAX);
        match self.cmp(min) {
            core::cmp::Ordering::Less => Err(TryFromDtBigError),
            _ => match self.cmp(max) {
                core::cmp::Ordering::Greater => Err(TryFromDtBigError),
                _ => Ok(self.num[0] as i32),
            },
        }
    }

    #[inline]
    pub const fn try_to_i64(self) -> Result<i64, TryFromDtBigError> {
        let min = Self::from_i64(i64::MIN);
        let max = Self::from_i64(i64::MAX);
        match self.cmp(min) {
            core::cmp::Ordering::Less => Err(TryFromDtBigError),
            _ => match self.cmp(max) {
                core::cmp::Ordering::Greater => Err(TryFromDtBigError),
                _ => Ok(self.num[0] as i64),
            },
        }
    }

    #[inline]
    pub const fn try_to_u8(self) -> Result<u8, TryFromDtBigError> {
        if self.is_negative()
            || matches!(
                self.cmp(Self::from_u8(u8::MAX)),
                core::cmp::Ordering::Greater
            )
        {
            Err(TryFromDtBigError)
        } else {
            Ok(self.num[0] as u8)
        }
    }

    #[inline]
    pub const fn try_to_u16(self) -> Result<u16, TryFromDtBigError> {
        if self.is_negative()
            || matches!(
                self.cmp(Self::from_u16(u16::MAX)),
                core::cmp::Ordering::Greater
            )
        {
            Err(TryFromDtBigError)
        } else {
            Ok(self.num[0] as u16)
        }
    }

    #[inline]
    pub const fn try_to_u32(self) -> Result<u32, TryFromDtBigError> {
        if self.is_negative()
            || matches!(
                self.cmp(Self::from_u32(u32::MAX)),
                core::cmp::Ordering::Greater
            )
        {
            Err(TryFromDtBigError)
        } else {
            Ok(self.num[0] as u32)
        }
    }

    #[inline]
    pub const fn try_to_u64(self) -> Result<u64, TryFromDtBigError> {
        if self.is_negative()
            || matches!(
                self.cmp(Self::from_u64(u64::MAX)),
                core::cmp::Ordering::Greater
            )
        {
            Err(TryFromDtBigError)
        } else {
            Ok(self.num[0] as u64)
        }
    }

    #[inline]
    pub const fn try_to_i128(self) -> Result<i128, TryFromDtBigError> {
        let min = Self::from_i128(i128::MIN);
        let max = Self::from_i128(i128::MAX);
        match self.cmp(min) {
            core::cmp::Ordering::Less => Err(TryFromDtBigError),
            _ => match self.cmp(max) {
                core::cmp::Ordering::Greater => Err(TryFromDtBigError),
                _ => {
                    let low = self.num[0] as u128;
                    let high = self.num[1] as u128;
                    Ok(((high << 64) | low) as i128)
                }
            },
        }
    }

    #[inline]
    pub const fn try_to_u128(self) -> Result<u128, TryFromDtBigError> {
        if self.is_negative()
            || matches!(
                self.cmp(Self::from_u128(u128::MAX)),
                core::cmp::Ordering::Greater
            )
        {
            Err(TryFromDtBigError)
        } else {
            let low = self.num[0] as u128;
            let high = self.num[1] as u128;
            Ok((high << 64) | low)
        }
    }

    /// Tries to convert this `DtBig` to `usize`. Returns `Err(TryFromDtBigError)` if out of range.
    #[inline]
    pub const fn try_to_usize(self) -> Result<usize, TryFromDtBigError> {
        if self.is_negative()
            || matches!(
                self.cmp(Self::from_u64(usize::MAX as u64)),
                core::cmp::Ordering::Greater
            )
        {
            Err(TryFromDtBigError)
        } else {
            Ok(self.num[0] as usize)
        }
    }

    /// Tries to convert this `DtBig` to `isize`. Returns `Err(TryFromDtBigError)` if out of range.
    #[inline]
    pub const fn try_to_isize(self) -> Result<isize, TryFromDtBigError> {
        let min = Self::from_isize(isize::MIN);
        let max = Self::from_isize(isize::MAX);
        match self.cmp(min) {
            core::cmp::Ordering::Less => Err(TryFromDtBigError),
            _ => match self.cmp(max) {
                core::cmp::Ordering::Greater => Err(TryFromDtBigError),
                _ => Ok(self.num[0] as isize),
            },
        }
    }

    /// Creates a new `DtBig` from a `f64` by truncating toward zero.
    ///
    /// - NaN becomes `ZERO`
    /// - +∞ becomes `MAX`
    /// - -∞ becomes `MIN`
    /// - Values with |x| < 1.0 become `ZERO`
    pub const fn from_f64(value: f64) -> Self {
        if value.is_nan() {
            return Self::ZERO;
        }
        if value.is_infinite() {
            return if value.is_sign_positive() {
                Self::MAX
            } else {
                Self::MIN
            };
        }

        let is_negative = value.is_sign_negative();
        let abs = value.abs();

        if abs < 1.0 {
            return Self::ZERO;
        }

        // Fast path: fits safely in i128 (exact, no precision loss)
        if abs <= i128::MAX as f64 {
            let n = abs as i128;
            let dt = Self::from_i128(n);
            return if is_negative { dt.neg() } else { dt };
        }

        // Large values (|x| > 2^127): decompose IEEE 754 double bits
        let bits = abs.to_bits();
        let biased_exp = ((bits >> 52) & 0x7FF) as i32;
        let mant_bits = bits & 0x000F_FFFF_FFFF_FFFF;

        if biased_exp == 0 {
            return Self::ZERO; // subnormal
        }

        let mantissa = (1u64 << 52) | mant_bits;
        let exponent = biased_exp - 1023 - 52;

        if exponent + 52 >= 319 {
            return if is_negative { Self::MIN } else { Self::MAX };
        }

        // Use the canonical fast from_u64 path
        let mant_dt = Self::from_u64(mantissa);

        let result = if exponent >= 0 {
            mant_dt.wrapping_shl(exponent as u32)
        } else {
            mant_dt.wrapping_shr((-exponent) as u32)
        };

        if is_negative { result.neg() } else { result }
    }
}

impl From<f64> for DtBig {
    #[inline(always)]
    fn from(value: f64) -> Self {
        Self::from_f64(value)
    }
}

// ──────────────────────────────────────────────────────────────
// Bitwise operations
// ──────────────────────────────────────────────────────────────

impl BitAnd for DtBig {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: Self) -> Self {
        Self::bitand(self, rhs)
    }
}

impl BitAnd<&DtBig> for DtBig {
    type Output = Self;

    #[inline(always)]
    fn bitand(self, rhs: &DtBig) -> Self {
        self & *rhs
    }
}

impl BitAndAssign for DtBig {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl BitAndAssign<&DtBig> for DtBig {
    #[inline(always)]
    fn bitand_assign(&mut self, rhs: &DtBig) {
        *self = *self & rhs;
    }
}

impl BitOr for DtBig {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: Self) -> Self {
        Self::bitor(self, rhs)
    }
}

impl BitOr<&DtBig> for DtBig {
    type Output = Self;

    #[inline(always)]
    fn bitor(self, rhs: &DtBig) -> Self {
        self | *rhs
    }
}

impl BitOrAssign for DtBig {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

impl BitOrAssign<&DtBig> for DtBig {
    #[inline(always)]
    fn bitor_assign(&mut self, rhs: &DtBig) {
        *self = *self | rhs;
    }
}

impl BitXor for DtBig {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: Self) -> Self {
        Self::bitxor(self, rhs)
    }
}

impl BitXor<&DtBig> for DtBig {
    type Output = Self;

    #[inline(always)]
    fn bitxor(self, rhs: &DtBig) -> Self {
        self ^ *rhs
    }
}

impl BitXorAssign for DtBig {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = *self ^ rhs;
    }
}

impl BitXorAssign<&DtBig> for DtBig {
    #[inline(always)]
    fn bitxor_assign(&mut self, rhs: &DtBig) {
        *self = *self ^ rhs;
    }
}

impl Not for DtBig {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self {
        Self::not(self)
    }
}

// ──────────────────────────────────────────────────────────────
// Convenience methods
// ──────────────────────────────────────────────────────────────

const TWO_POW_64: f64 = (1u128 << 64) as f64;

impl DtBig {
    /// Bitwise AND (constant-time, usable in `const` contexts).
    pub const fn bitand(self, rhs: Self) -> Self {
        let mut result = Self { num: [0; 5] };
        let mut i = 0;
        while i < 5 {
            result.num[i] = self.num[i] & rhs.num[i];
            i += 1;
        }
        result
    }

    /// Bitwise OR (constant-time, usable in `const` contexts).
    pub const fn bitor(self, rhs: Self) -> Self {
        let mut result = Self { num: [0; 5] };
        let mut i = 0;
        while i < 5 {
            result.num[i] = self.num[i] | rhs.num[i];
            i += 1;
        }
        result
    }

    /// Bitwise XOR (constant-time, usable in `const` contexts).
    pub const fn bitxor(self, rhs: Self) -> Self {
        let mut result = Self { num: [0; 5] };
        let mut i = 0;
        while i < 5 {
            result.num[i] = self.num[i] ^ rhs.num[i];
            i += 1;
        }
        result
    }

    /// Bitwise NOT (constant-time, usable in `const` contexts).
    pub const fn not(self) -> Self {
        let mut result = Self { num: [0; 5] };
        let mut i = 0;
        while i < 5 {
            result.num[i] = !self.num[i];
            i += 1;
        }
        result
    }

    /// Shifts the bits left by `rhs` positions, with wrap around.
    pub const fn rotate_left(self, rhs: u32) -> Self {
        let shift = (rhs % 320) as usize;
        if shift == 0 {
            return self;
        }
        let high = self.wrapping_shl(shift as u32);
        let low = self.wrapping_shr((320 - shift) as u32);
        // Use the inherent const `bitor` (the `|` operator cannot be used
        // inside `const fn` because `BitOr::bitor` is not declared `const`).
        Self::bitor(high, low)
    }

    /// Shifts the bits right by `rhs` positions, with wrap around.
    pub const fn rotate_right(self, rhs: u32) -> Self {
        let shift = (rhs % 320) as usize;
        if shift == 0 {
            return self;
        }
        let low = self.wrapping_shr(shift as u32);
        let high = self.wrapping_shl((320 - shift) as u32);
        // Use the inherent const `bitor`.
        Self::bitor(high, low)
    }

    /// Returns the sign of the number: `1` if positive, `0` if zero, `-1` if negative.
    pub const fn signum(self) -> Self {
        if self.is_zero() {
            Self::ZERO
        } else if (self.num[4] & (1u64 << 63)) != 0 {
            Self::from_num([u64::MAX; 5]) // -1
        } else {
            Self::ONE
        }
    }

    /// Returns `true` if the number is positive (greater than zero).
    #[inline(always)]
    pub const fn is_positive(self) -> bool {
        !self.is_zero() && (self.num[4] & (1u64 << 63)) == 0
    }

    /// Returns `true` if the number is negative (less than zero).
    #[inline(always)]
    pub const fn is_negative(self) -> bool {
        (self.num[4] & (1u64 << 63)) != 0
    }

    /// Absolute value (wrapping at `MIN`).
    #[inline(always)]
    pub const fn wrapping_abs(self) -> Self {
        if (self.num[4] & (1u64 << 63)) != 0 {
            Self::neg(self)
        } else {
            self
        }
    }

    /// Absolute value (clamps `MIN` to `MAX`).
    #[inline(always)]
    pub const fn saturating_abs(self) -> Self {
        if (self.num[4] & (1u64 << 63)) != 0 && self.wrapping_abs().is_zero() {
            Self::MAX
        } else {
            self.wrapping_abs()
        }
    }

    /// Absolute value. Returns `None` if the value is `MIN`.
    #[inline(always)]
    pub const fn checked_abs(self) -> Option<Self> {
        if (self.num[4] & (1u64 << 63)) != 0 && self.wrapping_abs().is_zero() {
            None
        } else {
            Some(self.wrapping_abs())
        }
    }

    /// Returns the absolute value and a boolean indicating whether overflow occurred.
    #[inline(always)]
    pub const fn overflowing_abs(self) -> (Self, bool) {
        if (self.num[4] & (1u64 << 63)) != 0 && self.wrapping_abs().is_zero() {
            (Self::MAX, true)
        } else {
            (self.wrapping_abs(), false)
        }
    }

    /// Number of leading zeros in the binary representation.
    #[inline]
    pub const fn leading_zeros(self) -> u32 {
        let mut count = 0u32;
        let mut i = 4;
        while i != usize::MAX {
            let zeros = self.num[i].leading_zeros();
            count += zeros;
            if zeros < 64 {
                break;
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
        count
    }

    /// Number of trailing zeros in the binary representation.
    #[inline]
    pub const fn trailing_zeros(self) -> u32 {
        let mut count = 0u32;
        let mut i = 0;
        while i < 5 {
            let zeros = self.num[i].trailing_zeros();
            count += zeros;
            if zeros < 64 {
                break;
            }
            i += 1;
        }
        count
    }

    /// Number of leading ones in the binary representation.
    #[inline]
    pub const fn leading_ones(self) -> u32 {
        let mut count = 0u32;
        let mut i = 4;
        while i != usize::MAX {
            let ones = self.num[i].leading_ones();
            count += ones;
            if ones < 64 {
                break;
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
        count
    }

    /// Number of trailing ones in the binary representation.
    #[inline]
    pub const fn trailing_ones(self) -> u32 {
        let mut count = 0u32;
        let mut i = 0;
        while i < 5 {
            let ones = self.num[i].trailing_ones();
            count += ones;
            if ones < 64 {
                break;
            }
            i += 1;
        }
        count
    }

    /// Returns the number of ones in the binary representation of `self`.
    #[inline]
    pub const fn count_ones(self) -> u32 {
        let mut count = 0u32;
        let mut i = 0;
        while i < 5 {
            count += self.num[i].count_ones();
            i += 1;
        }
        count
    }

    /// Returns the number of zeros in the binary representation of `self`.
    #[inline(always)]
    pub const fn count_zeros(self) -> u32 {
        320 - self.count_ones()
    }

    /// Converts to `f64` (lossy, rounds toward zero).
    #[inline]
    pub const fn to_f64(self) -> f64 {
        if self.is_zero() {
            return 0.0;
        }
        let is_neg = self.is_negative();
        let abs_val = self.wrapping_abs();
        let mut result = 0.0f64;
        // 2^64 is exactly representable in f64
        let mut i = 4usize;
        while i != usize::MAX {
            result = result * TWO_POW_64 + (abs_val.num[i] as f64);
            if i == 0 {
                break;
            }
            i -= 1;
        }
        if is_neg { -result } else { result }
    }

    /// Returns the little-endian byte representation (40 bytes).
    pub const fn to_le_bytes(self) -> [u8; 40] {
        let mut bytes = [0u8; 40];
        let mut i = 0;
        while i < 5 {
            let limb = self.num[i];
            let offset = i * 8;
            bytes[offset] = (limb & 0xff) as u8;
            bytes[offset + 1] = ((limb >> 8) & 0xff) as u8;
            bytes[offset + 2] = ((limb >> 16) & 0xff) as u8;
            bytes[offset + 3] = ((limb >> 24) & 0xff) as u8;
            bytes[offset + 4] = ((limb >> 32) & 0xff) as u8;
            bytes[offset + 5] = ((limb >> 40) & 0xff) as u8;
            bytes[offset + 6] = ((limb >> 48) & 0xff) as u8;
            bytes[offset + 7] = ((limb >> 56) & 0xff) as u8;
            i += 1;
        }
        bytes
    }

    /// Creates a `DtBig` from its little-endian byte representation.
    pub const fn from_le_bytes(bytes: [u8; 40]) -> Self {
        let mut num = [0u64; 5];
        let mut i = 0;
        while i < 5 {
            let offset = i * 8;
            num[i] = (bytes[offset] as u64)
                | ((bytes[offset + 1] as u64) << 8)
                | ((bytes[offset + 2] as u64) << 16)
                | ((bytes[offset + 3] as u64) << 24)
                | ((bytes[offset + 4] as u64) << 32)
                | ((bytes[offset + 5] as u64) << 40)
                | ((bytes[offset + 6] as u64) << 48)
                | ((bytes[offset + 7] as u64) << 56);
            i += 1;
        }
        Self { num }
    }

    /// Creates a `DtBig` from its big-endian byte representation.
    pub const fn from_be_bytes(bytes: [u8; 40]) -> Self {
        let mut num = [0u64; 5];
        let mut i = 0;
        while i < 5 {
            let offset = (4 - i) * 8; // highest limb first
            let limb = ((bytes[offset] as u64) << 56)
                | ((bytes[offset + 1] as u64) << 48)
                | ((bytes[offset + 2] as u64) << 40)
                | ((bytes[offset + 3] as u64) << 32)
                | ((bytes[offset + 4] as u64) << 24)
                | ((bytes[offset + 5] as u64) << 16)
                | ((bytes[offset + 6] as u64) << 8)
                | (bytes[offset + 7] as u64);
            num[i] = limb;
            i += 1;
        }
        Self { num }
    }

    /// Returns the big-endian byte representation (40 bytes).
    pub const fn to_be_bytes(self) -> [u8; 40] {
        let mut bytes = [0u8; 40];
        let mut i = 0;
        while i < 5 {
            let limb = self.num[4 - i]; // highest limb first
            let offset = i * 8;
            bytes[offset] = (limb >> 56) as u8;
            bytes[offset + 1] = (limb >> 48) as u8;
            bytes[offset + 2] = (limb >> 40) as u8;
            bytes[offset + 3] = (limb >> 32) as u8;
            bytes[offset + 4] = (limb >> 24) as u8;
            bytes[offset + 5] = (limb >> 16) as u8;
            bytes[offset + 6] = (limb >> 8) as u8;
            bytes[offset + 7] = limb as u8;
            i += 1;
        }
        bytes
    }

    pub const fn swap_bytes(self) -> Self {
        let mut num = self.num;
        // reverse byte order inside each u64
        let mut i = 0;
        while i < 5 {
            num[i] = num[i].swap_bytes();
            i += 1;
        }
        Self { num }
    }

    pub const fn reverse_bits(self) -> Self {
        let mut num = self.num;
        let mut i = 0;
        while i < 5 {
            num[i] = num[i].reverse_bits();
            i += 1;
        }
        // now reverse the order of the limbs
        let mut result = Self { num: [0; 5] };
        let mut j = 0;
        while j < 5 {
            result.num[j] = num[4 - j];
            j += 1;
        }
        result
    }
}

// ──────────────────────────────────────────────────────────────
// Formatting (Display + Hex)
// ──────────────────────────────────────────────────────────────

impl Display for DtBig {
    /// Formats the number in decimal (lossless, exact).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_zero() {
            return f.write_str("0");
        }

        let negative = self.is_negative();

        // Special case for MIN because wrapping_abs(MIN) == MIN and we don't want to loop forever
        if negative && self.is_min() {
            return f.write_str(
                "-1067993517960455041197510853084776057301352261178326384973520803911109862890320275011481043468288",
            );
        }

        let mut n = if negative { self.wrapping_abs() } else { *self };

        let mut buffer = [0u8; 200]; // enough for 2^319 (97 digits)
        let mut pos = buffer.len();
        let ten = DtBig::from(10u8);

        while !n.is_zero() {
            let rem = n.wrapping_rem(ten);
            pos -= 1;
            buffer[pos] = b'0' + (rem.num[0] as u8); // remainder is always 0-9
            n = n.wrapping_div(ten);
        }

        if negative {
            pos -= 1;
            buffer[pos] = b'-';
        }

        // Safety: we only ever write ASCII digits + optional '-'
        let s = core::str::from_utf8(&buffer[pos..]).map_err(|_| fmt::Error)?;
        f.write_str(s)
    }
}

impl LowerHex for DtBig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("0x")?;
        let mut started = false;
        for &limb in self.num.iter().rev() {
            if started {
                write!(f, "{:016x}", limb)?;
            } else if limb != 0 {
                write!(f, "{:x}", limb)?;
                started = true;
            }
        }
        if !started {
            f.write_str("0")?;
        }
        Ok(())
    }
}

impl UpperHex for DtBig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("0x")?;
        let mut started = false;
        for &limb in self.num.iter().rev() {
            if started {
                write!(f, "{:016X}", limb)?;
            } else if limb != 0 {
                write!(f, "{:X}", limb)?;
                started = true;
            }
        }
        if !started {
            f.write_str("0")?;
        }
        Ok(())
    }
}

// ──────────────────────────────────────────────────────────────
// POW
// ──────────────────────────────────────────────────────────────

impl DtBig {
    /// Raises `self` to the power of `exp`, wrapping on overflow
    /// (returns the low 320 bits of the full mathematical result).
    pub const fn wrapping_pow(self, exp: u32) -> Self {
        if exp == 0 {
            return Self::ONE;
        }
        if self.is_zero() {
            return Self::ZERO;
        }

        let mut base = self;
        let mut result = Self::ONE;
        let mut e = exp;

        while e > 0 {
            if (e & 1) != 0 {
                result = result.wrapping_mul(base);
            }
            base = base.wrapping_mul(base);
            e >>= 1;
        }
        result
    }

    /// Alias for `wrapping_pow` (matches the primitive integer API exactly).
    #[inline(always)]
    pub const fn pow(self, exp: u32) -> Self {
        self.wrapping_pow(exp)
    }

    /// Returns the power and a flag indicating whether overflow occurred.
    pub const fn overflowing_pow(self, exp: u32) -> (Self, bool) {
        if exp == 0 {
            return (Self::ONE, false);
        }
        if self.is_zero() {
            return (Self::ZERO, false);
        }

        let mut base = self;
        let mut result = Self::ONE;
        let mut e = exp;
        let mut overflow = false;

        while e > 0 {
            if (e & 1) != 0 {
                let (r, o) = result.overflowing_mul(base);
                result = r;
                overflow |= o;
            }
            let (b, o) = base.overflowing_mul(base);
            base = b;
            overflow |= o;
            e >>= 1;
        }
        (result, overflow)
    }

    /// Returns `None` on overflow.
    #[inline(always)]
    pub const fn checked_pow(self, exp: u32) -> Option<Self> {
        let (res, overflow) = self.overflowing_pow(exp);
        if overflow { None } else { Some(res) }
    }

    /// Saturating power (clamps to `MIN`/`MAX` on overflow).
    #[inline]
    pub const fn saturating_pow(self, exp: u32) -> Self {
        let (res, overflow) = self.overflowing_pow(exp);
        if !overflow {
            return res;
        }
        // Result is negative only when base is negative *and* exponent is odd
        let result_negative = self.is_negative() && (exp & 1) != 0;
        if result_negative {
            Self::MIN
        } else {
            Self::MAX
        }
    }
}

// ──────────────────────────────────────────────────────────────
// Parsing (FromStr)
// ──────────────────────────────────────────────────────────────

/// Error returned when parsing a string into `DtBig` fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseDtBigError {
    kind: ParseErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseErrorKind {
    Empty,
    InvalidDigit,
    Overflow,
}

impl ParseDtBigError {
    const fn new(kind: ParseErrorKind) -> Self {
        Self { kind }
    }
}

impl core::fmt::Display for ParseDtBigError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.kind {
            ParseErrorKind::Empty => f.write_str("cannot parse DtBig from empty string"),
            ParseErrorKind::InvalidDigit => f.write_str("invalid digit found in string"),
            ParseErrorKind::Overflow => f.write_str("number too large to fit in DtBig"),
        }
    }
}

#[cfg(feature = "std")]
impl core::error::Error for ParseDtBigError {}

impl FromStr for DtBig {
    type Err = ParseDtBigError;

    /// Parses a decimal string into a `DtBig`.
    ///
    /// Delegates to [`DtBig::parse_bytes`] after converting to bytes.
    /// See `parse_bytes` documentation for full behavior.
    #[inline(always)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_bytes(s.as_bytes())
    }
}

impl DtBig {
    /// Parses a decimal byte slice into a `DtBig`.
    ///
    /// - Supports optional leading `+` or `-` (ASCII bytes only).
    /// - Does **not** trim whitespace (matches `i128::from_str` etc.).
    /// - Leading zeros are allowed and ignored.
    /// - Empty slice, lone sign, or non-digit bytes → error.
    /// - Numbers outside `[MIN, MAX]` → `ParseDtBigError::Overflow`.
    pub fn parse_bytes(bytes: &[u8]) -> Result<Self, ParseDtBigError> {
        if bytes.is_empty() {
            return Err(ParseDtBigError::new(ParseErrorKind::Empty));
        }

        let (negative, digits) = match bytes {
            [b'-', rest @ ..] => (true, rest),
            [b'+', rest @ ..] => (false, rest),
            rest => (false, rest),
        };

        if digits.is_empty() {
            return Err(ParseDtBigError::new(ParseErrorKind::InvalidDigit));
        }

        if digits.len() > 110 {
            // Rare path: long byte slices. Still single pass, early exit on invalid digit.
            // Preserves exact original semantics (long all-zero slices = ZERO,
            // long valid non-zero = Overflow, any invalid = InvalidDigit).
            let mut is_all_zero = true;
            for &b in digits {
                let d = match b {
                    b'0'..=b'9' => b - b'0',
                    _ => return Err(ParseDtBigError::new(ParseErrorKind::InvalidDigit)),
                };
                if d != 0 {
                    is_all_zero = false;
                }
            }
            if is_all_zero {
                return Ok(Self::ZERO);
            } else {
                return Err(ParseDtBigError::new(ParseErrorKind::Overflow));
            }
        }

        // Common path (<= 110 digits) — fused decimal accumulation with overflow check
        let mut result = Self::ZERO;
        for &b in digits {
            let mut carry = match b {
                b'0'..=b'9' => (b - b'0') as u128,
                _ => return Err(ParseDtBigError::new(ParseErrorKind::InvalidDigit)),
            };

            let mut i = 0;
            while i < 5 {
                let tmp = (result.num[i] as u128) * 10 + carry;
                result.num[i] = tmp as u64;
                carry = tmp >> 64;
                i += 1;
            }

            // If there's any carry left after the 5 limbs, we've exceeded 2^320 → overflow.
            // Early return preserves the original fast-path performance.
            if carry != 0 {
                return Err(ParseDtBigError::new(ParseErrorKind::Overflow));
            }
        }

        // Final range check for values in [2^319, 2^320).
        // 2^319 (the MIN bit pattern) is allowed *only* for negative numbers.
        // All other cases with the sign bit set are out of range.
        if (result.num[4] & (1u64 << 63)) != 0 {
            if !negative || !result.eq(Self::MIN) {
                return Err(ParseDtBigError::new(ParseErrorKind::Overflow));
            }
        }

        Ok(if negative { Self::neg(result) } else { result })
    }
}

// ──────────────────────────────────────────────────────────────
// Binary and Octal formatting
// ──────────────────────────────────────────────────────────────

impl Binary for DtBig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("0b")?;
        let mut started = false;
        for &limb in self.num.iter().rev() {
            if started {
                write!(f, "{:064b}", limb)?;
            } else if limb != 0 {
                write!(f, "{:b}", limb)?;
                started = true;
            }
        }
        if !started {
            f.write_str("0")?;
        }
        Ok(())
    }
}

impl Octal for DtBig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("0o")?;
        let mut started = false;
        for &limb in self.num.iter().rev() {
            if started {
                write!(f, "{:022o}", limb)?;
            } else if limb != 0 {
                write!(f, "{:o}", limb)?;
                started = true;
            }
        }
        if !started {
            f.write_str("0")?;
        }
        Ok(())
    }
}

// ──────────────────────────────────────────────────────────────
// Iterator support (Sum + Product)
// ──────────────────────────────────────────────────────────────

impl Sum for DtBig {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |a, b| a.wrapping_add(b))
    }
}

impl<'a> Sum<&'a DtBig> for DtBig {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |a, &b| a.wrapping_add(b))
    }
}

impl Product for DtBig {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ONE, |a, b| a.wrapping_mul(b))
    }
}

impl<'a> Product<&'a DtBig> for DtBig {
    fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.fold(Self::ONE, |a, &b| a.wrapping_mul(b))
    }
}

// ──────────────────────────────────────────────────────────────
// Euclidean
// ──────────────────────────────────────────────────────────────

impl DtBig {
    /// Computes the quotient `q` such that `self = q * rhs + r` where `0 ≤ r < |rhs|`.
    /// This is the standard "Euclidean" remainder (always non-negative).
    ///
    /// Panics on division by zero (exactly like the `Div` impl).
    pub const fn div_euclid(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            panic!("attempt to divide by zero")
        }

        let q = self.wrapping_div(rhs);
        let r = self.wrapping_rem(rhs);

        // Remainder from truncating division has the sign of the dividend.
        // We adjust the quotient so the final remainder is always in [0, |rhs|).
        if r.is_negative() {
            if rhs.is_negative() {
                q.wrapping_add(Self::ONE)
            } else {
                q.wrapping_sub(Self::ONE)
            }
        } else {
            q
        }
    }

    /// Euclidean remainder.
    ///
    /// Returns the non-negative remainder `r` from Euclidean division
    /// (`0 ≤ r < |rhs|`).
    ///
    /// Panics on division by zero (exactly like the `Rem` impl).
    pub const fn rem_euclid(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            panic!("attempt to divide by zero")
        }

        let r = self.wrapping_rem(rhs);

        if r.is_negative() {
            if rhs.is_negative() {
                // r - rhs == r + |rhs| when rhs < 0
                r.wrapping_sub(rhs)
            } else {
                r.wrapping_add(rhs)
            }
        } else {
            r
        }
    }

    /// Wrapping Euclidean division (never panics, returns zero on div-by-zero).
    pub const fn wrapping_div_euclid(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            Self::ZERO
        } else {
            let q = self.wrapping_div(rhs);
            let r = self.wrapping_rem(rhs);

            if r.is_negative() {
                if rhs.is_negative() {
                    q.wrapping_add(Self::ONE)
                } else {
                    q.wrapping_sub(Self::ONE)
                }
            } else {
                q
            }
        }
    }

    /// Wrapping Euclidean remainder (always non-negative, never panics).
    pub const fn wrapping_rem_euclid(self, rhs: Self) -> Self {
        if rhs.is_zero() {
            Self::ZERO
        } else {
            let r = self.wrapping_rem(rhs);

            if r.is_negative() {
                if rhs.is_negative() {
                    r.wrapping_sub(rhs)
                } else {
                    r.wrapping_add(rhs)
                }
            } else {
                r
            }
        }
    }

    /// Checked Euclidean division. Returns `None` on division by zero.
    pub const fn checked_div_euclid(self, rhs: Self) -> Option<Self> {
        if rhs.is_zero() {
            None
        } else {
            Some(self.div_euclid(rhs))
        }
    }

    /// Checked Euclidean remainder. Returns `None` on division by zero.
    pub const fn checked_rem_euclid(self, rhs: Self) -> Option<Self> {
        if rhs.is_zero() {
            None
        } else {
            Some(self.rem_euclid(rhs))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_and_basics() {
        assert!(DtBig::ZERO.is_zero());
        assert!(DtBig::ONE.is_one());
        assert!(DtBig::NEG_ONE.is_neg_one());
        assert!(DtBig::MIN.is_min());
        assert_eq!(DtBig::MAX.wrapping_add(DtBig::ONE), DtBig::MIN);
    }

    #[test]
    fn basic_arithmetic() {
        let a = DtBig::from(12345i64);
        let b = DtBig::from(6789i64);
        let c = DtBig::from(-100i64);

        assert_eq!(a + b, DtBig::from(19134));
        assert_eq!(a - b, DtBig::from(5556));
        assert_eq!(a * b, DtBig::from(12345 * 6789));
        assert_eq!(b / a, DtBig::ZERO);
        assert_eq!(b % a, b);

        assert_eq!(a + c, DtBig::from(12245));
        assert_eq!(c * DtBig::from(5), DtBig::from(-500));
        assert_eq!(DtBig::from(1000) / DtBig::from(7), DtBig::from(142));
    }

    #[test]
    fn signed_edge_cases() {
        let min = DtBig::MIN;
        let max = DtBig::MAX;
        let neg_one = DtBig::NEG_ONE;

        // Negation (wrapping_neg of MIN is MIN in two's-complement)
        assert_eq!(min.wrapping_neg(), min);
        assert_eq!(min.saturating_neg(), max);
        assert_eq!(min.checked_neg(), None);

        // Classic signed overflow: MIN / -1
        assert_eq!(min.wrapping_div(neg_one), min);
        assert_eq!(min.checked_div(neg_one), None);
        assert_eq!(min.overflowing_div(neg_one), (min, true));

        assert!(max.is_positive());
        assert!(min.is_negative());
        assert_eq!(DtBig::ZERO.signum(), DtBig::ZERO);
        assert_eq!(max.signum(), DtBig::ONE);
        assert_eq!(min.signum(), DtBig::NEG_ONE);
    }

    #[test]
    fn bitwise_and_shifts() {
        let x = DtBig::from(0b101010i64);
        assert_eq!(x & DtBig::from(0b111100), DtBig::from(0b101000));
        assert_eq!(x | DtBig::from(0b111100), DtBig::from(0b111110));
        assert_eq!(x ^ DtBig::from(0b111111), DtBig::from(0b010101));

        assert_eq!(DtBig::ONE.wrapping_shl(10), DtBig::from(1024));
        assert_eq!(DtBig::from(1024).wrapping_shr(5), DtBig::from(32));
    }

    #[test]
    fn parsing() {
        let parsed: DtBig = "123456789012345".parse().unwrap();
        assert_eq!(parsed, DtBig::from(123456789012345i64));

        assert_eq!("-42".parse::<DtBig>().unwrap(), DtBig::from(-42));
        assert_eq!("0".parse::<DtBig>().unwrap(), DtBig::ZERO);
        assert_eq!("+100".parse::<DtBig>().unwrap(), DtBig::from(100));
    }

    #[test]
    fn byte_roundtrips() {
        let values = [
            DtBig::ZERO,
            DtBig::ONE,
            DtBig::NEG_ONE,
            DtBig::MIN,
            DtBig::MAX,
            DtBig::from(0x123456789abcdef0_i128),
            DtBig::from(-0x111222333444_i128),
        ];

        for &val in &values {
            assert_eq!(DtBig::from_le_bytes(val.to_le_bytes()), val);
            assert_eq!(DtBig::from_be_bytes(val.to_be_bytes()), val);
        }
    }
}
