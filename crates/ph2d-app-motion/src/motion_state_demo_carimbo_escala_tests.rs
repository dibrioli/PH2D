//! **SONDA — o carimbo com `point_scale > 0` na placa** (aberto 1 do handoff LÍNGUA; doc 120 §8.3
//! W1(b).1 fez só `point_scale = 0`). A rota de ANTES (`A`: o kernel registado sem a variante, com a
//! recusa `point_scale <= 0`) contra a de HOJE (`B`), no MESMO processo: a rota medida pela porta do
//! produto e o quadro inteiro (`o_quadro` + a espera da placa) intercalado em blocos rodados, resumo =
//! o MÍNIMO ([MEDIR_VELOCIDADE](../../../docs/DevOps/MEDIR_VELOCIDADE.md)).
//!
//! `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --profile smoke --lib \
//!     sonda_o_carimbo_com_escala -- --ignored --nocapture --test-threads=1`
//! (`PH2D_SONDA_CENAS=132,137`, `PH2D_SONDA_BLOCO`, `PH2D_SONDA_RODADAS`).

use super::medida::{Cena, mede, monta_demo};
use crate::motion_state::MotionState;
use ph2d_nodegraph::gpu::{ApplicableFn, GpuKernel, KernelResolver};
use ph2d_nodegraph::node::NodeTypeId;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

const CARIMBO: NodeTypeId = NodeTypeId::of("motion.duplicator");
/// As cenas que o handoff dá como caídas pelo carimbo, e as quatro fronteiras por confirmar.
const CENAS: &[&str] = &[
    "132", "136", "137", "139", "120", "111", "112", "119", "125", "141", "62", "131", "98", "134",
    "51",
];

static RECUSA_DE_HOJE: OnceLock<ApplicableFn> = OnceLock::new();

fn recusa_a_escala_do_ponto(p: &dyn Fn(&str) -> f32) -> bool {
    RECUSA_DE_HOJE.get().is_some_and(|f| f(p)) && p("point_scale") <= 0.0
}

/// A rota `A`: o kernel registado, sem variante, com a recusa de antes.
pub(super) fn rota_de_antes(m: &mut MotionState) {
    let k = *m
        .registry
        .gpu_kernel(CARIMBO)
        .expect("o carimbo regista kernel");
    let _ = RECUSA_DE_HOJE.set(k.applicable.expect("o carimbo declara recusas"));
    m.registry.register_gpu_kernel(
        CARIMBO,
        GpuKernel {
            variant_by_param: None,
            applicable: Some(recusa_a_escala_do_ponto),
            ..k
        },
    );
}

/// Multiplica o lado de toda grelha da cena por `k` (cortado no tecto de lado): o MESMO grafo com
/// `k²` vezes os pontos, para medir o tecto e não o custo fixo do despacho.
pub(super) fn amplia(m: &mut MotionState, k: f32) {
    let grelhas: Vec<_> = m
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "motion.grid")
        .map(|n| n.id)
        .collect();
    let teto = ph2d_nodegraph::node::LADO_MAX_DE_GRELHA as f32;
    for g in grelhas {
        for p in ["rows", "cols"] {
            let v = m
                .doc
                .graph
                .node_param_overrides(g)
                .and_then(|o| o.get(p).copied())
                .unwrap_or(3.0);
            m.doc.graph.set_param(g, p, (v * k).min(teto));
        }
    }
    // `PH2D_SONDA_RAIO_POR_ESCALA`: o raio escrito por um `Drive` no canal `Custom` (a `=137`: `0,8·|P|`)
    // divide-se por `k` — a galáxia cresce `k` vezes e o maior disco fica o de `×1`. Sem isto o
    // `amplia` mede contacto denso (os toques crescem com `N²`, doc 120 §8.3.4).
    if std::env::var("PH2D_SONDA_RAIO_POR_ESCALA").is_ok() {
        let drives: Vec<_> = m
            .doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == "motion.drive")
            .map(|n| n.id)
            .collect();
        for d in drives {
            let o = m.doc.graph.node_param_overrides(d);
            let p = |q: &str| o.and_then(|o| o.get(q).copied());
            if p("channel") == Some(9.0)
                && let Some(s) = p("scale")
            {
                m.doc.graph.set_param(d, "scale", s / k);
            }
        }
    }
}

