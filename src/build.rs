use std::io::{Command, fs};
use std::os;
use std::path::Path;

// TODO: include this from a common location
const SNAPPY_VERSION: &'static str  = "1.1.2";
const LEVELDB_VERSION: &'static str = "1.18";

fn main() {
    println!("[build] Started");

    // Step 1: Build snappy
    // ----------------------------------------------------------------------
    let leveldb_path = Path::new("..")
                            .join("deps")
                            .join(format!("leveldb-{}", LEVELDB_VERSION));

    // Clean the build directory first.
    println!("[leveldb] Cleaning");
    Command::new("make").args(&["-C", leveldb_path.as_str().unwrap()])
                        .arg("clean")
                        .status().unwrap();

    // Make the library
    println!("[leveldb] Building");
    let snappy_path = Path::new("..")
                           .join("deps")
                           .join(format!("snappy-{}", SNAPPY_VERSION));
    Command::new("make").args(&["-C", leveldb_path.as_str().unwrap()])
                        .env("LDFLAGS",  format!("-L{}",       snappy_path.as_str().unwrap()))
                        .env("CXXFLAGS", format!("-I{} -fPIC", snappy_path.as_str().unwrap()))
                        .status().unwrap();

    // Step 2: Copy to output directories
    // ----------------------------------------------------------------------
    let out_dir = Path::new(os::getenv("OUT_DIR").unwrap());

    println!("[build] Copying output files");
    fs::copy(&leveldb_path.join("libleveldb.a"), &out_dir.join("libleveldb.a")).unwrap();

    // Step 3: Tell Rust about what we link to
    // ----------------------------------------------------------------------
    let out_dir = os::getenv("OUT_DIR").unwrap();
    println!("cargo:rustc-flags=-L {} -l leveldb:static", out_dir);
    println!("[build] Finished");
}
