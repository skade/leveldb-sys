use std::collections::HashMap;
use std::ffi::CString;
use std::io::{BufferedReader, Command, File, fs};
use std::os;
use std::path::Path;

use std::path::BytesContainer;

const SNAPPY_VERSION: &'static str  = "1.1.2";
const LEVELDB_VERSION: &'static str = "1.18";


fn build_snappy() {
    // Step 1: Build snappy
    // ----------------------------------------------------------------------
    let snappy_path = Path::new("deps")
                           .join(format!("snappy-{}", SNAPPY_VERSION));

    // Clean the build directory first.
    println!("[snappy] Cleaning");
    Command::new("make").args(&["-C", snappy_path.as_str().unwrap()])
                        .arg("distclean")
                        .status().unwrap();

    // Configure the build
    println!("[snappy] Configuring");
    Command::new("./configure").cwd(&snappy_path)
                               .arg("CXXFLAGS=-fPIC")
                               .status().unwrap();

    // Call "make" to build the C library
    println!("[snappy] Building");
    Command::new("make").args(&["-C", snappy_path.as_str().unwrap()])
                        .status().unwrap();

    // Step 2: Copy to output directories
    // ----------------------------------------------------------------------
    let out_dir = Path::new(os::getenv("OUT_DIR").unwrap());

    println!("[build] Copying output files");
    fs::copy(&snappy_path.join(".libs").join("libsnappy.a"), &out_dir.join("libsnappy.a")).unwrap();
}

fn build_leveldb(with_snappy: bool) {
    // Step 1: Build LevelDB
    // ----------------------------------------------------------------------
    let leveldb_path = Path::new("deps")
                            .join(format!("leveldb-{}", LEVELDB_VERSION));

    // Clean the build directory first.
    println!("[leveldb] Cleaning");
    Command::new("make").args(&["-C", leveldb_path.as_str().unwrap()])
                        .arg("clean")
                        .status().unwrap();

    // Set up the process environment.  We essentially clone the existing
    // environment, and, if we're including Snappy, also include the appropriate
    // CXXFLAGS and LDFLAGS variables.
    let mut env_map: HashMap<CString, CString> = os::env_as_bytes().into_iter().map(|(k, v)| {
        (
            CString::from_slice(k.as_slice()),
            CString::from_slice(v.as_slice())
        )
    }).collect();

    if with_snappy {
        let linker_path = os::getenv("OUT_DIR").unwrap();
        env_map.insert(
            CString::from_slice("LDFLAGS".as_bytes()),
            CString::from_slice(format!("-L{}", linker_path).as_bytes())
        );

        let snappy_path = Path::new("deps")
                               .join(format!("snappy-{}", SNAPPY_VERSION));
        env_map.insert(
            CString::from_slice("CXXFLAGS".as_bytes()),
            CString::from_slice(format!("-I{} -fPIC", snappy_path.as_str().unwrap()).as_bytes())
        );
    } else {
        env_map.insert(
            CString::from_slice("CXXFLAGS".as_bytes()),
            CString::from_slice(format!("-fPIC").as_bytes())
        );
    }

    // Convert to the format that `env_set_all` is expecting.
    let env_arr: Vec<(CString, CString)> = env_map.into_iter().collect();

    // Build the library
    println!("[leveldb] Building");
    let mut pp = Command::new("make")
            .args(&["-C", leveldb_path.as_str().unwrap()])
            .env_set_all(env_arr.as_slice())
            .spawn().unwrap();
    let stdout_buff = pp.stdout.as_mut().unwrap().read_to_end().unwrap();
    let stdout = String::from_utf8_lossy(stdout_buff.as_slice());
    println!("[leveldb] Process stdout = \"{}\"", stdout.escape_default());

    let stderr_buff = pp.stderr.as_mut().unwrap().read_to_end().unwrap();
    let stderr = String::from_utf8_lossy(stderr_buff.as_slice());
    println!("[leveldb] Process stderr = \"{}\"", stderr.escape_default());

    // Step 2: Copy to output directories
    // ----------------------------------------------------------------------
    let out_dir = Path::new(os::getenv("OUT_DIR").unwrap());

    println!("[build] Copying output files");
    fs::copy(&leveldb_path.join("libleveldb.a"), &out_dir.join("libleveldb.a")).unwrap();
}

fn main() {
    println!("[build] Started");

    let have_snappy = os::getenv("CARGO_FEATURE_SNAPPY").is_some();

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
            let mut file = BufferedReader::new(File::open(&template_path));

            file.lines().map(|line| {
                let mut line = line.unwrap();
                if line.contains("-DSNAPPY") || line.contains("-lsnappy") {
                    let mut tmp = String::new();
                    tmp.push_str("true   #");
                    tmp.push_str(line.as_slice());
                    tmp
                } else {
                    line
                }
            }).collect()
        };

        let mut f = File::create(&detect_path);
        for line in new_lines.iter() {
            f.write_str(line.as_slice());
        }

        println!("[build] Patching complete");
    }

    // Set executable bits on the file
    fs::chmod(&detect_path, std::io::USER_EXEC).unwrap();

    // Build LevelDB
    build_leveldb(have_snappy);

    // Print the appropriate linker flags
    let out_dir = os::getenv("OUT_DIR").unwrap();
    let linker_flags = if have_snappy {
        "-l static=snappy -l static=leveldb -l stdc++"
    } else {
        "-l static=leveldb -l stdc++"
    };
    println!("cargo:rustc-flags=-L native={} {} -l stdc++", out_dir, linker_flags);

    println!("[build] Finished");
}
