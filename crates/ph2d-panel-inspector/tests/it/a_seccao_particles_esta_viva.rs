//! ⭐⭐⭐ **A secção PARTICLES é PINTADA e está VIVA sob o dedo** (TOP-20 #18, W3).
//!
//! ⚠️ **O gesto é REAL** (`Down` + `Up` no rectângulo que a pintura registou), e é o que falta à
//! cena de smoke: a tela virtual onde ela é fotografada não aceita eventos sintéticos, então o
//! clique só se prova aqui. *Um `WidgetEvent::Click` sintético passa sobre um controlo ausente do
//! `populate`* — e esta secção tinha exactamente esse buraco nas duas amostras de cor.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::particles_edits::{
    InspectorParticlesInfo, PARTICLES_NUMBERS, PARTICLES_TEXTS, ParticlesFieldEdit as E,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::{PointerButton, PointerEvent, PointerKind, PointerSource};
use ph2d_panel_inspector::ids;
use ph2d_panel_inspector::{InspectorPanel, InspectorState, set_current_inspector_particles};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 2600.0,
};
const SEC: u128 = 1_000_000_000;
const BITS: u64 = 0xFA15CA;

fn pointer(kind: PointerKind, x: f32, y: f32, t: u128) -> PointerEvent {
    PointerEvent {
        kind,
        x,
        y,
        button: PointerButton::Primary,
        source: PointerSource::Mouse,
        pressure: 1.0,
        timestamp_ns: t,
    }
}

/// O emissor da cena de smoke: uma fonte a emitir, num disco, no espaço do mundo.
fn info() -> InspectorParticlesInfo {
    InspectorParticlesInfo {
        entity_bits: BITS,
        emitting: true,
        one_shot: false,
        amount: 16.0,
        life: 1.0,
        life_random: 0.0,
        explosiveness: 0.0,
        prewarm: 0.0,
        time_scale: 1.0,
        seed: 0.0,
        shape: 1,
        shape_size: [0.5, 0.5],
        speed: 4.0,
        speed_random: 0.0,
        angle: 90.0,
        spread: 30.0,
        gravity: [0.0, -9.8],
        damping: 0.0,
        size: 0.15,
        size_random: 0.0,
        size_end: 1.0,
        color: [1.0, 1.0, 1.0, 1.0],
        color_end: [1.0, 0.0, 0.0, 0.0],
        space: 0,
        start_on: String::new(),
        stop_on: String::new(),
        restart_on: String::new(),
        finished_signal: String::new(),
        clock_playing: true,
        alive: 7,
        selected_count: 1,
    }
}

fn host(i: Option<InspectorParticlesInfo>) -> (MockPanelHost, InspectorState) {
    let h = MockPanelHost::with_panel::<InspectorPanel>();
    set_current_inspector_particles(i);
    (h, InspectorState::default())
}

fn rect_de(rects: &[(ph2d_a11y::NodeId, Rect)], id: ph2d_a11y::NodeId) -> Option<Rect> {
    rects.iter().find(|(n, _)| *n == id).map(|(_, r)| *r)
}

