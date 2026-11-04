//! ⭐ **O «Stroke settings» e as propriedades de uma camada, em cartão** (fatia G2): os mesmos ids,
//! condições e popovers de `paint_stroke_settings`, `paint_dashes`, `paint_markers`, `paint_expand` e
//! `paint_stack_rows`.

use crate::cards_tinta::{aberto, campo, escolha, menu, seg, seg_icone, token};
use crate::{ids, state};
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::panel::TextKey;
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Widget};
use ph2d_editor_core::widget::card_field::Glyph;
use ph2d_editor_core::widget::key_mark::Rotulo;
use ph2d_i18n::{tr, tr_with};
use ph2d_tool_vector::ids as vids;
use ph2d_tool_vector::{StrokeCap, StrokeJoin, StyleFields as F, VectorStyleSnapshot, params};
use ph2d_vec_scene::StrokeAlign;

fn position(c: &mut CardCtx<'_>, canal: &[NodeId; 3], ligado: [bool; 3], y: f32) -> f32 {
    let segs = [
        seg_icone(canal[0], IconId::StrokeInside, ligado[0]),
        seg_icone(canal[1], IconId::StrokeCenter, ligado[1]),
        seg_icone(canal[2], IconId::StrokeOutside, ligado[2]),
    ];
    c.row(
        &[Cell::new(Widget::Choice(&segs))
            .label(tr("panel.vector.stroke.align"))
            .key(canal)],
        y,
    )
}

/// ⭐ **O «Stroke settings»**: Position, o tipo, Cap e Join com ícones, tokens, tracejado, pontas e o
/// perfil de largura.
pub(crate) fn stroke_settings(c: &mut CardCtx<'_>, snap: &VectorStyleSnapshot, y: f32) -> f32 {
    const ALINHA: [NodeId; 3] = [
        vids::VECTOR_ALIGN_INNER,
        vids::VECTOR_ALIGN_CENTRE,
        vids::VECTOR_ALIGN_OUTER,
    ];
    const TIPO: &[NodeId] = &[
        vids::VECTOR_STROKE_KIND_SOLID,
        vids::VECTOR_STROKE_KIND_PATTERN,
        vids::VECTOR_STROKE_KIND_BRUSH,
    ];
    const CAP: &[NodeId] = &[
        vids::VECTOR_CAP_BUTT,
        vids::VECTOR_CAP_ROUND,
        vids::VECTOR_CAP_SQUARE,
    ];
    const JOIN: &[NodeId] = &[
        vids::VECTOR_JOIN_MITER,
        vids::VECTOR_JOIN_ROUND,
        vids::VECTOR_JOIN_BEVEL,
    ];
    let mut y = y;
    if state::open_layer().is_none() {
        let a = |k| snap.agrees(F::ALIGN) && snap.align == k;
        y = position(
            c,
            &ALINHA,
            [
                a(StrokeAlign::Inner),
                a(StrokeAlign::Centre),
                a(StrokeAlign::Outer),
            ],
            y,
        );
    }
    if let Some(k) = state::stroke_paint_kind() {
        let segs = [
            seg(
                TIPO[0],
                tr("panel.vector.stroke.solid"),
                k == state::StrokePaintKind::Solid,
            ),
            seg(
                TIPO[1],
                tr("panel.vector.stroke.pattern"),
                k == state::StrokePaintKind::Pattern,
            ),
            seg(
                TIPO[2],
                tr("panel.vector.stroke.brush"),
                k == state::StrokePaintKind::Brush,
            ),
        ];
        y = escolha(c, tr("panel.vector.stroke.type"), TIPO, &segs, y);
    }
    let cap = |v| snap.agrees(F::CAP) && snap.cap == v;
    let caps = [
        seg_icone(CAP[0], IconId::CardCapButt, cap(StrokeCap::Butt)),
        seg_icone(CAP[1], IconId::CardCapRound, cap(StrokeCap::Round)),
        seg_icone(CAP[2], IconId::CardCapSquare, cap(StrokeCap::Square)),
    ];
    let join = |v| snap.agrees(F::JOIN) && snap.join == v;
    let joins = [
        seg_icone(JOIN[0], IconId::CardJoinMiter, join(StrokeJoin::Miter)),
        seg_icone(JOIN[1], IconId::CardJoinRound, join(StrokeJoin::Round)),
        seg_icone(JOIN[2], IconId::CardJoinBevel, join(StrokeJoin::Bevel)),
    ];
    y = c.row(
        &[
            Cell::new(Widget::Choice(&caps))
                .label(tr("panel.vector.stroke.cap"))
                .key(CAP),
            Cell::new(Widget::Choice(&joins))
                .label(tr("panel.vector.stroke.join"))
                .key(JOIN),
        ],
        y,
    );
    if let Some(on) = state::stroke_above_effects() {
        y = c.check(
            crate::ids::VECTOR_STROKE_ABOVE_EFFECTS,
            Rotulo::vista(TextKey::new("panel.vector.stroke.above_effects")),
            on,
            y,
        );
    }
    if let Some(b) = state::token_bindings()
        .filter(|_| state::stroke_present() == Some(true) && state::open_layer().is_none())
    {
        y = token(
            c,
            "panel.vector.token.weight",
            crate::ids::VECTOR_TOKEN_WIDTH,
            b.width.as_deref(),
            y,
        );
        y = token(
            c,
            "panel.vector.token.color",
            crate::ids::VECTOR_TOKEN_STROKE,
            b.stroke.as_deref(),
            y,
        );
    }
    y = tracejado(c, snap, y);
    y = pontas(c, snap, y);
    if state::current_selection_count() > 0 {
        y = perfil(c, y);
    }
    y
}

