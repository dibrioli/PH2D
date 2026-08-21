//! Os gates do `field.shape` — doc 89 folha 08 (a célula do `motion.falloff`
//! *Shape*) e folha 10 (a mesma ausência do outro lado da porta).

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// O quadrado unitário centrado na origem — lado 2, de `(−1,−1)` a `(1,1)`.
///
/// ⚠️ Uma forma com aresta AXIAL e vértices exactos em `f32`: as distâncias que os
/// gates afirmam (`0.5`, `1.0`) saem sem épsilon, então um `assert_eq!` diz o que
/// quer dizer.
const SQUARE: [[f32; 2]; 4] = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];

// ─── A lei, medida directamente ──────────────────────────────────────────────

/// **A DISTÂNCIA É À FRONTEIRA, e a aresta de FECHO conta.**
///
/// ⚠️ Sem a aresta `(n−1)→0` o quadrado seria uma polilinha em `U`, e um ponto junto
/// ao lado esquerdo mediria a distância ao lado OPOSTO. O gate mede exactamente aí:
/// `(−0.9, 0)` está a `0.1` da aresta que fecha, e a `1.9` da que não fecharia.
#[test]
fn the_boundary_distance_includes_the_closing_edge() {
    let d = boundary_distance([-0.9, 0.0], &SQUARE).expect("há forma");
    assert!(
        (d - 0.1).abs() < 1e-6,
        "a aresta de fecho tem de existir, deu {d}"
    );
    // Dentro e fora à mesma distância: a fronteira não tem sinal.
    assert!((boundary_distance([0.0, 0.5], &SQUARE).unwrap() - 0.5).abs() < 1e-6);
    assert!((boundary_distance([0.0, 1.5], &SQUARE).unwrap() - 0.5).abs() < 1e-6);
}

/// **O INTERIOR É O INTERIOR** — e um vértice na linha do raio conta uma vez só.
///
/// ⚠️ O braço `(pi.y > y) != (pj.y > y)` é o que garante isso. Um `>=` de um lado
/// contaria duas vezes a aresta que TOCA a linha do raio, e o ponto sairia fora do
/// próprio polígono. O caso `y = 1.0` (a altura exacta de dois vértices) é o que o
/// mede.
#[test]
fn the_inside_test_counts_each_vertex_once() {
    assert!(inside_polygon([0.0, 0.0], &SQUARE), "o centro é dentro");
    assert!(!inside_polygon([2.0, 0.0], &SQUARE), "fora é fora");
    assert!(
        !inside_polygon([0.0, 1.0], &SQUARE),
        "a altura exacta do topo não pode contar duas vezes"
    );
    assert!(
        !inside_polygon([0.0, 0.0], &SQUARE[..2]),
        "dois pontos não têm interior"
    );
}

/// **`Filled Path`: dentro é SÓLIDO, fora decai. `Path Edges`: os dois lados decaem.**
///
/// É a diferença inteira entre os dois modos, e o ponto que a mede é o CENTRO — o
/// sítio mais longe da borda que ainda está dentro.
#[test]
fn filled_is_solid_inside_and_edges_hollow_it_out() {
    let filled = shape_mask([0.0, 0.0], &SQUARE, 0, 1.0, 0, false);
    let edges = shape_mask([0.0, 0.0], &SQUARE, 1, 1.0, 0, false);
    assert_eq!(filled, 1.0, "Filled Path: o miolo é cheio, exactamente");
    assert_eq!(edges, 0.0, "Path Edges: o centro está a 1.0 da borda ⇒ 0");
    // E FORA os dois modos concordam: o interior é a única coisa que os separa.
    for p in [[0.0_f32, 1.5], [2.0, 2.0]] {
        assert_eq!(
            shape_mask(p, &SQUARE, 0, 1.0, 0, false),
            shape_mask(p, &SQUARE, 1, 1.0, 0, false),
            "fora do polígono os dois modos são a MESMA lei"
        );
    }
}

