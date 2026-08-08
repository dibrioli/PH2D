//! **SONDA — o que um valor de slider PRODUZ na ponta da cauda.**
//!
//! Report do Enio (2026-08-08): *"sliders mal balanceados. A menor mudança faz um extremo
//! efeito. Exemplo: Saturação 0.9 já fica quase todo dessaturado."*
//!
//! Antes de qualquer hipótese: os knobs de decaimento são **taxas POR TICK**, aplicadas
//! uma vez a cada linha carregada. Um eco de idade `a` recebeu o operador `a` vezes, e a
//! idade máxima da cauda é `T = (length − 1) × spacing`. Logo o que o artista VÊ na ponta
//! é `valor^T` — **exponencial no slider e função de DOIS outros knobs**.
//!
//! Esta sonda não instrumenta nada: ela dirige o cook REAL (com o `pre` self-loop que o
//! editor plumba) e mede a razão entre o eco mais velho e a cabeça viva — exatamente as
//! três grandezas que os knobs nomeiam.
//!
//! Rodar: `cargo test -p ph2d-node-motion-trail --test measure_slider_response -- --ignored --nocapture`

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, EvalCtx};
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// Um ponto que anda, numa cor SATURADA — girar/dessaturar o branco não faz nada, e uma
/// fixture branca mediria todo knob de cor como ausente.
static SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.trail.probe.src"),
    name: "motion.trail.probe.src",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};

struct Src;

impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let x = ctx.playhead() as f32;
        ctx.emit(
            Stream::new(1)
                .with("P", Column::Vec2(vec![[x, 0.0]]))
                .with("tint", Column::Vec4(vec![[0.9, 0.15, 0.05, 1.0]]))
                .with("size", Column::Vec2(vec![[1.0, 1.0]])),
        );
    }
}

/// Cozinha `ticks` ticks do grafo REAL e devolve a saída final.
fn run(params: &[(&str, f32)], ticks: u32) -> Stream {
    let mut reg = NodeRegistry::new();
    reg.register(Box::new(Src)).expect("src");
    ph2d_node_motion_trail::register(&mut reg).expect("trail");

    let mut g = Graph::new();
    let src = g.add_node("motion.trail.probe.src");
    let tr = g.add_node("motion.trail");
    g.connect(Edge {
        from: (src, 0),
        to: (tr, 0),
        delayed: false,
    })
    .expect("feed");
    g.connect(Edge {
        from: (tr, 0),
        to: (tr, 1),
        delayed: true,
    })
    .expect("ring");
    for (k, v) in params {
        g.set_param(tr, *k, *v);
    }

    let mut cook = Cook::new();
    let mut last = Stream::new(0);
    for t in 0..ticks {
        let t = f64::from(t);
        last = cook.cook(&g, &reg, tr, t).expect("cooks")[0]
            .as_stream()
            .clone();
        cook.advance_tick(&g, &reg, t).expect("advance");
    }
    last
}

const LUMA: [f32; 3] = [0.213, 0.715, 0.072];

/// A grandeza que o knob `saturation` NOMEIA: quão longe da luma os canais estão.
fn spread(c: [f32; 4]) -> f32 {
    let l = LUMA[0] * c[0] + LUMA[1] * c[1] + LUMA[2] * c[2];
    (c[0] - l).abs() + (c[1] - l).abs() + (c[2] - l).abs()
}

/// `(alfa, tamanho, espalhamento)` do eco mais VELHO e da cabeça VIVA.
fn ends(s: &Stream) -> ([f32; 3], [f32; 3]) {
    let t = match s.get("tint") {
        Some(Column::Vec4(v)) => v.clone(),
        _ => panic!("tint"),
    };
    let z = match s.get("size") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("size"),
    };
    let n = t.len() - 1;
    (
        [t[0][3], z[0][0], spread(t[0])],
        [t[n][3], z[n][0], spread(t[n])],
    )
}

