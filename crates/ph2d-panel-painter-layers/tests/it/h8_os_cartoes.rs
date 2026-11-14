//! **Composite, Line, Clone, Inpaint, Mask, Deform, Selection e os verbos do Sculpt em linhas de cartão, fatia H8**
//! (`docs/CartoesUI/13_PAINTER_PLANO.md`). Pelo gesto real, nos rects pintados, e o que chega ao barramento é o MESMO do
//! painel de antes. O universo é o conjunto dos ids destas peças; só as barras saem, porque nos cartões a caixa é o chip.

use super::censo_dos_controlos::em_paralelo;
use super::h1_os_cartoes::{Mesa, pincel};
use super::h5_os_cartoes::{na_mesma_linha, sem_sobrepor};
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::cards::censo;
use ph2d_tool_painter::ids as p;
use ph2d_tool_painter::{
    BrushSettings, DEFORM_TEMPERAMENT_RESHAPE, DEFORM_TEMPERAMENT_TRANSFORM, LineKind, PaintMedia,
};
use std::collections::BTreeSet;

fn digital() -> BrushSettings {
    pincel(PaintMedia::Digital, "brush").0
}

fn com_linha(k: LineKind) -> BrushSettings {
    let mut b = digital();
    b.line_kind = k.to_wire();
    b
}

fn composto(ops: &[u8]) -> BrushSettings {
    let mut b = digital();
    b.composite_enabled = true;
    b.composite_len = ops.len();
    for (i, op) in ops.iter().enumerate() {
        b.composite_ops[i] = *op;
    }
    b
}

fn mascara() -> BrushSettings {
    let mut b = digital();
    b.is_mask = true;
    b
}

fn deform(t: u8, modo: u8, relevo: bool) -> BrushSettings {
    let mut b = digital();
    b.is_deform = true;
    b.deform_temperament = t;
    b.deform_mode = modo;
    b.deform_layer_has_relief = relevo;
    b
}

fn selecao(modo: u8, edita: bool) -> BrushSettings {
    let mut b = digital();
    b.is_selection = true;
    b.selection_mode = modo;
    b.selection_edit = edita;
    b
}

fn esculpe(familia: u8, cinzel: bool, infla: bool, filtra: bool, traco: bool) -> BrushSettings {
    let mut b = pincel(PaintMedia::Impasto, "brush").0;
    b.impasto = true;
    b.impasto_applies = true;
    b.is_sculpt = true;
    b.sculpt_knob_family = familia;
    b.sculpt_is_chisel = cinzel;
    b.sculpt_is_inflate = infla;
    b.sculpt_filters = filtra;
    b.sculpt_can_filter_stroke = traco;
    b
}

fn estados() -> Vec<(&'static str, BrushSettings)> {
    let mut clone_com_fonte = pincel(PaintMedia::Digital, "clone").0;
    clone_com_fonte.clone_has_source = true;
    let mut clone_armado = clone_com_fonte;
    clone_armado.clone_sample_armed = true;
    let mut autorado = composto(&[0, 2, 3]);
    autorado.composite_color_authored[0] = true;
    autorado.composite_hardness_authored[1] = true;
    let mut sem_quota = composto(&[0, 1]);
    sem_quota.composite_add_available = [false; ph2d_tool_painter::N_COMPOSITE_OPS];
    vec![
        ("linha none", com_linha(LineKind::None)),
        ("linha speed", com_linha(LineKind::Speed)),
        ("linha sketchy", com_linha(LineKind::Sketchy)),
        ("linha wire", com_linha(LineKind::Wire)),
        ("linha ribbon", com_linha(LineKind::Ribbon)),
        ("linha rough", com_linha(LineKind::Rough)),
        ("borracha", pincel(PaintMedia::Digital, "eraser").0),
        ("clone", pincel(PaintMedia::Digital, "clone").0),
        ("clone com fonte", clone_com_fonte),
        ("clone armado", clone_armado),
        ("inpaint", pincel(PaintMedia::Digital, "inpaint").0),
        ("composto vazio", composto(&[])),
        ("composto 3", composto(&[0, 1, 3])),
        ("composto autorado", autorado),
        ("composto sem quota", sem_quota),
        ("mascara", mascara()),
        ("deform nada", deform(u8::MAX, 0, false)),
        (
            "deform liquify",
            deform(DEFORM_TEMPERAMENT_RESHAPE, 0, false),
        ),
        ("deform relevo", deform(DEFORM_TEMPERAMENT_RESHAPE, 1, true)),
        (
            "deform reconstroi",
            deform(DEFORM_TEMPERAMENT_RESHAPE, 5, true),
        ),
        (
            "deform transform",
            deform(DEFORM_TEMPERAMENT_TRANSFORM, 0, false),
        ),
        ("selecao auto", selecao(0, false)),
        ("selecao livre", selecao(1, false)),
        ("selecao rect", selecao(2, false)),
        ("selecao edita", selecao(3, true)),
        ("selecao pen", selecao(4, false)),
        ("raio", esculpe(0, false, false, true, false)),
        ("raio traco", esculpe(0, false, false, true, true)),
        ("plano", esculpe(1, false, false, false, false)),
        ("cinzel", esculpe(1, true, false, false, false)),
        ("camada", esculpe(2, false, false, false, false)),
        ("infla", esculpe(2, false, true, false, false)),
    ]
}

