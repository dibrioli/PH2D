//! Os gates do rapier privado com VÁRIOS corpos (`docs/Macros/17_W19_plano.md` §3): caixas empilhadas
//! sobre um chão Static fora do objecto Macro. Os ids locais vão AO CONTRÁRIO da ordem do `Entity` e o
//! chão nasce por último — a ordem de nascimento tem de ser a da ponte, não a da trilha.

use super::*;
use crate::mundo::formato::{Componente, Inicial, Linha, Objeto, Simula};
use crate::mundo::palco_fisica::{DT, RETOMA};
use ph2d_core::Vec2;
use ph2d_ecs::scene::register_ecs_components;
use ph2d_ecs::{ChildOf, Transform};
use ph2d_macro::{Event, MacroDoc, RefRect, Track};
use ph2d_physics_ecs::{Collider, ColliderShape, InputTape, PhysicsBridge};

const DONO: u64 = 10;
const CHAO: u64 = 999;
const T_CORPO: u64 = 1_000_000;
const T_SIMULA: u64 = 2_000_000;

fn b<T: serde::Serialize>(v: &T) -> Vec<u8> {
    postcard::to_allocvec(v).expect("postcard")
}

fn caixa() -> Collider {
    Collider {
        shape: ColliderShape::Cuboid {
            half_x: 0.2,
            half_y: 0.2,
        },
        ..Collider::default()
    }
}

fn pose(i: usize) -> Transform {
    // Um nada de desvio em x: a pilha inclina e cada contacto conta.
    Transform::from_translation(Vec2::new(i as f32 * 0.03, 0.3 + i as f32 * 0.45))
}

/// O id local do corpo `i`: ao contrário da ordem em que nasce.
fn local(n: usize, i: usize) -> u32 {
    u32::try_from(n - i).expect("cabe")
}

fn registo() -> ComponentRegistry {
    let mut r = ComponentRegistry::new();
    register_ecs_components(&mut r);
    ph2d_physics_ecs::register_physics_components(&mut r);
    r
}

/// O mundo do FIM: o objecto Macro, os `n` corpos (por ordem de `Entity`), o chão por último.
fn mundo(n: usize, doc: Option<Vec<u8>>) -> (SimWorld, Entity, Vec<Entity>) {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let dono = w.spawn((StableId(DONO), Transform::default())).id();
    if let Some(doc) = doc {
        w.entity_mut(dono).insert(Macro {
            doc,
            ..Macro::default()
        });
    }
    let corpos = (0..n)
        .map(|i| {
            let corpo = RigidBody {
                kind: BodyKind::Dynamic,
            };
            let s = StableId(100 + i as u64);
            w.spawn((s, pose(i), corpo, caixa(), ChildOf(dono))).id()
        })
        .collect();
    w.spawn((
        StableId(CHAO),
        RigidBody {
            kind: BodyKind::Static,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: 20.0,
                half_y: 0.05,
            },
            ..Collider::default()
        },
        Transform::default(),
    ));
    (sim, dono, corpos)
}

fn bits_de(sim: &SimWorld, corpos: &[Entity]) -> Vec<[u32; 3]> {
    corpos
        .iter()
        .map(|&e| {
            let t = sim.world().get::<Transform>(e).expect("pose");
            [t.translation.x, t.translation.y, t.rotation].map(f32::to_bits)
        })
        .collect()
}

/// A corrida da gravação: a ponte global sobre o FIM, um tique por quadro.
fn gravada(n: usize, ate: u32) -> Vec<[u32; 3]> {
    let (mut sim, _, corpos) = mundo(n, None);
    let mut ponte = PhysicsBridge::new();
    for k in 1..=u64::from(ate) {
        ponte.dispatch(&mut sim, true, k);
    }
    bits_de(&sim, &corpos)
}