/// **A RAZÃO na ponta da cauda** — o número que o artista de fato vê.
fn tail_ratio(params: &[(&str, f32)], ticks: u32) -> [f32; 3] {
    let (old, live) = ends(&run(params, ticks));
    [
        old[0] / live[0].max(1e-9),
        old[1] / live[1].max(1e-9),
        old[2] / live[2].max(1e-9),
    ]
}

/// **A MEDIÇÃO PRINCIPAL: o valor do slider contra o que ele produz.**
#[test]
#[ignore = "sonda: cargo test -p ph2d-node-motion-trail --test measure_slider_response -- --ignored --nocapture"]
fn measure_what_a_slider_value_produces_at_the_tail() {
    // (length, spacing) — o default do nó, a esteira do smoke `PH2D_ECHO_SMOKE`, e o teto.
    let configs: [(f32, f32); 4] = [(8.0, 1.0), (6.0, 4.0), (32.0, 1.0), (8.0, 8.0)];
    let values = [0.99f32, 0.95, 0.90, 0.80, 0.50, 0.0];

    println!("\n=== SATURATION: o valor do slider -> o que sobra na ponta da cauda ===");
    println!(
        "{:<20} {:>4} {}",
        "config",
        "T",
        values
            .iter()
            .map(|v| format!("{v:>8.2}"))
            .collect::<String>()
    );
    println!("{}", "-".repeat(80));
    for (len, sp) in configs {
        let t = ((len - 1.0) * sp) as u32;
        let row: String = values
            .iter()
            .map(|&v| {
                let r = tail_ratio(
                    &[
                        ("length", len),
                        ("spacing", sp),
                        ("fade", 1.0),
                        ("shrink", 1.0),
                        ("saturation", v),
                    ],
                    (t + 6).max(24),
                );
                format!("{:>8.4}", r[2])
            })
            .collect();
        println!("len {len:>4.0} sp {sp:>3.0}      {t:>4} {row}");
    }

    println!("\n=== FADE (alfa) e SHRINK (tamanho) na ponta ===");
    for (len, sp) in configs {
        let t = ((len - 1.0) * sp) as u32;
        let ticks = (t + 6).max(24);
        let f = tail_ratio(
            &[
                ("length", len),
                ("spacing", sp),
                ("fade", 0.8),
                ("shrink", 0.93),
            ],
            ticks,
        );
        println!(
            "len {len:>4.0} sp {sp:>3.0}  T={t:>3}   fade 0.80 -> {:>8.4}   shrink 0.93 -> {:>8.4}",
            f[0], f[1]
        );
    }

    println!("\n=== A FAIXA UTIL do slider de saturacao (o que da entre 0.9 e 0.1 na ponta) ===");
    for (len, sp) in configs {
        let t = (len - 1.0) * sp;
        let hi = 0.9f32.powf(1.0 / t);
        let lo = 0.1f32.powf(1.0 / t);
        println!(
            "len {len:>4.0} sp {sp:>3.0}  T={t:>5.0}   faixa util = [{lo:.4} .. {hi:.4}]  \
             = {:>5.1}% de um slider 0..2",
            (hi - lo) / 2.0 * 100.0
        );
    }

    println!("\n=== OS ANGULOS: o total que a cauda percorre ===");
    for (len, sp) in configs {
        let t = (len - 1.0) * sp;
        println!(
            "len {len:>4.0} sp {sp:>3.0}  T={t:>5.0}   hue_shift 35 -> {:>7.0} deg totais   \
             spin 9 -> {:>7.0} deg totais",
            35.0 * t,
            9.0 * t
        );
    }
}

