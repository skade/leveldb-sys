extern crate leveldb_sys as sys;
extern crate tempfile;

#[test]
fn major_version() {
    let version = unsafe {
        sys::leveldb_major_version()
    };
    assert_eq!(version, 1);
}

#[test]
fn minor_version() {
    let version = unsafe {
        sys::leveldb_minor_version()
    };
    assert!(version >= 23);
}

#[test]
fn open_close() {
    use std::ffi::{c_char, CStr, CString};

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("db");
    let name = CString::new(path.to_str().unwrap()).unwrap();
    let name = name.as_ptr();
    unsafe {
        let mut error: *mut c_char = std::ptr::null_mut();
        let options = sys::leveldb_options_create();
        assert!(!options.is_null());
        sys::leveldb_options_set_create_if_missing(options, 1);
        let db = sys::leveldb_open(options, name, &mut error);

        if db.is_null() {
            assert!(!error.is_null());
            let msg = CStr::from_ptr(error).to_string_lossy();
            panic!("unable to open db: {}", msg);
        }

        sys::leveldb_close(db);

        sys::leveldb_options_destroy(options);
    }
}
