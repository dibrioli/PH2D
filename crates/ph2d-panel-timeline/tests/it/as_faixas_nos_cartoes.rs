//! ⭐ **As faixas e a lista de containers nos cartões** (doc 16, onda 3, maquete B3/A3): a coluna de
//! nomes muda de cara e de altura (`TIMELINE_FAIXA_H_PX`), e cada controlo continua a pedir o mesmo,
//! clicado onde se pintou, com um alvo pelo menos do tamanho de antes.

use ph2d_a11y::NodeId;
use ph2d_editor_core::panel::cards::ferramenta;
use ph2d_editor_core::zones::Rect;
use ph2d_host::PointerKind;
use ph2d_panel_timeline::state::{
    RenameKind, TimelinePanelState, drain_intents, set_current_timeline,
};
use ph2d_panel_timeline::tab::Tab;
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_timeline::{StackHost, StripSource, TimelineDoc, TimelineIntent, TimelineViewSnapshot};
use ph2d_ui_testkit::mesa::Mesa;

const JANELA: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 1930.0,
    h: 1040.0,
};

/// Dois containers («Walk» com 2 s dentro e posto na cena, «Jump» vazio) e duas faixas de cena.
fn documento() -> TimelineViewSnapshot {
    let mut doc = TimelineDoc::new();
    let walk = doc.add_container("Walk".into());
    let dentro = doc
        .add_lane_in(StackHost::Container(walk), "in".into())
        .expect("a faixa de dentro");
    doc.add_strip_to(
        StackHost::Container(walk),
        dentro,
        StripSource::Clip(0),
        0.0,
        2.0,
    )
    .expect("a peça de dentro");
    doc.add_container("Jump".into());
    let base = doc.add_lane("Base".into()).expect("a faixa Base");
    doc.add_lane("Upper body".into())
        .expect("a faixa Upper body");
    doc.add_strip_to(
        StackHost::Document,
        base,
        StripSource::Container(0),
        0.0,
        2.0,
    )
    .expect("Walk posto na cena");
    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph2d_core::Playhead::default(), false);
    snap
}

fn mesa(cartoes: bool, tab: Tab) -> Mesa<TimelinePanel> {
    ferramenta::force_cards_on_this_thread(Some(cartoes));
    set_current_timeline(Some(documento()));
    let _ = drain_intents();
    let mut mesa = Mesa::nova(TimelinePanelState {
        tab,
        ..TimelinePanelState::default()
    });
    mesa.assenta_em(JANELA);
    let _ = drain_intents();
    let _ = mesa.host.drained_actions();
    mesa
}

fn rect(mesa: &Mesa<TimelinePanel>, id: NodeId) -> Rect {
    mesa.rect(id)
        .unwrap_or_else(|| panic!("{id:?} não está pintado"))
}

/// Os alvos da coluna de nomes das faixas (aba Arrange) e da lista (aba Containers), com o
/// `w × h` MEDIDO nos cartões antes da onda 3 (1930 px; a conta era a do clássico).
fn alvos() -> Vec<(Tab, NodeId, f32, f32)> {
    let mut v = Vec::new();
    for i in 0..2 {
        v.push((Tab::Arrange, ids::TIMELINE_LANE_WEIGHT[i], 38.0, 16.0));
        v.push((Tab::Arrange, ids::TIMELINE_LANE_MUTE[i], 16.0, 16.0));
        v.push((Tab::Arrange, ids::TIMELINE_LANE_ADD_STRIP[i], 16.0, 16.0));
        v.push((Tab::Arrange, ids::TIMELINE_LANE_ROW[i], 92.0, 22.0));
    }
    v.push((Tab::Arrange, ids::TIMELINE_ADD_LANE, 174.0, 22.0));
    for i in 0..2 {
        v.push((Tab::Containers, ids::TIMELINE_CONT_RENAME[i], 16.0, 16.0));
        v.push((Tab::Containers, ids::TIMELINE_CONT_DELETE[i], 16.0, 16.0));
        // A LARGURA da barra é tempo × zoom (2 s na vista de agora), não tamanho de controlo: a coluna
        // que cresce 4 px para o nome caber come-a da vista. Só a ALTURA é do controlo.
        v.push((Tab::Containers, ids::TIMELINE_CONT_ROW[i], 1.0, 16.0));
    }
    v.push((Tab::Containers, ids::TIMELINE_ADD_CONTAINER, 174.0, 22.0));
    v
}

