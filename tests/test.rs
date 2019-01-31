mod tests {
    extern crate rnm;

    use std::{fs, env};
    use std::path::PathBuf;
    use rnm::dir::*;

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
            .filter_map(|r| r.ok())
            .find(|p | p == &modules_dir)
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
            .filter_map(|r| r.ok())
            .find(|p | p == &modules_dir)
            .unwrap();
        assert_eq!(result, modules_dir);

        current.push("custom_1");
        if current.exists() {
            fs::remove_dir_all(current.clone()).expect("should delete custom_1 directory")
        }
    }

    #[test]
    fn test_get_rec_multiple_dirs() {
        let mut current = env::current_dir().unwrap();
        current.push("custom_2/node_modules/custom_2_2/node_modules");

        if !current.exists() {
            fs::create_dir_all(current.clone())
                .expect("should create a custom_2/node_modules/custom_2_2/node_modules directory");
        }

        current.pop();
        current.pop();

        let modules_dir = current.clone();

        current.pop();
        current.pop();

        let result = get_rec(&current)
            .into_iter()
            .filter_map(|r| r.ok())
            .find(|r| r == &modules_dir)
            .unwrap();
        assert_eq!(result, modules_dir);

        current.push("custom_2");
        if current.exists() {
            fs::remove_dir_all(current.clone()).expect("should delete custom_2 directory")
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
