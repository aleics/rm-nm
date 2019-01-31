extern crate clap;
extern crate rayon;

use clap::{App, load_yaml};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{io, fs, env};
use std::path::PathBuf;

fn main() {
    let cli = load_yaml!("cli.yml");
    let matches = App::from_yaml(cli).get_matches();

    let recursive = matches.is_present("recursive");
    let directory = extract_directory(matches.value_of("directory")).unwrap();

    let results = if recursive {
        get_rec( &directory)
    } else {
        vec![get(&directory)]
    };

    results.into_par_iter()
        .filter_map(|result| result.ok())
        .for_each(|path| {
            if let Err(e) = fs::remove_dir(path) {
                println!("couldn't delete directory ({})", e)
            }
        });

}

fn extract_directory(arg_directory: Option<&str>) -> io::Result<PathBuf> {
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

fn get_rec(path: &PathBuf) -> Vec<io::Result<PathBuf>> {
    let mut vec: Vec<io::Result<PathBuf>> = Vec::new();
    if path.is_dir() {
        // remove node modules in the current path
        vec.push(get(path));

        let entries = path.read_dir()
            .expect(format!("couldn't read directory entries from directory {}", path.display()).as_str());
        for entry in entries {
            if let Ok(dir) = entry {
                vec.append(&mut get_rec(&dir.path()));
            }
        }
    }

    vec
}

fn get(path: &PathBuf) -> io::Result<PathBuf> {
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

mod tests {
    use super::*;

    fn prepare_dir<T>(test: T) -> ()
        where T: Fn(PathBuf) -> () {
        let mut current = env::current_dir().unwrap();
        current.push("node_modules");

        if !current.exists() {
            fs::create_dir(current.clone()).expect("should create a node_modules directory");
        }

        // set to initial current directory
        current.pop();

        test(current.clone());

        current.push("node_modules");
        if current.exists() {
            fs::remove_dir(current.clone()).expect("should remove a node_modules directory");
        }
    }

    #[test]
    fn test_extract_current_directory() {
        let directory = extract_directory(None).unwrap();
        assert_eq!(directory, env::current_dir().unwrap());
    }

    #[test]
    fn test_extract_some_non_existing_directory() {
        assert!(extract_directory(Some("/some/directory")).is_err());
    }

    #[test]
    fn test_extract_some_existing_directory() {
        let current = env::current_dir().unwrap();
        let path = extract_directory(current.to_str()).unwrap();
        assert_eq!(path, current);
    }

    #[test]
    fn test_get_dir() {
        prepare_dir(|current_dir| {
            match get(&current_dir) {
                Ok(result) => {
                    let mut rm_dir = current_dir.clone();
                    rm_dir.push("node_modules");
                    assert_eq!(result, rm_dir)
                },
                Err(e) => assert!(false, format!("Error {:?}", e))
            }
        });
    }

    #[test]
    fn test_get_non_existing_dir() {
        let mut dir = PathBuf::new();
        dir.push("/some/directory");

        assert!(get(&dir).is_err());
    }

    #[test]
    fn test_get_rec_dirs() {
        let mut current = env::current_dir().unwrap();
        current.push("custom_1/node_modules");

        if !current.exists() {
            fs::create_dir_all(current.clone())
                .expect("should create a custom_1/node_modules directory");
        }

        let modules_dir = current.clone();

        current.pop();
        current.pop();

        let result = get_rec(&current)
            .into_iter()
            .find(|r| r.is_ok())
            .unwrap()
            .unwrap();
        assert_eq!(result, modules_dir);

        current.push("custom_1");
        if current.exists() {
            fs::remove_dir_all(current.clone()).expect("should delete custom_1 directory")
        }

        current = env::current_dir().unwrap();
        current.push("custom_1/custom_1_1/node_modules");

        if !current.exists() {
            fs::create_dir_all(current.clone())
                .expect("should create a custom_1/custom_1_1/node_modules directory");
        }

        let modules_dir = current.clone();

        current.pop();
        current.pop();
        current.pop();

        let result = get_rec(&current)
            .into_iter()
            .find(|r| r.is_ok())
            .unwrap()
            .unwrap();
        assert_eq!(result, modules_dir);

        current.push("custom_1");
        if current.exists() {
            fs::remove_dir_all(current.clone()).expect("should delete custom_1 directory")
        }
    }

    #[test]
    fn test_get_rec_non_existing_dir() {
        let mut dir = PathBuf::new();
        dir.push("/some/directory");

        let result = get_rec(&dir);
        assert!(result.first().is_none());
    }
}