/// As barras destas peças: nos cartões, a caixa é o chip ligado a elas, e o painel continua a ouvi-las.
pub(super) fn barras() -> BTreeSet<NodeId> {
    let mut u: BTreeSet<NodeId> = [
        p::PAINTER_INPAINT_PATCH_SLIDER,
        p::PAINTER_INPAINT_QUALITY_SLIDER,
        p::PAINTER_INPAINT_SEARCH_SLIDER,
        p::PAINTER_DEFORM_SIZE_SLIDER,
        p::PAINTER_DEFORM_PRESSURE_SLIDER,
        p::PAINTER_DEFORM_DISTORTION_SLIDER,
        p::PAINTER_DEFORM_MOMENTUM_SLIDER,
        p::PAINTER_DEFORM_STRENGTH_SLIDER,
        p::PAINTER_SEL_THRESHOLD_SLIDER,
        p::PAINTER_SEL_STABILIZE_SLIDER,
        p::PAINTER_SEL_FEATHER_SLIDER,
        p::PAINTER_SEL_OPACITY_SLIDER,
        p::PAINTER_SEL_OFFSET_SLIDER,
        p::PAINTER_SCULPT_RADIUS_SLIDER,
        p::PAINTER_SCULPT_OFFSET_SLIDER,
        p::PAINTER_SCULPT_DEPTH_SLIDER,
        p::PAINTER_SCULPT_ANGLE_SLIDER,
        p::PAINTER_SCULPT_SMOOTH_SLIDER,
    ]
    .into_iter()
    .collect();
    u.extend(p::PAINTER_BRUSH_COMPOSITE_STRENGTH);
    u.extend(p::PAINTER_BRUSH_COMPOSITE_SIZE);
    u.extend(p::PAINTER_BRUSH_COMPOSITE_HARDNESS);
    for k in [
        LineKind::Sketchy,
        LineKind::Wire,
        LineKind::Ribbon,
        LineKind::Rough,
    ] {
        for b in ph2d_panel_painter_layers::line_barras::barras_de(k) {
            u.insert(b.slider);
        }
    }
    u
}

