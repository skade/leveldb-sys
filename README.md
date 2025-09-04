# leveldb-sys

Lowlevel bindings to the leveldb C library.

## Dependencies

* Your platforms C++ compiler (usually `gcc` or `clang` on Linux and Unix, Visual Studio Build environment on Windows)
* `cmake`

## Usage

If your project is using Cargo, drop the following lines in your Cargo.toml:

```
[dependencies]

leveldb-sys = "*"
```

## Features

`levelbd-sys` offers a `snappy` feature to build the snappy library.

The `vendor` feature (enabled by default) uses a bundled version of leveldb
and snappy. Disabling this feature (or setting the environment variable
`LEVELDB_NO_VENDOR=1`) will use the system-wide libraries.

## LICENSE

MIT

## BSD support

To build leveldb-sys you need to install `gmake` (GNU Make)
