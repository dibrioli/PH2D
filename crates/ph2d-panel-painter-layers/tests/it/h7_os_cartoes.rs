//! **Os cartões Stroke e Pen em linhas de cartão, fatia H7** (`docs/CartoesUI/13_PAINTER_PLANO.md`). Pelo gesto real, nos rects
//! pintados, e o que chega ao barramento é o MESMO do painel de antes. O universo sai do próprio painel de antes: o que aparece
//! entre os cabeçalhos destes cartões. Nenhum motor, preset ou pincel muda.

use super::h1_os_cartoes::{Mesa, esc, pincel};
use super::h5_os_cartoes::{na_mesma_linha, sem_sobrepor};
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids as core_ids;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::panel::cards::{censo, grip_of};
use ph2d_editor_core::widget::key_mark::{KeyMark, add_key_mark, key_mark_target, set_key_marks};
use ph2d_tool_painter::ids as p;
use ph2d_tool_painter::{BrushSettings, PaintMedia, StrokeMethod};
use std::collections::BTreeSet;

const TRACO: NodeId = p::PAINTER_BRUSH_STROKE_SECTION;
const CANETA: NodeId = p::PAINTER_PEN_SECTION;
const JITTER: NodeId = p::PAINTER_BRUSH_JITTER_SECTION;
const ABERTOS: [NodeId; 2] = [TRACO, CANETA];

/// As barras do Stroke e do Jitter: nos cartões, a caixa é o chip ligado a elas, e o painel continua a ouvi-las.
pub(super) const BARRAS: [NodeId; 25] = [
    p::PAINTER_BRUSH_SPACING,
    p::PAINTER_BRUSH_RATE,
    p::PAINTER_BRUSH_OFFSET,
    p::PAINTER_BRUSH_DASH_RATIO,
    p::PAINTER_BRUSH_DASH_LENGTH,
    p::PAINTER_BRUSH_INPUT_SAMPLES,
    p::PAINTER_BRUSH_STABILIZE,
    p::PAINTER_BRUSH_SPRAY_COUNT,
    p::PAINTER_BRUSH_JITTER,
    p::PAINTER_BRUSH_JITTER_SPACING,
    p::PAINTER_BRUSH_JITTER_SCALE,
    p::PAINTER_BRUSH_JITTER_ROTATE,
    p::PAINTER_BRUSH_GRID_CELL[0],
    p::PAINTER_BRUSH_GRID_CELL[1],
    p::PAINTER_BRUSH_GRID_OFFSET[0],
    p::PAINTER_BRUSH_GRID_OFFSET[1],
    p::PAINTER_BRUSH_GRID_FIT,
    p::PAINTER_MARCA_FIELDS[0],
    p::PAINTER_MARCA_FIELDS[1],
    p::PAINTER_MARCA_FIELDS[2],
    p::PAINTER_MARCA_FIELDS[3],
    p::PAINTER_MARCA_FIELDS[4],
    p::PAINTER_MARCA_FIELDS[5],
    p::PAINTER_MARCA_FIELDS[6],
    p::PAINTER_MARCA_FIELDS[7],
];

fn digital() -> BrushSettings {
    pincel(PaintMedia::Digital, "brush").0
}

fn com_metodo(m: StrokeMethod) -> BrushSettings {
    let mut b = digital();
    b.stroke_method = m.to_u8();
    b
}

fn figura(m: StrokeMethod) -> BrushSettings {
    let mut b = com_metodo(m);
    b.can_simplify = true;
    b
}

