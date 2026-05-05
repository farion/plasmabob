use std::sync::atomic::{AtomicU64, Ordering};

/// Provide a shared session id generator for editor UI windows/modals.
/// Exported here so multiple editor modules can obtain unique window ids.
pub fn next_window_session_id() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}
