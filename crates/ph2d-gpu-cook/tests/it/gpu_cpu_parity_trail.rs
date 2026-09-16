//! ⭐⭐ **PARIDADE CPU↔GPU DO `motion.trail`** (ciclo 7, W1d — doc 112) — o primeiro nó cujo
//! estágio é um [`StreamOp::Carry`](ph2d_nodegraph::gpu::StreamOp): filtrar o estado, juntar com o
//! vivo, e só depois o corpo.
//!
//! Um rastro errado falha de maneiras que um gate de uma coluna só não vê: a CONTAGEM (quem
//! sobreviveu), a ORDEM (carregados primeiro), a IDADE, a identidade das colunas que um lado não
//! tem (um `size` a zero apaga o eco), e o ritmo do espaçamento (a pergunta sobre o estado
//! INTEIRO). Por isso este gate compara, tique a tique, **todas** as colunas que a CPU emite, e
//! exige do lado da CPU que a cauda tenha crescido, envelhecido e desbotado.
//!
//! `#[ignore]`: precisa de adaptador.
//! ```text
//! cargo test -p ph2d-gpu-cook --test it -- --ignored --nocapture gpu_cpu_parity_trail
//! ```

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::{CookClock, GpuCook, plan};
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_render::SinkStyle;

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const DT: f64 = 1.0 / 60.0;
const VOLTAS: usize = 30;
/// A posição e o giro: o ε é das ONDAS a montante (o rastro só COPIA posições e soma um passo ao
/// giro); medido na irmã `gpu_cpu_parity_slit_scan` a `7,2e-5` e aqui, no `rot` a `~36°`,
/// `2,7e-5`.
const EPS_ONDA: f32 = 5e-4;
/// O resto: produtos de taxas (exactos) e a matriz de cor (sítios de FMA).
const EPS: f32 = 1e-5;
/// As colunas que são CONTAGENS ou etiquetas — iguais ao bit.
const EXACTAS: &[&str] = &["trail_age", "blend", "id", "Index", "Count"];

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_output::register(&mut reg).unwrap();
    ph2d_node_motion_falloff::register(&mut reg).unwrap();
    ph2d_node_motion_oscillator::register(&mut reg).unwrap();
    ph2d_node_motion_trail::register(&mut reg).unwrap();
    ph2d_node_motion_tint::register(&mut reg).unwrap();
    ph2d_node_motion_rotate::register(&mut reg).unwrap();
    reg
}

fn liga(g: &mut Graph, de: NodeId, para: (NodeId, u16), pre: bool) {
    g.connect(Edge {
        from: (de, 0),
        to: para,
        delayed: pre,
    })
    .unwrap();
}

/// `grid → oscillator → [falloff] → trail → output`, com o `pre` do rastro fechado sobre si.
fn cadeia(lado: f32, params: &[(&str, f32)], campo: bool) -> (Graph, NodeId, NodeId) {
    let mut g = Graph::new();
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", lado);
    g.set_param(grid, "cols", lado);
    let osc = g.add_node("motion.oscillator");
    g.set_param(osc, "amplitude", 3.0);
    g.set_param(osc, "frequency", 0.7);
    liga(&mut g, grid, (osc, 0), false);
    let mut fonte = osc;
    if campo {
        // ⚠️ COR e GIRO a montante: a matriz de cor não muda o BRANCO (a rotação de matiz e a
        // saturação deixam-no onde está) e uma cabeça sem `rot` lê zero de qualquer maneira —
        // duas mutações sobreviveram à 1.ª redacção desta fixture por isso.
        let tint = g.add_node("motion.tint");
        g.set_param(tint, "mode", 0.0);
        g.set_param(tint, "r", 0.83);
        g.set_param(tint, "g", 0.27);
        g.set_param(tint, "b", 0.12);
        g.set_param(tint, "a", 0.9);
        liga(&mut g, fonte, (tint, 0), false);
        let rot = g.add_node("motion.rotate");
        g.set_param(rot, "angle", 17.5);
        liga(&mut g, tint, (rot, 0), false);
        // ⚠️ E o giro MEXE-SE: com ele parado, o buffer reciclado de dois quadros antes já tinha o
        // valor certo nas linhas da cabeça, e uma cabeça que não o escrevesse passava (medido: a
        // mutação sobreviveu com o `rot` constante).
        let gira = g.add_node("motion.oscillator");
        g.set_param(gira, "channel", 2.0);
        g.set_param(gira, "amplitude", 40.0);
        g.set_param(gira, "frequency", 1.3);
        liga(&mut g, rot, (gira, 0), false);
        fonte = gira;
        // Um campo que VARIA: a janela de cada linha é mascarada por ele (e o eco HERDA a
        // máscara), então um predicado que o ignorasse carregaria linhas a mais.
        let foc = g.add_node("motion.falloff");
        g.set_param(foc, "radius", 4.3);
        g.set_param(foc, "center_x", 1.1);
        g.set_param(foc, "center_y", -0.4);
        liga(&mut g, fonte, (foc, 0), false);
        fonte = foc;
    }
    let tr = g.add_node("motion.trail");
    for (k, v) in params {
        g.set_param(tr, *k, *v);
    }
    liga(&mut g, fonte, (tr, 0), false);
    liga(&mut g, tr, (tr, 1), true);
    let out = g.add_node("motion.output");
    liga(&mut g, tr, (out, 0), false);
    (g, tr, out)
}

