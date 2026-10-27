//! Os tipos da onda 3 no mundo: a Spline IK escreve a corrente, o ponto de forma é um alvo.
//! Oráculo: os casos `spline_ik` e `vertice` da fixture (`docs/BlenderConstraints/03_W3_poderosas.md`).

use bevy_ecs::entity::Entity;
use std::sync::Arc;

use ph2d_constraints::{Action, Ajuste, Canais, Kind, PontoDeForma, SplineIk, Spring};
use ph2d_constraints_ecs::{
    Accoes, Caminhos, Constraints, Estado, Estados, Laco, Molas, Pontos, Simulador, evaluate,
};
use ph2d_ecs::{ChildOf, SimWorld, StableId, Transform, world_transform};
use ph2d_preview_drive::PreviewDrive;
use ph2d_skeleton_ecs::Bone;
use ph2d_vec_scene::VertId;

const FIXTURE: &str =
    include_str!("../../../../docs/BlenderConstraints/fixtures/blender_5_2_3_constraints_2d.json");
/// O `Transform` da casa é `f32`.
const TOL: f64 = 1e-4;

fn em(x: f32, y: f32) -> Transform {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t.translation.y = y;
    t
}

fn osso(sim: &mut SimWorld, id: u64, t: Transform, pai: Option<Entity>) -> Entity {
    let e = sim
        .world_mut()
        .spawn((
            StableId(id),
            t,
            Bone {
                length: 1.0,
                strength: 1.0,
                ..Default::default()
            },
        ))
        .id();
    if let Some(p) = pai {
        sim.world_mut().entity_mut(e).insert(ChildOf(p));
    }
    e
}

fn cabeca_e_cauda(sim: &SimWorld, e: Entity) -> [f64; 4] {
    let t = world_transform(sim.world(), e).unwrap();
    let (s, c) = f64::from(t.rotation).sin_cos();
    let l = f64::from(t.scale.x);
    let (x, y) = (f64::from(t.translation.x), f64::from(t.translation.y));
    [x, y, x + c * l, y + s * l]
}

fn quadro(sim: &mut SimWorld, drive: &mut PreviewDrive) {
    evaluate(sim, drive);
    drive.settle();
}

#[test]
fn spline_ik_lays_the_chain_on_the_curve() {
    let o: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    for (modo, ajuste) in [("NONE", Ajuste::Nenhum), ("FIT_CURVE", Ajuste::Esticar)] {
        let mut sim = SimWorld::new();
        let mut drive = PreviewDrive::default();
        let s1 = osso(&mut sim, 1, em(0.0, 0.0), None);
        let s2 = osso(&mut sim, 2, em(1.0, 0.0), Some(s1));
        let s3 = osso(&mut sim, 3, em(1.0, 0.0), Some(s2));
        // A forma do caso `curta`: a poligonal LOCAL sob um afim (1; 0,5), para o gate ler o mundo.
        sim.world_mut().spawn((StableId(20), em(1.0, 0.5)));
        sim.world_mut().insert_resource(Caminhos(
            [(StableId(20), vec![[-1.0, 0.5], [0.0, 0.5], [0.0, 1.5]])].into(),
        ));
        let mut c = Constraints::default();
        let id = c.push(
            Kind::SplineIk(SplineIk { chain: 3, ajuste }),
            Some(StableId(20)),
        );
        sim.world_mut().entity_mut(s3).insert(c);
        let autorado: Vec<Transform> = [s1, s2, s3]
            .iter()
            .map(|&e| *sim.world().get::<Transform>(e).unwrap())
            .collect();
        for _ in 0..3 {
            quadro(&mut sim, &mut drive);
            assert_eq!(
                sim.world().resource::<Estados>().of(s3, id),
                Some(Estado::Activa)
            );
            let caso = &o["spline_ik"][format!("curta.{modo}")];
            for (e, nome) in [(s1, "s1"), (s2, "s2"), (s3, "s3")] {
                let v = &caso[nome];
                let want =
                    ["cabeca_x", "cabeca_y", "cauda_x", "cauda_y"].map(|k| v[k].as_f64().unwrap());
                let got = cabeca_e_cauda(&sim, e);
                for (g, w) in got.iter().zip(want) {
                    assert!((g - w).abs() < TOL, "{modo}.{nome}: {got:?} ≠ {want:?}");
                }
            }
        }
        // Tirar a pilha devolve a corrente ao autorado.
        sim.world_mut().entity_mut(s3).remove::<Constraints>();
        quadro(&mut sim, &mut drive);
        for (e, t) in [s1, s2, s3].iter().zip(&autorado) {
            assert_eq!(sim.world().get::<Transform>(*e).unwrap(), t);
        }
    }
}

