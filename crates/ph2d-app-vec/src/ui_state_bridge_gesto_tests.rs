//! ⭐ **F22 — o gesto do artista vence o Show em voo** (plano 33 §6, `vetor_nova_ui/33_*`).

use super::*;
use ph2d_anim::{Easing, EasingFamily, EasingMode};
use ph2d_ecs::Transform;
use ph2d_ui_state::UiState;
use ph2d_vec_scene::{Paint, Rgba8};

const AZUL: [u8; 3] = [40, 120, 240];
const LARANJA: [u8; 3] = [240, 140, 40];
const VERDE: [u8; 3] = [40, 200, 90];

fn tinta(c: [u8; 3]) -> Option<Paint> {
    Some(Paint::Solid(Rgba8::new(c[0], c[1], c[2], 255)))
}

struct Mesa {
    sim: SimWorld,
    map: VecEntityMap,
    scene: VecScene,
    states: StateSets,
    machines: UiMachines,
    host: VecPathId,
    e: ph2d_ecs::Entity,
    /// O que a shell passa como «motor» (`PreviewDrive::drives`).
    conduzido: bool,
}

/// Um retângulo com Default (x 0, azul) e Hover (x 10, laranja), 2 s lineares, em repouso no
/// Default. ⚠️ A cor e não a Opacity: a opacidade do objeto não é canal de estado (o `install` não
/// a escreve), e uma edição dela nunca foi apagada.
fn mesa() -> Mesa {
    let mut sim = SimWorld::default();
    let mut map = VecEntityMap::new();
    let mut scene = VecScene::new();
    let host = scene.push_path(ph2d_vec_scene::rectangle([-1.0, -0.4], [1.0, 0.4]));
    let e = sim
        .world_mut()
        .spawn((Transform::IDENTITY, ph2d_ecs::VecPathRef(host)))
        .id();
    map.insert(host, e.to_bits());
    let mut states = StateSets::default();
    for (role, x, c) in [
        (StateRole::Default, 0.0, AZUL),
        (StateRole::Hover, 10.0, LARANJA),
    ] {
        sim.world_mut()
            .get_mut::<Transform>(e)
            .unwrap()
            .translation
            .x = x;
        scene.path_mut(host).unwrap().fill = tinta(c);
        let mut st = UiState::new(role);
        st.objects = vec![crate::ui_state_edit::capture(&sim, &scene, &map, host)];
        states.set(host, st);
    }
    sim.world_mut()
        .get_mut::<Transform>(e)
        .unwrap()
        .translation
        .x = 0.0;
    scene.path_mut(host).unwrap().fill = tinta(AZUL);
    states.set_duration(host, 2.0);
    states.set_easing(host, Easing::new(EasingFamily::Linear, EasingMode::InOut));
    Mesa {
        sim,
        map,
        scene,
        states,
        machines: UiMachines::new(),
        host,
        e,
        conduzido: false,
    }
}

impl Mesa {
    fn show_hover(&mut self) {
        let world = (&self.sim, &self.scene, &self.map);
        request_seen(
            &mut self.machines,
            &self.states,
            world,
            self.host,
            StateRole::Hover,
        );
    }

    /// Um quadro, pela porta do frame; devolve se a máquina pousou por um gesto.
    fn quadro(&mut self, dt: f64) -> bool {
        let conduzido = self.conduzido;
        let mut out = Cooked::default();
        dispatch(
            &mut self.machines,
            &mut self.states,
            (&mut self.sim, &mut self.scene, &self.map),
            dt,
            &mut out,
            &|_| conduzido,
        );
        out.pousou
    }

    fn x(&self) -> f32 {
        self.sim
            .world()
            .get::<Transform>(self.e)
            .unwrap()
            .translation
            .x
    }

    fn tinta(&self) -> Option<Paint> {
        self.scene.path(self.host).unwrap().fill.clone()
    }

    /// O artista escolhe uma cor no Fill (o painel escreve o `VecPath.fill`).
    fn pinta(&mut self, c: [u8; 3]) {
        self.scene.path_mut(self.host).unwrap().fill = tinta(c);
    }

