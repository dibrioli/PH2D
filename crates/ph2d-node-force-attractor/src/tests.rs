//! Os gates do `force.attractor`.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

// Instances at x = 2 (inside R=4), x = 9 (outside), and at the target.
static SRC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.attractor.test.src"),
    name: "force.attractor.test.src",
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
        ctx.emit(
            Stream::new(3)
                .with("P", Column::Vec2(vec![[2.0, 0.0], [9.0, 0.0], [0.0, 0.0]]))
                .with("falloff", Column::Scalar(vec![1.0, 1.0, 1.0])),
        );
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&Src),
            t if t == MANIFEST.id => Some(&ForceAttractor),
            _ => None,
        }
    }
}

fn accel_with(params: &[(&str, f32)]) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let src = g.add_node("force.attractor.test.src");
    let att = g.add_node("force.attractor");
    g.connect(Edge {
        from: (src, 0),
        to: (att, 0),
        delayed: false,
    })
    .unwrap();
    for (name, v) in params {
        g.set_param(att, *name, *v);
    }
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, att, 0.0).unwrap();
    match out[0].as_stream().get("accel").unwrap() {
        Column::Vec2(v) => v.clone(),
        _ => panic!("accel"),
    }
}

#[test]
fn pulls_toward_the_target_inside_the_radius_only() {
    let a = accel_with(&[("curve", 0.0)]); // Linear: legible closed form
    // x=2, target 0, R=4: w = 1 − 2/4 = 0.5 → accel = −5·0.5 = −2.5 in X.
    assert!((a[0][0] + 2.5).abs() < 1e-4, "inside: pulled toward target");
    assert_eq!(a[0][1], 0.0);
    // x=9 is outside R=4 → untouched.
    assert_eq!(a[1], [0.0, 0.0], "outside the radius: zero");
    // At the target (dead zone) → zero, not NaN.
    assert_eq!(a[2], [0.0, 0.0], "dead zone: zero");
}

#[test]
fn repel_flips_the_sign() {
    let a = accel_with(&[("curve", 0.0), ("repel", 1.0)]);
    assert!((a[0][0] - 2.5).abs() < 1e-4, "repel pushes away");
}

#[test]
fn falloff_column_gates_the_force() {
    // Same graph but the src emits falloff 0.5 on instance 0 → half force.
    static HALF_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("force.attractor.test.half"),
        name: "force.attractor.test.half",
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
    struct Half;
    impl NodeOp for Half {
        fn manifest(&self) -> &'static NodeManifest {
            &HALF_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(1)
                    .with("P", Column::Vec2(vec![[2.0, 0.0]]))
                    .with("falloff", Column::Scalar(vec![0.5])),
            );
        }
    }
    struct HalfOps;
    impl OpResolver for HalfOps {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == HALF_MAN.id => Some(&Half),
                t if t == MANIFEST.id => Some(&ForceAttractor),
                _ => None,
            }
        }
    }
    let mut g = Graph::new();
    let src = g.add_node("force.attractor.test.half");
    let att = g.add_node("force.attractor");
    g.connect(Edge {
        from: (src, 0),
        to: (att, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(att, "curve", 0.0);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &HalfOps, att, 0.0).unwrap();
    match out[0].as_stream().get("accel").unwrap() {
        Column::Vec2(v) => assert!((v[0][0] + 1.25).abs() < 1e-4, "falloff 0.5 halves it"),
        _ => panic!("accel"),
    }
}

#[test]
fn two_attractors_accumulate() {
    let mut g = Graph::new();
    let src = g.add_node("force.attractor.test.src");
    let a1: NodeId = g.add_node("force.attractor");
    let a2: NodeId = g.add_node("force.attractor");
    g.connect(Edge {
        from: (src, 0),
        to: (a1, 0),
        delayed: false,
    })
    .unwrap();
    g.connect(Edge {
        from: (a1, 0),
        to: (a2, 0),
        delayed: false,
    })
    .unwrap();
    for n in [a1, a2] {
        g.set_param(n, "curve", 0.0);
    }
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, a2, 0.0).unwrap();
    match out[0].as_stream().get("accel").unwrap() {
        Column::Vec2(v) => assert!((v[0][0] + 5.0).abs() < 1e-4, "chained forces sum"),
        _ => panic!("accel"),
    }
}

