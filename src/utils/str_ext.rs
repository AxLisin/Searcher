use colored::Colorize;

pub trait StrExt {
    fn colorize_matches(&self, indices: Vec<usize>) -> String;
}

impl StrExt for str {
    fn colorize_matches(&self, indices: Vec<usize>) -> String {
        self.chars()
            .enumerate()
            .map(|(i, c)| {
                if indices.contains(&i) {
                    c.to_string().red().bold().to_string()
                } else {
                    c.to_string().normal().to_string()
                }
            })
            .collect()
    }
}
