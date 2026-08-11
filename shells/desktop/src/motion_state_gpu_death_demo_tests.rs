//! Gates da cena `=27` — e a SONDA que produziu os números da mensagem de anúncio.
//!
//! A regra do plano 89: *toda wave ganha cena com números MEDIDOS, e a sonda headless roda
//! ANTES de a mensagem ser escrita*. Nesta jornada essa regra já se pagou duas vezes numa
//! fixture minha.

use super::*;
use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// A população, quadro a quadro, do documento REAL da cena.
fn population(secs: f64) -> Vec<usize> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_death_demo_document(&mut doc, &reg).expect("a cena é bem tipada");
    let mut cook = Cook::new();
    let mut n = Vec::new();
    for k in 0..=((secs * 60.0) as u64) {
        let t = k as f64 / 60.0;
        let s = cook.cook(&doc.graph, &reg, sinks[0], t).expect("cozinha")[0]
            .as_stream()
            .clone();
        n.push(s.count());
        cook.advance_tick(&doc.graph, &reg, t).expect("avança");
    }
    n
}

/// **A cena é bem tipada e o sink é único** — o mínimo que separa uma cena de um documento
/// que o `validate` recusa na abertura.
#[test]
fn the_scene_builds() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_death_demo_document(&mut doc, &reg).expect("a cena é bem tipada");
    assert_eq!(sinks.len(), 1);
}

/// **A MORTE é a única autora: sem a fiação a tela esvaziaria e ficaria vazia.**
///
/// É o oráculo forte da cena, o mesmo truque da `=24`: com `rate = 0`, se o evento de morte
/// não desse à luz, a população cairia de [`SEEDS`] a **zero** ao fim da primeira vida e
/// nunca mais subiria. Que ela CRESÇA é a prova de que a fiação está viva.
#[test]
fn nothing_but_death_gives_birth_and_the_population_grows() {
    let n = population(6.0);
    assert_eq!(n[0], SEEDS as usize, "a cena abre com as três sementes");
    let late = *n.last().expect("cozinhou");
    assert!(
        late > SEEDS as usize,
        "aos 6 s a cascata já multiplicou as sementes; medido {late} (série {n:?})"
    );
    // E ela nunca zera no meio: uma geração morre no MESMO quadro em que a seguinte nasce.
    assert!(
        n.iter().all(|c| *c > 0),
        "a tela nunca esvazia — a morte de uma geração É o nascimento da próxima ({n:?})"
    );
}

/// **O PRIMEIRO estouro é EXATO, e é ele que o olho conta.**
///
/// As três sementes morrem no mesmo tique (a variância é inerte nelas — o `motion.grid` não
/// escreve `id`, ver o doc do módulo), então a 1ª geração é `SEEDS · BURST` **na mosca**, sem
/// barra frouxa: `3 → 6`, medido em t = 1,10 s. É a asserção mais afiada que esta cena
/// permite, e ela morre se o `burst` for ignorado (a população ficaria em 3) **ou** se os
/// filhos herdassem a idade do cadáver (a população voltaria a 3 no tique seguinte).
#[test]
fn the_first_burst_is_exactly_seeds_times_burst() {
    let n = population(2.0);
    let first = *n.last().expect("cozinhou");
    assert_eq!(
        first,
        (SEEDS * BURST) as usize,
        "aos 2 s a 1ª geração já estourou e a 2ª ainda não; série {n:?}"
    );
    // E o degrau é ÚNICO: de 3 para 6 sem passar por 4 nem 5 (elas morrem juntas).
    let steps: Vec<usize> = n
        .windows(2)
        .filter(|w| w[0] != w[1])
        .map(|w| w[1])
        .collect();
    assert_eq!(
        steps,
        vec![(SEEDS * BURST) as usize],
        "um degrau só, e ele é o estouro: {n:?}"
    );
}

/// **E a cascata SEGUE multiplicando** — a 1ª geração não é um caso isolado.
///
/// Sem este gate o irmão acima passaria com um evento de morte que só disparasse UMA vez.
#[test]
fn the_cascade_keeps_multiplying() {
    let n = population(6.0);
    let late = *n.last().expect("cozinhou");
    // MEDIDO (`probe_population`): 41 aos 6 s. A barra fica bem abaixo porque o número exato
    // depende do hash das vidas, e o que ela tem de pegar é *"parou na 1ª geração"*.
    assert!(
        late > (SEEDS * BURST * BURST) as usize,
        "aos 6 s já passamos da 3ª geração ({} elementos); medido {late}",
        (SEEDS * BURST * BURST) as usize
    );
}