/// **A PENUMBRA TEM A LARGURA QUE O KNOB DIZ, em unidades de mundo.**
#[test]
fn the_penumbra_is_exactly_the_authored_distance_wide() {
    // Linear, para que o número seja lido directo. `d = 0.5` fora da borda:
    // `distance = 1` ⇒ 0.5 · `distance = 0.5` ⇒ 0 · `distance = 2` ⇒ 0.75.
    let at = |dist: f32| shape_mask([0.0, 1.5], &SQUARE, 0, dist, 0, false);
    assert!((at(1.0) - 0.5).abs() < 1e-6);
    assert!(at(0.5).abs() < 1e-6, "no limite exacto a máscara acaba");
    assert!((at(2.0) - 0.75).abs() < 1e-6);
}

/// **`distance = 0` É UMA BORDA DURA, não uma divisão por zero.**
#[test]
fn a_zero_distance_is_a_hard_edge() {
    assert_eq!(shape_mask([0.0, 0.0], &SQUARE, 0, 0.0, 0, false), 1.0);
    let outside = shape_mask([0.0, 1.5], &SQUARE, 0, 0.0, 0, false);
    assert_eq!(outside, 0.0, "fora, com borda dura, é zero");
    assert!(outside.is_finite());
}

/// **A PORTA VAZIA É A IDENTIDADE — e nem o `invert` a mexe.**
///
/// ⚠️ Inverter *"nenhum campo"* continua a ser nenhum campo. Se o `invert` agisse
/// aqui, largar o nó com o toggle ligado apagaria a cena inteira antes de o artista
/// ter ligado uma forma — o pior primeiro contacto possível com um nó.
#[test]
fn an_unwired_shape_is_the_identity_even_inverted() {
    for invert in [false, true] {
        for mode in [0, 1] {
            assert_eq!(
                shape_mask([3.0, -7.0], &[], mode, 1.0, 2, invert),
                1.0,
                "sem forma, o campo não existe (mode {mode}, invert {invert})"
            );
        }
    }
}

/// **MENOS DE TRÊS PONTOS DEGRADA COM GRAÇA** — um ponto é um ponto, dois são um
/// segmento, e nenhum dos dois tem interior.
///
/// ⚠️ É o que impede uma forma a ser construída ao vivo de piscar para preto no
/// segundo vértice.
#[test]
fn fewer_than_three_points_still_measures_a_distance() {
    let dot = [[0.0_f32, 0.0]];
    assert!((boundary_distance([3.0, 4.0], &dot).unwrap() - 5.0).abs() < 1e-6);
    let seg = [[-1.0_f32, 0.0], [1.0, 0.0]];
    assert!((boundary_distance([0.0, 2.0], &seg).unwrap() - 2.0).abs() < 1e-6);
    // …e o `Filled Path` sobre eles é a MESMA coisa que o `Path Edges`.
    assert_eq!(
        shape_mask([0.0, 0.5], &seg, 0, 1.0, 0, false),
        shape_mask([0.0, 0.5], &seg, 1, 1.0, 0, false),
    );
}

/// **VÉRTICES COINCIDENTES NÃO DIVIDEM POR ZERO.**
#[test]
fn a_degenerate_edge_is_finite() {
    let dup = [[0.0_f32, 0.0], [0.0, 0.0], [1.0, 0.0]];
    let d = boundary_distance([0.0, 1.0], &dup).expect("há forma");
    assert!(d.is_finite() && (d - 1.0).abs() < 1e-6, "deu {d}");
}

// ─── A costura: cozinhar pelo grafo ──────────────────────────────────────────

static ELEMS: NodeManifest = NodeManifest {
    id: NodeTypeId::of("field.shape.test.elems"),
    name: "field.shape.test.elems",
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
/// Três peças: o centro do quadrado, meio caminho na penumbra, e bem longe.
struct Elems;
impl NodeOp for Elems {
    fn manifest(&self) -> &'static NodeManifest {
        &ELEMS
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(3).with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 1.5], [0.0, 9.0]])));
    }
}

