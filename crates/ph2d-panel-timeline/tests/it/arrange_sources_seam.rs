//! ⭐⭐ **O ▾ do Arrange da Cena oferece os clips de TODAS as animações** (onda 4, item 5) — com
//! cliques e roda REAIS: os grupos por dono (cabeçalhos que não se clicam), o clip do Herói que o
//! `+` da lane põe no Arrange, e a lista longa que rola.

use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::InteractiveState;
use ph2d_editor_core::widget::DROPDOWN_SCROLLBAR_ID;
use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::ids::{self, timeline_source_opt};
use ph2d_panel_timeline::state::{TimelinePanelState, set_current_timeline};
use ph2d_panel_timeline::tab::Tab;
use ph2d_panel_timeline::{TimelinePanel, drain_intents};
use ph2d_timeline::{
    MAX_CLIPS, SCENE, StripSource, TimelineIntent, TimelineState, TimelineViewSnapshot,
    animation_view, apply_intent,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);
const HEROI: u64 = 7;

fn playhead() -> ph2d_core::Playhead {
    ph2d_core::Playhead::new(1.0 / 60.0)
}

/// A Cena («Main») com uma lane, e `donos` objectos com `por_dono` clips cada (o 1.º é o Herói).
fn documento(donos: u64, por_dono: usize) -> TimelineState {
    let mut st = TimelineState::new();
    for d in 0..donos {
        for c in 0..por_dono {
            let nome = if d == 0 && c == 0 {
                "andar".to_string()
            } else {
                format!("c{d}.{c}")
            };
            st.doc.add_clip_for(nome, HEROI + d).unwrap();
        }
    }
    apply_intent(&mut st, &mut playhead(), TimelineIntent::AddLane);
    animation_view::follow(&mut st, &mut playhead(), None, &[]);
    assert_eq!(st.view.owner, SCENE, "a vista é a da Cena");
    st
}

/// O que a shell publica: o `rebuild` e os nomes dos donos (que viriam do mundo).
fn publica(st: &mut TimelineState) -> TimelineViewSnapshot {
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(st, &playhead(), false);
    for g in &mut snap.animation.sources {
        if g.owner == HEROI {
            g.name = "Herói".into();
        }
    }
    set_current_timeline(Some(snap.clone()));
    snap
}

fn rect_of(regs: &[(NodeId, Rect)], id: NodeId) -> Option<Rect> {
    regs.iter().find(|(w, _)| *w == id).map(|(_, r)| *r)
}

fn clica(host: &mut MockPanelHost, panel: &mut TimelinePanelState, id: NodeId) {
    let regs = host.paint::<TimelinePanel>(panel, VIEWPORT);
    let r = rect_of(&regs, id).unwrap_or_else(|| panic!("{id:?} não foi pintado/registado"));
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<TimelinePanel>(panel, ev);
    }
}

fn aberto(host: &MockPanelHost) -> bool {
    matches!(
        host.store().get(ids::TIMELINE_CLIP_DD),
        Some(InteractiveState::Dropdown { open: true, .. })
    )
}

/// ⭐⭐ Cena + Herói ⇒ o ▾ lista os dois grupos; clicar no «andar» do Herói e no `+` da lane põe-no
/// no Arrange.
///
/// **Mutações que devem sangrar:** o `fill_sources` só encher a Cena · o `placed_clip` ignorar a
/// fonte de outro dono · os cabeçalhos registarem-se (`disabled` esquecido).
#[test]
fn o_arrange_da_cena_lista_os_dois_grupos_e_poe_o_clip_do_heroi() {
    let _ = drain_intents();
    let mut st = documento(1, 2);
    let snap = publica(&mut st);
    let grupos: Vec<(u64, Vec<&str>)> = snap
        .animation
        .sources
        .iter()
        .map(|g| (g.owner, g.clips.iter().map(|c| c.1.as_str()).collect()))
        .collect();
    assert_eq!(
        grupos,
        [(SCENE, vec!["Main"]), (HEROI, vec!["andar", "c0.1"])],
        "os grupos do Arrange da Cena"
    );

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut panel = TimelinePanelState {
        tab: Tab::Arrange,
        ..TimelinePanelState::default()
    };
    clica(&mut host, &mut panel, ids::TIMELINE_CLIP_DD);
    assert!(aberto(&host), "o ▾ não abriu");
    let regs = host.paint::<TimelinePanel>(&mut panel, VIEWPORT);
    // Linhas: 0 «Cena» · 1 Main · 2 «Herói» · 3 andar · 4 c0.1 — os cabeçalhos não se clicam.
    for (i, clicavel) in [(0, false), (1, true), (2, false), (3, true), (4, true)] {
        assert_eq!(
            rect_of(&regs, timeline_source_opt(i)).is_some(),
            clicavel,
            "linha {i}: clicável devia ser {clicavel}"
        );
    }

    let andar = st.doc.clip_named(HEROI, "andar").unwrap();
    clica(&mut host, &mut panel, timeline_source_opt(3));
    assert!(
        drain_intents().is_empty(),
        "um clip de OUTRO dono não é o clip activo da Cena — o `follow` desfazê-lo-ia"
    );
    assert!(!aberto(&host), "escolher fecha a lista");
    assert_eq!(panel.source_clip, Some((andar, "andar".into())));

    clica(&mut host, &mut panel, ids::TIMELINE_LANE_ADD_STRIP[0]);
    let pedidos = drain_intents();
    assert!(
        matches!(
            pedidos.as_slice(),
            [TimelineIntent::AddStrip { lane: 0, source: StripSource::Clip(i), .. }]
                if usize::from(*i) == andar
        ),
        "o + da lane não pôs o «andar» do Herói: {pedidos:?}"
    );
    let mut ph = playhead();
    for p in pedidos {
        apply_intent(&mut st, &mut ph, p);
    }
    assert_eq!(
        st.doc.stack()[0].strips[0].source,
        StripSource::Clip(u16::try_from(andar).unwrap()),
        "o Arrange da Cena não ficou com o clip do Herói"
    );
}

