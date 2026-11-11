//! **A JUNTA e o MUNDO chegam ao solver por TIQUE** (GEF02 onda 8, item 2 do handoff): como a forma
//! (8.3), uma curva na rigidez de uma mola ou na gravidade da Cena dá a mesma corrida com um ou três
//! tiques por quadro. Medido antes (sonda, 10/11): 44,3 cm de desvio na gravidade, 1,9 cm na mola.

use crate::physics_state::PhysicsState;
use crate::physics_tests::registry;
use ph2d_core::Vec2;
use ph2d_ecs::canal::familia::{JUNTA, MUNDO};
use ph2d_ecs::{Entity, FieldKey, Name, SimWorld, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, JointKind, PhysicsBridge};
use ph2d_physics_ecs::{PhysicsJoint, RigidBody};
use ph2d_preview_drive::PreviewDrive;
use ph2d_timeline::{PropKind, TimelineDoc};

const DT: f64 = 1.0 / 60.0;

fn chaves(doc: &mut TimelineDoc, e: u64, kk: FieldKey, ks: &[(f64, f32)]) {
    for &(t, v) in ks {
        let (t, v) = (
            ph2d_anim::RationalTime::from_seconds(t),
            ph2d_anim::AnimValue::Float(v),
        );
        doc.insert_key(e, PropKind::Field(kk), t, v, ph2d_anim::Interp::Linear);
    }
}

fn corpo(sim: &mut SimWorld, nome: &str, kind: BodyKind, y: f32) -> Entity {
    let bola = ColliderShape::Ball { radius: 0.2 };
    let c = Collider {
        shape: bola,
        ..Collider::default()
    };
    let t = Transform::from_translation(Vec2::new(0.0, y));
    sim.world_mut()
        .spawn((Name::new(nome), RigidBody { kind }, c, t))
        .id()
}

/// A mola (rigidez animada) ou a queda (gravidade da Cena animada). Devolve (o corpo, o dono da curva).
fn cena(junta: bool) -> (SimWorld, Entity, u64, TimelineDoc) {
    let mut sim = SimWorld::new();
    let mut doc = TimelineDoc::new();
    let (e, dono) = if junta {
        corpo(&mut sim, "Hook", BodyKind::Static, 6.0);
        let e = corpo(&mut sim, "Load", BodyKind::Dynamic, 5.0);
        let mola = PhysicsJoint {
            body_a: ph2d_ecs::stable_name_id("Hook"),
            body_b: ph2d_ecs::stable_name_id("Load"),
            kind: JointKind::Spring,
            max_length: 1.0,
            ..PhysicsJoint::default()
        };
        let t = Transform::from_translation(Vec2::new(0.0, 6.0));
        let m = sim.world_mut().spawn((Name::new("Spring"), mola, t)).id();
        ph2d_physics_ecs::resolve_body_names(sim.world_mut());
        let kk = FieldKey::new(JUNTA, ph2d_ecs::canal_junta::RIGIDEZ, 0, 0);
        let ks = [(0.1, 5.0), (0.5, 300.0), (0.9, 5.0), (1.3, 300.0)];
        chaves(&mut doc, m.to_bits(), kk, &ks);
        (e, m.to_bits())
    } else {
        let e = corpo(&mut sim, "Fall", BodyKind::Dynamic, 50.0);
        ph2d_ecs::garante_cena(sim.world_mut());
        let c = ph2d_ecs::bits_da_cena(sim.world()).expect("a Cena");
        let kk = FieldKey::new(MUNDO, ph2d_ecs::canal_mundo::GRAVIDADE_Y, 0, 0);
        let ks = [(0.1, -9.8), (0.5, -60.0), (0.9, 20.0), (1.3, -60.0)];
        chaves(&mut doc, c, kk, &ks);
        (e, c)
    };
    (sim, e, dono, doc)
}

/// O app inteiro de um caso: a cena, a ponte, o estado da física e o ledger do quadro.
struct App {
    sim: SimWorld,
    doc: TimelineDoc,
    ponte: PhysicsBridge,
    estado: PhysicsState,
    drive: PreviewDrive,
    e: Entity,
}

