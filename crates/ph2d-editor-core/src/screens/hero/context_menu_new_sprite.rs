//! **O diálogo New Sprite** (Add ▸ Pixel Sprite, ADR-0196; o *File ▸ New* do Aseprite) — filho do
//! [`super`] pelo tecto de LOC e irmão do *New Image* na forma: caixa centrada, filas de escolha,
//! CTA acentuado. Width e Height são os campos numéricos da casa, e são a verdade do tamanho: as
//! predefinições enchem-nos, e o segmento aceso deriva deles. Tudo sai das tabelas `ids::NEW_SPRITE_*`,
//! as mesmas que o despacho e o registo leem.

use super::super::context_menu_overlay::{ROW_H, pad_y};
use super::{DIALOG_W, paint_choice_button};
use crate::ids;
use crate::interaction::{HitIndex, WidgetStore};
use crate::paint::{paint_text, resolve};
use crate::panel::cards::ferramenta::campo;
use crate::panel::cards::{CardCtx, Cell, Seg, Widget};
use crate::panel::rows::RowCtx;
use crate::widget::card_field::CardFieldSpec;
use crate::widget::card_ink::CardInk;
use crate::widget::{Button, paint_button};
use crate::zones::Rect;
use ph2d_a11y::NodeId;
use ph2d_card_stack::metrics as m;
use ph2d_i18n::{tr, tr_with};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Spacing, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Título · rótulo + fila dos lados · rótulo + fila dos ecrãs · Width · Height · rótulo + fila do modo · rótulo + fila
/// do fundo · Cancel | OK.
const ROWS: f32 = 12.0; // LITERAL-PX-OK: uma CONTAGEM de filas, não uma medida

/// As filas dos tamanhos, `(id, texto, aceso)`: os lados, depois os ecrãs com o Custom. Pintar e medir leem estas.
fn filas_de_tamanho(store: &WidgetStore) -> [Vec<(NodeId, String, bool)>; 2] {
    let aceso = store.new_sprite_size_lit();
    let lados = ids::NEW_SPRITE_SIZES
        .iter()
        .map(|(s, id)| (*id, s.to_string(), *id == aceso))
        .collect();
    let mut ecras: Vec<_> = ids::NEW_SPRITE_SCREENS
        .iter()
        .map(|([w, h], id)| {
            let t = tr_with("pixel.new_sprite.screen_wh", &[("w", w), ("h", h)]);
            (*id, t, *id == aceso)
        })
        .collect();
    ecras.push((
        ids::NEW_SPRITE_CUSTOM,
        tr("pixel.new_sprite.custom").to_owned(),
        ids::NEW_SPRITE_CUSTOM == aceso,
    ));
    [lados, ecras]
}