#[test]
fn a_point_of_a_shape_is_a_target() {
    let o: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let mut forma = em(3.0, -1.0);
    forma.rotation = 30f32.to_radians();
    forma.scale.x = 2.0;
    forma.scale.y = 2.0;
    sim.world_mut().spawn((StableId(20), forma));
    sim.world_mut().insert_resource(Pontos(
        [(
            StableId(20),
            [[0.0, 0.0], [1.0, 0.5], [0.0, 1.0]]
                .into_iter()
                .zip(1..)
                .map(|(p, i)| (VertId(i), p))
                .collect(),
        )]
        .into(),
    ));
    let mut c = Constraints::default();
    let bom = c.push(
        Kind::PontoDeForma(PontoDeForma { vert: VertId(2) }),
        Some(StableId(20)),
    );
    let fora = c.push(
        Kind::PontoDeForma(PontoDeForma { vert: VertId(9) }),
        Some(StableId(20)),
    );
    c.item_mut(fora).unwrap().enabled = false;
    let dono = sim
        .world_mut()
        .spawn((StableId(1), Transform::IDENTITY, c))
        .id();
    quadro(&mut sim, &mut drive);
    let t = sim.world().get::<Transform>(dono).unwrap();
    let want = &o["vertice"]["dono"];
    assert!((f64::from(t.translation.x) - want["x"].as_f64().unwrap()).abs() < TOL);
    assert!((f64::from(t.translation.y) - want["y"].as_f64().unwrap()).abs() < TOL);
    assert_eq!(
        sim.world().resource::<Estados>().of(dono, bom),
        Some(Estado::Activa)
    );
    // Um índice que a forma não tem é um alvo que falta — dito, não adivinhado.
    sim.world_mut()
        .get_mut::<Constraints>(dono)
        .unwrap()
        .item_mut(fora)
        .unwrap()
        .enabled = true;
    quadro(&mut sim, &mut drive);
    assert_eq!(
        sim.world().resource::<Estados>().of(dono, fora),
        Some(Estado::SemAlvo)
    );
}

/// A acção do oráculo: quadro 1 → x 0, rot 0°; quadro 21 → x 4, rot 90°, linear (a 24 q/s).
fn accao_do_oraculo() -> (f64, ph2d_constraints_ecs::Amostra) {
    let dur = 20.0 / 24.0;
    let f = move |t: f64| {
        let u = (t / dur).clamp(0.0, 1.0);
        [
            Some(4.0 * u),
            None,
            Some(90f64.to_radians() * u),
            None,
            None,
        ]
    };
    (dur, Arc::new(f))
}

