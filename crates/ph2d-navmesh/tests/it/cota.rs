//! ⭐⭐ (W19, plano 30 §28.1–§28.2) **A COTA pela distância às áreas baratas** (`ph2d_nav::cota`) — o
//! heurístico da procura ponderada e o «alvo à vista». O juiz é o ORÁCULO ponderado (o grafo dos cantos
//! e Steiner, `ph2d_nav::oracle`), nunca a função sob teste; o CONTROLO é a cota global da W7
//! (`Polyanya::set_cota_global`), a de antes, que é o custo de hoje.

use ph2d_nav::agent::a_vista;
use ph2d_nav::cost::segment_cost;
use ph2d_nav::geom::dist;
use ph2d_nav::oracle::WeightedOracle;
use ph2d_nav::{Link, NavMesh, Plano, Polyanya, Query, V2};

use super::dominancia::{Lcg, cena_com};

/// O custo e os nós expandidos de uma procura ponderada.
fn procura(s: &mut Polyanya, m: &NavMesh, costs: &[f64], a: V2, z: V2) -> (f64, u64) {
    let e0 = s.stats.expanded;
    let p = s.find_path_costs(m, costs, a, z).expect("mesma ilha");
    (p.cost, s.stats.expanded - e0)
}

/// Pares `(a, z)` na mesma ilha das duas malhas.
fn pares(seed: u64, malhas: &[&NavMesh], n: usize, perto: Option<f64>) -> Vec<(V2, V2)> {
    let mut r = Lcg(seed * 31 + 7);
    let mut s = Polyanya::new();
    let mut v = Vec::new();
    while v.len() < n {
        let a = [r.next() * 30.0, r.next() * 20.0];
        let z = match perto {
            Some(l) => [
                a[0] + (r.next() * 2.0 - 1.0) * l,
                a[1] + (r.next() * 2.0 - 1.0) * l,
            ],
            None => [r.next() * 30.0, r.next() * 20.0],
        };
        if malhas
            .iter()
            .all(|m| m.locate(a).is_some() && m.locate(z).is_some() && s.find_path(m, a, z).is_ok())
        {
            v.push((a, z));
        }
    }
    v
}

/// ⭐ Uma estrada a `0,3` num canto longe de tudo não pesa nas procuras que não passam por ela. Medido
/// (a cena grande da sonda, `target/prova/w19/medir_cota_*.txt`): com a cota global `24 176 → 85 744`
/// nós por consulta; com a nova `24 176 → 24 232`. O CONTROLO é a cota global: pesa.
///
/// ⚠️ A régua «sem ela» é o mundo SEM estrada nenhuma — a malha E a tabela sem o `0,3`. Medida com a
/// mesma tabela dos dois lados, uma mutação que esquece a geometria (o heurístico ou a saída cedo pela
/// cota global) inflava os dois por igual e SOBREVIVIA (mutação H1/H2 da W19, `0/9`).
#[test]
fn uma_area_barata_longe_nao_pesa_na_procura() {
    let costs = [1.0, 4.0, 4.0, 4.0, 4.0, 0.3];
    let costs_sem = [1.0, 4.0, 4.0, 4.0, 4.0];
    let (mut sem, mut com, mut com_global) = (0u64, 0u64, 0u64);
    for seed in 1..=3u64 {
        let m_sem = cena_com(seed, &[], None);
        let m_com = cena_com(seed, &[], Some(([1.2, 18.8], 1.0)));
        let mut nova = Polyanya::new();
        let mut global = Polyanya::new();
        global.set_cota_global(true);
        for (a, z) in pares(seed, &[&m_sem, &m_com], 10, None) {
            sem += procura(&mut nova, &m_sem, &costs_sem, a, z).1;
            com += procura(&mut nova, &m_com, &costs, a, z).1;
            com_global += procura(&mut global, &m_com, &costs, a, z).1;
        }
    }
    eprintln!("nós: sem a estrada {sem} · com ela {com} · CONTROLO (cota global) {com_global}");
    assert!(
        com as f64 <= 1.1 * sem as f64,
        "a estrada longe pesou: {sem} → {com} nós"
    );
    assert!(
        com_global as f64 >= 1.5 * sem as f64,
        "a fixtura: com a cota global a estrada pesa ({sem} → {com_global})"
    );
}