/// Os ids destas peças. Barras de fora: nos cartões, a caixa é o chip.
fn universo() -> BTreeSet<NodeId> {
    let mut u: BTreeSet<NodeId> = [
        p::PAINTER_LINE_SOLID,
        p::PAINTER_LINE_TYPE,
        p::PAINTER_LINE_SKETCHY_MAGNETIFY,
        p::PAINTER_LINE_WIRE_CONNECTION,
        p::PAINTER_BRUSH_CLONE_SET_SOURCE,
        p::PAINTER_BRUSH_CLONE_ALIGNED,
        p::PAINTER_BRUSH_COMPOSITE_ENABLE,
        p::PAINTER_BRUSH_COMPOSITE_ADD,
        p::PAINTER_BRUSH_COMPOSITE_ADD_KIND,
        p::PAINTER_INPAINT_PATCH_CHIP,
        p::PAINTER_INPAINT_QUALITY_CHIP,
        p::PAINTER_INPAINT_SEARCH_CHIP,
        p::PAINTER_MASK_APPLY,
        p::PAINTER_DEFORM_RELIEF,
        p::PAINTER_DEFORM_SIZE_CHIP,
        p::PAINTER_DEFORM_PRESSURE_CHIP,
        p::PAINTER_DEFORM_DISTORTION_CHIP,
        p::PAINTER_DEFORM_MOMENTUM_CHIP,
        p::PAINTER_DEFORM_STRENGTH_CHIP,
        p::PAINTER_SEL_THRESHOLD_CHIP,
        p::PAINTER_SEL_STABILIZE_CHIP,
        p::PAINTER_SEL_FEATHER_CHIP,
        p::PAINTER_SEL_OPACITY_CHIP,
        p::PAINTER_SEL_OFFSET_CHIP,
        p::PAINTER_SEL_OFFSET_APPLY,
        p::PAINTER_SEL_OFFSET_APPLY_KEEP,
        p::PAINTER_SEL_EDIT,
        p::PAINTER_SEL_CONVERT,
        p::PAINTER_SEL_MERGE,
        p::PAINTER_SEL_SIMPLIFY,
        p::PAINTER_SCULPT_RADIUS_CHIP,
        p::PAINTER_SCULPT_OFFSET_CHIP,
        p::PAINTER_SCULPT_DEPTH_CHIP,
        p::PAINTER_SCULPT_ANGLE_CHIP,
        p::PAINTER_SCULPT_SMOOTH_CHIP,
        p::PAINTER_SCULPT_RAKE,
        p::PAINTER_SCULPT_FILTER,
        p::PAINTER_SCULPT_FILTER_STROKE,
    ]
    .into_iter()
    .collect();
    u.extend(p::PAINTER_BRUSH_COMPOSITE_STRENGTH_CHIP);
    u.extend(p::PAINTER_BRUSH_COMPOSITE_SIZE_CHIP);
    u.extend(p::PAINTER_BRUSH_COMPOSITE_HARDNESS_CHIP);
    u.extend(p::PAINTER_BRUSH_COMPOSITE_UP);
    u.extend(p::PAINTER_BRUSH_COMPOSITE_DOWN);
    u.extend(p::PAINTER_BRUSH_COMPOSITE_REMOVE);
    u.extend(p::PAINTER_BRUSH_COMPOSITE_COLOR);
    u.extend(p::PAINTER_BRUSH_COMPOSITE_COLOR_CLEAR);
    u.extend(p::PAINTER_BRUSH_COMPOSITE_HARDNESS_CLEAR);
    u.extend(p::PAINTER_BRUSH_COMPOSITE_ERASE_SCOPE);
    u.extend(p::PAINTER_MASK_BRUSH);
    u.extend(p::PAINTER_MASK_OP);
    u.extend(p::PAINTER_MASK_COLOR);
    u.extend(p::PAINTER_DEFORM_MODE_IDS.iter().copied());
    u.extend(p::PAINTER_DEFORM_TEMPERAMENT_IDS.iter().copied());
    u.extend(p::PAINTER_DEFORM_TRANSFORM_MODE_IDS.iter().copied());
    u.extend(p::PAINTER_DEFORM_ACTION_IDS.iter().copied());
    u.extend(p::PAINTER_SEL_MODE_IDS.iter().copied());
    u.extend(p::PAINTER_SEL_OP_IDS.iter().copied());
    u.extend(p::PAINTER_SEL_ACTION_IDS.iter().copied());
    u.extend(p::PAINTER_SEL_WAVE5_IDS.iter().copied());
    for k in [
        LineKind::Sketchy,
        LineKind::Wire,
        LineKind::Ribbon,
        LineKind::Rough,
    ] {
        for b in ph2d_panel_painter_layers::line_barras::barras_de(k) {
            u.insert(b.chip);
        }
    }
    u
}

fn deste(mesa: &Mesa, u: &BTreeSet<NodeId>) -> BTreeSet<NodeId> {
    mesa.ids().into_iter().filter(|i| u.contains(i)).collect()
}

