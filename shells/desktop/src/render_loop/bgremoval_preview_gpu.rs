//! A metade de GPU e de overlay da ponte da Remoção de fundo — o upload da prévia e as dicas por
//! cima dela. Filho por ASSUNTO (por `#[path]`) do [`super`], que guarda o downcast (ADR-0040 §3)
//! e chama estas duas funções no sítio exacto onde os blocos estavam.

use crate::app_state::{BgremovalPreview, BgremovalPreviewGpu};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::toast::{Toast, ToastQueue};
use ph2d_host::WindowSize;
use ph2d_i18n::tr_with;
use ph2d_render::{Camera2d, Sprite, SpriteRenderer};
// ⭐ O afim saiu para uma FOLHA porque quatro assuntos o partilhavam (HOWTO §1.2).
use ph2d_sprite_screen::sprite_image_to_screen_affine;
use ph2d_tokens::{ColorToken, StrokeToken, Theme};
use ph2d_vector::{Affine, Brush, Circle, Color, ImageQuality, Stroke, VectorScene};
use std::sync::Arc;

/// O ciclo de vida da textura de GPU da prévia (Lens F): sobe os pixels premultiplicados quando o
/// `Arc` da cache muda, e liberta a ranhura quando a prévia some.
pub(super) fn upload_preview(
    bgremoval_preview: &Option<BgremovalPreview>,
    bgremoval_preview_gpu: &mut Option<BgremovalPreviewGpu>,
    renderer: &mut SpriteRenderer,
    toasts: &mut ToastQueue,
) {
    // ── GPU lifecycle for the live-preview texture (Lens F, 2026-05-26) ──
    // Replaces the old Vello image draw of the preview RGBA. Owns a
    // transient `IndividualTextureStore` slot; uploads the
    // premultiplied preview pixels into it whenever the CPU-side
    // cache gets a fresh `Arc` (or the selection drifts to a
    // different sprite, or the size changes). NEXT frame's
    // `sim_extract` reads `bgremoval_preview_gpu` directly to emit a
    // `PreviewOverride` — the sprite pipeline samples from THIS
    // texture instead of the source sprite's original binding, so
    // the live preview goes through the SAME wgpu sprite shader as
    // Apply (`Rgba8UnormSrgb` + premul blend) → byte-for-byte parity,
    // no Vello-internal gamma/blend divergence.
    //
    // Premultiplication: byte-space `premultiply_rgba8` mirrors
    // EXACTLY what the Apply path does in
    // `SpriteImage::into_premultiplied`. Both produce identical bytes
    // that, when uploaded to `Rgba8UnormSrgb`, the GPU decodes to
    // identical linear values for the sprite shader's bilinear
    // sample. The gamma-correct variant from earlier (Fix C) is
    // intentionally NOT used here — its job was to compensate for
    // Vello's `Rgba8Unorm` raw-byte interpretation, which no longer
    // applies once the preview leaves the Vello path entirely.
    //
    // The 1-frame lag introduced by reading `bgremoval_preview_gpu`
    // on the NEXT frame's extract (this dispatch runs after the
    // current frame's `sim_extract`) is imperceptible: the live
    // preview is a continuous animation and a single ~16ms delay
    // between parameter change and visible response is below the
    // human flicker threshold.
    match bgremoval_preview.as_ref() {
        Some(preview) => {
            let cache_token = Arc::as_ptr(&preview.rgba) as usize;
            let needs_upload = match *bgremoval_preview_gpu {
                None => true,
                Some(gpu) => {
                    gpu.arc_token != cache_token
                        || gpu.entity_bits != preview.entity_bits
                        || gpu.width != preview.width
                        || gpu.height != preview.height
                }
            };
            if needs_upload {
                let mut premul_bytes = (*preview.rgba).clone();
                ph2d_render::premultiply_rgba8(&mut premul_bytes);
                let upload_result: Result<u32, _> = match *bgremoval_preview_gpu {
                    Some(gpu) => renderer
                        .replace_individual_pixels(
                            gpu.texture_id,
                            preview.width,
                            preview.height,
                            &premul_bytes,
                        )
                        .map(|()| gpu.texture_id),
                    None => {
                        renderer.acquire_individual(preview.width, preview.height, &premul_bytes)
                    }
                };
                match upload_result {
                    Ok(texture_id) => {
                        *bgremoval_preview_gpu = Some(BgremovalPreviewGpu {
                            texture_id,
                            width: preview.width,
                            height: preview.height,
                            arc_token: cache_token,
                            entity_bits: preview.entity_bits,
                        });
                    }
                    Err(e) => {
                        // Audit T1.6 R7 J1-3: surface GPU upload errors
                        // via toast instead of an eprintln the user
                        // never reads. The next frame retries
                        // automatically (we drop the stale slot below).
                        toasts.push(Toast::error(tr_with(
                            "shell.bgremoval_preview_gpu.preview_upload_failed",
                            &[("e", &e)],
                        )));
                        // Drop the stale slot; next frame retries.
                        release_preview_texture(renderer, bgremoval_preview_gpu);
                    }
                }
            }
        }
        None => {
            release_preview_texture(renderer, bgremoval_preview_gpu);
        }
    }
}

