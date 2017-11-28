// Copyright 2017 PingCAP, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// See the License for the specific language governing permissions and
// limitations under the License.


extern crate cc;
extern crate cmake;

use cc::Build;
use std::{env, str};
use cmake::Config;

fn main() {
    let mut cfg = build_rocksdb();

    cfg.cpp(true).file("crocksdb/c.cc");
    if !cfg!(target_os = "windows") {
        cfg.flag("-std=c++11");
    }
    cfg.compile("libcrocksdb.a");

    println!("cargo:rustc-link-lib=static=crocksdb");
}

fn build_rocksdb() -> Build {
    let mut build = Build::new();
    for e in env::vars() {
        println!("{:?}", e);
    }

    let mut cfg = Config::new("rocksdb");
    if cfg!(feature = "portable") {
        cfg.define("PORTABLE", "ON");
    }
    if cfg!(feature = "sse") {
        cfg.define("FORCE_SSE42", "ON");
    }
    let dst = cfg.register_dep("Z").define("WITH_ZLIB", "ON")
        .register_dep("BZIP2").define("WITH_BZ2", "ON")
        .register_dep("LZ4").define("WITH_LZ4", "ON")
        .register_dep("ZSTD").define("WITH_ZSTD", "ON")
        .register_dep("SNAPPY").define("WITH_SNAPPY", "ON")
        .build_target("rocksdb").build();
    let build_dir = format!("{}/build", dst.display());
    if cfg!(target_os = "windows") {
        let profile = match &*env::var("PROFILE").unwrap_or("debug".to_owned()) {
            "bench" | "release" => "Release",
            _ => "Debug",
        };
        println!("cargo:rustc-link-search=native={}/{}", build_dir, profile);
    } else {
        println!("cargo:rustc-link-search=native={}", build_dir);
    }

    let cur_dir = env::current_dir().unwrap();
    build.include(cur_dir.join("rocksdb").join("include"));

    println!("cargo:rustc-link-lib=static=rocksdb");
    println!("cargo:rustc-link-lib=static=z");
    println!("cargo:rustc-link-lib=static=bz2");
    println!("cargo:rustc-link-lib=static=lz4");
    println!("cargo:rustc-link-lib=static=zstd");
    println!("cargo:rustc-link-lib=static=snappy");
    // TODO: link stdc++ statically
    build
}
