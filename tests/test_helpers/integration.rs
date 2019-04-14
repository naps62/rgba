extern crate reqwest;

use std::io;
use std::fs::{self, File};


pub fn ensure_file_cached(url: &str, name: &str) {
    let mut req = reqwest::get(url).expect("request failed");

    let _ = fs::create_dir("cache");
    let filename = format!("cache/{}", name);

    let mut out = File::create(filename).expect("failed to create file");

    io::copy(&mut req, &mut out).expect("failed to copy");
    println!("{}", name);
}