#[test]
fn curves_are_endpoint_exact() {
    for k in 0..4 {
        assert_eq!(curve(k, 0.0), 0.0, "curve {k} at 0");
        assert!((curve(k, 1.0) - 1.0).abs() < 1e-6, "curve {k} at 1");
    }
}

#[test]
fn registers_and_resolves() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}

// ─────────────────────────────────────────────────────────────────────────────
// Doc 89 folha 02 — o ALVO como STREAM e o INTERCEPTO por partícula. Ver
// [`TARGET_MODE`] e [`LEAD`] para o mecanismo.
// ─────────────────────────────────────────────────────────────────────────────

/// **CADA ELEMENTO MIRA O ALVO MAIS PRÓXIMO DELE, não um alvo global.**
///
/// ⚠️ É a célula inteira. Dois alvos, um de cada lado: a partícula à esquerda tem de
/// escolher o da esquerda. FALSIFICADO por qualquer lei que devolvesse *um* alvo (o
/// primeiro, o centróide) — as duas partículas mirariam o mesmo ponto.
#[test]
fn each_element_aims_at_its_own_nearest_target() {
    let alvos = [[-10.0_f32, 0.0], [10.0, 0.0]];
    let esq = aim_at([-9.0, 0.0], [0.0; 2], &alvos, &[], 0.0).expect("há alvo");
    let dir = aim_at([9.0, 0.0], [0.0; 2], &alvos, &[], 0.0).expect("há alvo");
    assert_eq!(esq, alvos[0]);
    assert_eq!(dir, alvos[1]);
    assert_ne!(esq, dir, "um alvo global daria o mesmo ponto às duas");
}

/// **EMPATE DESEMPATA PELO ÍNDICE MAIS BAIXO** — uma ordem total, igual em toda
/// plataforma.
#[test]
fn a_distance_tie_breaks_by_the_lowest_index() {
    let alvos = [[-1.0_f32, 0.0], [1.0, 0.0]];
    assert_eq!(
        aim_at([0.0, 0.0], [0.0; 2], &alvos, &[], 0.0),
        Some(alvos[0]),
        "equidistante ⇒ o primeiro"
    );
}

/// **SEM ALVO NENHUM NÃO HÁ FORÇA — nem um zero somado por engano.**
///
/// ⚠️ `None` e `Some([0,0])` não são a mesma coisa: quem chama devolve `[0,0]` sem
/// somar, e é isso que faz um modo `Stream` sem fio ficar visivelmente parado em vez
/// de fingir que funciona.
#[test]
fn an_empty_target_stream_has_no_target_at_all() {
    assert_eq!(aim_at([1.0, 2.0], [0.0; 2], &[], &[], 0.0), None);
    assert_eq!(aim_at([1.0, 2.0], [3.0, 4.0], &[], &[], 1.5), None);
}

/// **`lead = 0` É A IDENTIDADE EXACTA** — mesmo com o alvo a mover-se depressa.
#[test]
fn a_zero_lead_aims_at_the_target_itself() {
    let alvos = [[5.0_f32, 0.0]];
    let vels = [[100.0_f32, 100.0]];
    assert_eq!(
        aim_at([0.0, 0.0], [9.0, 0.0], &alvos, &vels, 0.0),
        Some(alvos[0]),
        "sem antecipação, a mira é o alvo"
    );
}