/// Porque é que cada carimbo que continua fronteira o é: os três params das recusas e o cartão
/// que alimenta os pontos.
fn porque_o_carimbo_fica(m: &mut MotionState, t: f64) -> Vec<String> {
    let plano = crate::motion_bridge::gpu::plano_do_produto(m, t);
    let g = &m.doc.graph;
    let tipo = |n| g.node(n).map(|i| i.type_name.clone()).unwrap_or_default();
    plano
        .boundaries
        .iter()
        .filter(|(n, _)| tipo(*n) == "motion.duplicator")
        .map(|(n, _)| {
            let o = g.node_param_overrides(*n);
            let p = |k: &str| o.and_then(|o| o.get(k).copied()).unwrap_or(0.0);
            let pontos = g.input_edge(*n, 1).map(|(s, _, pre)| (tipo(s), pre));
            let forma = g.input_edge(*n, 0).map(|(s, _, pre)| (tipo(s), pre));
            format!(
                "pick {} transfer {} point_scale {} · forma {forma:?} · pontos {pontos:?}",
                p("pick"),
                p("transfer"),
                p("point_scale")
            )
        })
        .collect()
}

pub(super) fn numero(var: &str, omissao: usize) -> usize {
    std::env::var(var)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(omissao)
}

#[test]
#[ignore = "sonda: corra à mão"]
fn sonda_o_carimbo_com_escala() {
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("sem adapter");
    let (bloco, rodadas) = (
        numero("PH2D_SONDA_BLOCO", 20),
        numero("PH2D_SONDA_RODADAS", 7),
    );
    let so: Option<Vec<String>> = std::env::var("PH2D_SONDA_CENAS")
        .ok()
        .map(|v| v.split(',').map(str::to_string).collect());
    let escalas: Vec<f32> = std::env::var("PH2D_SONDA_ESCALAS")
        .unwrap_or_else(|_| "1,4,11".into())
        .split(',')
        .filter_map(|v| v.parse().ok())
        .collect();
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    eprintln!(
        "CARIMBO placa «{}» · bloco {bloco} · {rodadas} rodadas · load {}",
        gpu.adapter.get_info().name,
        carga().trim()
    );
    let dt = 1.0 / 60.0;
    let quadro = |m: &mut MotionState, ph: &mut ph2d_core::Playhead| -> Duration {
        ph.advance_ticks(1);
        let t0 = Instant::now();
        crate::motion_bridge::quadro::o_quadro(
            m,
            Some(&gpu),
            ph,
            dt,
            crate::motion_bridge::quadro::MaoDoEditor::default(),
        );
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        t0.elapsed()
    };
    let ms = |d: Duration| d.as_secs_f64() * 1e3;
    for &n in CENAS {
        if so.as_ref().is_some_and(|f| !f.iter().any(|x| x == n)) {
            continue;
        }
        let cena = Cena::Demo(n.to_string());
        let (Some(a), Some(b)) = (mede(&gpu, &cena, rota_de_antes), mede(&gpu, &cena, |_| {}))
        else {
            eprintln!("={n}: a cena não existe");
            continue;
        };
        eprintln!(
            "={n} ROTA\n  A {:?} · fronteiras {:?}\n    no processador {:?}\n  B {:?} · fronteiras {:?}\n    no processador {:?}",
            a.razoes, a.fronteiras, a.no_processador, b.razoes, b.fronteiras, b.no_processador
        );
        for &k in &escalas {
            // O quadro inteiro, A e B lado a lado no MESMO relógio da cena.
            let mut va = monta_demo(n).expect("a cena existe");
            rota_de_antes(&mut va);
            amplia(&mut va, k);
            let mut vb = monta_demo(n).expect("a cena existe");
            amplia(&mut vb, k);
            let mut ph = [ph2d_core::Playhead::new(dt), ph2d_core::Playhead::new(dt)];
            for p in &mut ph {
                p.play();
            }
            let (pa, pb) = ph.split_at_mut(1);
            for _ in 0..30 {
                quadro(&mut va, &mut pa[0]);
                quadro(&mut vb, &mut pb[0]);
            }
            if k == 1.0 {
                for r in porque_o_carimbo_fica(&mut vb, pb[0].time()) {
                    eprintln!("  B carimbo fronteira: {r}");
                }
            }
            let (mut ta, mut tb) = (Vec::new(), Vec::new());
            for r in 0..rodadas {
                let bloco_de = |m: &mut MotionState, p: &mut ph2d_core::Playhead| {
                    (0..bloco).map(|_| quadro(m, p)).sum::<Duration>() / bloco as u32
                };
                if r % 2 == 0 {
                    ta.push(bloco_de(&mut va, &mut pa[0]));
                    tb.push(bloco_de(&mut vb, &mut pb[0]));
                } else {
                    tb.push(bloco_de(&mut vb, &mut pb[0]));
                    ta.push(bloco_de(&mut va, &mut pa[0]));
                }
            }
            let resumo = |v: &mut Vec<Duration>| {
                v.sort();
                (ms(v[0]), ms(v[v.len() / 2]))
            };
            let ((a_min, a_med), (b_min, b_med)) = (resumo(&mut ta), resumo(&mut tb));
            let elementos = vb.gpu_cook.instances().map_or(0, |i| i.len());
            eprintln!(
                "={n} ×{k} QUADRO  A min {a_min:.3} ms (med {a_med:.3}) · B min {b_min:.3} ms (med {b_med:.3}) · B/A {:.3} · {elementos} instâncias na placa (B)",
                b_min / a_min
            );
        }
    }
    eprintln!("load no fim {}", carga().trim());
}

