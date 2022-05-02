use std::{collections::HashSet, env, fs, io, path::PathBuf};

pub fn whereis(file: &str) -> HashSet<PathBuf> {
    let vars = env::vars();
    let mut matches = HashSet::new();

    for (_, values) in vars {
        for path in values.split(";") {
            let mut path = PathBuf::from(path);
            path.push(file);
            if let Ok(path) = check_file(path) {
                matches.insert(path);
            }
        }
    }

    matches
}

fn check_file(path: PathBuf) -> io::Result<PathBuf> {
    let _ = fs::File::open(&path)?;

    Ok(path)
}
