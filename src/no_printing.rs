// 2.2043534s

use std::{fs, path::PathBuf};

use anyhow::Context;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use colored::{ColoredString, Colorize};
use rayon::{
    iter::{IntoParallelRefIterator, ParallelIterator},
    ThreadPoolBuilder,
};

fn nice_panic<T>(message: T) -> !
where
    T: std::fmt::Display,
{
    eprintln!("Error: {}", message);
    std::process::exit(0);
}

struct Matcher {
    query: String,
    fuzzy_matcher: Box<dyn FuzzyMatcher>,
}

impl Matcher {
    fn new(query: String) -> Self {
        Self {
            query,
            fuzzy_matcher: Box::new(SkimMatcherV2::default()),
        }
    }

    fn fmatch(&self, text: &str) -> Option<(i64, Vec<usize>)> {
        self.fuzzy_matcher.fuzzy_indices(text, &self.query)
    }
}

struct Searcher {
    base_dir: PathBuf,
    matcher: Matcher,
    verbose: bool,
}

impl Searcher {
    fn new(base_dir: PathBuf, query: String, verbose: bool) -> Self {
        Self {
            base_dir,
            verbose,
            matcher: Matcher::new(query),
        }
    }

    fn check_match(&self, path: &PathBuf, is_dir: bool) {
        let base_dir = &self.base_dir;
        let matcher = &self.matcher;

        let file_name = path.file_name().unwrap().to_str().unwrap();

        let relative_path = path.strip_prefix(base_dir).unwrap();
        let parent_dir = relative_path.parent().unwrap().to_str().unwrap();

        if let Some((_, indices)) = matcher.fmatch(file_name) {
            let colored_name: String = file_name
                .chars()
                .enumerate()
                .map(|(i, c)| {
                    if indices.contains(&i) {
                        c.to_string().red().bold().to_string()
                    } else {
                        c.to_string().normal().to_string()
                    }
                })
                .collect();

            if is_dir {
                println!(".\\{}\\{}", parent_dir, colored_name);
            } else {
                println!(".\\{}{}", parent_dir, colored_name);
            }
        }
    }

    fn search_directory(&self, path: &PathBuf) -> anyhow::Result<()> {
        let current_dir_children: Vec<PathBuf> = if let Ok(children) = fs::read_dir(path) {
            children.map(|entry| entry.unwrap().path()).collect()
        } else {
            if self.verbose {
                println!("Error reading directory: {:?}", path);
            }

            return Ok(());
        };

        current_dir_children.par_iter().for_each(|path| {
            let is_dir = path.is_dir();

            self.check_match(&path, is_dir);

            if is_dir {
                self.search_directory(&path).unwrap();
            }
        });

        anyhow::Ok(())
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let current_dir = std::env::current_dir().unwrap();

    let query = &args.get(1).unwrap_or_else(|| {
        nice_panic("No query provided");
    });
    let verbose = args.contains(&"--verbose".to_string());

    ThreadPoolBuilder::new()
        .num_threads(12)
        .build_global()
        .unwrap();

    let searcher = Searcher::new(
        "C:\\".into(),
        // current_dir.clone(),
        query.to_string(),
        verbose,
    );

    let start = std::time::Instant::now();

    searcher.search_directory(
        // &current_dir.clone()
        &PathBuf::from("C:\\Users")
    ).unwrap();

    println!("Time elapsed: {:?}", start.elapsed());
}
