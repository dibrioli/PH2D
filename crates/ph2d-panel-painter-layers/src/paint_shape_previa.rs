//! The Shape **tip preview**'s colour from the **Shape Color layers** (prompt 38) — split from
//! [`crate::paint_shape`] for the panel LOC cap. It paints the SAME stack the engine does.

use crate::state;
use ph2d_tool_painter::{
    BrushSettings, FootprintDeform, ImageMask, TextureKind, render_shape_preview,
};

/// The preview's colour from the **Shape Color layers** (prompt 38), per texel of the `side²` tip: straight
/// sRGB + alpha (coverage × the ramp's alpha) — `None` when the layers don't colour (ramp off, B&W on, or
/// no layer that paints). The SAME stack the engine paints ([`ph2d_tool_painter::ShapeRampLayers`]): the
/// Texture Color by the Shape's texture value — and, over a procedural pattern, the coverage is the bare
/// falloff (the pattern turned colour) —, the Falloff Color by the coverage; without a Shape texture the
/// Texture Color paints nothing (the tool's `TexturaSemForma`).
pub(crate) fn camadas_da_previa(
    brush: &BrushSettings,
    kind: TextureKind,
    falloff: &[f32],
    image: Option<&ImageMask>,
    side: u32,
    cov: &[u8],
) -> Option<Vec<[f32; 4]>> {
    if !brush.shape_color_ramp_enabled || brush.shape_color_ramp_bw {
        return None;
    }
    let inerte = state::current_inercias().inerte(ph2d_tool_painter::Dependente::TexturaSemForma);
    let (mut tlut, mut flut) = ([[0.0f32; 4]; 256], [[0.0f32; 4]; 256]);
    let n = |c: u8, s: &[[f32; 6]]| (c as usize).min(s.len());
    let tex = brush.shape_texture_color
        && !inerte
        && crate::paint_ramp_widget::build_preview_lut(
            true,
            false,
            brush.shape_color_ramp_mode,
            brush.shape_color_ramp_interp,
            &brush.shape_color_ramp_stops[..n(
                brush.shape_color_ramp_stop_count,
                &brush.shape_color_ramp_stops,
            )],
            &mut tlut,
        );
    let queda = brush.shape_falloff_color
        && crate::paint_ramp_widget::build_preview_lut(
            true,
            false,
            brush.shape_falloff_ramp_mode,
            brush.shape_falloff_ramp_interp,
            &brush.shape_falloff_ramp_stops[..n(
                brush.shape_falloff_ramp_stop_count,
                &brush.shape_falloff_ramp_stops,
            )],
            &mut flut,
        );
    if !tex && !queda {
        return None;
    }
    let render = |k: TextureKind, f: &[f32]| {
        let mut b = vec![0u8; (side * side) as usize];
        render_shape_preview(
            k,
            brush.shape_angle_deg,
            brush.shape_offset,
            brush.shape_size,
            brush.shape_params,
            FootprintDeform::new(brush.dab_flatten, brush.dab_angle_deg),
            f,
            None,
            image,
            &mut b,
            side,
            side,
        );
        b
    };
    let (visivel, valor) = if tex && kind != TextureKind::Image {
        (render(TextureKind::None, falloff), render(kind, &[1.0; 64]))
    } else {
        (cov.to_vec(), cov.to_vec())
    };
    let camadas = ph2d_tool_painter::ShapeRampLayers {
        texture: tex,
        falloff: queda.then_some(&flut[..]),
        falloff_first: brush.shape_falloff_first,
        blend: ph2d_tool_painter::BrushBlend::from_u8(brush.shape_color_blend),
        texture_opacity: brush.shape_texture_opacity,
        falloff_opacity: brush.shape_falloff_opacity,
    };
    let u = |b: u8| f32::from(b) / 255.0; // LITERAL-PX-OK: 8-bit byte normalize
    Some(
        visivel
            .iter()
            .zip(&valor)
            .map(|(&c, &v)| {
                let k = camadas.color(&tlut, u(v), u(c));
                [k[0], k[1], k[2], u(c) * k[3]]
            })
            .collect(),
    )
}