/// ⭐ As mesmas condições, nas DUAS direcções, em cada estado.
#[test]
fn these_pieces_show_the_same_controls_as_the_old_panel() {
    let u = universo();
    let mut vistos = BTreeSet::new();
    for (nome, b) in estados() {
        let antes = deste(&Mesa::nova(false, b, false), &u);
        let agora = deste(&Mesa::nova(true, b, false), &u);
        let so_antes: Vec<_> = antes.difference(&agora).collect();
        let so_agora: Vec<_> = agora.difference(&antes).collect();
        assert!(
            so_antes.is_empty() && so_agora.is_empty(),
            "{nome}: só antes {so_antes:?}, só agora {so_agora:?}"
        );
        vistos.extend(agora);
    }
    assert!(vistos.len() > 90, "a régua viu pouco: {}", vistos.len());
}

fn e_clique(mesa: &Mesa, id: NodeId) -> bool {
    let s = mesa.host.store();
    s.card_field(id).is_none() && !matches!(s.get(id), Some(InteractiveState::NumberInput { .. }))
}

/// Cada botão, caixa e segmento manda o MESMO. As listas abrem: mede-se em baixo.
#[test]
fn every_click_sends_the_same_action_as_the_old_panel() {
    let u = universo();
    let listas = [p::PAINTER_LINE_TYPE, p::PAINTER_BRUSH_COMPOSITE_ADD_KIND];
    let mut fila = Vec::new();
    for (nome, b) in estados() {
        let mesa = Mesa::nova(true, b, false);
        let cliques: Vec<NodeId> = deste(&mesa, &u)
            .into_iter()
            .filter(|id| !listas.contains(id) && e_clique(&mesa, *id))
            .collect();
        fila.extend(cliques.into_iter().map(|id| (nome, b, id)));
    }
    em_paralelo(&fila, |(nome, b, id)| {
        let velho = Mesa::nova(false, *b, false).clica(*id);
        let novo = Mesa::nova(true, *b, false).clica(*id);
        assert_eq!(velho, novo, "{nome}: {id:?}");
    });
    assert!(fila.len() >= 120, "viu {} cliques", fila.len());
}

/// O chip de uma barra só se escreve no gesto no painel de antes: o valor confere-se contra o pincel, a faixa contra o
/// painel de antes.
#[test]
fn every_number_shows_the_value_of_the_brush_and_the_range_of_the_old_panel() {
    let u = universo();
    let mut conferidos = 0;
    for (nome, mut b) in estados() {
        b.composite_strength[0] = 0.31;
        b.composite_size[0] = 1.7;
        b.composite_hardness[0] = 0.42;
        b.deform_pressure = 0.37;
        b.selection_feather = 0.21;
        b.sculpt_offset = 0.25;
        b.thread_opacity = 0.61;
        b.inpaint_quality = 0.5;
        let (velha, nova) = (Mesa::nova(false, b, false), Mesa::nova(true, b, false));
        let s_n = nova.host.store();
        for id in deste(&nova, &u) {
            if e_clique(&nova, id) {
                continue;
            }
            assert_eq!(
                velha.host.store().number_range(id),
                s_n.number_range(id),
                "{nome}: {id:?}"
            );
        }
        for (id, espera) in [
            (p::PAINTER_BRUSH_COMPOSITE_STRENGTH_CHIP[0], 0.31),
            // O Size em px de diâmetro: o efectivo × 2 × raio, inteiro (dono 08/11).
            (
                p::PAINTER_BRUSH_COMPOSITE_SIZE_CHIP[0],
                f64::from((2.0 * b.size_px * b.composite_layer_size_effective(0)).round()),
            ),
            (p::PAINTER_BRUSH_COMPOSITE_HARDNESS_CHIP[0], 0.42),
            (p::PAINTER_DEFORM_PRESSURE_CHIP, 0.37),
            (p::PAINTER_SEL_FEATHER_CHIP, 0.21),
            (p::PAINTER_SCULPT_OFFSET_CHIP, -0.5),
            (p::PAINTER_LINE_SKETCHY_OPACITY_CHIP, 0.61),
            // Quality: `3..12`, inteiro, a meio da pista.
            (p::PAINTER_INPAINT_QUALITY_CHIP, 8.0),
        ] {
            if nova.rect(id).is_none() {
                continue;
            }
            let v = s_n.number_value(id).expect("semeado");
            assert!(
                (v - espera).abs() < 0.51 * 1e-2 + f64::from(u8::from(espera == 8.0)),
                "{nome}: {id:?} {v}"
            );
            conferidos += 1;
        }
    }
    assert!(conferidos > 10, "conferiu {conferidos}");
}

