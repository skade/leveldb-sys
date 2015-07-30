use std::collections::HashMap;
use std::ffi::OsString;
use std::fs::File;
use std::fs;
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::io::{Write,BufReader};
use std::io::BufRead;
use std::process::Command;
use std::path::Path;

const SNAPPY_VERSION: &'static str  = "1.1.2";
const LEVELDB_VERSION: &'static str = "1.18";


fn build_snappy() {
    // Step 1: Build snappy
    // ----------------------------------------------------------------------
    let snappy_path = Path::new("deps")
                           .join(format!("snappy-{}", SNAPPY_VERSION));

    // Clean the build directory first.
    println!("[snappy] Cleaning");
    Command::new("make").args(&["-C", snappy_path.to_str().unwrap()])
                        .arg("distclean")
                        .status().ok().expect("make distclean failed");

    // Configure the build
    println!("[snappy] Configuring");
    Command::new("./configure").current_dir(&snappy_path)
                               .arg("CXXFLAGS=-fPIC")
                               .status().ok().expect("configure failed");

    // Call "make" to build the C library
    println!("[snappy] Building");
    Command::new("make").args(&["-C", snappy_path.to_str().unwrap()])
                        .status().ok().expect("make failed");

    // Step 2: Copy to output directories
    // ----------------------------------------------------------------------
    let env_var = &env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(env_var);

    println!("[build] Copying output files");
    let res = fs::copy(&snappy_path.join(".libs").join("libsnappy.a"), &out_dir.join("libsnappy.a"));
    res.ok().expect("copy of output files failed");
}

fn build_leveldb(with_snappy: bool) {
    // Step 1: Build LevelDB
    // ----------------------------------------------------------------------
    let leveldb_path = Path::new("deps")
                            .join(format!("leveldb-{}", LEVELDB_VERSION));

    // Clean the build directory first.
    println!("[leveldb] Cleaning");
    Command::new("make").args(&["-C", leveldb_path.to_str().unwrap()])
                        .arg("clean")
                        .status().ok().expect("clean failed");

    // Set up the process environment.  We essentially clone the existing
    // environment, and, if we're including Snappy, also include the appropriate
    // CXXFLAGS and LDFLAGS variables.
    let mut env_map: HashMap<OsString, OsString> = env::vars().map(|(k, v)| {
        (
            k.into(),
            v.into()
        )
    }).collect();

    if with_snappy {
        let linker_path = env::var("OUT_DIR").unwrap();
        env_map.insert(
            "LDFLAGS".into(),
            format!("-L{}", linker_path).into()
        );

        let snappy_path = Path::new("deps")
                               .join(format!("snappy-{}", SNAPPY_VERSION));
        env_map.insert(
            "CXXFLAGS".into(),
            format!("-I{} -fPIC", snappy_path.to_str().unwrap()).into()
        );
    } else {
        env_map.insert(
            "CXXFLAGS".into(),
            format!("-fPIC").into()
        );
    }

    let mut cmd = Command::new("make");

    println!("[leveldb] Building command");

    // Convert to the format that `env_set_all` is expecting.
    for (k,v) in env_map.into_iter() {
        cmd.env(k,v);
    }

    let path_arg = leveldb_path.to_str().expect("leveldb path is not a string");

    // Build the library
    println!("[leveldb] Building");
    cmd.args(&["-C", path_arg])
       .status().ok().expect("leveldb build failed");

    println!("[leveldb] Build finished");
    // Step 2: Copy to output directories
    // ----------------------------------------------------------------------
    let env_var = &env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(env_var);

    println!("[build] Copying output files");
    let res = fs::copy(&leveldb_path.join("libleveldb.a"), &out_dir.join("libleveldb.a"));
    res.ok().expect("copy of output files failed");
}

fn main() {
    println!("[build] Started");

    let have_snappy = env::var("CARGO_FEATURE_SNAPPY").is_ok();

    // If we have the appropriate feature, then we build snappy.
    if have_snappy {
        build_snappy();
    }

    // Copy the build_detect_platform file into the appropriate place.
    let template_path = Path::new("deps")
                             .join("build_detect_platform");
    let detect_path = Path::new("deps")
                           .join(format!("leveldb-{}", LEVELDB_VERSION))
                           .join("build_detect_platform");
    if have_snappy {
        println!("[build] Copying the `build_detect_platform` template");
        fs::copy(&template_path, &detect_path).unwrap();
    } else {
        println!("[build] Patching the `build_detect_platform` template");

        // If we aren't using snappy, remove the lines from
        // build_detect_platform that enable Snappy.  This prevents us from
        // picking up a system-local copy of Snappy.
        let new_lines: Vec<String> = {
            let file = File::open(&template_path).unwrap();
            let reader = BufReader::new(file);

            reader.lines().map(|line| {
                let line = line.unwrap();
                if line.contains("-DSNAPPY") || line.contains("-lsnappy") {
                    let mut tmp = String::new();
                    tmp.push_str("true   #");
                    tmp.push_str(line.as_ref());
                    tmp.push_str("\n");
                    tmp
                } else {
                    line
                }
            }).collect()
        };

        let mut f = File::create(&detect_path).unwrap();
        for line in new_lines.iter() {
            f.write_all(line.as_ref()).ok().expect("writing a line failed");
            f.write_all("\n".as_bytes()).ok().expect("writing a line failed");
        }

        println!("[build] Patching complete");
    }

    // Set executable bits on the file

    let mut perms = fs::metadata(&detect_path).ok().expect("metadata missing").permissions();
    let current_mode = perms.mode();
    perms.set_mode(0o100 | current_mode);
    fs::set_permissions(&detect_path, perms).ok().expect("permissions could not be set");

    // Build LevelDB
    build_leveldb(have_snappy);

    // Print the appropriate linker flags
    let out_dir = env::var("OUT_DIR").ok().expect("OUT_DIR missing");
    let linker_flags = if have_snappy {
        "-l static=snappy -l static=leveldb -l stdc++"
    } else {
        "-l static=leveldb -l stdc++"
    };
    println!("cargo:rustc-flags=-L native={} {} -l stdc++", out_dir, linker_flags);

    println!("[build] Finished");
}
