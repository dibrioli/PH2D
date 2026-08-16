use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

/// Uma fonte de 6 instâncias na origem — o wiggle É a saída.
static SRC_MAN2: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.wiggle.test.oct"),
    name: "motion.wiggle.test.oct",
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
struct Src2;
impl NodeOp for Src2 {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC_MAN2
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        ctx.emit(Stream::new(6).with("P", Column::Vec2(vec![[0.0, 0.0]; 6])));
    }
}
struct Ops2;
impl OpResolver for Ops2 {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC_MAN2.id => Some(&Src2),
            t if t == MANIFEST.id => Some(&MotionWiggle),
            _ => None,
        }
    }
}

fn rig() -> (Graph, NodeId) {
    let mut g = Graph::new();
    let src = g.add_node("motion.wiggle.test.oct");
    let w = g.add_node("motion.wiggle");
    g.connect(Edge {
        from: (src, 0),
        to: (w, 0),
        delayed: false,
    })
    .unwrap();
    (g, w)
}

/// A componente Y de cada elemento no instante `t`.
fn ys(g: &Graph, w: NodeId, t: f64) -> Vec<f32> {
    let mut c = Cook::new();
    let v = c.cook(g, &Ops2, w, t).expect("coze");
    match v[0].as_stream().get("P") {
        Some(Column::Vec2(p)) => p.iter().map(|q| q[1]).collect(),
        _ => panic!("sem P"),
    }
}

/// **UMA OITAVA É O CAMPO QUE SHIPAVA — BIT A BIT.**
///
/// O oráculo é a expressão congelada (`value_noise_2d(t·f, i+seed) · amp`), não
/// uma segunda chamada ao produto. ⚠️ Ela vale por ARITMÉTICA e não por
/// promessa: com uma oitava o `eval` da folha devolve `sum/total` com
/// `total = 1.0`, e `n / 1.0` é `n` em IEEE-754; o deslocamento de oitava é
/// `o = 0` ⇒ `x + 0.0`, que também é exacto.
#[test]
fn one_octave_is_the_field_that_shipped() {
    let (mut g, w) = rig();
    g.set_param(w, "channel", 1.0);
    g.set_param(w, "amplitude", 1.7);
    g.set_param(w, "frequency", 0.9);
    g.set_param(w, "seed", 3.0);
    let t = 0.37_f32;
    let got = ys(&g, w, t as f64);
    for (i, y) in got.iter().enumerate() {
        let want = noise::value_noise_2d(t * 0.9, i as f32 + 3.0) * 1.7;
        assert_eq!(
            y.to_bits(),
            want.to_bits(),
            "elemento {i}: {y} != {want} (o default TEM de reduzir ao bit)"
        );
    }
}

/// **AS OITAVAS ACRESCENTAM DETALHE** — e o CONTROLE é a metade que impede
/// *"qualquer coisa mudou"* de passar por *"o fractal chegou"*.
#[test]
fn the_octaves_add_detail_and_one_octave_is_the_control() {
    let (mut g, w) = rig();
    g.set_param(w, "channel", 1.0);
    g.set_param(w, "frequency", 0.9);
    let one = ys(&g, w, 0.37);
    g.set_param(w, "octaves", 4.0);
    let four = ys(&g, w, 0.37);
    let moved = one
        .iter()
        .zip(&four)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        moved > 1e-3,
        "quatro oitavas TÊM de mover o campo: {moved:.6}"
    );
    // O controle: re-pedir uma oitava devolve o campo do começo, ao bit.
    g.set_param(w, "octaves", 1.0);
    let again = ys(&g, w, 0.37);
    for (a, b) in one.iter().zip(&again) {
        assert_eq!(a.to_bits(), b.to_bits(), "uma oitava não é um estado novo");
    }
}

