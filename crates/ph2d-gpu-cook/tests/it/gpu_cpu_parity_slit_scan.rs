//! ⭐⭐ **PARIDADE CPU↔GPU DO `motion.slit_scan`** (ciclo 7, W1c — doc 112).
//!
//! A linha de atraso vive no `pre`, e as duas rotas guardam-na em FORMAS diferentes (32 colunas
//! `vec2` na CPU, quatro `mat4x4` no dispositivo — ver o cabeçalho do kernel). Por isso este gate
//! compara, tique a tique, a POSIÇÃO emitida **e** o anel, decodificando as faixas do dispositivo
//! para as 32 posições da CPU: um anel que avançasse errado só apareceria na posição `lag` tiques
//! depois, e numa fixture curta nunca.
//!
//! `#[ignore]`: precisa de adaptador.
//! ```text
//! cargo test -p ph2d-gpu-cook --test it -- --ignored --nocapture gpu_cpu_parity_slit_scan
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
/// Voltas: o anel tem 32 posições, e a fixture tem de o ENCHER e depois deslizá-lo.
const VOLTAS: usize = 44;
/// A posição e o anel. ⚠️ **O ε NÃO é do slit-scan** — a fonte é um `motion.oscillator`, cujo seno
/// parabólico tem sítios de FMA; o slit-scan só COPIA e interpola o que ela deu (o `lerp` é mais um
/// sítio de FMA, de ordem ULP). Medido aqui: pior `7,2e-5` (P e anel, nos quatro casos); o
/// `gpu_cpu_parity` mede a mesma onda noutra cadeia a `4,4e-4`. A barra é essa com folga — e fica
/// três ordens abaixo do que um anel avançado errado dá (a amplitude da onda, `3`).
const EPS: f32 = 5e-4;
const LADO: f32 = 20.0;

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
    ph2d_node_motion_slit_scan::register(&mut reg).unwrap();
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

/// `grid → oscillator → [falloff] → slit_scan → output`, com o `pre` do slit fechado sobre si.
/// A fonte MEXE-SE (senão o anel guardaria 32 cópias da mesma pose e o atraso seria invisível).
fn cadeia(lado: f32, lag: f32, campo: bool, ramp: f32) -> (Graph, NodeId, NodeId) {
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
        // Um `falloff` que VARIA ao longo da grelha: o atraso é `lag · posto · campo`, e um campo
        // constante deixaria verde um kernel que o ignorasse.
        let foc = g.add_node("motion.falloff");
        g.set_param(foc, "radius", 5.1);
        g.set_param(foc, "center_x", 1.3);
        g.set_param(foc, "center_y", 0.6);
        liga(&mut g, osc, (foc, 0), false);
        fonte = foc;
    }
    let ss = g.add_node("motion.slit_scan");
    g.set_param(ss, "lag", lag);
    g.set_param(ss, "ramp", ramp);
    liga(&mut g, fonte, (ss, 0), false);
    liga(&mut g, ss, (ss, 1), true);
    let out = g.add_node("motion.output");
    liga(&mut g, ss, (out, 0), false);
    (g, ss, out)
}

fn vec2(s: &Stream, c: &str) -> Vec<[f32; 2]> {
    match s.get(c) {
        Some(Column::Vec2(v)) => v.clone(),
        outra => panic!("a CPU não emitiu `{c}`: {outra:?}"),
    }
}

/// O pior `|Δ|` entre as posições da CPU e as do dispositivo.
fn pior(rotulo: &str, a: &[[f32; 2]], b: &[[f32; 2]]) -> f32 {
    assert_eq!(a.len(), b.len(), "{rotulo}: comprimentos");
    let mut m = 0.0f32;
    for (i, (p, q)) in a.iter().zip(b).enumerate() {
        for k in 0..2 {
            let d = (p[k] - q[k]).abs();
            assert!(
                d <= EPS,
                "{rotulo}: elemento {i}[{k}]: cpu {} disp {} (|Δ| {d:e})",
                p[k],
                q[k]
            );
            m = m.max(d);
        }
    }
    m
}

