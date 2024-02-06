extern crate shell_escape;

use shell_escape::escape;
use std::{
    env,
    path::{Path, PathBuf},
};

#[cfg(feature = "snappy")]
const SNAPPY_VERSION: &'static str = "1.1.7";
const LEVELDB_VERSION: &'static str = "1.22";
/// Directory name within `$OUT_DIR` where the static libraries should be built.
const LIBDIR: &'static str = "lib";

/// Escape a path for use by the shell.
fn escape_path(path: impl AsRef<Path>) -> String {
    escape(format!("{}", path.as_ref().display()).into()).into()
}

#[cfg(feature = "snappy")]
fn build_snappy() -> PathBuf {
    println!("[snappy] Building");

    let outdir = env::var("OUT_DIR").unwrap();
    let libdir = Path::new(&outdir).join(LIBDIR);

    // Paths containing double quotes seem to break the CMake build.
    if outdir.contains('"') {
        panic!("can't build at path containing double quotes");
    }

    env::set_var("NUM_JOBS", num_cpus::get().to_string());
    let dest_prefix =
        cmake::Config::new(Path::new("deps").join(format!("snappy-{}", SNAPPY_VERSION)))
            .define("BUILD_SHARED_LIBS", "OFF")
            .define("SNAPPY_BUILD_TESTS", "OFF")
            .define("HAVE_LIBZ", "OFF")
            .define("CMAKE_INSTALL_LIBDIR", escape_path(&libdir))
            .build();

    assert_eq!(
        dest_prefix.join(LIBDIR),
        libdir,
        "CMake should build Snappy in provided LIBDIR"
    );
    println!("cargo:rustc-link-search=native={}", libdir.display());
    println!("cargo:rustc-link-lib=static=snappy");

    dest_prefix
}

fn build_leveldb(snappy_prefix: Option<PathBuf>) {
    println!("[leveldb] Building");

    let outdir = env::var("OUT_DIR").unwrap();
    let libdir = Path::new(&outdir).join(LIBDIR);

    // Paths containing double quotes seem to break the CMake build.
    if outdir.contains('"') {
        panic!("can't build at path containing double quotes");
    }

    env::set_var("NUM_JOBS", num_cpus::get().to_string());
    let mut config =
        cmake::Config::new(Path::new("deps").join(format!("leveldb-{}", LEVELDB_VERSION)));
    config
        .define("LEVELDB_BUILD_TESTS", "OFF")
        .define("LEVELDB_BUILD_BENCHMARKS", "OFF")
        .define("CMAKE_INSTALL_LIBDIR", &libdir);
    if let Some(snappy_prefix) = snappy_prefix {
        #[cfg(target_env = "msvc")]
        let ldflags = format!("/LIBPATH:{}", escape_path(snappy_prefix.join(LIBDIR)));
        #[cfg(not(target_env = "msvc"))]
        let ldflags = format!("-L{}", escape_path(snappy_prefix.join(LIBDIR)));

        env::set_var("LDFLAGS", ldflags);

        config
            .define("HAVE_SNAPPY", "ON")
            .cflag(format!("-I{}", escape_path(snappy_prefix.join("include"))))
            .cxxflag(format!("-I{}", escape_path(snappy_prefix.join("include"))));
    } else {
        config.define("HAVE_SNAPPY", "OFF");
    }
    let dest_prefix = config.build();

    assert_eq!(
        dest_prefix.join(LIBDIR),
        libdir,
        "CMake should build LevelDB in provided LIBDIR"
    );
    println!("cargo:rustc-link-search=native={}", libdir.display());
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
    } else if target.contains("musl") {
        // We want to link to libstdc++ *statically*. This requires that the user passes the right
        // search path to rustc via `-Lstatic=/path/to/libstdc++`.
        println!("cargo:rustc-link-lib=static=stdc++");
    }

    println!("[build] Finished");
}
