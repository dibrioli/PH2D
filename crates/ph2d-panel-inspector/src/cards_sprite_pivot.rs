//! O Pivot de 9 pontos do cartão Look ([ficha](../../../docs/GameEngineFuncional/componentes/01_sprite.md)), ao lado do Flip.

use crate::ids;
use crate::slot_image;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::paint::{paint_text, resolve};
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget};
use ph2d_editor_core::zones::Rect;
use ph2d_editor_core::{IconId, ImageUseEdit, SpritePivot};
use ph2d_i18n::tr;
use ph2d_tokens::{ColorToken, Spacing, TypeToken};

const CASAS_POR_LADO: f32 = 3.0; // LITERAL-PX-OK: a grade é 3 × 3, conta e não geometria
/// A marca do Flip grava os dois eixos; a do Pivot, o pivô e a posição (`marcas_inspector::mascara`).
const FLIP_KEY: [NodeId; 2] = [ids::INSP_SPRITE_FLIP_X, ids::INSP_SPRITE_FLIP_Y];

/// Os nomes dos nove pontos, por linhas.
pub(crate) const NOMES_DO_PIVOT: [&str; 9] = [
    "card.insp.pivot.top_left",
    "card.insp.pivot.top",
    "card.insp.pivot.top_right",
    "card.insp.pivot.left",
    "card.insp.pivot.center",
    "card.insp.pivot.right",
    "card.insp.pivot.bottom_left",
    "card.insp.pivot.bottom",
    "card.insp.pivot.bottom_right",
];

