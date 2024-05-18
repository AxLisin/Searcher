use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

pub struct Matcher {
    query: String,
    fuzzy_matcher: Box<dyn FuzzyMatcher>,
}

impl Matcher {
    pub fn new(query: String) -> Self {
        Self {
            query,
            fuzzy_matcher: Box::new(SkimMatcherV2::default()),
        }
    }

    pub fn fmatch(&self, text: &str) -> Option<(i64, Vec<usize>)> {
        self.fuzzy_matcher.fuzzy_indices(text, &self.query)
    }
}