/// ⭐ Com áreas caras E baratas, a todos os pesos: a cota nova nunca dá um caminho mais caro que a global
/// (a de hoje), e o oráculo confirma cada um (`≤ 1 + 1e-3` dele — o oráculo a `0,1 m` está ACIMA do
/// óptimo). Medido (a sonda, `432` pares): igual ao bit em `431`; num, a nova acha um MAIS barato
/// (`−1,48e-4`, abaixo do oráculo; a global ficava `1,2e-4` acima dele). E expande menos.
#[test]
fn a_cota_nunca_encarece_um_caminho_e_o_oraculo_confirma() {
    let (mut nos_nova, mut nos_global, mut n) = (0u64, 0u64, 0usize);
    for seed in 1..=3u64 {
        let m = cena_com(seed, &[2, 4], None);
        for (caro, barato) in [(1.5, 0.6), (4.0, 0.3), (10.0, 0.9)] {
            let costs = [1.0, caro, barato, caro, barato];
            let orc = WeightedOracle::new(&m, &costs, 0.1);
            let mut nova = Polyanya::new();
            let mut global = Polyanya::new();
            global.set_cota_global(true);
            for (a, z) in pares(seed, &[&m], 6, None) {
                let (cn, en) = procura(&mut nova, &m, &costs, a, z);
                let (cg, eg) = procura(&mut global, &m, &costs, a, z);
                let (_, o) = orc.shortest(a, z).expect("o oráculo acha");
                let ctx = format!("semente {seed}, {costs:?}, {a:?} → {z:?}");
                assert!(cn <= cg * (1.0 + 1e-9), "{ctx}: a nova {cn}, a global {cg}");
                assert!(cn <= o * (1.0 + 1e-3), "{ctx}: a nova {cn}, o oráculo {o}");
                nos_nova += en;
                nos_global += eg;
                n += 1;
            }
        }
    }
    eprintln!("{n} pares: {nos_nova} nós com a cota nova, {nos_global} com a global");
    assert!(
        nos_nova < nos_global,
        "a nova expande {nos_nova}, a global {nos_global}"
    );
}

/// ⭐ (§28.2) **Quando o «alvo à vista» diz SIM, nenhum caminho é mais barato que a recta** — em cenas com
/// áreas caras e baratas, o oráculo nunca acha um caminho abaixo do custo da recta. A população: os SIM
/// numa tabela com custos `< 1` (a regra da W16 dizia NÃO a todos), e os NÃO onde a área barata perto
/// vence a recta (a mesma recta, sem área barata, seria SIM).
#[test]
fn quando_a_vista_diz_sim_nenhum_caminho_e_mais_barato() {
    let (mut sim, mut barata_vence) = (0usize, 0usize);
    for seed in 1..=3u64 {
        let m = cena_com(seed, &[2, 4], None);
        let costs = [1.0, 4.0, 0.3, 4.0, 0.3];
        let sem_baratas = [1.0, 4.0, 1.0, 4.0, 1.0];
        let orc = WeightedOracle::new(&m, &costs, 0.1);
        let q = Query {
            costs: &costs,
            links: &[],
        };
        let q_ctl = Query {
            costs: &sem_baratas,
            links: &[],
        };
        for (a, z) in pares(seed, &[&m], 60, Some(6.0)) {
            let Some(recta) = segment_cost(&m, &costs, a, z) else {
                continue;
            };
            let (_, o) = orc.shortest(a, z).expect("o oráculo acha");
            if a_vista(&m, &q, a, z) {
                assert!(
                    recta <= o * (1.0 + 1e-9),
                    "semente {seed}, {a:?} → {z:?}: à vista a {recta}, o oráculo acha {o}"
                );
                sim += 1;
            } else if a_vista(&m, &q_ctl, a, z) && o < dist(a, z) * (1.0 - 1e-6) {
                barata_vence += 1;
            }
        }
    }
    eprintln!(
        "à vista: {sim} SIM numa tabela com áreas baratas · {barata_vence} NÃO onde a barata vence"
    );
    assert!(sim >= 30, "a população dos SIM: {sim}");
    assert!(
        barata_vence >= 3,
        "a população dos NÃO pela área barata: {barata_vence}"
    );
}