/// As fases que a [`sonda_as_fases_do_quadro_hibrido`] imprime, na ordem.
pub(super) const FASES: [&str; 21] = [
    "tomadas",
    "preparo",
    "cercas",
    "plano",
    "marcha",
    "dirigidos",
    "cook_many",
    "costuras",
    "encode",
    "estagios",
    "saidas",
    "tap",
    "finish",
    "submit",
    "resto_cook_gpu",
    "depois",
    "resto_quadro",
    "o_quadro",
    "espera",
    "formas",
    "quadro",
];

/// Um quadro lido pelo relógio de fases, em µs, mais `(bytes, enviadas, reaproveitadas,
/// despachos, grupos criados, grupos reaproveitados)`.
pub(super) fn le_as_fases(
    m: &MotionState,
    espera: Duration,
    formas: Duration,
) -> ([f64; 21], [f64; 6]) {
    let f = m.fases;
    let us = |d: Duration| d.as_secs_f64() * 1e6;
    let hibrida = m.route_said.is_some_and(|r| r.contains("HIBRIDO"));
    let marcha_no_cook_gpu = if hibrida { f.marcha } else { Duration::ZERO };
    let marcha_na_bomba = f.marcha - marcha_no_cook_gpu;
    let dentro = f.cercas + f.plano + marcha_no_cook_gpu + f.dirigidos + f.cook_many;
    let resto_cook_gpu = f.cook_gpu.saturating_sub(dentro);
    let fora = f.tomadas + f.preparo + f.cook_gpu + marcha_na_bomba + f.depois;
    let c = f.cozimento;
    (
        [
            us(f.tomadas),
            us(f.preparo),
            us(f.cercas),
            us(f.plano),
            us(f.marcha),
            us(f.dirigidos),
            us(f.cook_many),
            us(c.costuras),
            us(c.encode),
            us(c.estagios),
            us(c.saidas),
            us(c.tap),
            us(c.encode.saturating_sub(c.estagios + c.saidas + c.tap)),
            us(c.submit),
            us(resto_cook_gpu),
            us(f.depois),
            us(f.total.saturating_sub(fora)),
            us(f.total),
            us(espera),
            us(formas),
            us(f.total + espera),
        ],
        [
            c.bytes_enviados as f64,
            f64::from(c.enviadas),
            f64::from(c.reaproveitadas),
            f64::from(c.despachos),
            f64::from(c.grupos_criados),
            f64::from(c.grupos_reaproveitados),
        ],
    )
}

