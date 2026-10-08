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

pub(super) fn newest_input(
    candidates: impl IntoIterator<Item = (RecallTarget, u64)>,
) -> Option<RecallTarget> {
    candidates
        .into_iter()
        .max_by_key(|(_, order)| *order)
        .map(|(target, _)| target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newest_submission_wins_regardless_of_queue_or_position() {
        use RecallTarget::*;
        assert_eq!(newest_input([]), None);
        assert_eq!(
            newest_input([(Queued(0), 1), (Pending(0), 2)]),
            Some(Pending(0))
        );
        assert_eq!(
            newest_input([(Queued(0), 3), (Pending(0), 2)]),
            Some(Queued(0))
        );
        assert_eq!(
            newest_input([(Queued(0), 5), (Queued(1), 3), (Rejected(0), 4)]),
            Some(Queued(0))
        );
        assert_eq!(
            newest_input([(Pending(0), 2), (Rejected(0), 4)]),
            Some(Rejected(0))
        );
        assert!(next_input_order() < next_input_order());
        assert_eq!(
            newest_input([(Queued(0), 3), (PreparingImages, 2)]),
            Some(Queued(0))
        );
    }
}