/// ⭐⭐ Com muitos clips a lista ROLA: a roda sobre ela desce até à última linha, que se clica.
#[test]
fn com_muitos_clips_a_roda_rola_ate_ao_ultimo() {
    let _ = drain_intents();
    let mut st = documento(6, MAX_CLIPS);
    publica(&mut st);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut panel = TimelinePanelState {
        tab: Tab::Arrange,
        ..TimelinePanelState::default()
    };
    clica(&mut host, &mut panel, ids::TIMELINE_CLIP_DD);
    let regs = host.paint::<TimelinePanel>(&mut panel, VIEWPORT);
    let n = 2 + 6 * (1 + MAX_CLIPS); // a Cena (cabeçalho + Main) e seis grupos
    let ultima = timeline_source_opt(n - 1);
    assert!(
        rect_of(&regs, ultima).is_none(),
        "pré-condição: a última linha começa fora"
    );
    assert!(
        rect_of(&regs, DROPDOWN_SCROLLBAR_ID).is_some(),
        "a lista longa não tem barra — o polegar não se agarraria"
    );
    let (dono, pop) = host
        .store()
        .dropdown_popover()
        .expect("o rect da lista não foi publicado — a roda não a acha");
    assert_eq!(dono, ids::TIMELINE_CLIP_DD);
    host.wheel_at(pop.x + pop.w * 0.5, pop.y + pop.h * 0.5, -100_000.0);
    assert!(
        host.store().panel_scroll(ids::TIMELINE_CLIP_DD) > 0.0,
        "a roda sobre a lista não a rolou"
    );
    clica(&mut host, &mut panel, ultima);
    let (index, nome) = panel
        .source_clip
        .clone()
        .expect("a última linha não escolheu nada");
    assert_eq!(st.doc.clips()[index].name, nome);
    assert_eq!(
        nome,
        format!("c5.{}", MAX_CLIPS - 1),
        "a última linha é o último clip"
    );
}

/// CONTROLO: na Keys, e na vista do Herói, o ▾ é o de sempre (a animação em vista, sem grupos).
#[test]
fn fora_do_arrange_da_cena_o_seletor_e_o_de_sempre() {
    let _ = drain_intents();
    let mut st = documento(1, 2);
    publica(&mut st);
    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut panel = TimelinePanelState {
        tab: Tab::Keys,
        ..TimelinePanelState::default()
    };
    clica(&mut host, &mut panel, ids::TIMELINE_CLIP_DD);
    let regs = host.paint::<TimelinePanel>(&mut panel, VIEWPORT);
    assert!(rect_of(&regs, ids::TIMELINE_CLIP_OPT[0]).is_some());
    assert!(rect_of(&regs, timeline_source_opt(1)).is_none());

    animation_view::follow(&mut st, &mut playhead(), Some(HEROI), &[]);
    let snap = publica(&mut st);
    assert!(
        snap.animation.sources.is_empty(),
        "a vista do Herói não agrupa"
    );
}

/// A faixa de ids é distinta em toda a extensão do recurso que diz ser (e dos ids fixos do painel).
#[test]
fn a_faixa_das_fontes_e_unica() {
    let mut vistos = std::collections::BTreeSet::new();
    for i in 0..ids::TIMELINE_SOURCE_ROWS {
        assert!(
            vistos.insert(timeline_source_opt(i).0),
            "colisão na linha {i}"
        );
    }
    let fixos = ids::TIMELINE_CLIP_OPT
        .iter()
        .chain(ids::TIMELINE_CONT_OPT.iter())
        .chain(ids::TIMELINE_ANIM_OPT.iter())
        .chain([ids::TIMELINE_CLIP_DD, DROPDOWN_SCROLLBAR_ID].iter());
    for id in fixos {
        assert!(
            !vistos.contains(&id.0),
            "{id:?} colide com a faixa das fontes"
        );
    }
}