/// As duas listas abrem por baixo da caixa, e a opção manda o mesmo do painel de antes.
#[test]
fn each_list_opens_under_its_box_and_its_option_sends_the_same() {
    let tipo = (
        com_linha(LineKind::Sketchy),
        p::PAINTER_LINE_TYPE,
        p::painter_line_type_option_id(LineKind::Rough.to_wire()),
    );
    let mais = (
        composto(&[0]),
        p::PAINTER_BRUSH_COMPOSITE_ADD_KIND,
        p::PAINTER_BRUSH_COMPOSITE_ADD_OPTION[1],
    );
    for (b, caixa, opcao) in [tipo, mais] {
        let manda = |cartoes: bool| {
            let mut m = Mesa::nova(cartoes, b, false);
            let r = m.rect(caixa).expect("a caixa");
            let _ = m.clica(caixa);
            m.assenta();
            let o = m.rect(opcao).expect("a opção");
            assert!(
                o.y >= r.y + r.h - 1.0 && (o.x - r.x).abs() < 3.0,
                "{o:?} × {r:?}"
            );
            m.clica(opcao)
        };
        assert_eq!(manda(false), manda(true), "{caixa:?}");
    }
}

/// A seta inerte na ponta não tem alvo; a quota gasta apaga o `+` e o menu.
#[test]
fn what_cannot_happen_has_no_target() {
    let m = Mesa::nova(true, composto(&[0, 1, 2]), false);
    assert!(m.rect(p::PAINTER_BRUSH_COMPOSITE_UP[0]).is_none());
    assert!(m.rect(p::PAINTER_BRUSH_COMPOSITE_DOWN[0]).is_some());
    assert!(m.rect(p::PAINTER_BRUSH_COMPOSITE_UP[2]).is_some());
    assert!(m.rect(p::PAINTER_BRUSH_COMPOSITE_DOWN[2]).is_none());
    let mut gasto = composto(&[0, 1]);
    gasto.composite_add_available = [false; ph2d_tool_painter::N_COMPOSITE_OPS];
    let g = Mesa::nova(true, gasto, false);
    assert!(g.rect(p::PAINTER_BRUSH_COMPOSITE_ADD).is_none());
    assert!(g.rect(p::PAINTER_BRUSH_COMPOSITE_ADD_KIND).is_none());
    let v = Mesa::nova(true, composto(&[0, 1]), false);
    assert!(v.rect(p::PAINTER_BRUSH_COMPOSITE_ADD).is_some());
}

/// Irmãos curtos dividem a linha (05 §4); Solid e o tipo têm o nome em cima, não à esquerda.
#[test]
fn short_siblings_share_a_row() {
    let digital = Mesa::nova(true, com_linha(LineKind::Sketchy), false);
    na_mesma_linha(&digital, p::PAINTER_LINE_SOLID, p::PAINTER_LINE_TYPE);
    let c = Mesa::nova(true, composto(&[0, 1]), false);
    na_mesma_linha(
        &c,
        p::PAINTER_BRUSH_COMPOSITE_STRENGTH_CHIP[0],
        p::PAINTER_BRUSH_COMPOSITE_SIZE_CHIP[0],
    );
    na_mesma_linha(
        &c,
        p::PAINTER_BRUSH_COMPOSITE_HARDNESS_CHIP[0],
        p::PAINTER_BRUSH_COMPOSITE_COLOR[0],
    );
    let cl = Mesa::nova(true, pincel(PaintMedia::Digital, "clone").0, false);
    na_mesma_linha(&cl, p::PAINTER_BRUSH_CLONE_ALIGNED, p::PAINTER_LINE_TYPE);
    let d = Mesa::nova(true, deform(DEFORM_TEMPERAMENT_RESHAPE, 0, false), false);
    na_mesma_linha(
        &d,
        p::PAINTER_DEFORM_SIZE_CHIP,
        p::PAINTER_DEFORM_PRESSURE_CHIP,
    );
    let s = Mesa::nova(true, selecao(0, false), false);
    na_mesma_linha(
        &s,
        p::PAINTER_SEL_THRESHOLD_CHIP,
        p::PAINTER_SEL_FEATHER_CHIP,
    );
}

