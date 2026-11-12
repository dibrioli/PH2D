//! ⭐ **O container aberto no lugar** (doc 17 item 10, maquete C2): o clique no corpo PINTADO de uma
//! instância abre-a (só nos cartões, só no Arrange); as lanes de dentro entram logo abaixo e empurram
//! as de baixo; arrastar uma fita de dentro sai embrulhado em `NoLugar`; as irmãs só dão a dica.

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
    w: 1600.0,
    h: 900.0,
};

struct Cena {
    st: TimelineState,
    /// A instância que se abre `[1, 3)` e a irmã `[4, 6)`, na lane 0.
    a: StripId,
    /// A fita de dentro, na lane de dentro 0.
    x: StripId,
}

fn cena() -> Cena {
    let mut st = TimelineState::new();
    let doc = &mut st.doc;
    let c = doc.add_container("Run & jump".into());
    let dentro = StackHost::Container(c);
    doc.add_lane_in(dentro, "Base".into()).unwrap();
    doc.add_lane_in(dentro, "Upper body".into()).unwrap();
    let x = doc
        .add_strip_to(dentro, 0, StripSource::Clip(0), 0.0, 2.0)
        .unwrap();
    doc.set_container_loop(c, Some((0.0, 2.0)), false);
    doc.add_lane("Base".into()).unwrap();
    doc.add_lane("FX".into()).unwrap();
    let src = StripSource::Container(u16::try_from(c).unwrap());
    let a = doc
        .add_strip_to(StackHost::Document, 0, src, 1.0, 3.0)
        .unwrap();
    doc.add_strip_to(StackHost::Document, 0, src, 4.0, 6.0)
        .unwrap();
    doc.add_strip_to(StackHost::Document, 1, StripSource::Clip(0), 0.0, 1.0)
        .unwrap();
    Cena { st, a, x }
}

fn snap_de(st: &mut TimelineState) -> TimelineViewSnapshot {
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(st, &ph2d_core::Playhead::default(), false);
    snap
}

fn mesa(cartoes: bool) -> (Mesa<TimelinePanel>, Cena) {
    ferramenta::force_cards_on_this_thread(Some(cartoes));
    let mut c = cena();
    set_current_timeline(Some(snap_de(&mut c.st)));
    let mut m = Mesa::nova(TimelinePanelState {
        tab: Tab::Arrange,
        ..TimelinePanelState::default()
    });
    m.assenta_em(JANELA);
    let _ = drain_intents();
    (m, c)
}

/// Abre a instância `a` pelo clique e devolve o snapshot que a shell carimbaria.
fn abre(m: &mut Mesa<TimelinePanel>, c: &mut Cena) {
    let _ = m.clica(ids::timeline_strip_hit_id(0, c.a.0, 2));
    m.assenta_em(JANELA);
    c.st.no_lugar = caminho().1;
    set_current_timeline(Some(snap_de(&mut c.st)));
    m.assenta_em(JANELA);
    let _ = drain_intents();
}

fn arrasta(m: &mut Mesa<TimelinePanel>, r: Rect, dx: f32, dy: f32) {
    let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let mut ev = m.ponteiro(PointerKind::Down, cx, cy);
    ev.extend(m.ponteiro(PointerKind::Move, cx + dx * 0.5, cy + dy * 0.5));
    ev.extend(m.ponteiro(PointerKind::Move, cx + dx, cy + dy));
    ev.extend(m.ponteiro(PointerKind::Up, cx + dx, cy + dy));
    let _ = m.aplica(ev);
    m.assenta_em(JANELA);
}

fn fecha() {
    let _ = drain_intents();
    set_current_timeline(None);
    ferramenta::force_cards_on_this_thread(None);
}

#[test]
fn o_clique_na_instancia_abre_e_as_lanes_de_dentro_empurram_as_de_baixo() {
    let (mut m, mut c) = mesa(true);
    let fx_antes = m.rect(ids::TIMELINE_LANE_ROW[1]).expect("a lane FX").y;
    abre(&mut m, &mut c);
    assert_eq!(
        caminho().1,
        Some(c.a),
        "a escolhida que toca um container abre"
    );
    let fx = m.rect(ids::TIMELINE_LANE_ROW[1]).expect("a lane FX").y;
    let h = ph2d_tokens::TIMELINE_FAIXA_H_PX;
    assert!(
        (fx - fx_antes - 2.0 * h).abs() < 0.5,
        "duas lanes de dentro empurram a FX duas faixas: {fx_antes} → {fx}"
    );
    let dentro = m
        .rect(ids::timeline_strip_hit_id(2, c.x.0, 2))
        .expect("a fita de dentro pintada");
    let base = m.rect(ids::TIMELINE_LANE_ROW[0]).expect("a lane Base");
    assert!(
        (dentro.y - base.y - h).abs() < h,
        "a lane de dentro mora logo abaixo da da instância"
    );
    fecha();
}

#[test]
fn arrastar_dentro_sai_embrulhado_com_a_lane_de_dentro() {
    let (mut m, mut c) = mesa(true);
    abre(&mut m, &mut c);
    let r = m
        .rect(ids::timeline_strip_hit_id(2, c.x.0, 2))
        .expect("a fita de dentro");
    arrasta(&mut m, r, 30.0, ph2d_tokens::TIMELINE_FAIXA_H_PX);
    let movs: Vec<_> = drain_intents()
        .into_iter()
        .filter_map(|i| match i {
            I::NoLugar { instancia, intent } => Some((instancia, *intent)),
            _ => None,
        })
        .collect();
    assert!(!movs.is_empty(), "o arrasto de dentro sai embrulhado");
    let (inst, ultimo) = movs.last().unwrap().clone();
    assert_eq!(inst, c.a);
    assert!(
        matches!(ultimo, I::MoveStrip { to_lane: 1, id, .. } if id == c.x),
        "desceu uma faixa = a lane de DENTRO 1: {ultimo:?}"
    );
    assert_eq!(caminho().1, Some(c.a), "editar dentro não fecha a peça");
    fecha();
}

