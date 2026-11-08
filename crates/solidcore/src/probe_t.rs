//! kernel-f probe timers (not for upstream).
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;
static T: Mutex<Option<HashMap<&'static str, (u64, u128)>>> = Mutex::new(None);
pub struct G(&'static str, Instant);
impl Drop for G {
    fn drop(&mut self) {
        let ns = self.1.elapsed().as_nanos();
        let mut t = T.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let e = t.get_or_insert_with(HashMap::new).entry(self.0).or_insert((0, 0));
        e.0 += 1;
        e.1 += ns;
    }
}
pub fn g(name: &'static str) -> G { G(name, Instant::now()) }
pub fn take() -> Vec<(&'static str, u64, u128)> {
    let mut t = T.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut v: Vec<_> = t.take().unwrap_or_default().into_iter().map(|(k, (c, n))| (k, c, n)).collect();
    v.sort_by_key(|x| std::cmp::Reverse(x.2));
    v
}
