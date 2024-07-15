use std::env;

#[cfg(feature = "snappy")]
fn link_snappy() {
    println!("cargo:rustc-link-lib=snappy");
}

fn link_leveldb() {
    println!("cargo:rustc-link-lib=leveldb");
}

fn main() {
    // If we have the appropriate feature, then we build snappy.
    #[cfg(feature = "snappy")]
    link_snappy();

    // Build LevelDB
    link_leveldb();

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
}
