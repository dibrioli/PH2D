//! Os gates da **NONA família: a ease que o artista DESENHA** (doc 89, folha 06).
//!
//! ⚠️ Arquivo próprio por TETO DE LOC (HR-18, 700 para `crates/`), o mesmo corte que
//! o irmão `offset_tests.rs` já usa: a lei no `lib.rs`, as provas por assunto.
//!
//! ## As três afirmações, e por que nenhuma sozinha basta
//!
//! 1. **Com curva, a rampa é a curva.** Sem isto, um `Custom` que ignorasse o text
//!    param leria a identidade e passaria despercebido — a saída ficaria plausível.
//! 2. **Sem curva, é o `Linear` AO BIT.** Sem isto, um `Custom` que devolvesse `0`
//!    numa curva ausente seria um controle morto (doc 90), e a lei *"o default
//!    reduzido é o mundo de antes"* deixaria de valer para a família nova.
//! 3. **A DIREÇÃO não a toca.** Sem isto, o `ease_dir` espelharia em silêncio o que
//!    o artista desenhou, e o que está na tela deixaria de ser o desenho dele.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// A curva das provas — um V invertido: `V(t) = 2t` até `½`, `2(1−t)` depois.
///
/// ⚠️ Ela **não é monótona**, e isso é a escolha: as oito famílias enumeradas são
/// todas monótonas (o `Back` sai da faixa mas continua a subir no fim), então uma
/// saída que a confundisse com qualquer uma delas é impossível. Uma curva monótona
/// deixaria o gate incapaz de separar *"leu a minha curva"* de *"caiu num `Quad`"*.
const CURVE_V: &str = "c1 0:0:L 0.5:1:L 1:0:L";

/// Cinco instâncias na origem — a rampa É a saída.
static SRC5: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.stagger.test.curve"),
    name: "motion.stagger.test.curve",
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
struct Src5;
impl NodeOp for Src5 {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC5
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(5).with("P", Column::Vec2(vec![[0.0, 0.0]; 5])));
    }
}
struct Ops5;
impl OpResolver for Ops5 {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC5.id => Some(&Src5),
            t if t == MANIFEST.id => Some(&MotionStagger),
            _ => None,
        }
    }
}

/// A rampa `0 → 1` que o NÓ produz, cozida pelo `Cook`.
///
/// ⚠️ **Pelo produto, nunca por uma segunda cópia da lei** — é a lição que o
/// `offset_tests.rs` registou: um oráculo-espelho deixa a mutação sobreviver.
fn ramp(setup: impl FnOnce(&mut Graph, NodeId)) -> Vec<f32> {
    let mut g = Graph::new();
    let src = g.add_node("motion.stagger.test.curve");
    let st = g.add_node("motion.stagger");
    g.connect(Edge {
        from: (src, 0),
        to: (st, 0),
        delayed: false,
    })
    .unwrap();
    g.set_param(st, "channel", 1.0); // Y
    g.set_param(st, "min", 0.0);
    g.set_param(st, "max", 1.0);
    setup(&mut g, st);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops5, st, 0.0).unwrap();
    match out[0].as_stream().get("P").unwrap() {
        Column::Vec2(v) => v.iter().map(|p| p[1]).collect(),
        _ => panic!("P"),
    }
}

fn custom(g: &mut Graph, st: NodeId) {
    g.set_param(st, "ease_curve", ease::EASE_CUSTOM as f32);
}

/// **A EASE CUSTOM É A CURVA AUTORADA.**
#[test]
fn the_custom_ease_is_the_authored_shape() {
    let got = ramp(|g, st| {
        custom(g, st);
        g.set_text_param(st, CURVE_KEY, CURVE_V);
    });
    // Cinco elementos ⇒ `raw = 0, ¼, ½, ¾, 1` ⇒ o V dá `0, ½, 1, ½, 0`.
    let want = [0.0, 0.5, 1.0, 0.5, 0.0];
    for (i, (a, b)) in got.iter().zip(&want).enumerate() {
        assert!(
            (a - b).abs() < 1e-6,
            "elemento {i}: deu {a}, a curva pede {b} (rampa {got:?})"
        );
    }
    // ⚠️ E o CONTROLE de que ela não é uma das enumeradas: nenhuma das oito volta a
    // descer. Sem esta linha, o gate acima passaria numa implementação que caísse
    // num `Quad` por acaso da fixture.
    assert!(
        got[4] < got[2],
        "a curva desenhada DESCE no fim — nenhuma familia enumerada faz isso"
    );
}