/// Corre as duas rotas e compara `P` e o anel inteiro. Devolve as posições da CPU por volta.
fn paridade(
    gpu: &GpuContext,
    reg: &NodeRegistry,
    rotulo: &str,
    (g, ss, out): (Graph, NodeId, NodeId),
) -> Vec<Vec<[f32; 2]>> {
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
    let mut posicoes = Vec::new();
    let (mut pior_p, mut pior_anel) = (0.0f32, 0.0f32);
    for volta in 0..VOLTAS {
        let t = volta as f64 * DT;
        let cpu = cook.cook(&g, reg, ss, t).expect("cpu cook");
        let cpu = cpu[0].as_stream();
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
        let p = vec2(cpu, "P");
        let dp = gc.read_column_vec2(gpu, ss, "P").expect("P do dispositivo");
        pior_p = pior_p.max(pior(&format!("{rotulo}, volta {volta}, P"), &p, &dp));
        // O anel: as quatro matrizes, 16 faixas por elemento cada; a posição `k` nas faixas
        // `2(k−1)` e `2(k−1)+1`, e a faixa `l` na matriz `l/16`, deslocamento `l%16`.
        let aneis: Vec<Vec<f32>> = (0..4)
            .map(|c| {
                gc.read_column(gpu, ss, &format!("ss_ring{c}"))
                    .unwrap_or_else(|| panic!("{rotulo}: o anel {c} não voltou"))
            })
            .collect();
        for k in 1..=32usize {
            let cpu_k = vec2(cpu, &format!("ss_{k}"));
            let dev_k: Vec<[f32; 2]> = (0..cpu_k.len())
                .map(|e| {
                    let faixa = |l: usize| aneis[l / 16][e * 16 + l % 16];
                    let l = 2 * (k - 1);
                    [faixa(l), faixa(l + 1)]
                })
                .collect();
            pior_anel = pior_anel.max(pior(
                &format!("{rotulo}, volta {volta}, anel {k}"),
                &cpu_k,
                &dev_k,
            ));
        }
        posicoes.push(p);
        cook.advance_tick(&g, reg, t).expect("cpu tick");
    }
    eprintln!("{rotulo}: pior P {pior_p:e}, pior anel {pior_anel:e}");
    posicoes
}

/// ⚠️ **O CONTROLO DA NÃO-VACUIDADE** — a cauda ESTÁ atrasada: a última linha não está onde a
/// primeira estaria se ninguém atrasasse nada (as duas partem da mesma onda). Sem isto, um kernel
/// que devolvesse a pose viva concordaria com uma CPU de `lag = 0`.
fn atrasou(rotulo: &str, cadeia_viva: (Graph, NodeId, NodeId), posicoes: &[Vec<[f32; 2]>]) {
    let reg = registry();
    let (g, ss, _) = cadeia_viva;
    let mut cook = Cook::new();
    let mut maior = 0.0f32;
    for (volta, p) in posicoes.iter().enumerate() {
        let viva = cook.cook(&g, &reg, ss, volta as f64 * DT).expect("cpu");
        let viva = vec2(viva[0].as_stream(), "P");
        for (a, b) in p.iter().zip(&viva) {
            maior = maior.max((a[1] - b[1]).abs());
        }
        cook.advance_tick(&g, &reg, volta as f64 * DT)
            .expect("tick");
    }
    assert!(maior > 0.1, "{rotulo}: o slit não atrasou nada ({maior})");
}

#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_slit_scan_matches_the_cpu_tick_by_tick() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for (rotulo, lag, campo, ramp) in [
        ("lag de omissão", 12.0, false, 0.0),
        ("lag fraccionário com campo", 7.5, true, 0.0),
        ("lag acima do anel (preso a 32)", 40.0, false, 0.0),
        // ⭐ `Delay By = Field` (ciclo 7, W3): o campo sozinho decide.
        ("atraso pelo campo", 9.5, true, 1.0),
    ] {
        let posicoes = paridade(&gpu, &reg, rotulo, cadeia(LADO, lag, campo, ramp));
        atrasou(rotulo, cadeia(LADO, 0.0, campo, ramp), &posicoes);
    }
}

/// Os casos em que o slit é a IDENTIDADE — `lag = 0` e um elemento só — e o anel continua a
/// encher-se (o `push` da CPU corre sempre). ⚠️ Um `lag` ilegível não chega aqui: o grafo recusa
/// um param não-finito (`set_param`), e o `lag_ticks` que o engoliria tem gate na crate do nó.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_slit_scan_identities_match_the_cpu() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    for (rotulo, lado, lag) in [("lag zero", LADO, 0.0), ("um elemento", 1.0, 12.0)] {
        paridade(&gpu, &reg, rotulo, cadeia(lado, lag, false, 0.0));
    }
}
