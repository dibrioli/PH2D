//! (GEF02 09/11) **Auto Key dos pontos do Outline pelo histórico real**: o passe do quadro (`propagar_topologia`
//! + `apply_samples`), o passo global ligado à chave (`push_linked` + `liga_gesto_da_cena`) e o Ctrl+Z de
//! `apply_undo` (a fila global e, se a transação casa, a da timeline): cada gesto é UM Ctrl+Z que devolve os
//! pontos E as chaves.

use super::*;
use crate::undo::{ProjectState, ProjectUndo, SelectionMark};
use ph2d_ecs::canal_fisica as f;
use ph2d_physics_ecs::{Collider, ColliderOutline, ColliderShape};

struct C {
    sim: ph2d_ecs::SimWorld,
    reg: ph2d_ecs::scene::ComponentRegistry,
    e: u64,
    st: TimelineState,
    ph: Playhead,
    ak: AutokeyState,
    fila: ProjectUndo,
    base: ProjectState,
}

fn foto(sim: &mut ph2d_ecs::SimWorld, reg: &ph2d_ecs::scene::ComponentRegistry) -> ProjectState {
    ProjectState::capture(
        &Default::default(),
        sim,
        &Default::default(),
        &ph2d_flip::FlipDoc::new(),
        &Default::default(),
        &Default::default(),
        &Default::default(),
        &[],
        reg,
        &mut ph2d_ecs::scene::incremental::CaptureCache::new(),
        None,
    )
}

/// O quadrado à mão (`custom`), Auto Key armado, parado em 0 s; um quadro já assentou a base.
fn cena() -> C {
    let mut reg = ph2d_ecs::scene::ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    let mut sim = ph2d_ecs::SimWorld::new();
    let o = ColliderOutline {
        rings: vec![vec![[0, 0], [64, 0], [64, 64], [0, 64]]].into(),
        picture_px: [4, 4],
        custom: true,
        ..ColliderOutline::default()
    };
    let shape = ColliderShape::Outline {
        half_x: 2.0,
        half_y: 2.0,
    };
    let c = Collider {
        shape,
        ..Collider::default()
    };
    let e = sim
        .world_mut()
        .spawn((ph2d_ecs::Transform::default(), c, o))
        .id()
        .to_bits();
    let mut ph = Playhead::new(1.0 / 60.0);
    ph.pause();
    let (st, ak, fila) = (
        TimelineState::new(),
        AutokeyState::default(),
        ProjectUndo::default(),
    );
    let base = foto(&mut sim, &reg);
    let mut c = C {
        sim,
        reg,
        e,
        st,
        ph,
        ak,
        fila,
        base,
    };
    quadro(&mut c);
    c
}

/// Um quadro: a família publica, o Auto Key decide, e o gate pós-quadro regista o passo ligado.
fn quadro(c: &mut C) {
    ph2d_app_physics::canais_corpo::publicar(&mut c.sim, &[c.e]);
    let pose = sample_pose(
        super::super::timeline_bridge::Mundo::from(c.sim.world()),
        c.e,
    );
    c.ak.campos = [(c.e, campos_lidos(&c.sim, c.e))].into();
    propagar_topologia(&mut c.st, &c.ak);
    let mut toasts = ph2d_editor_core::ToastQueue::new();
    apply_samples(
        &mut c.st,
        &c.ph,
        &[(c.e, pose)],
        false,
        true,
        false,
        &mut c.ak,
        &mut toasts,
    );
    let agora = foto(&mut c.sim, &c.reg);
    if agora == c.base {
        c.st.history.esquece_gesto_da_cena();
        return;
    }
    let linha = c.st.history.liga_gesto_da_cena();
    let pre = std::mem::replace(&mut c.base, agora);
    c.fila
        .push_linked(pre, SelectionMark::default(), None, None, linha);
}

/// A mão escreve o Outline (os pontos, ou o Fit) e o quadro corre.
fn mexe(c: &mut C, novo: impl FnOnce(&mut ColliderOutline)) {
    let e = ph2d_ecs::Entity::from_bits(c.e);
    novo(&mut c.sim.world_mut().get_mut::<ColliderOutline>(e).expect("o"));
    quadro(c);
}

fn pontos(c: &mut C, r: &[[i32; 2]]) {
    mexe(c, |o| o.rings = vec![r.to_vec()].into());
}

/// O Ctrl+Z de `apply_undo`: a fila global e, com a transação dela no topo da timeline, a chave junto.
fn ctrl_z(c: &mut C) -> ProjectState {
    let linha = c.fila.linha_do_topo(false);
    let agora = foto(&mut c.sim, &c.reg);
    let (volta, _) = c
        .fila
        .undo(agora, SelectionMark::default())
        .expect("há um passo");
    if linha.is_some() && c.st.history.transacao_do_topo(false) == linha {
        assert!(c.st.undo());
    }
    volta
}

fn chaves_do_ponto(c: &C, j: u16) -> usize {
    let kk = ph2d_ecs::FieldKey::new(ph2d_ecs::canal::familia::FISICA, f::CONTORNO_PONTO_X, 0, j);
    c.st.doc
        .binding_for(c.e, PropKind::Field(kk))
        .and_then(|b| c.st.doc.active_clip().track(b.target))
        .map_or(0, |t| t.keys().len())
}

/// O gesto, depois UM Ctrl+Z: o mundo e a timeline voltam ao de antes do gesto.
fn um_passo(c: &mut C, gesto: impl FnOnce(&mut C)) {
    let (mundo, doc, n) = (c.base.clone(), c.st.doc.to_bytes(), c.fila.depth());
    gesto(c);
    assert_eq!(c.fila.depth(), n + 1, "UM passo global");
    assert_eq!(ctrl_z(c), mundo, "o Ctrl+Z devolve os pontos");
    assert_eq!(c.st.doc.to_bytes(), doc, "…e as chaves, no MESMO Ctrl+Z");
}

/// ⭐ Arrastar um ponto com Auto Key = uma chave (todos os pontos) e UM Ctrl+Z tira as duas coisas.
#[test]
fn arrastar_um_ponto_com_auto_key_e_um_ctrl_z() {
    let mut c = cena();
    um_passo(&mut c, |c| {
        pontos(c, &[[0, 0], [80, 0], [64, 64], [0, 64]]);
        assert_eq!(chaves_do_ponto(c, 1), 1, "a chave nasceu");
    });
}

/// ⭐ Duplo clique numa forma com DUAS chaves: as duas ganham o ponto, e UM Ctrl+Z tira-o das duas e do
/// mundo. O Fit também é UM passo (a trilha sai e volta). *Mutação: sem propagar ⇒ vermelho.*
#[test]
fn acrescentar_um_ponto_com_duas_chaves_e_o_fit_sao_um_ctrl_z_cada() {
    let mut c = cena();
    pontos(&mut c, &[[0, 0], [80, 0], [64, 64], [0, 64]]);
    c.ph.seek(1.0);
    quadro(&mut c);
    pontos(&mut c, &[[0, 0], [96, 0], [64, 64], [0, 64]]);
    assert_eq!(chaves_do_ponto(&c, 1), 2);
    um_passo(&mut c, |c| {
        pontos(c, &[[0, 0], [96, 0], [64, 64], [32, 64], [0, 64]]);
        assert_eq!(chaves_do_ponto(c, 4), 2, "as DUAS chaves ganharam o ponto");
    });
    um_passo(&mut c, |c| {
        mexe(c, |o| o.custom = false);
        assert_eq!(chaves_do_ponto(c, 0), 0, "o Fit apagou a trilha");
    });
}