/// **O MESMO VALOR, CONFIGURAÇÕES DIFERENTES** — o segundo defeito, e o mais caro:
/// o número certo do knob é função de OUTROS dois knobs.
#[test]
#[ignore = "sonda: ver o irmao acima"]
fn measure_how_much_another_knob_moves_the_answer() {
    println!("\n=== saturation = 0.9, mexendo SO no length/spacing ===");
    for (len, sp) in [
        (2.0f32, 1.0f32),
        (4.0, 1.0),
        (8.0, 1.0),
        (16.0, 1.0),
        (8.0, 2.0),
        (8.0, 4.0),
        (6.0, 4.0),
    ] {
        let t = ((len - 1.0) * sp) as u32;
        let r = tail_ratio(
            &[
                ("length", len),
                ("spacing", sp),
                ("fade", 1.0),
                ("shrink", 1.0),
                ("saturation", 0.9),
            ],
            (t + 6).max(24),
        );
        println!(
            "len {len:>4.0} sp {sp:>3.0}  T={t:>3}  ->  saturacao na ponta {:>8.4}",
            r[2]
        );
    }
}

/// **SONDA — a idade que o eco mais VELHO de fato alcança**, por configuração.
///
/// ⚠️ A lei do alvo precisa desse número, e derivá-lo por aritmética paralela à regra de
/// sobrevivência é como as duas divergem: o `(k−1)·s` que eu escrevi errou por um tick em
/// todo `spacing > 1`. Pergunte ao ANEL.
#[test]
#[ignore = "sonda: ver os irmaos acima"]
fn measure_the_age_the_oldest_echo_reaches() {
    println!("\nlen  sp | (len-1)*sp |  idades vivas (regime)");
    for (len, sp) in [
        (2.0f32, 1.0f32),
        (4.0, 1.0),
        (8.0, 1.0),
        (16.0, 1.0),
        (8.0, 2.0),
        (6.0, 4.0),
        (8.0, 8.0),
        (3.0, 3.0),
        (5.0, 2.0),
    ] {
        let ticks = ((len as u32) * (sp as u32)) * 4 + 40;
        let s = run(&[("length", len), ("spacing", sp)], ticks);
        let mut a = match s.get("trail_age") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => vec![],
        };
        a.sort_by(f32::total_cmp);
        println!("{len:>3.0} {sp:>3.0} | {:>10.0} |  {a:?}", (len - 1.0) * sp);
    }
}

/// **SONDA — a idade do eco mais velho OSCILA?** (a cadência de promoção e a de descarte
/// não são travadas uma na outra quando `spacing > 1`).
#[test]
#[ignore = "sonda: ver os irmaos acima"]
fn measure_whether_the_oldest_age_is_stable() {
    for (len, sp) in [(8.0f32, 1.0f32), (8.0, 2.0), (6.0, 4.0), (8.0, 8.0)] {
        let mut reg = NodeRegistry::new();
        reg.register(Box::new(Src)).expect("src");
        ph2d_node_motion_trail::register(&mut reg).expect("trail");
        let mut g = Graph::new();
        let src = g.add_node("motion.trail.probe.src");
        let tr = g.add_node("motion.trail");
        g.connect(Edge {
            from: (src, 0),
            to: (tr, 0),
            delayed: false,
        })
        .unwrap();
        g.connect(Edge {
            from: (tr, 0),
            to: (tr, 1),
            delayed: true,
        })
        .unwrap();
        g.set_param(tr, "length", len);
        g.set_param(tr, "spacing", sp);
        let mut cook = Cook::new();
        let (mut oldest, mut counts) = (Vec::new(), Vec::new());
        let total = ((len as u32) * (sp as u32)) * 3 + 60;
        for t in 0..total {
            let t = f64::from(t);
            let s = cook.cook(&g, &reg, tr, t).unwrap()[0].as_stream().clone();
            cook.advance_tick(&g, &reg, t).unwrap();
            if let Some(Column::Scalar(v)) = s.get("trail_age") {
                oldest.push(v.iter().copied().fold(0.0f32, f32::max) as u32);
                counts.push(v.len());
            }
        }
        let tail = &oldest[oldest.len().saturating_sub(16)..];
        let ct = &counts[counts.len().saturating_sub(16)..];
        println!(
            "len {len:>3.0} sp {sp:>2.0} | nominal (k-1)*s = {:>3.0} | idades finais {tail:?}\n\
             {:22} | contagens {ct:?}",
            (len - 1.0) * sp,
            ""
        );
    }
}
