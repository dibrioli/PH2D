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

/// **A ZONA** — a população da simulação. ⚠️ Desde 07/10 a saída carimba uma estrela em cada
/// centelha, e a estrela (`source.shape`) só existe com as membranas da ponte publicadas; a lei
/// que estes gates medem é a do nascimento, que vive na zona (o que o carimbo recebe).
fn zona(doc: &MotionDoc, sink: ph2d_nodegraph::graph::NodeId) -> ph2d_nodegraph::graph::NodeId {
    let carimbo = doc.graph.input_edge(sink, 0).expect("o sink tem entrada").0;
    doc.graph
        .input_edge(carimbo, 1)
        .expect("o carimbo tem pontos")
        .0
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
        let s = cook
            .cook(&doc.graph, &reg, zona(&doc, sinks[0]), t)
            .expect("cozinha")[0]
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
        let s = cook
            .cook(&doc.graph, &reg, zona(&doc, sinks[0]), t)
            .expect("cozinha")[0]
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
/// `cargo test -p ph2d-app-motion --lib death_demo::tests::probe -- --ignored --nocapture`
#[test]
#[ignore = "sonda: imprime, não afirma"]
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
#[ignore = "sonda: imprime, não afirma"]
fn probe_siblings() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_death_demo_document(&mut doc, &reg).expect("cena");
    let mut cook = Cook::new();
    for k in 0..=150u64 {
        let t = k as f64 / 60.0;
        let s = cook
            .cook(&doc.graph, &reg, zona(&doc, sinks[0]), t)
            .expect("cozinha")[0]
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

/// Pixels acesos por estrela abaixo dos quais a centelha não se LÊ como estrela. Uma estrela de
/// seis pontas de raio `r` px acende `~1,5·r²`: `~18` a `0,07` (o Bug #14) e `~175` a `0,22`.
const PIXELS_POR_ESTRELA_MIN: usize = 80;

/// ⛔ **Bug #14 — as estrelas da SAÍDA da zona LÊEM-SE no ecrã**, pela porta do produto: o quadro
/// cozido por `coze_o_quadro`, a placa de formas decidida com a MESMA pergunta do host
/// (`fase_vector_bands.rs`: o buffer do dispositivo se o cozimento é vivo, senão as
/// `vector_instances`) e desenhada à escala da câmera do produto (`48 px` por unidade).
///
/// ⚠️ O defeito não era perda nenhuma: as duas rotas entregam as estrelas em todo quadro. Era o
/// TAMANHO (`~3 px`), e a sonda `PH2D_PAN_DIAG` cega à rota do dispositivo que o «confirmou».
#[test]
#[ignore = "placa: requires a GPU adapter; run with --ignored on a dev machine"]
fn the_sparks_draw_legible_stars_on_both_routes() {
    use crate::motion_shape_placa::PlacaDeFormas;
    use crate::motion_shape_placa::gpu_tests::{LADO, compara, le_a_camada};
    let Ok(gpu) = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None) else {
        return;
    };
    let k = 48.0;
    let camara = ph2d_vector::Affine::translate((f64::from(LADO) * 0.5, f64::from(LADO) * 0.5))
        * ph2d_vector::Affine::scale_non_uniform(k, -k);
    for placa in [false, true] {
        let mut m = crate::motion_state::MotionState::new();
        m.gpu_enabled = placa;
        m.sinks = build_gpu_death_demo_document(&mut m.doc, &m.registry).expect("a cena");
        let mut playhead = ph2d_core::Playhead::new(1.0 / 60.0);
        for quadro in 1..=3u64 {
            playhead.seek(quadro as f64 / 60.0);
            crate::motion_bridge::quadro::coze_o_quadro(&mut m, &gpu, &playhead, 1.0 / 60.0);
            let rota = m.route_said.unwrap_or("");
            assert_eq!(rota.starts_with("device"), placa, "a rota tomada: {rota}");
            let mut p = PlacaDeFormas::default();
            let dispositivo = m.gpu_cook.formas().filter(|_| m.gpu_live);
            let n = dispositivo.map_or(m.pump.vector_instances.len(), |f| f.len() as usize);
            let decidiu = match dispositivo {
                Some(f) => p.decide_do_dispositivo(
                    true,
                    f.len(),
                    &m.formas_no_dispositivo,
                    &m.shape_store,
                    &mut m.placa_geometrias,
                    camara,
                ),
                None => p.decide(
                    true,
                    &m.pump.vector_instances,
                    &m.shape_store,
                    &mut m.placa_geometrias,
                    camara,
                ),
            };
            assert!(
                decidiu,
                "placa={placa} q={quadro}: a placa recusou as estrelas"
            );
            let buffer = m
                .gpu_cook
                .formas()
                .filter(|_| m.gpu_live)
                .map(|f| f.buffer());
            p.desenha(&gpu, (LADO, LADO), &m.placa_geometrias, buffer)
                .expect("a placa desenha");
            let camada = le_a_camada(&gpu, &p);
            let acesos = compara(&camada, &camada).2;
            assert_eq!(n, 3, "as três centelhas antes do 1.º estouro");
            assert!(
                acesos >= n * PIXELS_POR_ESTRELA_MIN,
                "placa={placa} q={quadro}: {acesos} pixels para {n} estrelas — uma estrela de \
                 ~{} px por cópia não se lê",
                acesos / n.max(1)
            );
        }
    }
}

/// Metade da largura e da altura do que a câmera por omissão mostra, em mundo: `height_world = 10`
/// num recorte `16:9`.
const MEIA_CAMERA: [f32; 2] = [5.0 * 16.0 / 9.0, 5.0];

/// Quantas das posições caem dentro da câmera por omissão (centrada na origem).
fn a_vista(p: &[[f32; 2]]) -> usize {
    p.iter()
        .filter(|q| q[0].abs() <= MEIA_CAMERA[0] && q[1].abs() <= MEIA_CAMERA[1])
        .count()
}

/// As posições da zona nos tiques `marcas`, do documento `doc`.
fn posicoes_em(
    doc: &MotionDoc,
    sink: ph2d_nodegraph::graph::NodeId,
    marcas: &[u64],
) -> Vec<Vec<[f32; 2]>> {
    let reg = registry();
    let mut cook = Cook::new();
    let mut fora = Vec::new();
    for k in 0..=marcas.iter().copied().max().unwrap_or(0) {
        let t = k as f64 / 60.0;
        let s = cook
            .cook(&doc.graph, &reg, zona(doc, sink), t)
            .expect("cozinha")[0]
            .as_stream()
            .clone();
        if marcas.contains(&k) {
            fora.push(match s.get("P") {
                Some(ph2d_nodegraph::attr::Column::Vec2(v)) => v.clone(),
                _ => vec![],
            });
        }
        cook.advance_tick(&doc.graph, &reg, t).expect("avança");
    }
    fora
}

/// ⭐ **A CASCATA FICA À VISTA** (doc 123 §7): a `2`, `4`, `6` e `8 s` pelo menos `90 %` das
/// centelhas estão dentro da câmera por omissão. Até 08/10 o vento e o `curl` levavam-na para
/// cima e para a direita, e aos `6 s` a tela estava VAZIA (`0/41`). O CONTROLO é a população a
/// crescer — uma cena que retivesse tudo por não nascer ninguém passaria a primeira metade.
#[test]
fn the_cascade_stays_in_view() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_death_demo_document(&mut doc, &reg).expect("cena");
    let fotos = posicoes_em(&doc, sinks[0], &[120, 240, 360, 480]);
    for (k, p) in fotos.iter().enumerate() {
        let (v, n) = (a_vista(p), p.len());
        assert!(
            v * 10 >= n * 9,
            "aos {} s só {v} de {n} centelhas estão à vista — a cascata sai da câmera",
            2 * (k + 1)
        );
    }
    assert!(
        fotos[3].len() > fotos[0].len() * 4,
        "controlo: a cascata tem de CRESCER ({} → {})",
        fotos[0].len(),
        fotos[3].len()
    );
}

