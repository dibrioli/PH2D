//! **A MEDIÇÃO na porta do PRODUTO** (TOP-20 #13) — o que o plano 10 §6.3 prometeu
//! re-medir.
//!
//! ⚠️⚠️ **A wave anterior desta linha pagou a lição:** o tecto da fábrica foi medido
//! na porta de CÓPIA (`17 %` de um quadro) e a porta do PRODUTO deu **`32 %`** —
//! *uma sonda que mede um sucedâneo para sempre mede outro programa*. Aqui as duas
//! sondas anteriores mediram o `move_character` sozinho (o `character_slide_probe`)
//! e a lei sozinha (o corpus); esta mede a ponte inteira, que é o que corre.
//!
//! Correr: `cargo test -p ph2d-physics-ecs --test it measure_topdown -- --nocapture --ignored`

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, LockRotation, PhysicsBridge, RigidBody, TopDownPlayer,
};
use ph2d_platformer::PlayerInput;

const V: f32 = 4.0;
const DT: f32 = 1.0 / 60.0;
const ORCAMENTO: f32 = V * DT;

fn cena(n: usize) -> (SimWorld, PhysicsBridge, Vec<Entity>) {
    let mut sim = SimWorld::new();
    sim.world_mut().spawn((
        Name::new("Parede"),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 1.0,
                // ⚠️⚠️ **A 1.ª redacção pôs `20 000` aqui para caber mil corpos, e
                // com esse AABB o cast devolve `hits = 0` e o corpo ATRAVESSA a
                // parede.** O sintoma lê-se como um resultado — «andou o orçamento
                // inteiro» é exactamente o que esta wave quer ver — e só o corpo a
                // aparecer DENTRO do cenário o denuncia. A parede acompanha a
                // população, e nada mais.
                half_y: (n as f32 * 0.3).max(20.0),
            },
            ..Collider::default()
        },
        Transform::from_translation(Vec2::new(0.0, 0.0)),
    ));
    let mut quem = Vec::with_capacity(n);
    for k in 0..n {
        quem.push(
            sim.world_mut()
                .spawn((
                    Name::new(format!("P{k}")),
                    RigidBody {
                        kind: BodyKind::Kinematic,
                    },
                    Collider {
                        shape: ColliderShape::Ball { radius: 0.2 },
                        ..Collider::default()
                    },
                    LockRotation,
                    TopDownPlayer {
                        speed: V,
                        // ⚠️⚠️ **`Free`, e a 1.ª redacção desta sonda deixou o
                        // default (`EightWay`) — ela mediu o QUANTIZADOR:** a 20°
                        // o encaixe de 45 manda a direcção para 0°, logo o corpo ia
                        // de cabeça contra a parede e a tabela lia um «limiar» a
                        // 22,5° que é a fronteira do encaixe, não a lei do deslize.
                        direction_mode: 0,
                        ..TopDownPlayer::default()
                    },
                    // ⚠️⚠️ **LONGE da parede, e a 1.ª redacção desta sonda punha-o
                    // ENCOSTADO (`x = −1,21`) — o que ela mediu foi um corpo DENTRO
                    // da parede.** O primeiro tique de uma cena tem o BVH vazio (a
                    // casa já o documenta no irmão `player_push`), então o
                    // controlador não vê nada, deixa passar o pedido inteiro, e a
                    // partir daí a leitura é de um corpo preso no cenário. Ele
                    // chega à parede sozinho nos tiques de assentamento.
                    Transform::from_translation(Vec2::new(-3.0, k as f32 * 0.5)),
                ))
                .id(),
        );
    }
    (sim, PhysicsBridge::new(), quem)
}

fn pos(sim: &SimWorld, nome: &str) -> Vec2 {
    let mut achado = None;
    let mut q = sim.world().try_query::<(&Name, &Transform)>().unwrap();
    for (n, t) in q.iter(sim.world()) {
        if n.as_str() == nome {
            achado = Some(t.translation);
        }
    }
    achado.expect("a cena tem de conter o corpo")
}

#[test]
#[ignore = "medicao: imprime a tabela do plano 10, nao afirma lei nenhuma"]
fn mede_o_deslize_na_porta_do_produto() {
    println!("\n## PONTE (o produto) — fraccao do orcamento por angulo de incidencia");
    println!("# V={V} m/s · dt={DT:.6} ⇒ orcamento {ORCAMENTO:.6} m/tique");
    println!("# ⚠️ a projeccao (a lei do platformer) daria `sin θ`; o orcamento da' `1,00`");
    println!("# ang  fraccao_do_orcamento  sin(ang)  razao_contra_a_projeccao");
    for passo in 0..=18 {
        let ang = passo as f32 * 5.0;
        let a = ang.to_radians();
        let (mut sim, mut bridge, quem) = cena(1);
        bridge.set_player_input(
            quem[0],
            PlayerInput {
                drive: a.cos(),
                drive_y: a.sin(),
                ..PlayerInput::default()
            },
        );
        // Assenta (ele viaja até à parede) e depois mede UM tique.
        for t in 1..=90u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        let antes = pos(&sim, "P0");
        bridge.dispatch(&mut sim, true, 91);
        let d = pos(&sim, "P0") - antes;
        let n = (d.x * d.x + d.y * d.y).sqrt() / ORCAMENTO;
        let s = a.sin().max(1.0e-6);
        println!("P {ang:.0} {n:.4} {s:.4} {:.4}", n / s);
    }
}

#[test]
#[ignore = "medicao: imprime a tabela do plano 10, nao afirma lei nenhuma"]
fn mede_o_custo_de_um_mover_por_tique() {
    println!("\n## PONTE — custo de N movers de vista de cima por tique");
    println!("# ⚠️ Todos ENCOSTADOS a' parede e a 45°, que e' o caso que gasta o laco inteiro");
    println!("# n  ms_por_tique  us_por_mover  % de um quadro de 60 fps");
    for n in [1usize, 10, 100, 1000] {
        let (mut sim, mut bridge, quem) = cena(n);
        for &q in &quem {
            bridge.set_player_input(
                q,
                PlayerInput {
                    drive: core::f32::consts::FRAC_1_SQRT_2,
                    drive_y: core::f32::consts::FRAC_1_SQRT_2,
                    ..PlayerInput::default()
                },
            );
        }
        for t in 1..=90u64 {
            bridge.dispatch(&mut sim, true, t);
        }
        // O MÍNIMO de 5, com a mediana ao lado: a carga de fundo desta máquina
        // nunca desce, e uma leitura única mede o escalonador.
        let mut amostras = Vec::new();
        for k in 0..5u64 {
            let t0 = std::time::Instant::now();
            bridge.dispatch(&mut sim, true, 91 + k);
            amostras.push(t0.elapsed().as_secs_f64() * 1e3);
        }
        amostras.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let ms = amostras[0];
        println!(
            "C {n} {ms:.4} {:.3} {:.3}",
            ms * 1000.0 / n as f64,
            ms / 16.67 * 100.0
        );
    }
    println!(
        "# load {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
}
