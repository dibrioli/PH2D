//! Os gates do **peso por entrada** ([`super::WEIGHTS`]) — doc 89, folha 08.
//!
//! ⚠️ **O gate mais importante deste arquivo não é aritmético.** O nó descarta as entradas
//! vazias antes de reduzir, então um peso indexado pela POSIÇÃO na lista de contribuintes
//! valeria para outra porta assim que um fio fosse desligado — e isso é invisível numa
//! fixture com as quatro entradas ligadas, que é a fixture que qualquer um escreveria.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

fn snap_p(p: Vec<[f32; 2]>) -> Snap {
    Snap {
        count: p.len(),
        cols: vec![("P".to_string(), Column::Vec2(p))],
    }
}

fn xs(s: &Stream) -> Vec<f32> {
    match s.get("P").expect("P") {
        Column::Vec2(v) => v.iter().map(|q| q[0]).collect(),
        _ => panic!("P"),
    }
}

/// **A MÉDIA PONDERADA NORMALIZA PELA SOMA DOS PESOS.**
///
/// ⚠️ O oráculo é escolhido para falsificar a normalização errada: `0·1 + 4·3 = 12`, e o que
/// se afirma é `3` (dividido por `Σw = 4`) — dividir pela CONTAGEM daria `6`, e não dividir
/// daria `12`. Três respostas distintas para os mesmos dois pontos.
#[test]
fn the_weighted_average_divides_by_the_sum_of_the_weights() {
    let a = snap_p(vec![[0.0, 0.0]]);
    let b = snap_p(vec![[4.0, 0.0]]);
    assert_eq!(xs(&mix(MODE_AVG, &[&a, &b], &[], &[1.0, 3.0])), vec![3.0]);
    assert_eq!(
        xs(&mix(MODE_AVG, &[&a, &b], &[], &[3.0, 1.0])),
        vec![1.0],
        "e o peso é de quem o carrega, não do lugar"
    );
}

/// **A SOMA PONDERADA NÃO NORMALIZA** — senão seria uma média com outro nome.
#[test]
fn the_weighted_sum_stays_a_sum() {
    let a = snap_p(vec![[0.0, 0.0]]);
    let b = snap_p(vec![[4.0, 0.0]]);
    assert_eq!(xs(&mix(MODE_ADD, &[&a, &b], &[], &[1.0, 3.0])), vec![12.0]);
}

/// **TODOS OS PESOS A `1` É O QUE SEMPRE FOI, AO BIT** — e o oráculo é a lei ANTIGA, escrita
/// aqui em duas linhas, não um número que eu copiei da saída de hoje.
#[test]
fn all_weights_at_one_reproduce_the_law_that_shipped() {
    let cases = [
        vec![[0.1, 0.0], [2.7, 1.0]],
        vec![[-3.3, 4.0], [0.0, -1.5]],
        vec![[1e-7, 1e7], [0.3, 0.3]],
    ];
    for c in cases {
        let (a, b) = (snap_p(c.clone()), snap_p(vec![[7.7, 0.9]; 2]));
        let old_mean: Vec<f32> = c.iter().map(|q| (q[0] + 7.7) * 0.5).collect();
        let old_sum: Vec<f32> = c.iter().map(|q| q[0] + 7.7).collect();
        assert_eq!(xs(&mix(MODE_AVG, &[&a, &b], &[], &[1.0, 1.0])), old_mean);
        assert_eq!(xs(&mix(MODE_ADD, &[&a, &b], &[], &[1.0, 1.0])), old_sum);
    }
}

/// **`Σ w = 0` É ZERO, NUNCA `NaN`.**
///
/// ⚠️ Um `0/0` põe `NaN` na posição de cada elemento, e um `NaN` em `P` desenha nada — a cena
/// desaparece e nada no ecrã diz porquê. Zero lê-se como *"desliguei tudo"*: todas as peças na
/// origem, visível e explicável.
#[test]
fn every_weight_at_zero_is_the_origin_not_a_nan() {
    let a = snap_p(vec![[3.0, 0.0]]);
    let b = snap_p(vec![[5.0, 0.0]]);
    let got = xs(&mix(MODE_AVG, &[&a, &b], &[], &[0.0, 0.0]));
    assert_eq!(got, vec![0.0]);
    assert!(got.iter().all(|x| x.is_finite()), "nada de NaN: {got:?}");
}