/// As rotas da [`sonda_as_fases_do_quadro_hibrido`]: `A` o carimbo recusado (a de antes) · `B` hoje ·
/// `S1` hoje com UMA varredura do passe em cada sink · `SO` hoje sem a ordenação dos baldes do passe
/// · `C` hoje sem colisor (o `Drive` que escreve `collider` passa a escrever o nome do controlo:
/// nenhum passe arma). ⚠️ As saídas procuram-se pelo TIPO: `m.sinks` só se enche no 1.º quadro.
fn arma_a_rota(m: &mut MotionState, rota: &str) {
    match rota {
        "A" => rota_de_antes(m),
        "SO" => m.gpu_cook.passe_sem_ordenar(true),
        "S1" => {
            for s in saidas(m) {
                m.doc
                    .graph
                    .set_param(s, ph2d_eval_motion::SINK_COLLIDE_ITERATIONS_PARAM, 1.0);
            }
        }
        "C" => {
            let nomeados: Vec<_> = m
                .doc
                .graph
                .node_text_params()
                .iter()
                .filter(|(_, t)| t.get("column").is_some_and(|c| c == "collider"))
                .map(|(n, _)| *n)
                .collect();
            for n in nomeados {
                m.doc.graph.set_text_param(n, "column", "colisor");
            }
        }
        _ => {}
    }
}

fn saidas(m: &MotionState) -> Vec<ph2d_nodegraph::graph::NodeId> {
    m.doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "motion.output")
        .map(|n| n.id)
        .collect()
}

