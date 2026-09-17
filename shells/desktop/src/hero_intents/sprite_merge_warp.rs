//! Os passos 2 e 3 do [`super::drain_merge_sprites`] — a caixa de união com a grelha de saída, e o
//! warp que a enche. Filho por ASSUNTO do `sprite_merge` (como o `resample`), para os campos privados
//! do `SrcRecord` continuarem alcançáveis; o dreno chama as duas funções no sítio exacto dos blocos.

use super::{SrcRecord, bilinear_sample_premul, world_to_image};
use ph2d_editor_core::{Toast, ToastQueue};
use ph2d_i18n::{tr, tr_with};
use ph2d_render::SpriteRenderer;

/// A caixa de união e a grelha de saída que o [`merge_grid`] devolve, com os nomes que o dreno usa.
pub(super) struct MergeGrid {
    pub(super) union_min_x: f32,
    pub(super) union_max_x: f32,
    pub(super) union_min_y: f32,
    pub(super) union_max_y: f32,
    pub(super) union_w_m: f32,
    pub(super) union_h_m: f32,
    pub(super) out_pm: f32,
    pub(super) out_w: u32,
    pub(super) out_h: u32,
}

/// Os passos 2 e 2.5: a caixa de união em metros — alinhada à grelha de pixels da fonte primária
/// quando ela é axial — e o tamanho da saída; `None` depois do toast que diz porque não há saída.
pub(super) fn merge_grid(
    srcs: &[SrcRecord],
    project_pm: f32,
    renderer: &SpriteRenderer,
    toasts: &mut ToastQueue,
) -> Option<MergeGrid> {
    // Step 2 — union bbox in world meters.
    let mut union_min_x = srcs
        .iter()
        .map(|s| s.world_min_x)
        .fold(f32::INFINITY, f32::min);
    let mut union_max_x = srcs
        .iter()
        .map(|s| s.world_max_x)
        .fold(f32::NEG_INFINITY, f32::max);
    let mut union_min_y = srcs
        .iter()
        .map(|s| s.world_min_y)
        .fold(f32::INFINITY, f32::min);
    let mut union_max_y = srcs
        .iter()
        .map(|s| s.world_max_y)
        .fold(f32::NEG_INFINITY, f32::max);
    if union_max_x <= union_min_x || union_max_y <= union_min_y {
        toasts.push(Toast::error(tr("shell.sprite_merge_warp.merge_sprites")));
        return None;
    }

    // Step 2.5 — pixel-grid alignment (Enio 2026-05-27 "o merge não
    // modificar nada das imagens prévias sobrepostas"). For the common
    // case where the first source is axis-aligned at unit scale, two
    // changes make the output LOSSLESS for that source AND any source
    // sharing its grid:
    //
    //   (a) Output density matches the source's native px/m instead of
    //       the (possibly different) project px/m. No up/down-sampling.
    //   (b) Union bbox snaps to the source's pixel boundaries. Output
    //       pixel `o` then aligns with source pixel `o + k` (integer k)
    //       → `img_x` from the inverse warp is exactly integer →
    //       bilinear at integer reads ONE sample → no half-pixel blur.
    //
    // Rotated / scaled sources still bilinear-resample (unavoidable),
    // but the dark-fringe is fixed by the premul-space sampling above.
    let primary = &srcs[0];
    let primary_axis_aligned = primary.rot.abs() < 1e-4
        && (primary.scale_x - 1.0).abs() < 1e-4
        && (primary.scale_y - 1.0).abs() < 1e-4;
    let out_pm = if primary_axis_aligned && primary.size_w > 0.0 && primary.size_h > 0.0 {
        let px_per_m_w = primary.w as f32 / primary.size_w;
        let px_per_m_h = primary.h as f32 / primary.size_h;
        // Sanity: square pixels for an axis-aligned unit-scale sprite.
        // Average defensively against floating-point asymmetry.
        (px_per_m_w + px_per_m_h) * 0.5
    } else {
        project_pm
    };
    if primary_axis_aligned {
        // Primary's pixel boundaries in world.
        let p_left = primary.tx - primary.size_w * 0.5 + primary.anchor_x;
        let p_top = primary.ty + primary.size_h * 0.5 + primary.anchor_y;
        let snap_to_grid = |coord: f32, origin: f32, ceil: bool| {
            let offset = (coord - origin) * out_pm;
            let snapped = if ceil { offset.ceil() } else { offset.floor() };
            origin + snapped / out_pm
        };
        // Floor for min, ceil for max — grow the union outward so no
        // contribution from any source gets clipped.
        union_min_x = snap_to_grid(union_min_x, p_left, false);
        union_max_x = snap_to_grid(union_max_x, p_left, true);
        union_min_y = snap_to_grid(union_min_y, p_top, false);
        union_max_y = snap_to_grid(union_max_y, p_top, true);
    }
    let union_w_m = union_max_x - union_min_x;
    let union_h_m = union_max_y - union_min_y;

    let out_w = (union_w_m * out_pm).round().max(1.0) as u32;
    let out_h = (union_h_m * out_pm).round().max(1.0) as u32;
    let max_dim = renderer.max_texture_dimension_2d();
    if out_w > max_dim || out_h > max_dim {
        toasts.push(Toast::error(tr_with(
            "shell.sprite_merge_warp.merge_sprites_output",
            &[("out_w", &out_w), ("out_h", &out_h), ("max_dim", &max_dim)],
        )));
        return None;
    }
    Some(MergeGrid {
        union_min_x,
        union_max_x,
        union_min_y,
        union_max_y,
        union_w_m,
        union_h_m,
        out_pm,
        out_w,
        out_h,
    })
}