/// O escopo da borracha mostra o nome inteiro e a cor sai da linha: a borracha não deposita.
#[test]
fn the_eraser_layer_trades_the_colour_for_its_scope() {
    let m = Mesa::nova(true, composto(&[3]), false);
    assert!(m.rect(p::PAINTER_BRUSH_COMPOSITE_ERASE_SCOPE[0]).is_some());
    assert!(m.rect(p::PAINTER_BRUSH_COMPOSITE_COLOR[0]).is_none());
}

#[test]
fn no_control_of_these_pieces_overlaps_another() {
    let u = universo();
    for (nome, b) in estados() {
        let m = Mesa::nova(true, b, false);
        sem_sobrepor(&m, &deste(&m, &u), nome);
    }
}

fn censo_de(b: BrushSettings) -> (Mesa, censo::Censo) {
    let mut m = Mesa::nova(true, b, false);
    let (rects, c) = censo::grava(|| {
        let (h, st, j) = (&mut m.host, &mut m.st, m.janela);
        h.paint::<ph2d_panel_painter_layers::PainterLayersPanel>(st, j)
    });
    m.rects = rects;
    (m, c)
}

#[test]
fn no_text_is_cut_and_no_row_sticks_in_any_state() {
    for (nome, b) in estados() {
        let (_, c) = censo_de(b);
        for (regra, lista) in [
            ("cortado", c.textos_cortados()),
            ("grudada", c.linhas_grudadas()),
            ("irmãos", c.irmaos_separados()),
        ] {
            assert!(lista.is_empty(), "{nome}: {regra} {lista:#?}");
        }
    }
}

/// ⭐ **O censo não está cego.** Todo controlo destas peças cai numa linha que o censo anotou: a população da lei da
/// compactação desce porque há menos linhas, não porque a régua deixou de ver.
/// *Mutação: tirar `anota_linha` de uma porta dos cartões ⇒ vermelho.*
#[test]
fn every_control_falls_on_a_row_the_census_saw() {
    let u = universo();
    let mut vistos = 0;
    for (nome, b) in estados() {
        let (m, c) = censo_de(b);
        for id in deste(&m, &u) {
            let r = m.rect(id).expect("pintado");
            let meio = r.y + r.h * 0.5;
            let visto = c
                .linhas
                .iter()
                .any(|l| meio >= l.topo && meio <= l.fundo && r.x + r.w > l.x && r.x < l.x + l.w);
            assert!(visto, "{nome}: {id:?} {r:?} fora do censo");
            vistos += 1;
        }
    }
    assert!(vistos > 300, "viu {vistos}");
}

// ── Reforço: o que sobreviveu às contraprovas de H5 a H7 ─────────────────────────────────────────────────────────

/// A ordem das duas camadas de cor é a do pincel. *Mutação: `t_ordem_fixa` ⇒ vermelho.*
#[test]
fn the_colour_layers_keep_the_order_of_the_brush() {
    let y_de = |m: &Mesa, id| m.rect(id).expect("a camada").y;
    let mut b = super::h5_os_cartoes::com_rampa();
    b.shape_falloff_color = true;
    b.shape_texture_color = true;
    for primeira in [true, false] {
        b.shape_falloff_first = primeira;
        let m = Mesa::nova(true, b, false);
        let queda = y_de(&m, p::PAINTER_SHAPE_FALLOFF_COLOR);
        let textura = y_de(&m, p::PAINTER_SHAPE_TEXTURE_COLOR);
        assert_eq!(queda < textura, primeira, "{queda} × {textura}");
    }
}

