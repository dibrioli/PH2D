//! ⭐⭐⭐ **A MALHA POR MOSAICOS** (plano 30 §2.4, W6) — as três leis que a tornam a MESMA malha:
//!
//! 1. por mosaicos ≡ inteira: a mesma área andável e o mesmo caminho mais curto, e a procura sobre a
//!    malha montada continua a ser o EXACTO (as costuras não abrem nem fecham nada);
//! 2. incremental ≡ a frio: depois de mexer num obstáculo, a malha actualizada é, AO BIT, a de uma
//!    construção nova com a mesma entrada — e só os mosaicos tocados se reconstroem;
//! 3. a porta: fechar desvia o caminho, abrir devolve o original ao bit.
//!
//! ⚠️ O CONTROLO de cada uma está ao lado: a varredura tem piso de população, e o «só os tocados» é
//! medido contra o total de mosaicos.

use ph2d_nav::{NavMesh, NoPath, Polyanya, oracle};
use ph2d_navmesh::{Corner, Params, Shape, TiledMesh, build};

use crate::cena::{Lcg, obstaculos, retangulo};

fn params(seed: u64) -> Params {
    Params {
        agent_radius: [0.0, 0.3, 0.7][(seed % 3) as usize],
        corner: if seed.is_multiple_of(5) {
            Corner::Miter
        } else {
            Corner::Round
        },
        disk_sides: 8,
        merge: seed.is_multiple_of(2),
    }
}

/// Os pares de pontos dentro das DUAS malhas.
fn pontos(rng: &mut Lcg, a: &NavMesh, b: &NavMesh, n: usize) -> Vec<[f64; 2]> {
    let mut v = Vec::new();
    let mut tentativas = 0;
    while v.len() < n && tentativas < 10_000 {
        tentativas += 1;
        let p = [rng.range(0.0, 16.0), rng.range(0.0, 12.0)];
        if a.locate(p).is_some() && b.locate(p).is_some() {
            v.push(p);
        }
    }
    v
}

#[test]
fn por_mosaicos_e_a_mesma_malha_que_inteira() {
    let mut s = Polyanya::new();
    let mut comparados = 0usize;
    let mut com_volta = 0usize;
    let mut pior_area = 0.0f64;
    let mut pior_caminho = 0.0f64;
    for seed in 1..=24u64 {
        let mut rng = Lcg(seed);
        let obs = obstaculos(&mut rng, 3 + (seed as usize % 6), 16.0, 12.0);
        let p = params(seed);
        let inteira = build(&retangulo(16.0, 12.0), &obs, &p)
            .expect("constrói")
            .mesh;
        // Mosaicos de 3 m e de 5 m: uma região de 16 × 12 m fica partida em 20 e em 12 pedaços.
        let lado = if seed % 2 == 0 { 3.0 } else { 5.0 };
        let mut t = TiledMesh::new(p, lado);
        assert!(t.update(&retangulo(16.0, 12.0), &obs));
        assert_eq!(t.stats().failed, 0, "semente {seed}: um mosaico recusou");
        let m = t.mesh();
        // ⚠️ A régua da área: cada ponto de costura arredonda à grelha, logo a fronteira entre
        // mosaicos move-se no máximo UMA unidade (`1/65 536 m`) — uma faixa desse tamanho ao longo
        // de todas as costuras é o máximo que a área pode mudar.
        let costura =
            ((16.0f64 / lado).ceil() - 1.0) * 12.0 + ((12.0f64 / lado).ceil() - 1.0) * 16.0;
        let da = (m.area() - inteira.area()).abs();
        assert!(
            da <= costura / ph2d_navmesh::lattice::SCALE,
            "semente {seed}: a área mudou {da} m² com {costura} m de costura"
        );
        pior_area = pior_area.max(da / costura * ph2d_navmesh::lattice::SCALE);
        for (i, a) in pontos(&mut rng, &inteira, m, 12).iter().enumerate() {
            for b in pontos(&mut Lcg(seed * 1_000 + i as u64), &inteira, m, 4) {
                let (ri, rm) = (s.find_path(&inteira, *a, b), s.find_path(m, *a, b));
                match (ri, rm) {
                    (Ok(pi), Ok(pm)) => {
                        pior_caminho = pior_caminho.max((pi.length - pm.length).abs());
                        let (_, ex) = oracle::shortest(m, *a, b).expect("o exacto acha-o também");
                        assert!(
                            (pm.length - ex).abs() <= 1e-9 * ex.max(1.0),
                            "semente {seed}: na malha montada a procura deu {} e o exacto {ex}",
                            pm.length
                        );
                        if pm.points.len() > 2 {
                            com_volta += 1;
                        }
                        comparados += 1;
                    }
                    (Err(NoPath::Unreachable), Err(NoPath::Unreachable)) => {}
                    (x, y) => panic!("semente {seed}: inteira {x:?} contra mosaicos {y:?}"),
                }
            }
        }
    }
    // O caminho só encosta às PAREDES, e uma parede cortada por uma costura dobra no máximo meia
    // unidade da grelha no ponto do corte ⇒ o comprimento muda nessa ordem.
    assert!(
        pior_caminho <= 4.0 / ph2d_navmesh::lattice::SCALE,
        "um caminho mudou {pior_caminho} m (a faixa da costura usou {pior_area} unidades)"
    );
    assert!(comparados >= 600, "comparou só {comparados} pares");
    assert!(com_volta >= 100, "só {com_volta} caminhos viraram");
}