pub(super) fn estados() -> Vec<(&'static str, BrushSettings)> {
    vec![
        ("digital", digital()),
        ("molhado", pincel(PaintMedia::WetPaint, "brush").0),
        ("aguada", pincel(PaintMedia::Watercolor, "brush").0),
        ("impasto", pincel(PaintMedia::Impasto, "brush").0),
        ("borracha", pincel(PaintMedia::Digital, "eraser").0),
        ("smear", pincel(PaintMedia::Digital, "smear").0),
        ("blur", pincel(PaintMedia::Digital, "blur").0),
        ("clone", pincel(PaintMedia::Digital, "clone").0),
        ("inpaint", pincel(PaintMedia::Digital, "inpaint").0),
        ("dots", com_metodo(StrokeMethod::Dots)),
        ("airbrush", com_metodo(StrokeMethod::Airbrush)),
        ("anchored", com_metodo(StrokeMethod::Anchored)),
        ("drag dot", com_metodo(StrokeMethod::DragDot)),
        ("line", com_metodo(StrokeMethod::Line)),
        ("arc", figura(StrokeMethod::Arc)),
        ("ellipse", figura(StrokeMethod::Ellipse)),
        ("polygon", figura(StrokeMethod::Polygon)),
        ("free hand", figura(StrokeMethod::FreeHand)),
        ("grid", com_metodo(StrokeMethod::GridStamp)),
    ]
}

fn so(cartoes: bool, b: BrushSettings, abertos: &[NodeId]) -> Mesa {
    let mut m = Mesa::nova(cartoes, b, false);
    if cartoes {
        m.so_abertos(abertos);
    }
    if m.host.store().is_collapsed(JITTER) {
        m.host.store_mut().toggle_collapsed(JITTER);
    }
    m.assenta();
    m
}

fn abre(cartoes: bool, b: BrushSettings) -> Mesa {
    so(cartoes, b, &ABERTOS)
}

fn do_corpo(b: BrushSettings) -> BTreeSet<NodeId> {
    let abertos = abre(true, b).ids();
    let dobrados = so(true, b, &[]).ids();
    abertos.difference(&dobrados).copied().collect()
}

fn entre(m: &Mesa, de: NodeId, ate: NodeId) -> BTreeSet<NodeId> {
    let (Some(de), Some(ate)) = (m.rect(grip_of(de)), m.rect(grip_of(ate))) else {
        return BTreeSet::new();
    };
    let cromo = [
        TRACO,
        CANETA,
        grip_of(CANETA),
        p::PAINTER_BRUSH_STROKE_RESET,
        p::PAINTER_PEN_RESET,
    ];
    m.ids()
        .into_iter()
        .filter(|id| !cromo.contains(id) && !BARRAS.contains(id))
        .filter(|id| {
            m.rect(*id)
                .is_some_and(|r| r.y >= de.y + de.h && r.y < ate.y)
        })
        .collect()
}

fn do_painel_de_antes(b: BrushSettings) -> BTreeSet<NodeId> {
    let m = abre(false, b);
    let mut ids = entre(&m, TRACO, CANETA);
    // O Clone não tem Symmetry: o cartão seguinte é o Tiling.
    let seguinte = if m.rect(grip_of(p::PAINTER_BRUSH_SYMMETRY_SECTION)).is_some() {
        p::PAINTER_BRUSH_SYMMETRY_SECTION
    } else {
        p::PAINTER_BRUSH_TILING_SECTION
    };
    ids.extend(entre(&m, CANETA, seguinte));
    ids
}