static TEMPLATE: NodeManifest = NodeManifest {
    id: NodeTypeId::of("field.shape.test.square"),
    name: "field.shape.test.square",
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
struct Template;
impl NodeOp for Template {
    fn manifest(&self) -> &'static NodeManifest {
        &TEMPLATE
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(4).with("P", Column::Vec2(SQUARE.to_vec())));
    }
}

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == ELEMS.id => Some(&Elems),
            t if t == TEMPLATE.id => Some(&Template),
            t if t == MANIFEST.id => Some(&FieldShape),
            _ => None,
        }
    }
}

/// Cozinha `elems(3) → field.shape`, com a forma ligada ou não.
fn cook(wired: bool) -> (Vec<f32>, NodeId, Graph) {
    let mut g = Graph::new();
    let e = g.add_node("field.shape.test.elems");
    let fs = g.add_node("field.shape");
    g.set_param(fs, "distance", 1.0);
    g.set_param(fs, "curve", 0.0); // Linear, para o número ser lido directo
    g.connect(Edge {
        from: (e, 0),
        to: (fs, 0),
        delayed: false,
    })
    .unwrap();
    if wired {
        let t = g.add_node("field.shape.test.square");
        g.connect(Edge {
            from: (t, 0),
            to: (fs, 1),
            delayed: false,
        })
        .unwrap();
    }
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, fs, 0.0).unwrap();
    let s = out[0].as_stream();
    assert!(s.get("P").is_some(), "a geometria passa intacta");
    let f = match s.get("falloff").unwrap() {
        Column::Scalar(v) => v.clone(),
        _ => panic!("falloff"),
    };
    (f, fs, g)
}

/// **A GEOMETRIA DE UMA PORTA VIRA MÁSCARA — a célula, encenada.**
///
/// ⚠️ **O terceiro elemento é o controlo que importa:** ele está a 8 unidades da
/// borda, muito além da penumbra de 1. Sem ele, um campo que devolvesse `1` em todo o
/// lado (o bug de a porta não ser lida) passaria pelas duas primeiras asserções.
#[test]
fn a_shape_on_the_second_port_becomes_the_mask() {
    let (f, ..) = cook(true);
    assert_eq!(f[0], 1.0, "o centro está dentro: sólido");
    assert!((f[1] - 0.5).abs() < 1e-6, "a meio da penumbra");
    assert_eq!(f[2], 0.0, "longe da forma: nada");
}

/// **DESLIGADO, O NÓ NÃO MUDA UM BIT.**
#[test]
fn with_nothing_wired_the_mask_is_all_ones() {
    let (f, ..) = cook(false);
    assert_eq!(f, vec![1.0; 3]);
}

/// **O NÓ REGISTA, E DECLARA O QUE PRODUZ.**
///
/// ⚠️ Sem o `Produces("falloff")` o diagnoser da casa não sabe que este nó escreve uma
/// coluna transiente — e um `falloff` que ninguém consome é INERTE, que é exactamente
/// o defeito silencioso do ADR-0155. Ele é CPU-only, então não há `ColumnBinding` de
/// onde a derivação pudesse tirá-lo: a declaração é o único canal.
#[test]
fn the_node_registers_and_declares_the_column_it_produces() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
    let c = reg.couplings(MANIFEST.id).expect("o nó declara couplings");
    assert!(
        c.iter()
            .any(|k| matches!(k, ph2d_node_registry::Coupling::Produces("falloff"))),
        "tem de declarar a produção do `falloff`: {c:?}"
    );
}