/// **O censo do passe** da cena `n` a `×k`, pela rota da CPU depois de 30 quadros, sobre a corrente
/// que chega a cada saída: os pares que se TOCAM (círculos de alcance, pares ordenados, força
/// bruta) contra os candidatos da grelha da CPU; a grelha da PLACA (lado `2·alcance_max`, a chave
/// `grid::cpu_bucket`) em baldes ocupados, o maior e `Σk²` (o trabalho da ordenação por inserção);
/// e o passe inteiro da CPU (16 varreduras) no relógio.
fn censo_do_passe(n: &str, k: f32, dt: f64) {
    use ph2d_nodegraph::attr::Column;
    let Some(mut c) = monta_demo(n) else { return };
    amplia(&mut c, k);
    c.gpu_enabled = false;
    let alimenta: Vec<_> = saidas(&c)
        .iter()
        .filter_map(|&s| c.doc.graph.input_edge(s, 0).map(|(f, _, _)| (s, f)))
        .collect();
    let mut ph = ph2d_core::Playhead::new(dt);
    ph.play();
    for _ in 0..30 {
        ph.advance_ticks(1);
        crate::motion_bridge::quadro::o_quadro(
            &mut c,
            None,
            &mut ph,
            dt,
            crate::motion_bridge::quadro::MaoDoEditor::default(),
        );
    }
    // O `o_quadro` rearma as tomadas dele: um tique a mais pela bomba, com as nossas e sem o passe.
    #[expect(clippy::cast_possible_truncation, reason = "sonda: tiques pequenos")]
    let tique = (ph.time() / dt).round() as u64 + 1;
    let t = tique as f64 * dt;
    crate::motion_externals::publish_all(&mut c, t);
    c.pump
        .set_taps(&alimenta.iter().map(|(_, f)| *f).collect::<Vec<_>>());
    c.pump.mark_dirty();
    c.pump.set_separa_o_desenho(false);
    let sinks: Vec<_> = alimenta.iter().map(|(s, _)| *s).collect();
    let _ = c.pump.pump(
        &c.doc.graph,
        &c.registry,
        &sinks,
        tique,
        t,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
    );
    for (saida, f) in alimenta {
        let Some(s) = c
            .pump
            .tap_streams()
            .iter()
            .find(|(no, _)| *no == f)
            .map(|(_, s)| s.clone())
        else {
            eprintln!("CENSO ={n} ×{k}: sem tomada do nó {f:?}");
            continue;
        };
        let (Some(cols), Some(Column::Vec2(p))) = (ph2d_contact::colisores(&s), s.get("P")) else {
            eprintln!("CENSO ={n} ×{k} saída {saida:?}: não declara colisor");
            continue;
        };
        let pecas: Vec<(usize, f32)> = cols
            .iter()
            .enumerate()
            .filter_map(|(i, c)| c.map(|c| (i, c.alcance())))
            .filter(|(_, a)| *a > 0.0)
            .collect();
        let alc_max = pecas.iter().fold(0.0f32, |a, (_, b)| a.max(*b));
        let mut toques = 0usize;
        for (x, &(i, ai)) in pecas.iter().enumerate() {
            for &(j, aj) in &pecas[x + 1..] {
                let d = [p[i][0] - p[j][0], p[i][1] - p[j][1]];
                if d[0] * d[0] + d[1] * d[1] < (ai + aj) * (ai + aj) {
                    toques += 2;
                }
            }
        }
        let (cand_cpu, grandes) = ph2d_contact::candidatos_e_grandes(&cols, p, &[]);
        #[expect(clippy::cast_possible_truncation, reason = "sonda: peças < 2³²")]
        let nb = ph2d_gpu_cook::grid::num_buckets(p.len() as u32);
        let mut baldes = vec![0u64; nb as usize];
        for &(i, _) in &pecas {
            baldes[ph2d_gpu_cook::grid::cpu_bucket(p[i], 2.0 * alc_max, nb) as usize] += 1;
        }
        let ocupados = baldes.iter().filter(|&&b| b > 0).count();
        let maior = baldes.iter().max().copied().unwrap_or(0);
        let soma_k2: u64 = baldes.iter().map(|b| b * b).sum();
        // As grelhas da C2 (doc 120 §8.3.5), por CÉLULA verdadeira (sem o hash): os cursores que cada
        // peça visitaria e os candidatos — a de hoje (3×3 de lado `2R`), a fixa 5×5 de lado `R` e a
        // ADAPTATIVA (lado `R`, o quadrado `r_i + R` à volta de cada peça).
        let grelha = |lado: f32| {
            let mut c: std::collections::HashMap<(i32, i32), u64> = Default::default();
            for &(i, _) in &pecas {
                #[expect(clippy::cast_possible_truncation, reason = "sonda")]
                let k = (
                    (p[i][0] / lado).floor() as i32,
                    (p[i][1] / lado).floor() as i32,
                );
                *c.entry(k).or_default() += 1;
            }
            c
        };
        let conta = |g: &std::collections::HashMap<(i32, i32), u64>,
                     lado: f32,
                     alcance: &dyn Fn(f32) -> f32| {
            let (mut cursores, mut cand) = (0u64, 0u64);
            for &(i, a) in &pecas {
                let r = alcance(a);
                #[expect(clippy::cast_possible_truncation, reason = "sonda")]
                let (x0, x1, y0, y1) = (
                    ((p[i][0] - r) / lado).floor() as i32,
                    ((p[i][0] + r) / lado).floor() as i32,
                    ((p[i][1] - r) / lado).floor() as i32,
                    ((p[i][1] + r) / lado).floor() as i32,
                );
                for x in x0..=x1 {
                    for y in y0..=y1 {
                        cursores += 1;
                        cand += g.get(&(x, y)).copied().unwrap_or(0);
                    }
                }
                cand -= 1;
            }
            (cursores, cand)
        };
        let g2 = grelha(2.0 * alc_max);
        let g1 = grelha(alc_max);
        let n_p = pecas.len().max(1) as f64;
        // Hoje: a célula da peça ± 1 (o quadrado `[c−1, c+1]` de lado `2R`).
        let hoje = conta(&g2, 2.0 * alc_max, &|_| 2.0 * alc_max * 0.999_999);
        let fixa = conta(&g1, alc_max, &|_| 2.0 * alc_max);
        let adapt = conta(&g1, alc_max, &|a| a + alc_max);
        // A de hoje com os cursores MASCARADOS: só as células que o quadrado `r_i + R` alcança.
        let mascara = conta(&g2, 2.0 * alc_max, &|a| a + alc_max);
        #[expect(clippy::cast_precision_loss, reason = "sonda")]
        let f = |(c, k): (u64, u64)| {
            format!(
                "{:.1} cursores · {:.1} candidatos",
                c as f64 / n_p,
                k as f64 / n_p
            )
        };
        eprintln!(
            "GRELHAS ={n} ×{k} por peça · hoje 3×3@2R: {} · fixa 5×5@R: {} · adaptativa r+R @R: {} · máscara r+R @2R: {} · toques {:.1}",
            f(hoje),
            f(fixa),
            f(adapt),
            f(mascara),
            toques as f64 / n_p
        );
        let t = Instant::now();
        let (_, r) = ph2d_contact::passe::separa_com_relatorio(&s, 16);
        let cpu_ms = t.elapsed().as_secs_f64() * 1e3;
        #[expect(clippy::cast_precision_loss, reason = "sonda")]
        let por = |x: usize| x as f64 / pecas.len().max(1) as f64;
        eprintln!(
            "CENSO ={n} ×{k} saída {saida:?}: {} peças · alcance_max {alc_max:.3} · TOQUES {toques} ({:.1} por peça) · candidatos CPU {cand_cpu} ({:.1} por peça, {grandes} grandes) · baldes da placa {ocupados}/{nb} ocupados, o maior {maior}, Σk² {soma_k2} · passe da CPU {cpu_ms:.1} ms ({} varreduras, {} candidatos)",
            pecas.len(),
            por(toques),
            por(cand_cpu),
            r.varreduras,
            r.candidatos
        );
    }
}

