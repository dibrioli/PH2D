//! ⭐⭐⭐ **A PERGUNTA DE ANTES DE QUALQUER LINHA do TOP-20 #14 (`ProjectileMotion`)**:
//! *um corpo DINÂMICO com restituição já é um projéctil?*
//!
//! `CLAUDE.md` §5.0: **antes de construir um item de lista aberta, MEÇA se a composição já o
//! exprime** — *«o que se perde ao não reconferir não é tempo, é construir o que já existe»*.
//!
//! ⚠️ **Sonda, não gate.** Ela corre com `--ignored` e imprime; o que ela decide é se o componente
//! tem razão de existir, e o quê dele.
//!
//! ```text
//! cargo test -p ph2d-physics-ecs --test it mede_o_que_a_composicao -- --ignored --nocapture
//! ```
//!
//! # As perguntas, uma por bloco
//!
//! A) **RICOCHETE** — com `restitution = 1`, a rapidez sobrevive ao toque? Em que ângulos?
//! B) **A MASSA CONTA?** — o mesmo tiro contra uma caixa leve: o projéctil desvia-se?
//! C) **O que a casa NÃO tem**, contado por `grep` e não de cabeça.

use ph2d_core::Vec2;
use ph2d_ecs::{Entity, Name, SimWorld, Transform};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, GravityScale, InitialVelocity, PhysicsBridge, RigidBody,
};

const DT: f32 = 1.0 / 60.0;

fn parede(sim: &mut SimWorld, nome: &str, em: Vec2, meio: (f32, f32)) {
    sim.world_mut().spawn((
        Name::new(nome),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: meio.0,
                half_y: meio.1,
            },
            // ⚠️ **A restituição COMBINA-SE entre os dois lados** — pôr `1` só na bala mediria
            // outra coisa (a regra de combinação, que nesta casa é um knob à parte).
            restitution: 1.0,
            friction: 0.0,
            ..Collider::default()
        },
        Transform::from_translation(em),
    ));
}

/// Uma «bala» feita **só com o que a casa já tem**: corpo dinâmico, sem gravidade, a saltar.
fn bala(sim: &mut SimWorld, em: Vec2, v: Vec2) -> Entity {
    sim.world_mut()
        .spawn((
            Name::new("Bala"),
            RigidBody {
                kind: BodyKind::Dynamic,
            },
            Collider {
                shape: ColliderShape::Ball { radius: 0.1 },
                restitution: 1.0,
                friction: 0.0,
                ..Collider::default()
            },
            // Sem arco: o que se mede aqui é o toque, não a queda.
            GravityScale(0.0),
            InitialVelocity {
                linvel: [v.x, v.y],
                angvel: 0.0,
            },
            Transform::from_translation(em),
        ))
        .id()
}

fn pose(sim: &SimWorld, quem: Entity) -> Vec2 {
    sim.world()
        .get::<Transform>(quem)
        .map_or(Vec2::new(f32::NAN, f32::NAN), |t| t.translation)
}

/// Corre `tiques` e devolve as poses, para daí sair a velocidade por diferença.
fn corre(sim: &mut SimWorld, bridge: &mut PhysicsBridge, quem: Entity, tiques: u64) -> Vec<Vec2> {
    let mut saida = Vec::new();
    for t in 1..=tiques {
        bridge.dispatch(sim, true, t);
        saida.push(pose(sim, quem));
    }
    saida
}

/// A rapidez entre duas poses consecutivas, em m/s.
fn rapidez(a: Vec2, b: Vec2) -> f32 {
    ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt() / DT
}

#[test]
#[ignore = "sonda de medição — corre com --ignored --nocapture"]
fn mede_o_que_a_composicao_ja_da() {
    println!("\n=== A) O RICOCHETE DE UM CORPO DINÂMICO (restitution = 1 dos dois lados) ===");
    println!("  angulo |  antes  |  depois |  razao | desvio do espelho");
    for graus in [90.0_f32, 75.0, 60.0, 45.0, 30.0, 15.0] {
        let mut sim = SimWorld::new();
        // Parede vertical com a face em x = 0; a bala vem da esquerda.
        parede(&mut sim, "Parede", Vec2::new(1.0, 0.0), (1.0, 20.0));
        let r = graus.to_radians();
        let v = Vec2::new(12.0 * r.sin(), -12.0 * r.cos());
        let quem = bala(&mut sim, Vec2::new(-1.5, 1.0), v);
        let mut bridge = PhysicsBridge::new();
        let poses = corre(&mut sim, &mut bridge, quem, 90);

        // A rapidez ANTES: o 5.º intervalo, já assente e ainda longe da parede.
        let antes = rapidez(poses[4], poses[5]);
        // O toque: o primeiro intervalo em que a componente x do movimento inverte o sinal.
        let mut toque = None;
        for i in 1..poses.len() - 1 {
            let dx0 = poses[i].x - poses[i - 1].x;
            let dx1 = poses[i + 1].x - poses[i].x;
            if dx0 > 0.0 && dx1 < 0.0 {
                toque = Some(i);
                break;
            }
        }
        let Some(i) = toque else {
            println!("  {graus:5.0}° | {antes:7.3} |    (nao tocou a parede)");
            continue;
        };
        // A rapidez DEPOIS: cinco tiques adiante do toque, longe do contacto.
        let j = (i + 6).min(poses.len() - 2);
        let depois = rapidez(poses[j], poses[j + 1]);
        // O espelho: a lei de arcade que o componente vai escrever.
        let esperado_x = -v.x;
        let real_x = (poses[j + 1].x - poses[j].x) / DT;
        println!(
            "  {graus:5.0}° | {antes:7.3} | {depois:7.3} | {:6.3} | vx {real_x:7.3} contra {esperado_x:7.3} do espelho",
            depois / antes
        );
    }

    println!("\n=== B) A MASSA CONTA? O MESMO tiro contra uma caixa LEVE ===");
    for (nome, densidade) in [
        ("parede estatica", None),
        ("caixa dinamica leve", Some(0.05)),
    ] {
        let mut sim = SimWorld::new();
        match densidade {
            None => parede(&mut sim, "Parede", Vec2::new(1.0, 0.0), (1.0, 20.0)),
            Some(d) => {
                sim.world_mut().spawn((
                    Name::new("Caixa"),
                    RigidBody {
                        kind: BodyKind::Dynamic,
                    },
                    Collider {
                        shape: ColliderShape::Cuboid {
                            half_x: 1.0,
                            half_y: 2.0,
                        },
                        density: d,
                        restitution: 1.0,
                        friction: 0.0,
                        ..Collider::default()
                    },
                    GravityScale(0.0),
                    Transform::from_translation(Vec2::new(1.0, 0.0)),
                ));
            }
        }
        let quem = bala(&mut sim, Vec2::new(-1.5, 0.0), Vec2::new(12.0, 0.0));
        let mut bridge = PhysicsBridge::new();
        let poses = corre(&mut sim, &mut bridge, quem, 90);
        let fim = poses[poses.len() - 1];
        let ultima = rapidez(poses[poses.len() - 2], fim);
        println!(
            "  {nome:22} ⇒ x final {:7.3} · rapidez final {ultima:7.3}",
            fim.x
        );
    }

    println!("\n=== C) O QUE A CASA NAO TEM (contado, nao lembrado) ===");
    println!(
        "  alcance percorrido · homing · «a flecha aponta para onde voa» · tecto de ricochetes"
    );
    println!("  ⇒ nenhum destes e' um campo de componente nenhum hoje; ver o §1 do plano.");
}
