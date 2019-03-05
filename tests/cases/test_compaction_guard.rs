// Copyright 2019 PingCAP, Inc.
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

use rocksdb::{ColumnFamilyOptions, CompactionGuard, DBOptions, Writable, DB};
use std::sync::Arc;
use tempdir::TempDir;

struct SimpleGuards {
    guards: Vec<Vec<u8>>,
}

impl CompactionGuard for SimpleGuards {
    fn get_guards_in_range(&self, _: &[u8], _: &[u8]) -> Vec<Vec<u8>> {
        self.guards.clone()
    }
}

#[test]
fn test_compaction_guard() {
    let path = TempDir::new("_rust_rocksdb_compaction_guard_test").expect("");
    let mut cf_opts = ColumnFamilyOptions::new();
    let mut guards = vec![];
    guards.push(b"k5".to_vec());
    guards.push(b"k8".to_vec());
    cf_opts
        .set_compaction_guard(Arc::new(SimpleGuards { guards: guards }))
        .unwrap();
    let mut opts = DBOptions::new();
    opts.create_if_missing(true);
    let db = DB::open_cf(
        opts,
        path.path().to_str().unwrap(),
        vec![("default", cf_opts)],
    )
    .unwrap();
    let samples = vec![
        (b"k1".to_vec(), b"value1".to_vec()),
        (b"k2".to_vec(), b"value2".to_vec()),
        (b"k3".to_vec(), b"value3".to_vec()),
        (b"k4".to_vec(), b"value4".to_vec()),
        (b"k5".to_vec(), b"value5".to_vec()),
        (b"k6".to_vec(), b"value6".to_vec()),
        (b"k7".to_vec(), b"value7".to_vec()),
        (b"k8".to_vec(), b"value8".to_vec()),
        (b"k9".to_vec(), b"value9".to_vec()),
    ];
    // Generate one sst file in level 0.
    for &(ref k, ref v) in &samples {
        db.put(k, v).unwrap();
    }
    db.flush(true).unwrap();
    // Generate another sst file with the same content in level 0.
    for &(ref k, ref v) in &samples {
        db.put(k, v).unwrap();
    }
    db.flush(true).unwrap();

    // Trigger a manual compaction, will generate 2 sst files in level 1.
    // because of guard.
    db.compact_range(None, None);
    assert_eq!(
        db.get_property_int("rocksdb.num-files-at-level1").unwrap(),
        3
    );
}
