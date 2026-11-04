//! O cartão Shape Color em linhas de cartão (fatia H5, `docs/CartoesUI/13_PAINTER_PLANO.md`): mesmos ids, mesmos eventos.
//! Cada caixa de camada vira um grupo; o editor da rampa continua no pintor dele, capturado (arrasta punhos, lê a cor pelo
//! `PaintCtx`), e os dois painéis partilham-no.

use crate::cards_captura::{Captura, chave_da_peca, x_de, y_de};
use crate::cards_linhas::{Item, botoes, fila, grupo, numero};
use crate::paint_ramp_widget::{RampIds, RampView, paint_ramp_body, ramp_body_height};
use crate::paint_shape_ramp::{Corpo, falloff_ramp_ids, pinta_corpo, shape_ramp_ids};
use crate::paint_wetpaint::Numero;
use crate::{esmaecer, number_field as nf, paint, paint_brush, state};
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::panel::PaintCtx;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Widget};
use ph2d_editor_core::widget::card_ink::CardInk;
use ph2d_i18n::tr;
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{BrushBlend, BrushSettings, Dependente, LinhaDaFerramenta};

const SECAO: ph2d_a11y::NodeId = pids::PAINTER_SHAPE_RAMP_SECTION;
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

/// O corpo de cada camada que pede o `PaintCtx`: o editor da rampa, ou a caixa inteira esmaecida e sem clique.
pub(crate) type Pecas = [Option<Captura>; 2];

fn rampa(
    ctx: &mut PaintCtx,
    ink: CardInk,
    x: f32,
    n: u64,
    pinta: impl FnOnce(&mut PaintCtx, ph2d_tokens::Theme, f32, f32) -> f32,
) -> Captura {
    let chave = chave_da_peca(SECAO, n);
    let topo = ctx.slot.y;
    let tema = ink.theme;
    let x = x_de(chave, x);
    Captura::da_peca(ctx, chave, SECAO, ink, (x, y_de(chave, topo)), |ctx, y| {
        pinta(ctx, tema, x, y)
    })
}

/// Corre as peças à vista, uma vez por quadro, com a chave da posição na pilha.
pub(crate) fn pecas(
    ctx: &mut PaintCtx,
    ink: CardInk,
    (x, w): (f32, f32),
    brush: BrushSettings,
) -> Pecas {
    if !brush.shape_color_ramp_enabled {
        return [None, None];
    }
    let lista = camadas(&brush);
    let mut fora: Pecas = [None, None];
    for (n, camada) in lista.iter().enumerate() {
        let corpo = &camada.corpo;
        fora[n] = if camada.inerte {
            Some(rampa(ctx, ink, x, n as u64, |ctx, tema, x, y| {
                inerte(ctx, tema, (x, w), y, corpo)
            }))
        } else if corpo.ligada {
            let (ids, vista): (RampIds, RampView<'_>) = (corpo.rids, corpo.view);
            Some(rampa(ctx, ink, x, n as u64, |ctx, tema, x, y| {
                paint_ramp_body(ctx, tema, x, w, y, &ids, vista)
            }))
        } else {
            None
        };
    }
    fora
}

/// Sem textura na forma, a cor dela não tem o que colorir: a caixa fica inteira, esmaecida e sem clique.
fn inerte(
    ctx: &mut PaintCtx,
    tema: ph2d_tokens::Theme,
    (x, w): (f32, f32),
    y: f32,
    corpo: &Corpo<'_>,
) -> f32 {
    let linhas = if corpo.ligada { 2.0 } else { 1.0 };
    let rampa = if corpo.ligada {
        ramp_body_height(w, false)
    } else {
        0.0
    };
    let h = linhas * ph2d_tokens::row_pitch_px() + rampa;
    let area = ph2d_editor_core::zones::Rect::new(x, y, w, h);
    let hit = ctx.host.hit_index_mut();
    hit.push_clip(ph2d_editor_core::zones::Rect::new(x, y, 0.0, 0.0));
    crate::esmaecer::esmaecido(ctx, tema, area, |ctx| {
        pinta_corpo(ctx, tema, x, w, y, corpo);
    });
    ctx.host.hit_index_mut().pop_clip();
    y + h
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
        .filter(|c| c.corpo.ligada && !c.inerte)
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
    let store = ctx.host.store_mut();
    crate::cards_wet::arma_numeros(
        store,
        &numeros(b, b),
        &numeros(b, &paint_brush::FALLBACK_BRUSH),
    );
    paint::register_button(store, SOBE);
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
    let id = pids::PAINTER_SHAPE_COLOR_BLEND;
    if crate::cards_linhas::aberta(ctx.host.store(), id)
        && let Some(r) = ctx.host.hit_index_mut().rect_for(id)
    {
        crate::state_shape_cor::set_pending_blend_dd(Some((r, b.shape_color_blend)));
    }
}

fn camada(c: &mut CardCtx<'_>, k: &Camada<'_>, peca: Option<&Captura>, y: f32) -> f32 {
    let y = grupo(c, k.titulo, y);
    if k.inerte {
        let y = if k.de_cima {
            y
        } else {
            botoes(c, &[(SOBE, tr("panel.painter_layers.ramp.move_up"))], y)
        };
        return peca.map_or(y, |p| p.repete(c, y + m::FIELD_ROW_GAP));
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
    let y = peca.map_or(y, |p| p.repete(c, y + m::FIELD_ROW_GAP));
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
pub(crate) fn corpo(c: &mut CardCtx<'_>, p: &Pecas, y: f32) -> f32 {
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
    for (k, peca) in camadas(&b).iter().zip(p) {
        y = camada(c, k, peca.as_ref(), y);
    }
    y
}
