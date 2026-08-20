//! Os gates do `value.cursor`.
//!
//! ⚠️ **Este é o primeiro nó do repo com DUAS saídas**, então a primeira coisa que se prova
//! não é a aritmética — é que a segunda porta existe, coze, e traz outro número. Um gate que
//! só lesse a porta `0` passaria por igual num nó que emitisse `x` duas vezes.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::Graph;

struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        (ty == MANIFEST.id).then_some(&ValueCursor as &dyn NodeOp)
    }
}

/// Cozinha um `value.cursor` sozinho com o cursor publicado em `at` (ou sem canal nenhum) e
/// devolve as duas saídas.
fn run(at: Option<[f32; 2]>) -> (Vec<f32>, Vec<f32>) {
    let mut g = Graph::new();
    let n = g.add_node("value.cursor");
    let mut cook = Cook::new();
    if let Some(p) = at {
        cook.set_external(
            ph2d_nodegraph::external::CURSOR,
            Stream::new(1).with("P", Column::Vec2(vec![p])),
        );
    }
    let out = cook.cook(&g, &Ops, n, 0.0).expect("coze");
    let col = |k: usize| match out[k].as_stream().get(VALUE_COL) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    };
    (col(0), col(1))
}

/// **AS DUAS PORTAS COZEM, E NÃO SÃO O MESMO NÚMERO.**
#[test]
fn both_ports_cook_and_they_are_not_the_same_number() {
    let (x, y) = run(Some([3.5, -1.25]));
    assert_eq!(x, vec![3.5], "a porta 0 é o X do cursor");
    assert_eq!(y, vec![-1.25], "a porta 1 é o Y");
    // ⚠️ O controle explícito: um nó que emitisse `x` duas vezes passaria nas duas linhas
    // acima se o oráculo fosse simétrico. Aqui não é, e é de propósito.
    assert_ne!(x, y, "as duas saídas têm de ser dois números");
}

/// **A CONTAGEM SEGUE A GEOMETRIA** — desligada, o global de comprimento 1.
///
/// ⚠️ É o comprimento que um param dirigido lê (ele toma o primeiro valor), e o que a regra
/// de broadcast desta casa segura para todos os elementos.
#[test]
fn unconnected_is_the_length_one_global() {
    let (x, y) = run(Some([1.0, 2.0]));
    assert_eq!(x.len(), 1, "um valor global");
    assert_eq!(y.len(), 1);
}

/// **A CONTAGEM SEGUE A GEOMETRIA (a outra metade)** — ligada, um campo do tamanho da lista,
/// com o mesmo número em todas as linhas.
#[test]
fn connected_is_a_field_as_long_as_the_geometry() {
    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("value.cursor.test.src"),
        name: "value.cursor.test.src",
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
            ctx.emit(Stream::new(4).with("P", Column::Vec2(vec![[0.0, 0.0]; 4])));
        }
    }
    struct Both;
    impl OpResolver for Both {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == MANIFEST.id => Some(&ValueCursor),
                _ => None,
            }
        }
    }
    let mut g = Graph::new();
    let src = g.add_node("value.cursor.test.src");
    let cur = g.add_node("value.cursor");
    g.connect(ph2d_nodegraph::graph::Edge {
        from: (src, 0),
        to: (cur, 0),
        delayed: false,
    })
    .expect("liga");
    let mut cook = Cook::new();
    cook.set_external(
        ph2d_nodegraph::external::CURSOR,
        Stream::new(1).with("P", Column::Vec2(vec![[9.0, 8.0]])),
    );
    let out = cook.cook(&g, &Both, cur, 0.0).expect("coze");
    match out[0].as_stream().get(VALUE_COL) {
        Some(Column::Scalar(v)) => assert_eq!(*v, vec![9.0; 4], "quatro linhas, o mesmo X"),
        other => panic!("v: {other:?}"),
    }
}

/// **SEM CANAL, ZERO — e parado.**
///
/// ⚠️ Um host sem rato, ou um cozimento de teste, não tem `$cursor`. A alternativa (não
/// emitir coluna) faria um param dirigido cair no default do param, e o centro **saltaria**
/// de volta ao número autorado no primeiro quadro sem publicação. Zero é uma posição de mundo
/// real e fica quieta.
#[test]
fn an_absent_channel_is_the_origin_and_it_holds_still() {
    let (x, y) = run(None);
    assert_eq!(x, vec![0.0]);
    assert_eq!(y, vec![0.0]);
    // E o controle: com canal, NÃO é zero — senão o gate acima passaria sobre um nó morto.
    let (lx, ly) = run(Some([4.0, 5.0]));
    assert_eq!((lx, ly), (vec![4.0], vec![5.0]));
}

/// **O CURSOR MEXEU ⇒ O NÓ RECOZE** — a razão de o `Effect::Pure` ser seguro aqui.
///
/// ⚠️ Sem isto o desenho inteiro cai: um nó puro sem entradas é exactamente o que um memo
/// serve do cache para sempre. O que o salva é o cozimento guardar **quais externals o nó leu
/// e em que revisão** (doc 65) — este gate prova esse acoplamento no MESMO `Cook`, que é onde
/// o memo vive; um `Cook` novo por leitura não provaria nada.
#[test]
fn moving_the_cursor_re_cooks_the_pure_node() {
    let mut g = Graph::new();
    let n = g.add_node("value.cursor");
    let mut cook = Cook::new();
    let read = |cook: &mut Cook, g: &Graph| match cook.cook(g, &Ops, n, 0.0).expect("coze")[0]
        .as_stream()
        .get(VALUE_COL)
    {
        Some(Column::Scalar(v)) => v.first().copied().unwrap_or(f32::NAN),
        _ => f32::NAN,
    };
    cook.set_external(
        ph2d_nodegraph::external::CURSOR,
        Stream::new(1).with("P", Column::Vec2(vec![[1.0, 0.0]])),
    );
    assert_eq!(read(&mut cook, &g), 1.0);
    cook.set_external(
        ph2d_nodegraph::external::CURSOR,
        Stream::new(1).with("P", Column::Vec2(vec![[7.0, 0.0]])),
    );
    assert_eq!(
        read(&mut cook, &g),
        7.0,
        "o memo tem de invalidar quando o external que o nó leu muda de revisão"
    );
}

/// **VÁRIOS PONTOS PUBLICADOS DÃO A MÉDIA**, não o primeiro por acidente.
#[test]
fn several_published_points_average_instead_of_the_first_winning() {
    let mut g = Graph::new();
    let n = g.add_node("value.cursor");
    let mut cook = Cook::new();
    cook.set_external(
        ph2d_nodegraph::external::CURSOR,
        Stream::new(2).with("P", Column::Vec2(vec![[0.0, 0.0], [4.0, 10.0]])),
    );
    let out = cook.cook(&g, &Ops, n, 0.0).expect("coze");
    let first = |k: usize| match out[k].as_stream().get(VALUE_COL) {
        Some(Column::Scalar(v)) => v[0],
        _ => f32::NAN,
    };
    assert_eq!((first(0), first(1)), (2.0, 5.0));
}