/// O custo de um caminho com atalhos: os troços andados e o preço de cada salto.
fn custo_com_atalhos(
    m: &NavMesh,
    costs: &[f64],
    links: &[Link],
    pts: &[V2],
    hops: &[ph2d_nav::Hop],
) -> f64 {
    let mut c = 0.0;
    for (i, w) in pts.windows(2).enumerate() {
        match hops.iter().find(|h| h.at == i) {
            Some(h) => {
                let l = links.iter().find(|l| l.id == h.link).expect("o atalho");
                c += l.crossing_cost();
            }
            None => c += segment_cost(m, costs, w[0], w[1]).expect("o troço anda-se na malha"),
        }
    }
    c
}

/// ⭐ (§28.1) **A poda dos ATALHOS pela cota nunca perde o caminho mais barato** — contra a força bruta: o
/// mínimo de ir a direito e de ir por cada ponta do teleporte, cada troço pela procura ponderada sem
/// atalhos. Com áreas BARATAS na cena (a poda de um troço pela recta seria inadmissível: mutação H5).
#[test]
fn com_atalhos_a_poda_pela_cota_nunca_perde_o_mais_barato() {
    let costs = [1.0, 4.0, 0.3, 4.0, 0.3];
    let (mut pares_n, mut pelo_atalho) = (0usize, 0usize);
    for seed in 1..=3u64 {
        let m = cena_com(seed, &[2, 4], None);
        let pontas = |p: V2| m.nearest_point(p, None).expect("na malha").0;
        let links = [Link {
            id: 0,
            from: pontas([2.0, 2.0]),
            to: pontas([27.0, 17.0]),
            two_way: true,
            teleport: true,
            cost: 1.0,
        }];
        let q = Query {
            costs: &costs,
            links: &links,
        };
        let mut s = Polyanya::new();
        let mut p = |x: V2, y: V2| {
            s.find_path_costs(&m, &costs, x, y)
                .map_or(f64::INFINITY, |r| r.cost)
        };
        for (a, z) in pares(seed, &[&m], 20, None) {
            let (la, lb) = (links[0].from, links[0].to);
            let c = links[0].crossing_cost();
            let esperado = p(a, z)
                .min(p(a, la) + c + p(lb, z))
                .min(p(a, lb) + c + p(la, z));
            let mut plano = match Plano::begin(&m, &mut Polyanya::new(), &q, a, z) {
                Ok(pl) => pl,
                Err(r) => {
                    let (pts, hops, _) = r.expect("um caminho");
                    let got = custo_com_atalhos(&m, &costs, &links, &pts, &hops);
                    assert!(
                        got <= esperado * (1.0 + 1e-9),
                        "semente {seed}, {a:?} → {z:?}: {got} > {esperado}"
                    );
                    continue;
                }
            };
            let r = loop {
                if let Some(r) = plano.run(&m, &q, u64::MAX) {
                    break r;
                }
            };
            let (pts, hops, _) = r.expect("um caminho");
            let got = custo_com_atalhos(&m, &costs, &links, &pts, &hops);
            assert!(
                got <= esperado * (1.0 + 1e-9),
                "semente {seed}, {a:?} → {z:?}: o produto {got}, a força bruta {esperado}"
            );
            pares_n += 1;
            pelo_atalho += usize::from(!hops.is_empty());
        }
    }
    eprintln!("{pares_n} pares com procura, {pelo_atalho} pelo atalho");
    assert!(
        pelo_atalho >= 5,
        "a população dos que usam o atalho: {pelo_atalho}"
    );
}

