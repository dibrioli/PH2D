//! ⭐ **Os widgets próprios no corpo do cartão** (fim da H9, `docs/CartoesUI/13_PAINTER_PLANO.md`): o título que
//! dobra, a prévia, o editor da rampa e as linhas de cada camada da forma. Mesmos ids e mesmos eventos do painel de
//! antes. A referência da rampa é o cartão Gradient da Widget Gallery: a barra, e o stop escolhido por baixo dela.

use crate::cards_linhas::{Item, aberta, com_lei, fila, filete, registra_lista};
use crate::cards_tela::Tela;
use crate::paint_ramp_widget::{BAR_H, MARK_R, RampIds, RampView, paint_ramp_bar};
use crate::paint_shape_layers::{OPACITY_PCT, enc, layer_color_readback};
use crate::paint_texture::{Previa, altura_do_preview};
use crate::{esmaecer, number_field, paint::register_button, state};
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{paint_icon, paint_text};
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget, corpo as linhas};
use ph2d_editor_core::panel::{PaintCtx, TextKey};
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_editor_core::widget::{
    CheckboxValue, IconButtonStyle, IconGlyph, TextInputState, paint_icon_button,
};
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::{tr, tr_with};
use ph2d_tokens::{ColorToken, StrokeToken};
use ph2d_tool_painter::ids as pids;
use ph2d_tool_painter::{BlendMode, BrushSettings, RampColorMode, RampInterp};
use ph2d_vector::{Affine, Compose, Mix, RoundedRect, VelloBlend};

/// Quantas linhas cabem no véu de uma rampa inerte: na coluna mais estreita, cada campo tem a sua.
const LINHAS_DO_VEU: f32 = 10.0; // LITERAL-PX-OK: conta de linhas, medida na coluna estreita
const PASSO_DA_POSICAO: f64 = 0.01; // LITERAL-PX-OK: passo do campo Position
const CASAS_DA_POSICAO: usize = 3;

/// A prévia a toda a largura, a partir de `y`.
pub(crate) fn previa(c: &mut CardCtx<'_>, p: Option<&Previa>, y: f32) -> f32 {
    let Some(p) = p else {
        return y;
    };
    let topo = y + m::FIELD_ROW_GAP;
    let h = altura_do_preview(c.w);
    p.pinta(c.scene, c.ink.theme, Rect::new(c.x, topo, c.w, h));
    topo + h
}

/// Um título de grupo que dobra: o filete, a seta e o nome no tom dos rótulos (nunca um 2.º título), com o Reset do
/// grupo à direita, se ele o tem.
pub(crate) fn titulo_que_dobra(
    c: &mut CardCtx<'_>,
    id: NodeId,
    chave: &str,
    reset: Option<NodeId>,
    y: f32,
) -> f32 {
    let topo = filete(c, y) + m::FIELD_ROW_GAP;
    let h = m::ICON_CELL;
    let lado = m::ICON_GLYPH;
    let centro = topo + (h - lado) * 0.5;
    let seta = Rect::new(c.x, centro, lado, lado);
    let icone = if c.store.is_collapsed(id) {
        IconId::ChevronRight
    } else {
        IconId::ChevronDown
    };
    paint_icon(
        c.scene,
        icone,
        seta,
        c.ink.color(ColorToken::CardLabel),
        StrokeToken::Default.px(),
    );
    let reservado = if reset.is_some() { h } else { 0.0 };
    let texto_x = c.x + lado + m::SEG_ICON_GAP;
    paint_text(
        c.text_system,
        c.scene,
        tr(chave),
        texto_x,
        topo + (h - m::LABEL_PX) * 0.5,
        m::LABEL_PX,
        (c.x + c.w - reservado - texto_x).max(0.0),
        c.ink.color(ColorToken::CardLabel),
    );
    if let Some(r) = reset {
        let caixa = Rect::new(c.x + c.w - h, topo, h, h);
        paint_icon_button(
            caixa,
            IconGlyph::Builtin(IconId::Reset),
            IconButtonStyle::Plain,
            c.store.button_visual(r),
            c.scene,
            c.ink.theme,
        );
        c.hit_index.register(r, caixa);
    }
    c.hit_index
        .register(id, Rect::new(c.x, topo, (c.w - reservado).max(0.0), h));
    topo + h
}

fn escolhida(v: &RampView<'_>) -> usize {
    v.stops
        .iter()
        .position(|s| s[5] as u8 == v.selected_id)
        .unwrap_or(0)
}

fn rgba_do_stop(s: [f32; 6]) -> [u8; 4] {
    [enc(s[1]), enc(s[2]), enc(s[3]), enc(s[4])]
}