/// **A MÁSCARA MULTIPLICA NO QUE JÁ LÁ ESTAVA** — o contrato MOPs que faz os campos
/// comporem, e o que separa um `field.*` de um `field.remap` (que REESCREVE).
#[test]
fn the_mask_multiplies_into_an_existing_falloff() {
    static PRE: NodeManifest = NodeManifest {
        id: NodeTypeId::of("field.shape.test.pre"),
        name: "field.shape.test.pre",
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
    struct Pre;
    impl NodeOp for Pre {
        fn manifest(&self) -> &'static NodeManifest {
            &PRE
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(2)
                    .with("P", Column::Vec2(vec![[0.0, 0.0], [0.0, 9.0]]))
                    .with("falloff", Column::Scalar(vec![0.25, 1.0])),
            );
        }
    }
    struct PreOps;
    impl OpResolver for PreOps {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == PRE.id => Some(&Pre),
                t if t == TEMPLATE.id => Some(&Template),
                t if t == MANIFEST.id => Some(&FieldShape),
                _ => None,
            }
        }
    }
    let mut g = Graph::new();
    let pre = g.add_node("field.shape.test.pre");
    let sq = g.add_node("field.shape.test.square");
    let fs = g.add_node("field.shape");
    g.set_param(fs, "distance", 1.0);
    for (a, b, port) in [(pre, fs, 0u16), (sq, fs, 1)] {
        g.connect(Edge {
            from: (a, 0),
            to: (b, port),
            delayed: false,
        })
        .unwrap();
    }
    let mut cook = Cook::new();
    let out = cook.cook(&g, &PreOps, fs, 0.0).unwrap();
    match out[0].as_stream().get("falloff").unwrap() {
        Column::Scalar(v) => {
            assert_eq!(v[0], 0.25, "0.25 × 1 (dentro) — o campo COMPÕE");
            assert_eq!(v[1], 0.0, "1 × 0 (longe) — e o produto pode zerar");
        }
        _ => panic!("falloff"),
    }
}

/// **O CUSTO É `O(n · m)`, e o número está MEDIDO** — `-- --ignored`, porque um
/// relógio não é um gate.
///
/// A alternativa (uma grelha espacial sobre as arestas) é infra a sério, e só se paga
/// se a medição a pedir. **Ela ainda não pediu** — medido em `--release`, nesta
/// workstation, num laço SERIAL (o nó corre `par_build`, então divida pelos cores):
///
/// | elementos | vértices | ms (serial) | ns/elemento |
/// |---|---|---|---|
/// | 65.536 | 8 | 1,38 | 21,0 |
/// | 65.536 | 32 | 5,51 | 84,1 |
/// | 65.536 | 64 | **10,42** | 159,0 |
/// | 65.536 | 256 | 40,91 | 624,3 |
///
/// ⚠️ **Linear no número de vértices, como o `O(n·m)` diz** — e a SOMA das máscaras é a
/// mesma nas quatro linhas, o que é o controlo: o losango amostrado mais fino é a mesma
/// forma, então amostrar melhor não pode mudar a resposta, só o relógio.
#[test]
#[ignore = "medição de relógio, não um gate — `-- --ignored`, máquina calma"]
fn measure_the_shape_field_cost() {
    let n = 65_536usize;
    for m in [8usize, 32, 64, 256] {
        // Um polígono regular de `m` lados, raio 4 — sem transcendentais no gate: os
        // vértices saem de um passeio poligonal simples.
        let poly: Vec<[f32; 2]> = (0..m)
            .map(|k| {
                #[expect(clippy::cast_precision_loss, reason = "índices pequenos")]
                let t = k as f32 / m as f32;
                // Um losango amostrado: |x| + |y| = 4, percorrido uniformemente.
                let u = t * 4.0;
                let (q, f) = (u as i32, u - u.floor());
                match q {
                    0 => [4.0 * (1.0 - f), 4.0 * f],
                    1 => [-4.0 * f, 4.0 * (1.0 - f)],
                    2 => [-4.0 * (1.0 - f), -4.0 * f],
                    _ => [4.0 * f, -4.0 * (1.0 - f)],
                }
            })
            .collect();
        #[expect(clippy::cast_precision_loss, reason = "índices")]
        let pts: Vec<[f32; 2]> = (0..n)
            .map(|i| [(i % 256) as f32 * 0.05 - 6.0, (i / 256) as f32 * 0.05 - 6.0])
            .collect();
        let t0 = std::time::Instant::now();
        let mut acc = 0.0_f32;
        for p in &pts {
            acc += shape_mask(*p, &poly, 0, 1.0, 2, false);
        }
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        #[expect(clippy::cast_precision_loss, reason = "uma contagem")]
        let per = ms * 1e6 / n as f64;
        println!(
            "[field.shape] {n} elementos x {m} vertices: {ms:.2} ms serial - {per:.1} ns/elemento (soma {acc})"
        );
    }
}