/// ⭐ As mesmas condições, nas DUAS direcções, em cada estado. O universo sai do painel de antes.
#[test]
fn stroke_and_pen_show_the_same_controls_as_the_old_panel() {
    let mut vistos = BTreeSet::new();
    for (nome, b) in estados() {
        let (antes, agora) = (do_painel_de_antes(b), do_corpo(b));
        let so_antes: Vec<_> = antes.difference(&agora).collect();
        let so_agora: Vec<_> = agora.difference(&antes).collect();
        assert!(
            so_antes.is_empty() && so_agora.is_empty(),
            "{nome}: só antes {so_antes:?}, só agora {so_agora:?}"
        );
        vistos.extend(agora);
    }
    assert!(vistos.len() > 60, "a régua viu pouco: {}", vistos.len());
    for id in [
        core_ids::PAINTER_BRUSH_STROKE_METHOD,
        p::PAINTER_BRUSH_CONTINUOUS_SPACING,
        p::PAINTER_BRUSH_SPACE_ATTEN,
        p::PAINTER_BRUSH_LINE_DIMENSIONS,
        p::PAINTER_BRUSH_EDGE_TO_EDGE,
        p::PAINTER_BRUSH_GRID_SHOW,
        p::PAINTER_BRUSH_STROKE_APPLY,
        p::PAINTER_BRUSH_STROKE_APPLY_KEEP,
        p::PAINTER_BRUSH_STROKE_DELETE,
        p::PAINTER_BRUSH_STROKE_EDIT,
        p::PAINTER_BRUSH_STROKE_SIMPLIFY,
        p::PAINTER_BRUSH_STROKE_MERGE,
        p::PAINTER_STROKE_OP_ADD,
        p::PAINTER_BRUSH_OFFSET_TRIM,
        p::PAINTER_BRUSH_JITTER_UNIT,
        p::PAINTER_PEN_ROTATION,
        p::PAINTER_PEN_SPEED_OPACITY,
    ] {
        assert!(vistos.contains(&id), "{id:?} nunca à vista");
    }
}

fn numeros(b: BrushSettings) -> Vec<NodeId> {
    let m = abre(true, b);
    do_corpo(b)
        .into_iter()
        .filter(|id| m.host.store().card_field(*id).is_some())
        .collect()
}

#[test]
fn typing_each_number_sends_the_same_action_as_the_old_panel() {
    let mut vistos = BTreeSet::new();
    for (nome, b) in estados() {
        let ids = numeros(b);
        vistos.extend(ids.iter().copied());
        let (mut velha, mut nova) = (abre(false, b), abre(true, b));
        for id in ids {
            for texto in ["0.04", "500", "-500"] {
                let ev = velha.host.type_into_number(id, texto);
                let a = velha.aplica(ev);
                let ev = nova.host.type_into_number(id, texto);
                let n = nova.aplica(ev);
                assert_eq!(a, n, "{nome}: {id:?} com {texto}");
            }
        }
    }
    assert!(vistos.len() >= 40, "viu {} números", vistos.len());
}

/// Cada caixa ligada mostra o que o pincel tem, pela lei da pista, e tem a faixa de antes.
#[test]
fn each_box_shows_the_brush_value_and_the_range_of_before() {
    let mut b = digital();
    b.spacing = 0.37;
    b.stabilizer = 0.21;
    b.jitter_scale = 0.44;
    b.marca[6] = 0.5;
    let (velha, nova) = (abre(false, b), abre(true, b));
    for (id, esperado) in [
        (p::PAINTER_BRUSH_SPACING_CHIP, 0.37),
        (p::PAINTER_BRUSH_STABILIZE_CHIP, 0.21),
        (p::PAINTER_BRUSH_JITTER_SCALE_CHIP, 0.44),
        (p::PAINTER_MARCA_CHIPS[6], 0.5),
    ] {
        let v = nova.host.store().number_value(id).expect("semeado");
        assert!((v - esperado).abs() < 1e-3, "{id:?}: {v}");
    }
    for id in numeros(b) {
        assert_eq!(
            velha.host.store().number_range(id),
            nova.host.store().number_range(id),
            "faixa de {id:?}"
        );
    }
    for id in p::PAINTER_PEN_FIELDS {
        if nova.rect(id).is_some() && id != p::PAINTER_PEN_ROTATION {
            assert_eq!(
                velha.host.store().number_value(id),
                nova.host.store().number_value(id),
                "{id:?}"
            );
        }
    }
}

