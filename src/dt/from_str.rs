use crate::{Dt, TimeParts};

impl Dt {
    #[inline]
    pub fn from_str(
        s: &str,
        fmt: &str,
        inp_can_end_before_fmt: bool,
        fmt_can_end_before_inp: bool,
        allow_partial_date: bool,
    ) -> Option<Dt> {
        TimeParts::from_str(
            fmt,
            s,
            inp_can_end_before_fmt,
            fmt_can_end_before_inp,
            allow_partial_date,
        )
        .and_then(|p| p.to_time_point())
        .ok()
    }
}
