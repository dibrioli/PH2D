//! ⭐ **Appearance e o painel do preenchimento em cartões** (fatia G2, `docs/CartoesUI/12_VECTOR_PLANO.md`):
//! os mesmos ids e condições do corpo de antes (`paint_appearance`, `paint_fill_type`), com a cara do
//! cartão. Aqui também as marcas e as sementes de tudo o que a fatia pinta como campo: o corpo antigo
//! recebia o valor ao pintar, e o campo do cartão lê o store.

use crate::{ids, state};
use ph2d_a11y::NodeId;
use ph2d_editor_core::IconId;
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::panel::cards::{CardCtx, Cell, Seg, Widget, corpo};
use ph2d_editor_core::widget::card_field::{CardFieldSpec, Glyph};
use ph2d_editor_core::widget::card_ink;
use ph2d_editor_core::zones::Rect;
use ph2d_i18n::tr;
use ph2d_tool_vector::StyleFields as F;
use ph2d_tool_vector::ids as vids;
use state::{FillKind, PathFillRule};

const CEM_POR_CENTO: f64 = 100.0; // LITERAL-PX-OK: neutro de um campo em %, não geometria
const PASSO_MEDIO: f64 = 0.1; // LITERAL-PX-OK: passo do valor (fator, px), não geometria
const PASSO_FINO: f64 = 0.01; // LITERAL-PX-OK: passo do valor (fração), não geometria

pub(crate) fn spec(step: f64, scrub: f64, default: Option<f64>) -> CardFieldSpec {
    CardFieldSpec {
        step,
        scrub,
        default,
    }
}

pub(crate) fn dica(store: &mut WidgetStore, id: NodeId, chave: &'static str) {
    if store.tooltip_for(id).is_none() {
        store.set_tooltip(id, tr(chave));
    }
}

/// Semeia o campo `id`, salvo com a mão nele (foco ou scrub).
pub(crate) fn semeia(store: &mut WidgetStore, id: NodeId, valor: f64, misto: bool) {
    if store.focus_id() == Some(id) || store.number_input_drag().map(|d| d.id) == Some(id) {
        return;
    }
    if misto {
        store.blank_number_input(id);
    } else {
        store.set_number_value(id, valor);
    }
}

/// O passo de um comprimento em mundo: 1 px, ou 1 cm no projecto em metros.
fn passo_do_comprimento() -> f64 {
    if state::length_suffix() == "px" {
        1.0
    } else {
        PASSO_FINO
    }
}