/// Um só aceso em Every Dab / Every Stroke: as duas pinturas têm as mesmas tintas, só trocadas.
/// *Mutação: `a_quando` (o 1.º aceso sempre) ⇒ vermelho.*
#[test]
fn only_one_option_of_the_colour_jitter_is_lit() {
    let pinta = |escopo: u8| {
        let mut b = digital();
        b.color_jitter_scope = escopo;
        let mut m = Mesa::nova(true, b, false);
        m.so_abertos(&[p::PAINTER_BRUSH_RANDOMIZE_SECTION]);
        let (h, st) = (&mut m.host, &mut m.st);
        let mut d =
            h.paint_draw_data::<ph2d_panel_painter_layers::PainterLayersPanel>(st, m.janela);
        d.sort_unstable();
        d
    };
    assert_eq!(pinta(0), pinta(1));
}

/// A barra e a lista levam a marca de animação que o item promete. *Mutação: `l_barra_sem_marca`, `l_lista_sem_marca`.*
#[test]
fn a_bar_and_a_list_wear_their_animation_mark() {
    use ph2d_editor_core::widget::key_mark::{
        KeyMark, add_key_mark, key_mark_target, set_key_marks,
    };
    for (id, b) in [
        (p::PAINTER_BRUSH_SPACING_CHIP, digital()),
        (p::PAINTER_PEN_ROTATION, digital()),
    ] {
        set_key_marks(std::collections::BTreeMap::new());
        add_key_mark(id, KeyMark::Chave);
        let m = Mesa::nova(true, b, false);
        let tem = m
            .ids()
            .into_iter()
            .any(|i| key_mark_target(i).is_some_and(|t| t.fields.contains(&id)));
        set_key_marks(std::collections::BTreeMap::new());
        assert!(tem, "{id:?} sem marca");
    }
}

/// A barra que a ferramenta diz inerte diz porquê, em Stroke, Pen e nestas peças.
/// *Mutação: `tr_sem_dica` ⇒ vermelho. `ab_sem_dica` sobrevive e é equivalente: nenhum `Dependente` cobre as barras
/// de `arma_barra` (Line, Composite, Inpaint, Deform, Selection, Sculpt), medido em `inercia.rs`.*
#[test]
fn an_inert_bar_says_why() {
    let mut vistas = 0;
    let todas: Vec<NodeId> = super::h7_os_cartoes::BARRAS
        .iter()
        .copied()
        .chain(barras())
        .collect();
    let estados_todos = super::h7_os_cartoes::estados().into_iter().chain(estados());
    for (nome, b) in estados_todos {
        let m = Mesa::nova(true, b, false);
        for barra in &todas {
            let chip = m.host.store().linked_number(*barra);
            let a_vista = chip.is_some_and(|c| m.rect(c).is_some());
            if !a_vista || b.inercias.dica_de(*barra).is_none() {
                continue;
            }
            let dica = m.host.store().tooltip_for(*barra);
            assert!(dica.is_some_and(|t| !t.is_empty()), "{nome}: {barra:?}");
            vistas += 1;
        }
    }
    assert!(vistas > 0, "nenhuma barra inerte nos estados");
}

