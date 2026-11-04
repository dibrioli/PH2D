//! O cartão Shape Color em linhas de cartão (fatia H5, `docs/CartoesUI/13_PAINTER_PLANO.md`): mesmos ids, mesmos eventos.
//! Cada caixa de camada vira um grupo; o editor da rampa é o de `cards_rampa` (arrasta punhos, lê a cor pelo
//! `PaintCtx`), e os dois painéis partilham-no.

use crate::cards_linhas::{Item, botoes, fila, grupo, numero};
use crate::paint_ramp_widget::RampView;
use crate::paint_shape_ramp::{Corpo, falloff_ramp_ids, shape_ramp_ids};
use crate::paint_wetpaint::Numero;
use crate::{cards_rampa, esmaecer, number_field as nf, paint_brush, state};
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Widget};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{BrushBlend, BrushSettings, Dependente, LinhaDaFerramenta};

const SOBE: ph2d_a11y::NodeId = pids::PAINTER_SHAPE_COLOR_SWAP;

/// A resposta do `plano_do_corpo`.
pub(crate) fn presente(b: &BrushSettings) -> bool {
    !b.is_inpaint && !b.shape_per_layer_color && b.le(LinhaDaFerramenta::Forma)
}

pub(crate) fn titulo() -> &'static str {
    tr("panel.painter_layers.shape.shape_color")
}

pub(crate) fn reset() -> ph2d_a11y::NodeId {
    shape_ramp_ids().reset
}

/// Smear, Blur, Clone e Borracha não pintam cor: o B&W não se oferece.
fn bw_preso(b: &BrushSettings) -> bool {
    b.paints_no_color() || b.eraser || b.is_mask
}

fn textura_inerte() -> bool {
    state::current_inercias().inerte(Dependente::TexturaSemForma)
}

/// Uma camada do tom, na ordem em que se lê.
struct Camada<'a> {
    titulo: &'static str,
    corpo: Corpo<'a>,
    de_cima: bool,
    blend: Option<u8>,
    inerte: bool,
}

fn camadas(b: &BrushSettings) -> [Camada<'_>; 2] {
    let bloqueado = bw_preso(b);
    let n = usize::from(b.shape_color_ramp_stop_count).min(b.shape_color_ramp_stops.len());
    let vista_t = RampView {
        enabled: true,
        bw: b.shape_color_ramp_bw || bloqueado,
        bw_locked: bloqueado,
        bw_button: false,
        mode: b.shape_color_ramp_mode,
        interp: b.shape_color_ramp_interp,
        alpha: None,
        stops: &b.shape_color_ramp_stops[..n],
        selected_id: state::selected_shape_ramp_stop(),
    };
    let q = usize::from(b.shape_falloff_ramp_stop_count).min(b.shape_falloff_ramp_stops.len());
    let vista_q = RampView {
        mode: b.shape_falloff_ramp_mode,
        interp: b.shape_falloff_ramp_interp,
        stops: &b.shape_falloff_ramp_stops[..q],
        selected_id: crate::state_shape_cor::selected_falloff_stop(),
        ..vista_t
    };
    let inerte = textura_inerte();
    let ambas = b.shape_texture_color && !inerte && b.shape_falloff_color;
    let de_textura = |de_cima: bool| Camada {
        titulo: "panel.painter_layers.ramp.texture_color",
        corpo: Corpo {
            id: pids::PAINTER_SHAPE_TEXTURE_COLOR,
            ligada: b.shape_texture_color,
            rids: shape_ramp_ids(),
            view: vista_t,
            id_op: pids::PAINTER_SHAPE_TEXTURE_OPACITY,
            opacidade: b.shape_texture_opacity,
        },
        de_cima,
        blend: (de_cima && ambas).then_some(b.shape_color_blend),
        inerte,
    };
    let de_queda = |de_cima: bool| Camada {
        titulo: "panel.painter_layers.ramp.falloff_color",
        corpo: Corpo {
            id: pids::PAINTER_SHAPE_FALLOFF_COLOR,
            ligada: b.shape_falloff_color,
            rids: falloff_ramp_ids(),
            view: vista_q,
            id_op: pids::PAINTER_SHAPE_FALLOFF_OPACITY,
            opacidade: b.shape_falloff_opacity,
        },
        de_cima,
        blend: (de_cima && ambas).then_some(b.shape_color_blend),
        inerte: false,
    };
    if b.shape_falloff_first {
        [de_queda(true), de_textura(false)]
    } else {
        [de_textura(true), de_queda(false)]
    }
}

fn opacidade(c: &Corpo<'_>, valor: f32) -> Numero {
    numero(
        "panel.painter_layers.ramp.opacity",
        c.id_op,
        valor.clamp(0.0, 1.0),
        (0.0, 1.0),
        nf::FINE_STEP,
        2,
    )
}

/// As opacidades à vista: as outras leem-se no capturado, e a da caixa inerte está no pintor dela.
fn numeros(l: &BrushSettings, v: &BrushSettings) -> Vec<Numero> {
    if !l.shape_color_ramp_enabled {
        return Vec::new();
    }
    let (da_lei, de_valor) = (camadas(l), camadas(v));
    da_lei
        .iter()
        .filter(|c| c.corpo.ligada)
        .map(|c| {
            let valor = de_valor
                .iter()
                .find(|o| o.corpo.id == c.corpo.id)
                .map_or(0.0, |o| o.corpo.opacidade);
            opacidade(&c.corpo, valor)
        })
        .collect()
}

