//! [`ToastQueue`] — non-modal notification stream (ADR-0023 §2).
//!
//! "Notificações flutuantes não-modais no topo do canvas
//! (ex.: 'Undo: Brush Stroke', 'Saved 2 s ago') — informam sem
//! interromper."
//!
//! - **Stack** at the top-center of the canvas (above all panels).
//! - **Auto-dismiss** after `Toast::ttl_frames` (default ~3 s @ 60 Hz).
//! - **Live region** in the a11y tree (per ADR-0023 §10) so screen
//!   readers announce the latest toast without stealing focus.
//! - **Bounded queue** (32 entries) — runaway notifications get
//!   dropped silently. Per HR-9 backpressure principle (same as
//!   `ph2d_script::WriteQueue`).

use ph2d_a11y::{Live, Node, NodeBuilder, Role};
use std::collections::VecDeque;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ToastSeverity {
    Info,
    Success,
    Warning,
    ErrorState,
}

impl ToastSeverity {
    /// Mapping to the `accesskit::Live` priority. Errors interrupt
    /// (`Assertive`); everything else is polite.
    pub fn live_priority(self) -> Live {
        match self {
            Self::ErrorState => Live::Assertive,
            _ => Live::Polite,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub message: String,
    pub severity: ToastSeverity,
    pub ttl_frames: u32,
    /// Frames since the toast was pushed. Caller calls `tick` once
    /// per frame; toast removes itself when `age >= ttl_frames`.
    pub age: u32,
}

impl Toast {
    pub const DEFAULT_TTL_FRAMES: u32 = 180; // 3 s @ 60 Hz

    pub fn new(message: impl Into<String>, severity: ToastSeverity) -> Self {
        Self {
            message: message.into(),
            severity,
            ttl_frames: Self::DEFAULT_TTL_FRAMES,
            age: 0,
        }
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, ToastSeverity::Info)
    }
    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, ToastSeverity::Success)
    }
    pub fn warning(message: impl Into<String>) -> Self {
        Self::new(message, ToastSeverity::Warning)
    }
    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, ToastSeverity::ErrorState)
    }

    pub fn ttl_frames(mut self, n: u32) -> Self {
        self.ttl_frames = n;
        self
    }

    /// Build the AccessKit live-region node for this toast. Screen
    /// readers will announce the message according to severity.
    pub fn build_a11y(&self) -> Node {
        NodeBuilder::new(Role::GenericContainer)
            .label(&self.message)
            .live(self.severity.live_priority())
            .build()
    }
}

#[derive(Default)]
pub struct ToastQueue {
    inner: VecDeque<Toast>,
    cap: usize,
}

impl ToastQueue {
    pub const DEFAULT_CAP: usize = 32;

    pub fn new() -> Self {
        Self::with_cap(Self::DEFAULT_CAP)
    }

    pub fn with_cap(cap: usize) -> Self {
        Self {
            inner: VecDeque::new(),
            cap,
        }
    }

    /// Push a toast. Returns false if the queue is full (silent drop).
    pub fn push(&mut self, toast: Toast) -> bool {
        if self.inner.len() >= self.cap {
            return false;
        }
        self.inner.push_back(toast);
        true
    }

    /// Tick every toast by 1 frame; drop expired ones.
    pub fn tick(&mut self) {
        for t in &mut self.inner {
            t.age = t.age.saturating_add(1);
        }
        while self.inner.front().is_some_and(|t| t.age >= t.ttl_frames) {
            self.inner.pop_front();
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Toast> {
        self.inner.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_iterate() {
        let mut q = ToastQueue::new();
        q.push(Toast::info("Saved"));
        q.push(Toast::warning("Slow load"));
        assert_eq!(q.len(), 2);
        let labels: Vec<&str> = q.iter().map(|t| t.message.as_str()).collect();
        assert_eq!(labels, vec!["Saved", "Slow load"]);
    }

    #[test]
    fn full_queue_drops_silently() {
        let mut q = ToastQueue::with_cap(2);
        assert!(q.push(Toast::info("a")));
        assert!(q.push(Toast::info("b")));
        assert!(!q.push(Toast::info("c"))); // dropped
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn tick_expires_old_toasts() {
        let mut q = ToastQueue::new();
        q.push(Toast::info("ephemeral").ttl_frames(2));
        for _ in 0..3 {
            q.tick();
        }
        assert!(q.is_empty());
    }

    #[test]
    fn tick_keeps_fresh_toasts() {
        let mut q = ToastQueue::new();
        q.push(Toast::info("fresh").ttl_frames(60));
        for _ in 0..30 {
            q.tick();
        }
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn error_severity_is_assertive() {
        assert_eq!(ToastSeverity::ErrorState.live_priority(), Live::Assertive);
        assert_eq!(ToastSeverity::Info.live_priority(), Live::Polite);
        assert_eq!(ToastSeverity::Warning.live_priority(), Live::Polite);
        assert_eq!(ToastSeverity::Success.live_priority(), Live::Polite);
    }

    #[test]
    fn convenience_constructors() {
        assert_eq!(Toast::info("x").severity, ToastSeverity::Info);
        assert_eq!(Toast::success("x").severity, ToastSeverity::Success);
        assert_eq!(Toast::warning("x").severity, ToastSeverity::Warning);
        assert_eq!(Toast::error("x").severity, ToastSeverity::ErrorState);
    }

    #[test]
    fn a11y_node_uses_live_region() {
        let t = Toast::error("Save failed");
        let n = t.build_a11y();
        assert_eq!(n.role(), Role::GenericContainer);
        assert_eq!(n.label(), Some("Save failed"));
        assert_eq!(n.live(), Some(Live::Assertive));
    }
}
