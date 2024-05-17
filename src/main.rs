use std::{cmp::min, fs, path::PathBuf, sync::Mutex};

use anyhow::Context;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use colored::{ColoredString, Colorize};
use rayon::{
    iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator},
    ThreadPoolBuilder,
};

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
    matches: Mutex<Vec<(i64, String)>>,
    last_printed: Mutex<Vec<String>>
}

impl Searcher {
    fn new(base_dir: PathBuf, query: String, verbose: bool) -> Self {
        Self {
            base_dir,
            verbose,
            matches: Mutex::new(Vec::new()),
            matcher: Matcher::new(query),
            last_printed: Mutex::new(Vec::new()),
        }
    }

    fn check_match(&self, path: &PathBuf, is_dir: bool) {
        let base_dir = &self.base_dir;
        let matcher = &self.matcher;

        let file_name = path.file_name().unwrap().to_str().unwrap();

        let relative_path = path.strip_prefix(base_dir).unwrap();
        let parent_dir = relative_path.parent().unwrap().to_str().unwrap();

        if let Some((score, indices)) = matcher.fmatch(file_name) {
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

            let mut matches = self.matches.lock().unwrap();

            if is_dir {
                matches.push((score, format!(".\\{}\\{}", parent_dir, colored_name)));
            } else {
                matches.push((score, format!(".\\{}\\{}", parent_dir, colored_name)));
            }

            let mut last_printed = self.last_printed.lock().unwrap();
            let slice_index = min(matches.len(), 10);

            let extra_matches = matches.len() - slice_index;

            // matches.sort_by(|a, b| b.0.cmp(&a.0));
            
            let matches: Vec<String> = matches[0..slice_index]
                .iter()
                .map(|(_, s)| s.clone())
                .collect();

            if matches == last_printed.to_vec() {
                print!("\r... {} more matches", extra_matches);
                return;
            }

            print!("{esc}[2J{esc}[1;1H", esc = 27 as char); // clear the screen
            println!("{}", matches.join("\n"));

            *last_printed = matches.clone();

            drop(last_printed);
            drop(matches);
        }
    }

    fn search_directory(&self, path: &PathBuf) -> anyhow::Result<()> {
        let Ok(children) = std::fs::read_dir(path) else {
            if self.verbose {
                println!("Error reading directory: {:?}", path);
            }

            return Ok(());
        };

        children
            .filter_map(|entry| Some(entry.unwrap().path()))
            .par_bridge()
            .for_each(|path| {
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

    let Some(query) = args.get(1) else {
        eprintln!("No query provided");
        return;
    };
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

    searcher
        .search_directory(
            // &current_dir.clone()
            &PathBuf::from("C:\\Users"),
        )
        .unwrap();

    println!("\nTime elapsed: {:?}", start.elapsed());
}
