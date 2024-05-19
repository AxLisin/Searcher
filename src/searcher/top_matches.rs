use std::cmp::min;

pub fn get_top_matches(matches: &mut [(i64, String)]) -> (Vec<String>, usize) {
    let slice_index = min(matches.len(), 10);
    let extra_matches = matches.len() - slice_index;

    matches.sort_by(|a, b| b.0.cmp(&a.0));

    let matches: Vec<String> = matches[0..slice_index]
        .iter()
        .map(|(_, s)| s.clone())
        .collect();

    (matches, extra_matches)
}
