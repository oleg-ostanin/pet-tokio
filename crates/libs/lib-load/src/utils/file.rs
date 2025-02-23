use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use serde::Deserialize;
use serde_json::Value;

pub(crate) fn from_file<T: for<'a> Deserialize<'a>>(path: impl AsRef<Path>) -> T {
    //serde_json::from_value::<T>(json).unwrap()

    let mut path_buf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path_buf.push("resources/");
    path_buf.push(path);

    let file = File::open(path_buf).expect("Should be there");
    let reader = BufReader::new(file);
    let value: Value = serde_json::from_reader(reader).expect("Should be valid");
    serde_json::from_value::<T>(value).expect("Should be valid")
}