#[test]
fn incremental_e_a_frio_dao_o_mesmo() {
    let reg = retangulo(16.0, 12.0);
    let mut rng = Lcg(77);
    let mut obs = obstaculos(&mut rng, 9, 16.0, 12.0);
    let p = params(1);
    let mut viva = TiledMesh::new(p, 3.0);
    viva.update(&reg, &obs);
    let total = viva.stats().tiles;
    assert!(
        !viva.update(&reg, &obs),
        "nada mudou e a malha diz que mudou"
    );
    assert_eq!(viva.stats().rebuilt, 0);
    let mut parciais = 0;
    for passo in 0..12 {
        // Mexe UM obstáculo (o círculo pequeno vai dando a volta pela região).
        obs[passo % 9] = Shape::Circle {
            center: [1.0 + passo as f64 * 1.2, 2.0 + (passo % 4) as f64 * 2.5],
            radius: 0.6,
        };
        assert!(viva.update(&reg, &obs));
        let r = viva.stats().rebuilt;
        assert!(
            r >= 1,
            "passo {passo}: mudou e nenhum mosaico se reconstruiu"
        );
        if r < total {
            parciais += 1;
        }
        let mut fria = TiledMesh::new(p, 3.0);
        fria.update(&reg, &obs);
        let (a, b) = (viva.mesh(), fria.mesh());
        assert_eq!(a.verts(), b.verts(), "passo {passo}: os vértices diferem");
        assert_eq!(a.polys(), b.polys(), "passo {passo}: os polígonos diferem");
    }
    // O CONTROLO de «só os tocados»: tem de haver actualizações que NÃO refizeram tudo.
    assert!(
        parciais >= 10,
        "só {parciais} de 12 mudanças foram parciais em {total} mosaicos"
    );
}

#[test]
fn a_porta_fecha_e_abre() {
    // Duas salas ligadas por uma porta de 1 m no meio de uma parede em x = 8.
    let reg = retangulo(16.0, 12.0);
    let parede = |y0: f64, y1: f64| Shape::Convex(vec![[7.8, y0], [8.2, y0], [8.2, y1], [7.8, y1]]);
    let aberta = vec![parede(0.0, 5.5), parede(6.5, 12.0)];
    let mut fechada = aberta.clone();
    fechada.push(parede(5.4, 6.6));
    let p = Params {
        agent_radius: 0.3,
        ..Params::default()
    };
    let mut t = TiledMesh::new(p, 4.0);
    let mut s = Polyanya::new();
    let (a, b) = ([2.0, 6.0], [14.0, 6.0]);
    t.update(&reg, &aberta);
    let antes = s
        .find_path(t.mesh(), a, b)
        .expect("a porta aberta deixa passar");
    assert!((antes.length - 12.0).abs() < 1e-9, "pela porta é a direito");
    assert!(t.update(&reg, &fechada));
    assert!(
        t.stats().rebuilt < t.stats().tiles,
        "fechar UMA porta refez os {} mosaicos",
        t.stats().tiles
    );
    assert_eq!(
        s.find_path(t.mesh(), a, b).map(|p| p.length),
        Err(NoPath::Unreachable),
        "com a porta fechada as salas separam-se"
    );
    assert!(t.update(&reg, &aberta));
    let depois = s.find_path(t.mesh(), a, b).expect("reaberta");
    assert_eq!(
        depois.points, antes.points,
        "reabrir devolve o caminho ao bit"
    );
}