/// Com o store MUTÁVEL, antes de pintar: marcas, dicas e sementes das secções de G2.
pub(crate) fn prepare(store: &mut WidgetStore) {
    use crate::paint_dashes::DASH_STEP;
    use ph2d_tool_vector::params;
    let p = passo_do_comprimento();
    for (id, step, scrub, default) in [
        (ids::VECTOR_OBJ_OPACITY_NUM, 1.0, 1.0, Some(CEM_POR_CENTO)),
        (vids::VECTOR_FILL_OPACITY_NUM, 1.0, 1.0, Some(CEM_POR_CENTO)),
        (
            vids::VECTOR_STROKE_OPACITY_NUM,
            1.0,
            1.0,
            Some(CEM_POR_CENTO),
        ),
        (vids::VECTOR_WIDTH_NUM, 1.0, PASSO_MEDIO, None),
        (crate::ids::VECTOR_PAINT_DX, p, p, Some(0.0)),
        (crate::ids::VECTOR_PAINT_DY, p, p, Some(0.0)),
        (crate::ids::VECTOR_PAINT_DILATE, p, p, Some(0.0)),
        (vids::VECTOR_GRAD_ANGLE_NUM, 1.0, 1.0, Some(0.0)),
        (
            vids::VECTOR_GRAD_INFLUENCE_NUM,
            PASSO_MEDIO,
            PASSO_FINO,
            Some(1.0),
        ),
        (
            vids::VECTOR_GRAD_JITTER_NUM,
            PASSO_FINO,
            PASSO_FINO,
            Some(0.0),
        ),
        (
            vids::VECTOR_MARKER_SCALE,
            params::MARKER_SCALE.step,
            params::MARKER_SCALE.step,
            None,
        ),
        (
            vids::VECTOR_MARKER_SCALE_END,
            params::MARKER_SCALE.step,
            params::MARKER_SCALE.step,
            None,
        ),
        (
            vids::VECTOR_MARKER_ROUND,
            params::MARKER_ROUND.step,
            params::MARKER_ROUND.step,
            None,
        ),
    ] {
        store.mark_card_field(id, spec(step, scrub, default));
    }
    for id in vids::VECTOR_DASH_IDS
        .into_iter()
        .chain(vids::VECTOR_GAP_IDS)
    {
        store.mark_card_field(id, spec(DASH_STEP, DASH_STEP, None));
    }
    for (_, _, chip) in crate::paint_sections::expand::SLIDERS {
        store.mark_card_field(chip, spec(PASSO_FINO, PASSO_FINO, None));
    }
    for i in 0..ph2d_vec_scene::MAX_PAINT_LAYERS {
        let (op, w) = (
            ids::vector_paint_row_opacity_id(i),
            ids::vector_paint_row_width_id(i),
        );
        store.mark_card_field(op, spec(1.0, 1.0, Some(CEM_POR_CENTO)));
        store.mark_card_field(w, spec(1.0, 1.0, None));
        dica(
            store,
            ids::vector_paint_row_id(i),
            "card.vector.paint_settings",
        );
        dica(
            store,
            ids::vector_paint_swatch_id(i),
            "card.vector.paint_color",
        );
        dica(
            store,
            ids::vector_paint_eye_id(i),
            "card.vector.paint_visible",
        );
        dica(
            store,
            ids::vector_paint_del_id(i),
            "card.vector.paint_remove",
        );
        dica(store, ids::vector_paint_up_id(i), "card.vector.paint_up");
        dica(
            store,
            ids::vector_paint_down_id(i),
            "card.vector.paint_down",
        );
    }
    for (id, chave) in [
        (ids::VECTOR_FILL_ROW, "card.vector.paint_settings"),
        (ids::VECTOR_STROKE_SETTINGS, "card.vector.paint_settings"),
        (vids::VECTOR_FILL_SWATCH, "card.vector.paint_color"),
        (vids::VECTOR_STROKE_SWATCH, "card.vector.paint_color"),
        (ids::VECTOR_FILL_EYE, "card.vector.paint_visible"),
        (ids::VECTOR_STROKE_EYE, "card.vector.paint_visible"),
        (ids::VECTOR_FILL_PRESENT, "card.vector.paint_remove"),
        (ids::VECTOR_STROKE_PRESENT, "card.vector.paint_remove"),
        (ids::VECTOR_FILL_PROPS_CLOSE, "card.vector.close_settings"),
        (ids::VECTOR_STROKE_PROPS_CLOSE, "card.vector.close_settings"),
        (ids::VECTOR_PAINT_PROPS_CLOSE, "card.vector.close_settings"),
        (vids::VECTOR_CAP_BUTT, "card.insp.cap_butt"),
        (vids::VECTOR_CAP_ROUND, "card.insp.cap_round"),
        (vids::VECTOR_CAP_SQUARE, "card.insp.cap_square"),
        (vids::VECTOR_JOIN_MITER, "card.insp.join_miter"),
        (vids::VECTOR_JOIN_ROUND, "card.insp.join_round"),
        (vids::VECTOR_JOIN_BEVEL, "card.insp.join_bevel"),
        (ids::VECTOR_PAINT_JOIN_MITER, "card.insp.join_miter"),
        (ids::VECTOR_PAINT_JOIN_ROUND, "card.insp.join_round"),
        (ids::VECTOR_PAINT_JOIN_BEVEL, "card.insp.join_bevel"),
        (vids::VECTOR_ALIGN_INNER, "card.insp.align_inside"),
        (vids::VECTOR_ALIGN_CENTRE, "card.insp.align_center"),
        (vids::VECTOR_ALIGN_OUTER, "card.insp.align_outside"),
        (ids::VECTOR_PAINT_ALIGN_INNER, "card.insp.align_inside"),
        (ids::VECTOR_PAINT_ALIGN_CENTRE, "card.insp.align_center"),
        (ids::VECTOR_PAINT_ALIGN_OUTER, "card.insp.align_outside"),
    ] {
        dica(store, id, chave);
    }
    semeia_as_linhas(store);
    crate::cards_fx::prepare(store);
}