/// Com o store MUTÁVEL, uma vez por quadro: os botões, as listas, os dois números e a cor que o seletor devolve.
pub(crate) fn prepara(ctx: &mut PaintCtx, ids: &RampIds, v: RampView<'_>) {
    let sel = escolhida(&v);
    crate::paint_ramp_leitura::ramp_color_readback(ctx, ids, v, sel);
    let store = ctx.host.store_mut();
    for (id, dica) in [
        (ids.add, "card.painter.ramp.add_tip"),
        (ids.remove, "card.painter.ramp.remove_tip"),
        (ids.invert, "card.painter.ramp.invert_tip"),
    ] {
        register_button(store, id);
        store.set_tooltip(id, tr(dica));
    }
    register_button(store, ids.swatch);
    register_button(store, ids.reset);
    store.set_tooltip(ids.reset, tr("panel.painter_layers.brush.reset_section"));
    registra_lista(store, ids.mode, usize::from(v.mode));
    registra_lista(store, ids.interp, usize::from(v.interp));
    let Some(s) = v.stops.get(sel.min(v.stops.len().saturating_sub(1))) else {
        return;
    };
    arma(
        store,
        ids.stop_index,
        sel as f32,
        (0.0, (v.stops.len() - 1) as f32),
        1.0,
        0,
    );
    arma(
        store,
        ids.stop_pos,
        s[0],
        (0.0, 1.0),
        PASSO_DA_POSICAO,
        CASAS_DA_POSICAO,
    );
}

fn arma(
    store: &mut WidgetStore,
    id: NodeId,
    valor: f32,
    (min, max): (f32, f32),
    passo: f64,
    casas: usize,
) {
    let _ = store.register_if_absent(
        id,
        InteractiveState::NumberInput {
            state: TextInputState::Normal,
            value: f64::from(valor),
            buffer: String::new(),
            caret: 0,
            last_committed: f64::from(valor),
            selection_anchor: None,
        },
    );
    store.mark_card_field(
        id,
        CardFieldSpec {
            step: passo,
            scrub: passo,
            default: None,
        },
    );
    number_field::arm_field(store, id, valor, min, max, passo, casas);
}

/// Onde a caixa aberta ficou é o que a lista das opções precisa.
pub(crate) fn listas_abertas(ctx: &mut PaintCtx, ids: &RampIds, v: RampView<'_>) {
    for (id, atual, guarda) in [
        (ids.mode, v.mode, ids.set_pending_mode),
        (ids.interp, v.interp, ids.set_pending_interp),
    ] {
        if aberta(ctx.host.store(), id)
            && let Some(r) = ctx.host.hit_index_mut().rect_for(id)
        {
            guarda(Some((r, atual)));
        }
    }
}

/// Um número sem marca de animação: o stop escolhido é vista do painel, não uma propriedade do pincel.
fn campo<'a>(id: NodeId, rotulo: &'a str) -> Cell<'a> {
    Cell::new(Widget::Field {
        id,
        glyph: Glyph::Text(linhas::letra_da_grandeza(rotulo)),
    })
    .label(rotulo)
}

/// O editor: as duas listas, a barra com uma pega por stop, e o stop escolhido por baixo.
pub(crate) fn corpo(c: &mut CardCtx<'_>, ids: &RampIds, v: RampView<'_>, y: f32) -> f32 {
    let sel = escolhida(&v);
    let listas = [
        Item::Lista {
            id: ids.mode,
            chave: "card.painter.ramp.color_mode",
            texto: tr(RampColorMode::from_u8(v.mode).label_key()),
        },
        Item::Lista {
            id: ids.interp,
            chave: "card.painter.ramp.interpolation",
            texto: tr(RampInterp::from_u8(v.interp).label_key()),
        },
    ];
    let y = fila(c, &listas, y);
    let topo = y + m::FIELD_ROW_GAP;
    let barra = Rect::new(c.x, topo, c.w, BAR_H);
    paint_ramp_bar(&mut Tela::do_cartao(c), barra, ids, v, sel);
    let y = topo + BAR_H + MARK_R;
    let Some(s) = v.stops.get(sel.min(v.stops.len().saturating_sub(1))) else {
        return y;
    };
    let y = c.compact(
        &[
            campo(ids.stop_index, tr("card.painter.ramp.stop")),
            campo(ids.stop_pos, tr("card.painter.ramp.position")),
        ],
        y,
    );
    let rgba = rgba_do_stop(*s);
    let codigo = format!("{:02X}{:02X}{:02X}", rgba[0], rgba[1], rgba[2]);
    let botoes = [
        Seg {
            id: ids.add,
            icon: None,
            text: "+",
            on: false,
        },
        Seg {
            id: ids.remove,
            icon: None,
            text: "−",
            on: false,
        },
        Seg {
            id: ids.invert,
            icon: None,
            text: "I",
            on: false,
        },
    ];
    c.row(
        &[
            Cell::new(Widget::Swatch {
                id: ids.swatch,
                rgba: Some(rgba),
                text: &codigo,
                edit: None,
                detach: None,
            })
            .label(tr("card.painter.ramp.color")),
            Cell::new(Widget::Bare(&botoes)),
        ],
        y,
    )
}

