//! ⭐ **As linhas de dentro da peça aberta são do layout** (doc 17 item 10): com a instância aberta
//! na lane A, as linhas de dentro empurram B e C; o arrasto de uma fita de fora e a largada de um chip
//! leem a linha sob o ponteiro pelas MESMAS bandas que a pintura usa — gesto real sobre os rects
//! pintados.

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::cards::ferramenta;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_panel_timeline::state::{
    TimelinePanelState, caminho, drain_intents, set_current_timeline,
};
use ph2d_panel_timeline::tab::Tab;
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_timeline::{
    StackHost, StripId, StripSource, TimelineIntent as I, TimelineState, TimelineViewSnapshot,
};
use ph2d_ui_testkit::mesa::Mesa;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1930.0,
    h: 1040.0,
};

struct Cena {
    st: TimelineState,
    /// A instância de «Walk» `[1, 3)` na lane A.
    a: StripId,
    /// Uma fita de clipe `[4, 5)` na lane A.
    y: StripId,
}

/// «Walk» (duas lanes de dentro) e «Jump» vazio; lanes A, B, C.
fn cena() -> Cena {
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    let walk = doc.add_container("Walk".into());
    let dentro = StackHost::Container(walk);
    doc.add_lane_in(dentro, "Base".into()).unwrap();
    doc.add_lane_in(dentro, "Upper".into()).unwrap();
    doc.add_strip_to(dentro, 0, StripSource::Clip(0), 0.0, 2.0)
        .unwrap();
    doc.add_container("Jump".into());
    for n in ["A", "B", "C"] {
        doc.add_lane(n.into()).unwrap();
    }
    let src = StripSource::Container(u16::try_from(walk).unwrap());
    let a = doc
        .add_strip_to(StackHost::Document, 0, src, 1.0, 3.0)
        .unwrap();
    let y = doc
        .add_strip_to(StackHost::Document, 0, StripSource::Clip(0), 4.0, 5.0)
        .unwrap();
    Cena { st, a, y }
}

fn snap_de(st: &mut TimelineState) -> TimelineViewSnapshot {
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(st, &ph2d_core::Playhead::default(), false);
    snap
}

/// A mesa com a instância `a` aberta pelo clique (e o snapshot que a shell carimbaria).
fn aberta() -> (Mesa<TimelinePanel>, Cena) {
    ferramenta::force_cards_on_this_thread(Some(true));
    let mut c = cena();
    set_current_timeline(Some(snap_de(&mut c.st)));
    let mut m = Mesa::nova(TimelinePanelState {
        tab: Tab::Arrange,
        ..TimelinePanelState::default()
    });
    m.assenta_em(JANELA);
    let _ = m.clica(ids::timeline_strip_hit_id(0, c.a.0, 2));
    m.assenta_em(JANELA);
    c.st.no_lugar = caminho().1;
    assert_eq!(c.st.no_lugar, Some(c.a), "a instância abriu");
    set_current_timeline(Some(snap_de(&mut c.st)));
    m.assenta_em(JANELA);
    let _ = drain_intents();
    (m, c)
}

fn rect(m: &Mesa<TimelinePanel>, id: NodeId) -> Rect {
    m.rect(id).unwrap_or_else(|| panic!("{id:?} não pintado"))
}

fn meio_y(m: &Mesa<TimelinePanel>, id: NodeId) -> f32 {
    let r = rect(m, id);
    r.y + r.h * 0.5
}

fn fecha() {
    let _ = drain_intents();
    set_current_timeline(None);
    ferramenta::force_cards_on_this_thread(None);
}

/// Arrasta o corpo da fita `y` até à altura `ate_y` e devolve o último `MoveStrip` de FORA.
fn arrasta_a_fita(m: &mut Mesa<TimelinePanel>, c: &Cena, ate_y: f32) -> usize {
    let r = rect(m, ids::timeline_strip_hit_id(0, c.y.0, 2));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut ev = m.ponteiro(PointerKind::Down, cx, cy);
    ev.extend(m.ponteiro(PointerKind::Move, cx + 4.0, (cy + ate_y) * 0.5));
    ev.extend(m.ponteiro(PointerKind::Move, cx + 8.0, ate_y));
    ev.extend(m.ponteiro(PointerKind::Up, cx + 8.0, ate_y));
    let _ = m.aplica(ev);
    m.assenta_em(JANELA);
    drain_intents()
        .into_iter()
        .rev()
        .find_map(|i| match i {
            I::MoveStrip { to_lane, id, .. } if id == c.y => Some(to_lane),
            _ => None,
        })
        .expect("o arrasto da fita de fora")
}