/// Com o store MUTÁVEL, antes de pintar.
pub(crate) fn prepare(ctx: &mut PaintCtx, b: &BrushSettings) {
    if !presente(b) {
        return;
    }
    let ids = shape_ramp_ids();
    if b.shape_color_ramp_enabled {
        for k in camadas(b).iter().filter(|k| k.corpo.ligada) {
            crate::cards_rampa::prepara(ctx, &k.corpo.rids, k.corpo.view);
        }
    }
    let store = ctx.host.store_mut();
    crate::cards_wet::arma_numeros(
        store,
        &numeros(b, b),
        &numeros(b, &paint_brush::FALLBACK_BRUSH),
    );
    if textura_inerte() && store.picker_target() == Some(ids.swatch) {
        store.set_picker_target(None);
    }
    for id in [
        ids.enable,
        ids.bw,
        pids::PAINTER_SHAPE_TEXTURE_COLOR,
        pids::PAINTER_SHAPE_FALLOFF_COLOR,
        pids::PAINTER_SHAPE_COLOR_BLEND,
    ] {
        let _ = esmaecer::poe_dica(store, &[id]);
    }
}

/// O Blend abriu no cartão.
pub(crate) fn listas_abertas(ctx: &mut PaintCtx, b: &BrushSettings) {
    if b.shape_color_ramp_enabled {
        for k in camadas(b).iter().filter(|k| k.corpo.ligada && !k.inerte) {
            crate::cards_rampa::listas_abertas(ctx, &k.corpo.rids, k.corpo.view);
        }
    }
    let id = pids::PAINTER_SHAPE_COLOR_BLEND;
    if crate::cards_linhas::aberta(ctx.host.store(), id)
        && let Some(r) = ctx.host.hit_index_mut().rect_for(id)
    {
        crate::state_shape_cor::set_pending_blend_dd(Some((r, b.shape_color_blend)));
    }
}

/// A caixa inteira esmaecida e sem clique: a Texture Color sem forma que a leia.
fn inerte(c: &mut CardCtx<'_>, k: &Camada<'_>, y: f32) -> f32 {
    c.hit_index.push_clip(Rect::new(c.x, y, 0.0, 0.0));
    let fim = cards_rampa::com_veu(c, y, |c, y| {
        let liga = crate::cards_linhas::marca(
            k.corpo.id,
            "panel.painter_layers.ramp.enabled",
            k.corpo.ligada,
        );
        let y = c.row(&[liga], y);
        if !k.corpo.ligada {
            return y;
        }
        let y = cards_rampa::corpo(c, &k.corpo.rids, k.corpo.view, y);
        fila(c, &[Item::Num(opacidade(&k.corpo, k.corpo.opacidade))], y)
    });
    c.hit_index.pop_clip();
    fim
}

fn camada(c: &mut CardCtx<'_>, k: &Camada<'_>, y: f32) -> f32 {
    let y = grupo(c, k.titulo, y);
    if k.inerte {
        let y = if k.de_cima {
            y
        } else {
            botoes(c, &[(SOBE, tr("panel.painter_layers.ramp.move_up"))], y)
        };
        return inerte(c, k, y);
    }
    let liga = crate::cards_linhas::marca(
        k.corpo.id,
        "panel.painter_layers.ramp.enabled",
        k.corpo.ligada,
    );
    let y = if k.de_cima {
        c.row(&[liga], y)
    } else {
        let sobe = Cell::new(Widget::Button {
            id: SOBE,
            text: tr("panel.painter_layers.ramp.move_up"),
            primary: false,
        });
        c.pair(liga, sobe, y)
    };
    if !k.corpo.ligada {
        return y;
    }
    let y = cards_rampa::corpo(c, &k.corpo.rids, k.corpo.view, y);
    let mut itens = vec![Item::Num(opacidade(&k.corpo, k.corpo.opacidade))];
    if let Some(modo) = k.blend {
        itens.push(Item::Lista {
            id: pids::PAINTER_SHAPE_COLOR_BLEND,
            chave: "panel.painter_layers.ramp.blend",
            texto: tr(BrushBlend::from_u8(modo).name_key()),
        });
    }
    fila(c, &itens, y)
}

/// O corpo.
pub(crate) fn corpo(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let b = state::current_brush().unwrap_or(paint_brush::FALLBACK_BRUSH);
    let ids = shape_ramp_ids();
    let mut topo = vec![Item::Caixa {
        id: ids.enable,
        chave: "panel.painter_layers.ramp.use_color_ramp",
        on: b.shape_color_ramp_enabled,
    }];
    if b.shape_color_ramp_enabled && !bw_preso(&b) {
        topo.push(Item::Caixa {
            id: ids.bw,
            chave: "panel.painter_layers.ramp.bw",
            on: b.shape_color_ramp_bw,
        });
    }
    let mut y = fila(c, &topo, y);
    if !b.shape_color_ramp_enabled {
        return y;
    }
    for k in &camadas(&b) {
        y = camada(c, k, y);
    }
    y
}
