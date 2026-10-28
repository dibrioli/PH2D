//! O ponto de simulação no mundo (`docs/BlenderConstraints/07_W6_pontas.md` §5): o dono segue a
//! partícula `id` do objecto Motion; morta, ele volta à pose dele; renascida, volta a segui-la.

use ph2d_constraints::{Kind, PontoDeSimulacao};
use ph2d_constraints_ecs::{Constraints, Estado, Estados, depois_do_motion, evaluate};
use ph2d_ecs::motion_generator::Particulas;
use ph2d_ecs::{SimWorld, StableId, Transform};
use ph2d_preview_drive::PreviewDrive;

const MOTION: StableId = StableId(20);

fn em(x: f32, y: f32) -> Transform {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t.translation.y = y;
    t
}

fn quadro(sim: &mut SimWorld, drive: &mut PreviewDrive) {
    evaluate(sim, drive);
    drive.settle();
}

/// A corrente do tique `k`: a partícula 7 anda em parábola, a 8 só vive em `k ≥ 3`.
fn corrente(k: u32) -> std::collections::BTreeMap<u32, [f64; 2]> {
    let t = f64::from(k) * 0.25;
    let mut m = std::collections::BTreeMap::from([(7, [t, 0.5 + t * t])]);
    if k >= 3 {
        m.insert(8, [-t, 1.0]);
    }
    m
}

fn publica(sim: &mut SimWorld, m: std::collections::BTreeMap<u32, [f64; 2]>) {
    sim.world_mut()
        .insert_resource(Particulas([(MOTION, m)].into()));
}

/// O objecto Motion rodado 30° e escalado 2 (o mesmo afim do `vertice` do ponto de forma) e um dono
/// autorado em (5, 5) preso à partícula 7.
fn cena() -> (SimWorld, ph2d_ecs::Entity, u32) {
    let mut sim = SimWorld::new();
    let mut objeto = em(3.0, -1.0);
    objeto.rotation = 30f32.to_radians();
    objeto.scale.x = 2.0;
    objeto.scale.y = 2.0;
    sim.world_mut().spawn((MOTION, objeto));
    let mut c = Constraints::default();
    let item = c.push(
        Kind::PontoDeSimulacao(PontoDeSimulacao { id: 7 }),
        Some(MOTION),
    );
    let dono = sim.world_mut().spawn((StableId(1), em(5.0, 5.0), c)).id();
    (sim, dono, item)
}

fn no_mundo(p: [f64; 2]) -> [f64; 2] {
    let (s, c) = 30f64.to_radians().sin_cos();
    [
        3.0 + 2.0 * (c * p[0] - s * p[1]),
        -1.0 + 2.0 * (s * p[0] + c * p[1]),
    ]
}

/// O quadro do PRODUTO: a pilha lê as partículas do cozimento ANTERIOR (`antes`), o Motion coze
/// (`agora`), e o passe depois do cozimento relê o dono preso.
fn quadro_do_produto(
    sim: &mut SimWorld,
    drive: &mut PreviewDrive,
    antes: Option<std::collections::BTreeMap<u32, [f64; 2]>>,
    agora: std::collections::BTreeMap<u32, [f64; 2]>,
) {
    match antes {
        Some(m) => publica(sim, m),
        None => {
            sim.world_mut().remove_resource::<Particulas>();
        }
    }
    evaluate(sim, drive);
    publica(sim, agora);
    depois_do_motion(sim, drive);
    drive.settle();
}

/// ⭐ ZERO atraso (CPU): no quadro `k` o dono está na partícula do cozimento `k`, embora a pilha
/// tenha corrido antes dele com a do `k − 1`; por scrub, o mesmo bit.
#[test]
fn a_pinned_object_follows_its_particle_and_play_equals_scrub() {
    let (mut sim, dono, item) = cena();
    let mut drive = PreviewDrive::default();
    let mut tocado = Vec::new();
    for k in 0..12u32 {
        let antes = k.checked_sub(1).map(corrente);
        quadro_do_produto(&mut sim, &mut drive, antes, corrente(k));
        let t = *sim.world().get::<Transform>(dono).unwrap();
        let want = no_mundo(corrente(k)[&7]);
        assert!(
            (f64::from(t.translation.x) - want[0]).abs() < 1e-4
                && (f64::from(t.translation.y) - want[1]).abs() < 1e-4,
            "tique {k}: {:?} ≠ {want:?} (atraso de um quadro?)",
            t.translation
        );
        assert_eq!(
            sim.world().resource::<Estados>().of(dono, item),
            Some(Estado::Activa)
        );
        tocado.push(t);
    }
    // O scrub: os mesmos tiques por ordem salteada (a pilha lê o tique visitado antes) dão o MESMO bit.
    let (mut sim, dono, _) = cena();
    let mut drive = PreviewDrive::default();
    let mut antes = None;
    for k in [9, 2, 11, 0, 5, 5, 3, 10, 1, 7, 4, 8, 6] {
        quadro_do_produto(&mut sim, &mut drive, antes, corrente(k));
        antes = Some(corrente(k));
        let t = *sim.world().get::<Transform>(dono).unwrap();
        assert_eq!(t, tocado[k as usize], "tique {k} por scrub ≠ a tocar");
    }
}