/// ⭐ (§28.1, H5) **A fixtura DIRIGIDA da poda dos atalhos** — o teletransporte deixa o corpo na ponta de uma
/// estrada barata que leva ao alvo, e a recta dessa ponta ao alvo (`12 m`) é mais longa que o caminho a
/// pé até ele (`~7`): podar o troço pela RECTA (a cota sem a estrada) perde o atalho, que custa `~5,6`. Uma
/// parede ao meio com um vão à direita: o alvo, em cima, só se alcança a pé pelo vão.
#[test]
fn o_atalho_que_acaba_numa_estrada_barata_nao_e_podado() {
    use ph2d_navmesh::{Area, Params, Shape, build_with_areas};
    let caixa = |x0: f64, y0: f64, x1: f64, y1: f64| {
        Shape::Convex(vec![[x0, y0], [x1, y0], [x1, y1], [x0, y1]])
    };
    let reg = vec![[0.0, 0.0], [14.0, 0.0], [14.0, 6.0], [0.0, 6.0]];
    let paredes = [caixa(0.0, 2.5, 8.5, 3.5), caixa(9.5, 2.5, 14.0, 3.5)];
    let estrada = [Area {
        shape: caixa(0.5, 4.2, 13.5, 5.8),
        id: 1,
        dentro: true,
    }];
    let params = Params {
        agent_radius: 0.1,
        ..Params::default()
    };
    let m = build_with_areas(&reg, &paredes, &estrada, &params)
        .expect("constrói")
        .mesh;
    let costs = [1.0, 0.3];
    let (s, t) = ([13.0, 1.0], [13.0, 5.0]);
    let links = [Link {
        id: 0,
        from: [12.0, 1.0],
        to: [1.0, 5.0],
        two_way: false,
        teleport: true,
        cost: 1.0,
    }];
    let q = Query {
        costs: &costs,
        links: &links,
    };
    let mut busca = Polyanya::new();
    let mut p = |x: V2, y: V2| {
        busca
            .find_path_costs(&m, &costs, x, y)
            .expect("anda-se")
            .cost
    };
    let a_pe = p(s, t);
    let pelo_atalho = p(s, links[0].from) + links[0].crossing_cost() + p(links[0].to, t);
    assert!(
        pelo_atalho < a_pe && dist(links[0].to, t) + 1.0 >= a_pe,
        "a fixtura: o atalho vence ({pelo_atalho} < {a_pe}) e a recta da ponta ao alvo não ({})",
        dist(links[0].to, t)
    );
    let r = match Plano::begin(&m, &mut Polyanya::new(), &q, s, t) {
        Err(r) => r,
        Ok(mut pl) => loop {
            if let Some(r) = pl.run(&m, &q, u64::MAX) {
                break r;
            }
        },
    };
    let (pts, hops, _) = r.expect("um caminho");
    let got = custo_com_atalhos(&m, &costs, &links, &pts, &hops);
    eprintln!(
        "a pé {a_pe:.3} · pelo atalho {pelo_atalho:.3} · o produto {got:.3} ({} saltos)",
        hops.len()
    );
    assert!(
        !hops.is_empty() && got <= pelo_atalho * (1.0 + 1e-9),
        "o produto perdeu o atalho: {got} (a pé {a_pe}, pelo atalho {pelo_atalho})"
    );
}

/// ⭐ (§28.1, H2) **A saída cedo vale com uma área barata LONGE** — um caminho em campo aberto, a estrada
/// num canto: a fase geral é a resposta, e a procura expande EXACTAMENTE os nós da tabela sem a estrada
/// (o CONTROLO, a mesma malha). Com a saída cedo pela cota global, a ponderada corria sempre por cima.
#[test]
fn com_a_estrada_longe_a_saida_cedo_poupa_a_ponderada() {
    let m = cena_com(1, &[], Some(([1.2, 18.8], 1.0)));
    let com = [1.0, 1.0, 1.0, 1.0, 1.0, 0.3];
    let sem = [1.0; 6];
    let mut s = Polyanya::new();
    let mut iguais = 0;
    for (a, z) in pares(1, &[&m], 30, None) {
        if dist(a, [1.2, 18.8]) + dist(z, [1.2, 18.8]) < dist(a, z) + 6.0 {
            continue;
        }
        let (cc, ec) = procura(&mut s, &m, &com, a, z);
        let (cs, es) = procura(&mut s, &m, &sem, a, z);
        // (O custo da geral soma-se troço a troço; o da uniforme é o comprimento: o último bit difere.)
        assert!(
            (cc - cs).abs() <= 1e-12 * cs.max(1.0),
            "{a:?} → {z:?}: o custo mudou ({cs} → {cc})"
        );
        assert_eq!(
            ec, es,
            "{a:?} → {z:?}: a ponderada correu ({es} → {ec} nós)"
        );
        iguais += 1;
    }
    assert!(
        iguais >= 10,
        "a população dos pares longe da estrada: {iguais}"
    );
}