fn achatada(c: &Column) -> Vec<f32> {
    match c {
        Column::Scalar(v) => v.clone(),
        Column::Vec2(v) => v.concat(),
        // O `Vec3` do dispositivo tem passada de 16 bytes (uma faixa de folga).
        Column::Vec3(v) => v.iter().flat_map(|x| [x[0], x[1], x[2], 0.0]).collect(),
        Column::Vec4(v) => v.concat(),
    }
}

/// O que a CPU emitiu num tique, para os controlos.
struct Tique {
    contagem: usize,
    idade_max: f32,
    alfa_min: f32,
}

/// Corre as duas rotas `voltas` tiques e compara TODAS as colunas da CPU.
fn paridade(
    gpu: &GpuContext,
    reg: &NodeRegistry,
    rotulo: &str,
    (g, tr, out): (Graph, NodeId, NodeId),
    voltas: usize,
) -> Vec<Tique> {
    g.validate(reg).expect("bem tipada");
    let plano = plan(&g, reg, reg, out);
    assert!(
        plano.is_fully_gpu(),
        "{rotulo}: a cadeia tem de ser do dispositivo: {:?}",
        plano.boundaries
    );
    let mut cook = Cook::new();
    let mut gc = GpuCook::new();
    gc.retain_streams_for_debug(true);
    let mut tiques = Vec::new();
    let mut pior = 0.0f32;
    for volta in 0..voltas {
        let t = volta as f64 * DT;
        let cpu: Stream = cook.cook(&g, reg, tr, t).expect("cpu cook")[0]
            .as_stream()
            .clone();
        gc.cook(
            gpu,
            &g,
            reg,
            reg,
            &plano,
            &[],
            CookClock::at(t),
            DEFAULT_UV,
            DEFAULT_SIZE,
            SinkStyle::PLAIN,
        )
        .expect("gpu cook");
        // ⚠️ **O CONJUNTO de colunas é o mesmo** — nem a mais: um caso identidade que corresse o
        // `Carry` inteiro desenharia igual e emitiria `trail_age`/`size`/`tint` que a CPU não emite.
        let mut cpu_nomes: Vec<&str> = cpu.columns().map(|(n, _)| n.as_str()).collect();
        cpu_nomes.sort_unstable();
        let mut disp_nomes: Vec<&str> = gc
            .node_columns(tr)
            .unwrap_or(&[])
            .iter()
            .map(String::as_str)
            .collect();
        disp_nomes.sort_unstable();
        assert_eq!(cpu_nomes, disp_nomes, "{rotulo}: volta {volta}, as colunas");
        for (nome, col) in cpu.columns() {
            let a = achatada(col);
            let b = gc.read_column(gpu, tr, nome).unwrap_or_else(|| {
                panic!("{rotulo}: volta {volta}, `{nome}` não voltou do dispositivo")
            });
            assert_eq!(
                a.len(),
                b.len(),
                "{rotulo}: volta {volta}, `{nome}` — contagens (cpu {} linhas)",
                cpu.count()
            );
            let eps = if nome == "P" || nome == "rot" {
                EPS_ONDA
            } else if EXACTAS.contains(&nome.as_str()) {
                0.0
            } else {
                EPS
            };
            for (k, (x, y)) in a.iter().zip(&b).enumerate() {
                let d = if x == y { 0.0 } else { (x - y).abs() };
                assert!(
                    d <= eps,
                    "{rotulo}: volta {volta}, `{nome}`[{k}]: cpu {x} disp {y} (|Δ| {d:e})"
                );
                if nome != "P" && nome != "rot" {
                    pior = pior.max(d);
                }
            }
        }
        let escalar = |c: &str| match cpu.get(c) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let alfas: Vec<f32> = match cpu.get("tint") {
            Some(Column::Vec4(v)) => v.iter().map(|t| t[3]).collect(),
            _ => Vec::new(),
        };
        tiques.push(Tique {
            contagem: cpu.count(),
            idade_max: escalar("trail_age").into_iter().fold(0.0, f32::max),
            alfa_min: alfas.into_iter().fold(1.0, f32::min),
        });
        cook.advance_tick(&g, reg, t).expect("cpu tick");
    }
    eprintln!("{rotulo}: pior |Δ| fora da posição e do giro {pior:e}");
    tiques
}