#[test]
fn action_picks_the_clip_time_from_the_target_channel() {
    let o: serde_json::Value = serde_json::from_str(FIXTURE).unwrap();
    let (dur, amostra) = accao_do_oraculo();
    for ar in ["-45.0", "0.0", "45.0", "90.0", "135.0"] {
        let mut sim = SimWorld::new();
        let mut drive = PreviewDrive::default();
        let mut alvo = Transform::IDENTITY;
        alvo.rotation = ar.parse::<f32>().unwrap().to_radians();
        sim.world_mut().spawn((StableId(20), alvo));
        let mut c = Constraints::default();
        let id = c.push(
            Kind::Action(Action {
                ate_t: dur,
                ..Action::default()
            }),
            Some(StableId(20)),
        );
        let dono = sim
            .world_mut()
            .spawn((StableId(1), Transform::IDENTITY, c))
            .id();
        // Sem o clip, o item diz porquê e não mexe.
        quadro(&mut sim, &mut drive);
        assert_eq!(
            sim.world().resource::<Estados>().of(dono, id),
            Some(Estado::SemClip)
        );
        sim.world_mut()
            .insert_resource(Accoes([((dono.to_bits(), 0), amostra.clone())].into()));
        quadro(&mut sim, &mut drive);
        let t = sim.world().get::<Transform>(dono).unwrap();
        let want = &o["action"][ar];
        let n = |k: &str| want[k].as_f64().unwrap();
        assert!(
            (f64::from(t.translation.x) - n("x")).abs() < TOL,
            "{ar}: {t:?}"
        );
        assert!(
            (f64::from(t.rotation).to_degrees() - n("rot_deg")).abs() < 1e-3,
            "{ar}: {t:?}"
        );
    }
}

/// A entrada da cena 5: o corpo sobe de 0 a 3 entre os quadros 5 e 15 e pára.
fn salto(k: i64) -> [f64; 3] {
    let u = ((k as f64 - 5.0) / 10.0).clamp(0.0, 1.0);
    [0.0, 3.0 * u * u * (3.0 - 2.0 * u), 0.0]
}

/// ⭐ Determinismo (plano §2.4): tocar 0→40 e arrastar a timeline 0→40→12→40 dão o MESMO bit em
/// cada quadro visitado, e o atraso não é zero (a mola existe).
#[test]
fn a_spring_rewinds_to_the_same_bit() {
    let mola = Spring::default();
    let mut tocar = Simulador::default();
    let play: Vec<[f64; 3]> = (0..=40)
        .map(|f| tocar.atraso(f, &mola, 24.0, &mut salto))
        .collect();
    assert!(
        play.iter().any(|a| a[1].abs() > 0.1),
        "a mola atrasa: {play:?}"
    );
    let mut arrastar = Simulador::default();
    for f in [0, 40, 12, 40, 7, 8, 9, 33] {
        let got = arrastar.atraso(f, &mola, 24.0, &mut salto);
        assert_eq!(
            got.map(f64::to_bits),
            play[f as usize].map(f64::to_bits),
            "quadro {f}"
        );
    }
    // Entre quadros (o ecrã a 60 Hz sobre 24 q/s): a mistura dos dois quadros, ao bit em qualquer ordem.
    let mut tocar = Simulador::default();
    let entre: Vec<[f64; 3]> = (0..=100)
        .map(|i| tocar.atraso_em(f64::from(i) * 0.4, &mola, 24.0, &mut salto))
        .collect();
    assert!(
        entre[13..=50].windows(2).all(|w| w[0] != w[1]),
        "com o corpo a andar (quadros 5–20) o atraso muda a cada imagem"
    );
    let mut saltar = Simulador::default();
    for i in [77, 3, 50, 49, 100, 12] {
        let got = saltar.atraso_em(f64::from(i) * 0.4, &mola, 24.0, &mut salto);
        assert_eq!(
            got.map(f64::to_bits),
            entre[i as usize].map(f64::to_bits),
            "q {i}"
        );
    }
    // Editar a animação invalida o que já se calculou: a resposta segue a entrada NOVA.
    let mut parado = |_| [0.0; 3];
    assert_eq!(arrastar.atraso(33, &mola, 24.0, &mut parado), [0.0; 3]);
}

