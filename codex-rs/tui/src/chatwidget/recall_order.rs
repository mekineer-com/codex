//! Ordering follows submission, not which queue currently owns a message.

use std::sync::atomic::{AtomicU64, Ordering};

pub(super) fn next_input_order() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RecallTarget {
    Queued(usize),
    Rejected(usize),
    Pending(usize),
    PreparingImages,
}
