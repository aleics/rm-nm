extern crate clap;
extern crate rayon;

use clap::{App, load_yaml};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use std::{io, fs};
use std::path::PathBuf;
use rm_nm::dir::*;

fn main() {
    let cli = load_yaml!("cli.yml");
    let matches = App::from_yaml(cli).get_matches();

    let recursive = matches.is_present("recursive");
    let directory = extract_directory(matches.value_of("directory")).unwrap();

    let paths: Vec<PathBuf> = if recursive {
        get_rec( &directory)
    } else {
        vec![get(&directory)]
    }.into_par_iter().filter_map(|result| result.ok()).collect();


    let count = paths.len();
    if count > 0 {
        println!("Are you sure you want to delete {} directories? (Y/n/l)", count);
        let stdin = io::stdin();
        let mut confirm_action: ConfirmAction = ConfirmAction::NONE;
        let mut action_defined = false;

        while !action_defined {
            let confirm = &mut String::new();
            stdin.read_line(confirm).unwrap();
            confirm.pop(); // pop new line
            if let Ok(action) = confirm.parse::<ConfirmAction>() {
                action_defined = true;
                confirm_action = action;
            }
        }

        let rm_fn: fn(p: Vec<PathBuf>) = |p| {
            p.into_par_iter()
                .for_each(|path| {
                    if let Err(e) = fs::remove_dir_all(path) {
                        println!("couldn't delete directory ({})", e)
                    }
                });
        };

        let list_fn: fn(p: Vec<PathBuf>) = |p| {
            p.into_par_iter()
                .for_each(|path| {
                    println!("{}", path.display());
                });
        };

        match confirm_action {
            ConfirmAction::RM => { rm_fn(paths) },
            ConfirmAction::LIST => { list_fn(paths) },
            ConfirmAction::NONE => {}
        };
    } else {
        println!("No 'node_modules' directory found.")
    }
}
