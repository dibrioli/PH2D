use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

// Source: 3 instances on a line (x = 0,1,2) with an existing rot [0,0,0].
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.stagger.test.src"),
    name: "motion.stagger.test.src",
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
        &SRC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]])));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&Src),
            t if t == MANIFEST.id => Some(&MotionStagger),
            _ => None,
        }
    }
}

fn staggered_y(setup: impl FnOnce(&mut Graph, ph2d_nodegraph::graph::NodeId)) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let src = g.add_node("motion.stagger.test.src");
    let st = g.add_node("motion.stagger");
    g.connect(Edge {
        from: (src, 0),
        to: (st, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(st, "channel", 1.0); // Y
    setup(&mut g, st);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, st, 0.0).unwrap();
    match out[0].as_stream().get("P").unwrap() {
        Column::Vec2(v) => v.clone(),
        _ => panic!("P"),
    }
}

#[test]
fn linear_ramp_adds_min_to_max_across_the_stream() {
    // channel=Y, min=0, max=2, linear: raw = 0, 0.5, 1 → Δy = 0, 1, 2 added.
    let p = staggered_y(|g, st| {
        g.set_param(st, "min", 0.0);
        g.set_param(st, "max", 2.0);
    });
    assert_eq!(p, vec![[0.0, 0.0], [1.0, 1.0], [2.0, 2.0]]);
}

#[test]
fn reverse_flips_the_ramp_endpoints() {
    // reverse: raw = 1, 0.5, 0 → Δy = 2, 1, 0. X passes through unchanged.
    let p = staggered_y(|g, st| {
        g.set_param(st, "min", 0.0);
        g.set_param(st, "max", 2.0);
        g.set_param(st, "reverse", 1.0);
    });
    assert_eq!(p, vec![[0.0, 2.0], [1.0, 1.0], [2.0, 0.0]]);
}

#[test]
fn easing_applies_a_non_linear_curve_to_the_ramp() {
    // Quad-In (curve 1, dir 0) bends the ramp below the diagonal at the
    // midpoint: min=0,max=4 over 3 instances → mid Δy = 4·ease(1,0,0.5) = 1
    // (vs 2 for linear). Proves ease_curve/ease_dir flow into the delta.
    let p = staggered_y(|g, st| {
        g.set_param(st, "min", 0.0);
        g.set_param(st, "max", 4.0);
        g.set_param(st, "ease_curve", 1.0); // Quad
        g.set_param(st, "ease_dir", 0.0); // In
    });
    assert_eq!(p[0][1], 0.0); // endpoint
    assert_eq!(p[1][1], 1.0); // eased midpoint (below linear's 2)
    assert_eq!(p[2][1], 4.0); // endpoint
}

#[test]
fn single_instance_takes_min() {
    // n == 1 → raw = 0 → delta = min (no divide-by-zero on `n-1`).
    assert_eq!(
        stagger_delta(3.0, 9.0, 0, 0, 0.0, 1.0, None, &ph2d_ease::Shape::DEFAULT),
        3.0
    );
}

/// The focus field gates the ramp (audit 2026-07-10: untested until now):
/// a source that carries `falloff` [1, 0.5, 0] sees the linear 0→2 ramp
/// scaled per instance — the masked tail doesn't move at all.
#[test]
fn falloff_scales_the_ramp_per_instance() {
    static FSRC_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.stagger.test.fsrc"),
        name: "motion.stagger.test.fsrc",
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
    struct FSrc;
    impl NodeOp for FSrc {
        fn manifest(&self) -> &'static NodeManifest {
            &FSRC_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(3)
                    .with("P", Column::Vec2(vec![[0.0, 0.0]; 3]))
                    .with("falloff", Column::Scalar(vec![1.0, 0.5, 0.0])),
            );
        }
    }
    struct FOps;
    impl OpResolver for FOps {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == FSRC_MAN.id => Some(&FSrc),
                t if t == MANIFEST.id => Some(&MotionStagger),
                _ => None,
            }
        }
    }
    let mut g = Graph::new();
    let src = g.add_node("motion.stagger.test.fsrc");
    let st = g.add_node("motion.stagger");
    g.connect(Edge {
        from: (src, 0),
        to: (st, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(st, "channel", 1.0); // Y
    g.set_param(st, "min", 0.0);
    g.set_param(st, "max", 2.0);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &FOps, st, 0.0).unwrap();
    match out[0].as_stream().get("P").unwrap() {
        // Linear raw = 0, 0.5, 1 → Δy = 0, 1, 2; × falloff [1, 0.5, 0]
        // → 0, 0.5, 0. The masked LAST instance stays put even though the
        // raw ramp peaks there.
        Column::Vec2(v) => assert_eq!(v, &vec![[0.0, 0.0], [0.0, 0.5], [0.0, 0.0]]),
        _ => panic!("P"),
    }
}

#[test]
fn registers_and_resolves() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}

