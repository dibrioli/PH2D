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
/// `S1` hoje com UMA varredura do passe em cada sink · `C` hoje sem colisor (o `Drive` que escreve
/// `collider` passa a escrever o nome do controlo: nenhum passe arma).
fn arma_a_rota(m: &mut MotionState, rota: &str) {
    match rota {
        "A" => rota_de_antes(m),
        "S1" => {
            for s in m.sinks.clone() {
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