/// **AS IRMÃS SE SEPARAM — o report do Enio, virado gate.**
///
/// *"não se dividem, as filhas ficam juntas como uma só"* (2026-08-10), e a medição deu o
/// mecanismo: as duas nasciam com `P` **e** `vel` bit-idênticos, e **toda força deste
/// catálogo é função da POSIÇÃO** — `curl(P)` dá a duas partículas no mesmo ponto a mesma
/// aceleração, para sempre. Não era afinação: era impossibilidade.
///
/// O gate mede a única coisa que o olho vê — **quantos pontos distintos há na tela** — meio
/// segundo depois do 1º estouro. Com o impulso são [`SEEDS`]·[`BURST`]; sem ele, [`SEEDS`],
/// porque cada par viaja empilhado.
#[test]
fn the_siblings_of_a_burst_fly_apart() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_death_demo_document(&mut doc, &reg).expect("cena");
    let mut cook = Cook::new();
    let mut distinct = 0usize;
    for k in 0..=100u64 {
        let t = k as f64 / 60.0;
        let s = cook.cook(&doc.graph, &reg, sinks[0], t).expect("cozinha")[0]
            .as_stream()
            .clone();
        if k == 100 {
            // Meio segundo depois do estouro (medido em k = 66).
            let p = match s.get("P") {
                Some(ph2d_nodegraph::attr::Column::Vec2(v)) => v.clone(),
                _ => vec![],
            };
            assert_eq!(p.len(), (SEEDS * BURST) as usize, "as seis da 1ª geração");
            // Distintos a 1 mm — bem abaixo da separação medida e bem acima do ruído.
            let mut seen: Vec<[f32; 2]> = Vec::new();
            for q in &p {
                if !seen
                    .iter()
                    .any(|r| (r[0] - q[0]).abs() < 1e-3 && (r[1] - q[1]).abs() < 1e-3)
                {
                    seen.push(*q);
                }
            }
            distinct = seen.len();
        }
        cook.advance_tick(&doc.graph, &reg, t).expect("avança");
    }
    assert_eq!(
        distinct,
        (SEEDS * BURST) as usize,
        "cada irmã é um ponto próprio na tela; com `burst_speed = 0` seriam {} \
         (cada par empilhado), que é exatamente o que o Enio viu",
        SEEDS as usize
    );
}

/// **A SONDA** — imprime a população quadro a quadro, de onde saem os números do anúncio.
///
/// `cargo test -p ph2d-host-desktop --lib death_demo::tests::probe -- --ignored --nocapture`
#[test]
#[ignore]
fn probe_population() {
    let n = population(8.0);
    for (k, c) in n.iter().enumerate() {
        if k % 15 == 0 {
            eprintln!("t={:.2}s  n={c}", k as f64 / 60.0);
        }
    }
    eprintln!("série completa: {n:?}");
}

/// **SONDA do report do Enio** (*"não se dividem, as filhas ficam juntas como uma só"*):
/// imprime a posição e a velocidade de cada elemento nos tiques em torno do 1º estouro.
#[test]
#[ignore]
fn probe_siblings() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_death_demo_document(&mut doc, &reg).expect("cena");
    let mut cook = Cook::new();
    for k in 0..=150u64 {
        let t = k as f64 / 60.0;
        let s = cook.cook(&doc.graph, &reg, sinks[0], t).expect("cozinha")[0]
            .as_stream()
            .clone();
        if (66..=70).contains(&k) || k == 90 || k == 120 || k == 150 {
            let p = match s.get("P") {
                Some(ph2d_nodegraph::attr::Column::Vec2(v)) => v.clone(),
                _ => vec![],
            };
            let vel = match s.get("vel") {
                Some(ph2d_nodegraph::attr::Column::Vec2(v)) => v.clone(),
                _ => vec![],
            };
            eprintln!("k={k} n={}", s.count());
            for (i, (pp, vv)) in p.iter().zip(vel.iter()).enumerate().take(6) {
                eprintln!(
                    "   [{i}] P=({:.4},{:.4})  vel=({:.4},{:.4})",
                    pp[0], pp[1], vv[0], vv[1]
                );
            }
        }
        cook.advance_tick(&doc.graph, &reg, t).expect("avança");
    }
}
