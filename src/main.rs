use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::path::Path;
use text_colorizer::*;
use walkdir::WalkDir;

fn main() {
    let args = parse_args();
    // println!("Using Args:\n    {}", args);
    let entries: Entries = get_dirs_and_files(args.root_dir);
    // println!("{}", entries);
    // println!("{:?}", entries.dirs);
    // println!("{:?}", entries.files);
    let file_map = create_ext_map(entries.files);
    println!("{:?}", file_map);
}

fn get_dirs_and_files(root_dir: String) -> Entries {
    let (dirs, files): (Vec<_>, Vec<_>) = WalkDir::new(root_dir)
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(entry) => Some(entry),
            Err(e) => {
                eprintln!("{} {:?}", "Error reading entry:".red().bold(), e);
                None
            }
        })
        .filter_map(|entry| match fs::metadata(entry.path()) {
            Ok(md) => Some((entry.path().to_path_buf(), md)),
            Err(e) => {
                eprintln!(
                    "{} {:?}: {}",
                    "Error reading metadata:".red().bold(),
                    entry.path(),
                    e
                );
                None
            }
        })
        .partition(|(_, md)| md.is_dir());

    let entries = Entries {
        dirs: dirs
            .into_iter()
            .map(|(path, _)| path.display().to_string())
            .collect(),
        files: files
            .into_iter()
            .map(|(path, _)| path.display().to_string())
            .collect(),
    };
    entries
}

fn create_ext_map(files: Vec<String>) -> HashMap<String, Vec<String>> {
    let mut file_map: HashMap<String, Vec<String>> = HashMap::new();

    for f in files {
        let path = Path::new(&f);

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext = ext.to_lowercase();
            file_map.entry(ext).or_insert_with(Vec::new).push(f);
        }
    }
    file_map
}

#[derive(Debug)]
struct Args {
    root_dir: String,
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "root_dir: {}", self.root_dir)
    }
}

fn parse_args() -> Args {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() != 1 {
        print_usage();
        eprintln!(
            "{} wrong number of args: expected 1 got {}. ",
            "Error:".bold().red(),
            args.len()
        );
        std::process::exit(1);
    }
    Args {
        root_dir: args[0].clone(),
    }
}

#[derive(Debug)]
struct Entries {
    dirs: Vec<String>,
    files: Vec<String>,
}

impl fmt::Display for Entries {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {:?}\n{} {:?}",
            "Entries.dirs: ".bold().green(),
            self.dirs,
            "Entries.files:".bold().green(),
            self.files
        )
    }
}

fn print_usage() {
    eprintln!(
        "{} - Create directories for each file type",
        "file_sorter".green()
    );
    eprintln!("Usage: file_sorter <ROOT_DIR>");
}