/// **SONDA — ONDE está o custo fixo da rota híbrida** (doc 120 §8.3.1, o aberto do custo fixo): a
/// MESMA cena por cada rota de [`arma_a_rota`] (`PH2D_SONDA_ROTAS`, omissão `A,B`), intercaladas em
/// blocos rodados, e por cada fase do quadro o MÍNIMO das médias de bloco (mediana ao lado). O
/// `quadro` é o da sonda irmã (`o_quadro` + a espera da placa); o passe de formas
/// (`decide_o_quadro`, da shell) cronometra-se À PARTE, depois da espera. Com `PH2D_SONDA_ESCALAS`
/// os casos são `PH2D_SONDA_CENAS × escalas`; o `Relatorio` do passe imprime-se por rota.
///
/// `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test -p ph2d-app-motion --profile smoke --lib \
///     sonda_as_fases_do_quadro_hibrido -- --ignored --nocapture --test-threads=1`
#[test]
#[ignore = "sonda: corra à mão"]
fn sonda_as_fases_do_quadro_hibrido() {
    let gpu = ph2d_gpu::GpuContext::new(ph2d_gpu::GpuContext::default_instance(), None)
        .expect("sem adapter");
    let (bloco, rodadas) = (
        numero("PH2D_SONDA_BLOCO", 20),
        numero("PH2D_SONDA_RODADAS", 7),
    );
    let so: Option<Vec<String>> = std::env::var("PH2D_SONDA_CENAS")
        .ok()
        .map(|v| v.split(',').map(str::to_string).collect());
    let carga = || std::fs::read_to_string("/proc/loadavg").unwrap_or_default();
    // O preço do próprio relógio: um par `Instant::now` + `elapsed`.
    let n = 100_000u32;
    let t0 = Instant::now();
    let mut soma = Duration::ZERO;
    for _ in 0..n {
        soma += std::hint::black_box(Instant::now()).elapsed();
    }
    let par_ns = t0.elapsed().as_secs_f64() * 1e9 / f64::from(n);
    eprintln!(
        "FASES placa «{}» · bloco {bloco} · {rodadas} rodadas · load {} · relógio {par_ns:.1} ns/par ({:?})",
        gpu.adapter.get_info().name,
        carga().trim(),
        soma.is_zero()
    );
    let dt = 1.0 / 60.0;
    let cam = crate::motion_shape_placa::gpu_tests::camara();
    // Um quadro: o `o_quadro`, a espera da placa, e o passe de formas da shell.
    let quadro = |m: &mut MotionState,
                  ph: &mut ph2d_core::Playhead,
                  placa: &mut crate::motion_shape_placa::PlacaDeFormas| {
        ph.advance_ticks(1);
        crate::motion_bridge::quadro::o_quadro(
            m,
            Some(&gpu),
            ph,
            dt,
            crate::motion_bridge::quadro::MaoDoEditor::default(),
        );
        let t = Instant::now();
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let espera = t.elapsed();
        let t = Instant::now();
        let _ = crate::motion_shape_placa::decide_o_quadro(placa, &gpu, m, (true, true), cam);
        le_as_fases(m, espera, t.elapsed())
    };
    let fixos: Vec<(String, f32)> = [
        ("120", 1.0),
        ("112", 1.0),
        ("125", 1.0),
        ("119", 1.0),
        ("111", 1.0),
        ("132", 1.0),
        ("136", 1.0),
        ("132", 11.0),
        ("136", 11.0),
    ]
    .iter()
    .map(|&(n, k)| (n.to_string(), k))
    .collect();
    let casos: Vec<(String, f32)> = match (std::env::var("PH2D_SONDA_ESCALAS"), &so) {
        (Ok(e), Some(cenas)) => cenas
            .iter()
            .flat_map(|n| {
                e.split(',')
                    .filter_map(|v| v.parse().ok())
                    .map(move |k| (n.clone(), k))
            })
            .collect(),
        _ => fixos,
    };
    let rotas: Vec<String> = std::env::var("PH2D_SONDA_ROTAS")
        .unwrap_or_else(|_| "A,B".into())
        .split(',')
        .map(str::to_string)
        .collect();
    for (n, k) in &casos {
        let (n, k) = (n.as_str(), *k);
        if so.as_ref().is_some_and(|f| !f.iter().any(|x| x == n)) {
            continue;
        }
        if monta_demo(n).is_none() {
            eprintln!("FASES ={n}: a cena não existe");
            continue;
        }
        if std::env::var("PH2D_SONDA_CENSO").is_ok() {
            censo_do_passe(n, k, dt);
        }
        let mut ms: Vec<MotionState> = rotas
            .iter()
            .map(|r| {
                let mut m = monta_demo(n).expect("a cena existe");
                arma_a_rota(&mut m, r);
                amplia(&mut m, k);
                m
            })
            .collect();
        let mut ph: Vec<ph2d_core::Playhead> = rotas
            .iter()
            .map(|_| {
                let mut p = ph2d_core::Playhead::new(dt);
                p.play();
                p
            })
            .collect();
        let mut placas: Vec<crate::motion_shape_placa::PlacaDeFormas> =
            rotas.iter().map(|_| Default::default()).collect();
        for _ in 0..30 {
            for i in 0..rotas.len() {
                quadro(&mut ms[i], &mut ph[i], &mut placas[i]);
            }
        }
        for (rota, m) in rotas.iter().zip(ms.iter_mut()) {
            let t = m.doc.graph.clone();
            let plano = crate::motion_bridge::gpu::plano_do_produto(m, 0.5);
            let tipos: Vec<String> = plano
                .boundaries
                .iter()
                .filter_map(|(b, _)| t.node(*b).map(|i| i.type_name.clone()))
                .collect();
            eprintln!(
                "FASES ={n} ×{k} {rota} rota «{}» · {} estágios no plano · {} fronteiras {tipos:?}",
                m.route_said.unwrap_or("?"),
                plano.stages.len(),
                tipos.len()
            );
        }
        // Por rota: as médias de cada bloco (fases) e a soma dos contadores; a ordem roda a cada
        // rodada.
        let mut blocos: Vec<Vec<[f64; 21]>> = rotas.iter().map(|_| Vec::new()).collect();
        let mut contas = vec![[0.0f64; 6]; rotas.len()];
        for r in 0..rodadas {
            for j in 0..rotas.len() {
                let i = (j + r) % rotas.len();
                let mut s = [0.0f64; 21];
                for _ in 0..bloco {
                    let (f, c) = quadro(&mut ms[i], &mut ph[i], &mut placas[i]);
                    for (a, b) in s.iter_mut().zip(f) {
                        *a += b / bloco as f64;
                    }
                    for (a, b) in contas[i].iter_mut().zip(c) {
                        *a += b;
                    }
                }
                blocos[i].push(s);
            }
        }
        let quadros = (bloco * rodadas) as f64;
        for (i, rota) in rotas.iter().enumerate() {
            let r = ms[i].pump.ultimo_relatorio();
            eprintln!(
                "FASES ={n} ×{k} {rota} passe: {} peças · {} varreduras · {} candidatos ({:.1} por peça) · {} grandes · {} instâncias na placa",
                r.pecas,
                r.varreduras,
                r.candidatos,
                r.vizinhos_por_peca(),
                r.grandes,
                ms[i].gpu_cook.instances().map_or(0, |v| v.len())
            );
            let linha: Vec<String> = FASES
                .iter()
                .enumerate()
                .map(|(j, nome)| {
                    let mut v: Vec<f64> = blocos[i].iter().map(|b| b[j]).collect();
                    v.sort_by(f64::total_cmp);
                    format!("{nome} {:.1} ({:.1})", v[0], v[v.len() / 2])
                })
                .collect();
            let c = contas[i];
            eprintln!(
                "FASES ={n} ×{k} {rota} µs · {} · por quadro: costuras {:.0} B, {:.2} enviadas, {:.2} reaproveitadas · {:.1} despachos de kernel · grupos {:.1} criados, {:.1} da cache",
                linha.join(" · "),
                c[0] / quadros,
                c[1] / quadros,
                c[2] / quadros,
                c[3] / quadros,
                c[4] / quadros,
                c[5] / quadros
            );
        }
    }
    eprintln!("FASES load no fim {}", carga().trim());
}
