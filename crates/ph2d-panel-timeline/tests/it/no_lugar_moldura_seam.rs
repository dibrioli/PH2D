//! ⭐ **A moldura acompanha a peça** (smoke do dono 11/11): com a instância aberta no lugar, o
//! arrasto PINTADO de uma fita de dentro, aplicado quadro a quadro como a shell o aplica, muda o
//! container — e a instância que mostrava a peça INTEIRA continua a mostrá-la, à MESMA velocidade.

use ph2d_editor_core::panel::cards::ferramenta;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_panel_timeline::state::{
    TimelinePanelState, caminho, drain_intents, set_current_timeline,
};
use ph2d_panel_timeline::tab::Tab;
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_timeline::{StackHost, StripId, StripSource, TimelineState, TimelineViewSnapshot};
use ph2d_ui_testkit::mesa::Mesa;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1600.0,
    h: 900.0,
};

struct Cena {
    st: TimelineState,
    c: usize,
    a: StripId,
    b: StripId,
    jump: StripId,
}

/// A montagem do `PH2D_NEST_SMOKE=4` em miniatura: Base walk `[0, 1.25)` + jump `[1, 2)`, Upper
/// body swing `[1.25, 1.75)`; o container 2 s, posto duas vezes na lane Base da cena.
fn cena() -> Cena {
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    let c = doc.add_container("Run & jump".into());
    let dentro = StackHost::Container(c);
    doc.add_lane_in(dentro, "Base".into()).unwrap();
    doc.add_lane_in(dentro, "Upper body".into()).unwrap();
    let clip = StripSource::Clip(0);
    doc.add_strip_to(dentro, 0, clip, 0.0, 1.25).unwrap();
    let jump = doc.add_strip_to(dentro, 0, clip, 1.0, 2.0).unwrap();
    doc.add_strip_to(dentro, 1, clip, 1.25, 1.75).unwrap();
    doc.add_lane("Base".into()).unwrap();
    doc.add_lane("FX".into()).unwrap();
    let src = StripSource::Container(u16::try_from(c).unwrap());
    let a = doc
        .add_strip_to(StackHost::Document, 0, src, 1.0, 3.0)
        .unwrap();
    let b = doc
        .add_strip_to(StackHost::Document, 0, src, 4.0, 6.0)
        .unwrap();
    Cena { st, c, a, b, jump }
}

fn snap_de(st: &mut TimelineState) -> TimelineViewSnapshot {
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(st, &ph2d_core::Playhead::default(), false);
    snap
}

/// O que a shell faz a cada quadro: drena, aplica ao modelo, carimba a peça aberta, republica.
fn quadro(m: &mut Mesa<TimelinePanel>, c: &mut Cena) {
    let mut ph = ph2d_core::Playhead::default();
    for i in drain_intents() {
        ph2d_timeline::apply_intent(&mut c.st, &mut ph, i);
    }
    c.st.no_lugar = caminho().1;
    set_current_timeline(Some(snap_de(&mut c.st)));
    m.assenta_em(JANELA);
}

fn mesa() -> (Mesa<TimelinePanel>, Cena) {
    ferramenta::force_cards_on_this_thread(Some(true));
    let mut c = cena();
    set_current_timeline(Some(snap_de(&mut c.st)));
    let mut m = Mesa::nova(TimelinePanelState {
        tab: Tab::Arrange,
        ..TimelinePanelState::default()
    });
    m.assenta_em(JANELA);
    let _ = drain_intents();
    let _ = m.clica(ids::timeline_strip_hit_id(0, c.a.0, 2));
    m.assenta_em(JANELA);
    quadro(&mut m, &mut c);
    assert_eq!(caminho().1, Some(c.a), "a peça abriu");
    (m, c)
}

/// Arrasta o alvo `id` em `dx` px, em `passos` quadros aplicados um a um.
fn arrasta(m: &mut Mesa<TimelinePanel>, c: &mut Cena, id: ph2d_editor_core::NodeId, dx: f32) {
    let r = m.rect(id).unwrap_or_else(|| panic!("{id:?} pintado"));
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let ev = m.ponteiro(PointerKind::Down, cx, cy);
    let _ = m.aplica(ev);
    quadro(m, c);
    let passos = 12;
    for k in 1..=passos {
        let x = cx + dx * k as f32 / passos as f32;
        let ev = m.ponteiro(PointerKind::Move, x, cy);
        let _ = m.aplica(ev);
        quadro(m, c);
    }
    let ev = m.ponteiro(PointerKind::Up, cx + dx, cy);
    let _ = m.aplica(ev);
    quadro(m, c);
}

fn fora(c: &Cena, id: StripId) -> ph2d_timeline::ClipStrip {
    c.st.doc.stack()[0]
        .strips
        .iter()
        .find(|s| s.id == id)
        .unwrap()
        .clone()
}

/// Píxeis por segundo, medidos na fita PINTADA de dentro (jump tem 1 s).
fn px_por_s(m: &Mesa<TimelinePanel>, jump: StripId) -> f32 {
    m.rect(ids::timeline_strip_hit_id(2, jump.0, 2))
        .expect("jump pintado")
        .w
}

#[test]
fn arrastar_o_jump_para_la_do_fim_estica_a_moldura_e_nao_mexe_na_velocidade() {
    let (mut m, mut c) = mesa();
    let pps = px_por_s(&m, c.jump);
    // 1,5 s para a direita: o jump passa a `[2.5, 3.5)`, o container a 3,5 s.
    let alvo = ids::timeline_strip_hit_id(2, c.jump.0, 2);
    arrasta(&mut m, &mut c, alvo, 1.5 * pps);
    let len = c.st.doc.container_length_seconds(c.c);
    assert!((len - 3.5).abs() < 0.05, "o container cresceu: {len}");
    for id in [c.a, c.b] {
        let s = fora(&c, id);
        assert!(
            (s.speed - 1.0).abs() < 1e-9,
            "a velocidade de fora fica: {}",
            s.speed
        );
        assert!(
            (s.src_out - len).abs() < 1e-9 && (s.span() - len).abs() < 1e-9,
            "a moldura acompanha a peça: slice {} span {}",
            s.slice(),
            s.span()
        );
    }
    let _ = drain_intents();
    set_current_timeline(None);
    ferramenta::force_cards_on_this_thread(None);
}

/// Cada alça da fita de dentro (corpo, cortes, esticões) arrastada a direito: a de FORA nunca
/// muda de velocidade.
#[test]
fn nenhuma_alca_de_dentro_retima_a_instancia_de_fora() {
    for edge in [0_u8, 1, 2, 5, 6] {
        let (mut m, mut c) = mesa();
        let pps = px_por_s(&m, c.jump);
        let alvo = ids::timeline_strip_hit_id(2, c.jump.0, edge);
        arrasta(&mut m, &mut c, alvo, 0.8 * pps);
        for id in [c.a, c.b] {
            let s = fora(&c, id);
            assert!(
                (s.speed - 1.0).abs() < 1e-9,
                "alça {edge}: a velocidade de fora ficou {}",
                s.speed
            );
        }
        let _ = drain_intents();
        set_current_timeline(None);
        ferramenta::force_cards_on_this_thread(None);
    }
}
