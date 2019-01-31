use std::str::FromStr;
use std::{env, io};
use std::path::PathBuf;

#[derive(Debug)]
pub enum ConfirmAction {
    RM, LIST, NONE
}

impl FromStr for ConfirmAction {
    type Err = ();

    fn from_str(s: &str) -> Result<ConfirmAction, ()> {
        match s.to_lowercase().as_str() {
            "y" => Ok(ConfirmAction::RM),
            "n" => Ok(ConfirmAction::NONE),
            "l" => Ok(ConfirmAction::LIST),
            _ => Err(()),
        }
    }
}


pub fn extract_directory(arg_directory: Option<&str>) -> io::Result<PathBuf> {
    arg_directory.map_or_else(|| env::current_dir(), |d| {
        let mut path = PathBuf::new();
        path.push(d);

        if !path.exists() {
            Err(
                io::Error::new(io::ErrorKind::NotFound, format!("directory {:?} not found.", path.display()))
            )
        } else {
            Ok(path)
        }
    })
}

pub fn get_rec(path: &PathBuf) -> Vec<io::Result<PathBuf>> {
    let mut vec: Vec<io::Result<PathBuf>> = Vec::new();
    if path.is_dir() {
        // remove node modules in the current path
        let result = get(path);

        if result.is_err() { // only iterate further if no node_modules has been found
            let entries = path.read_dir()
                .expect(format!("couldn't read directory entries from directory {}", path.display()).as_str());
            for entry in entries {
                if let Ok(dir) = entry {
                    vec.append(&mut get_rec(&dir.path()));
                }
            }
        } else {
            vec.push(result);
        }
    }

    vec
}

pub fn get(path: &PathBuf) -> io::Result<PathBuf> {
    let mut modules_path = path.clone();
    modules_path.push("node_modules");
    if modules_path.exists() && modules_path.is_dir()  {
        Ok(modules_path)
    } else {
        Err(
            io::Error::new(io::ErrorKind::NotFound, format!("directory {:?} not found.", modules_path.display()))
        )
    }
}
