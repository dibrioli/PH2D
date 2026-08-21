//! Os gates do `motion.cull`.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// Fraction mode keeps the FIRST `amount·n` elements. FALSIFIED if it kept a
/// different count or the wrong (non-leading) ones.
#[test]
fn fraction_keeps_the_leading_elements() {
    let keep = keep_indices(10, MODE_FRACTION, 0.3, false, &[]);
    assert_eq!(keep, vec![0, 1, 2], "0.3 * 10 -> the first 3");
}

/// Fraction 0 empties the stream; 1 keeps all.
#[test]
fn fraction_endpoints() {
    assert!(
        keep_indices(8, MODE_FRACTION, 0.0, false, &[]).is_empty(),
        "0 -> none"
    );
    assert_eq!(
        keep_indices(8, MODE_FRACTION, 1.0, false, &[]).len(),
        8,
        "1 -> all"
    );
}

/// Invert keeps the complement (the trailing elements under Fraction).
#[test]
fn invert_keeps_the_complement() {
    let keep = keep_indices(10, MODE_FRACTION, 0.3, true, &[]);
    assert_eq!(keep, vec![3, 4, 5, 6, 7, 8, 9], "the other 7");
}

/// Falloff mode keeps the elements whose mask is ≥ the threshold.
#[test]
fn falloff_threshold_keeps_above() {
    let falloff = vec![0.1, 0.9, 0.5, 1.0, 0.2];
    let keep = keep_indices(5, 1, 0.5, false, &falloff);
    assert_eq!(keep, vec![1, 2, 3], "falloff ≥ 0.5");
}

