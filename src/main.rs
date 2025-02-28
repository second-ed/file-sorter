use std::env;
use std::fmt;
use std::fs;
use text_colorizer::*;
use walkdir::WalkDir;

fn main() {
    let args = parse_args();
    println!("Using Args:\n    {}", args);

    let (dirs, files): (Vec<_>, Vec<_>) = WalkDir::new(args.root_dir)
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

    println!("{}", entries);
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

fn parse_args() -> Args {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.len() != 1 {
        print_usage();
        eprintln!(
            "{} wrong number of args: expected 1 got {}. ",
            "Error:".red().bold(),
            args.len()
        );
        std::process::exit(1);
    }
    Args {
        root_dir: args[0].clone(),
    }
}

fn print_usage() {
    eprintln!(
        "{} - Create directories for each file type",
        "file_sorter".green()
    );
    eprintln!("Usage: file_sorter <ROOT_DIR>");
}
