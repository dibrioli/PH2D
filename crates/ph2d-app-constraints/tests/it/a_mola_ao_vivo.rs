//! ⭐ **A mola ao vivo com o tempo parado** (W6, `docs/BlenderConstraints/07_W6_pontas.md` ponta 4, o
//! idioma do Spine 4.2): arrastar o corpo com a timeline PARADA faz a mola reagir no relógio de parede,
//! só no preview, e assentar de volta na resposta determinística. Sem movimento, nada nasce.

use bevy_ecs::entity::Entity;
use ph2d_app_constraints::quadro;
use ph2d_app_constraints::relogio::{Parede, Relogio};
use ph2d_constraints::{Canais, Kind, Spring};
use ph2d_constraints_ecs::{Constraints, Molas};
use ph2d_ecs::{SimWorld, StableId, Transform, world_transform};
use ph2d_preview_drive::PreviewDrive;
use ph2d_timeline::TimelineDoc;

/// Um quadro do app ao instante parado `t = 1 s`, com o relógio de parede em `parede`: devolve o `x`
/// mostrado do dono.
fn quadro_parado(c: &mut Cena, parede: f64) -> f32 {
    c.sim.world_mut().insert_resource(Parede(parede));
    let r = Relogio {
        doc: &c.doc,
        t: 1.0,
        laco: None,
    };
    quadro(
        &mut c.sim,
        &mut ph2d_vec_scene::VecScene::new(),
        &mut c.drive,
        Some(r),
    );
    c.drive.settle();
    world_transform(c.sim.world(), c.dono)
        .unwrap()
        .translation
        .x
}

struct Cena {
    sim: SimWorld,
    drive: PreviewDrive,
    doc: TimelineDoc,
    dono: Entity,
}

fn cena() -> Cena {
    let mut sim = SimWorld::new();
    let mut c = Constraints::default();
    c.push(
        Kind::Spring(Spring {
            canais: Canais::LOCATION,
            ..Spring::default()
        }),
        None,
    );
    let dono = sim
        .world_mut()
        .spawn((StableId(1), Transform::default(), c))
        .id();
    Cena {
        sim,
        drive: PreviewDrive::default(),
        doc: TimelineDoc::default(),
        dono,
    }
}

fn vivas(sim: &SimWorld) -> usize {
    sim.world()
        .get_resource::<Molas>()
        .map_or(0, |m| m.vivas.len())
}

/// (a) Parado, o corpo arrastado 1 m: a mola fica para trás nos quadros seguintes e assenta de volta
/// no determinístico (0 para um corpo sem animação), largando o estado ao vivo.
#[test]
fn a_dragged_body_with_the_timeline_stopped_shakes_the_spring_and_it_settles() {
    let mut c = cena();
    let dt = 1.0 / 60.0;
    let mut parede = 0.0;
    for _ in 0..3 {
        assert_eq!(quadro_parado(&mut c, parede), 0.0);
        parede += dt;
    }
    let dono = c.dono;
    c.sim
        .world_mut()
        .get_mut::<Transform>(dono)
        .unwrap()
        .translation
        .x = 1.0;
    let mut xs = Vec::new();
    for _ in 0..600 {
        xs.push(quadro_parado(&mut c, parede));
        parede += dt;
    }
    assert!(xs[0] < 0.9, "a mola não ficou para trás: {}", xs[0]);
    assert!(
        xs[1] > xs[0],
        "a mola não anda no relógio de parede: {:?}",
        &xs[..3]
    );
    assert_eq!(
        *xs.last().unwrap(),
        1.0,
        "a mola não assentou no determinístico"
    );
    assert_eq!(vivas(&c.sim), 0, "o estado ao vivo sobreviveu ao repouso");
}

/// (c) Parado e sem movimento: o caminho de custo zero — nenhuma mola ao vivo nasce.
#[test]
fn a_still_scene_with_the_timeline_stopped_creates_no_live_spring() {
    let mut c = cena();
    for i in 0..120 {
        assert_eq!(quadro_parado(&mut c, f64::from(i) / 60.0), 0.0);
        assert_eq!(
            vivas(&c.sim),
            0,
            "uma mola ao vivo nasceu sem nada se mexer (quadro {i})"
        );
    }
}

/// B23.2: a mola ao vivo a balançar pede o quadro seguinte ao laço a pedido (sem entrada, a 4 q/s ela
/// saltava); assentada, deixa-o dormir.
#[test]
fn a_mola_ao_vivo_pede_quadro_ate_assentar() {
    use ph2d_app_host::quadro::{PRAZO, no_laco};
    let acorda = || {
        let agora = std::time::Instant::now();
        no_laco(|l| l.acorda(&[], agora)) == Some(agora + PRAZO)
    };
    let mut c = cena();
    let dt = 1.0 / 60.0;
    let mut parede = 0.0;
    for _ in 0..3 {
        quadro_parado(&mut c, parede);
        parede += dt;
    }
    assert!(acorda(), "parada e sem arrasto, a mola pediu quadro");
    let dono = c.dono;
    c.sim
        .world_mut()
        .get_mut::<Transform>(dono)
        .unwrap()
        .translation
        .x = 1.0;
    quadro_parado(&mut c, parede);
    parede += dt;
    quadro_parado(&mut c, parede);
    assert!(
        vivas(&c.sim) > 0 && !acorda(),
        "a mola a balançar não pediu quadro"
    );
    for _ in 0..600 {
        parede += dt;
        quadro_parado(&mut c, parede);
        _ = acorda(); // a shell lê o laço depois de cada quadro
    }
    parede += dt;
    quadro_parado(&mut c, parede);
    assert!(
        vivas(&c.sim) == 0 && acorda(),
        "a mola assentada segurou o laço"
    );
}