#[test]
fn a_spring_adds_its_lag_at_its_place_in_the_stack() {
    let mut sim = SimWorld::new();
    let mut drive = PreviewDrive::default();
    let mut c = Constraints::default();
    let id = c.push(
        Kind::Spring(Spring {
            canais: Canais::LOCATION,
            ..Spring::default()
        }),
        None,
    );
    c.item_mut(id).unwrap().influence = 0.5;
    let dono = sim.world_mut().spawn((StableId(1), em(1.0, 2.0), c)).id();
    sim.world_mut().insert_resource(Molas {
        atrasos: [((dono.to_bits(), id), [0.4, -2.0, 1.0])].into(),
        ..Molas::default()
    });
    quadro(&mut sim, &mut drive);
    let t = sim.world().get::<Transform>(dono).unwrap();
    assert_eq!(
        (t.translation.x, t.translation.y, t.rotation),
        (1.2, 1.0, 0.0)
    );
}

/// Um corpo que nunca pára: vai e volta 3 m num ciclo de 48 quadros (a entrada de 48 é a de 0).
fn balanco(k: i64) -> [f64; 3] {
    [
        3.0 * (std::f64::consts::TAU * k as f64 / 48.0).sin(),
        0.0,
        0.0,
    ]
}

/// O maior passo do atraso entre duas imagens a 0,2 quadros, longe das pontas do laço.
fn passo_de_dentro(s: &mut Simulador, laco: Laco, entrada: &mut dyn FnMut(i64) -> [f64; 3]) -> f64 {
    let mola = Spring::default();
    let v: Vec<[f64; 3]> = (10..230)
        .map(|i| s.atraso_no_laco(f64::from(i) * 0.2, Some(laco), &mola, 24.0, &mut *entrada))
        .collect();
    v.windows(2)
        .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
        .fold(0.0, f64::max)
}

/// ⭐ Report do dono 25/10: «no loop e pingpong deveria haver uma transição inteligente nos extremos».
/// Em LAÇO a mola não renasce em repouso no recomeço: corre a órbita que se repete (a do laço tocado
/// há muito tempo), e o fim junta-se ao começo como qualquer outro par de imagens.
#[test]
fn a_loop_joins_its_end_to_its_start() {
    let mola = Spring::default();
    let laco = Laco {
        de: 0.0,
        ate: 48.0,
        vaivem: false,
        para_a_frente: true,
    };
    let mut s = Simulador::default();
    let dentro = passo_de_dentro(&mut s, laco, &mut balanco);
    let fim = s.atraso_no_laco(47.8, Some(laco), &mola, 24.0, &mut balanco);
    let comeco = s.atraso_no_laco(0.0, Some(laco), &mola, 24.0, &mut balanco);
    let costura = (fim[0] - comeco[0]).hypot(fim[1] - comeco[1]);
    assert!(
        costura <= 1.5 * dentro,
        "a costura do laço salta {costura} m (dentro: {dentro} m por imagem)"
    );
    // Arrastar dentro do laço dá o bit de tocar.
    let mut outro = Simulador::default();
    for q in [30.4, 0.6, 47.8, 12.2] {
        let a = outro.atraso_no_laco(q, Some(laco), &mola, 24.0, &mut balanco);
        let b = s.atraso_no_laco(q, Some(laco), &mola, 24.0, &mut balanco);
        assert_eq!(a.map(f64::to_bits), b.map(f64::to_bits), "q {q}");
    }
}

/// Em VAIVÉM a volta não é o filme da ida ao contrário (uma mola não «desbalança»): ela continua a
/// reagir, para a frente no tempo, ao corpo que volta — e vira na ponta sem salto.
#[test]
fn ping_pong_turns_without_a_jump_and_never_runs_backwards() {
    let mola = Spring::default();
    // O corpo vai de 0 a 3 m nos quadros 0–24 e espera até ao 48.
    let mut ida = |k: i64| {
        let u = (k as f64 / 24.0).clamp(0.0, 1.0);
        [3.0 * u * u * (3.0 - 2.0 * u), 0.0, 0.0]
    };
    let laco = |para_a_frente| Laco {
        de: 0.0,
        ate: 48.0,
        vaivem: true,
        para_a_frente,
    };
    let mut s = Simulador::default();
    let dentro = passo_de_dentro(&mut s, laco(true), &mut ida);
    let antes = s.atraso_no_laco(47.9, Some(laco(true)), &mola, 24.0, &mut ida);
    let depois = s.atraso_no_laco(47.9, Some(laco(false)), &mola, 24.0, &mut ida);
    let virada = (antes[0] - depois[0]).hypot(antes[1] - depois[1]);
    assert!(
        virada <= 1.5 * dentro,
        "a virada salta {virada} m (dentro: {dentro})"
    );
    let vai = s.atraso_no_laco(6.0, Some(laco(true)), &mola, 24.0, &mut ida);
    let volta = s.atraso_no_laco(6.0, Some(laco(false)), &mola, 24.0, &mut ida);
    assert!(
        (vai[0] - volta[0]).abs() > 1e-3,
        "na volta a mola repete a ida ao contrário: {vai:?} = {volta:?}"
    );
}

