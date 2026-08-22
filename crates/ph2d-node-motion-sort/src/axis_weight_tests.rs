//! **O EIXO E O PESO** — os gates das duas células da folha 08 (a direção arbitrária e a
//! chave como campo). Irmão por `#[path]`, então `super` é a raiz da crate.

use super::*;

const O: [f32; 2] = [0.0, 0.0];

/// Quatro pontos em cruz, cada um com um `x`/`y` distinto — a fixture mais barata em que a
/// ordem por X, por Y e por uma diagonal são as TRÊS diferentes.
fn cross() -> Vec<[f32; 2]> {
    vec![[3.0, 1.0], [1.0, 3.0], [-2.0, 2.0], [2.0, -2.0]]
}

/// **`axis_angle = 0` É O MUNDO DE SEMPRE, NAS DUAS CHAVES.**
///
/// ⚠️ O gate afirma a igualdade contra o `p.x`/`p.y` CRU, não contra outra chamada da mesma
/// função com o mesmo ângulo: comparar `f(x)` consigo mesmo é um `assert` que não pode falhar.
#[test]
fn an_axis_angle_of_zero_is_the_bare_x_and_y() {
    let p = cross();
    let kx = keys(&p, KEY_X, O, 0, 0.0, &[]);
    let ky = keys(&p, KEY_Y, O, 0, 0.0, &[]);
    for (i, q) in p.iter().enumerate() {
        assert_eq!(kx[i].to_bits(), q[0].to_bits(), "X cru no ponto {i}");
        assert_eq!(ky[i].to_bits(), q[1].to_bits(), "Y cru no ponto {i}");
    }
}

/// **O ÂNGULO ROTACIONA O EIXO, e a 90° o `X` vira o `Y`** — é o que torna o param uma
/// generalização do enum em vez de um segundo eixo ao lado dele.
#[test]
fn ninety_degrees_turns_the_x_key_into_the_y_key() {
    let p = cross();
    let turned = permutation(&keys(&p, KEY_X, O, 0, 90.0, &[]), false, 0);
    let plain_y = permutation(&keys(&p, KEY_Y, O, 0, 0.0, &[]), false, 0);
    assert_eq!(turned, plain_y, "X girado 90° é o Y");

    // E o CONTROLE de que a fixture separa: uma DIAGONAL não é nenhum dos dois.
    let diag = permutation(&keys(&p, KEY_X, O, 0, 45.0, &[]), false, 0);
    let plain_x = permutation(&keys(&p, KEY_X, O, 0, 0.0, &[]), false, 0);
    assert_ne!(diag, plain_x, "a diagonal tem de diferir do X");
    assert_ne!(diag, plain_y, "e do Y — senão a cena não mostra nada novo");
}

/// **O ÂNGULO É INERTE FORA DAS DUAS CHAVES DE EIXO** — e é por isso que o painel o esconde
/// lá. Sem este gate o `ParamGate` seria uma opinião.
#[test]
fn the_axis_angle_does_nothing_to_the_other_keys() {
    let p = cross();
    for key in [KEY_RADIAL, KEY_RANDOM, 4] {
        let a = keys(&p, key, O, 7, 0.0, &[]);
        let b = keys(&p, key, O, 7, 33.0, &[]);
        assert_eq!(a, b, "a chave {key} não pode ouvir o ângulo");
    }
}

/// **A CHAVE COMO CAMPO ORDENA PELO PESO** — e a regra 1→N desta casa vale nela.
#[test]
fn the_weight_key_sorts_by_the_field() {
    let p = cross();
    // Um peso que INVERTE a ordem de chegada.
    let w = vec![9.0, 5.0, 1.0, 7.0];
    let perm = permutation(&keys(&p, KEY_WEIGHT, O, 0, 0.0, &w), false, 0);
    assert_eq!(perm, vec![2, 1, 3, 0], "a ordem é a dos pesos crescentes");

    // Um peso de UM valor vale para todos (broadcast) ⇒ empate ⇒ a ordem estável.
    let one = permutation(&keys(&p, KEY_WEIGHT, O, 0, 0.0, &[4.0]), false, 0);
    assert_eq!(
        one,
        vec![0, 1, 2, 3],
        "broadcast: todos iguais, ordem estável"
    );
}