/// **A ANTECIPAÇÃO É POR PARTÍCULA: quem está mais longe (ou mais devagar) lidera
/// mais.**
///
/// Alvo em `(10, 0)` a subir a `1/s`. A partícula A está a 10 de distância a `1/s` ⇒
/// tempo 10 s, tectado em 2 ⇒ mira `(10, 2)`. A partícula B está a 10 mas a `10/s` ⇒
/// tempo 1 s ⇒ mira `(10, 1)`. **Dois números diferentes para o mesmo alvo** — é
/// exactamente o que a célula dizia não ser exprimível.
#[test]
fn the_lead_is_each_particles_own_time_to_arrive() {
    let alvos = [[10.0_f32, 0.0]];
    let vels = [[0.0_f32, 1.0]];
    let lento = aim_at([0.0, 0.0], [1.0, 0.0], &alvos, &vels, 2.0).expect("alvo");
    let rapido = aim_at([0.0, 0.0], [10.0, 0.0], &alvos, &vels, 2.0).expect("alvo");
    assert!(
        (lento[1] - 2.0).abs() < 1e-6,
        "o lento bate no tecto: {lento:?}"
    );
    assert!(
        (rapido[1] - 1.0).abs() < 1e-6,
        "o rápido antecipa 1 s: {rapido:?}"
    );
    assert_ne!(lento, rapido, "o intercepto tem de ser POR partícula");
}

/// **O TECTO CORTA A SINGULARIDADE, E A PARTÍCULA PARADA MIRA O HORIZONTE INTEIRO.**
///
/// ⚠️ `distância / velocidade` explode a velocidade zero. O `lead` é um TECTO escrito
/// pelo artista, não um multiplicador — então o valor no limite é um número que ele
/// próprio deu, e é finito. FALSIFICADO por um `inf` ou um `NaN` na mira.
#[test]
fn a_still_particle_leads_by_the_whole_horizon_and_never_diverges() {
    let alvos = [[3.0_f32, 0.0]];
    let vels = [[0.0_f32, 2.0]];
    let mira = aim_at([0.0, 0.0], [0.0, 0.0], &alvos, &vels, 1.5).expect("alvo");
    assert!(
        mira.iter().all(|v| v.is_finite()),
        "nada de inf/NaN: {mira:?}"
    );
    assert!((mira[1] - 3.0).abs() < 1e-6, "2 × 1,5 s = 3: {mira:?}");
}

/// **UM ALVO SEM COLUNA `vel` NÃO ANTECIPA NADA** — a lista vazia lê `[0,0]`, e um
/// alvo parado está onde está.
#[test]
fn a_target_without_velocity_is_never_led() {
    let alvos = [[4.0_f32, 4.0]];
    assert_eq!(
        aim_at([0.0, 0.0], [1.0, 0.0], &alvos, &[], 2.0),
        Some(alvos[0])
    );
}

/// **O MODO `Stream` RECUSA O DEVICE, E O `Point` NÃO.**
#[test]
fn the_stream_mode_refuses_the_device_and_the_default_does_not() {
    let f = GPU_KERNEL.applicable.expect("o kernel declara a recusa");
    assert!(f(&|_: &str| 0.0), "Point: o device continua a valer");
    assert!(
        !f(&|n: &str| if n == TARGET_MODE { 1.0 } else { 0.0 }),
        "Stream: o vizinho mais próximo numa porta-template não tem canal no device"
    );
}

/// **CADA CONTROLE DO ALVO SÓ APARECE NO MODO QUE O LÊ.**
#[test]
fn every_target_control_is_gated_to_the_mode_that_reads_it() {
    let by = |p: &str| {
        PARAM_GATES
            .iter()
            .find(|g| g.param == p)
            .unwrap_or_else(|| panic!("`{p}` tem de ser gateado"))
    };
    assert_eq!(by("target_x").values, &[0]);
    assert_eq!(by("target_y").values, &[0]);
    assert_eq!(by(LEAD).values, &[1], "a antecipação é do modo Stream");
    for g in PARAM_GATES {
        assert_eq!(g.when, TARGET_MODE);
        // CONTROLE: um nome mal escrito esconderia nada e passaria verde.
        assert!(
            MANIFEST.params.iter().any(|p| p.name == g.param),
            "`{}` não é param deste nó",
            g.param
        );
    }
}

