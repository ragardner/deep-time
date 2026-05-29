use crate::{
    ClassifiedDate, Dt, Lang, TimeTraits, classify_date, generate_syslog_candidates,
    try_compatible_formats,
};

/// Parses syslog-style dates missing the year (e.g. "Mar  5 10:23:45", "Dec 31 23:59:59").
///
/// - Try current year first.
/// - If the parsed date is **more than 2 days in the future**, assume previous year.
///   This covers the classic December-in-January case while tolerating clock drift.
///
/// Pass `reference_date` when reprocessing historical logs for perfect reproducibility.
/// If `reference_date` is `None` and the `std` feature is enabled, real system time is used.
#[inline]
pub(crate) fn parse_syslog_no_year(input: &str, lang: Lang, ref_time: &Option<Dt>) -> Option<Dt> {
    let now = if let Some(tp) = ref_time {
        *tp
    } else {
        #[cfg(feature = "std")]
        {
            Dt::now().ok()?
        }
        #[cfg(not(feature = "std"))]
        {
            return None; // no reference → can't parse relative year
        }
    };

    let g = now.to_ymd();
    let this_year = g.yr;

    let try_with_year = |year: i64| -> Option<Dt> {
        let s = alloc::format!("{} {}", year, input);

        // Pass the same reference time down to classify_date
        let cls = classify_date(&s, lang, ref_time).ok()?;

        match cls {
            ClassifiedDate::Cls(c) => try_compatible_formats(&s, generate_syslog_candidates(&c)),
            _ => None,
        }
    };

    if let Some(dt) = try_with_year(this_year) {
        // Compare against the same reference time
        if dt > now + 2.days() {
            return try_with_year(this_year - 1);
        }
        return Some(dt);
    }

    try_with_year(this_year - 1)
}
