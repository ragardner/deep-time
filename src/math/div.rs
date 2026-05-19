use crate::Real;

/// const friendly rem euclid function for float.
pub const fn rem_euclid_f(lhs: Real, rhs: Real) -> Real {
    let r = lhs % rhs;
    if r < f!(0.0) {
        if rhs > f!(0.0) { r + rhs } else { r - rhs }
    } else {
        r
    }
}