/// **A PORTA DESLIGADA NO MODO `Weight` É A IDENTIDADE, NÃO O CAOS.**
///
/// ⚠️ E o par: nos OUTROS modos a porta é ignorada mesmo quando ligada — senão o `key` deixaria
/// de ser a resposta única a *"o que é a chave?"*, que é a razão de o modo existir em vez de
/// uma porta que vence em silêncio.
#[test]
fn an_unwired_weight_is_the_identity_and_a_wired_one_is_ignored_elsewhere() {
    let p = cross();
    let empty = permutation(&keys(&p, KEY_WEIGHT, O, 0, 0.0, &[]), false, 0);
    assert_eq!(
        empty,
        vec![0, 1, 2, 3],
        "sem coluna, a lista sai como entrou"
    );

    let w = vec![9.0, 5.0, 1.0, 7.0];
    for key in [KEY_RADIAL, KEY_X, KEY_Y, KEY_RANDOM, 4] {
        assert_eq!(
            keys(&p, key, O, 3, 0.0, &w),
            keys(&p, key, O, 3, 0.0, &[]),
            "a chave {key} não pode ler a porta de peso"
        );
    }
}

/// **A COSTURA: o `eval` de facto LÊ a porta 1 e o param do eixo.**
///
/// ⚠️ Os gates acima chamam `keys` à mão — eles provam a LEI. Uma lei correcta que o `eval`
/// nunca invoca (porque leu a porta errada, ou esqueceu o param) é o modo de falha clássico
/// desta casa, e só uma passagem pelo cook o pega.
#[test]
fn the_eval_reads_the_weight_port_and_the_axis_param() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    /// Quatro peças em cruz, marcadas por um `size` que diz quem é quem.
    struct Src;
    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.sort.axis.test.src"),
        name: "motion.sort.axis.test.src",
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
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(
                Stream::new(4)
                    .with("P", Column::Vec2(cross()))
                    .with("size", Column::Scalar(vec![0.0, 1.0, 2.0, 3.0])),
            );
        }
    }

    /// Um campo de valor com pesos que invertem a ordem de chegada.
    struct W;
    static W_MAN: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.sort.axis.test.w"),
        name: "motion.sort.axis.test.w",
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
    impl NodeOp for W {
        fn manifest(&self) -> &'static NodeManifest {
            &W_MAN
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            ctx.emit(Stream::new(4).with(VALUE_COL, Column::Scalar(vec![9.0, 5.0, 1.0, 7.0])));
        }
    }

    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            static S: Src = Src;
            static X: W = W;
            static SORT: MotionSort = MotionSort;
            match ty {
                t if t == SRC.id => Some(&S),
                t if t == W_MAN.id => Some(&X),
                t if t == MANIFEST.id => Some(&SORT),
                _ => None,
            }
        }
    }

    let run = |key: f32, axis: f32, wire_weight: bool| -> Vec<f32> {
        let mut g = Graph::new();
        let src = g.add_node("motion.sort.axis.test.src");
        let s = g.add_node("motion.sort");
        g.set_param(s, "key", key);
        g.set_param(s, AXIS_ANGLE, axis);
        g.connect(Edge {
            from: (src, 0),
            to: (s, 0),
            delayed: false,
        })
        .unwrap();
        if wire_weight {
            let w = g.add_node("motion.sort.axis.test.w");
            g.connect(Edge {
                from: (w, 0),
                to: (s, 1),
                delayed: false,
            })
            .unwrap();
        }
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, s, 0.0).unwrap();
        match out[0].as_stream().get("size") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        }
    };

    // O eixo: `X` a 0° contra `X` a 90° — e o segundo tem de bater o `Y` a 0°.
    let x0 = run(KEY_X as f32, 0.0, false);
    let x90 = run(KEY_X as f32, 90.0, false);
    let y0 = run(KEY_Y as f32, 0.0, false);
    assert_ne!(x0, x90, "o `eval` tem de LER o `axis_angle`");
    assert_eq!(x90, y0, "e 90° tem de dar o `Y`");

    // O peso: ligado no modo `Weight` manda; nos outros a porta é ignorada.
    assert_eq!(
        run(KEY_WEIGHT as f32, 0.0, true),
        vec![2.0, 1.0, 3.0, 0.0],
        "o `eval` tem de LER a porta 1 no modo Weight"
    );
    assert_eq!(
        run(KEY_X as f32, 0.0, true),
        x0,
        "e ignorá-la fora dele — o `key` é a resposta única"
    );
}
