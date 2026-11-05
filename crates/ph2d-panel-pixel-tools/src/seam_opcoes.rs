//! As OPÇÕES da ferramenta no painel PIXEL: o campo, a caixa e o segmento que cada id escreve em
//! [`PixelOptions`] — a tabela que o clique e a pintura partilham.

use crate::ids;
use ph2d_a11y::NodeId;
use ph2d_editor_core::pixel_panel::PixelOptions;

/// O valor do campo `id` nas opções.
pub(crate) fn numero(o: &PixelOptions, id: NodeId) -> f64 {
    match id {
        ids::SIZE => f64::from(o.size),
        ids::ANGLE => f64::from(o.angle),
        ids::TOLERANCE => f64::from(o.tolerance),
        ids::SPRAY_WIDTH => f64::from(o.spray_width),
        ids::REPLACE_TOLERANCE => f64::from(o.replace_tolerance),
        ids::WAND_TOLERANCE => f64::from(o.wand_tolerance),
        ids::TEXT_SIZE => f64::from(o.text_px),
        _ => f64::from(o.spray_speed),
    }
}

/// As opções com o campo `id` em `v` (preso à faixa dele, inteiro).
pub(crate) fn com_numero(mut o: PixelOptions, id: NodeId, v: f64) -> PixelOptions {
    let (min, max) = ids::NUMBERS
        .iter()
        .find(|(n, _)| *n == id)
        .map_or((0.0, 0.0), |(_, (a, b, _))| (*a, *b));
    if !v.is_finite() {
        return o;
    }
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "presa à faixa do campo antes da conversão"
    )]
    let (u, i) = {
        let r = ph2d_editor_core::math::safe_clamp_f64(v.round(), min, max);
        (r as u32, r as i32)
    };
    match id {
        ids::SIZE => o.size = u,
        ids::ANGLE => o.angle = i,
        ids::TOLERANCE => o.tolerance = u8::try_from(u).unwrap_or(u8::MAX),
        ids::SPRAY_WIDTH => o.spray_width = u,
        ids::SPRAY_SPEED => o.spray_speed = u,
        ids::REPLACE_TOLERANCE => o.replace_tolerance = u8::try_from(u).unwrap_or(u8::MAX),
        ids::WAND_TOLERANCE => o.wand_tolerance = u8::try_from(u).unwrap_or(u8::MAX),
        ids::TEXT_SIZE => o.text_px = u,
        _ => {}
    }
    o
}

/// A caixa `id` nas opções (`&mut` do campo dela).
pub(crate) fn caixa(o: &mut PixelOptions, id: NodeId) -> Option<&mut bool> {
    Some(match id {
        ids::PIXEL_PERFECT => &mut o.pixel_perfect,
        ids::CONTIGUOUS => &mut o.contiguous,
        ids::GRADIENT_RAMP => &mut o.gradient_ramp,
        ids::WAND_CONTIGUOUS => &mut o.wand_contiguous,
        ids::AUTO_SELECT_LAYER => &mut o.auto_select_layer,
        ids::SYM_H => &mut o.sym_h,
        ids::SYM_V => &mut o.sym_v,
        ids::SYM_D1 => &mut o.sym_d1,
        ids::SYM_D2 => &mut o.sym_d2,
        ids::TILE_X => &mut o.tile_x,
        ids::TILE_Y => &mut o.tile_y,
        crate::seam_formas::SHAPE_FILL => &mut o.shape_fill,
        _ => return None,
    })
}

/// As opções com o segmento `id` escolhido, se é um.
pub(crate) fn com_segmento(mut o: PixelOptions, id: NodeId) -> Option<PixelOptions> {
    if let Some((shape, _)) = ids::SHAPES.iter().find(|(_, s)| *s == id) {
        o.shape = *shape;
        return Some(o);
    }
    if let Some((ink, _)) = ids::INKS.iter().find(|(_, s)| *s == id) {
        o.ink = *ink;
        return Some(o);
    }
    if let Some((d, ..)) = ids::GRADIENT_DITHERS.iter().find(|(_, s, _)| *s == id) {
        o.gradient_dither = *d;
        return Some(o);
    }
    match id {
        ids::GRADIENT_LINEAR => o.gradient_radial = false,
        ids::GRADIENT_RADIAL => o.gradient_radial = true,
        ids::CONNECT_4 => o.eight = false,
        ids::CONNECT_8 => o.eight = true,
        ids::SAMPLE_LAYER => o.sample_visible = false,
        ids::SAMPLE_VISIBLE => o.sample_visible = true,
        ids::WAND_CONNECT_4 => o.wand_eight = false,
        ids::WAND_CONNECT_8 => o.wand_eight = true,
        ids::WAND_SAMPLE_LAYER => o.wand_sample_visible = false,
        ids::WAND_SAMPLE_VISIBLE => o.wand_sample_visible = true,
        _ => {
            return crate::seam_selecao::com_segmento(o, id)
                .or_else(|| crate::seam_formas::com_opcao(o, id));
        }
    }
    Some(o)
}
