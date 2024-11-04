use std::sync::{Arc, LockResult, Mutex, MutexGuard, PoisonError};
use lazy_static::lazy_static;

lazy_static!(
    static ref C:Arc<Mutex<Cache>> = Arc::new(Mutex::new(Cache{}));
);
fn main()->anyhow::Result<()> {
    // C.lock()?.get("abc"); // 编译不过，但下面的可以编译过
    C.lock().unwrap().get("abc");
    Ok(())
}

struct Cache {}

impl Cache{
    pub fn get(&self, s:&str)->String{
        s.to_string()
    }
}