/// O véu de uma rampa que não age: esmaece tudo o que `pintar` desenhar. Sem medida prévia, porque a camada só pinta
/// o que lhe chega.
pub(crate) fn com_veu(
    c: &mut CardCtx<'_>,
    y: f32,
    pintar: impl FnOnce(&mut CardCtx<'_>, f32) -> f32,
) -> f32 {
    let forma = RoundedRect::new(
        f64::from(c.x),
        f64::from(y),
        f64::from(c.x + c.w),
        f64::from(y + LINHAS_DO_VEU * (m::FIELD_H + m::LABEL_PX + m::FIELD_ROW_GAP)),
        0.0,
    );
    c.scene.push_layer_shape(
        VelloBlend::new(Mix::Normal, Compose::SrcOver),
        ph2d_editor_core::property_row::inativa::alfa_inativo(c.ink.theme),
        Affine::IDENTITY,
        &forma,
    );
    let fim = pintar(c, y);
    c.scene.pop_layer();
    fim
}

fn quantas_camadas(b: &BrushSettings) -> usize {
    usize::from(b.shape_layer_count).min(b.shape_layer_color_on.len())
}

/// As linhas de cada camada, com o store MUTÁVEL.
pub(crate) fn prepara_camadas(ctx: &mut PaintCtx, b: &BrushSettings) {
    for i in 0..quantas_camadas(b) {
        let iu = i as u8;
        let check = pids::painter_shape_layer_color_check_id(iu);
        let store = ctx.host.store_mut();
        register_button(store, check);
        let _ = esmaecer::poe_dica(store, &[check]);
        if i > 0 {
            registra_lista(
                store,
                pids::painter_shape_layer_blend_id(iu),
                usize::from(b.shape_layer_blend[i]),
            );
        }
        arma(
            store,
            pids::painter_shape_layer_opacity_id(iu),
            b.shape_layer_opacity[i] * OPACITY_PCT,
            (0.0, OPACITY_PCT),
            1.0,
            0,
        );
        if b.shape_layer_color_on[i] {
            let amostra = pids::painter_shape_layer_color_swatch_id(iu);
            register_button(store, amostra);
            layer_color_readback(ctx, amostra, iu, b.shape_layer_color[i]);
        }
    }
}

/// Uma lista de Blend aberta de cada vez: a primeira, de cima para baixo.
pub(crate) fn camadas_abertas(ctx: &mut PaintCtx, b: &BrushSettings) {
    for i in 1..quantas_camadas(b) {
        let id = pids::painter_shape_layer_blend_id(i as u8);
        if !aberta(ctx.host.store(), id) {
            continue;
        }
        if state::pending_shape_blend_dd().is_some() {
            if let Some(InteractiveState::Dropdown { open, .. }) = ctx.host.store_mut().get_mut(id)
            {
                *open = false;
            }
        } else if let Some(r) = ctx.host.hit_index_mut().rect_for(id) {
            state::set_pending_shape_blend_dd(Some((i as u8, r, b.shape_layer_blend[i])));
        }
    }
}

/// A cor de cada camada, e o Blend com a opacidade. A camada de baixo não tem Blend, como a de fundo em Layers.
pub(crate) fn linhas_por_camada(c: &mut CardCtx<'_>, b: &BrushSettings, y: f32) -> f32 {
    let mut y = y;
    for i in 0..quantas_camadas(b) {
        let iu = i as u8;
        let check = pids::painter_shape_layer_color_check_id(iu);
        let on = b.shape_layer_color_on[i];
        let nome = tr_with("panel.painter_layers.shape.layer_color", &[("n", &(i + 1))]);
        let rgb = b.shape_layer_color[i];
        let codigo = format!("{:02X}{:02X}{:02X}", enc(rgb[0]), enc(rgb[1]), enc(rgb[2]));
        y = com_lei(c, check, y, |c, y| {
            let marca = Cell::new(Widget::Check {
                id: check,
                label: Rotulo::vista(TextKey::new("panel.painter_layers.shape.layer_color"))
                    .com_texto(&nome),
                value: if on {
                    CheckboxValue::Checked
                } else {
                    CheckboxValue::Unchecked
                },
                off: false,
            });
            if on {
                let cor = Cell::new(Widget::Swatch {
                    id: pids::painter_shape_layer_color_swatch_id(iu),
                    rgba: Some([enc(rgb[0]), enc(rgb[1]), enc(rgb[2]), u8::MAX]),
                    text: &codigo,
                    edit: None,
                    detach: None,
                });
                c.pair(marca, cor, y)
            } else {
                c.row(&[marca], y)
            }
        });
        let opacidade = campo(
            pids::painter_shape_layer_opacity_id(iu),
            tr("card.painter.ramp.layer_opacity"),
        );
        y = if i > 0 {
            let id = pids::painter_shape_layer_blend_id(iu);
            let texto = tr(BlendMode::from_u8(b.shape_layer_blend[i]).label_key());
            let lista = Cell::new(Widget::Dropdown {
                id,
                text: texto,
                options: &[],
            })
            .label(tr("panel.painter_layers.shape.blend"));
            c.compact(&[lista, opacidade], y)
        } else {
            c.row(&[opacidade], y)
        };
    }
    y
}