/// ⭐ **A junção em T é reparada** (a prova de mutação mostrou a reparação sem régua): com raio `0`, um
/// losango cuja ponta toca EXACTAMENTE a costura `x = 4` (a meio dela) dá ao mosaico da esquerda um vértice na
/// linha que o da direita não tem. Sem a reparação a costura vira parede ali; com ela, nenhuma
/// parede da malha montada fica deitada sobre a costura.
#[test]
fn a_juncao_em_t_da_costura_e_reparada() {
    let reg = retangulo(8.0, 8.0);
    // ⚠️ A ponta a MEIO da costura (`y = 5`): em `y = 4` ela caía no canto de quatro mosaicos, onde
    // o vizinho já tem o vértice, e o gate passava sem a reparação (a mutação apanhou-o).
    let losango = Shape::Convex(vec![[3.0, 5.0], [3.5, 4.5], [4.0, 5.0], [3.5, 5.5]]);
    let p = Params {
        agent_radius: 0.0,
        ..Params::default()
    };
    let mut t = TiledMesh::new(p, 4.0);
    t.update(&reg, &[losango]);
    let m = t.mesh();
    let na_costura: Vec<_> = m
        .walls()
        .iter()
        .filter(|&&(a, b)| m.vert(a)[0] == 4.0 && m.vert(b)[0] == 4.0)
        .collect();
    assert!(
        na_costura.is_empty(),
        "paredes deitadas na costura: {na_costura:?}"
    );
    // O CONTROLO: a fixtura tem MESMO o vértice na costura (a ponta do losango).
    assert!(m.verts().contains(&[4.0, 5.0]));
}

/// (W7) Os m² de cada área numa malha.
fn area_por_id(m: &NavMesh, id: u16) -> f64 {
    (0..m.polys().len() as u32)
        .filter(|&p| m.area_id(p) == id)
        .map(|p| {
            let v = &m.polys()[p as usize].verts;
            let n = v.len();
            (0..n)
                .map(|i| {
                    let (a, b) = (m.vert(v[i]), m.vert(v[(i + 1) % n]));
                    a[0] * b[1] - b[0] * a[1]
                })
                .sum::<f64>()
                * 0.5
        })
        .sum()
}

#[test]
fn com_areas_por_mosaicos_e_a_mesma_malha_que_inteira() {
    let mut s = Polyanya::new();
    let mut comparados = 0usize;
    let mut refractaram = 0usize;
    for seed in 1..=16u64 {
        let mut rng = Lcg(seed * 3 + 11);
        let obs = obstaculos(&mut rng, 3 + (seed as usize % 5), 16.0, 12.0);
        let ars = crate::areas::areas(&mut rng, 1 + (seed % 4) as usize, 16.0, 12.0);
        let p = params(seed);
        let inteira = ph2d_navmesh::build_with_areas(&retangulo(16.0, 12.0), &obs, &ars, &p)
            .expect("constrói")
            .mesh;
        let lado = if seed % 2 == 0 { 3.0 } else { 5.0 };
        let mut t = TiledMesh::new(p, lado);
        assert!(t.update_with_areas(&retangulo(16.0, 12.0), &obs, &ars));
        assert_eq!(t.stats().failed, 0, "semente {seed}: um mosaico recusou");
        let m = t.mesh();
        // A régua: a faixa de UMA unidade ao longo das costuras (a lei da área andável, acima),
        // agora por área — a fronteira de uma área cortada por uma costura mexe igual.
        let costura =
            ((16.0f64 / lado).ceil() - 1.0) * 12.0 + ((12.0f64 / lado).ceil() - 1.0) * 16.0;
        for id in std::iter::once(0).chain(ars.iter().map(|a| a.id)) {
            let da = (area_por_id(m, id) - area_por_id(&inteira, id)).abs();
            assert!(
                da <= costura / ph2d_navmesh::lattice::SCALE,
                "semente {seed}: a área {id} mudou {da} m²"
            );
        }
        let costs: Vec<f64> = (0..=ars.len())
            .map(|i| if i == 0 { 1.0 } else { 3.0 })
            .collect();
        for (i, a) in pontos(&mut rng, &inteira, m, 6).iter().enumerate() {
            for b in pontos(&mut Lcg(seed * 977 + i as u64), &inteira, m, 3) {
                let (Ok(pi), Ok(pm)) = (
                    s.find_path_costs(&inteira, &costs, *a, b),
                    s.find_path_costs(m, &costs, *a, b),
                ) else {
                    continue;
                };
                // As malhas diferem nas costuras, logo a grelha das fronteiras também: a régua é o
                // erro da procura medido contra o oráculo (§3 da sonda), não o bit.
                assert!(
                    (pi.cost - pm.cost).abs() <= 2e-2 * pi.cost.max(1.0),
                    "semente {seed}: {a:?} → {b:?}: inteira {} contra mosaicos {}",
                    pi.cost,
                    pm.cost
                );
                refractaram += usize::from(pi.cost > pi.length * (1.0 + 1e-9));
                comparados += 1;
            }
        }
    }
    assert!(comparados >= 200, "só {comparados} pares");
    assert!(
        refractaram >= 40,
        "só {refractaram} caminhos pagaram custo — as áreas não estorvam"
    );
}

