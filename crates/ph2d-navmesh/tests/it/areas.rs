//! ⭐⭐ **A CONSTRUÇÃO COM ÁREAS DE CUSTO** (W7, plano 30 §2.5): a fronteira de cada área vira aresta
//! da malha, cada polígono sabe a área onde está, e sem áreas nada muda.
//!
//! ⚠️ A régua da etiqueta não usa a construção: cada ponto amostrado é classificado pelo anel RECUADO
//! de cada área (o mesmo `inflate` que a construção usa, em aritmética inteira exacta), com a regra da
//! prioridade (manda a 1.ª da lista) — e só se afastam os pontos a menos de duas unidades da grelha de
//! uma fronteira, onde o arredondamento do Clipper decide.

use ph2d_nav::NavMesh;
use ph2d_navmesh::lattice::{P, orient, to_lattice};
use ph2d_navmesh::{Area, DISK_SIDES, Params, Shape, build, build_with_areas, inflate};

use crate::cena::{Lcg, caixa_rodada, obstaculos, retangulo};

/// `n` áreas misturadas (caixas rodadas e círculos), ids `1..=n`, algumas a sair da região.
pub fn areas(rng: &mut Lcg, n: usize, w: f64, h: f64) -> Vec<Area> {
    (0..n)
        .map(|i| {
            let c = [rng.range(-1.0, w + 1.0), rng.range(-1.0, h + 1.0)];
            let shape = if rng.next_u32().is_multiple_of(2) {
                let (a, b) = (rng.range(-1.0, 1.0), rng.range(-1.0, 1.0));
                let l = (a * a + b * b).sqrt().max(1e-6);
                Shape::Convex(caixa_rodada(
                    c,
                    [rng.range(0.5, 4.0), rng.range(0.5, 4.0)],
                    [a / l, b / l],
                ))
            } else {
                Shape::Circle {
                    center: c,
                    radius: rng.range(0.5, 3.5),
                }
            };
            Area {
                shape,
                id: (i + 1) as u16,
            }
        })
        .collect()
}

/// A impressão de uma malha: os bits dos vértices, os anéis e as áreas.
fn impressao(m: &NavMesh) -> Vec<u64> {
    let mut v: Vec<u64> = m
        .verts()
        .iter()
        .flat_map(|p| [p[0].to_bits(), p[1].to_bits()])
        .collect();
    for (i, p) in m.polys().enumerate() {
        v.push(u64::from(m.area_id(i as u32)) << 32 | p.verts.len() as u64);
        v.extend(p.verts.iter().map(|&x| u64::from(x)));
    }
    v
}

#[test]
fn sem_areas_a_construcao_e_a_de_sempre_ao_bit() {
    for seed in 1..=12u64 {
        let mut rng = Lcg(seed);
        let obs = obstaculos(&mut rng, 6 + seed as usize, 20.0, 14.0);
        let params = Params {
            agent_radius: [0.0, 0.35, 0.6][(seed % 3) as usize],
            merge: seed % 4 != 0,
            ..Params::default()
        };
        let a = build(&retangulo(20.0, 14.0), &obs, &params).expect("constrói");
        let b = build_with_areas(&retangulo(20.0, 14.0), &obs, &[], &params).expect("constrói");
        assert_eq!(impressao(&a.mesh), impressao(&b.mesh), "semente {seed}");
        assert!(!b.mesh.has_areas());
    }
}

/// Onde está `p`, contra o anel `ring` (convexo, anti-horário): `Some(true)` dentro, `Some(false)`
/// fora, `None` a menos de duas unidades da fronteira (indeciso).
fn dentro(ring: &[P], p: P) -> Option<bool> {
    let n = ring.len();
    let mut fora = false;
    for i in 0..n {
        let (a, b) = (ring[i], ring[(i + 1) % n]);
        let o = orient(a, b, p);
        let l2 = (i128::from(b.0 - a.0)).pow(2) + (i128::from(b.1 - a.1)).pow(2);
        if o * o < 4 * l2 {
            // A menos de 2 unidades da RECTA desta aresta: perto da fronteira só se a projecção cai
            // no segmento; conservador — indeciso.
            return None;
        }
        if o < 0 {
            fora = true;
        }
    }
    Some(!fora)
}

