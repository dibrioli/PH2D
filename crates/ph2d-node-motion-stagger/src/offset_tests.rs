//! Os gates do **Offset** — o deslize cíclico da rampa (grupo O).

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// Uma fonte de 5 instâncias na origem — a rampa É a saída.
static SRC5: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.stagger.test.off"),
    name: "motion.stagger.test.off",
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

/// A rampa que o NÓ produz — cozida pelo `Cook`, não por uma segunda cópia da
/// lei.
///
/// ⚠️ **A primeira versão desta função re-implementava o `eval`**, e a mutação
/// *"o `offset` é ignorado"* **SOBREVIVEU**: os gates mediam a minha cópia e não
/// o produto. É o oráculo-espelho que esta casa já documentou.
fn ramp(offset: f32) -> Vec<f32> {
    let mut g = Graph::new();
    let src = g.add_node("motion.stagger.test.off");
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
    g.set_param(st, "offset", offset);
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops5, st, 0.0).unwrap();
    match out[0].as_stream().get("P").unwrap() {
        Column::Vec2(v) => v.iter().map(|q| q[1]).collect(),
        _ => panic!("P"),
    }
}

/// **O NEUTRO NÃO TOCA A PONTA DA RAMPA.**
///
/// ⚠️ Este é o gate que a guarda `offset != 0.0` existe para satisfazer, e ele é
/// sobre **um elemento só**: o último senta exactamente em `1.0`, e `frac(1.0)`
/// é `0.0` — um `frac` aplicado sempre mandaria a peça do fim para o começo da
/// rampa **com o knob no neutro**. Os outros elementos não notariam.
#[test]
fn the_neutral_offset_leaves_the_end_of_the_ramp_alone() {
    let r = ramp(0.0);
    assert_eq!(r[0], 0.0, "a rampa começa em 0");
    assert_eq!(r[4], 1.0, "e ACABA em 1 — é aqui que um `frac` cego morde");
    for w in r.windows(2) {
        assert!(w[1] > w[0], "e ela é monótona: {r:?}");
    }
}

/// **O DESLIZE ROLA A RAMPA, E A ROLAGEM É EXACTA.**
///
/// ⚠️ **Duas afirmações minhas caíram aqui, e as duas eram sobre a mesma coisa:**
/// uma rampa INCLUSIVA (`0` … `1`, os dois extremos presentes) **não é um
/// ciclo** — rolar ciclicamente trata `0` e `1` como o mesmo ponto, então a
/// ponta cai em cima do começo. Escrevi *"uma volta inteira é o neutro"* (é
/// falso: com `offset = 1` o último elemento vai a `0`) e *"o alcance não
/// encolhe"* (também falso: com `offset = 0,5` o máximo passa de `1,0` a
/// `0,75`, porque o `1,0` deu a volta). O que é verdade — e o que o artista vê —
/// é que a rampa **ROLA**, e é isso que este gate afirma, ponto a ponto.
#[test]
fn the_offset_rolls_the_ramp_exactly() {
    let a = ramp(0.0);
    assert_eq!(a, vec![0.0, 0.25, 0.5, 0.75, 1.0], "a rampa de referência");
    let b = ramp(0.25);
    let want = [0.25_f32, 0.5, 0.75, 0.0, 0.25];
    for (i, (got, w)) in b.iter().zip(want).enumerate() {
        assert!(
            (got - w).abs() < 1e-6,
            "elemento {i}: {got} contra {w} — a rolagem é exacta, {b:?}"
        );
    }
}
