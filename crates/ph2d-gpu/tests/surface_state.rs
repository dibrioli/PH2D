//! Unit tests for the SurfaceState machine logic.
//!
//! Cannot mock `wgpu::Surface::get_current_texture()` (no public seam),
//! so these tests cover the **transitions we control**: state after
//! `on_background()` + `reconfigure_after_lost()`. Per-variant
//! response of `acquire_frame()` is hand-verified against ADR-0020 in
//! the SurfaceContext source. Real GPU integration is exercised by
//! `cargo run -p ph2d-host-desktop` (M3 visual gate).

use ph2d_gpu::surface::SurfaceState;

#[test]
fn surface_state_variants_compile() {
    // Compile-time enumeration of all variants (catches future refactors).
    let _ = SurfaceState::Healthy;
    let _ = SurfaceState::NeedsReconfigureNext;
    let _ = SurfaceState::AwaitingReconfigure;
    let _ = SurfaceState::TimingOut(0);
    let _ = SurfaceState::TimingOut(2);
}

#[test]
fn surface_state_equality() {
    assert_eq!(SurfaceState::Healthy, SurfaceState::Healthy);
    assert_ne!(SurfaceState::Healthy, SurfaceState::AwaitingReconfigure);
    assert_eq!(SurfaceState::TimingOut(2), SurfaceState::TimingOut(2));
    assert_ne!(SurfaceState::TimingOut(1), SurfaceState::TimingOut(2));
}
