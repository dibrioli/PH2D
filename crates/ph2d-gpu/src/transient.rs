//! TransientPool — cache of per-frame transient textures (depth,
//! MSAA, intermediate render targets).
//!
//! M3 status: skeleton. `clear()` is the only operational method —
//! invoked by [`crate::SurfaceContext`] on `Lost` recovery and on
//! `Background` lifecycle (per ADR-0020). Texture allocation +
//! recycling lands in M5 when the render graph needs intermediate
//! targets.

#[derive(Default)]
pub struct TransientPool {
    // M5 will add: HashMap<TransientKey, wgpu::Texture>, watermark
    // counter, GC policy. For now, the only state is "exists vs
    // cleared" — tracked via the `cleared_count` for unit tests.
    cleared_count: u32,
}

impl TransientPool {
    pub fn new() -> Self {
        Self::default()
    }

    /// Drop all cached transients. Called on:
    /// - `SurfaceError::Lost` recovery
    /// - `Lifecycle::Background` (proactive VRAM release)
    ///
    /// Both per ADR-0020.
    pub fn clear(&mut self) {
        self.cleared_count = self.cleared_count.saturating_add(1);
    }

    /// Number of `clear()` calls so far (test/debug only).
    pub fn cleared_count(&self) -> u32 {
        self.cleared_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool_starts_with_zero_cleared() {
        let p = TransientPool::new();
        assert_eq!(p.cleared_count(), 0);
    }

    #[test]
    fn clear_increments_counter() {
        let mut p = TransientPool::new();
        p.clear();
        p.clear();
        p.clear();
        assert_eq!(p.cleared_count(), 3);
    }

    #[test]
    fn clear_saturates_at_u32_max() {
        let mut p = TransientPool {
            cleared_count: u32::MAX,
        };
        p.clear();
        assert_eq!(p.cleared_count(), u32::MAX);
    }
}