pub(in crate::screens::hero) fn paint_new_sprite_dialog(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    viewport: Rect,
) {
    let gap = Spacing::Xs.px();
    let superficie = crate::widget::lista_flutuante::Superficie::de(theme);
    let _tinta = superficie.veste();
    if let Some(ink) = superficie.tinta() {
        // A largura sai do texto medido das filas; a altura, da função que pinta, num ensaio descartável: uma conta só.
        let filas = filas_de_tamanho(store);
        let segs: Vec<Vec<Seg<'_>>> = filas
            .iter()
            .map(|f| {
                f.iter()
                    .map(|(id, t, on)| Seg {
                        id: *id,
                        icon: None,
                        text: t,
                        on: *on,
                    })
                    .collect()
            })
            .collect();
        let dialog_w = super::caixa::largura_das_filas(
            &mut super::caixa::Alvo {
                scene: &mut *scene,
                text_system: &mut *text_system,
                hit_index: &mut *hit_index,
                store,
                viewport,
            },
            ink,
            DIALOG_W,
            &[&segs[0], &segs[1]],
        );
        let (cw, cx0) = (dialog_w - m::CARD_PAD_X * 2.0, m::CARD_PAD_X);
        let (mut ensaio, mut alvos) = (VectorScene::new(), HitIndex::default());
        let fim = corpo_em_cartao(
            &mut ensaio,
            text_system,
            &mut alvos,
            store,
            ink,
            cx0,
            cw,
            0.0,
        );
        let total_h = fim + m::CARD_PAD_BOTTOM;
        let x = (viewport.x + (viewport.w - dialog_w) * 0.5).max(viewport.x);
        let y = (viewport.y + (viewport.h - total_h) * 0.5).max(viewport.y);
        superficie.moldura_de_caixa(scene, Rect::new(x, y, dialog_w, total_h));
        corpo_em_cartao(scene, text_system, hit_index, store, ink, x + cx0, cw, y);
        return;
    }
    let total_h = pad_y() * 2.0 + ROW_H * ROWS + ph2d_tokens::control_gap_px() * (ROWS - 1.0);
    // A largura sai dos rótulos medidos das filas de tamanho.
    let dialog_w = filas_de_tamanho(store)
        .iter()
        .map(|f| {
            let rotulos: Vec<&str> = f.iter().map(|(_, t, _)| t.as_str()).collect();
            super::fila_de_botoes_pede(text_system, &rotulos) + Spacing::Md.px() * 2.0
        })
        .fold(DIALOG_W, f32::max);
    let rect_x = (viewport.x + (viewport.w - dialog_w) * 0.5).max(viewport.x);
    let rect_y = (viewport.y + (viewport.h - total_h) * 0.5).max(viewport.y);
    let rect = Rect::new(rect_x, rect_y, dialog_w, total_h);
    superficie.moldura_de_caixa(scene, rect);
    let inner_x = rect.x + Spacing::Md.px();
    let inner_w = rect.w - Spacing::Md.px() * 2.0;
    let font = TypeToken::Sm.px();
    let label = |scene: &mut VectorScene, ts: &mut TextSystem, key: &str, y: f32, token| {
        paint_text(
            ts,
            scene,
            tr(key),
            inner_x,
            y + (ROW_H - font) * 0.5,
            font,
            inner_w,
            resolve(token, theme),
        );
    };
    let mut y = rect.y + pad_y();
    label(
        scene,
        text_system,
        "pixel.new_sprite.title",
        y,
        ColorToken::Text1,
    );
    y += ph2d_tokens::row_pitch_px();
    let [lados, ecras] = filas_de_tamanho(store);
    for (chave, fila) in [
        ("pixel.new_sprite.size_px", lados),
        ("pixel.new_sprite.screen_px", ecras),
    ] {
        label(scene, text_system, chave, y, ColorToken::Text3);
        y += ph2d_tokens::row_pitch_px();
        let opts: Vec<(NodeId, &str, bool)> =
            fila.iter().map(|(i, t, o)| (*i, t.as_str(), *o)).collect();
        y = choice_row(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            &opts,
            inner_x,
            inner_w,
            y,
        );
    }

    let mut r = RowCtx {
        scene: &mut *scene,
        text_system: &mut *text_system,
        store,
        hit_index: &mut *hit_index,
        theme,
        inner_x,
        inner_w,
        row_h: ROW_H,
        row_gap: ph2d_tokens::control_gap_px(),
        font,
        open_fold: None,
    };
    y = r.labeled_number_field_unit(
        tr("pixel.new_sprite.width"),
        ids::NEW_SPRITE_WIDTH,
        1.0,
        Some(crate::widget::Unit::Px),
        y,
    );
    y = r.labeled_number_field_unit(
        tr("pixel.new_sprite.height"),
        ids::NEW_SPRITE_HEIGHT,
        1.0,
        Some(crate::widget::Unit::Px),
        y,
    );

    let choice = store.new_sprite_choice();
    label(
        scene,
        text_system,
        "pixel.new_sprite.color_mode",
        y,
        ColorToken::Text3,
    );
    y += ph2d_tokens::row_pitch_px();
    let modes: Vec<(NodeId, &str, bool)> = ids::NEW_SPRITE_MODES
        .iter()
        .map(|(m, id, k)| (*id, tr(k), *m == choice.mode))
        .collect();
    y = choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        &modes,
        inner_x,
        inner_w,
        y,
    );
    label(
        scene,
        text_system,
        "pixel.new_sprite.background",
        y,
        ColorToken::Text3,
    );
    y += ph2d_tokens::row_pitch_px();
    let bgs: Vec<(NodeId, &str, bool)> = ids::NEW_SPRITE_BGS
        .iter()
        .map(|(b, id, k)| (*id, tr(k), *b == choice.background))
        .collect();
    y = choice_row(
        scene,
        text_system,
        theme,
        hit_index,
        store,
        &bgs,
        inner_x,
        inner_w,
        y,
    );

    // Cancel | OK — o OK à direita e acentuado (Enter), o Cancel à esquerda (Esc).
    let half = ((inner_w - gap) * 0.5).max(1.0);
    for (id, key, x, accent) in [
        (
            ids::NEW_SPRITE_CANCEL,
            "pixel.new_sprite.cancel",
            inner_x,
            false,
        ),
        (
            ids::NEW_SPRITE_OK,
            "pixel.new_sprite.ok",
            inner_x + half + gap,
            true,
        ),
    ] {
        let b_rect = Rect::new(x, y, half, ROW_H);
        hit_index.register(id, b_rect);
        let b = Button::new(id, tr(key)).visual(store.button_visual(id));
        let b = if accent { b.accent() } else { b };
        paint_button(&b, b_rect, scene, text_system, theme);
    }
}