thread_local! {
    /// Os campos X e Y abertos à mão, com o pivô num dos nove pontos.
    static PIVOT_ABERTO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Os campos X e Y estão à vista? Sempre em Custom; nos nove pontos, só se a mão os abriu.
fn pivot_campos_abertos(f: &slot_image::SpriteFacts) -> bool {
    f.pivot_ponto.is_none() || PIVOT_ABERTO.with(std::cell::Cell::get)
}

pub(crate) fn prepara_campos_do_pivot(store: &mut WidgetStore) {
    let f = slot_image::factos();
    let foco = store.focus_id();
    let mao = store.number_input_drag().map(|d| d.id);
    for (i, id) in ids::INSP_SPRITE_PIVOT_XY.into_iter().enumerate() {
        store.mark_card_field(
            id,
            ph2d_editor_core::widget::card_field::CardFieldSpec {
                step: 1.0,
                scrub: 1.0,
                default: Some(0.0),
            },
        );
        if let Some(f) = f.filter(|_| foco != Some(id) && mao != Some(id)) {
            let v = f64::from(f.pivot_px[i].round());
            if store.number_value(id) != Some(v) {
                store.set_number_value(id, v);
            }
        }
    }
}

/// A grade 3 × 3 do Pivot, o nome do ponto ao lado; devolve o fundo.
fn grade_do_pivot(c: &mut CardCtx<'_>, y: f32, f: slot_image::SpriteFacts) -> f32 {
    let theme = c.ink.theme;
    let casa = m::FIELD_H * 0.5;
    let lado = casa * CASAS_POR_LADO;
    let caixa = Rect::new(c.x, y, lado, lado);
    ph2d_editor_core::paint::fill_rounded_rect(
        c.scene,
        caixa,
        ph2d_tokens::Radius::Sm.px(),
        resolve(ColorToken::Bg2, theme),
    );
    let aceso = f
        .pivot_ponto
        .map(|p| usize::from(p[1]) * 3 + usize::from(p[0]));
    for (i, id) in ids::INSP_SPRITE_PIVOT.into_iter().enumerate() {
        let r = Rect::new(
            c.x + (i % 3) as f32 * casa,
            y + (i / 3) as f32 * casa,
            casa,
            casa,
        );
        let on = aceso == Some(i);
        let d = if on {
            Spacing::Sm.px()
        } else {
            Spacing::Xxs.px()
        };
        let cor = if on {
            ColorToken::Accent
        } else {
            ColorToken::Text3
        };
        ph2d_editor_core::paint::fill_rounded_rect(
            c.scene,
            Rect::new(r.x + (r.w - d) * 0.5, r.y + (r.h - d) * 0.5, d, d),
            d * 0.5,
            resolve(cor, theme),
        );
        c.hit_index.register(id, r);
    }
    let nome = aceso.map_or(tr("card.insp.pivot.custom"), |i| tr(NOMES_DO_PIVOT[i]));
    let tx = c.x + lado + Spacing::Xs.px();
    let tw = (c.w - lado - Spacing::Xs.px()).max(0.0);
    paint_text(
        c.text_system,
        c.scene,
        nome,
        tx,
        y + (lado - TypeToken::Sm.px()) * 0.5,
        TypeToken::Sm.px(),
        tw,
        resolve(ColorToken::Text1, theme),
    );
    c.hit_index
        .register(ids::INSP_SPRITE_PIVOT_NOME, Rect::new(tx, y, tw, lado));
    y + lado + m::FIELD_ROW_GAP
}

/// Flip e Pivot lado a lado. Os ids do Flip continuam caixas de marcar: o clique emite o mesmo `Toggled`.
pub(crate) fn flip_e_pivot(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let aceso = |store: &WidgetStore, id: NodeId| {
        matches!(
            store.checkbox(id).map(|(_, v)| v),
            Some(ph2d_editor_core::widget::CheckboxValue::Checked)
        )
    };
    let segs = [
        Seg {
            id: ids::INSP_SPRITE_FLIP_X,
            icon: Some(IconId::FlipH),
            text: "",
            on: aceso(c.store, ids::INSP_SPRITE_FLIP_X),
        },
        Seg {
            id: ids::INSP_SPRITE_FLIP_Y,
            icon: Some(IconId::FlipV),
            text: "",
            on: aceso(c.store, ids::INSP_SPRITE_FLIP_Y),
        },
    ];
    let [(xa, wa), (xb, wb)] = ph2d_editor_core::panel::cards::corpo::colunas_iguais(c.x, c.w);
    let (x0, w0) = (c.x, c.w);
    (c.x, c.w) = (xa, wa);
    let fim_flip = c.row(
        &[Cell::new(Widget::Icons(&segs))
            .label(tr("card.insp.flip"))
            .key(&FLIP_KEY)],
        y,
    );
    let Some(f) = crate::state::current_inspector_sprite()
        .and_then(|i| slot_image::factos().filter(|f| f.entity_bits == i.entity_bits))
    else {
        (c.x, c.w) = (x0, w0);
        return fim_flip;
    };
    (c.x, c.w) = (xb, wb);
    c.row(
        &[Cell::new(Widget::Readout { text: "" })
            .label(tr("card.insp.pivot"))
            .key(&ids::INSP_SPRITE_PIVOT_XY)],
        y,
    );
    // A grade começa onde a caixa do Flip começa: logo abaixo dos rótulos.
    let topo = fim_flip - m::FIELD_H;
    let fim_grade = grade_do_pivot(c, topo, f);
    (c.x, c.w) = (x0, w0);
    let fim = fim_flip.max(fim_grade);
    if !pivot_campos_abertos(&f) {
        return fim;
    }
    let (gx, gy) = (tr("card.glyph.x"), tr("card.glyph.y"));
    c.row(
        &[
            Cell::new(Widget::Field {
                id: ids::INSP_SPRITE_PIVOT_XY[0],
                glyph: ph2d_editor_core::widget::card_field::Glyph::Text(gx),
            })
            .label(tr("card.insp.pivot_px")),
            Cell::new(Widget::Field {
                id: ids::INSP_SPRITE_PIVOT_XY[1],
                glyph: ph2d_editor_core::widget::card_field::Glyph::Text(gy),
            }),
        ],
        fim,
    )
}

/// Os cliques e os números do Pivot. `true` = consumido.
pub(crate) fn evento_do_pivot(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let pivot = match ev {
        WidgetEvent::Click(id) if id == ids::INSP_SPRITE_PIVOT_NOME => {
            PIVOT_ABERTO.with(|a| a.set(!a.get()));
            return true;
        }
        WidgetEvent::Click(id) => {
            let Some(i) = ids::INSP_SPRITE_PIVOT.iter().position(|o| *o == id) else {
                return false;
            };
            SpritePivot::Ponto([(i % 3) as u8, (i / 3) as u8])
        }
        WidgetEvent::ValueChanged(id) if ids::INSP_SPRITE_PIVOT_XY.contains(&id) => {
            let [a, b] = ids::INSP_SPRITE_PIVOT_XY.map(|n| host.store().number_value(n));
            SpritePivot::Px([a.unwrap_or(0.0) as f32, b.unwrap_or(0.0) as f32])
        }
        _ => return false,
    };
    let Some(sprite) = crate::state::current_inspector_sprite() else {
        return true;
    };
    host.bus_mut().push(EditorAction::ImageUseEdit {
        entity_bits: sprite.entity_bits,
        edit: ImageUseEdit::Pivot(pivot),
    });
    true
}