/// Um clique real no centro de `id`; devolve as acções que chegaram ao barramento.
fn clica(
    h: &mut MockPanelHost,
    st: &mut InspectorState,
    rects: &[(ph2d_a11y::NodeId, Rect)],
    id: ph2d_a11y::NodeId,
) -> Vec<EditorAction> {
    let r = rect_de(rects, id).unwrap_or_else(|| panic!("{id:?} não foi pintado"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = h.drained_actions();
    let mut evs = h.dispatch_pointer_event(pointer(PointerKind::Down, cx, cy, SEC));
    evs.extend(h.dispatch_pointer_event(pointer(PointerKind::Up, cx, cy, SEC + SEC / 100)));
    assert!(
        !evs.is_empty(),
        "o ponteiro sobre {id:?} não virou evento — pintado e ausente do `populate`"
    );
    for ev in evs {
        h.apply_panel_event::<InspectorPanel>(st, ev);
    }
    h.drained_actions()
}

fn edicoes(acoes: &[EditorAction]) -> Vec<E> {
    acoes
        .iter()
        .filter_map(|a| match a {
            EditorAction::InspectorParticlesEdit { entity_bits, edit } => {
                assert_eq!(*entity_bits, BITS);
                Some(edit.clone())
            }
            _ => None,
        })
        .collect()
}

/// ⭐⭐ **A secção inteira é pintada** — as 19 linhas de número, os 4 sinais, as duas caixas, os
/// dois segmentados e as duas amostras de cor.
///
/// **Mutação que deve sangrar:** cortar uma linha do `corpo` do pintor.
#[test]
fn a_seccao_inteira_e_pintada() {
    let (mut h, mut st) = host(Some(info()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let pintado = |id| rect_de(&rects, id).is_some();
    for (i, &id) in ids::INSP_PART_NUM.iter().enumerate() {
        assert!(
            pintado(id),
            "o número {i} ({:?}) não foi pintado",
            PARTICLES_NUMBERS[i]
        );
    }
    for (i, &id) in ids::INSP_PART_TEXT.iter().enumerate() {
        assert!(
            pintado(id),
            "o sinal {i} ({:?}) não foi pintado",
            PARTICLES_TEXTS[i]
        );
    }
    for id in [ids::INSP_PART_EMITTING, ids::INSP_PART_ONE_SHOT] {
        assert!(pintado(id));
    }
    for id in ids::INSP_PART_SHAPE {
        assert!(pintado(id));
    }
    for id in ids::INSP_PART_SPACE {
        assert!(pintado(id));
    }
    for id in [ids::INSP_PART_COLOR, ids::INSP_PART_COLOR_END] {
        assert!(pintado(id), "uma amostra de cor não foi pintada");
    }
    set_current_inspector_particles(None);
}

/// ⚠️ **Sem o componente não se pinta nada da secção** — ADR-0166.
#[test]
fn sem_o_componente_nada_da_seccao_e_pintado() {
    let (mut h, mut st) = host(None);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(rect_de(&rects, ids::INSP_PART_NUM[0]).is_none());
    assert!(rect_de(&rects, ids::INSP_PART_EMITTING).is_none());
    assert!(rect_de(&rects, ids::INSP_PART_COLOR).is_none());
}

/// ⭐⭐ **As duas medidas da forma só existem fora do PONTO** — num ponto elas são inertes, e um
/// controlo vivo que não muda nada é a doença que esta secção evita por construção.
///
/// **Mutação que deve sangrar:** pintar as duas linhas sempre.
#[test]
fn as_medidas_da_forma_somem_num_ponto() {
    let com_disco = info();
    let (mut h, mut st) = host(Some(com_disco));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(
        rect_de(&rects, ids::INSP_PART_NUM[7]).is_some(),
        "Disc: a largura"
    );
    assert!(
        rect_de(&rects, ids::INSP_PART_NUM[8]).is_some(),
        "Disc: a altura"
    );

    let (mut h, mut st) = host(Some(InspectorParticlesInfo { shape: 0, ..info() }));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert!(
        rect_de(&rects, ids::INSP_PART_NUM[7]).is_none(),
        "Point: a largura"
    );
    assert!(
        rect_de(&rects, ids::INSP_PART_NUM[8]).is_none(),
        "Point: a altura"
    );
    set_current_inspector_particles(None);
}

/// ⭐⭐⭐ **Um chip de FORMA chega ao barramento com o índice dele** — a lei da fileira, e o
/// discriminador é o índice: um `position` sobre a tabela errada manda o vizinho.
///
/// **Mutação que deve sangrar:** tirar os `INSP_PART_SHAPE` do `populate_particles` (o clique morre
/// no `is_focusable`), ou trocar a tabela lida no despacho.
#[test]
fn um_chip_de_forma_chega_ao_barramento_com_o_indice_dele() {
    let (mut h, mut st) = host(Some(info()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let a = clica(&mut h, &mut st, &rects, ids::INSP_PART_SHAPE[3]);
    assert_eq!(edicoes(&a), [E::Shape(3)]);
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let a = clica(&mut h, &mut st, &rects, ids::INSP_PART_SPACE[1]);
    assert_eq!(edicoes(&a), [E::Space(1)]);
    set_current_inspector_particles(None);
}

/// ⭐ **A caixa manda o CONTRÁRIO do que o instantâneo diz** — ela é semeada pela `sync`, nunca
/// pelo valor de partida do `populate`.
///
/// **Mutação que deve sangrar:** tirar a semente das caixas do `sync_particles`.
#[test]
fn a_caixa_manda_o_contrario_do_instantaneo() {
    let (mut h, mut st) = host(Some(info()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    let a = clica(&mut h, &mut st, &rects, ids::INSP_PART_EMITTING);
    assert_eq!(edicoes(&a), [E::Emitting(false)]);
    set_current_inspector_particles(None);
}

/// ⭐⭐⭐ **Uma amostra de cor ABRE o selector** — e sem o registo dela no `populate` o clique morre
/// em silêncio, com o rectângulo pintado na mesma. *É a espécie de controlo morto que nenhum gate
/// de registo apanha.*
///
/// **Mutação que deve sangrar:** tirar as duas amostras do `populate_particles`.
#[test]
fn uma_amostra_de_cor_abre_o_selector() {
    let (mut h, mut st) = host(Some(info()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    // ⚠️ O clique numa amostra NÃO é uma edição: ele arma o selector, e a cor volta pela semente.
    let a = clica(&mut h, &mut st, &rects, ids::INSP_PART_COLOR_END);
    assert_eq!(edicoes(&a), [], "abrir o selector não escreve no documento");
    assert_eq!(
        h.store().picker_target(),
        Some(ids::INSP_PART_COLOR_END),
        "o selector não ficou apontado à amostra"
    );
    // ⭐ E a semente dela é a cor do DOCUMENTO, não a do vizinho.
    assert_eq!(
        h.store().widget_color(ids::INSP_PART_COLOR_END),
        Some([255, 0, 0, 0])
    );
    set_current_inspector_particles(None);
}

/// ⭐⭐⭐ **A cor ESCOLHIDA chega ao documento** — a segunda metade, e a que morre em silêncio: o
/// clique arma o selector, o selector escreve na tabela lateral, e é a SEMENTE do painel que leva
/// a divergência ao barramento. *Sem este braço o fio está completo até ao painel e acaba ali.*
///
/// **Mutação que deve sangrar:** apagar o `push` do braço `picker == Some(id)` da `sync_particles`.
#[test]
fn a_cor_escolhida_chega_ao_documento() {
    let (mut h, mut st) = host(Some(info()));
    let rects = h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    clica(&mut h, &mut st, &rects, ids::INSP_PART_COLOR);
    // O selector devolve verde — é o que um arrasto na roda teria deixado na tabela lateral.
    h.set_widget_color(ids::INSP_PART_COLOR, [0, 255, 0, 255]);
    let _ = h.drained_actions();
    h.paint::<InspectorPanel>(&mut st, VIEWPORT);
    assert_eq!(
        edicoes(&h.drained_actions()),
        [E::Color(false, [0.0, 1.0, 0.0, 1.0])],
        "a cor escolhida não chegou ao emissor"
    );
    set_current_inspector_particles(None);
}