/// Uma linha por par, e a do par seguinte só quando o anterior traça (`paint_dashes::visible_dash_rows`).
fn tracejado(c: &mut CardCtx<'_>, snap: &VectorStyleSnapshot, y: f32) -> f32 {
    let visiveis = crate::paint_dashes::visible_dash_rows(&snap.dash);
    let mut y = y;
    for k in 0..visiveis {
        let canal = [vids::VECTOR_DASH_IDS[k], vids::VECTOR_GAP_IDS[k]];
        let nome = tr_with("panel.vector.stroke.dash_gap", &[("n", &(k + 1))]);
        y = c.row(
            &[
                Cell::new(campo(canal[0], Glyph::Text(tr("card.vector.glyph.dash")))).label(&nome),
                Cell::new(campo(canal[1], Glyph::Text(tr("card.vector.glyph.gap")))).key(&canal),
            ],
            y,
        );
    }
    y
}

/// As duas pontas numa linha, o tamanho de cada uma, o arredondamento e «Both Ends».
fn pontas(c: &mut CardCtx<'_>, snap: &VectorStyleSnapshot, y: f32) -> f32 {
    let (dd_a, dd_b) = (
        crate::paint_markers::marker_dd_id(0),
        crate::paint_markers::marker_dd_id(1),
    );
    let face = |slot: usize, campo: F| {
        if snap.agrees(campo) {
            let m = if slot == params::MARKER_SLOT_END {
                snap.marker_end
            } else {
                snap.marker_start
            };
            crate::nomes_do_motor::ponta(m.label())
        } else {
            tr("panel.vector.mixed")
        }
    };
    let (ka, kb) = ([dd_a], [dd_b]);
    let y = c.row(
        &[
            Cell::new(Widget::Dropdown {
                id: dd_a,
                text: face(0, F::MARKER_START),
                options: &[],
            })
            .label(tr("panel.vector.marker.start"))
            .key(&ka),
            Cell::new(Widget::Dropdown {
                id: dd_b,
                text: face(1, F::MARKER_END),
                options: &[],
            })
            .label(tr("panel.vector.marker.end"))
            .key(&kb),
        ],
        y,
    );
    for (slot, id) in [(0, dd_a), (1, dd_b)] {
        if let Some(chip) = aberto(c, id) {
            state::set_pending_marker_dd(Some((slot, chip)));
        }
    }
    let (tam, round, ambas) = (
        crate::nomes_do_motor::marcador(params::MARKER_SCALE.label),
        crate::nomes_do_motor::marcador(params::MARKER_ROUND.label),
        crate::nomes_do_motor::marcador(params::BOTH_ENDS.label),
    );
    const TAM: &[NodeId] = &vids::VECTOR_MARKER_SCALE_IDS;
    const RO: &[NodeId] = &[vids::VECTOR_MARKER_ROUND];
    let y = c.row(
        &[
            Cell::new(campo(TAM[0], Glyph::Text(tr("card.vector.glyph.start")))).label(tam),
            Cell::new(campo(TAM[1], Glyph::Text(tr("card.vector.glyph.end")))).key(TAM),
        ],
        y,
    );
    c.row(
        &[
            Cell::new(campo(RO[0], Glyph::Text(tr("card.vector.glyph.round"))))
                .label(round)
                .key(RO),
            Cell::new(Widget::Button {
                id: vids::VECTOR_MARKER_BOTH,
                text: crate::nomes_do_motor::marcador(params::both_ends_label(snap.both_ends())),
                primary: false,
            })
            .label(ambas),
        ],
        y,
    )
}