/// **O `Blend` IGNORA OS PESOS** — ali quem responde *"quanto de cada um?"* é o campo `blend`.
#[test]
fn blend_is_deaf_to_the_weights() {
    let a = snap_p(vec![[0.0, 0.0]]);
    let b = snap_p(vec![[4.0, 0.0]]);
    let flat = xs(&mix(MODE_BLEND, &[&a, &b], &[0.25], &[1.0, 1.0]));
    let skew = xs(&mix(MODE_BLEND, &[&a, &b], &[0.25], &[9.0, 0.1]));
    assert_eq!(flat, skew, "o campo blend é a única porta neste modo");
    assert_eq!(flat, vec![1.0]);
    // …e o painel esconde os quatro, para não haver duas portas VISÍVEIS para uma pergunta.
    for w in WEIGHTS {
        let gate = PARAM_GATES
            .iter()
            .find(|g| g.param == w)
            .unwrap_or_else(|| panic!("{w} tem de ser gateado"));
        assert!(
            !gate.values.contains(&(MODE_BLEND as i32)),
            "{w} não pode aparecer no Blend"
        );
        assert!(gate.values.contains(&(MODE_AVG as i32)), "{w} vale no Avg");
    }
}

/// **O PESO SEGUE A PORTA, NÃO A POSIÇÃO NA LISTA DE LIGADOS** — o gate de costura, ao nível
/// do cozimento, com as portas `in1`/`in2` VAZIAS.
///
/// ⚠️ Este é o defeito que uma fixture cheia esconde: com só `in0` e `in3` ligados, o segundo
/// contribuinte é a porta **3**. Um peso lido por posição usaria o `weight_1` ali — e o
/// artista veria o slider errado responder. O controle prova as duas direções: mexer no
/// `weight_3` muda a resposta, e mexer no `weight_1` **não** muda.
#[test]
fn a_weight_belongs_to_its_port_even_when_the_ones_before_it_are_empty() {
    const fn src(id: &'static str) -> NodeManifest {
        NodeManifest {
            id: NodeTypeId::of(id),
            name: id,
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: INST_VEC2,
            }],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[ParamSpec {
                name: "x",
                default: 0.0,
            }],
            lowerings: &[LoweringKind::Cpu],
        }
    }
    static SRC: NodeManifest = src("motion.mixer.test.wsrc");
    struct Src;
    impl NodeOp for Src {
        fn manifest(&self) -> &'static NodeManifest {
            &SRC
        }
        fn eval(&self, ctx: &mut EvalCtx<'_>) {
            let x = ctx.param("x");
            ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[x, 0.0]])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionMixer),
                _ => None,
            }
        }
    }
    let mut reg = NodeRegistry::new();
    register(&mut reg).expect("regista");

    let run = |w1: f32, w3: f32| {
        let mut g = Graph::new();
        let a = g.add_node("motion.mixer.test.wsrc");
        g.set_param(a, "x", 0.0);
        let d = g.add_node("motion.mixer.test.wsrc");
        g.set_param(d, "x", 4.0);
        let m = g.add_node("motion.mixer");
        g.set_param(m, WEIGHTS[1], w1);
        g.set_param(m, WEIGHTS[3], w3);
        // ⚠️ As portas 1 e 2 ficam VAZIAS — é isso que a fixture existe para produzir.
        for (from, port) in [(a, 0u16), (d, 3)] {
            g.connect(Edge {
                from: (from, 0),
                to: (m, port),
                delayed: false,
            })
            .expect("liga");
        }
        let mut cook = Cook::new();
        xs(cook.cook(&g, &Ops, m, 0.0).expect("coze")[0].as_stream())
    };
    assert_eq!(run(1.0, 3.0), vec![3.0], "o peso da porta 3 é que pesa");
    assert_eq!(
        run(9.0, 3.0),
        vec![3.0],
        "e o `weight_1` — o segundo POSICIONAL — não pode tocar em nada: a porta 1 está vazia"
    );
    assert_eq!(run(1.0, 1.0), vec![2.0], "o controle positivo: o peso mexe");
}