/// O passo 3: o warp para trás e a composição «over» premultiplicada; devolve a saída e, no modo
/// camadas, o buffer de cada fonte.
pub(super) fn composite(
    srcs: &[SrcRecord],
    out_w: u32,
    out_h: u32,
    union_min_x: f32,
    union_max_y: f32,
    out_pm: f32,
    to_layers: bool,
) -> (Vec<u8>, Vec<Vec<u8>>) {
    // Step 3 — backward-warp + premultiplied "over" composite.
    // Bytes sampled from `src.rgba` are already premultiplied (Step 1
    // normalised every source via `into_premultiplied`), so the
    // per-pixel inner loop just bilerps in premul space and runs the
    // canonical Porter-Duff "over" without re-multiplying by alpha.
    let n_pixels = (out_w as usize) * (out_h as usize);
    let mut out_rgba = vec![0u8; n_pixels * 4];
    // Um buffer por fonte, só no modo camadas — ver o parâmetro `to_layers`.
    let mut layer_rgba: Vec<Vec<u8>> = if to_layers {
        vec![vec![0u8; n_pixels * 4]; srcs.len()]
    } else {
        Vec::new()
    };
    for out_y in 0..out_h {
        let wy = union_max_y - (out_y as f32 + 0.5) / out_pm;
        for out_x in 0..out_w {
            let wx = union_min_x + (out_x as f32 + 0.5) / out_pm;
            let mut acc_r = 0.0_f32;
            let mut acc_g = 0.0_f32;
            let mut acc_b = 0.0_f32;
            let mut acc_a = 0.0_f32;
            let idx = ((out_y as usize) * (out_w as usize) + (out_x as usize)) * 4;
            for (si, src) in srcs.iter().enumerate() {
                if wx < src.world_min_x
                    || wx > src.world_max_x
                    || wy < src.world_min_y
                    || wy > src.world_max_y
                {
                    continue;
                }
                let (img_x, img_y) = world_to_image(wx, wy, src);
                let Some((pr_u8, pg_u8, pb_u8, pa_u8)) =
                    bilinear_sample_premul(&src.rgba, src.w, src.h, img_x, img_y)
                else {
                    continue;
                };
                if pa_u8 == 0 {
                    continue;
                }
                // ⚠️ A camada guarda o que ESTA fonte pôs neste pixel, **antes** do «over» com as
                // outras: é isso que faz dela uma camada em vez de uma fatia do resultado.
                if to_layers {
                    layer_rgba[si][idx..idx + 4].copy_from_slice(&[pr_u8, pg_u8, pb_u8, pa_u8]);
                }
                let pa = pa_u8 as f32 * (1.0 / 255.0);
                let pr = pr_u8 as f32 * (1.0 / 255.0);
                let pg = pg_u8 as f32 * (1.0 / 255.0);
                let pb = pb_u8 as f32 * (1.0 / 255.0);
                let inv = 1.0 - pa;
                acc_r = pr + acc_r * inv;
                acc_g = pg + acc_g * inv;
                acc_b = pb + acc_b * inv;
                acc_a = pa + acc_a * inv;
            }
            out_rgba[idx] = (acc_r * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
            out_rgba[idx + 1] = (acc_g * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
            out_rgba[idx + 2] = (acc_b * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
            out_rgba[idx + 3] = (acc_a * 255.0 + 0.5).clamp(0.0, 255.0) as u8;
        }
    }
    (out_rgba, layer_rgba)
}
