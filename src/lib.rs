use std::path::PathBuf;

use path_matchers::{PathMatcher, glob};

pub mod ansi_state_machine;

pub fn path_matches(list: &Vec<String>, path: &str) -> bool {
    for item in list {
        let matcher =
            glob(item.as_str()).expect("Invalid path string! Please provide a correct one.");
        if matcher.matches(&PathBuf::from(&path)) {
            return true;
        }
    }

    return false;
}