/// Deterministic + cooks through the registry: the `amount` value input drives the
/// keep count and every column is filtered to the survivors.
#[test]
fn registers_and_culls_through_the_cook() {
    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.cull.test.src"),
        name: "motion.cull.test.src",
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
            let p: Vec<[f32; 2]> = (0..10).map(|i| [i as f32, 0.0]).collect();
            let s: Vec<f32> = (0..10).map(|i| i as f32).collect();
            ctx.emit(
                Stream::new(10)
                    .with("P", Column::Vec2(p))
                    .with("size", Column::Scalar(s)),
            );
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionCull),
                _ => None,
            }
        }
    }
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());

    let mut g = Graph::new();
    let src = g.add_node("motion.cull.test.src");
    let c = g.add_node("motion.cull");
    g.set_param(c, "amount", 0.4); // keep the first 4
    g.connect(Edge {
        from: (src, 0),
        to: (c, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, c, 0.0).unwrap();
    let st = out[0].as_stream();
    assert_eq!(st.count(), 4, "0.4 × 10 kept");
    match (st.get("P").unwrap(), st.get("size").unwrap()) {
        (Column::Vec2(pv), Column::Scalar(sv)) => {
            assert_eq!(pv.len(), 4, "P filtered");
            assert_eq!(
                sv,
                &vec![0.0, 1.0, 2.0, 3.0],
                "size filtered to the survivors"
            );
        }
        _ => panic!("columns"),
    }
}

/// **O teto mantém os mais NOVOS** — a decisão da wave, e ela NÃO é gosto: o
/// `motion.emitter`, o irmão que a folha 13 cita, já a tem escrita com o motivo (*"a dense
/// young jet, not a frozen ancient cloud"*), e numa zona o prefixo é o mais VELHO porque o
/// `motion.combine` apende os recém-nascidos ao estado. FALSIFICADO se guardasse o prefixo.
#[test]
fn the_count_mode_keeps_the_newest_not_the_oldest() {
    let keep = keep_indices(10, MODE_COUNT, 3.0, false, &[]);
    assert_eq!(keep, vec![7, 8, 9], "a CAUDA, nunca o prefixo");
}

/// **Abaixo do teto o modo é um NO-OP, e é isso que o separa do Fraction.** Uma fração rala
/// a população o tempo todo (0,5 de 4 mantém 2); um teto de 100 sobre 4 não toca em nada.
#[test]
fn below_the_cap_the_count_mode_touches_nothing() {
    assert_eq!(
        keep_indices(4, MODE_COUNT, 100.0, false, &[]),
        vec![0, 1, 2, 3],
        "um teto que ninguém atingiu não corta"
    );
    // O CONTROLE, ao lado: o mesmo número lido como FRAÇÃO rala metade.
    assert_eq!(
        keep_indices(4, MODE_FRACTION, 0.5, false, &[]).len(),
        2,
        "o controle: uma fração corta mesmo com folga"
    );
}

/// O teto é uma contagem, então ele **satura** em vez de estourar: `max` maior que a
/// população mantém tudo, `0` mantém nada, e nenhum dos dois indexa fora do stream.
#[test]
fn the_cap_saturates_at_both_ends() {
    assert!(keep_indices(6, MODE_COUNT, 0.0, false, &[]).is_empty());
    assert_eq!(keep_indices(6, MODE_COUNT, 6.0, false, &[]).len(), 6);
    assert_eq!(keep_indices(0, MODE_COUNT, 9.0, false, &[]).len(), 0);
    assert_eq!(keep_indices(6, MODE_COUNT, -4.0, false, &[]).len(), 0);
}

/// **A porta `amount` alimenta o número que o MODO usa.** Sem isto o teto seria o único
/// número do nó que um `value.*` não alcança — e o socket já está lá, vivo, animando a
/// fração; deixá-lo mudo num modo é a metade de um controle.
#[test]
fn the_value_port_feeds_whichever_number_the_mode_reads() {
    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.cull.test.port.src"),
        name: "motion.cull.test.port.src",
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
            let p: Vec<[f32; 2]> = (0..10).map(|i| [i as f32, 0.0]).collect();
            ctx.emit(Stream::new(10).with("P", Column::Vec2(p)));
        }
    }
    static VAL: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.cull.test.port.value"),
        name: "motion.cull.test.port.value",
        inputs: &[],
        outputs: &[PortSpec {
            name: "out",
            ty: VALUE,
        }],
        effect: Effect::Pure,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    };
    struct Val;
    impl NodeOp for Val {
        fn manifest(&self) -> &'static NodeManifest {
            &VAL
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(1).with(VALUE_COL, Column::Scalar(vec![2.0])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == VAL.id => Some(&Val),
                t if t == MANIFEST.id => Some(&MotionCull),
                _ => None,
            }
        }
    }

    let mut g = Graph::new();
    let src = g.add_node("motion.cull.test.port.src");
    let val = g.add_node("motion.cull.test.port.value");
    let c = g.add_node("motion.cull");
    g.set_param(c, "mode", MODE_COUNT as f32);
    g.set_param(c, "max", 9.0); // o param diz 9...
    for (from, to, port) in [(src, c, 0u16), (val, c, 1)] {
        g.connect(Edge {
            from: (from, 0),
            to: (to, port),
            delayed: false,
        })
        .unwrap();
    }
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, c, 0.0).unwrap();
    // ...e a porta diz 2, então sobram 2 — o fio vence o param, no modo Count como já
    // vencia no Fraction.
    assert_eq!(out[0].as_stream().count(), 2);

    // ⚠️ E a METADE que o nome deste gate promete e a de cima NÃO testa: **sem** fio, qual
    // param o modo lê. Com a porta ligada os dois ramos leem o mesmo número, então uma
    // mutação que faça o Count ler `amount` passa aqui em cima — medido. Sem fio, `max` e
    // `amount` discordam de propósito, e só um dos dois pode estar certo.
    let mut g2 = Graph::new();
    let src2 = g2.add_node("motion.cull.test.port.src");
    let c2 = g2.add_node("motion.cull");
    g2.set_param(c2, "mode", MODE_COUNT as f32);
    g2.set_param(c2, "max", 2.0);
    g2.set_param(c2, "amount", 9.0);
    g2.connect(Edge {
        from: (src2, 0),
        to: (c2, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook2 = Cook::new();
    let out2 = cook2.cook(&g2, &Ops, c2, 0.0).unwrap();
    assert_eq!(
        out2[0].as_stream().count(),
        2,
        "o modo Count lê `max`; ler `amount` daria 9"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Doc 89 folha 08 — a RENUMERAÇÃO. Ver [`REINDEX`] para o mecanismo e o porquê do
// default. Os gates cozinham pelo `Graph`/`Cook`/`OpResolver`.
// ─────────────────────────────────────────────────────────────────────────────

/// Uma fonte de 9 peças com as DUAS colunas de identidade honestas (o que uma
/// `motion.grid(3×3)` entrega), mais uma máscara que corta as quatro últimas.
static ID_SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.cull.test.ident"),
    name: "motion.cull.test.ident",
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
struct IdSrc;
impl NodeOp for IdSrc {
    fn manifest(&self) -> &'static NodeManifest {
        &ID_SRC
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        #[expect(clippy::cast_precision_loss, reason = "nove elementos")]
        let idx: Vec<f32> = (0..9).map(|i| i as f32).collect();
        ctx.emit(
            Stream::new(9)
                .with("P", Column::Vec2(vec![[0.0, 0.0]; 9]))
                .with("Index", Column::Scalar(idx))
                .with("Count", Column::Scalar(vec![9.0; 9])),
        );
    }
}
/// Uma fonte NUA: dez peças sem coluna de identidade nenhuma — o caso que prova que
/// o knob CUNHA em vez de remendar.
static BARE_SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.cull.test.bare"),
    name: "motion.cull.test.bare",
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
struct BareSrc;
impl NodeOp for BareSrc {
    fn manifest(&self) -> &'static NodeManifest {
        &BARE_SRC
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(10).with("P", Column::Vec2(vec![[0.0, 0.0]; 10])));
    }
}

struct IdOps;
impl OpResolver for IdOps {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == ID_SRC.id => Some(&IdSrc),
            t if t == BARE_SRC.id => Some(&BareSrc),
            t if t == MANIFEST.id => Some(&MotionCull),
            _ => None,
        }
    }
}

