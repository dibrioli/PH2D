//! ⭐⭐ (W19, plano 30 §28.1–§28.2) **A COTA pela distância às áreas baratas** (`ph2d_nav::cota`) — o
//! heurístico da procura ponderada e o «alvo à vista». O juiz é o ORÁCULO ponderado (o grafo dos cantos
//! e Steiner, `ph2d_nav::oracle`), nunca a função sob teste; o CONTROLO é a cota global da W7
//! (`Polyanya::set_cota_global`), a de antes, que é o custo de hoje.

use ph2d_nav::agent::a_vista;
use ph2d_nav::cost::segment_cost;
use ph2d_nav::geom::dist;
use ph2d_nav::oracle::WeightedOracle;
use ph2d_nav::{NavMesh, Polyanya, Query, V2};

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
#[test]
fn uma_area_barata_longe_nao_pesa_na_procura() {
    let costs = [1.0, 4.0, 4.0, 4.0, 4.0, 0.3];
    let (mut sem, mut com, mut com_global) = (0u64, 0u64, 0u64);
    for seed in 1..=3u64 {
        let m_sem = cena_com(seed, &[], None);
        let m_com = cena_com(seed, &[], Some(([1.2, 18.8], 1.0)));
        let mut nova = Polyanya::new();
        let mut global = Polyanya::new();
        global.set_cota_global(true);
        for (a, z) in pares(seed, &[&m_sem, &m_com], 10, None) {
            sem += procura(&mut nova, &m_sem, &costs, a, z).1;
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
