//! Centralized panic hook (skeleton — full impl per SKILL §12.5
//! requires ECS state hash + last frame ID + emergency save arena;
//! those land in M4 [ECS] and M13 [save] respectively).
//!
//! For M2: install a hook that prints a structured panic line so
//! shell logs (and CI runs) get a recognizable signature instead of
//! the default Rust panic format.
//!
//! Subsystems can register additional "panic context providers" via
//! [`PanicContext`] — strings appended to the panic message at the
//! moment of panic. Frame ID provider lands in M2 (here); state hash
//! in M4.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};

/// Process-global frame counter. Updated by the main loop each tick.
/// Lives in `ph2d-core` so any subsystem can read it without a
/// `MainLoop` handle.
static FRAME_ID: AtomicU64 = AtomicU64::new(0);

pub fn set_frame_id(frame: u64) {
    FRAME_ID.store(frame, Ordering::Relaxed);
}

pub fn current_frame_id() -> u64 {
    FRAME_ID.load(Ordering::Relaxed)
}

/// Marker that the panic hook was installed exactly once.
static INSTALLED: OnceLock<()> = OnceLock::new();

/// Install the global panic hook. Idempotent — second call is a no-op.
///
/// Format of the panic line:
/// ```text
/// PH2D PANIC frame=12345 thread="game" location="src/foo.rs:42" message="..."
/// ```
pub fn install_panic_hook() {
    INSTALLED.get_or_init(|| {
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            let frame = current_frame_id();
            let thread = std::thread::current()
                .name()
                .map(str::to_owned)
                .unwrap_or_else(|| String::from("<unnamed>"));
            let loc = info
                .location()
                .map(|l| format!("{}:{}", l.file(), l.line()))
                .unwrap_or_else(|| String::from("<unknown>"));
            let msg = info.payload();
            let msg_str = if let Some(s) = msg.downcast_ref::<&'static str>() {
                (*s).to_string()
            } else if let Some(s) = msg.downcast_ref::<String>() {
                s.clone()
            } else {
                String::from("<non-string panic payload>")
            };
            eprintln!(
                "PH2D PANIC frame={frame} thread={thread:?} location={loc:?} message={msg_str:?}"
            );
            // Delegate to the default hook so backtrace + hooks
            // installed by test harnesses still run.
            default_hook(info);
        }));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_id_round_trips() {
        set_frame_id(42);
        assert_eq!(current_frame_id(), 42);
        set_frame_id(0); // reset for other tests
    }

    #[test]
    fn install_is_idempotent() {
        install_panic_hook();
        install_panic_hook();
        install_panic_hook();
        // No assert — just must not panic itself.
    }
}
