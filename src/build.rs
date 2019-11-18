use std::{
    env,
    path::{Path, PathBuf},
};

#[cfg(feature = "snappy")]
const SNAPPY_VERSION: &'static str = "1.1.7";
const LEVELDB_VERSION: &'static str = "1.22";

#[cfg(feature = "snappy")]
fn build_snappy() -> PathBuf {
    println!("[snappy] Building");

    env::set_var("NUM_JOBS", num_cpus::get().to_string());
    let dest_prefix =
        cmake::Config::new(Path::new("deps").join(format!("snappy-{}", SNAPPY_VERSION)))
            .define("BUILD_SHARED_LIBS", "OFF")
            .define("SNAPPY_BUILD_TESTS", "OFF")
            .define("HAVE_LIBZ", "OFF")
            .build();

    println!(
        "cargo:rustc-link-search=native={}",
        dest_prefix.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=snappy");

    dest_prefix
}

fn build_leveldb(snappy_prefix: Option<PathBuf>) {
    println!("[leveldb] Building");

    env::set_var("NUM_JOBS", num_cpus::get().to_string());
    let mut config =
        cmake::Config::new(Path::new("deps").join(format!("leveldb-{}", LEVELDB_VERSION)));
    config
        .define("LEVELDB_BUILD_TESTS", "OFF")
        .define("LEVELDB_BUILD_BENCHMARKS", "OFF");
    if let Some(snappy_prefix) = snappy_prefix {
        #[cfg(target_env = "msvc")]
        let ldflags = format!("/LIBPATH:{}", snappy_prefix.join("lib").display());
        #[cfg(not(target_env = "msvc"))]
        let ldflags = format!("-L{}", snappy_prefix.join("lib").display());
        env::set_var("LDFLAGS", ldflags);
        config
            .define("HAVE_SNAPPY", "ON")
            .cflag(format!("-I{}", snappy_prefix.join("include").display()))
            .cxxflag(format!("-I{}", snappy_prefix.join("include").display()));
    } else {
        config.define("HAVE_SNAPPY", "OFF");
    }
    let dest_prefix = config.build();

    println!(
        "cargo:rustc-link-search=native={}",
        dest_prefix.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=leveldb");
}

fn main() {
    println!("[build] Started");

    // If we have the appropriate feature, then we build snappy.
    #[cfg(feature = "snappy")]
    let snappy_prefix = Some(build_snappy());
    #[cfg(not(feature = "snappy"))]
    let snappy_prefix: Option<PathBuf> = None;

    // Build LevelDB
    build_leveldb(snappy_prefix);

    // Link to the standard C++ library
    let target = env::var("TARGET").unwrap();
    if target.contains("apple") || target.contains("freebsd") {
        println!("cargo:rustc-link-lib=c++");
    } else if target.contains("gnu") || target.contains("netbsd") || target.contains("openbsd") {
        println!("cargo:rustc-link-lib=stdc++");
    }

    println!("[build] Finished");
}