/// **SEM CURVA, A CUSTOM É O `Linear` — AO BIT.**
///
/// A lei do default reduzido, e a que impede a família nova de nascer como um
/// controle morto: escolher `Custom` sem desenhar nada não pode congelar a fileira.
#[test]
fn an_unset_custom_ease_is_the_linear_ramp_bit_for_bit() {
    let linear = ramp(|_, _| {});
    let bare = ramp(custom);
    assert_eq!(
        linear, bare,
        "curva ausente = o Linear, ao bit (e nunca um zero morto)"
    );
    // O controle POSITIVO: a fixture de facto anda — senão a igualdade acima seria
    // dois campos parados a concordarem.
    assert!(
        linear[4] - linear[0] > 0.9,
        "a rampa de controle percorre a faixa: {linear:?}"
    );
}

/// **A DIREÇÃO NÃO TOCA NA CURVA DESENHADA** — nem quando ela está autorada.
///
/// ⚠️ As três direções têm de dar a MESMA rampa. Uma que espelhasse faria o
/// artista ver algo que ele não desenhou, com o painel a esconder o culpado (o
/// `ease_dir` está gateado fora nesta família).
#[test]
fn the_custom_ease_ignores_the_direction() {
    let at = |dir: f32| {
        ramp(|g, st| {
            custom(g, st);
            g.set_text_param(st, CURVE_KEY, CURVE_V);
            g.set_param(st, "ease_dir", dir);
        })
    };
    let base = at(0.0);
    assert_eq!(base, at(1.0), "In e Out dao a mesma curva desenhada");
    assert_eq!(base, at(2.0), "e In-Out tambem");
    // ⚠️ O controle que impede isto de passar por vacuidade: numa família
    // ENUMERADA a direção MUDA a rampa. Sem ele, um `ease_dir` inteiramente morto
    // (em todas as famílias) passaria neste gate.
    let quad = |dir: f32| {
        ramp(|g, st| {
            g.set_param(st, "ease_curve", 1.0); // Quad
            g.set_param(st, "ease_dir", dir);
        })
    };
    assert_ne!(
        quad(0.0),
        quad(1.0),
        "controle: numa familia enumerada a direcao MUDA a rampa"
    );
}

/// **O PAINEL OFERECE A FAMÍLIA NOVA, E ESCONDE A DIREÇÃO NELA.**
///
/// ⚠️ Os dois lados da costura, contra a mesma fonte: um `EASE_CUSTOM` que a `eval`
/// entende e o painel não oferece é um gesto inalcançável; e um `ease_dir` VISÍVEL
/// numa família que o ignora é o knob morto que o doc 90 caçou — *o mesmo defeito
/// que já custou uma wave neste nó, na posição em que ele é o primeiro gesto.*
#[test]
fn the_panel_offers_the_custom_family_and_hides_the_direction_in_it() {
    let row = PARAM_HINTS
        .iter()
        .find(|h| h.param == "ease_curve")
        .expect("a linha da ease");
    let ParamWidget::Enum { labels } = row.widget else {
        panic!("a ease é um seletor nomeado")
    };
    assert_eq!(
        labels.len() as i32 - 1,
        ease::EASE_CUSTOM,
        "a ultima etiqueta é a Custom"
    );
    assert_eq!(labels[ease::EASE_CUSTOM as usize], "Custom");
    assert!((row.max - ease::EASE_CUSTOM as f32).abs() < f32::EPSILON);

    // A linha da CURVA existe — senão a família nova nasce sem como ser desenhada.
    let curve_row = PARAM_HINTS
        .iter()
        .find(|h| h.param == CURVE_KEY)
        .expect("a linha da curva");
    assert!(matches!(curve_row.widget, ParamWidget::Curve));

    // E o gate da direção NÃO lista a Custom (nem o Linear).
    let dir_gate = PARAM_GATES
        .iter()
        .find(|g| g.param == "ease_dir")
        .expect("o gate da direcao");
    assert!(
        !dir_gate.values.contains(&ease::EASE_CUSTOM),
        "a Custom ignora a direcao, entao ela nao pode aparecer"
    );
    assert!(!dir_gate.values.contains(&0), "nem o Linear");
    // O controle: ele lista as sete que de facto a usam.
    assert_eq!(dir_gate.values, &[1, 2, 3, 4, 5, 6, 7]);
}