/// O que o corpo antigo recebia ao pintar: as linhas da pilha e os números ligados a uma barra.
fn semeia_as_linhas(store: &mut WidgetStore) {
    let snap = state::current_snapshot();
    let pct = |alfa: u8| f64::from(alfa) * CEM_POR_CENTO / 255.0; // LITERAL-PX-OK: byte→percent
    // Um percentual mostra-se inteiro: `0.65 × 100` lia-se «65.000».
    for (id, v, misto) in [
        (
            vids::VECTOR_FILL_OPACITY_NUM,
            pct(snap.fill[3]).round(),
            !snap.agrees(F::FILL_A),
        ),
        (
            vids::VECTOR_STROKE_OPACITY_NUM,
            pct(snap.stroke[3]).round(),
            !snap.agrees(F::STROKE_A),
        ),
        (
            vids::VECTOR_WIDTH_NUM,
            snap.stroke_width_px,
            !snap.agrees(F::WIDTH),
        ),
    ] {
        semeia(store, id, v, misto);
    }
    if let Some(a) = state::current_appearance() {
        for (i, r) in a.layers.iter().enumerate() {
            let fracao = (f64::from(r.opacity) * CEM_POR_CENTO).round();
            semeia(
                store,
                ids::vector_paint_row_opacity_id(i),
                fracao,
                r.mixed.opacity,
            );
            semeia(
                store,
                ids::vector_paint_row_width_id(i),
                r.width,
                r.mixed.width,
            );
        }
    }
    if let Some(g) = state::current_grad_angle() {
        semeia(store, vids::VECTOR_GRAD_ANGLE_NUM, g.round(), false);
    }
    if let Some(v) = state::current_grad_influence() {
        semeia(store, vids::VECTOR_GRAD_INFLUENCE_NUM, v, false);
    }
    if let Some(v) = state::current_grad_jitter() {
        semeia(store, vids::VECTOR_GRAD_JITTER_NUM, v, false);
    }
    let trilhos = crate::paint_sections::expand::live_tracks(store);
    for (i, (_, _, chip)) in crate::paint_sections::expand::SLIDERS.iter().enumerate() {
        let v = if i == 3 {
            f64::from(trilhos[i])
        } else {
            ph2d_tool_vector::params::slider_to_wprofile(trilhos[i])
        };
        semeia(store, *chip, v, false);
    }
}

pub(crate) fn campo<'a>(id: NodeId, glyph: Glyph<'a>) -> Widget<'a> {
    Widget::Field { id, glyph }
}

/// A lista suspensa deste `id` está aberta? Lembra a tinta do cartão para o popover (ele pinta-se
/// depois, fora do corpo) e devolve a caixa, onde ele se ancora.
pub(crate) fn aberto(c: &mut CardCtx<'_>, id: NodeId) -> Option<Rect> {
    let open = matches!(
        c.store.get(id),
        Some(InteractiveState::Dropdown { open: true, .. })
    );
    if !open {
        return None;
    }
    let _ = card_ink::troca_popover(Some((id, c.ink)));
    c.hit_index.rect_for(id)
}

/// Uma linha `rótulo` + a caixa de um menu com a face `face`. A abertura é a do `Dropdown` de sempre:
/// o despacho, o light-dismiss e as opções não mudam. Devolve o fundo e a caixa, se aberto.
pub(crate) fn menu(
    c: &mut CardCtx<'_>,
    id: NodeId,
    rotulo: &str,
    face: &str,
    y: f32,
) -> (f32, Option<Rect>) {
    let k = [id];
    let y = c.row(
        &[Cell::new(Widget::Dropdown {
            id,
            text: face,
            options: &[],
        })
        .label(rotulo)
        .key(&k)],
        y,
    );
    (y, aberto(c, id))
}