/// O corpo em linhas de cartão: a unidade no rótulo, a escolha segmentada, os mesmos ids e estado.
/// Devolve o fim do conteúdo; `topo` é o topo da caixa.
#[allow(clippy::too_many_arguments)]
fn corpo_em_cartao(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    ink: CardInk,
    x: f32,
    w: f32,
    topo: f32,
) -> f32 {
    let titulo_y = topo + m::CARD_PAD_TOP;
    paint_text(
        text_system,
        scene,
        tr("pixel.new_sprite.title"),
        x,
        titulo_y + crate::paint::folga_do_texto(m::FIELD_H, m::VALUE_PX),
        m::VALUE_PX,
        w,
        ink.color(ColorToken::KanbanCardFg),
    );
    for id in ids::NEW_SPRITE_FIELDS {
        let spec = CardFieldSpec {
            step: 1.0,
            scrub: 0.25, // LITERAL-PX-OK: células por px de arrasto, o `set_number_drag_rate` da abertura
            default: None,
        };
        store.mark_card_field_at_paint(id, spec);
    }
    let choice = store.new_sprite_choice();
    let filas = filas_de_tamanho(store);
    let [seg_lados, seg_ecras]: [Vec<Seg<'_>>; 2] = [&filas[0], &filas[1]].map(|f| {
        f.iter()
            .map(|(id, t, on)| Seg {
                id: *id,
                icon: None,
                text: t,
                on: *on,
            })
            .collect()
    });
    let segs = |linhas: Vec<(NodeId, &'static str, bool)>| -> Vec<Seg<'static>> {
        linhas
            .into_iter()
            .map(|(id, k, on)| Seg {
                id,
                icon: None,
                text: tr(k),
                on,
            })
            .collect()
    };
    let modes = segs(
        ids::NEW_SPRITE_MODES
            .iter()
            .map(|(v, id, k)| (*id, *k, *v == choice.mode))
            .collect(),
    );
    let bgs = segs(
        ids::NEW_SPRITE_BGS
            .iter()
            .map(|(v, id, k)| (*id, *k, *v == choice.background))
            .collect(),
    );
    let mut c = CardCtx::flutuante(scene, text_system, store, hit_index, ink, x, w);
    let y = titulo_y + m::FIELD_H;
    let y = c.row(
        &[Cell::new(Widget::Choice(&seg_lados)).label(tr("pixel.new_sprite.size_px"))],
        y,
    );
    let y = c.row(
        &[Cell::new(Widget::Choice(&seg_ecras)).label(tr("pixel.new_sprite.screen_px"))],
        y,
    );
    let y = c.row(
        &[
            campo(ids::NEW_SPRITE_WIDTH, tr("pixel.new_sprite.width_px")),
            campo(ids::NEW_SPRITE_HEIGHT, tr("pixel.new_sprite.height_px")),
        ],
        y,
    );
    let y = c.row(
        &[Cell::new(Widget::Choice(&modes)).label(tr("pixel.new_sprite.color_mode"))],
        y,
    );
    let y = c.row(
        &[Cell::new(Widget::Choice(&bgs)).label(tr("pixel.new_sprite.background"))],
        y,
    );
    c.row(
        &[
            Cell::new(Widget::Button {
                id: ids::NEW_SPRITE_CANCEL,
                text: tr("pixel.new_sprite.cancel"),
                primary: false,
            }),
            Cell::new(Widget::Button {
                id: ids::NEW_SPRITE_OK,
                text: tr("pixel.new_sprite.ok"),
                primary: true,
            }),
        ],
        y,
    )
}

/// Uma fila de escolhas de largura igual (acentuada a escolhida).
#[allow(clippy::too_many_arguments)]
fn choice_row(
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
    hit_index: &mut HitIndex,
    store: &WidgetStore,
    opts: &[(NodeId, &str, bool)],
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    let gap = Spacing::Xs.px();
    #[expect(clippy::cast_precision_loss, reason = "três escolhas")]
    let n = opts.len() as f32;
    let bw = ((w - gap * (n - 1.0)) / n).max(1.0);
    for (i, (id, lbl, on)) in opts.iter().enumerate() {
        #[expect(clippy::cast_precision_loss, reason = "três escolhas")]
        let bx = x + i as f32 * (bw + gap);
        let r = Rect::new(bx, y, bw, ROW_H);
        paint_choice_button(
            scene,
            text_system,
            theme,
            hit_index,
            store,
            *id,
            lbl,
            r,
            *on,
        );
    }
    y + ph2d_tokens::row_pitch_px()
}
