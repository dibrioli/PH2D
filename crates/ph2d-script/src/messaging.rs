//! C2.1 — Defold-style messaging bus.
//!
//! Design: hash-interned message names (string → u32 id O(1) lookup),
//! global FIFO queue (same sender→same target preserves order by
//! construction), handlers registered per message id.
//!
//! Throughput target (HR-4): 100_000 messages/frame in ≤ 1.5 ms.
//!
//! Thread-safety: NO. Single-threaded by design (per HR-4 audio is the
//! only subsystem on a separate thread). MessageBus lives on the main
//! thread inside `ScriptRuntime` or next to it.
//!
//! Out of scope in this skeleton:
//! - Arbitrary payload via postcard (current: u64 inline for the bench).
//! - Schema enforcement in dev (Cargo feature `mcp-schema`).
//! - Cross-runtime/cross-thread bridging.

use std::collections::HashMap;

/// Hash-interned identifier for a message name. Construct via
/// `MessageBus::intern`. Always sequential starting at 0; used as index
/// into a Vec of handler slots for O(1) cache-friendly dispatch.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MessageId(pub u32);

/// Opaque entity handle (matches the convention in HR-8).
pub type EntityId = u64;

#[derive(Copy, Clone, Debug)]
pub struct Message {
    pub sender: EntityId,
    pub target: EntityId,
    pub id: MessageId,
    /// Inline payload for the C2.1 bench. Real impl will use a postcard
    /// buffer in a per-frame arena (bumpalo) — see HR-3.
    pub payload: u64,
}

pub type Handler = Box<dyn FnMut(&Message)>;

/// Defold-style message bus.
/// - Hash-interned ids (string → sequential u32 once, then O(1) array lookup).
/// - FIFO global queue (mesma ordem para qualquer par sender→target).
/// - Handlers stored in a Vec indexed by MessageId, not a HashMap, to keep
///   dispatch hot path branch-light and cache-friendly.
pub struct MessageBus {
    queue: Vec<Message>,
    interned: HashMap<String, MessageId>,
    /// One entry per intern id. Empty Vec means no handler registered.
    handlers: Vec<Vec<Handler>>,
}

impl MessageBus {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
            interned: HashMap::new(),
            handlers: Vec::new(),
        }
    }

    /// Pre-size queue + interning maps to avoid runtime growth in hot path.
    pub fn with_capacity(messages: usize, distinct_names: usize) -> Self {
        Self {
            queue: Vec::with_capacity(messages),
            interned: HashMap::with_capacity(distinct_names),
            handlers: Vec::with_capacity(distinct_names),
        }
    }

    /// O(1) amortized; allocates only on first sight of a name.
    pub fn intern(&mut self, name: &str) -> MessageId {
        if let Some(id) = self.interned.get(name) {
            return *id;
        }
        let id = MessageId(self.handlers.len() as u32);
        self.interned.insert(name.to_owned(), id);
        self.handlers.push(Vec::new());
        id
    }

    /// O(1). Send is just a push; dispatching happens on `dispatch_all`.
    #[inline]
    pub fn send(&mut self, sender: EntityId, target: EntityId, id: MessageId, payload: u64) {
        self.queue.push(Message {
            sender,
            target,
            id,
            payload,
        });
    }

    /// Register a handler for a given message id. Multiple handlers per id
    /// are supported; called in registration order.
    pub fn on(&mut self, id: MessageId, handler: Handler) {
        self.handlers[id.0 as usize].push(handler);
    }

    /// Drain the queue, calling every handler for each message in FIFO order.
    /// Returns the number of messages processed.
    pub fn dispatch_all(&mut self) -> usize {
        let processed = self.queue.len();
        // Drain in FIFO order — slice iter then clear() is faster than
        // pop_front loop. handlers[id] is array indexing, no hashing.
        for msg in self.queue.iter() {
            let id = msg.id.0 as usize;
            if id < self.handlers.len() {
                for h in self.handlers[id].iter_mut() {
                    h(msg);
                }
            }
        }
        self.queue.clear();
        processed
    }

    pub fn pending(&self) -> usize {
        self.queue.len()
    }
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}