fn doc(n: usize, ate: u32, conferencia: &[[u32; 3]]) -> Vec<u8> {
    let mut objetos = vec![Objeto {
        id: 0,
        original: DONO,
        quem: Quem::Macro,
    }];
    let mut linhas = vec![Linha {
        id: 0,
        pai: None,
        componentes: vec![],
    }];
    let mut mudancas = Vec::new();
    for i in 0..n {
        let id = local(n, i);
        objetos.push(Objeto {
            id,
            original: 100 + i as u64,
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
        let corpo = RigidBody {
            kind: BodyKind::Dynamic,
        };
        mudancas.push(Mudanca::Muda {
            id,
            pai: None,
            componentes: vec![
                (CORPO.into(), Some(b(&corpo))),
                ("ph2d::physics::Collider".into(), Some(b(&caixa()))),
            ],
        });
    }
    let simula = Evento::Simula(Simula {
        de: 0,
        ate,
        fita: InputTape::new().to_wire(),
        conferencia: conferencia
            .iter()
            .enumerate()
            .map(|(i, p)| (local(n, i), p.map(f32::from_bits)))
            .collect(),
        obstaculos: {
            let (s, dono, _) = mundo(n, None);
            super::super::palco_fisica::impressao(s.world(), dono)
        },
    });
    let passo = Evento::Passo {
        novos: vec![],
        mudancas,
        recursos: vec![],
    };
    let ev = |t: u64, e: &Evento| Event {
        t: MacroTime(t),
        payload: e.escreve(),
    };
    ph2d_macro::codec::write(&MacroDoc {
        reference: RefRect { w: 1.0, h: 1.0 },
        tracks: vec![Track {
            family: super::super::formato::FAMILIA.into(),
            target: DONO,
            initial: Inicial {
                objetos,
                linhas,
                recursos: vec![],
            }
            .escreve(),
            events: vec![ev(T_CORPO, &passo), ev(T_SIMULA, &simula)],
            edicoes: vec![],
            nasce: None,
            continua: None,
        }],
    })
}

struct Pilha {
    sim: SimWorld,
    reg: ComponentRegistry,
    dono: Entity,
    corpos: Vec<Entity>,
    drive: PreviewDrive,
    palcos: PalcosDoMundo,
}

impl Pilha {
    fn nova(n: usize, ate: u32, conferencia: &[[u32; 3]]) -> Self {
        let (sim, dono, corpos) = mundo(n, Some(doc(n, ate, conferencia)));
        Self {
            sim,
            reg: registo(),
            dono,
            corpos,
            drive: PreviewDrive::default(),
            palcos: PalcosDoMundo::default(),
        }
    }

    fn no_tique(&mut self, k: u64) -> Vec<[u32; 3]> {
        let us = T_SIMULA + (k as f64 * DT * 1e6).round() as u64;
        self.sim
            .world_mut()
            .entity_mut(self.dono)
            .insert(MacroTempo {
                segundos: us as f64 / 1e6,
            });
        let cx = Contexto::novo(&self.reg, PhysicsSettings::default());
        self.palcos.segue(&mut self.sim, &cx, &mut self.drive);
        self.drive.settle();
        bits_de(&self.sim, &self.corpos)
    }

    fn recozido(&self) -> &super::super::palco_fisica::Recozido {
        self.palcos.recozido(self.dono, 0).expect("palco")
    }
}

#[test]
fn tres_corpos_empilhados_nascem_pela_ordem_da_ponte() {
    const ATE: u32 = 240;
    let viu = gravada(3, ATE);
    assert_ne!(
        viu,
        bits_de(&mundo(3, None).0, &mundo(3, None).2),
        "controlo: a pilha mexeu"
    );
    let mut p = Pilha::nova(3, ATE, &viu);
    assert_eq!(
        p.no_tique(u64::from(ATE)),
        viu,
        "o recozido é o bit da corrida gravada"
    );
    assert_eq!(
        p.palcos.erro_de(p.dono.to_bits()),
        None,
        "e a conferência confere"
    );
}

#[test]
fn retomar_de_um_ponto_da_o_mesmo_bit_que_do_zero() {
    const N: usize = 8;
    const ATE: u32 = 600;
    let mut p = Pilha::nova(N, ATE, &[]);
    let fim = p.no_tique(u64::from(ATE));
    // Só saltos para trás (para a frente anda os tiques que deve).
    for k in [537u64, 459, 300, 299, 121, 120, 61, 59, 1, 0] {
        let antes = p.recozido().passos();
        let retomado = p.no_tique(k);
        let replayados = p.recozido().passos() - antes;
        assert!(
            replayados < RETOMA,
            "o tique {k} replaya {replayados} passos (< {RETOMA}: veio de um ponto)"
        );
        let do_zero = Pilha::nova(N, ATE, &[]).no_tique(k);
        assert_eq!(
            retomado, do_zero,
            "o tique {k} retomado é o bit do recozido do zero"
        );
    }
    assert_eq!(
        p.no_tique(u64::from(ATE)),
        fim,
        "e para a frente outra vez, o mesmo fim"
    );
    assert_eq!(p.recozido().do_zero, 1, "nunca se montou de novo");
}