/// ⭐ Os alvos da coluna de nomes não encolheram com a cara nova: cada um mede pelo menos o que media
/// antes da onda 3 nos cartões (peso 38 × 16, mudo e `+` 16 × 16, a etiqueta da faixa 92 × 22, lápis e
/// lixo 16 × 16, a barra do container 89,1 × 16, os dois «+» da régua 174 × 22).
#[test]
fn os_alvos_das_faixas_nao_encolheram() {
    for (tab, id, w, h) in alvos() {
        let m = mesa(true, tab);
        let r = rect(&m, id);
        assert!(
            r.w >= w && r.h >= h,
            "{tab:?} {id:?}: alvo {}×{} < {w}×{h}",
            r.w,
            r.h
        );
    }
    ferramenta::force_cards_on_this_thread(None);
}

/// ⭐ A faixa e a linha da lista medem a altura da maquete nos cartões (30), e as do clássico a de
/// sempre (22) com os alvos de antes, ao pixel: `PH2D_CARDS=0` não mudou.
#[test]
fn a_faixa_mede_a_maquete_nos_cartoes_e_o_classico_e_o_de_antes() {
    for (tab, linha) in [
        (
            Tab::Arrange,
            [ids::TIMELINE_LANE_ROW[0], ids::TIMELINE_LANE_ROW[1]],
        ),
        (
            Tab::Containers,
            [ids::TIMELINE_CONT_RENAME[0], ids::TIMELINE_CONT_RENAME[1]],
        ),
    ] {
        let m = mesa(true, tab);
        let (a, b) = (rect(&m, linha[0]), rect(&m, linha[1]));
        assert!(
            (b.y - a.y - ph2d_tokens::TIMELINE_FAIXA_H_PX).abs() < 0.01,
            "{tab:?}: as linhas distam {} nos cartões",
            b.y - a.y
        );
    }
    let m = mesa(true, Tab::Arrange);
    assert!(
        (rect(&m, ids::TIMELINE_LANE_ROW[0]).h - ph2d_tokens::TIMELINE_FAIXA_H_PX).abs() < 0.01
    );

    for (tab, id, w, h) in alvos() {
        if matches!(id, i if i == ids::TIMELINE_ADD_LANE || i == ids::TIMELINE_ADD_CONTAINER)
            || ids::TIMELINE_CONT_ROW.contains(&id)
            || ids::TIMELINE_LANE_ROW.contains(&id)
        {
            continue; // a largura destes depende da coluna/zoom, não da faixa
        }
        let m = mesa(false, tab);
        let r = rect(&m, id);
        assert_eq!((r.w, r.h), (w, h), "{tab:?} {id:?} mudou no clássico");
    }
    let m = mesa(false, Tab::Arrange);
    let (a, b) = (
        rect(&m, ids::TIMELINE_LANE_ROW[0]),
        rect(&m, ids::TIMELINE_LANE_ROW[1]),
    );
    assert_eq!(
        (a.h, b.y - a.y),
        (ph2d_tokens::ROW_H_PX, ph2d_tokens::ROW_H_PX)
    );
    ferramenta::force_cards_on_this_thread(None);
}

/// ⭐ Nos cartões, cada controlo da faixa pede o que pedia, clicado onde se pintou: o olho silencia a
/// faixa certa e o `+` põe a fonte nela.
#[test]
fn o_olho_e_o_mais_da_faixa_pedem_o_de_sempre() {
    for i in 0..2 {
        let mut m = mesa(true, Tab::Arrange);
        let _ = m.clica(ids::TIMELINE_LANE_MUTE[i]);
        assert_eq!(
            drain_intents(),
            vec![TimelineIntent::SetLaneMuted {
                lane: i,
                muted: true
            }],
            "o olho da faixa {i}"
        );
        let _ = m.clica(ids::TIMELINE_LANE_ADD_STRIP[i]);
        let pedidos = drain_intents();
        assert!(
            matches!(pedidos.as_slice(), [TimelineIntent::AddStrip { lane, .. }] if *lane == i),
            "o + da faixa {i}: {pedidos:?}"
        );
    }
    ferramenta::force_cards_on_this_thread(None);
}

/// ⭐ O peso na pílula continua um campo: arrastá-lo pelo alvo pintado pede o peso novo da faixa.
#[test]
fn arrastar_o_peso_na_pilula_pede_o_peso() {
    let mut m = mesa(true, Tab::Arrange);
    let r = rect(&m, ids::TIMELINE_LANE_WEIGHT[1]);
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert_eq!(m.host.hit_at(x, y), Some(ids::TIMELINE_LANE_WEIGHT[1]));
    let mut ev = m.ponteiro(PointerKind::Down, x, y);
    for dx in [-20.0, -40.0, -60.0] {
        ev.extend(m.ponteiro(PointerKind::Move, x + dx, y));
    }
    ev.extend(m.ponteiro(PointerKind::Up, x - 60.0, y));
    let _ = m.aplica(ev);
    let pedidos = drain_intents();
    assert!(
        pedidos.iter().any(|p| matches!(
            p,
            TimelineIntent::SetLaneWeight { lane: 1, weight } if *weight < 1.0
        )),
        "o arrasto do peso não pediu nada: {pedidos:?}"
    );
    ferramenta::force_cards_on_this_thread(None);
}