impl App {
    fn novo(junta: bool) -> Self {
        let (sim, e, _, doc) = cena(junta);
        let (ponte, estado, drive) = Default::default();
        Self {
            sim,
            doc,
            ponte,
            estado,
            drive,
            e,
        }
    }

    /// Um quadro pela porta do produto: a timeline no alvo, os canais do quadro, o `dispatch`.
    fn quadro(&mut self, alvo: u64) {
        let t = alvo as f64 * DT;
        ph2d_timeline::apply_from_doc(self.sim.world_mut(), &mut self.doc, t);
        let (reg, doc) = (registry(), &self.doc);
        crate::aplicar_canais(
            &mut self.sim,
            &reg,
            &mut self.ponte,
            &mut self.estado,
            (&mut self.drive, doc),
        );
        let mut relogio = ph2d_core::Playhead::new(DT);
        relogio.seek(t);
        relogio.play();
        let livros = (
            &mut self.drive,
            &mut self.estado.conducao,
            &mut self.estado.mundo,
        );
        crate::bridge::dispatch::dispatch(
            &mut self.ponte,
            &mut self.sim,
            &relogio,
            DT,
            &mut self.doc,
            true,
            ph2d_physics_ecs::PlayerInput::default(),
            &mut ph2d_physics_ecs::InputTape::new(),
            livros,
        );
    }

    fn y(&self) -> f32 {
        self.ponte.body_pose(self.e).map_or(f32::NAN, |p| p.1)
    }
}

/// A altura do corpo em cada tique múltiplo de 3, a `passo` tiques por quadro.
fn corre(junta: bool, passo: u64) -> (Vec<f32>, App) {
    let mut app = App::novo(junta);
    let mut ys = Vec::new();
    for alvo in (passo..=120).step_by(passo as usize) {
        app.quadro(alvo);
        if alvo % 3 == 0 {
            ys.push(app.y());
        }
    }
    (ys, app)
}

/// ⭐ Três tiques por quadro (máquina lenta) dão a corrida de um por quadro, na mola e na queda.
/// *Mutação: o `put` sem a junta, ou sem o mundo ⇒ o caso dele reprova.*
#[test]
fn a_mola_e_a_gravidade_da_curva_dao_a_mesma_corrida_com_varios_tiques_por_quadro() {
    for (nome, junta) in [("a mola", true), ("a queda", false)] {
        let (um, _) = corre(junta, 1);
        let (tres, _) = corre(junta, 3);
        let pior = um
            .iter()
            .zip(&tres)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0, f32::max);
        assert!(
            pior < 1e-5,
            "{nome}: {pior:.5} m de desvio entre 1 e 3 tiques"
        );
    }
}

/// O autorado não muda pelos tiques: a ponte guarda a gravidade autorada, a curva conduz o vivo.
#[test]
fn os_tiques_do_mundo_guardam_o_autorado_do_documento() {
    let (_, app) = corre(false, 3);
    let g = ph2d_physics_ecs::PhysicsSettings::default().gravity_y;
    assert_eq!(app.ponte.settings().gravity_y, g, "a gravidade autorada");
    assert_ne!(
        app.ponte.settings_vivos().gravity_y,
        g,
        "a curva conduz o mundo vivo"
    );
}

/// A mão ganha à curva no quadro em que muda o autorado — em TODOS os tiques desse quadro.
/// *Mutação: o tique sem a mão do quadro ⇒ o último tique devolve a curva.*
#[test]
fn a_mao_ganha_a_curva_em_todos_os_tiques_do_quadro() {
    let mut app = App::novo(false);
    for alvo in (3..=60).step_by(3) {
        app.quadro(alvo);
    }
    let mut s = app.ponte.settings();
    s.gravity_y = -1.5;
    app.ponte.set_settings(s);
    app.quadro(63);
    assert_eq!(
        app.ponte.settings_vivos().gravity_y,
        -1.5,
        "a mão no quadro em que mudou"
    );
}