/// Cozinha `ident(9) → cull(Fraction 0.5)` com o `reindex` pedido, e devolve as duas
/// colunas de identidade da saída.
fn cook_identity(reindex_on: Option<f32>) -> (Vec<f32>, Vec<f32>) {
    let mut g = Graph::new();
    let src = g.add_node("motion.cull.test.ident");
    let cu = g.add_node("motion.cull");
    g.set_param(cu, "amount", 0.5);
    // ⚠️ O braço `None` NÃO escreve o param: é ele que prova que o DEFAULT é o
    // comportamento de hoje, e não uma escrita explícita de `0.0` a fingi-lo.
    if let Some(v) = reindex_on {
        g.set_param(cu, "reindex", v);
    }
    g.connect(Edge {
        from: (src, 0),
        to: (cu, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &IdOps, cu, 0.0).unwrap();
    let s = out[0].as_stream();
    let col = |n: &str| match s.get(n) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => panic!("falta a coluna `{n}`"),
    };
    (col("Index"), col("Count"))
}

/// **O DEFEITO, ENCENADO: sem o knob, o `Count` DESCREVE A LISTA DE ANTES.**
///
/// ⚠️ Este gate pina o comportamento de HOJE, e é ele que torna o outro honesto: sem
/// o controlo negativo, *"o reindex escreve 5"* não distingue uma cura de um nó que
/// já escrevia 5. `Index = [0..4]` já estava certo (o Fraction guarda o PREFIXO), e o
/// `Count = 9` é a metade que mente — o `motion.tint` divide por `Count − 1 = 8` e o
/// degradê pára a meio.
#[test]
fn without_the_knob_the_count_still_describes_the_list_from_before() {
    let (idx, cnt) = cook_identity(None);
    assert_eq!(idx, vec![0.0, 1.0, 2.0, 3.0, 4.0]);
    assert_eq!(cnt, vec![9.0; 5], "a lista tem 5 e a coluna diz 9");
    assert_eq!(
        cook_identity(Some(0.0)),
        (idx, cnt),
        "escrever 0 tem de ser o mesmo que não escrever nada"
    );
}

/// **AS DUAS COLUNAS, JUNTAS** — e *juntas* é a lei, não uma conveniência.
///
/// ⚠️ Meia cura FAZ MAL: num nó cujo `Index` também mentisse, corrigir só o `Count`
/// faria a rampa alcançar **metade, duas vezes**. O gate exige as duas.
#[test]
fn the_knob_renumbers_both_identity_columns_for_the_surviving_list() {
    let (idx, cnt) = cook_identity(Some(1.0));
    assert_eq!(idx, vec![0.0, 1.0, 2.0, 3.0, 4.0], "0..n−1");
    assert_eq!(cnt, vec![5.0; 5], "e a contagem é a da lista que sobrou");
}

/// **ELE CUNHA AS COLUNAS QUE NÃO EXISTIAM** — a lei do `motion.combine`.
///
/// Com o knob ligado o artista pediu identidade para a lista de saída. Uma lista sem
/// `Index` é uma em que o nó a jusante inventa o dele pela posição: a mesma resposta
/// **por acidente**, até alguém pôr um `sort` no meio.
#[test]
fn the_knob_mints_the_columns_a_bare_stream_never_had() {
    let mut g = Graph::new();
    let src = g.add_node("motion.cull.test.bare"); // 10 peças, sem Index/Count
    let cu = g.add_node("motion.cull");
    g.set_param(cu, "amount", 0.3);
    g.set_param(cu, "reindex", 1.0);
    g.connect(Edge {
        from: (src, 0),
        to: (cu, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &IdOps, cu, 0.0).unwrap();
    let s = out[0].as_stream();
    assert_eq!(s.count(), 3);
    match (s.get("Index"), s.get("Count")) {
        (Some(Column::Scalar(i)), Some(Column::Scalar(c))) => {
            assert_eq!(i, &vec![0.0, 1.0, 2.0]);
            assert_eq!(c, &vec![3.0; 3]);
        }
        _ => panic!("as duas colunas têm de nascer"),
    }
}

/// **UMA LISTA VAZIA NÃO ENTRA EM PÂNICO NEM CUNHA UM `Count` MENTIROSO.**
#[test]
fn an_empty_survivor_list_is_still_honest() {
    let mut g = Graph::new();
    let src = g.add_node("motion.cull.test.ident");
    let cu = g.add_node("motion.cull");
    g.set_param(cu, "amount", 0.0);
    g.set_param(cu, "reindex", 1.0);
    g.connect(Edge {
        from: (src, 0),
        to: (cu, 0),
        delayed: false,
    })
    .unwrap();
    let mut cook = Cook::new();
    let out = cook.cook(&g, &IdOps, cu, 0.0).unwrap();
    assert_eq!(out[0].as_stream().count(), 0);
}

/// **LIGADO, O NÓ RECUSA O DEVICE** — e desligado (o default) nada recua.
///
/// ⚠️ É a metade que impede a divergência: a compactação no device é maquinaria do
/// `StreamOp`, e renumerar ali seria uma SEGUNDA implementação da mesma lei. O gate
/// mede as duas direcções, senão um `applicable` que devolvesse `false` sempre
/// passaria por vácuo — e teria posto o nó inteiro na CPU sem ninguém notar.
#[test]
fn the_renumbering_refuses_the_device_and_the_default_does_not() {
    let on = |name: &str| if name == REINDEX { 1.0 } else { 0.0 };
    let off = |_: &str| 0.0;
    let f = GPU_KERNEL
        .applicable
        .expect("o kernel tem de declarar a recusa");
    assert!(f(&off), "desligado: o device continua a valer");
    assert!(!f(&on), "ligado: o nó recua para a CPU");
}