/// O catálogo de perfis, os quatro números que o autoram, e o verbo que o consolida.
fn perfil(c: &mut CardCtx<'_>, y: f32) -> f32 {
    use crate::paint_sections::expand::SLIDERS;
    let ativo = state::current_selection_profile();
    let segs: Vec<_> = ph2d_vec_scene::WIDTH_PRESETS
        .iter()
        .enumerate()
        .take(crate::ids::MAX_WIDTH_PRESETS)
        .map(|(i, p)| {
            seg(
                crate::ids::vector_width_preset_id(i),
                tr(p.key),
                ativo == Some(i),
            )
        })
        .collect();
    let y = escolha(c, tr("panel.vector.expand.profile"), &[], &segs, y);
    let mut y = y;
    for par in SLIDERS.chunks(2) {
        let barras: Vec<[NodeId; 1]> = par.iter().map(|(_, s, _)| [*s]).collect();
        let nomes: Vec<&str> = par.iter().map(|(k, _, _)| k.tr()).collect();
        let cells: Vec<Cell<'_>> = par
            .iter()
            .enumerate()
            .map(|(i, (_, _, chip))| {
                Cell::new(campo(
                    *chip,
                    Glyph::Text(ph2d_editor_core::panel::cards::corpo::letra_da_grandeza(
                        nomes[i],
                    )),
                ))
                .label(nomes[i])
                .key(&barras[i])
            })
            .collect();
        y = c.row(&cells, y);
    }
    c.full(
        None,
        Widget::Button {
            id: vids::VECTOR_EXPAND_POWER_STROKE,
            text: tr("panel.vector.expand.apply_power_stroke"),
            primary: false,
        },
        y,
    )
}

/// ⭐ A Position de um contorno EXTRA: os glifos da base, com os ids da camada aberta.
pub(crate) fn layer_position(c: &mut CardCtx<'_>, row: &state::PaintRow, y: f32) -> f32 {
    const ALINHA: [NodeId; 3] = [
        ids::VECTOR_PAINT_ALIGN_INNER,
        ids::VECTOR_PAINT_ALIGN_CENTRE,
        ids::VECTOR_PAINT_ALIGN_OUTER,
    ];
    let a = |k| !row.mixed.align && row.align == k;
    position(
        c,
        &ALINHA,
        [
            a(StrokeAlign::Inner),
            a(StrokeAlign::Centre),
            a(StrokeAlign::Outer),
        ],
        y,
    )
}

/// As propriedades da camada ABERTA: onde ela desenha, o offset de CAD, a quina dele, a mistura.
pub(crate) fn layer_props(c: &mut CardCtx<'_>, row: &state::PaintRow, y: f32) -> f32 {
    const XY: &[NodeId] = &[crate::ids::VECTOR_PAINT_DX, crate::ids::VECTOR_PAINT_DY];
    const GROW: &[NodeId] = &[crate::ids::VECTOR_PAINT_DILATE];
    const QUINA: &[NodeId] = &[
        ids::VECTOR_PAINT_JOIN_MITER,
        ids::VECTOR_PAINT_JOIN_ROUND,
        ids::VECTOR_PAINT_JOIN_BEVEL,
    ];
    let (desloca, cresce) = if state::length_suffix() == "px" {
        (tr("card.vector.shift_px"), tr("card.vector.grow_px"))
    } else {
        (tr("card.vector.shift_m"), tr("card.vector.grow_m"))
    };
    let y = c.row(
        &[
            Cell::new(campo(XY[0], Glyph::Text(tr("card.glyph.x")))).label(desloca),
            Cell::new(campo(XY[1], Glyph::Text(tr("card.glyph.y")))).key(XY),
        ],
        y,
    );
    let mut y = c.row(
        &[Cell::new(campo(GROW[0], Glyph::Icon(IconId::CardWeight)))
            .label(cresce)
            .key(GROW)],
        y,
    );
    // A quina só com o offset armado: sem esquina a formar, três ícones que não mudam nada.
    if row.dilate != 0.0 {
        let j = |k: u8| !row.mixed.dilate_join && row.dilate_join == k;
        let segs = [
            seg_icone(QUINA[0], IconId::CardJoinMiter, j(0)),
            seg_icone(QUINA[1], IconId::CardJoinRound, j(1)),
            seg_icone(QUINA[2], IconId::CardJoinBevel, j(2)),
        ];
        y = c.row(
            &[Cell::new(Widget::Choice(&segs))
                .label(tr("panel.vector.paint.join"))
                .key(QUINA)],
            y,
        );
    }
    let face = if row.mixed.blend {
        tr("panel.vector.mixed")
    } else {
        tr(row.blend.label_key())
    };
    let (y, chip) = menu(
        c,
        crate::ids::VECTOR_PAINT_BLEND,
        tr("panel.vector.paint.blend"),
        face,
        y,
    );
    if chip.is_some() {
        state::set_pending_paint_blend_dd(chip);
    }
    y
}