/// A porta do ALVO, cozinhada: dois pontos, um de cada lado da fila de partículas.
static TGT_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("force.attractor.test.target"),
    name: "force.attractor.test.target",
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
struct Tgt;
impl NodeOp for Tgt {
    fn manifest(&self) -> &'static NodeManifest {
        &TGT_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // As partículas da fonte estão em x = 2, 9 e 0. Alvos em 1 e 10: a do meio
        // é a única que escolhe o da direita.
        ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[1.0, 0.0], [10.0, 0.0]])));
    }
}
struct TgtOps;
impl OpResolver for TgtOps {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN.id => Some(&Src),
            t if t == TGT_MAN.id => Some(&Tgt),
            t if t == MANIFEST.id => Some(&ForceAttractor),
            _ => None,
        }
    }
}

/// Cozinha `src(3) → attractor`, com a porta do alvo ligada ou não.
fn accel_streamed(params: &[(&str, f32)], wired: bool) -> Vec<[f32; 2]> {
    let mut g = Graph::new();
    let src = g.add_node("force.attractor.test.src");
    let att = g.add_node("force.attractor");
    g.connect(Edge {
        from: (src, 0),
        to: (att, 0),
        delayed: false,
    })
    .unwrap();
    if wired {
        let t = g.add_node("force.attractor.test.target");
        g.connect(Edge {
            from: (t, 0),
            to: (att, 1),
            delayed: false,
        })
        .unwrap();
    }
    for (name, v) in params {
        g.set_param(att, *name, *v);
    }
    let mut cook = Cook::new();
    let out = cook.cook(&g, &TgtOps, att, 0.0).unwrap();
    match out[0].as_stream().get("accel").unwrap() {
        Column::Vec2(v) => v.clone(),
        _ => panic!("accel"),
    }
}

/// **O ALVO COMO STREAM CHEGA À FORÇA, PELO COOK — e cada peça puxa para o seu.**
///
/// As três partículas estão em `x = 2, 9, 0`; os alvos em `1` e `10`. A do meio (`9`)
/// é a única mais perto do alvo da direita, então ela puxa para **+x** enquanto as
/// outras duas puxam para **−x** e **+x**… ⚠️ e é o SINAL de cada uma que se afirma,
/// porque um alvo global daria o mesmo sinal a todas as que estivessem do mesmo lado.
#[test]
fn a_target_stream_reaches_the_force_through_the_cook() {
    let a = accel_streamed(&[(TARGET_MODE, 1.0), ("radius", 4.0)], true);
    assert!(a[0][0] < 0.0, "a peça em x=2 puxa para o alvo em 1: {a:?}");
    assert!(a[1][0] > 0.0, "a peça em x=9 puxa para o alvo em 10: {a:?}");
    assert!(a[2][0] > 0.0, "a peça em x=0 puxa para o alvo em 1: {a:?}");
}

/// **O MODO `Point` É BYTE-IDÊNTICO AO QUE SHIPAVA, com a porta ligada ou não.**
///
/// ⚠️ O braço da porta LIGADA é o que importa: um nó que lesse a porta sem olhar o modo
/// mudaria toda cena que viesse a ligar um alvo por engano.
#[test]
fn point_mode_is_untouched_whether_or_not_a_target_is_wired() {
    let base = accel_with(&[("radius", 4.0)]);
    assert_eq!(accel_streamed(&[("radius", 4.0)], false), base);
    assert_eq!(
        accel_streamed(&[("radius", 4.0)], true),
        base,
        "a porta ligada não pode mexer no modo Point"
    );
}

/// **`Stream` SEM FIO NÃO FAZ FORÇA NENHUMA** — visivelmente parado, em vez de fingir.
#[test]
fn stream_mode_with_nothing_wired_produces_no_force() {
    let a = accel_streamed(&[(TARGET_MODE, 1.0), ("radius", 4.0)], false);
    assert_eq!(a, vec![[0.0, 0.0]; 3], "sem alvo, sem força: {a:?}");
}