/// ⭐ **A prova do piso da população** (coordenador 04/11): em cada meio de fábrica, todo controlo nativo cai numa linha do
/// censo. O número de linhas desceu porque irmãos dividem a linha e os corpos deixaram a captura, não porque a régua
/// cegou. Fora ficam só o cromo do cartão e as telas próprias (curva, visor, gizmo, editor da rampa), declaradas.
#[test]
fn in_each_medium_every_native_control_falls_on_a_row_the_census_saw() {
    use ph2d_editor_core::ids as core_ids;
    use ph2d_editor_core::panel::cards::grip_of;
    let mut total = (0usize, 0usize, 0usize);
    let mut cegos = Vec::new();
    for meio in [
        PaintMedia::Digital,
        PaintMedia::Watercolor,
        PaintMedia::Impasto,
        PaintMedia::WetPaint,
    ] {
        let (m, c) = censo_de(pincel(meio, "brush").0);
        total.0 += c.linhas.len();
        let secoes = super::h1_os_cartoes::TODAS;
        for id in m.ids() {
            let cromo = secoes.iter().any(|s| *s == id || grip_of(*s) == id)
                || [
                    core_ids::PAINTER_LAYERS_PANEL,
                    p::PAINTER_SIDEBAR_TOGGLE_DOCK,
                    p::PAINTER_LAYERS_TOGGLE_DOCK,
                ]
                .contains(&id);
            if cromo {
                continue;
            }
            let r = m.rect(id).expect("pintado");
            let meio_y = r.y + r.h * 0.5;
            let visto = c.linhas.iter().any(|l| {
                meio_y >= l.topo && meio_y <= l.fundo && r.x + r.w > l.x && r.x < l.x + l.w
            });
            total.1 += 1;
            if visto {
                continue;
            }
            total.2 += 1;
            // Fora só o ícone quadrado do cabeçalho e a tela própria (visor, alça, título que dobra).
            let numero = matches!(
                m.host.store().get(id),
                Some(InteractiveState::NumberInput { .. } | InteractiveState::Slider { .. })
            );
            let botao = matches!(
                m.host.store().get(id),
                Some(InteractiveState::Button { .. } | InteractiveState::Dropdown { .. })
            );
            if numero || (botao && (r.w - r.h).abs() > 1.0) {
                cegos.push(format!(
                    "{meio:?} y {:.0} x {:.0} w {:.0} {id:?}",
                    r.y, r.x, r.w
                ));
            }
        }
    }
    println!("linhas {} controlos {} fora {}", total.0, total.1, total.2);
    assert!(cegos.is_empty(), "o censo não os vê: {cegos:#?}");
    assert!(total.2 <= TELAS_FORA_DO_CENSO, "fora {}", total.2);
}

/// Os alvos de tela própria e os resets do cabeçalho nos quatro meios: medido em 04/11; só desce.
const TELAS_FORA_DO_CENSO: usize = 69;

/// Mixing, Symmetry e Tiling aparecem onde apareciam, agora sem correr o corpo antigo.
/// *Mutação: `n_sem_presenca` ⇒ vermelho.*
#[test]
fn the_native_sections_appear_where_the_old_panel_shows_them() {
    use ph2d_editor_core::panel::cards::grip_of;
    for (nome, b) in estados() {
        let (velha, nova) = (Mesa::nova(false, b, false), Mesa::nova(true, b, false));
        for sec in [
            p::PAINTER_BRUSH_SYMMETRY_SECTION,
            p::PAINTER_BRUSH_TILING_SECTION,
        ] {
            assert_eq!(
                velha.rect(grip_of(sec)).is_some(),
                nova.rect(grip_of(sec)).is_some(),
                "{nome}: {sec:?}"
            );
        }
        let mix = p::PAINTER_WATERCOLOR_MIX;
        assert_eq!(
            velha.rect(mix).is_some(),
            nova.rect(mix).is_some(),
            "{nome}: Mixing"
        );
    }
}

/// O que abre de fábrica, linha a linha: de onde sai a altura de abertura.
#[test]
#[ignore = "diagnóstico: imprime as linhas, não afirma nada"]
fn diag_as_linhas_da_abertura() {
    use ph2d_panel_painter_layers::PainterLayersPanel;
    use ph2d_panel_painter_layers::cards::force_cards_on_this_thread;
    use ph2d_panel_painter_layers::state::{
        PainterLayersPanelState, set_current_brush, set_current_dock_shows_layers,
    };
    force_cards_on_this_thread(Some(true));
    set_current_brush(None);
    set_current_dock_shows_layers(false);
    let mut host = ph2d_ui_testkit::MockPanelHost::with_panel::<PainterLayersPanel>();
    let mut st = PainterLayersPanelState;
    // O painel inteiro sem rolar (no Wet Paint passou de 6000 px: ver `h1_os_cartoes::COLUNA`).
    let j = ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1600.0, 9000.0);
    for _ in 0..30 {
        let _ = host.paint::<PainterLayersPanel>(&mut st, j);
    }
    let (_, c) = censo::grava(|| host.paint::<PainterLayersPanel>(&mut st, j));
    for l in &c.linhas {
        let nomes: Vec<&str> = l.celulas.iter().map(|x| x.nome.as_str()).collect();
        println!(
            "{:6.0}..{:6.0} x {:5.0} w {:4.0} {nomes:?}",
            l.topo, l.fundo, l.x, l.w
        );
    }
}
