use crocksdb_ffi::{self};

/// Options while opening a file in the cloud to read/write
pub struct CloudEnvOptions {
    pub inner: *mut crocksdb_ffi::CloudEnvOptions,
}

impl CloudEnvOptions {
    pub fn new() -> CloudEnvOptions {
        unsafe {
            CloudEnvOptions {
                inner: crocksdb_ffi::cloud_envoptions_create(),
            }
        }
    }
}

impl Drop for CloudEnvOptions {
    fn drop(&mut self) {
        unsafe {
            crocksdb_ffi::cloud_envoptions_destroy(self.inner);
        }
    }
}