/// ⚠️ **O CONTROLO DA NÃO-VACUIDADE** — a cauda cresceu para lá da viva, envelheceu e desbotou.
/// (Com o CAMPO a maioria das linhas tem janela curta — é o que ele faz —, logo a régua é «há
/// cauda», não «há duas vezes a viva».)
fn cresceu(rotulo: &str, tiques: &[Tique], vivos: usize) {
    let maior = tiques.iter().map(|t| t.contagem).max().unwrap_or(0);
    let idade = tiques.iter().map(|t| t.idade_max).fold(0.0, f32::max);
    let alfa = tiques.iter().map(|t| t.alfa_min).fold(1.0, f32::min);
    assert!(
        maior > vivos + vivos / 2 && idade >= 3.0 && alfa < 0.5,
        "{rotulo}: a fixture não exercita o rastro (contagem {maior}, idade {idade}, alfa {alfa})"
    );
}

/// O rastro de omissão, e o com TUDO armado: espaçamento (o ritmo da promoção), a cor (a matriz
/// derivada), o giro (a variante do `rot`), o teto de estreia, o modo (a variante do `blend`) e o
/// CAMPO (a janela por linha).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_trail_matches_the_cpu_tick_by_tick() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let lado = 12.0;
    let vivos = 144;
    let t = paridade(
        &gpu,
        &reg,
        "rastro de omissão",
        cadeia(lado, &[], false),
        VOLTAS,
    );
    cresceu("rastro de omissão", &t, vivos);
    let armado = [
        ("length", 6.0),
        ("spacing", 3.0),
        ("fade", 0.2),
        ("shrink", 0.4),
        ("hue_shift", 90.0),
        ("saturation", 0.3),
        ("spin", 45.0),
        ("alpha_max", 0.6),
        ("echo_blend", 3.0),
    ];
    let t = paridade(
        &gpu,
        &reg,
        "rastro armado",
        cadeia(lado, &armado, true),
        VOLTAS,
    );
    cresceu("rastro armado", &t, vivos);
    // ⚠️ As QUATRO variantes do corpo: o armado corre a de giro E modo, o de omissão a simples, e
    // estas duas as do meio — sem elas, uma variante só com giro errado passava (medido).
    let so_giro = [("spin", 45.0), ("spacing", 2.0)];
    let t = paridade(&gpu, &reg, "só giro", cadeia(lado, &so_giro, true), 20);
    cresceu("só giro", &t, vivos);
    let so_modo = [("echo_blend", 5.0), ("fade", 0.3)];
    let t = paridade(&gpu, &reg, "só modo", cadeia(lado, &so_modo, true), 20);
    cresceu("só modo", &t, vivos);
}

