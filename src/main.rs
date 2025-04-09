use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use text_colorizer::*;

fn main() {
    let args = parse_args();
    let entries: DirEntries =
        dbg!(get_files_dirs(args.root_dir.clone()).expect("Failed to get dirs and files"));

    let ext_map = dbg!(get_ext_map(&entries.files));
    let dirs_to_create = dbg!(get_dirs_to_create(&entries, &ext_map));
    let name_pairs = dbg!(get_name_pairs(ext_map, &entries.root));
    let _ = create_dirs(dirs_to_create);
    let _ = rename_files(name_pairs);
}

#[derive(Debug)]
struct DirEntries {
    root: String,
    dirs: Vec<String>,
    files: Vec<String>,
}

impl fmt::Display for DirEntries {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {:?}\n{} {:?}",
            "DirEntries.dirs: ".bold().green(),
            self.dirs,
            "DirEntries.files:".bold().green(),
            self.files
        )
    }
}

fn get_files_dirs(root_dir: String) -> Result<DirEntries, io::Error> {
    let entries: Vec<_> = fs::read_dir(&root_dir)?
        .filter_map(|entry| entry.ok())
        .collect();

    let (dirs, files): (Vec<String>, Vec<String>) =
        entries
            .into_iter()
            .fold((Vec::new(), Vec::new()), |(mut dirs, mut files), entry| {
                let path = entry.path().to_string_lossy().to_string();
                match entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    true => dirs.push(path),
                    false => files.push(path),
                }
                (dirs, files)
            });

    let entries = DirEntries {
        root: root_dir,
        dirs: dirs,
        files: files,
    };
    Ok(entries)
}

fn get_ext_map(files: &Vec<String>) -> HashMap<String, Vec<String>> {
    let mut ext_map: HashMap<String, Vec<String>> = HashMap::new();

    for f in files {
        let path = Path::new(&f);

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext = format!("_{}", ext.to_lowercase());
            ext_map.entry(ext).or_insert_with(Vec::new).push(f.clone());
        }
    }
    ext_map
}

fn get_dirs_to_create(entries: &DirEntries, ext_map: &HashMap<String, Vec<String>>) -> Vec<String> {
    let mut dirs_to_create: Vec<String> = Vec::new();

    for (key, _) in ext_map.iter() {
        let path = Path::new(&entries.root)
            .join(&key)
            .to_string_lossy()
            .to_string();

        if !entries.dirs.contains(&path) && path != entries.root {
            dirs_to_create.push(path);
        }
    }
    dirs_to_create
}

fn get_name_pairs(ext_map: HashMap<String, Vec<String>>, root: &String) -> HashMap<String, String> {
    let mut name_pairs: HashMap<String, String> = HashMap::new();

    for (k, files) in ext_map {
        for f in files {
            let new_name = f.replace(root, &format!("{}/{}", root, k));
            name_pairs.insert(f, new_name);
        }
    }
    name_pairs
}

fn create_dirs(dirs_to_create: Vec<String>) -> io::Result<()> {
    for d in dirs_to_create {
        fs::create_dir_all(&d)?;
    }
    Ok(())
}

fn rename_files(name_pairs: HashMap<String, String>) -> io::Result<()> {
    for (old_name, new_name) in name_pairs {
        fs::rename(old_name, new_name)?;
    }
    Ok(())
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

fn print_usage() {
    eprintln!(
        "{} - Create directories for each file type",
        "file_sorter".green()
    );
    eprintln!("Usage: file_sorter <ROOT_DIR>");
}
