//! **The scene ready for the app post-stack smoke** (`PH2D_POST_STACK_SMOKE=1`, ADR-0145).
//!
//! ## What it shows
//!
//! A frame-wide HDR colour grade applied to `game_rt` before the tonemap: a **strong
//! vignette** darkening the frame edges, plus a lift in exposure, a warm tint and a
//! touch of contrast/saturation — the cinematic "look" of a post stack. The default
//! editor background is a dark neutral grey (`0.047`) filling the whole frame, so the
//! vignette has something to darken edge-to-edge; the exposure lift brings it up so the
//! centre-vs-corner falloff reads clearly.
//!
//! ```text
//!   game_rt (whole scene) ──[PostStack grade]──▶ game_rt ──▶ AgX tonemap
//! ```
//!
//! This is the **Option A** of doc 66 the glow (`fx.glow`) is NOT: a vignette is a
//! frame-anchored, subtractive operation — "only the Motion layer" cannot express it, so
//! it lives in the app post-stack, not in the Motion graph (doc 67 §6).
//!
//! ## Fatia 1 — no UI yet
//!
//! This smoke arms `App.grade` **in code** because the UI is fatia 2 (`ProjectSettings`
//! plus a Settings panel). The pass, the vignette maths and the byte-identical-when-neutral
//! skip are what fatia 1 proves. Set the grade back to `GradeParams::default()` (or don't
//! run the smoke) and the post-stack block is skipped → the frame is byte-identical.

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_POST_STACK_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `motion_fx_smoke`. No-op sem a env.
    pub(crate) fn post_stack_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        // A demonstrative cinematic grade: bright enough that the vignette's
        // centre-vs-corner falloff is unmistakable over the dark editor background.
        self.grade = ph2d_render::GradeParams {
            exposure: 2.0, // +2 stops: lift the 0.047 grey so the vignette reads
            contrast: 1.15,
            saturation: 1.1,
            tint: [1.05, 1.0, 0.9], // a warm cast
            vignette: 0.85,         // a strong, clearly-visible vignette
            vignette_radius: 0.25,  // darkening starts a quarter of the way out
            vignette_softness: 0.55,
        };
        eprintln!(
            "[post-stack smoke] grade armada: vinheta 0.85 + exposição +2 stops + tint quente + \
             contraste/saturação. As BORDAS do frame escurecem (o centro fica claro). \
             is_neutral={} (deve ser false).",
            self.grade.is_neutral()
        );
    }
}
