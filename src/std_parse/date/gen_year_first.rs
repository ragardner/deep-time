use crate::{
    AmBuilder, DateClassification, DateToken, append_to_all, get_compatible_time_suffixes,
};
use std::string::String;
use std::vec;
use std::vec::Vec;

pub(crate) fn generate_ambiguous_year_first_candidates(class: &DateClassification) -> Vec<String> {
    if !class.has_year || class.num_named > 0 {
        return vec![];
    }

    let suffixes = get_compatible_time_suffixes(class);
    let mut candidates = Vec::with_capacity(60);

    let tokens = &class.tokens;
    let mut builders: Vec<AmBuilder> = vec![AmBuilder {
        pieces: Vec::with_capacity(12),
        seen_year: false,
        seen_month: false,
        seen_day: false,
    }];

    for &token in tokens {
        match token {
            DateToken::Hyphen => append_to_all(&mut builders, "-"),
            DateToken::Slash => append_to_all(&mut builders, "/"),
            DateToken::Dot => append_to_all(&mut builders, "."),
            DateToken::Space => append_to_all(&mut builders, " "),
            DateToken::Comma => append_to_all(&mut builders, ","),
            DateToken::Digits(_) => {
                let mut new_builders = Vec::with_capacity(builders.len() * 4);

                for b in &builders {
                    let filtered =
                        token
                            .to_fmt_year_first()
                            .iter()
                            .copied()
                            .filter(|&spec| match spec {
                                // Year formats (including compact combined)
                                "%Y" | "%y" | "%G" | "%C" | "%Y%m%d" | "%y%m%d" => !b.seen_year,
                                // Month formats (only after year)
                                "%m" | "%-m" | "%_m" => b.seen_year && !b.seen_month,
                                // Day formats (only after year + month)
                                "%d" | "%e" | "%-d" | "%_d" | "%j" => {
                                    b.seen_year && b.seen_month && !b.seen_day
                                }
                                _ => false,
                            });

                    for spec in filtered {
                        let mut new_b = b.clone();
                        new_b.pieces.push(spec);

                        // Update state - handle combined formats like %Y%m%d / %y%m%d
                        match spec {
                            "%Y" | "%y" | "%G" | "%C" => new_b.seen_year = true,
                            "%m" | "%-m" | "%_m" => new_b.seen_month = true,
                            "%d" | "%e" | "%-d" | "%_d" | "%j" => new_b.seen_day = true,
                            "%Y%m%d" | "%y%m%d" => {
                                new_b.seen_year = true;
                                new_b.seen_month = true;
                                new_b.seen_day = true;
                            }
                            _ => {}
                        }
                        new_builders.push(new_b);
                    }
                }
                builders = new_builders;
            }
            _ => {} // ignore Alpha, W etc.
        }
    }

    // ── Assemble only complete dates ──
    for b in builders {
        if !(b.seen_year && b.seen_month && b.seen_day) {
            continue;
        }

        let date_part: String = b.pieces.concat();

        if class.time.is_none() {
            candidates.push(date_part);
            continue;
        }
        for suffix in &suffixes {
            let mut s = date_part.clone();
            s.push_str(suffix);
            candidates.push(s);
        }
    }

    candidates
}
