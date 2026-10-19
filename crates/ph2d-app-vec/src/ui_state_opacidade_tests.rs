//! ⭐ **F22b — a opacidade do objeto é canal de estado** (dono 19/10: *«passe a animar a
//! opacidade»*; plano 33 §10). O `capture` grava o `VecPath.opacity`, o `install` escreve-o, e quem
//! SAI de um estado acaba em `0` exato.

use super::*;
use ph2d_anim::{Easing, EasingFamily, EasingMode};
use ph2d_ecs::Transform;
use ph2d_ui_state::UiState;

struct Mesa {
    sim: SimWorld,
    map: VecEntityMap,
    scene: VecScene,
    states: StateSets,
    machines: UiMachines,
    host: VecPathId,
}

fn forma(sim: &mut SimWorld, map: &mut VecEntityMap, scene: &mut VecScene, x: f64) -> VecPathId {
    let id = scene.push_path(ph2d_vec_scene::rectangle([x - 1.0, -0.4], [x + 1.0, 0.4]));
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, ph2d_ecs::VecPathRef(id)))
        .id();
    map.insert(id, e.to_bits());
    id
}

fn opacidade(scene: &VecScene, id: VecPathId) -> f32 {
    scene.path(id).unwrap().opacity.get()
}

/// O hospedeiro a 100 % no Default e a 30 % no Hover; `extra` = um segundo objeto gravado SÓ no
/// Default (ele sai no Hover). 1 s linear, em repouso no Default.
fn mesa(extra: bool) -> (Mesa, Option<VecPathId>) {
    let (mut sim, mut map, mut scene) = (SimWorld::default(), VecEntityMap::new(), VecScene::new());
    let host = forma(&mut sim, &mut map, &mut scene, 0.0);
    let filho = extra.then(|| forma(&mut sim, &mut map, &mut scene, 3.0));
    let mut states = StateSets::default();
    for (role, a) in [(StateRole::Default, 1.0), (StateRole::Hover, 0.3)] {
        crate::appearance::set_opacity(&mut scene, &[host], a);
        let mut st = UiState::new(role);
        st.objects = vec![crate::ui_state_edit::capture(&sim, &scene, &map, host)];
        if let (Some(f), StateRole::Default) = (filho, role) {
            st.objects
                .push(crate::ui_state_edit::capture(&sim, &scene, &map, f));
        }
        states.set(host, st);
    }
    crate::appearance::set_opacity(&mut scene, &[host], 1.0);
    states.set_duration(host, 1.0);
    states.set_easing(host, Easing::new(EasingFamily::Linear, EasingMode::InOut));
    let m = Mesa {
        sim,
        map,
        scene,
        states,
        machines: UiMachines::new(),
        host,
    };
    (m, filho)
}

impl Mesa {
    fn show(&mut self, role: StateRole) {
        let world = (&self.sim, &self.scene, &self.map);
        request_seen(&mut self.machines, &self.states, world, self.host, role);
    }

    fn quadro(&mut self, dt: f64) {
        dispatch(
            &mut self.machines,
            &mut self.states,
            (&mut self.sim, &mut self.scene, &self.map),
            dt,
            &mut Cooked::default(),
            &|_| false,
        );
    }
}

/// O `capture` grava a opacidade da forma; o `install` escreve-a de volta.
#[test]
fn a_opacidade_viaja_na_pose() {
    let (mut m, _) = mesa(false);
    crate::appearance::set_opacity(&mut m.scene, &[m.host], 0.45);
    let pose = crate::ui_state_edit::capture(&m.sim, &m.scene, &m.map, m.host);
    assert!(
        (pose.opacity - 0.45).abs() < 1e-6,
        "o Rec não gravou a opacidade: {}",
        pose.opacity
    );
    crate::appearance::set_opacity(&mut m.scene, &[m.host], 1.0);
    crate::ui_state_edit::install(&mut m.sim, &mut m.scene, &m.map, &pose);
    assert!(
        (opacidade(&m.scene, m.host) - 0.45).abs() < 1e-6,
        "o install não a escreveu"
    );
}

/// O Show do Hover ANIMA a opacidade (passa pelo meio) e chega exata.
#[test]
fn o_show_anima_a_opacidade_e_chega_exato() {
    let (mut m, _) = mesa(false);
    m.show(StateRole::Hover);
    m.quadro(0.5);
    let meio = opacidade(&m.scene, m.host);
    assert!(meio > 0.4 && meio < 0.9, "não passou pelo meio: {meio}");
    m.quadro(1.0);
    assert!(
        (opacidade(&m.scene, m.host) - 0.3).abs() < 1e-6,
        "não chegou aos 30 %"
    );
    m.show(StateRole::Default);
    m.quadro(2.0);
    assert!(
        (opacidade(&m.scene, m.host) - 1.0).abs() < 1e-6,
        "não voltou aos 100 %"
    );
}

/// Quem SAI de um estado desvanece e acaba em `0` EXATO — não no resto do penúltimo quadro.
#[test]
fn quem_sai_acaba_em_zero_exato() {
    let (mut m, filho) = mesa(true);
    let filho = filho.unwrap();
    m.show(StateRole::Hover);
    m.quadro(0.5);
    let meio = opacidade(&m.scene, filho);
    assert!(meio > 0.0 && meio < 1.0, "quem sai não desvaneceu: {meio}");
    m.quadro(0.49);
    m.quadro(0.2);
    assert_eq!(
        opacidade(&m.scene, filho),
        0.0,
        "quem saiu ficou com um resto"
    );
    m.show(StateRole::Default);
    m.quadro(2.0);
    assert!(
        (opacidade(&m.scene, filho) - 1.0).abs() < 1e-6,
        "quem volta não reapareceu"
    );
}

/// ⭐ A migração dos ficheiros anteriores ao degrau: as poses gravadas sem a opacidade (todas a
/// `1`) passam a ter a da forma — o Show desses projetos continua a não a mexer.
#[test]
fn a_migracao_poe_em_cada_pose_a_opacidade_da_forma() {
    let (mut m, filho) = mesa(true);
    let filho = filho.unwrap();
    crate::appearance::set_opacity(&mut m.scene, &[m.host, filho], 0.4);
    let n = crate::ui_state_edit::opacity_from_scene(&mut m.states, &m.scene);
    assert_eq!(n, 3, "três poses (host ×2, filho ×1)");
    for st in m.states.get(m.host) {
        for p in &st.objects {
            assert!(
                (p.opacity - 0.4).abs() < 1e-6,
                "pose {} ficou a {}",
                p.id,
                p.opacity
            );
        }
    }
}
