use crate::DateClassification;
use alloc::string::String;
use alloc::vec::Vec;

/// Returns a small list of candidate format strings specifically for a few syslog formats.
pub(crate) fn generate_syslog_candidates(class: &DateClassification) -> Vec<String> {
    let mut candidates = Vec::with_capacity(4);
    let day_variants = ["%d", "%e"];
    for &day in &day_variants {
        // Full time with seconds (%H:%M:%S) → exactly 2 colons in the time part
        if class.num_colon >= 2 {
            candidates.push(alloc::format!("%Y %b {day} %H:%M:%S"));
        }
        // Short time without seconds (%H:%M) → exactly 1 colon in the time part
        if class.num_colon == 1 {
            candidates.push(alloc::format!("%Y %b {day} %H:%M"));
        }
    }
    candidates
}