/// A tinta da máscara de protecção e o anel do pincel, por cima da prévia (Vello, dicas de UI).
#[allow(clippy::too_many_arguments)]
pub(super) fn draw_overlays(
    bgremoval_preview: &Option<BgremovalPreview>,
    protect_tint: Option<(Arc<Vec<u8>>, u32, u32)>,
    brush_ring: Option<(f32, u32)>,
    sim: &SimWorld,
    hero: &HeroScreen,
    camera: &Camera2d,
    window_size: WindowSize,
    vector_scene: &mut VectorScene,
    theme: Theme,
) {
    // ── Protection-mask tint + brush-size ring (Vello, UI hints) ──────────
    // The two remaining Vello overlays. They are UI affordances, not
    // image data — alpha-blended hints on top of the live preview.
    // They can stay in Vello because they don't need byte-for-byte
    // parity with anything. Gated on the preview being loaded so they
    // disappear in sync with the sprite-pipeline live preview.
    if let Some(preview) = bgremoval_preview {
        let entity = ph2d_ecs::Entity::from_bits(preview.entity_bits);
        // ⚠️ Pose de MUNDO — vide o doc do `sprite_image_to_screen_affine`.
        if let (Some(tr), Some(sprite)) = (
            ph2d_ecs::world_transform(sim.world(), entity),
            sim.world().get::<Sprite>(entity),
        ) {
            // A grelha desta sprite (ADR-0164 F1 passo 6) — ausente = uma célula.
            let grid = sim.world().get::<ph2d_ecs::SpriteGrid>(entity).copied();
            let quality = match hero.project.image_filter {
                ph2d_editor_core::ImageFilterMode::PixelArt => ImageQuality::Low,
                ph2d_editor_core::ImageFilterMode::Smooth => ImageQuality::Medium,
            };
            // Protection-mask tint — same affine the suppressed
            // sprite would use, so the tint tracks the live preview
            // pixel-for-pixel even when the sprite is rotated/scaled.
            if let Some((tint, tw, th)) = &protect_tint {
                let tint_to_screen =
                    sprite_image_to_screen_affine(*tw, *th, tr, sprite, grid, camera, window_size);
                vector_scene.draw_image_rgba_transformed(tint, *tw, *th, tint_to_screen, quality);
            }
            // Brush-size ring at the cursor — the source-px radius
            // mapped to screen via the footprint scale (extracted
            // from the affine's per-axis magnitude).
            if let (Some((r_src, src_w)), Some((cur_x, cur_y))) = (
                brush_ring,
                crate::input_dispatch::protect_brush::brush_cursor(),
            ) && src_w > 0
            {
                // `Affine` matrix is [a b c; d e f]; the column vector
                // `[a, d]` is image-X mapped to screen — its magnitude
                // is the per-pixel scale on the X axis.
                let m = sprite_image_to_screen_affine(
                    preview.width,
                    preview.height,
                    tr,
                    sprite,
                    grid,
                    camera,
                    window_size,
                )
                .as_coeffs();
                let pixel_scale = (m[0] * m[0] + m[1] * m[1]).sqrt() as f32; // |col 0|
                let src_to_screen = pixel_scale * preview.width as f32 / src_w as f32;
                let r_screen = r_src * src_to_screen;
                let accent = ColorToken::Accent.resolve(theme);
                let color = Color::from_rgba8(accent.r, accent.g, accent.b, 255);
                vector_scene.inner_mut().stroke(
                    &Stroke::new(StrokeToken::Default.px() as f64),
                    Affine::IDENTITY,
                    &Brush::Solid(color),
                    None,
                    &Circle::new((cur_x as f64, cur_y as f64), r_screen as f64),
                );
            }
        }
    }
}

/// Release the live-preview's `IndividualTextureStore` slot (if any)
/// and zero the cache. Called when the preview cache turns `None`
/// (tool deactivated, source unavailable, post-Apply transition) and
/// when an upload errors out — next frame's lifecycle re-acquires
/// from scratch.
fn release_preview_texture(
    renderer: &mut SpriteRenderer,
    bgremoval_preview_gpu: &mut Option<BgremovalPreviewGpu>,
) {
    if let Some(gpu) = bgremoval_preview_gpu.take() {
        renderer.individual_mut().release(gpu.texture_id);
    }
}