/// Uma escolha segmentada que quebra quando não cabe.
pub(crate) fn escolha(
    c: &mut CardCtx<'_>,
    rotulo: &str,
    canal: &[NodeId],
    segs: &[Seg<'_>],
    y: f32,
) -> f32 {
    corpo::choices(
        c.scene,
        c.text_system,
        c.store,
        c.hit_index,
        c.ink,
        c.x,
        c.w,
        y,
        rotulo,
        canal,
        segs,
    )
}

pub(crate) fn seg<'a>(id: NodeId, text: &'a str, on: bool) -> Seg<'a> {
    Seg {
        id,
        icon: None,
        text,
        on,
    }
}

pub(crate) fn seg_icone<'a>(id: NodeId, icon: IconId, on: bool) -> Seg<'a> {
    Seg {
        id,
        icon: Some(icon),
        text: "",
        on,
    }
}

/// ⭐ **O cartão Appearance**: a opacidade e a mistura do OBJECTO, numa linha, como no Inspector.
pub(crate) fn appearance(c: &mut CardCtx<'_>, y: f32) -> f32 {
    const OP: &[NodeId] = &[ids::VECTOR_OBJ_OPACITY_NUM];
    const BL: &[NodeId] = &[crate::ids::VECTOR_OBJ_BLEND];
    let Some(a) = state::current_appearance() else {
        return y;
    };
    let face = if a.mixed_blend {
        tr("panel.vector.mixed")
    } else {
        tr(a.blend.label_key())
    };
    let y = c.row(
        &[
            Cell::new(campo(OP[0], Glyph::Icon(IconId::CardOpacity)))
                .label(tr("card.insp.opacity"))
                .key(OP),
            Cell::new(Widget::Dropdown {
                id: BL[0],
                text: face,
                options: &[],
            })
            .label(tr("card.insp.blend"))
            .key(BL),
        ],
        y,
    );
    if let Some(chip) = aberto(c, BL[0]) {
        state::set_pending_obj_blend_dd(Some(chip));
    }
    y
}

/// O token do preenchimento de BASE: no ⚙ de uma camada extra não há o que ligar.
fn token_do_fill(c: &mut CardCtx<'_>, y: f32) -> f32 {
    match state::token_bindings().filter(|_| state::open_layer().is_none()) {
        Some(b) => token(
            c,
            "panel.vector.token",
            crate::ids::VECTOR_TOKEN_FILL,
            b.fill.as_deref(),
            y,
        ),
        None => y,
    }
}

/// Uma linha de TOKEN (`paint_tokens`): a caixa e o popover de sempre, na cara do cartão.
pub(crate) fn token(
    c: &mut CardCtx<'_>,
    chave: &'static str,
    id: NodeId,
    bound: Option<&str>,
    y: f32,
) -> f32 {
    let Some(slot) = ids::token_slot_of(id) else {
        return y;
    };
    let (y, aberto) = menu(c, id, tr(chave), crate::paint_tokens::chip_label(bound), y);
    if let Some(chip) = aberto {
        state::set_pending_token_dd(Some((slot.code, chip)));
    }
    y
}

fn par_de_botoes(c: &mut CardCtx<'_>, a: (NodeId, &str), b: (NodeId, &str), y: f32) -> f32 {
    c.row(
        &[
            Cell::new(Widget::Button {
                id: a.0,
                text: a.1,
                primary: false,
            }),
            Cell::new(Widget::Button {
                id: b.0,
                text: b.1,
                primary: false,
            }),
        ],
        y,
    )
}