/// **O MULTIPLICADOR É INERTE NUMA OITAVA E VIVO EM QUATRO** — as duas metades,
/// porque só a segunda prova que ele chega ao campo e só a primeira prova que o
/// default não o deixa mexer no mundo de antes.
#[test]
fn the_amp_multiplier_is_inert_at_one_octave_and_alive_at_four() {
    let (mut g, w) = rig();
    g.set_param(w, "channel", 1.0);
    let a = ys(&g, w, 0.41);
    g.set_param(w, "amp_mult", 0.9);
    let b = ys(&g, w, 0.41);
    for (x, y) in a.iter().zip(&b) {
        assert_eq!(
            x.to_bits(),
            y.to_bits(),
            "com UMA oitava o multiplicador não tem uma segunda oitava a multiplicar"
        );
    }
    g.set_param(w, "octaves", 4.0);
    let c = ys(&g, w, 0.41);
    g.set_param(w, "amp_mult", 0.1);
    let d = ys(&g, w, 0.41);
    let moved = c
        .iter()
        .zip(&d)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0f32, f32::max);
    assert!(
        moved > 1e-3,
        "com QUATRO ele decide o peso das oitavas: {moved:.6}"
    );
}

/// **O LAÇO FECHA NO TEMPO** — e o CONTROLE é o mesmo campo SEM laço, que não
/// fecha. Sem ele, *"as duas pontas batem"* seria satisfeito por um campo
/// constante.
#[test]
fn the_loop_closes_and_the_open_field_does_not() {
    let (mut g, w) = rig();
    g.set_param(w, "channel", 1.0);
    g.set_param(w, "frequency", 1.3);
    let l = 2.0_f64;
    let open_a = ys(&g, w, 0.0);
    let open_b = ys(&g, w, l);
    let open = open_a
        .iter()
        .zip(&open_b)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    assert!(
        open > 1e-3,
        "o CONTROLE: sem laço as pontas NÃO batem ({open:.6})"
    );
    g.set_param(w, "loop_len", l as f32);
    let a = ys(&g, w, 0.0);
    let b = ys(&g, w, l);
    for (i, (x, y)) in a.iter().zip(&b).enumerate() {
        assert!(
            (x - y).abs() < 1e-5,
            "elemento {i}: a volta tem de fechar — {x} contra {y}"
        );
    }
}

/// **AS OITAVAS NÃO CAEM TODAS NO MESMO CANTO DA GRADE.**
///
/// ⚠️ Este é o gate que o deslocamento de oitava existe para satisfazer, e ele é
/// sobre o CANTO DEGENERADO — não sobre o eixo. Em `t = 0` com `seed = 0` o
/// elemento `0` tem `px = 0` e `py = 0`, e a lacunaridade multiplica zero por
/// dois: sem deslocamento **todas** as oitavas amostram o mesmo canto, a soma
/// colapsa numa oitava só e o valor fica preso em `-1.0` — o extremo do alcance.
/// Uma peça atirada ao extremo no instante zero de toda reprodução é visível.
///
/// ⚠️ **A fixture TEM de estar no canto** (`t = 0`, `seed = 0`, o elemento `0`):
/// em qualquer outro ponto a escala por oitava já separa as linhas, e a mutação
/// sobrevive — foi o que aconteceu com a primeira versão deste gate.
#[test]
fn the_octaves_do_not_all_land_on_the_same_lattice_corner() {
    let (mut g, w) = rig();
    g.set_param(w, "channel", 1.0);
    g.set_param(w, "amplitude", 1.0);
    g.set_param(w, "seed", 0.0);
    let one = ys(&g, w, 0.0)[0];
    assert!(
        (one + 1.0).abs() < 1e-5,
        "o CONTROLE: uma oitava no canto É o hash do canto, {one}"
    );
    g.set_param(w, "octaves", 4.0);
    let four = ys(&g, w, 0.0)[0];
    assert!(
        (four - one).abs() > 0.1,
        "quatro oitavas no canto degenerado não podem colapsar na primeira: \
         {four} contra {one}"
    );
}
