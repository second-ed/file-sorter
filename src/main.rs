/// sorts loose top level files in a directory into directories named after the extension
/// creates a directory per extension
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path::Path;
use text_colorizer::*;

fn main() {
    let args = Args::new();
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

    let (dirs, files): (Vec<String>, Vec<String>) = entries
        .into_iter()
        .filter(|entry| {
            if let Ok(name) = entry.file_name().into_string() {
                !name.starts_with('.')
            } else {
                false
            }
        })
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
        dirs,
        files,
    };
    Ok(entries)
}

fn get_ext_map(files: &Vec<String>) -> HashMap<String, Vec<String>> {
    let mut ext_map: HashMap<String, Vec<String>> = HashMap::new();

    for f in files {
        let ext = get_ext(Path::new(f));
        ext_map.entry(ext).or_default().push(f.clone());
    }
    ext_map
}

fn get_ext(f: &Path) -> String {
    f.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| format!("_{}", ext.to_lowercase()))
        .unwrap_or_else(|| "_no_ext".to_string())
}

fn get_dirs_to_create(entries: &DirEntries, ext_map: &HashMap<String, Vec<String>>) -> Vec<String> {
    let dirs_to_create: Vec<String> = ext_map
        .iter()
        .map(|(key, _)| {
            Path::new(&entries.root)
                .join(key)
                .to_string_lossy()
                .to_string()
        })
        .filter(|path| !entries.dirs.contains(path) && *path != entries.root)
        .collect();
    dirs_to_create
}

fn get_name_pairs(ext_map: HashMap<String, Vec<String>>, root: &String) -> HashMap<String, String> {
    let name_pairs: HashMap<String, String> = ext_map
        .into_iter()
        .flat_map(|(k, files)| {
            files.into_iter().map(move |f| {
                let new_name = f.replace(root, &format!("{}/{}", root, k));
                (f, new_name)
            })
        })
        .collect();
    name_pairs
}

fn create_dirs(dirs_to_create: Vec<String>) -> io::Result<()> {
    dirs_to_create
        .into_iter()
        .try_for_each(|d| fs::create_dir_all(&d))
}

fn rename_files(name_pairs: HashMap<String, String>) -> io::Result<()> {
    name_pairs
        .into_iter()
        .try_for_each(|(old_name, new_name)| fs::rename(old_name, new_name))
}

#[derive(Debug)]
struct Args {
    root_dir: String,
}

impl Args {
    fn new() -> Args {
        let args: Vec<String> = env::args().skip(1).collect();

        if args.len() != 1 {
            eprintln!(
                "{} - Create directories for each file type",
                "file_sorter".bold().green()
            );
            eprintln!("Usage: file_sorter <ROOT_DIR>");
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
}

impl fmt::Display for Args {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "root_dir: {}", self.root_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ext_map() {
        let files = vec![
            String::from("some.csv"),
            String::from("wooo.parquet"),
            String::from("another.parquet"),
            String::from("bleurgh.xlsx"),
        ];
        let expected_result = HashMap::from([
            (String::from("_csv"), vec![String::from("some.csv")]),
            (
                String::from("_parquet"),
                vec![
                    String::from("wooo.parquet"),
                    String::from("another.parquet"),
                ],
            ),
            (String::from("_xlsx"), vec![String::from("bleurgh.xlsx")]),
        ]);

        assert_eq!(get_ext_map(&files), expected_result);
    }

    #[test]
    fn test_get_dirs_to_create() {
        let entries = DirEntries {
            root: String::from("blah"),
            dirs: vec![String::from("blah/_csv"), String::from("blah/_parquet")],
            files: vec![
                String::from("some.csv"),
                String::from("wooo.parquet"),
                String::from("another.parquet"),
                String::from("bleurgh.xlsx"),
            ],
        };

        let ext_map = HashMap::from([
            (String::from("_csv"), vec![String::from("some.csv")]),
            (
                String::from("_parquet"),
                vec![
                    String::from("wooo.parquet"),
                    String::from("another.parquet"),
                ],
            ),
            (String::from("_xlsx"), vec![String::from("bleurgh.xlsx")]),
        ]);

        let expected_result = vec![String::from("blah/_xlsx")];

        assert_eq!(get_dirs_to_create(&entries, &ext_map), expected_result);
    }

    #[test]
    fn test_get_name_pairs() {
        let ext_map = HashMap::from([
            (
                String::from("_csv"),
                vec![
                    String::from("User/some/person/docs/report.csv"),
                    String::from("User/some/person/docs/data.csv"),
                    String::from("User/some/person/docs/doc.csv"),
                ],
            ),
            (
                String::from("_parquet"),
                vec![
                    String::from("User/some/person/docs/report.parquet"),
                    String::from("User/some/person/docs/data.parquet"),
                    String::from("User/some/person/docs/doc.parquet"),
                ],
            ),
            (
                String::from("_pdf"),
                vec![
                    String::from("User/some/person/docs/report.pdf"),
                    String::from("User/some/person/docs/data.pdf"),
                    String::from("User/some/person/docs/doc.pdf"),
                ],
            ),
        ]);

        let root = String::from("User/some/person/docs");

        let expected_result = HashMap::from([
            (
                String::from("User/some/person/docs/report.csv"),
                String::from("User/some/person/docs/_csv/report.csv"),
            ),
            (
                String::from("User/some/person/docs/data.csv"),
                String::from("User/some/person/docs/_csv/data.csv"),
            ),
            (
                String::from("User/some/person/docs/doc.csv"),
                String::from("User/some/person/docs/_csv/doc.csv"),
            ),
            (
                String::from("User/some/person/docs/report.parquet"),
                String::from("User/some/person/docs/_parquet/report.parquet"),
            ),
            (
                String::from("User/some/person/docs/data.parquet"),
                String::from("User/some/person/docs/_parquet/data.parquet"),
            ),
            (
                String::from("User/some/person/docs/doc.parquet"),
                String::from("User/some/person/docs/_parquet/doc.parquet"),
            ),
            (
                String::from("User/some/person/docs/report.pdf"),
                String::from("User/some/person/docs/_pdf/report.pdf"),
            ),
            (
                String::from("User/some/person/docs/data.pdf"),
                String::from("User/some/person/docs/_pdf/data.pdf"),
            ),
            (
                String::from("User/some/person/docs/doc.pdf"),
                String::from("User/some/person/docs/_pdf/doc.pdf"),
            ),
        ]);

        assert_eq!(get_name_pairs(ext_map, &root), expected_result);
    }
}
