//! A coleta: uma cena ECS com Luz, sprite emissivo e oclusor → a `CenaDeLuz` esperada.
//!
//! O esperado é calculado AQUI a partir dos números de autoria (cantos do quad por
//! `rotação · (âncora + q∘tamanho) + translação`), não pela função que se testa.

use bevy_ecs::world::World;
use ph2d_ecs::{GlobalTransform, SpriteEmissive, StableId, Transform, Visibility};
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_ecs::{BloqueiaLuz, CenaDeLuz, Emissor, Luz, Oclusor, TipoDeLuz};
use ph2d_render::Sprite;

const TOL: f32 = 1e-5;

fn pose(x: f32, y: f32, rot: f32) -> (Transform, GlobalTransform) {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t.translation.y = y;
    t.rotation = rot;
    (t, GlobalTransform::from_transform(t))
}

/// Os cantos na ordem do quad do shader: (−½,−½) (½,−½) (½,½) (−½,½).
fn cantos(x: f32, y: f32, rot: f32, tamanho: [f32; 2], ancora: [f32; 2]) -> Vec<[f32; 2]> {
    let (s, c) = (f64::from(rot).sin(), f64::from(rot).cos());
    [[-0.5, -0.5], [0.5, -0.5], [0.5, 0.5], [-0.5, 0.5]]
        .iter()
        .map(|q: &[f64; 2]| {
            let lx = f64::from(ancora[0]) + q[0] * f64::from(tamanho[0]);
            let ly = f64::from(ancora[1]) + q[1] * f64::from(tamanho[1]);
            [
                (c * lx - s * ly + f64::from(x)) as f32,
                (s * lx + c * ly + f64::from(y)) as f32,
            ]
        })
        .collect()
}

fn perto(a: &[[f32; 2]], b: &[[f32; 2]]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .all(|(p, q)| (p[0] - q[0]).abs() < TOL && (p[1] - q[1]).abs() < TOL)
}

fn rgb_perto(a: [f32; 3], b: [f32; 3]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < TOL)
}

fn luz(cor: [f32; 3], intensidade: f32, tamanho_m: f32, tipo: TipoDeLuz) -> Luz {
    Luz {
        cor,
        intensidade,
        tamanho_m,
        alcance_m: f32::INFINITY,
        tipo,
    }
}

type Passo = Box<dyn Fn(&mut World)>;

/// A cena como uma lista de spawns; `invertida` corre-os de trás para a frente.
fn povoar(invertida: bool) -> World {
    let mut passos: Vec<Passo> = vec![
        // StableId 1 — sprite emissivo: radiância = tint·self_tint (rgb·a) × intensidade.
        Box::new(|w| {
            let mut spr = Sprite::atlas(0, [2.0, 1.0], [0.5, 1.0, 1.0, 1.0]);
            spr.anchor = [0.25, 0.0];
            spr.self_tint = [1.0, 0.5, 1.0, 0.5];
            let (t, g) = pose(-3.0, 1.0, 0.5);
            w.spawn((spr, SpriteEmissive(4.0), t, g, StableId(1)));
        }),
        // StableId 2 — oclusor com opacidade parcial.
        Box::new(|w| {
            let (t, g) = pose(0.5, -2.0, -0.3);
            let spr = Sprite::atlas(0, [1.0, 3.0], [1.0; 4]);
            w.spawn((spr, BloqueiaLuz { opacidade: 0.75 }, t, g, StableId(2)));
        }),
        // StableId 3 — a luz de disco: radiância = cor·I/(2·tamanho).
        Box::new(|w| {
            let (t, g) = pose(1.0, 2.0, 0.3);
            let l = luz([1.0, 0.5, 0.25], 2.0, 0.5, TipoDeLuz::Ponto);
            w.spawn((l, t, g, StableId(3)));
        }),
        // StableId 4 — ambiente: radiância do céu = cor·I/(2π).
        Box::new(|w| {
            let (t, g) = pose(9.0, 9.0, 0.0);
            let l = luz(
                [0.2, 0.2, 0.4],
                std::f32::consts::PI,
                0.0,
                TipoDeLuz::Ambiente,
            );
            w.spawn((l, t, g, StableId(4)));
        }),
        // StableId 5 — luz pontual exata (tamanho 0).
        Box::new(|w| {
            let (t, g) = pose(-1.0, -1.0, 0.0);
            let l = luz([0.0, 1.0, 0.0], 3.0, 0.0, TipoDeLuz::Ponto);
            w.spawn((l, t, g, StableId(5)));
        }),
        // Fora da cena: escondida.
        Box::new(|w| {
            let (t, g) = pose(5.0, 5.0, 0.0);
            let l = luz([1.0; 3], 1.0, 0.1, TipoDeLuz::Ponto);
            w.spawn((l, Visibility::hidden(), t, g, StableId(6)));
        }),
        // Fora da cena: emissivo 0 e sem bloqueio.
        Box::new(|w| {
            let (t, g) = pose(6.0, 6.0, 0.0);
            let spr = Sprite::atlas(0, [1.0, 1.0], [1.0; 4]);
            w.spawn((spr, SpriteEmissive(0.0), t, g, StableId(7)));
        }),
        // Fora da cena: intensidade NaN (projeto corrompido) apaga, nunca envenena.
        Box::new(|w| {
            let (t, g) = pose(7.0, 7.0, 0.0);
            let l = luz([1.0; 3], f32::NAN, 0.1, TipoDeLuz::Ponto);
            w.spawn((l, t, g, StableId(8)));
        }),
        // Emissivo que também bloqueia entra UMA vez, como emissor (opaco).
        Box::new(|w| {
            let (t, g) = pose(2.0, 3.0, 1.1);
            let spr = Sprite::atlas(0, [0.5, 0.5], [1.0, 0.0, 0.0, 1.0]);
            let b = BloqueiaLuz { opacidade: 1.0 };
            w.spawn((spr, SpriteEmissive(2.0), b, t, g, StableId(9)));
        }),
    ];
    if invertida {
        passos.reverse();
    }
    let mut w = World::new();
    for p in &passos {
        p(&mut w);
    }
    w
}

