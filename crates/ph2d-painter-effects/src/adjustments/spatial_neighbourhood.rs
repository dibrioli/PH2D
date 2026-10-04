//! ⭐⭐⭐⭐ **THE NEIGHBOURHOOD HOOK** — «which neighbours does a sample have» is ONE
//! door (`docs/3D/30` §2 and §14, W6).
//!
//! The neighbourhood kinds are written ONCE over a [`Neighbourhood`]: the premultiplied
//! boundary, the unsharp combine, Bloom's bright pass and glow add, the local-tone
//! correction of Shadows/Highlights. The only thing a neighbourhood owns is the LOW-PASS —
//! *a Gaussian of σ = [`gaussian_sigma`]`(radius)` over the neighbours*. Two exist:
//!
//! - the image GRID ([`AdjustWindow`]): separable, clamp-to-edge, radius in px — the 2D
//!   of today, AO BIT (its impl calls the very kernels the 2D always ran);
//! - a SURFACE: the sculpt piece's reticle (`ph2d_mesh_colors::difusao`, implemented in
//!   `ph2d-app-sculpt3d`), the heat kernel with the radius in piece units. A grid low-pass
//!   there would blur along the sample ORDER, not the surface.
//!
//! ⛔ Motion, Chroma and Halftone read the image PLANE (a direction, a centre, a screen of
//! dots) — [`AdjustmentKind::reads_the_image_plane`] —, which a surface does not have: they
//! exist on the grid only, and a surface never receives them (the piece refuses them at its
//! door and the panel greys them out with the reason).

use super::*;

/// σ of the Gaussian a blur of `radius` is — the 3σ truncation: the kernel reaches
/// `ceil(radius)` texels on the grid ([`gaussian_weights`]). ONE mapping for the grid,
/// the surface and the GPU.
#[must_use]
pub fn gaussian_sigma(radius: f32) -> f32 {
    radius.max(0.0) / 3.0
}

/// ⭐⭐⭐ **Who a sample's neighbours are** — the low-pass a neighbourhood kind runs on.
pub trait Neighbourhood: Sync {
    /// The layout of the buffer (`width × height`, row-major) — what the coordinate kinds
    /// (`Noise`) hash, and the length every buffer handed to this neighbourhood has.
    fn window(&self) -> AdjustWindow;
    /// The Gaussian low-pass of σ = [`gaussian_sigma`]`(radius)`, in place, on a
    /// PREMULTIPLIED RGBA buffer. `radius ≤ 0` is the identity.
    fn blur4(&self, radius: f32, buf: &mut [[f32; 4]]);
    /// The same low-pass of a scalar field.
    fn blur1(&self, radius: f32, buf: &mut [f32]);
    /// Bloom's glow low-pass, full resolution out (premultiplied light). The grid
    /// overrides it with its downsampled pyramid (the GPU's mirror).
    fn glow(&self, radius: f32, mut bright: Vec<[f32; 4]>) -> Vec<[f32; 4]> {
        self.blur4(radius, &mut bright);
        bright
    }
    /// Is the buffer the image PLANE (a direction, a centre, a screen mean something)?
    fn image_plane(&self) -> bool;
}

/// ⭐ **The image grid** — the 2D of today: the separable kernels, clamp-to-edge.
impl Neighbourhood for AdjustWindow {
    fn window(&self) -> AdjustWindow {
        *self
    }
    fn blur4(&self, radius: f32, buf: &mut [[f32; 4]]) {
        separable_blur_premul(radius, buf, *self);
    }
    fn blur1(&self, radius: f32, buf: &mut [f32]) {
        tonal::separable_blur_scalar(radius, buf, *self);
    }
    fn glow(&self, radius: f32, bright: Vec<[f32; 4]>) -> Vec<[f32; 4]> {
        tonal::grid_glow(radius, bright, *self)
    }
    fn image_plane(&self) -> bool {
        true
    }
}

/// ⭐⭐⭐ **Apply any adjustment over a neighbourhood** — the one dispatch. Per-pixel kinds
/// delegate to [`apply_adjustment`](super::apply_adjustment) verbatim; the neighbourhood
/// kinds run their law over `nb`; the image-plane kinds need the grid.
pub fn apply_adjustment_on(
    kind: &AdjustmentKind,
    params: &AdjustmentParams,
    acc: &mut [[f32; 4]],
    nb: &dyn Neighbourhood,
) {
    let win = nb.window();
    debug_assert_eq!(
        params.kind(),
        *kind,
        "apply_adjustment_on: kind/params variant mismatch"
    );
    debug_assert_eq!(
        acc.len(),
        (win.width as usize) * (win.height as usize),
        "apply_adjustment_on: acc length must equal the neighbourhood's window"
    );
    if kind.reads_the_image_plane() && !nb.image_plane() {
        debug_assert!(
            false,
            "{kind:?} reads the image plane: a surface never receives it"
        );
        return;
    }
    match (kind, params) {
        (AdjustmentKind::GaussianBlur, AdjustmentParams::GaussianBlur(p)) => {
            apply_gaussian_on(p, acc, nb)
        }
        (AdjustmentKind::Sharpen, AdjustmentParams::Sharpen(p)) => apply_sharpen_on(p, acc, nb),
        (AdjustmentKind::MotionBlur, AdjustmentParams::MotionBlur(p)) => {
            apply_motion_blur(p, acc, win)
        }
        (AdjustmentKind::ChromaticAberration, AdjustmentParams::ChromaticAberration(p)) => {
            apply_chromatic_aberration(p, acc, win)
        }
        (AdjustmentKind::Noise, AdjustmentParams::Noise(p)) => apply_noise(p, acc, win),
        (AdjustmentKind::Halftone, AdjustmentParams::Halftone(p)) => apply_halftone(p, acc, win),
        (AdjustmentKind::Bloom, AdjustmentParams::Bloom(p)) => tonal::apply_bloom_on(p, acc, nb),
        (AdjustmentKind::ShadowsHighlights, AdjustmentParams::ShadowsHighlights(p)) => {
            tonal::apply_shadows_highlights_on(p, acc, nb)
        }
        // Every other kind is per-pixel and position-independent.
        _ => super::compute::apply_adjustment(kind, params, acc),
    }
}

/// The Gaussian blur over `nb`, in premultiplied display tones (P4) — colour AND coverage
/// feather together. `radius ≤ 0` is the identity (no premultiply round trip).
pub fn apply_gaussian_on(p: &GaussianBlurParams, acc: &mut [[f32; 4]], nb: &dyn Neighbourhood) {
    if p.radius <= 0.0 {
        return;
    }
    premultiply(acc);
    nb.blur4(p.radius, acc);
    unpremultiply(acc);
}

/// Unsharp-mask sharpen over `nb`: `out = base + amount·(base − blur(base))` across ALL 4
/// premultiplied channels (the alpha edge sharpens with the colour). Negatives clamp at 0.
pub fn apply_sharpen_on(p: &SharpenParams, acc: &mut [[f32; 4]], nb: &dyn Neighbourhood) {
    if p.amount == 0.0 || p.radius <= 0.0 {
        return;
    }
    premultiply(acc);
    let base = acc.to_vec(); // premultiplied base
    nb.blur4(p.radius, acc); // acc = blur(base), premultiplied
    for (out, b) in acc.iter_mut().zip(base.iter()) {
        for c in 0..4 {
            out[c] = (b[c] + p.amount * (b[c] - out[c])).max(0.0);
        }
        out[3] = out[3].min(1.0); // coverage stays in [0, 1] after the overshoot
    }
    unpremultiply(acc);
}
