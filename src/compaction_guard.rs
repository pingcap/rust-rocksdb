use crocksdb_ffi::{self, DBCompactionGuard};
use libc::c_void;
use std::sync::Arc;
use std::{mem, slice};

/// `CompactionGuard` allows an application to provide guards for compaction.
pub trait CompactionGuard {
    fn get_guards_in_range(&self, start: &[u8], end: &[u8]) -> Vec<Vec<u8>>;
}

#[repr(C)]
pub struct CompactionGuardProxy {
    guard: Arc<CompactionGuard>,
}

extern "C" fn destructor(guard: *mut c_void) {
    unsafe {
        Box::from_raw(guard as *mut CompactionGuardProxy);
    }
}

extern "C" fn clean_guard(m: *mut c_void) {
    unsafe {
        libc::free(m);
    }
}

extern "C" fn get_guards_in_range(
    guard: *mut c_void,
    start: *const u8,
    start_len: u32,
    end: *const u8,
    end_len: u32,
    total: *mut u32,
    lens: *mut *mut u32,
) -> *mut *mut u8 {
    unsafe {
        let guard = &mut *(guard as *mut CompactionGuardProxy);
        let start = slice::from_raw_parts(start, start_len as usize);
        let end = slice::from_raw_parts(end, end_len as usize);
        let mut guards = guard.guard.get_guards_in_range(start, end);

        *total = guards.len() as u32;
        if *total > 0 {
            let mut res = libc::malloc(*total as usize * mem::size_of::<*mut u8>()) as *mut *mut u8;
            let mut l = libc::malloc(*total as usize * mem::size_of::<u32>()) as *mut u32;
            *lens = l;
            for key in guards.drain(..) {
                assert!(!key.is_empty());
                let cloned = libc::malloc(key.len());
                libc::memcpy(cloned, key.as_ptr() as *const c_void, key.len());
                *res = cloned as *mut u8;
                res = res.add(1);
                *l = key.len() as u32;
                l = l.add(1);
            }
            res.sub(*total as usize)
        } else {
            0 as *mut *mut u8
        }
    }
}

pub struct CompactionGuardHandle {
    pub inner: *mut DBCompactionGuard,
}

impl Drop for CompactionGuardHandle {
    fn drop(&mut self) {
        //        unsafe {
        //            crocksdb_ffi::crocksdb_compactionguard_destory(self.inner);
        //        }
    }
}

pub unsafe fn new_compaction_gurad(
    g: Arc<CompactionGuard>,
) -> Result<CompactionGuardHandle, String> {
    let proxy = Box::into_raw(Box::new(CompactionGuardProxy { guard: g }));
    let res = crocksdb_ffi::crocksdb_compactionguard_create(
        proxy as *mut c_void,
        destructor,
        clean_guard,
        get_guards_in_range,
    );
    Ok(CompactionGuardHandle { inner: res })
}