    fn em_voo(&self) -> bool {
        self.machines[&self.host].is_animating()
    }
}

/// O artista escolhe verde no Fill a meio do Show: o verde fica, e a forma pára onde estava.
#[test]
fn uma_edicao_a_meio_do_show_fica() {
    let mut m = mesa();
    m.show_hover();
    m.quadro(0.5);
    let meio = m.x();
    assert!(
        meio > 1.0 && meio < 9.0,
        "a fixture não está a meio: {meio}"
    );
    m.pinta(VERDE);
    assert!(m.quadro(0.1), "o quadro do pouso não avisou o undo");
    assert_eq!(m.tinta(), tinta(VERDE), "o Show apagou a cor escolhida");
    assert!(!m.em_voo(), "o Show continuou depois do gesto");
    assert!(
        (m.x() - meio).abs() < 1e-6,
        "a forma saltou: {} contra {meio}",
        m.x()
    );
    assert_eq!(
        live_role(&m.machines, Some(m.host)),
        None,
        "o readout diz um papel que a cena não mostra"
    );
    for _ in 0..30 {
        assert!(!m.quadro(0.1), "o aviso repetiu-se sem gesto");
    }
    assert!(
        m.tinta() == tinta(VERDE) && (m.x() - meio).abs() < 1e-6,
        "a máquina voltou a escrever"
    );
}

/// O arrasto no canvas a meio do Show: a forma fica onde o artista a pôs.
#[test]
fn um_arrasto_a_meio_do_show_fica() {
    let mut m = mesa();
    m.show_hover();
    m.quadro(0.5);
    m.sim
        .world_mut()
        .get_mut::<Transform>(m.e)
        .unwrap()
        .translation
        .x = -7.0;
    m.quadro(0.1);
    assert!(
        (m.x() + 7.0).abs() < 1e-6,
        "o Show levou a forma: x {}",
        m.x()
    );
    assert!(!m.em_voo(), "o Show continuou depois do arrasto");
}

/// ⚠️ Controlo: sem ninguém a mexer, o Show não se interrompe a si mesmo (a releitura passa pelo
/// MESMO `f32` que o `install`) e chega exato.
#[test]
fn um_show_sem_toque_chega_exato() {
    let mut m = mesa();
    m.show_hover();
    for i in 0..19 {
        assert!(!m.quadro(0.1), "um quadro sem gesto avisou o undo");
        assert!(m.em_voo(), "o Show parou sozinho no quadro {i}");
    }
    for _ in 0..3 {
        m.quadro(0.1);
    }
    assert!(!m.em_voo());
    assert!((m.x() - 10.0).abs() < 1e-6, "não chegou: x {}", m.x());
    assert_eq!(
        live_role(&m.machines, Some(m.host)),
        StateRole::ALL.iter().position(|&r| r == StateRole::Hover)
    );
}

/// ⚠️ Controlo: um objeto que um MOTOR conduz (a timeline) não interrompe o Show.
#[test]
fn um_objeto_que_um_motor_conduz_nao_interrompe() {
    let mut m = mesa();
    m.conduzido = true;
    m.show_hover();
    m.quadro(0.5);
    m.sim
        .world_mut()
        .get_mut::<Transform>(m.e)
        .unwrap()
        .translation
        .x = -7.0;
    m.quadro(0.1);
    assert!(m.em_voo(), "o motor interrompeu o Show");
}

/// Depois de pousar, um Show novo parte da cena e chega.
#[test]
fn depois_do_gesto_o_show_seguinte_parte_da_cena() {
    let mut m = mesa();
    m.show_hover();
    m.quadro(0.5);
    m.pinta(VERDE);
    m.quadro(0.1);
    m.show_hover();
    assert!(m.em_voo(), "o Show seguinte foi recusado");
    m.quadro(3.0);
    assert!((m.x() - 10.0).abs() < 1e-6, "não chegou: x {}", m.x());
    assert_eq!(m.tinta(), tinta(LARANJA), "não chegou à cor do Hover");
}