/// ⭐ Na lista em cartões, o lápis abre o nome do container certo, o lixo apaga-o, e o duplo-clique
/// REAL na barra entra nele.
#[test]
fn os_verbos_da_lista_nos_cartoes_pedem_o_de_sempre() {
    for i in 0..2 {
        let mut m = mesa(true, Tab::Containers);
        let _ = m.clica(ids::TIMELINE_CONT_RENAME[i]);
        let aberto = m.st.clip_rename.map(|c| (c.kind, c.index));
        assert_eq!(aberto, Some((RenameKind::Container, i)), "o lápis {i}");
        let mut m = mesa(true, Tab::Containers);
        let _ = m.clica(ids::TIMELINE_CONT_DELETE[i]);
        assert_eq!(
            drain_intents(),
            vec![TimelineIntent::RemoveContainer { index: i }],
            "o lixo {i}"
        );

        let mut m = mesa(true, Tab::Containers);
        let r = rect(&m, ids::TIMELINE_CONT_ROW[i]);
        let (cx, cy) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
        assert_eq!(m.host.hit_at(cx, cy), Some(ids::TIMELINE_CONT_ROW[i]));
        let mut t: u128 = 1_000_000_000;
        for _ in 0..2 {
            for kind in [PointerKind::Down, PointerKind::Up] {
                let _ = m.host.dispatch_pointer_event(ph2d_host::PointerEvent {
                    x: cx,
                    y: cy,
                    pressure: 1.0,
                    kind,
                    source: ph2d_host::PointerSource::Mouse,
                    button: ph2d_host::PointerButton::Primary,
                    timestamp_ns: t,
                });
                t += 50_000_000;
            }
        }
        m.assenta_em(JANELA);
        assert_eq!(
            ph2d_panel_timeline::state::edit_host(),
            StackHost::Container(i),
            "o duplo-clique na barra {i} entra"
        );
        ph2d_panel_timeline::state::reset_trail();
    }
    ferramenta::force_cards_on_this_thread(None);
}

/// «Run & jump» posto DUAS vezes na cena (o uso diz «used 2×») e as faixas «Base» e «Upper body».
fn documento_de_nomes() -> TimelineViewSnapshot {
    let mut doc = TimelineDoc::new();
    doc.add_container("Run & jump".into());
    let base = doc.add_lane("Base".into()).expect("a faixa Base");
    let cima = doc
        .add_lane("Upper body".into())
        .expect("a faixa Upper body");
    for (faixa, t) in [(base, 0.0), (cima, 1.0)] {
        doc.add_strip_to(
            StackHost::Document,
            faixa,
            StripSource::Container(0),
            t,
            1.0,
        )
        .expect("Run & jump posto na cena");
    }
    let mut st = ph2d_timeline::TimelineState::new();
    st.doc = doc;
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(&mut st, &ph2d_core::Playhead::default(), false);
    snap
}

/// ⭐⭐ Os nomes da coluna saem INTEIROS no painel docado de 1290 px (a janela de 1930 do dono): a
/// coluna tem o piso do nome mais largo com o ícone e os controlos, como a da aba Keys. *Mutação:
/// `geom::min_label_w_medido` devolver só `min_label_w` ⇒ «Upper bo…» ⇒ vermelho.*
#[test]
fn os_nomes_das_faixas_e_da_lista_saem_inteiros_a_1290() {
    let janela = Rect {
        w: 1290.0,
        ..JANELA
    };
    for (tab, textos) in [
        (Tab::Arrange, &["Upper body", "Base"][..]),
        (Tab::Containers, &["Run & jump", "used 2×"][..]),
    ] {
        ferramenta::force_cards_on_this_thread(Some(true));
        set_current_timeline(Some(documento_de_nomes()));
        let _ = drain_intents();
        let mut m0 = Mesa::<TimelinePanel>::nova(TimelinePanelState {
            tab,
            ..TimelinePanelState::default()
        });
        m0.assenta_em(janela);
        let (h, st) = (&mut m0.host, &mut m0.st);
        let medidos = h.medindo_a_pintura::<TimelinePanel>(st, janela);
        for t in textos {
            let pintados: Vec<_> = medidos.iter().filter(|x| x.texto == *t).collect();
            assert!(!pintados.is_empty(), "{tab:?}: {t:?} não pintado");
            for x in pintados {
                assert!(x.coube(), "{tab:?}: {t:?} cortado em {:?}", x.pintado);
            }
        }
    }
    ferramenta::force_cards_on_this_thread(None);
}