#[test]
fn a_dead_particle_gives_the_owner_back_its_pose_and_a_reborn_one_takes_it_again() {
    let (mut sim, dono, item) = cena();
    let mut drive = PreviewDrive::default();
    // Antes de nascer: sem alvo, o dono fica onde está (nunca na origem).
    publica(
        &mut sim,
        corrente(3).into_iter().filter(|(i, _)| *i != 7).collect(),
    );
    quadro(&mut sim, &mut drive);
    assert_eq!(*sim.world().get::<Transform>(dono).unwrap(), em(5.0, 5.0));
    assert_eq!(
        sim.world().resource::<Estados>().of(dono, item),
        Some(Estado::SemAlvo)
    );
    // Nasce: segue-a.
    publica(&mut sim, corrente(4));
    quadro(&mut sim, &mut drive);
    assert_ne!(*sim.world().get::<Transform>(dono).unwrap(), em(5.0, 5.0));
    // Morre: a pose AUTORADA volta, ao bit.
    publica(
        &mut sim,
        corrente(5).into_iter().filter(|(i, _)| *i != 7).collect(),
    );
    quadro(&mut sim, &mut drive);
    assert_eq!(*sim.world().get::<Transform>(dono).unwrap(), em(5.0, 5.0));
    // Sem leitura nenhuma do Motion (o objecto não cozeu): o mesmo.
    sim.world_mut().remove_resource::<Particulas>();
    quadro(&mut sim, &mut drive);
    assert_eq!(*sim.world().get::<Transform>(dono).unwrap(), em(5.0, 5.0));
    // O mesmo id renasce (o scrub para trás, ou o `ID_WRAP`): volta a segui-la.
    publica(&mut sim, corrente(6));
    quadro(&mut sim, &mut drive);
    let t = *sim.world().get::<Transform>(dono).unwrap();
    let want = no_mundo(corrente(6)[&7]);
    assert!((f64::from(t.translation.x) - want[0]).abs() < 1e-4);
    assert_eq!(
        sim.world().resource::<Estados>().of(dono, item),
        Some(Estado::Activa)
    );
}

/// Gravar e abrir: o item sobrevive pelo `WorldSnapshot` (postcard), e a variante é APPEND — os
/// índices das que já estavam gravadas não mudam (o ponto de forma continua a 12).
#[test]
fn a_particle_pin_survives_save_and_open() {
    use ph2d_ecs::scene::{ComponentRegistry, WorldSnapshot, snapshot_to_world, world_to_snapshot};
    use ph2d_ecs::{TransformPropagationState, WorklistBuf};
    let mut reg = ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    ph2d_constraints_ecs::register_constraint_components(&mut reg);
    let (mut sim, _, _) = cena();
    let gravada: Vec<Constraints> = sim
        .world_mut()
        .query::<&Constraints>()
        .iter(sim.world())
        .cloned()
        .collect();
    let mut snap = WorldSnapshot::new();
    let (mut st, mut wl) = (
        TransformPropagationState::new(sim.world_mut()),
        WorklistBuf::default(),
    );
    world_to_snapshot(sim.world_mut(), &mut st, &mut wl, &reg, &mut snap).unwrap();
    let bytes = postcard::to_allocvec(&snap).unwrap();
    let lido: WorldSnapshot = postcard::from_bytes(&bytes).unwrap();
    let mut outro = SimWorld::new();
    snapshot_to_world(outro.world_mut(), &lido, &reg).unwrap();
    let aberta: Vec<Constraints> = outro
        .world_mut()
        .query::<&Constraints>()
        .iter(outro.world())
        .cloned()
        .collect();
    assert_eq!(aberta, gravada);
    assert_eq!(
        aberta[0].items[0].kind,
        Kind::PontoDeSimulacao(PontoDeSimulacao { id: 7 })
    );
    let forma = Kind::PontoDeForma(ph2d_constraints::PontoDeForma::default());
    assert_eq!(postcard::to_allocvec(&forma).unwrap()[0], 12);
    let simulacao = postcard::to_allocvec(&aberta[0].items[0].kind).unwrap();
    assert_eq!(
        simulacao,
        [17, 7],
        "a variante nova é a última (17), o id em varint"
    );
}
