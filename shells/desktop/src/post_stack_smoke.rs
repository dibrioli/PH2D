//! **The scene ready for the app post-stack smoke** (`PH2D_POST_STACK_SMOKE=1`, ADR-0145).
//!
//! ## What it shows
//!
//! A frame-wide HDR colour grade applied to `game_rt` before the tonemap. This smoke
//! arms the **vignette alone** (no exposure/tint/contrast), so the effect is
//! unmistakable: the frame EDGES darken while the CENTRE is left untouched.
//!
//! It also overrides the editor clear to a **neutral middle-grey** ([`is_active`],
//! read by `render_loop`): on the usual dark `0.047` backdrop the edge-darkening is
//! invisible, and lifting it with exposure only brightens the CENTRE — which reads as
//! the *opposite* of a vignette (Enio: *"clareia o centro"*). A vignette IS darkened
//! edges, so it must be shown over something already LIT — but middle grey, not the
//! "almost white" a bright value became after the tonemap, changing nothing but the
//! corners.
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

/// Is this smoke armed? The render loop reads it to lift the editor clear to a
/// neutral middle-grey, so the vignette reads as darkened EDGES over an untouched
/// centre — never as a centre lift (which is what the dark backdrop turned it into).
pub(crate) fn is_active() -> bool {
    on()
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `motion_fx_smoke`. No-op sem a env.
    pub(crate) fn post_stack_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        // The VIGNETTE alone, over the middle-grey canvas this smoke installs. No
        // exposure — a lift on the dark backdrop was what made the vignette look like
        // "brightening the centre" (Enio). Here the centre is untouched and only the
        // corners fall off, which is what a vignette IS.
        self.grade = ph2d_render::GradeParams {
            exposure: 0.0,
            contrast: 1.0,
            saturation: 1.0,
            tint: [1.0, 1.0, 1.0],
            vignette: 0.85,        // a strong, clearly-visible edge darkening
            vignette_radius: 0.35, // the centre stays clean out to a third of the way
            vignette_softness: 0.5,
        };
        eprintln!(
            "[post-stack smoke] vinheta 0.85 sobre um campo cinza MEDIO: as BORDAS do \
             frame escurecem e o CENTRO fica intocado (sem brilho no centro, sem branco). \
             is_neutral={} (deve ser false).",
            self.grade.is_neutral()
        );
    }
}
