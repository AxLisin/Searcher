use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
};

use rayon::iter::{ParallelBridge, ParallelIterator};

use crate::{
    matcher::matcher::Matcher,
    searcher::top_matches::get_top_matches,
    utils::{clear_screen::clear_screen, str_ext::StrExt},
};

pub struct Searcher {
    base_dir: PathBuf,
    matcher: Matcher,
    verbose: bool,
    matches: Arc<Mutex<Vec<(i64, String)>>>,
    last_printed: Arc<Mutex<Vec<String>>>,
}

impl Searcher {
    pub fn new(base_dir: PathBuf, query: String, verbose: bool) -> Self {
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

    pub fn search(&self, path: &PathBuf) -> anyhow::Result<()> {
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

        let matches = self.matches.lock().unwrap();
        let (matches, extra_matches) = get_top_matches(&mut matches.clone());

        println!("finished");

        print!("{esc}[2J{esc}[1;1H", esc = 27 as char);

        println!("{}", matches.join("\n"));
        println!("... {} more matches", extra_matches);
        Ok(())
    }
}