#[test]
fn scrubbing_each_number_moves_only_it_and_escape_gives_it_back() {
    for b in [digital(), figura(StrokeMethod::Arc)] {
        let ids = numeros(b);
        assert!(ids.len() >= 20, "só {}", ids.len());
        for id in &ids {
            let le = |m: &Mesa, i: NodeId| m.host.store().number_value(i).expect("semeado");
            let mut mesa = abre(true, b);
            let antes = le(&mesa, *id);
            let outros: Vec<(NodeId, f64)> = ids
                .iter()
                .filter(|o| *o != id)
                .map(|o| (*o, le(&mesa, *o)))
                .collect();
            let mut ev = mesa.arrasta_a_pega(*id, 60.0);
            let mut no_gesto = le(&mesa, *id);
            if (no_gesto - antes).abs() < 1e-9 {
                mesa = abre(true, b);
                ev = mesa.arrasta_a_pega(*id, -60.0);
                no_gesto = le(&mesa, *id);
            }
            assert!((no_gesto - antes).abs() > 1e-9, "{id:?} não se moveu");
            let accoes = mesa.aplica(ev);
            assert!(
                !accoes.is_empty(),
                "{id:?}: o painel não avisou a ferramenta"
            );
            for (o, v) in &outros {
                assert!(
                    (le(&mesa, *o) - v).abs() < 1e-9,
                    "{o:?} mexeu-se com {id:?}"
                );
            }
            mesa.quadro();
            assert!(
                (le(&mesa, *id) - no_gesto).abs() < 1e-6,
                "a semente puxou {id:?}"
            );
            let _ = esc(&mut mesa);
            assert!((le(&mesa, *id) - antes).abs() < 1e-6, "{id:?}: o Esc");
        }
    }
}

#[test]
fn every_click_sends_the_same_action_as_the_old_panel() {
    let mut vistos = 0;
    let medem = [
        "digital",
        "line",
        "ellipse",
        "free hand",
        "grid",
        "anchored",
    ];
    for (nome, b) in estados().into_iter().filter(|(n, _)| medem.contains(n)) {
        let mesa = abre(true, b);
        let cliques: Vec<NodeId> = do_corpo(b)
            .into_iter()
            .filter(|id| {
                let s = mesa.host.store();
                s.card_field(*id).is_none()
                    && !matches!(s.get(*id), Some(InteractiveState::NumberInput { .. }))
            })
            .collect();
        for id in cliques {
            let velho = abre(false, b).clica(id);
            let novo = abre(true, b).clica(id);
            assert_eq!(velho, novo, "{nome}: {id:?}");
            vistos += 1;
        }
    }
    assert!(vistos >= 20, "viu {vistos} cliques");
}

#[test]
fn each_list_opens_under_its_box_and_its_option_sends_the_same() {
    for (caixa, opcao) in [
        (
            core_ids::PAINTER_BRUSH_STROKE_METHOD,
            p::painter_brush_stroke_method_option_id(0),
        ),
        (
            p::PAINTER_BRUSH_JITTER_UNIT,
            p::painter_brush_jitter_unit_option_id(1),
        ),
        (
            p::PAINTER_PEN_ROTATION,
            p::painter_pen_rotation_option_id(1),
        ),
    ] {
        let escolhe = |cartoes: bool| {
            let mut mesa = abre(cartoes, digital());
            let chip = mesa.rect(caixa).expect("a caixa");
            let _ = mesa.clica(caixa);
            mesa.assenta();
            let r = mesa.rect(opcao).expect("a opção aberta");
            assert!(r.y >= chip.y, "{caixa:?} abriu acima: {chip:?} → {r:?}");
            mesa.clica(opcao)
        };
        let (velho, novo) = (escolhe(false), escolhe(true));
        assert!(!novo.is_empty(), "{caixa:?}: nada no barramento");
        assert_eq!(velho, novo, "{caixa:?}");
    }
}