/// ⭐⭐ Da lane A (aberta, duas linhas de dentro) para a B PINTADA três linhas abaixo: cai na B, não
/// na C; sobre uma linha de dentro fica na A (as linhas de dentro são da instância). *Mutação: o
/// arrasto de fora voltar a contar linhas inteiras (`lane_at`) ⇒ cai na C, e na B sobre a de dentro.*
#[test]
fn arrastar_de_fora_le_a_lane_pelas_bandas_pintadas() {
    let (mut m, c) = aberta();
    let b = meio_y(&m, ids::TIMELINE_LANE_ROW[1]);
    assert_eq!(arrasta_a_fita(&mut m, &c, b), 1, "a fita cai na lane B");
    let (mut m, c) = aberta();
    let dentro = meio_y(&m, ids::TIMELINE_NO_LUGAR_MUTE[0]);
    assert_eq!(
        arrasta_a_fita(&mut m, &c, dentro),
        0,
        "sobre a linha de dentro fica na lane da instância"
    );
    fecha();
}

/// Leva o chip a `(x, y)` e larga; devolve os intents.
fn larga_o_chip(m: &mut Mesa<TimelinePanel>, chip: NodeId, (x, y): (f32, f32)) -> Vec<I> {
    let r = rect(m, chip);
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let _ = m.ponteiro(PointerKind::Down, cx, cy);
    let _ = m.ponteiro(PointerKind::Move, cx + 12.0, cy + 12.0);
    let _ = m.ponteiro(PointerKind::Move, x, y);
    m.assenta_em(JANELA);
    let _ = m.ponteiro(PointerKind::Up, x, y);
    m.assenta_em(JANELA);
    drain_intents()
}

fn x_do_tempo(m: &Mesa<TimelinePanel>) -> f32 {
    rect(m, ids::TIMELINE_RULER).x + 300.0
}

/// ⭐⭐ O chip «Jump» largado na B pintada põe a peça na B; numa linha de DENTRO, a peça entra no
/// container (`NoLugar` com a lane de dentro). *Mutação: o alvo voltar a
/// `floor((y - topo) / h)` ⇒ a B vira a C e a de dentro vira uma lane de fora.*
#[test]
fn o_chip_largado_cai_na_linha_pintada_sob_o_ponteiro() {
    let (mut m, c) = aberta();
    let x = x_do_tempo(&m);
    let b = meio_y(&m, ids::TIMELINE_LANE_ROW[1]);
    let feitos = larga_o_chip(&mut m, ids::TIMELINE_CONT_OPT[1], (x, b));
    assert!(
        feitos
            .iter()
            .any(|i| matches!(i, I::AddStrip { lane: 1, .. })),
        "a peça na B: {feitos:?}"
    );

    let (mut m, _) = aberta();
    let dentro = meio_y(&m, ids::TIMELINE_NO_LUGAR_MUTE[1]);
    let feitos = larga_o_chip(&mut m, ids::TIMELINE_CONT_OPT[1], (x, dentro));
    assert!(
        feitos.iter().any(|i| matches!(i, I::NoLugar { instancia, intent }
            if *instancia == c.a
                && matches!(**intent, I::AddStrip { lane: 1, source: StripSource::Container(1), .. }))),
        "a peça entra na lane de dentro 1: {feitos:?}"
    );
    assert!(
        !feitos.iter().any(|i| matches!(i, I::AddStrip { .. })),
        "nenhuma peça de fora: {feitos:?}"
    );

    fecha();
}

