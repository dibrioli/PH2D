//! **A sonda da §3 do plano** (`docs/Macros/17_W19_plano.md`): o custo por quadro de conduzir `S(t)` e de
//! recozer um `Simula` — variantes no MESMO processo, intercaladas em blocos rodados, resumo = o MÍNIMO.
//! `cargo test --profile smoke -p ph2d-app-macros --lib mede_o_palco_do_mundo -- --ignored --nocapture`

use super::*;
use crate::mundo::formato::{Componente, Inicial, Linha, Objeto, Simula};
use crate::mundo::palco_fisica::DT;
use ph2d_core::Vec2;
use ph2d_ecs::scene::register_ecs_components;
use ph2d_ecs::{ChildOf, Transform};
use ph2d_macro::{Event, MacroDoc, RefRect, Track};
use ph2d_physics_ecs::{Collider, ColliderShape, InputTape};
use std::time::Instant;

const T_CORPO: u64 = 1_000_000;
const T_SIMULA: u64 = 2_000_000;
const ATE: u32 = 3_600;

fn b<T: serde::Serialize>(v: &T) -> Vec<u8> {
    postcard::to_allocvec(v).expect("postcard")
}

fn pose(i: usize) -> Transform {
    Transform::from_translation(Vec2::new(i as f32 * 0.6 - 30.0, 5.0 + (i % 7) as f32 * 0.6))
}

fn doc(n: usize) -> Vec<u8> {
    let mut objetos = vec![Objeto {
        id: 0,
        original: 1_000,
        quem: Quem::Macro,
    }];
    let mut linhas = vec![Linha {
        id: 0,
        pai: None,
        componentes: vec![],
    }];
    let mut mudancas = Vec::new();
    for i in 0..n {
        let id = u32::try_from(i + 1).expect("cabe");
        objetos.push(Objeto {
            id,
            original: 1_001 + i as u64,
            quem: Quem::Mundo,
        });
        linhas.push(Linha {
            id,
            pai: None,
            componentes: vec![Componente {
                nome: TRANSFORM.into(),
                bytes: b(&pose(i)),
            }],
        });
        mudancas.push(Mudanca::Muda {
            id,
            pai: None,
            componentes: vec![
                (
                    CORPO.into(),
                    Some(b(&RigidBody {
                        kind: BodyKind::Dynamic,
                    })),
                ),
                ("ph2d::physics::Collider".into(), Some(b(&bola()))),
            ],
        });
    }
    let inicial = Inicial {
        objetos,
        linhas,
        recursos: vec![],
    };
    let corpo = Evento::Passo {
        novos: vec![],
        mudancas,
        recursos: vec![],
    };
    let simula = Evento::Simula(Simula {
        de: 0,
        ate: ATE,
        fita: InputTape::new().to_wire(),
        conferencia: vec![],
    });
    ph2d_macro::codec::write(&MacroDoc {
        reference: RefRect { w: 1.0, h: 1.0 },
        tracks: vec![Track {
            family: super::super::formato::FAMILIA.into(),
            target: 1_000,
            initial: inicial.escreve(),
            events: vec![
                Event {
                    t: MacroTime(T_CORPO),
                    payload: corpo.escreve(),
                },
                Event {
                    t: MacroTime(T_SIMULA),
                    payload: simula.escreve(),
                },
            ],
            edicoes: vec![],
            nasce: None,
            continua: None,
        }],
    })
}

fn bola() -> Collider {
    Collider {
        shape: ColliderShape::Ball { radius: 0.25 },
        ..Collider::default()
    }
}

struct Cena {
    sim: SimWorld,
    reg: ComponentRegistry,
    dono: Entity,
    drive: PreviewDrive,
    palcos: PalcosDoMundo,
}

