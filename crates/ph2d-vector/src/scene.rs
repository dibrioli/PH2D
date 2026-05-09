//! [`VectorScene`] — wrapper over `vello::Scene`.
//!
//! Owns the encoded scene graph for one frame. Reset between frames
//! via [`VectorScene::reset`]. Inner `vello::Scene` exposed via
//! [`VectorScene::inner`] / [`VectorScene::inner_mut`] for callers
//! that need the full Vello API; the convenience helpers below
//! cover the 80 % case (rect / path fill).

use vello::Scene;
use vello::kurbo::{Affine, BezPath, Rect};
use vello::peniko::{Brush, Color, Fill};

pub struct VectorScene {
    inner: Scene,
}

impl VectorScene {
    pub fn new() -> Self {
        Self {
            inner: Scene::new(),
        }
    }

    /// Drop every encoded path. Call once at frame start before
    /// re-emitting draw commands; Vello's internal allocations are
    /// reused across frames so this is cheap.
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    pub fn inner(&self) -> &Scene {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut Scene {
        &mut self.inner
    }

    /// Fill an axis-aligned rect with a solid color. The most common
    /// "draw a panel background" case.
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.inner.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(color),
            None,
            &rect,
        );
    }

    /// Fill an arbitrary path with a brush.
    pub fn fill_path(&mut self, path: &BezPath, brush: &Brush, transform: Affine) {
        self.inner.fill(Fill::NonZero, transform, brush, None, path);
    }
}

impl Default for VectorScene {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_scene_is_empty() {
        // Vello doesn't expose a "command count" public API; we
        // settle for "construction succeeds + reset is idempotent
        // on an empty scene".
        let mut s = VectorScene::new();
        s.reset();
        s.reset();
        // Inner accessor returns the same Scene reference twice —
        // proves the wrapper isn't accidentally cloning.
        let p1 = s.inner() as *const Scene;
        let p2 = s.inner() as *const Scene;
        assert!(std::ptr::eq(p1, p2));
    }

    #[test]
    fn fill_rect_does_not_panic() {
        // Smoke test the helper. We can't easily inspect the encoded
        // commands without GPU, but if Vello's encoder panics on a
        // malformed input this would surface here.
        let mut s = VectorScene::new();
        s.fill_rect(Rect::new(0.0, 0.0, 100.0, 50.0), Color::WHITE);
        s.fill_rect(Rect::new(10.0, 10.0, 20.0, 20.0), Color::BLACK);
    }

    #[test]
    fn fill_path_with_brush() {
        let mut s = VectorScene::new();
        let mut path = BezPath::new();
        path.move_to((0.0, 0.0));
        path.line_to((10.0, 0.0));
        path.line_to((10.0, 10.0));
        path.line_to((0.0, 10.0));
        path.close_path();
        s.fill_path(&path, &Brush::Solid(Color::WHITE), Affine::IDENTITY);
    }

    #[test]
    fn reset_after_fills_is_idempotent() {
        let mut s = VectorScene::new();
        s.fill_rect(Rect::new(0.0, 0.0, 1.0, 1.0), Color::WHITE);
        s.fill_rect(Rect::new(0.0, 0.0, 1.0, 1.0), Color::BLACK);
        s.reset();
        // After reset, drawing again must not panic.
        s.fill_rect(Rect::new(2.0, 2.0, 3.0, 3.0), Color::WHITE);
    }
}
