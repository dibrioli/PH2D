//! ⭐⭐ **«Animation: Hero ▾»** com CLIQUES REAIS (doc ObjectsAndComponents 01 §6): o chip pinta,
//! abre, cada opção chega ao documento — e o seletor de clips mostra só a animação em vista,
//! a falar em índices do DOCUMENTO.

use ph2d_editor_core::zones::Rect;
use ph2d_panel_timeline::state::{TimelinePanelState, set_current_timeline};
use ph2d_panel_timeline::{TimelinePanel, ids};
use ph2d_timeline::{
    AnimationIntent, SCENE, TimelineIntent, TimelineState, TimelineViewSnapshot, animation_view,
    apply_intent,
};
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect::new(0.0, 0.0, 1600.0, 900.0);
const HEROI: u64 = 7;
const ESPADA: u64 = 8;

fn playhead() -> ph2d_core::Playhead {
    ph2d_core::Playhead::new(1.0 / 60.0)
}

/// A Cena com «Main», o Herói com «andar» e «pular».
fn documento() -> TimelineState {
    let mut st = TimelineState::new();
    st.doc.add_clip_for("andar".into(), HEROI).unwrap();
    st.doc.add_clip_for("pular".into(), HEROI).unwrap();
    st
}

/// O que a shell publica: o `rebuild` e o cabeçalho (cujos nomes viriam do mundo).
fn publica(st: &mut TimelineState, choices: &[(u64, &str)], offer: Option<(u64, &str)>) -> TimelineViewSnapshot {
    let mut snap = TimelineViewSnapshot::default();
    snap.rebuild(st, &playhead(), false);
    snap.animation.owner = st.view.owner;
    snap.animation.choices = choices.iter().map(|(b, n)| (*b, n.to_string())).collect();
    snap.animation.offer = offer.map(|(b, n)| (b, n.to_string()));
    set_current_timeline(Some(snap.clone()));
    snap
}

/// Pinta, acha `id` e clica no meio dele; devolve os pedidos que chegaram ao documento.
fn clica(host: &mut MockPanelHost, state: &mut TimelinePanelState, id: ph2d_editor_core::NodeId) -> Vec<TimelineIntent> {
    let regs = host.paint::<TimelinePanel>(state, VIEWPORT);
    let r = regs
        .iter()
        .find(|(w, _)| *w == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("{id:?} não foi pintado/registado"));
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        host.apply_panel_event::<TimelinePanel>(state, ev);
    }
    ph2d_panel_timeline::drain_intents()
}

#[test]
fn o_chip_segue_o_heroi_e_o_menu_volta_a_cena() {
    let _ = ph2d_panel_timeline::drain_intents();
    let mut st = documento();
    let mut ph = playhead();
    animation_view::follow(&mut st, &mut ph, Some(HEROI), &[]);
    let snap = publica(&mut st, &[(SCENE, ""), (HEROI, "Herói")], None);
    assert_eq!(snap.clips, ["andar", "pular"], "o seletor mostra só a animação do Herói");

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut panel = TimelinePanelState::default();
    assert!(clica(&mut host, &mut panel, ids::TIMELINE_ANIM_DD).is_empty(), "abrir não é editar");
    let pedidos = clica(&mut host, &mut panel, ids::TIMELINE_ANIM_OPT[0]);
    assert_eq!(pedidos, [TimelineIntent::Animation(AnimationIntent::Show(SCENE))]);

    for p in pedidos {
        apply_intent(&mut st, &mut ph, p);
    }
    let snap = publica(&mut st, &[(SCENE, ""), (HEROI, "Herói")], None);
    assert_eq!(snap.clips, ["Main"], "a Cena mostra a dela");
    // O ▾ fica até a selecção MUDAR.
    animation_view::follow(&mut st, &mut ph, Some(HEROI), &[]);
    assert_eq!(st.view.owner, SCENE);
    animation_view::follow(&mut st, &mut ph, None, &[]);
    animation_view::follow(&mut st, &mut ph, Some(HEROI), &[]);
    assert_eq!(st.view.owner, HEROI, "seleccionar de novo volta a seguir");
}

#[test]
fn escolher_um_clip_da_lista_fala_no_indice_do_documento() {
    let _ = ph2d_panel_timeline::drain_intents();
    let mut st = documento();
    animation_view::follow(&mut st, &mut playhead(), Some(HEROI), &[]);
    publica(&mut st, &[(SCENE, ""), (HEROI, "Herói")], None);

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut panel = TimelinePanelState::default();
    clica(&mut host, &mut panel, ids::TIMELINE_CLIP_DD);
    let pedidos = clica(&mut host, &mut panel, ids::TIMELINE_CLIP_OPT[1]);
    let pular = st.doc.clips().iter().position(|c| c.name == "pular").unwrap();
    assert_eq!(pedidos, [TimelineIntent::SetActiveClip { index: pular }]);
}

#[test]
fn a_oferta_da_animacao_a_espada_e_as_chaves_vao_para_ela() {
    let _ = ph2d_panel_timeline::drain_intents();
    let mut st = documento();
    let mut ph = playhead();
    animation_view::follow(&mut st, &mut ph, Some(ESPADA), &[HEROI]);
    assert_eq!(st.view.owner, HEROI, "a Espada sem Animação mostra a do pai");
    publica(&mut st, &[(SCENE, ""), (HEROI, "Herói")], Some((ESPADA, "Espada")));

    let mut host = MockPanelHost::with_panel::<TimelinePanel>();
    let mut panel = TimelinePanelState::default();
    clica(&mut host, &mut panel, ids::TIMELINE_ANIM_DD);
    let pedidos = clica(&mut host, &mut panel, ids::TIMELINE_ANIM_OPT[2]);
    assert_eq!(pedidos, [TimelineIntent::Animation(AnimationIntent::Add(ESPADA))]);
    for p in pedidos {
        apply_intent(&mut st, &mut ph, p);
    }
    assert!(st.doc.has_animation(ESPADA));
    assert!(st.doc.clip_belongs_to(st.doc.active_index(), ESPADA), "o clip activo é o novo");

    // A escolha 1 do dono: a chave vai para a animação do cabeçalho.
    apply_intent(
        &mut st,
        &mut ph,
        TimelineIntent::AddKey {
            entity: ESPADA,
            prop: ph2d_timeline::PropKind::Rotation,
            t: ph2d_anim::RationalTime::from_seconds(0.0),
            value: ph2d_anim::AnimValue::Float(1.0),
            interp: ph2d_anim::Interp::Linear,
        },
    );
    let alvo = st.doc.binding_for(ESPADA, ph2d_timeline::PropKind::Rotation).unwrap().target;
    let com_a_chave: Vec<usize> = (0..st.doc.clips().len())
        .filter(|&i| st.doc.clips()[i].clip.track(alvo).is_some())
        .collect();
    assert_eq!(com_a_chave, [st.doc.active_index()]);
    assert!(st.doc.clip_belongs_to(com_a_chave[0], ESPADA));
}