#[test]
fn com_areas_incremental_e_a_frio_dao_o_mesmo() {
    for seed in 1..=8u64 {
        let mut rng = Lcg(seed * 5 + 2);
        let obs = obstaculos(&mut rng, 5, 16.0, 12.0);
        let mut ars = crate::areas::areas(&mut rng, 3, 16.0, 12.0);
        let p = params(seed);
        let mut t = TiledMesh::new(p, 4.0);
        t.update_with_areas(&retangulo(16.0, 12.0), &obs, &ars);
        let total = t.stats().tiles;
        // Mexe UMA área (a 2.ª), e troca o número de outra: só os mosaicos delas se refazem.
        if let Shape::Circle { center, .. } | Shape::Capsule { a: center, .. } = &mut ars[1].shape {
            center[0] += 0.37;
        } else if let Shape::Convex(v) = &mut ars[1].shape {
            v.iter_mut().for_each(|q| q[0] += 0.37);
        }
        assert!(t.update_with_areas(&retangulo(16.0, 12.0), &obs, &ars));
        let refeitos = t.stats().rebuilt;
        assert!(
            refeitos < total,
            "semente {seed}: refez {refeitos} de {total} mosaicos"
        );
        let mut frio = TiledMesh::new(p, 4.0);
        frio.update_with_areas(&retangulo(16.0, 12.0), &obs, &ars);
        let (a, b) = (t.mesh(), frio.mesh());
        assert_eq!(a.verts(), b.verts(), "semente {seed}: os vértices");
        assert_eq!(a.polys(), b.polys(), "semente {seed}: os polígonos");
        assert!(
            (0..a.polys().len() as u32).all(|q| a.area_id(q) == b.area_id(q)),
            "semente {seed}: as áreas"
        );
        // CONTROLO: sem mudança, nada se refaz.
        assert!(!t.update_with_areas(&retangulo(16.0, 12.0), &obs, &ars));
        assert_eq!(t.stats().rebuilt, 0);
    }
}

/// Uma caixa alinhada de centro `c` e meias-medidas `h`.
fn caixa(c: [f64; 2], h: [f64; 2]) -> Shape {
    Shape::Convex(vec![
        [c[0] - h[0], c[1] - h[1]],
        [c[0] + h[0], c[1] - h[1]],
        [c[0] + h[0], c[1] + h[1]],
        [c[0] - h[0], c[1] + h[1]],
    ])
}