#[test]
fn a_irma_so_da_a_dica_e_nao_se_arrasta() {
    let (mut m, mut c) = mesa(true);
    abre(&mut m, &mut c);
    let irma = ids::timeline_no_lugar_irma_id(0, c.x.0, 1);
    let r = m.rect(irma).expect("a cópia irmã pintada");
    assert_eq!(
        m.host.store().tooltip_for(irma),
        Some(ph2d_i18n::tr("panel.timeline.no_lugar.same_piece"))
    );
    // Nem quina nem alça: cada canto da irmã é a dica, nunca um alvo de strip.
    let folga = 2.0;
    for (x, y) in [
        (r.x + folga, r.y + folga),
        (r.x + r.w - folga, r.y + folga),
        (r.x + folga, r.y + r.h - folga),
        (r.x + r.w - folga, r.y + r.h - folga),
        (r.x + r.w * 0.5, r.y + r.h * 0.5),
    ] {
        assert_eq!(
            m.host.hit_at(x, y),
            Some(irma),
            "a irmã não tem alça em ({x}, {y})"
        );
    }
    arrasta(&mut m, r, 30.0, 0.0);
    assert!(
        !drain_intents()
            .iter()
            .any(|i| matches!(i, I::NoLugar { .. } | I::MoveStrip { .. })),
        "a irmã não edita"
    );
    fecha();
}

#[test]
fn sem_cartoes_nada_abre() {
    let (mut m, mut c) = mesa(false);
    let _ = m.clica(ids::timeline_strip_hit_id(0, c.a.0, 2));
    m.assenta_em(JANELA);
    assert_eq!(caminho().1, None, "PH2D_CARDS=0 como hoje");
    c.st.no_lugar = Some(c.a);
    set_current_timeline(Some(snap_de(&mut c.st)));
    m.assenta_em(JANELA);
    assert!(
        m.rect(ids::timeline_strip_hit_id(2, c.x.0, 2)).is_none(),
        "nem com o modelo aberto o clássico pinta as de dentro"
    );
    fecha();
}

#[test]
fn o_esc_fecha_a_peca_aberta() {
    let (mut m, mut c) = mesa(true);
    abre(&mut m, &mut c);
    let _ = m.esc();
    m.assenta_em(JANELA);
    assert_eq!(caminho().1, None);
    fecha();
}

#[test]
fn a_barra_do_loop_aberta_e_a_do_container_embrulhada() {
    let (mut m, mut c) = mesa(true);
    abre(&mut m, &mut c);
    let r = m
        .rect(ids::timeline_loop_brace_id(2))
        .expect("a barra do loop do container");
    arrasta(&mut m, r, 20.0, 0.0);
    let ultimo = drain_intents()
        .into_iter()
        .rev()
        .find(|i| matches!(i, I::NoLugar { .. } | I::SetLoop { .. }))
        .expect("o arrasto do loop");
    assert!(
        matches!(&ultimo, I::NoLugar { instancia, intent }
            if *instancia == c.a && matches!(**intent, I::SetContainerLoop { .. })),
        "{ultimo:?}"
    );
    fecha();
}

/// ⭐⭐ O encaixe ao quadro DENTRO da peça aberta é na grelha do CONTAINER: com a instância a
/// velocidade 2 a começar em 1,01 s, arrastar o corpo, a quina vermelha ou a verde de uma fita de
/// dentro grava bordas que são múltiplos exatos de `1/fps` no tempo do container. *Mutação: o
/// `strip_drag::snapped_em` voltar a encaixar na cena ⇒ as bordas caem fora da grelha.*
#[test]
fn dentro_da_peca_aberta_o_encaixe_e_na_grelha_do_container() {
    for (edge, dx) in [(2u8, 37.0), (1, -37.0), (6, -37.0)] {
        let (mut m, mut c) = mesa(true);
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
        // A fatia que o retime segura (a cena não tem o clipe 0).
        let x = &mut c.st.doc.container_stack_mut(0).expect("o container")[0].strips[0];
        (x.src_in, x.src_out) = (0.0, 2.0);
        set_current_timeline(Some(snap_de(&mut c.st)));
        m.assenta_em(JANELA);
        abre(&mut m, &mut c);
        let fps = snap_de(&mut c.st).fps;
        assert!(fps > 0.0 && c.st.flags.frame_snap, "o encaixe armado");
        let r = m
            .rect(ids::timeline_strip_hit_id(2, c.x.0, edge))
            .unwrap_or_else(|| panic!("a alça {edge} da fita de dentro"));
        arrasta(&mut m, r, dx, 0.0);
        for i in drain_intents() {
            ph2d_timeline::apply_intent(&mut c.st, &mut ph, i);
        }
        let s = &c.st.doc.container_stack(0).expect("o container")[0].strips[0];
        assert_ne!(
            (s.t_start, s.t_end),
            (0.0, 2.0),
            "alça {edge}: a fita mexeu"
        );
        for t in [s.t_start, s.t_end] {
            let q = t * fps;
            assert!(
                (q - q.round()).abs() < 1e-6,
                "alça {edge}: a borda {t} cai fora da grelha do container ({q} quadros)"
            );
        }
        fecha();
    }
}