/// ⭐ O container não se larga no próprio interior aberto (seria um ciclo). *Mutação: tirar o teste
/// do interior aberto do `dentro_de_si` ⇒ sai um `NoLugar { AddStrip }` de «Walk» dentro de «Walk».*
#[test]
fn o_container_nao_se_larga_no_proprio_interior() {
    let (mut m, _) = aberta();
    let x = x_do_tempo(&m);
    let dentro = meio_y(&m, ids::TIMELINE_NO_LUGAR_MUTE[0]);
    let feitos = larga_o_chip(&mut m, ids::TIMELINE_CONT_OPT[0], (x, dentro));
    assert!(
        !feitos
            .iter()
            .any(|i| matches!(i, I::AddStrip { .. } | I::NoLugar { .. })),
        "recusa: {feitos:?}"
    );
    fecha();
}

/// ⭐ O campo de renome de uma lane tem a altura da linha PINTADA (a banda dos cartões), e mora na
/// lane B empurrada pelas linhas de dentro. *Mutação: o campo voltar a `ROW_H_PX` ⇒ vermelho.*
#[test]
fn o_renome_da_lane_tem_a_altura_da_linha_pintada() {
    use ph2d_panel_timeline::state::{ClipRename, RenameKind};
    let (mut m, _) = aberta();
    m.st.clip_rename = Some(ClipRename {
        kind: RenameKind::Lane,
        index: 1,
        opened: false,
    });
    m.assenta_em(JANELA);
    m.assenta_em(JANELA);
    let campo = rect(&m, ids::TIMELINE_CLIP_RENAME_INPUT);
    let linha = rect(&m, ids::TIMELINE_LANE_ROW[1]);
    assert!(
        (campo.y - linha.y).abs() < 0.5 && (campo.h - linha.h).abs() < 0.5,
        "o campo {campo:?} cobre a linha {linha:?}"
    );
    fecha();
}

/// ⭐⭐ O chip largado numa linha de DENTRO de uma instância a velocidade 2 que começa em 1,01 s: a
/// peça entra no quadro do container e a velocidade 1 lá dentro. *Mutação: a largada encaixar na
/// cena ou não escalar a duração ⇒ início fora da grelha, ou a peça a meia velocidade.*
#[test]
fn o_chip_largado_dentro_cai_na_grelha_do_container_a_velocidade_um() {
    let (mut m, mut c) = aberta();
    let mut ph = ph2d_core::Playhead::default();
    for i in [
        I::MoveStrip {
            lane: 0,
            to_lane: 0,
            id: c.a,
            t_start: 1.01,
        },
        I::SetStripSpeed {
            lane: 0,
            id: c.a,
            speed: 2.0,
        },
    ] {
        ph2d_timeline::apply_intent(&mut c.st, &mut ph, i);
    }
    // «Jump» com 2 s de conteúdo, para a duração da peça ser a dele.
    let jump = StackHost::Container(1);
    c.st.doc.add_lane_in(jump, "J".into()).unwrap();
    c.st.doc
        .add_strip_to(jump, 0, StripSource::Clip(0), 0.0, 2.0)
        .unwrap();
    c.st.doc.container_stack_mut(1).unwrap()[0].strips[0].src_out = 2.0;
    set_current_timeline(Some(snap_de(&mut c.st)));
    m.assenta_em(JANELA);
    let fps = snap_de(&mut c.st).fps;
    let x = rect(&m, ids::timeline_strip_hit_id(0, c.a.0, 2)).x + 13.0;
    let dentro = meio_y(&m, ids::TIMELINE_NO_LUGAR_MUTE[1]);
    for i in larga_o_chip(&mut m, ids::TIMELINE_CONT_OPT[1], (x, dentro)) {
        ph2d_timeline::apply_intent(&mut c.st, &mut ph, i);
    }
    let lane = &c.st.doc.container_stack(0).expect("Walk")[1];
    let s = lane
        .strips
        .first()
        .expect("a peça entrou na lane de dentro 1");
    let q = s.t_start * fps;
    assert!(
        (q - q.round()).abs() < 1e-6,
        "início {} fora da grelha",
        s.t_start
    );
    assert!(
        (s.t_end - s.t_start - 2.0).abs() < 1e-9 && (s.speed - 1.0).abs() < 1e-9,
        "a peça dura o container (2 s) a velocidade 1 lá dentro: {s:?}"
    );
    fecha();
}
