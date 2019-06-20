extern crate reqwest;
extern crate zip;

use std::fs::{self, File};
use std::io;

pub fn ensure_file_cached(url: &str, name: &str) {
  let mut req = reqwest::get(url).expect("request failed");

  let _ = fs::create_dir("cache");
  let filename = format!("cache/{}", name);

  let mut out = File::create(filename).expect("failed to create file");

  io::copy(&mut req, &mut out).expect("failed to copy");
  println!("{}", name);
}

pub fn ensure_zip_extracted(zip_name: &str) {
  let zip_path = format!("cache/{}", zip_name);

  let file = File::open(zip_path).unwrap();
  let mut archive = zip::ZipArchive::new(file).unwrap();

  for i in 0..archive.len() {
    let mut file = archive.by_index(i).unwrap();
    let mut outpath = std::path::PathBuf::new();
    outpath.push("cache/");
    outpath.push(file.sanitized_name());

    if (&*file.name()).ends_with('/') {
      fs::create_dir_all(&outpath).unwrap();
    } else {
      if let Some(p) = outpath.parent() {
        if !p.exists() {
          fs::create_dir_all(&p).unwrap();
        }
      }
      let mut outfile = fs::File::create(&outpath).unwrap();
      io::copy(&mut file, &mut outfile).unwrap();
    }

    // Get and Set permissions
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;

      if let Some(mode) = file.unix_mode() {
        fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
      }
    }
  }
}