#[test]
fn cada_poligono_sabe_a_area_onde_esta_com_a_primeira_a_mandar() {
    let mut amostras = 0usize;
    let mut em_area = 0usize;
    let mut sobrepostas = 0usize;
    for seed in 1..=16u64 {
        let mut rng = Lcg(seed * 7 + 1);
        let (w, h) = (24.0, 16.0);
        let obs = obstaculos(&mut rng, 8, w, h);
        let ars = areas(&mut rng, 5, w, h);
        let r = [0.0, 0.4][(seed % 2) as usize];
        let params = Params {
            agent_radius: r,
            ..Params::default()
        };
        let b = build_with_areas(&retangulo(w, h), &obs, &ars, &params)
            .unwrap_or_else(|e| panic!("semente {seed}: {e:?}"));
        let m = &b.mesh;
        let aneis: Vec<Vec<P>> = ars
            .iter()
            .map(|a| inflate::inflate(&a.shape, r, params.corner, DISK_SIDES))
            .collect();
        for _ in 0..4_000 {
            let p = [rng.range(0.0, w), rng.range(0.0, h)];
            let Some(poly) = m.locate(p) else { continue };
            let q = to_lattice(p);
            let lados: Vec<Option<bool>> = aneis.iter().map(|ring| dentro(ring, q)).collect();
            if lados.iter().any(Option::is_none) {
                continue;
            }
            let quem: Vec<u16> = lados
                .iter()
                .zip(&ars)
                .filter(|(d, _)| **d == Some(true))
                .map(|(_, a)| a.id)
                .collect();
            let esperado = quem.first().copied().unwrap_or(0);
            sobrepostas += usize::from(quem.len() > 1);
            em_area += usize::from(esperado != 0);
            amostras += 1;
            assert_eq!(
                m.area_id(poly),
                esperado,
                "semente {seed}: o ponto {p:?} está na área {esperado} e o polígono diz {}",
                m.area_id(poly)
            );
        }
    }
    // O piso: uma varredura sem pontos em áreas (ou sem sobreposições) mediria nada.
    assert!(amostras >= 30_000, "só {amostras} amostras");
    assert!(em_area >= 3_000, "só {em_area} amostras dentro de áreas");
    assert!(
        sobrepostas >= 100,
        "só {sobrepostas} amostras em sobreposições"
    );
}

#[test]
fn a_fronteira_de_uma_area_e_passagem_e_nunca_parede() {
    // Uma área a cortar o chão ao meio: sem obstáculos, a malha é UMA ilha, e as paredes são só as
    // da região (a fronteira da área não entra nas paredes).
    let ars = [Area {
        shape: Shape::Convex(vec![[8.0, -1.0], [12.0, -1.0], [12.0, 11.0], [8.0, 11.0]]),
        id: 3,
    }];
    let b =
        build_with_areas(&retangulo(20.0, 10.0), &[], &ars, &Params::default()).expect("constrói");
    let m = &b.mesh;
    assert_eq!(m.island_count(), 1);
    assert!(m.has_areas());
    let perimetro: f64 = m
        .walls()
        .iter()
        .map(|&(u, w)| ph2d_nav::geom::dist(m.vert(u), m.vert(w)))
        .sum();
    assert!(
        (perimetro - 60.0).abs() < 1e-6,
        "as paredes medem {perimetro} m — a fronteira da área virou parede?"
    );
    let area3: f64 = (0..m.poly_count() as u32)
        .filter(|&p| m.area_id(p) == 3)
        .map(|p| {
            let v = m.poly(p).verts;
            let n = v.len();
            (0..n)
                .map(|i| {
                    let (a, b) = (m.vert(v[i]), m.vert(v[(i + 1) % n]));
                    a[0] * b[1] - b[0] * a[1]
                })
                .sum::<f64>()
                * 0.5
        })
        .sum();
    assert!((area3 - 40.0).abs() < 1e-6, "a área 3 mede {area3} m²");
}

#[test]
fn cem_cenas_com_areas_nenhuma_recusa() {
    let mut construidas = 0usize;
    for seed in 1..=100u64 {
        let mut rng = Lcg(seed * 131 + 5);
        let (w, h) = (30.0, 20.0);
        let obs = obstaculos(&mut rng, 4 + (seed % 12) as usize, w, h);
        let ars = areas(&mut rng, 1 + (seed % 7) as usize, w, h);
        let params = Params {
            agent_radius: [0.0, 0.25, 0.5][(seed % 3) as usize],
            merge: seed % 5 != 0,
            ..Params::default()
        };
        let b = build_with_areas(&retangulo(w, h), &obs, &ars, &params)
            .unwrap_or_else(|e| panic!("semente {seed}: {e:?}"));
        construidas += usize::from(b.mesh.has_areas());
    }
    assert!(
        construidas >= 90,
        "só {construidas} cenas tiveram áreas na malha"
    );
}