/// ⭐ O `Simulador` genérico (`N` canais, 06_W5 §3.1) dá à pose da Spring (`N = 3`) o MESMO bit que o
/// de antes: a impressão FNV-1a de saltos, instantes contínuos, laço e vaivém, medida no código de
/// `fe0777808d` (antes da troca).
#[test]
fn the_pose_spring_is_the_same_bit_after_the_generic_simulator() {
    let mola = Spring {
        ressalto: 0.8,
        ..Spring::default()
    };
    let mut bits = Vec::new();
    let mut s = Simulador::default();
    for f in [0, 5, 13, 40, 7, 33, 34, 90] {
        bits.extend(s.atraso(f, &mola, 24.0, &mut salto));
    }
    for q in [12.4, 3.7, 40.2, 40.9] {
        bits.extend(s.atraso_em(q, &mola, 24.0, &mut salto));
    }
    for (vaivem, para_a_frente) in [(false, true), (true, true), (true, false)] {
        let laco = Laco {
            de: 0.0,
            ate: 48.0,
            vaivem,
            para_a_frente,
        };
        for q in [30.4, 0.6, 47.8, 12.2] {
            bits.extend(s.atraso_no_laco(q, Some(laco), &mola, 24.0, &mut balanco));
        }
    }
    let fnv = bits.iter().fold(0xcbf2_9ce4_8422_2325_u64, |h, v| {
        v.to_bits()
            .to_le_bytes()
            .iter()
            .fold(h, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x0100_0000_01b3))
    });
    assert_eq!((bits.len(), fnv), (72, IMPRESSAO), "impressão {fnv:#018x}");
}

const IMPRESSAO: u64 = 0x40b3_b313_e43b_d8ce;

/// ⭐ A mola de UM valor (o «Valor», 06_W5 §3.1) é a lei do `motion.spring` passo a passo: o estado
/// parte do repouso e cada quadro é um `passo` com a entrada desse quadro.
#[test]
fn a_one_value_spring_is_the_law_step_by_step() {
    use ph2d_constraints_ecs::Lei;
    use ph2d_node_motion_spring::law::{MODE_TIME, passo, physics_of};
    let lei = Lei {
        duracao: 0.5,
        ressalto: 0.6,
        angular: [false],
    };
    let (tension, friction) = physics_of(MODE_TIME, 0.0, 0.0, 0.5, 0.6);
    let dt = (1.0 / 24.0_f64) as f32;
    let entrada = |k: i64| [if k < 5 { 0.0 } else { 270.0 }];
    let mut s: Simulador<1> = Simulador::default();
    let (mut x, mut v, mut pico) = (0.0_f32, 0.0_f32, 0.0_f32);
    for f in 0..60 {
        if f > 0 {
            (x, v) = passo(x, v, entrada(f)[0] as f32, tension, friction, dt);
        }
        let got = s.atraso(f, lei, 24.0, &mut |k| entrada(k))[0] + entrada(f)[0];
        assert!(
            (got - f64::from(x)).abs() < 1e-9,
            "quadro {f}: {got} contra a lei {x}"
        );
        pico = pico.max(x);
    }
    assert!(
        pico > 270.0,
        "com ressalto 0,6 a mola passa do alvo (pico {pico})"
    );
}