/// ⭐ **O furo que corta a QUINA de quatro mosaicos, com as paredes do recinto encostadas, não vaza**
/// (a cena `PH2D_NAV_SMOKE=4`, fotografada em 03/10: a malha do vermelho tinha chão DENTRO da lava). O
/// Clipper devolvia o anel andável de um mosaico com um ESPIGÃO sobre a costura, e a paridade etiquetava
/// o entalhe como chão. ⚠️ Herdado da W6 (medido no HEAD dela). CONTROLO: a mesma cena sem as paredes
/// nunca vazou — são elas que tiram o anel do rectângulo do mosaico.
#[test]
fn o_furo_na_quina_de_quatro_mosaicos_nao_vaza() {
    let reg = vec![[-5.8, -2.0], [5.8, -2.0], [5.8, 3.6], [-5.8, 3.6]];
    let lava = caixa([0.0, 0.2], [0.6, 2.2]);
    let paredes = vec![
        caixa([0.0, 3.85], [6.3, 0.25]),
        caixa([0.0, -2.25], [6.3, 0.25]),
        caixa([-6.05, 0.8], [0.25, 2.8]),
        caixa([6.05, 0.8], [0.25, 2.8]),
    ];
    let mut obs = paredes.clone();
    obs.push(lava);
    let p = Params {
        agent_radius: 0.3515625,
        ..Params::default()
    };
    let mut t = TiledMesh::new(p, 15.0);
    t.update(&reg, &obs);
    let inteira = build(&reg, &obs, &p).expect("constrói").mesh;
    // ⚠️ A régua é a DISTÂNCIA ao rectângulo da lava contra o raio (a quina recuada é REDONDA): um
    // ponto a menos de `r` dela não pode ser chão. A 1.ª redacção amostrava o rectângulo alargado e
    // acusou um ponto da quina, a `0,36 m` dela.
    let mut dentro = 0;
    for i in 0..40 {
        for j in 0..80 {
            let q = [-1.0 + 2.0 * f64::from(i) / 39.0, -2.0 + 4.8 * f64::from(j) / 79.0];
            let dx = (q[0].abs() - 0.6).max(0.0);
            let dy = ((q[1] - 0.2).abs() - 2.2).max(0.0);
            if (dx * dx + dy * dy).sqrt() >= p.agent_radius - 1e-3 {
                continue;
            }
            assert!(t.mesh().locate(q).is_none(), "o ponto {q:?} da lava é chão nos mosaicos");
            assert!(inteira.locate(q).is_none(), "o ponto {q:?} da lava é chão na inteira");
            dentro += 1;
        }
    }
    assert!(dentro >= 1_000, "só {dentro} pontos dentro da lava");
    assert!(
        (t.mesh().area() - inteira.area()).abs() < 1e-3,
        "{} contra {}",
        t.mesh().area(),
        inteira.area()
    );
}

/// A régua da W6 (por mosaicos ≡ inteira) em regiões que CRUZAM as costuras em coordenadas negativas,
/// com paredes encostadas à região — a família que os gates de cima (no quadrante positivo, sem
/// paredes no bordo) não viam.
#[test]
fn em_regioes_negativas_com_paredes_no_bordo_por_mosaicos_e_a_inteira() {
    let mut casos = 0;
    for seed in 1..=16u64 {
        let mut rng = Lcg(seed * 41 + 3);
        let (w, h) = (rng.range(6.0, 14.0), rng.range(4.0, 9.0));
        let c = [rng.range(-3.0, 3.0), rng.range(-3.0, 3.0)];
        let reg = vec![
            [c[0] - w, c[1] - h],
            [c[0] + w, c[1] - h],
            [c[0] + w, c[1] + h],
            [c[0] - w, c[1] + h],
        ];
        let mut obs = vec![
            caixa([c[0], c[1] + h + 0.25], [w + 0.5, 0.25]),
            caixa([c[0], c[1] - h - 0.25], [w + 0.5, 0.25]),
            caixa([c[0] - w - 0.25, c[1]], [0.25, h]),
            caixa([c[0] + w + 0.25, c[1]], [0.25, h]),
        ];
        for o in obstaculos(&mut rng, 6, 2.0 * w, 2.0 * h) {
            obs.push(match o {
                Shape::Convex(v) => Shape::Convex(
                    v.iter()
                        .map(|q| [q[0] + c[0] - w, q[1] + c[1] - h])
                        .collect(),
                ),
                Shape::Circle { center, radius } => Shape::Circle {
                    center: [center[0] + c[0] - w, center[1] + c[1] - h],
                    radius,
                },
                Shape::Capsule { a, b, radius } => Shape::Capsule {
                    a: [a[0] + c[0] - w, a[1] + c[1] - h],
                    b: [b[0] + c[0] - w, b[1] + c[1] - h],
                    radius,
                },
            });
        }
        let p = params(seed);
        let inteira = build(&reg, &obs, &p).expect("constrói").mesh;
        for lado in [3.0, 15.0] {
            let mut t = TiledMesh::new(p, lado);
            t.update(&reg, &obs);
            assert_eq!(t.stats().failed, 0, "semente {seed}: um mosaico recusou");
            let costura = 4.0 * (w + h) * (2.0 * w.max(h) / lado + 1.0);
            let da = (t.mesh().area() - inteira.area()).abs();
            assert!(
                da <= costura / ph2d_navmesh::lattice::SCALE,
                "semente {seed}, mosaico {lado}: a área mudou {da} m²"
            );
            casos += 1;
        }
    }
    assert_eq!(casos, 32);
}