/// ⭐ **O olho da faixa, de ponta a ponta** (dono 10/11: *«veja se o olhinho está fazendo o trabalho
/// dele»*): o clique pintado → `SetLaneMuted` → `apply_intent` grava `lane.muted` → a pose avaliada
/// no meio da peça é a de SEM a faixa; outro clique devolve a sacudida; Ctrl+Z desfaz. Nos dois modos.
#[test]
fn o_olho_da_faixa_tira_e_devolve_a_faixa_da_pose() {
    use ph2d_anim::{AnimValue, Interp, RationalTime};
    use ph2d_ecs::{Entity, Transform, World};
    use ph2d_timeline::{PropKind, TimelineState, apply_from_doc, apply_intent};
    let mut world = World::new();
    let e = world.spawn(Transform::default()).id().to_bits();
    let mut st = TimelineState::new();
    let key = |doc: &mut TimelineDoc, t: f64, v: f32| {
        let at = RationalTime::from_seconds(t);
        doc.insert_key(
            e,
            PropKind::TranslationX,
            at,
            AnimValue::Float(v),
            Interp::Linear,
        );
    };
    key(&mut st.doc, 0.0, 1.0);
    key(&mut st.doc, 2.0, 1.0);
    let shake = st.doc.add_clip("shake".into());
    st.doc.set_active(shake);
    for (t, v) in [(0.0, 0.0), (0.25, 0.6), (0.5, 0.0)] {
        key(&mut st.doc, t, v);
    }
    st.doc.set_active(0);
    let src = |c: usize| StripSource::Clip(u16::try_from(c).expect("cabe"));
    for (nome, c, t0, t1) in [("Base", 0, 0.0, 2.0), ("FX", shake, 1.0, 1.5)] {
        let l = st.doc.add_lane(nome.into()).expect("faixa");
        st.doc
            .add_strip_to(StackHost::Document, l, src(c), t0, t1)
            .expect("peça");
    }
    let mut ph = ph2d_core::Playhead::default();
    let mut x_em = |doc: &mut TimelineDoc| {
        apply_from_doc(&mut world, doc, 1.25);
        world
            .get::<Transform>(Entity::from_bits(e))
            .expect("xf")
            .translation
            .x
    };
    let mut sem_fx = st.doc.clone();
    sem_fx.remove_lane_in(StackHost::Document, 1);
    let (x_sem, x_com) = (x_em(&mut sem_fx), x_em(&mut st.doc));
    assert!(
        (x_com - x_sem).abs() > 0.3,
        "a sacudida mexe: {x_com} × {x_sem}"
    );

    for cartoes in [true, false] {
        ferramenta::force_cards_on_this_thread(Some(cartoes));
        let mut clica_o_olho = |st: &mut TimelineState| {
            let mut snap = TimelineViewSnapshot::default();
            snap.rebuild(st, &ph, false);
            set_current_timeline(Some(snap));
            let mut m = Mesa::<TimelinePanel>::nova(TimelinePanelState {
                tab: Tab::Arrange,
                ..TimelinePanelState::default()
            });
            m.assenta_em(JANELA);
            let _ = drain_intents();
            // A dica diz o que o clique FARÁ: fechado ⇒ «Unmute», aberto ⇒ «Mute».
            let chave = if st.doc.stack()[1].muted {
                "panel.timeline.lane_unmute"
            } else {
                "panel.timeline.lane_mute"
            };
            let dica = m.host.store().tooltip_for(ids::TIMELINE_LANE_MUTE[1]);
            assert_eq!(dica, Some(ph2d_i18n::tr(chave)), "a dica do olho");
            let _ = m.clica(ids::TIMELINE_LANE_MUTE[1]);
            for i in drain_intents() {
                apply_intent(st, &mut ph, i);
            }
        };
        clica_o_olho(&mut st);
        assert!(st.doc.stack()[1].muted, "o olho fechou a FX ({cartoes})");
        assert_eq!(
            x_em(&mut st.doc),
            x_sem,
            "muda = a pose sem a faixa ({cartoes})"
        );
        clica_o_olho(&mut st);
        assert!(!st.doc.stack()[1].muted, "o olho reabriu ({cartoes})");
        assert_eq!(
            x_em(&mut st.doc),
            x_com,
            "aberta, a sacudida volta ({cartoes})"
        );
        clica_o_olho(&mut st);
        apply_intent(&mut st, &mut ph, TimelineIntent::Undo);
        assert!(!st.doc.stack()[1].muted, "Ctrl+Z desfaz o mudo ({cartoes})");
    }
    ferramenta::force_cards_on_this_thread(None);
}
