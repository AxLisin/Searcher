use rayon::ThreadPoolBuilder;

mod matcher {
    pub mod matcher;
}

mod searcher {
    pub mod searcher;
    pub mod top_matches;
}

mod utils {
    pub mod clear_screen;
    pub mod str_ext;
}

use crate::searcher::searcher::Searcher;

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

    let searcher = Searcher::new(current_dir.clone(), query.to_string(), verbose);

    let start = std::time::Instant::now();

    searcher.search(&current_dir.clone()).unwrap();

    println!("\nTime elapsed: {:?}", start.elapsed());
}
