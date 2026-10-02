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
        corner: if seed % 5 == 0 {
            Corner::Miter
        } else {
            Corner::Round
        },
        disk_sides: 8,
        merge: seed % 2 == 0,
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