/// ⭐ **O painel do preenchimento** (F44): a regra do compound, o tipo, os controlos do degradê e o
/// token. Age sobre a forma selecionada; sem ela, só o token.
pub(crate) fn fill_props(c: &mut CardCtx<'_>, y: f32) -> f32 {
    let Some(kind) = state::current_fill_kind() else {
        return token_do_fill(c, y);
    };
    let mut y = y;
    if let Some(rule) = state::current_fill_rule() {
        let segs = [
            seg(
                crate::ids::VECTOR_FILL_RULE_NONZERO,
                tr("panel.vector.arrange.non_zero"),
                rule == PathFillRule::NonZero,
            ),
            seg(
                crate::ids::VECTOR_FILL_RULE_EVENODD,
                tr("panel.vector.arrange.even_odd"),
                rule == PathFillRule::EvenOdd,
            ),
        ];
        const REGRA: &[NodeId] = &[
            crate::ids::VECTOR_FILL_RULE_NONZERO,
            crate::ids::VECTOR_FILL_RULE_EVENODD,
        ];
        y = escolha(c, tr("panel.vector.arrange.fill_rule"), REGRA, &segs, y);
    }
    let segs = [
        (
            vids::VECTOR_FILL_KIND_SOLID,
            "panel.vector.arrange.solid",
            FillKind::Solid,
        ),
        (
            vids::VECTOR_FILL_KIND_LINEAR,
            "panel.vector.arrange.linear",
            FillKind::Linear,
        ),
        (
            vids::VECTOR_FILL_KIND_RADIAL,
            "panel.vector.arrange.radial",
            FillKind::Radial,
        ),
        (
            vids::VECTOR_FILL_KIND_MULTI,
            "panel.vector.arrange.multi",
            FillKind::MultiPoint,
        ),
        (
            vids::VECTOR_FILL_KIND_PATTERN,
            "panel.vector.arrange.pattern",
            FillKind::Pattern,
        ),
    ]
    .map(|(id, chave, k)| seg(id, tr(chave), kind == k));
    // O losango é o da TINTA inteira: o tipo é membro dela (como no corpo de antes).
    y = escolha(
        c,
        tr("panel.vector.stroke.type"),
        &[vids::VECTOR_FILL_KIND_SOLID],
        &segs,
        y,
    );
    if kind == FillKind::MultiPoint {
        y = par_de_botoes(
            c,
            (
                crate::ids::VECTOR_GRAD_ADD_POINT,
                tr("panel.vector.arrange.add_point"),
            ),
            (
                crate::ids::VECTOR_GRAD_REMOVE_POINT,
                tr("panel.vector.arrange.remove_point"),
            ),
            y,
        );
        y = pontos_do_multi(c, y);
    }
    if kind == FillKind::Linear || kind == FillKind::Radial {
        y = par_de_botoes(
            c,
            (
                crate::ids::VECTOR_GRAD_ADD_STOP,
                tr("panel.vector.arrange.add_stop"),
            ),
            (
                crate::ids::VECTOR_GRAD_REMOVE_STOP,
                tr("panel.vector.arrange.remove_stop"),
            ),
            y,
        );
    }
    if kind == FillKind::Linear {
        const BARRA: &[NodeId] = &[crate::ids::VECTOR_GRAD_ANGLE];
        y = c.row(
            &[Cell::new(campo(
                vids::VECTOR_GRAD_ANGLE_NUM,
                Glyph::Icon(IconId::CardAngle),
            ))
            .label(tr("card.vector.angle_deg"))
            .key(BARRA)],
            y,
        );
    }
    token_do_fill(c, y)
}

/// Influência e grão do ponto selecionado de um degradê multi-ponto (cada um só se ele os publica).
fn pontos_do_multi(c: &mut CardCtx<'_>, y: f32) -> f32 {
    const INF: &[NodeId] = &[crate::ids::VECTOR_GRAD_INFLUENCE];
    const JIT: &[NodeId] = &[crate::ids::VECTOR_GRAD_JITTER];
    let (rot_i, rot_j) = (
        tr("panel.vector.arrange.influence"),
        tr("panel.vector.arrange.jitter"),
    );
    let mut cells: Vec<Cell<'_>> = Vec::new();
    if state::current_grad_influence().is_some() {
        cells.push(
            Cell::new(campo(
                vids::VECTOR_GRAD_INFLUENCE_NUM,
                Glyph::Text(corpo::letra_da_grandeza(rot_i)),
            ))
            .label(rot_i)
            .key(INF),
        );
    }
    if state::current_grad_jitter().is_some() {
        cells.push(
            Cell::new(campo(
                vids::VECTOR_GRAD_JITTER_NUM,
                Glyph::Text(corpo::letra_da_grandeza(rot_j)),
            ))
            .label(rot_j)
            .key(JIT),
        );
    }
    if cells.is_empty() {
        y
    } else {
        c.row(&cells, y)
    }
}