/// ⚠️ **Os casos em que a CPU devolve a entrada VIVA** — um eco só (com e sem o modo por cima) — e
/// o espaçamento acima do tecto (preso a 16).
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_trail_identities_match_the_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for (rotulo, params) in [
        ("um eco", vec![("length", 1.0)]),
        (
            "um eco com modo",
            vec![("length", 1.0), ("echo_blend", 2.0)],
        ),
        (
            "espaçamento acima do tecto",
            vec![("spacing", 40.0), ("length", 3.0)],
        ),
    ] {
        let t = paridade(&gpu, &reg, rotulo, cadeia(8.0, &params, false), 20);
        if params[0].1 == 1.0 {
            assert!(
                t.iter().all(|x| x.contagem == 64),
                "{rotulo}: a CPU devolve a viva"
            );
        }
    }
}

/// **SONDA, não gate — o `MAX_INSTANCES` do rastro, medido no DISPOSITIVO** (doc 112 §4-quater,
/// `CLAUDE.md` §0.0). O tecto `262 144` foi medido no caminho de CPU; com o `Carry` o recurso
/// mudou. `grid side² → oscillator → trail(length 32) → output` com a cauda CHEIA (depois de 36
/// tiques), a mediana por quadro nas duas rotas.
///
/// ⚠️ A sonda não passa do tecto que mede: para amostrar acima dele, suba o `MAX_INSTANCES` do
/// rastro localmente (as linhas `CORTADO` dizem quando isso não foi feito).
///   cargo test -p ph2d-gpu-cook --release --test it trail_row_ceiling_probe -- --ignored --nocapture
#[test]
#[ignore = "perf probe; requires a GPU adapter"]
fn trail_row_ceiling_probe() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let median = |mut v: Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    eprintln!("  vivos       │ linhas     │ disp ms │  CPU ms");
    for side in [90.0f32, 181.0, 256.0, 313.0, 443.0] {
        let (g, _, out) = cadeia(side, &[("length", 32.0)], false);
        let n = (side * side) as usize;
        let plano = plan(&g, &reg, &reg, out);
        assert!(plano.is_fully_gpu(), "{:?}", plano.boundaries);
        let mut gc = GpuCook::new();
        let mut disp = Vec::new();
        let mut linhas = 0u32;
        for f in 0..56u64 {
            let t0 = std::time::Instant::now();
            gc.cook(
                &gpu,
                &g,
                &reg,
                &reg,
                &plano,
                &[],
                CookClock {
                    playhead: f as f64 * DT,
                    tick: Some(f),
                },
                DEFAULT_UV,
                DEFAULT_SIZE,
                SinkStyle::PLAIN,
            )
            .expect("gpu cook");
            let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
            if f >= 36 {
                disp.push(t0.elapsed().as_secs_f64() * 1000.0);
            }
            linhas = gc.instances().map_or(0, |b| b.len());
        }
        let mut cook = Cook::new();
        let mut cpu = Vec::new();
        for f in 0..40u64 {
            let t = f as f64 * DT;
            let t0 = std::time::Instant::now();
            let mut buf = Vec::new();
            ph2d_eval_motion::evaluate_motion_into(
                &mut cook,
                &g,
                &reg,
                out,
                t,
                DEFAULT_UV,
                DEFAULT_SIZE,
                &mut buf,
            )
            .expect("cpu");
            if f >= 36 {
                cpu.push(t0.elapsed().as_secs_f64() * 1000.0);
            }
            cook.advance_tick(&g, &reg, t).expect("tick");
        }
        let nota = if (linhas as usize) < n * 32 {
            "  ← CORTADO pelo tecto"
        } else {
            ""
        };
        eprintln!(
            "  {n:>11} │ {linhas:>10} │ {:>7.2} │ {:>7.2}{nota}",
            median(disp),
            median(cpu)
        );
    }
}