impl Cena {
    fn nova(n: usize) -> Self {
        let mut reg = ComponentRegistry::new();
        register_ecs_components(&mut reg);
        ph2d_physics_ecs::register_physics_components(&mut reg);
        let mut sim = SimWorld::new();
        let w = sim.world_mut();
        w.spawn((
            StableId(1),
            RigidBody {
                kind: BodyKind::Static,
            },
            Collider {
                shape: ColliderShape::Cuboid {
                    half_x: 80.0,
                    half_y: 0.1,
                },
                ..Collider::default()
            },
            Transform::default(),
        ));
        let dono = w
            .spawn((
                StableId(1_000),
                Transform::default(),
                Macro {
                    doc: doc(n),
                    ..Macro::default()
                },
            ))
            .id();
        for i in 0..n {
            w.spawn((
                StableId(1_001 + i as u64),
                pose(i),
                RigidBody {
                    kind: BodyKind::Dynamic,
                },
                bola(),
                ChildOf(dono),
            ));
        }
        Self {
            sim,
            reg,
            dono,
            drive: PreviewDrive::default(),
            palcos: PalcosDoMundo::default(),
        }
    }

    /// Um quadro do palco em `us`, com a captura (`substitute` + `restore`) do `post_frame_undo`.
    fn quadro(&mut self, us: u64) {
        self.sim
            .world_mut()
            .entity_mut(self.dono)
            .insert(MacroTempo {
                segundos: us as f64 / 1e6,
            });
        let cx = Contexto {
            registo: &self.reg,
            definicoes: PhysicsSettings::default(),
        };
        self.palcos.segue(&mut self.sim, &cx, &mut self.drive);
        let vivo = self.drive.substitute(&mut self.sim);
        PreviewDrive::restore(&mut self.sim, &vivo);
        self.drive.settle();
    }
}

fn no_tique(k: u32) -> u64 {
    T_SIMULA + (f64::from(k) * DT * 1e6).round() as u64
}

fn ms(f: impl FnOnce()) -> f64 {
    let t = Instant::now();
    f();
    t.elapsed().as_secs_f64() * 1e3
}

#[derive(Clone, Copy, Debug)]
enum Variante {
    /// Conduzir `S(t)` parado num instante (corpos cinemáticos, sem física).
    Conduz(usize),
    /// Conduzir com `t` a saltar entre antes e depois do passo do corpo (põe e tira a cada quadro).
    Troca(usize),
    /// Recozer o `Simula` do zero até `k` tiques.
    DoZero(usize, u32),
    /// Com a ponte guardada no tique 600: um quadro que anda 1 tique.
    Guardada(usize),
}

fn corre(v: Variante) -> f64 {
    match v {
        Variante::Conduz(n) => {
            let mut c = Cena::nova(n);
            c.quadro(1_500_000);
            (0..20)
                .map(|_| ms(|| c.quadro(1_500_000)))
                .fold(f64::MAX, f64::min)
        }
        Variante::Troca(n) => {
            let mut c = Cena::nova(n);
            c.quadro(500_000);
            (0..20)
                .map(|i| ms(|| c.quadro(if i % 2 == 0 { 1_500_000 } else { 500_000 })))
                .fold(f64::MAX, f64::min)
        }
        Variante::DoZero(n, k) => {
            let mut c = Cena::nova(n);
            c.quadro(1_500_000);
            ms(|| c.quadro(no_tique(k)))
        }
        Variante::Guardada(n) => {
            let mut c = Cena::nova(n);
            c.quadro(no_tique(600));
            (601..621)
                .map(|k| ms(|| c.quadro(no_tique(k))))
                .fold(f64::MAX, f64::min)
        }
    }
}

#[test]
#[ignore = "sonda de velocidade (--profile smoke)"]
fn mede_o_palco_do_mundo() {
    let mut vs = Vec::new();
    for n in [1, 10, 100] {
        vs.push(Variante::Conduz(n));
        vs.push(Variante::Troca(n));
        vs.push(Variante::Guardada(n));
        for k in [60, 600, ATE] {
            vs.push(Variante::DoZero(n, k));
        }
    }
    let mut minimo = vec![f64::MAX; vs.len()];
    for bloco in 0..5 {
        for j in 0..vs.len() {
            let i = (j + bloco * 3) % vs.len();
            minimo[i] = minimo[i].min(corre(vs[i]));
        }
    }
    for (v, m) in vs.iter().zip(&minimo) {
        println!("{v:?}\t{m:.3} ms");
    }
}
