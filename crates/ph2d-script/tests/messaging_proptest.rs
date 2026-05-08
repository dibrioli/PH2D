//! Property-based tests for `ph2d_script::messaging`.
//!
//! Properties enforced:
//! 1. `intern("name")` é determinístico (same string → same id, sequential).
//! 2. Send + dispatch_all preserva contagem (FIFO global, sem perda).
//! 3. Handlers para id sem registro são no-op (não panic).

use ph2d_script::{MessageBus, MessageId};
use proptest::prelude::*;
use std::cell::Cell;
use std::rc::Rc;

proptest! {
    #[test]
    fn intern_is_deterministic(names in proptest::collection::vec("[a-z_]{1,16}", 1..32)) {
        let mut bus = MessageBus::new();
        let ids: Vec<MessageId> = names.iter().map(|n| bus.intern(n)).collect();
        // Re-interning the same name returns the same id.
        for (i, n) in names.iter().enumerate() {
            prop_assert_eq!(bus.intern(n), ids[i]);
        }
        // Distinct names get distinct ids (where strings differ).
        for (i, ni) in names.iter().enumerate() {
            for (j, nj) in names.iter().enumerate() {
                if ni != nj {
                    prop_assert_ne!(ids[i], ids[j]);
                }
            }
        }
    }

    #[test]
    fn dispatch_processes_all_sent(n in 0usize..2_000) {
        let mut bus = MessageBus::with_capacity(n.max(1), 4);
        let id = bus.intern("ping");
        let count = Rc::new(Cell::new(0u64));
        let c = count.clone();
        bus.on(id, Box::new(move |_| { c.set(c.get() + 1); }));

        for i in 0..n {
            bus.send(0, 1, id, i as u64);
        }
        prop_assert_eq!(bus.pending(), n);
        let processed = bus.dispatch_all();
        prop_assert_eq!(processed, n);
        prop_assert_eq!(count.get() as usize, n);
        prop_assert_eq!(bus.pending(), 0);
    }

    #[test]
    fn unregistered_message_id_is_noop(payloads in proptest::collection::vec(any::<u64>(), 0..100)) {
        let mut bus = MessageBus::new();
        let known = bus.intern("known");
        // Synthesize an unregistered id beyond what we've interned.
        let unknown = MessageId(999_999);
        for (i, p) in payloads.iter().enumerate() {
            // Alternate between known (no handler registered either, so noop)
            // and unknown id; both must be safely processed.
            let id = if i % 2 == 0 { known } else { unknown };
            bus.send(0, 1, id, *p);
        }
        let processed = bus.dispatch_all();
        prop_assert_eq!(processed, payloads.len());
    }
}