/// **TROCAR AS PONTAS JÁ INVERTE A RAMPA — e o `reverse` É OUTRA COISA.**
///
/// ⚠️ **Este gate nasceu de uma célula REFUTADA** (folha 06 linha 31,
/// *"Min>Max auto-invertido"*, Cavalry, marcada P2 como *omissão de
/// ergonomia*). Medido em 2026-08-22: `min + ease(t)·(max − min)` **já** dá a
/// rampa descendente quando o artista troca os dois números — não há gesto
/// extra, não há nó a mais, e o *default inteligente* que a referência
/// documenta é o que este nó sempre fez.
///
/// ⚠️ **E o que ficaria por construir seria uma REDEFINIÇÃO, não uma adição.**
/// Com uma ease não-linear há DUAS rampas descendentes possíveis: a que sai de
/// `min` devagar (a nossa — a ease descreve *como o valor deixa a primeira
/// ponta e chega à segunda*, e trocar os números troca as pontas, não a lei) e
/// a que é a rampa ascendente LIDA AO CONTRÁRIO. A segunda já tem um param
/// dedicado: `reverse`. Construir um "auto-invert" seria ou um no-op na curva
/// linear, ou o `reverse` a mudar de significado em silêncio.
///
/// Este gate pina as três metades: a troca inverte · o `reverse` espelha · os
/// dois **diferem** numa ease não-linear (senão o par seria redundante e a
/// refutação estaria a defender uma distinção que não existe).
#[test]
fn swapping_the_endpoints_inverts_and_reverse_is_a_different_thing() {
    let ramp = |lo: f32, hi: f32, curve: i32| -> Vec<f32> {
        (0..5)
            .map(|i| {
                stagger_delta(
                    lo,
                    hi,
                    curve,
                    0,
                    i as f32 / 4.0,
                    1.0,
                    None,
                    &ph2d_ease::Shape::DEFAULT,
                )
            })
            .collect()
    };
    for curve in 0..=7 {
        let up = ramp(0.0, 1.0, curve);
        let down = ramp(1.0, 0.0, curve);
        // (1) A troca REFLETE a rampa no valor: `down = 1 − up`, exacto.
        //
        // ⚠️ **E não «é monótona a descer»** — a primeira versão deste gate
        // pediu isso e reprovou nas curvas **Back** e **Bounce**, que
        // ultrapassam de propósito. Uma barra que código correto não consegue
        // satisfazer não é rigor; a lei verdadeira é a reflexão, e ela vale
        // para as oito porque sai da própria fórmula
        // (`min + e·(max−min)` com `[1,0]` é `1 − e`).
        assert!((down[0] - 1.0).abs() < 1e-6, "curva {curve}: comeca em min");
        assert!((down[4] - 0.0).abs() < 1e-6, "curva {curve}: acaba em max");
        for (d, u) in down.iter().zip(&up) {
            assert!(
                (d - (1.0 - u)).abs() < 1e-6,
                "curva {curve}: {d} vs {}",
                1.0 - u
            );
        }
        // (2) O `reverse` ESPELHA a rampa ascendente.
        let mirrored: Vec<f32> = (0..5)
            .map(|i| {
                stagger_delta(
                    0.0,
                    1.0,
                    curve,
                    0,
                    1.0 - i as f32 / 4.0,
                    1.0,
                    None,
                    &ph2d_ease::Shape::DEFAULT,
                )
            })
            .collect();
        let up_rev: Vec<f32> = up.iter().rev().copied().collect();
        for (a, b) in mirrored.iter().zip(&up_rev) {
            assert!((a - b).abs() < 1e-6, "curva {curve}: {a} vs {b}");
        }
    }
    // (3) ⚠️ E as duas leituras DIFEREM numa ease não-linear — é o que torna a
    // refutação uma distinção real e não uma desculpa. Na linear coincidem, de
    // propósito: lá não há nada a distinguir.
    let linear_gap = ramp(1.0, 0.0, 0)
        .iter()
        .zip(ramp(0.0, 1.0, 0).iter().rev())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        linear_gap < 1e-6,
        "na linear as duas coincidem: {linear_gap}"
    );
    let eased_gap = ramp(1.0, 0.0, 1)
        .iter()
        .zip(ramp(0.0, 1.0, 1).iter().rev())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        eased_gap > 0.3,
        "numa ease as duas tem de divergir, senao o `reverse` seria redundante: {eased_gap}"
    );
}

/// **SONDA: as duas leituras da rampa invertida, lado a lado** — o instrumento
/// que produziu a refutação acima.
///
/// `cargo test -p ph2d-node-motion-stagger measure_reversed_endpoints -- --ignored --nocapture`
#[test]
#[ignore = "sonda, não um gate — `-- --ignored --nocapture`"]
fn measure_reversed_endpoints() {
    for curve in 0..=7 {
        let up: Vec<f32> = (0..5)
            .map(|i| {
                stagger_delta(
                    0.0,
                    1.0,
                    curve,
                    0,
                    i as f32 / 4.0,
                    1.0,
                    None,
                    &ph2d_ease::Shape::DEFAULT,
                )
            })
            .collect();
        let down: Vec<f32> = (0..5)
            .map(|i| {
                stagger_delta(
                    1.0,
                    0.0,
                    curve,
                    0,
                    i as f32 / 4.0,
                    1.0,
                    None,
                    &ph2d_ease::Shape::DEFAULT,
                )
            })
            .collect();
        // O que um "swap + reverse" daria: a mesma rampa lida ao contrário.
        let mirror: Vec<f32> = up.iter().rev().copied().collect();
        println!("curva {curve}:");
        println!("  min<max : {up:.3?}");
        println!("  min>max : {down:.3?}");
        println!("  espelho : {mirror:.3?}");
    }
}