fn esperado() -> CenaDeLuz {
    CenaDeLuz {
        emissores: vec![
            Emissor::Poligono {
                vertices: cantos(-3.0, 1.0, 0.5, [2.0, 1.0], [0.25, 0.0]),
                radiancia: [0.5 * 0.5 * 4.0, 0.5 * 0.5 * 4.0, 1.0 * 0.5 * 4.0],
                alcance_m: f32::INFINITY,
            },
            Emissor::Disco {
                centro: [1.0, 2.0],
                raio: 0.5,
                radiancia: [2.0, 1.0, 0.5],
                alcance_m: f32::INFINITY,
            },
            Emissor::Ponto {
                centro: [-1.0, -1.0],
                intensidade: [0.0, 3.0, 0.0],
                alcance_m: f32::INFINITY,
            },
            Emissor::Poligono {
                vertices: cantos(2.0, 3.0, 1.1, [0.5, 0.5], [0.0, 0.0]),
                radiancia: [2.0, 0.0, 0.0],
                alcance_m: f32::INFINITY,
            },
        ],
        oclusores: vec![Oclusor {
            vertices: cantos(0.5, -2.0, -0.3, [1.0, 3.0], [0.0, 0.0]),
            opacidade: 0.75,
        }],
        ceu: [0.1, 0.1, 0.2],
    }
}

fn emissor_perto(a: &Emissor, b: &Emissor) -> bool {
    match (a, b) {
        (
            Emissor::Disco {
                centro: c1,
                raio: r1,
                radiancia: l1,
                alcance_m: a1,
            },
            Emissor::Disco {
                centro: c2,
                raio: r2,
                radiancia: l2,
                alcance_m: a2,
            },
        ) => perto(&[*c1], &[*c2]) && (r1 - r2).abs() < TOL && rgb_perto(*l1, *l2) && a1 == a2,
        (
            Emissor::Ponto {
                centro: c1,
                intensidade: i1,
                alcance_m: a1,
            },
            Emissor::Ponto {
                centro: c2,
                intensidade: i2,
                alcance_m: a2,
            },
        ) => perto(&[*c1], &[*c2]) && rgb_perto(*i1, *i2) && a1 == a2,
        (
            Emissor::Poligono {
                vertices: v1,
                radiancia: l1,
                alcance_m: a1,
            },
            Emissor::Poligono {
                vertices: v2,
                radiancia: l2,
                alcance_m: a2,
            },
        ) => perto(v1, v2) && rgb_perto(*l1, *l2) && a1 == a2,
        _ => false,
    }
}

#[test]
fn a_cena_ecs_vira_a_cena_de_luz_esperada() {
    let obtido = cena_de_luz(&povoar(false));
    let esperado = esperado();

    assert_eq!(
        obtido.emissores.len(),
        esperado.emissores.len(),
        "emissores: {obtido:#?}"
    );
    for (i, (o, e)) in obtido.emissores.iter().zip(&esperado.emissores).enumerate() {
        assert!(
            emissor_perto(o, e),
            "emissor {i}:\n obtido {o:?}\n esperado {e:?}"
        );
    }
    assert_eq!(
        obtido.oclusores.len(),
        1,
        "oclusores: {:#?}",
        obtido.oclusores
    );
    assert!(perto(
        &obtido.oclusores[0].vertices,
        &esperado.oclusores[0].vertices
    ));
    assert!((obtido.oclusores[0].opacidade - 0.75).abs() < TOL);
    assert!(
        rgb_perto(obtido.ceu, esperado.ceu),
        "céu {:?} ≠ {:?}",
        obtido.ceu,
        esperado.ceu
    );
}

/// A ordem é a do `StableId`, nunca a do arquétipo nem a do spawn: ao contrário dá a MESMA cena.
#[test]
fn a_ordem_da_cena_nao_depende_da_ordem_de_spawn() {
    let a = cena_de_luz(&povoar(false));
    assert!(
        !a.emissores.is_empty(),
        "controlo: a cena tem de ter emissores"
    );
    assert_eq!(cena_de_luz(&povoar(true)), a);
}

#[test]
fn um_mundo_sem_luz_da_a_cena_vazia() {
    let mut w = World::new();
    let (t, g) = pose(0.0, 0.0, 0.0);
    w.spawn((Sprite::atlas(0, [1.0, 1.0], [1.0; 4]), t, g));
    assert_eq!(cena_de_luz(&w), CenaDeLuz::default());
}
