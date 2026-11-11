//! Regra B na reprodução do mundo (GEF02 5.5): uma forma com trilha de pontos (FISICA 51/52) tem UM escritor
//! do contorno — a animação; a macro que toca não o troca (nem o número de pontos, nem a posição).

use super::*;
use crate::mundo::formato::{Componente, Inicial, Linha, Objeto};
use ph2d_ecs::scene::register_ecs_components;
use ph2d_macro::{Event, MacroDoc, RefRect, Track};
use ph2d_physics_ecs::{Collider, ColliderOutline, ColliderShape};

const DONO: u64 = 10;
const FORMA: u64 = 11;
const OUTLINE: &str = "ph2d::physics::ColliderOutline";

fn contorno(n: i32) -> ColliderOutline {
    let anel: Vec<[i32; 2]> = (0..n)
        .map(|k| {
            let a = f64::from(k) * std::f64::consts::TAU / f64::from(n);
            [(a.cos() * 160.0) as i32, (a.sin() * 160.0) as i32]
        })
        .collect();
    ColliderOutline {
        rings: std::sync::Arc::from(vec![anel]),
        custom: true,
        ..ColliderOutline::default()
    }
}

fn b<T: serde::Serialize>(v: &T) -> Vec<u8> {
    postcard::to_allocvec(v).expect("postcard")
}

/// A trilha: a forma nasce com 4 pontos e, a 1 s, passa a 6.
fn doc() -> Vec<u8> {
    let inicial = Inicial {
        objetos: vec![
            Objeto {
                id: 0,
                original: DONO,
                quem: Quem::Macro,
            },
            Objeto {
                id: 1,
                original: FORMA,
                quem: Quem::Mundo,
            },
        ],
        linhas: vec![
            Linha {
                id: 0,
                pai: None,
                componentes: vec![],
            },
            Linha {
                id: 1,
                pai: None,
                componentes: vec![Componente {
                    nome: OUTLINE.into(),
                    bytes: b(&contorno(4)),
                }],
            },
        ],
        recursos: vec![],
    };
    let seis = Evento::Passo {
        novos: vec![],
        mudancas: vec![Mudanca::Muda {
            id: 1,
            pai: None,
            componentes: vec![(OUTLINE.into(), Some(b(&contorno(6))))],
        }],
        recursos: vec![],
    };
    ph2d_macro::codec::write(&MacroDoc {
        reference: RefRect { w: 1.0, h: 1.0 },
        tracks: vec![Track {
            family: super::super::formato::FAMILIA.into(),
            target: DONO,
            initial: inicial.escreve(),
            events: vec![Event {
                t: MacroTime(1_000_000),
                payload: seis.escreve(),
            }],
            edicoes: vec![],
            nasce: None,
            continua: None,
        }],
    })
}

/// Toca a 1,5 s; `animada` = a forma tem trilha de pontos. Devolve o nº de pontos da forma.
fn pontos_depois_de_tocar(animada: bool, auto_key: bool) -> usize {
    let mut sim = SimWorld::new();
    let w = sim.world_mut();
    let dono = w
        .spawn((
            StableId(DONO),
            ph2d_ecs::Transform::default(),
            Macro {
                doc: doc(),
                ..Macro::default()
            },
            MacroTempo { segundos: 1.5 },
        ))
        .id();
    let forma = w
        .spawn((
            StableId(FORMA),
            ph2d_ecs::Transform::default(),
            Collider {
                shape: ColliderShape::Outline {
                    half_x: 1.0,
                    half_y: 1.0,
                },
                ..Collider::default()
            },
            contorno(4),
            ph2d_ecs::ChildOf(dono),
        ))
        .id();
    let formas = if animada {
        std::iter::once(forma.to_bits()).collect()
    } else {
        Default::default()
    };
    ph2d_shape_handles::travados::publica(w, formas, auto_key);
    let mut reg = ComponentRegistry::new();
    register_ecs_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    let mut drive = PreviewDrive::default();
    let cx = Contexto::novo(&reg, PhysicsSettings::default());
    PalcosDoMundo::default().segue(&mut sim, &cx, &mut drive);
    drive.settle();
    sim.world()
        .get::<ColliderOutline>(forma)
        .map_or(0, |o| o.rings.iter().map(Vec::len).sum())
}

#[test]
fn a_macro_que_toca_nao_troca_o_contorno_de_uma_forma_com_pontos_animados() {
    assert_eq!(
        pontos_depois_de_tocar(false, false),
        6,
        "controlo: sem trilha de pontos a macro conduz o contorno"
    );
    for auto_key in [false, true] {
        assert_eq!(
            pontos_depois_de_tocar(true, auto_key),
            4,
            "com trilha de pontos (Auto Key {auto_key}) a animação é o único escritor do contorno"
        );
    }
}
