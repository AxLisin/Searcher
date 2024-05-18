use std::{cmp::min, fs, path::PathBuf, sync::{Arc, Mutex}, thread, time::Duration};

use anyhow::Context;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use colored::{ColoredString, Colorize};
use rayon::{
    iter::{IntoParallelRefIterator, ParallelBridge, ParallelIterator},
    ThreadPoolBuilder,
};

fn clear_screen() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
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

trait StrExt {
    fn colorize_matches(&self, indices: Vec<usize>) -> String;
}

impl StrExt for str {
    fn colorize_matches(&self, indices: Vec<usize>) -> String {
        self
            .chars()
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

fn get_top_matches(matches: &mut Vec<(i64, String)>) -> (Vec<String>, usize) {
    let slice_index = min(matches.len(), 10);
    let extra_matches = matches.len() - slice_index;

    matches.sort_by(|a, b| b.0.cmp(&a.0));
    
    let matches: Vec<String> = matches[0..slice_index]
        .iter()
        .map(|(_, s)| s.clone())
        .collect();

    (matches, extra_matches)
}

struct Searcher {
    base_dir: PathBuf,
    matcher: Matcher,
    verbose: bool,
    matches: Arc<Mutex<Vec<(i64, String)>>>,
    last_printed: Arc<Mutex<Vec<String>>>
}

impl Searcher {
    fn new(base_dir: PathBuf, query: String, verbose: bool) -> Self {
        Self {
            base_dir,
            verbose,
            matches: Arc::new(Mutex::new(Vec::new())),
            matcher: Matcher::new(query),
            last_printed: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn check_match(&self, path: &PathBuf, is_dir: bool) {
        let base_dir = &self.base_dir;
        let matcher = &self.matcher;

        let file_name = path.file_name().unwrap().to_str().unwrap();

        let relative_path = path.strip_prefix(base_dir).unwrap();
        let parent_dir = relative_path.parent().unwrap().to_str().unwrap();

        if let Some((score, indices)) = matcher.fmatch(file_name) {
            let colored_name = file_name.colorize_matches(indices);
            let formatted_string: String;

            if is_dir {
                formatted_string = format!(".\\{}\\{}", parent_dir, colored_name);
            } else {
                formatted_string = format!(".\\{}\\{}", parent_dir, colored_name);
            }

            let mut matches = self.matches.lock().unwrap();

            matches.push((score, formatted_string));
        }
    }



    fn search(&self, path: &PathBuf) -> anyhow::Result<()> {
        let matches = Arc::clone(&self.matches);
        let last_printed = Arc::clone(&self.last_printed);

        let completed_search = Arc::new(Mutex::new(false));
        let completed_search_clone = Arc::clone(&completed_search);

        thread::spawn(move || {
            let mut last_printed = last_printed.lock().unwrap();

            loop {
                let matches_ref = matches.lock().unwrap();
                let mut matches = matches_ref.clone();
                let completed_search_clone = completed_search_clone.lock().unwrap();

                if *completed_search_clone {
                    break;
                }

                drop(matches_ref);

                let (matches, extra_matches) = get_top_matches(&mut matches);

                if matches == *last_printed {
                    print!("\r... {} more matches", extra_matches);
                    continue;
                }
                
                clear_screen();
                println!("{}", matches.join("\n"));
    
                *last_printed = matches;
            }
        });

        self.search_directory(path).unwrap();
        
        *completed_search.lock().unwrap() = true;

        // print_thread.thread().unpark();
        

        
        let matches = self.matches.lock().unwrap();
        let (matches, extra_matches) = get_top_matches(&mut matches.clone());

        println!("finished");

        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

        println!("{}", matches.join("\n"));
        println!("... {} more matches", extra_matches);

        // println!("Found {} matches", self.matches.lock().unwrap().len());

        Ok(())
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
        .num_threads(14)
        .build_global()
        .unwrap();

    let searcher = Searcher::new(
        // "C:\\".into(),
        current_dir.clone(),
        query.to_string(),
        verbose,
    );

    let start = std::time::Instant::now();

    searcher
        .search(
            &current_dir.clone()
            // &PathBuf::from("C:\\Users"),
        )
        .unwrap();

    println!("\nTime elapsed: {:?}", start.elapsed());
}
