#[cfg(test)]
mod time_range_tests {
    use deep_time::{Dt, Scale, TimeRange};

    #[inline]
    fn ymd(yr: i64, mo: u8, day: u8) -> Dt {
        Dt::from_ymd(yr, mo, day)
    }

    #[inline]
    fn hr(n: i64) -> Dt {
        Dt::from_hr(n, Scale::TAI)
    }

    #[test]
    fn until_includes_end_when_reachable() {
        let start = ymd(2000, 1, 1);
        let end = ymd(2000, 1, 2);
        let step = hr(1);

        let v: Vec<_> = start.every(step).until(end).collect();

        assert_eq!(v.len(), 25);
        assert_eq!(v.first(), Some(&start));
        assert_eq!(v.last(), Some(&end));
        assert_eq!(start.every(step).until(end).len(), 25);
    }

    #[test]
    fn up_to_excludes_end_when_reachable() {
        let start = ymd(2000, 1, 1);
        let end = ymd(2000, 1, 2);
        let step = hr(1);

        let v: Vec<_> = start.every(step).up_to(end).collect();

        assert_eq!(v.len(), 24);
        assert_eq!(v.first(), Some(&start));
        // last should be 2000-01-01 23:00
        let mut expected_last = start;
        expected_last.add_hr(23);
        assert_eq!(v.last(), Some(&expected_last));
        assert_eq!(start.every(step).up_to(end).len(), 24);
    }

    #[test]
    fn exclusive_when_end_not_on_step_boundary() {
        let start = ymd(2000, 1, 1);
        let mut end = ymd(2000, 1, 1);
        end.add_hr(25); // ← must be mut
        let step = hr(6);

        let v: Vec<_> = start.every(step).up_to(end).collect();

        // 0h,6h,12h,18h,24h are all < 25h → 5 points
        assert_eq!(v.len(), 5);
        assert_eq!(start.every(step).up_to(end).len(), 5);
    }

    // === Zero step ===

    #[test]
    fn zero_step_inclusive_same_point() {
        let t = ymd(2025, 6, 15);
        let zero = Dt::from_sec(0, Scale::TAI);

        let v: Vec<_> = t.every(zero).until(t).collect();
        assert_eq!(v, vec![t]);
        assert_eq!(t.every(zero).until(t).len(), 1);
    }

    #[test]
    fn zero_step_exclusive_and_mismatch() {
        let t = ymd(2025, 6, 15);
        let other = ymd(2025, 6, 16);
        let zero = Dt::from_sec(0, Scale::TAI);

        assert!(t.every(zero).up_to(t).collect::<Vec<_>>().is_empty());
        assert!(t.every(zero).until(other).collect::<Vec<_>>().is_empty());
    }

    // === Descending ranges ===

    #[test]
    fn down_to_descending_inclusive() {
        let start = ymd(2000, 1, 2);
        let end = ymd(2000, 1, 1);
        let step = hr(-1);

        let v: Vec<_> = start.every(step).down_to(end).collect();

        assert_eq!(v.len(), 25);
        assert_eq!(v.first(), Some(&start));
        assert_eq!(v.last(), Some(&end));
    }

    #[test]
    fn for_n_steps_produces_exactly_n_points() {
        let start = ymd(2000, 1, 1);
        let step = hr(2);

        let v: Vec<_> = start.for_n_steps(5, step).collect();

        assert_eq!(v.len(), 5);
        assert_eq!(v[0], start);

        let mut expected_last = start;
        expected_last.add_hr(8); // 0 + 4*2h
        assert_eq!(v[4], expected_last);

        // This now works because we return impl ExactSizeIterator
        assert_eq!(start.for_n_steps(5, step).len(), 5);
    }

    #[test]
    fn next_n_skips_start() {
        let start = ymd(2000, 1, 1);
        let step = hr(3);

        let v: Vec<_> = start.next_n(4, step).collect();

        assert_eq!(v.len(), 4);
        let mut first = start;
        first.add_hr(3);
        assert_eq!(v[0], first);
    }

    // === ExactSizeIterator correctness ===

    #[test]
    fn len_reports_remaining_correctly() {
        let start = ymd(2000, 1, 1);
        let end = ymd(2000, 1, 2);
        let step = hr(6);

        let mut r = start.every(step).until(end);
        assert_eq!(r.len(), 5);

        let _ = r.next();
        assert_eq!(r.len(), 4);

        let _ = r.next();
        assert_eq!(r.len(), 3);

        let remaining: Vec<_> = r.collect();
        assert_eq!(remaining.len(), 3);
    }

    #[test]
    fn exact_size_iterator_contract() {
        let start = ymd(2000, 1, 1);
        let end = ymd(2000, 1, 2);
        let step = hr(1);

        let mut r = start.every(step).up_to(end);
        let original_len = r.len();

        let mut count = 0usize;
        while r.next().is_some() {
            count += 1;
            assert_eq!(r.len(), original_len - count);
        }
        assert_eq!(count, original_len);
    }

    // === Boundary cases ===

    #[test]
    fn start_equals_end_inclusive() {
        let t = ymd(2025, 4, 1);
        let step = hr(1);

        let v: Vec<_> = t.every(step).until(t).collect();
        assert_eq!(v.len(), 1);
        assert_eq!(v[0], t);
    }

    #[test]
    fn start_equals_end_exclusive() {
        let t = ymd(2025, 4, 1);
        let step = hr(1);

        let v: Vec<_> = t.every(step).up_to(t).collect();
        assert!(v.is_empty());
        assert_eq!(t.every(step).up_to(t).len(), 0);
    }

    // === API consistency ===

    #[test]
    fn builder_vs_direct_constructors() {
        let start = ymd(2000, 1, 1);
        let end = ymd(2000, 1, 2);
        let step = hr(12);

        let via_builder: Vec<_> = start.every(step).until(end).collect();
        let via_inclusive: Vec<_> = TimeRange::inclusive(start, end, step).collect();

        assert_eq!(via_builder, via_inclusive);
    }
}