/// O Reset do Stroke é um verbo; o do Pen é um menu, com as duas opções do cabeçalho de antes.
#[test]
fn the_reset_lives_in_the_header_and_the_pen_one_is_still_a_menu() {
    let id = p::PAINTER_BRUSH_STROKE_RESET;
    let (velho, novo) = (
        abre(false, digital()).clica(id),
        abre(true, digital()).clica(id),
    );
    assert!(!novo.is_empty());
    assert_eq!(velho, novo);
    let escolhe = |cartoes: bool| {
        let mut mesa = abre(cartoes, digital());
        let _ = mesa.clica(p::PAINTER_PEN_RESET);
        mesa.assenta();
        let opcao = p::painter_pen_reset_option_id(p::PAINTER_PEN_RESET_NEUTRO);
        assert!(
            mesa.rect(opcao).is_some(),
            "o menu não abriu (cartões: {cartoes})"
        );
        mesa.clica(opcao)
    };
    let (velho, novo) = (escolhe(false), escolhe(true));
    assert!(!novo.is_empty());
    assert_eq!(velho, novo);
}

#[test]
fn the_jitter_title_folds_its_rows() {
    let mut m = abre(true, digital());
    assert!(m.rect(p::PAINTER_BRUSH_JITTER_UNIT).is_some());
    let _ = m.clica(JITTER);
    m.assenta();
    assert!(m.rect(p::PAINTER_BRUSH_JITTER_UNIT).is_none());
    assert!(m.rect(p::PAINTER_BRUSH_INPUT_SAMPLES_CHIP).is_some());
}

#[test]
fn each_control_keeps_its_animation_mark() {
    for (nome, b) in estados() {
        let ids = do_painel_de_antes(b);
        let alvos = |cartoes: bool| -> BTreeSet<NodeId> {
            set_key_marks(std::collections::BTreeMap::new());
            for id in &ids {
                add_key_mark(*id, KeyMark::Chave);
            }
            let m = abre(cartoes, b);
            let fora = m
                .ids()
                .into_iter()
                .filter(|i| key_mark_target(*i).is_some() && ids.contains(i))
                .collect();
            set_key_marks(std::collections::BTreeMap::new());
            fora
        };
        let (velho, novo) = (alvos(false), alvos(true));
        let faltam: Vec<_> = velho.difference(&novo).collect();
        assert!(faltam.is_empty(), "{nome}: sem marca {faltam:?}");
    }
}

#[test]
fn no_control_of_these_cards_overlaps_another() {
    for (nome, b) in estados() {
        sem_sobrepor(&abre(true, b), &do_corpo(b), nome);
    }
}

#[test]
fn short_siblings_share_a_row() {
    let m = abre(true, digital());
    for (x, y) in [
        (
            p::PAINTER_BRUSH_SPACING_CHIP,
            p::PAINTER_BRUSH_CONTINUOUS_SPACING,
        ),
        (
            p::PAINTER_BRUSH_JITTER_UNIT,
            p::PAINTER_BRUSH_SPRAY_COUNT_CHIP,
        ),
        (p::PAINTER_PEN_FIELDS[0], p::PAINTER_PEN_FIELDS[1]),
    ] {
        na_mesma_linha(&m, x, y);
    }
    let arco = abre(true, figura(StrokeMethod::Arc));
    na_mesma_linha(
        &arco,
        p::PAINTER_BRUSH_OFFSET_CHIP,
        p::PAINTER_BRUSH_OFFSET_TRIM,
    );
    na_mesma_linha(
        &arco,
        p::PAINTER_BRUSH_STROKE_APPLY,
        p::PAINTER_BRUSH_STROKE_APPLY_KEEP,
    );
}

#[test]
fn no_text_is_cut_and_no_row_sticks_in_any_state() {
    for (nome, b) in estados() {
        let (_, c) = censo::grava(|| abre(true, b));
        let cortados = c.textos_cortados();
        assert!(cortados.is_empty(), "{nome}: {cortados:#?}");
        let estes = censo::Censo {
            linhas: c
                .linhas
                .iter()
                .filter(|l| ABERTOS.contains(&l.secao))
                .cloned()
                .collect(),
            textos: Vec::new(),
            pecas: Vec::new(),
            listas_soltas: Vec::new(),
        };
        let grudadas = estes.linhas_grudadas();
        assert!(grudadas.is_empty(), "{nome}: {grudadas:#?}");
    }
}
