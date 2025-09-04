use std::env;

#[cfg(feature = "vendor")]
mod build {
    use std::env;
    use std::path::Path;

    const LIBDIR: &'static str = "lib";

    #[cfg(feature = "snappy")]
    pub fn build_snappy() -> std::path::PathBuf {
        println!("[snappy] Building");

        let outdir = env::var("OUT_DIR").unwrap();
        let libdir = Path::new(&outdir).join(LIBDIR);

        env::set_var("NUM_JOBS", num_cpus::get().to_string());
        let dest_prefix = cmake::Config::new(Path::new("deps").join("snappy"))
            .define("BUILD_SHARED_LIBS", "OFF")
            .define("SNAPPY_BUILD_BENCHMARKS", "OFF")
            .define("SNAPPY_BUILD_TESTS", "OFF")
            .define("HAVE_LIBZ", "OFF")
            .define("HAVE_LIBLZO2", "OFF")
            .define("HAVE_LIBLZ4", "OFF")
            .define("CMAKE_INSTALL_LIBDIR", &libdir)
            .build();

        assert_eq!(
            dest_prefix.join(LIBDIR),
            libdir,
            "CMake should build Snappy in provided LIBDIR"
        );
        println!("cargo:rustc-link-search=native={}", libdir.display());

        dest_prefix
    }

    pub fn build_leveldb(snappy_prefix: Option<&Path>) {
        println!("[leveldb] Building");

        let outdir = env::var("OUT_DIR").unwrap();
        let libdir = Path::new(&outdir).join(LIBDIR);

        env::set_var("NUM_JOBS", num_cpus::get().to_string());

        let mut config = cmake::Config::new(Path::new("deps").join("leveldb"));
        config
            .define("LEVELDB_BUILD_TESTS", "OFF")
            .define("LEVELDB_BUILD_BENCHMARKS", "OFF")
            .define("CMAKE_INSTALL_LIBDIR", &libdir)
            .define("HAVE_CRC32C", "OFF");

        #[cfg(feature = "snappy")]
        config.define("HAVE_SNAPPY", "ON");
        #[cfg(not(feature = "snappy"))]
        config.define("HAVE_SNAPPY", "OFF");

        if let Some(snappy_prefix) = snappy_prefix {
            #[cfg(target_env = "msvc")]
            let ldflags = format!("/LIBPATH:{}", snappy_prefix.join(LIBDIR).display());
            #[cfg(not(target_env = "msvc"))]
            let ldflags = format!("-L{}", snappy_prefix.join(LIBDIR).display());

            env::set_var("LDFLAGS", ldflags);

            config
                .cflag(format!("-I{}", snappy_prefix.join("include").display()))
                .cxxflag(format!("-I{}", snappy_prefix.join("include").display()));
        }
        let dest_prefix = config.build();

        assert_eq!(
            dest_prefix.join(LIBDIR),
            libdir,
            "CMake should build LevelDB in provided LIBDIR"
        );
        println!("cargo:rustc-link-search=native={}", libdir.display());
    }
}

#[cfg(feature = "snappy")]
fn link_snappy() {
    #[cfg(feature = "vendor")]
    println!("cargo:rustc-link-lib=static=snappy");

    #[cfg(not(feature = "vendor"))]
    println!("cargo:rustc-link-lib=snappy");
}

fn link_leveldb() {
    #[cfg(feature = "vendor")]
    println!("cargo:rustc-link-lib=static=leveldb");

    #[cfg(not(feature = "vendor"))]
    println!("cargo:rustc-link-lib=leveldb");
}

fn main() {
    #[cfg(all(feature = "snappy", feature = "vendor"))]
    let snappy_prefix = Some(build::build_snappy());

    #[cfg(all(not(feature = "snappy"), feature = "vendor"))]
    let snappy_prefix: Option<std::path::PathBuf> = None;

    #[cfg(feature = "snappy")]
    link_snappy();

    #[cfg(feature = "vendor")]
    build::build_leveldb(snappy_prefix.as_deref());

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