/// **SONDA da cascata à vista** (doc 123 §7): a população, a caixa das posições da zona e quantas
/// caem DENTRO da câmera por omissão, a cada meio segundo.
#[test]
#[ignore = "sonda: onde a cascata da =27 esta' a cada meio segundo"]
fn sonda_onde_a_cascata_esta() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_gpu_death_demo_document(&mut doc, &reg).expect("cena");
    let marcas: Vec<u64> = (0..=20).map(|i| i * 30).collect();
    for (k, p) in posicoes_em(&doc, sinks[0], &marcas).iter().enumerate() {
        let (mut x0, mut x1, mut y0, mut y1) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for q in p {
            x0 = x0.min(q[0]);
            x1 = x1.max(q[0]);
            y0 = y0.min(q[1]);
            y1 = y1.max(q[1]);
        }
        eprintln!(
            "t={:.1}s n={} dentro={} x=[{x0:.2},{x1:.2}] y=[{y0:.2},{y1:.2}]",
            k as f64 * 0.5,
            p.len(),
            a_vista(p)
        );
    }
}

/// **SONDA do desenho da `=27` à vista** (doc 123 §7): o vento, a atração e o arrasto da cena
/// variados nos PRÓPRIOS nós dela — quantas centelhas a câmera vê a `2`, `4`, `6` e `8 s`. Uma
/// rodada, todas as variantes; `(1,4 · 0 · 0)` é a cena de antes de 08/10.
#[test]
#[ignore = "sonda: as variantes da =27 a vista"]
fn sonda_a_cascata_a_vista_por_variante() {
    let reg = registry();
    let mut variantes = vec![(1.4_f32, 0.0_f32, 0.0_f32)];
    for vento in [1.4, 0.7] {
        for forca in [2.0, 4.0, 6.0] {
            for arrasto in [0.0, 0.5, 1.0] {
                variantes.push((vento, forca, arrasto));
            }
        }
    }
    eprintln!("vento forca arrasto | dentro/n a 2s 4s 6s 8s");
    for (vento, forca, arrasto) in variantes {
        let mut doc = MotionDoc::default();
        let sinks = build_gpu_death_demo_document(&mut doc, &reg).expect("cena");
        for (tipo, param, v) in [
            ("force.wind", "strength", vento),
            ("force.attractor", "strength", forca),
            ("force.drag", "coefficient", arrasto),
        ] {
            let id = doc
                .graph
                .nodes()
                .iter()
                .find(|n| n.type_name == tipo)
                .map(|n| n.id)
                .expect(tipo);
            doc.graph.set_param(id, param, v);
        }
        let linha: String = posicoes_em(&doc, sinks[0], &[120, 240, 360, 480])
            .iter()
            .map(|p| format!(" {}/{}", a_vista(p), p.len()))
            .collect();
        eprintln!("{vento:>5} {forca:>5} {arrasto:>7} |{linha}");
    }
}
