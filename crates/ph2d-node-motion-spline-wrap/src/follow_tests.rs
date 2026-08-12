//! **A GEOMETRIA SEGUE O FRAME** (doc 89 folha 04 — o P0, e a folha diz que é *"a
//! omissão mais VISÍVEL da família"*: texto numa curva que não gira lê como
//! quebrado).
//!
//! ⚠️ A célula fechava com *"não há precedente no catálogo. Verificado:
//! `motion.distribute_curve` também não escreve `rot`"* — e essa metade **já era
//! falsa** quando alguém foi construir isto: o irmão ganhou `align` em
//! `dfd7bf895`, no mesmo dia da folha. O precedente existe, e é por isso que estes
//! gates medem CONCORDÂNCIA com ele em vez de uma lei nova.

use super::*;
use ph2d_nodegraph::cook::{Cook, OpResolver};
use ph2d_nodegraph::graph::{Edge, Graph};

/// Uma fila horizontal de quatro, com uma rotação PRÓPRIA de 30° — a coluna que
/// prova se o nó compõe ou atropela.
static SRC: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.spline_wrap.follow.src"),
    name: "motion.spline_wrap.follow.src",
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "with_rot",
            default: 1.0,
        },
        // ⚠️ A máscara por elemento deste nó é a COLUNA `falloff`; o `amount` é
        // uma PORTA de valor (um dial global), e a primeira versão deste arquivo
        // tentou dirigi-lo com `set_param` — que não faz nada. O gate falhou
        // ALTO em vez de passar em silêncio, que é para o que ele existe.
        ParamSpec {
            name: "mask",
            default: -1.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};
struct Src;
impl NodeOp for Src {
    fn manifest(&self) -> &'static NodeManifest {
        &SRC
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let mut s = Stream::new(4).with(
            "P",
            Column::Vec2(vec![[0.0, 0.0], [1.0, 0.0], [2.0, 0.0], [3.0, 0.0]]),
        );
        if ctx.param("with_rot") != 0.0 {
            s.set("rot", Column::Scalar(vec![30.0; 4]));
        }
        let m = ctx.param("mask");
        if m >= 0.0 {
            s.set("falloff", Column::Scalar(vec![m; 4]));
        }
        ctx.emit(s);
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        match ty {
            t if t == SRC.id => Some(&Src),
            t if t == MANIFEST.id => Some(&MotionSplineWrap),
            _ => None,
        }
    }
}

/// Embrulha a fila num arco e devolve o stream cozido.
fn wrapped(follow: f32, mask: Option<f32>, with_rot: bool) -> Stream {
    let mut g = Graph::new();
    let src = g.add_node("motion.spline_wrap.follow.src");
    g.set_param(src, "with_rot", f32::from(u8::from(with_rot)));
    g.set_param(src, "mask", mask.unwrap_or(-1.0));
    let sw = g.add_node("motion.spline_wrap");
    g.set_param(sw, "follow_rotation", follow);
    // Um arco pronunciado: a tangente varre um bom ângulo do começo ao fim.
    for (k, v) in [
        ("p0x", -3.0),
        ("p0y", 0.0),
        ("p1x", -1.0),
        ("p1y", 4.0),
        ("p2x", 1.0),
        ("p2y", 4.0),
        ("p3x", 3.0),
        ("p3y", 0.0),
    ] {
        g.set_param(sw, k, v);
    }
    g.connect(Edge {
        from: (src, 0),
        to: (sw, 0),
        delayed: false,
    })
    .expect("in");
    let mut cook = Cook::new();
    let out = cook.cook(&g, &Ops, sw, 0.0).expect("cook");
    out[0].as_stream().clone()
}

fn rot_of(s: &Stream) -> Option<Vec<f32>> {
    match s.get("rot") {
        Some(Column::Scalar(v)) => Some(v.clone()),
        _ => None,
    }
}

/// **Desligado, o `rot` atravessa INTOCADO — e não escrito com o mesmo valor.**
///
/// ⚠️ A distinção é a que torna a identidade estrutural: com o toggle em zero a
/// coluna é COPIADA pelo mesmo laço que copia todas as outras, então um stream que
/// nunca teve `rot` continua sem ter. Escrevê-la com o valor de antes passaria
/// neste gate e criaria uma coluna do nada no outro.
#[test]
fn with_the_toggle_off_the_rotation_column_is_passed_through_untouched() {
    let s = wrapped(0.0, None, true);
    assert_eq!(
        rot_of(&s),
        Some(vec![30.0; 4]),
        "a rotacao propria sobrevive"
    );

    let bare = wrapped(0.0, None, false);
    assert!(
        rot_of(&bare).is_none(),
        "e um stream SEM rot continua sem -- a coluna nao e inventada"
    );
}

/// **Ligado, cada elemento vira para onde a curva vai.**
///
/// O arco sobe e desce, então o começo tem tangente subindo (ângulo positivo), o
/// meio é horizontal e o fim desce (negativo) — a assinatura de *seguir*, e não a
/// de um ângulo constante que um `set` preguiçoso daria.
#[test]
fn with_the_toggle_on_each_element_turns_with_the_curve() {
    let s = wrapped(1.0, None, false);
    let r = rot_of(&s).expect("ligado, o nó escreve a rotacao");
    assert!(r[0] > 15.0, "o comeco do arco sobe: {r:?}");
    assert!(r[3] < -15.0, "e o fim desce: {r:?}");
    assert!(
        r[0] > r[1] && r[1] > r[2] && r[2] > r[3],
        "e a volta e MONOTONICA ao longo da curva, nao um valor unico: {r:?}"
    );
}

/// **A rotação própria COMPÕE com a da curva** — este nó é um modificador sobre um
/// layout que já pode estar orientado, não uma fonte que cunha a orientação.
///
/// ⚠️ É onde ele DIVERGE do irmão de propósito: o `motion.distribute_curve` faz
/// `set` porque não há nada com que compor. Somar aqui e atribuir lá são a mesma
/// regra — *a rotação da curva entra no que já existe* — vista dos dois lados.
#[test]
fn the_elements_own_rotation_composes_with_the_curves() {
    let bare = rot_of(&wrapped(1.0, None, false)).expect("rot");
    let turned = rot_of(&wrapped(1.0, None, true)).expect("rot");
    for (i, (b, t)) in bare.iter().zip(&turned).enumerate() {
        assert!(
            (t - (b + 30.0)).abs() < 1e-4,
            "elemento {i}: {t} deveria ser {b} + 30 (a rotacao propria)"
        );
    }
}

/// **A MÁSCARA (a coluna `falloff`) vale para a volta tanto quanto para a
/// posição.**
///
/// ⚠️ Esta é a metade que carrega o peso. Um elemento meio-embrulhado tem de estar
/// **meio-virado**: mascarar a posição e não a rotação deixaria um sprite em pé
/// numa curva que ele só está a montar em parte, e o falloff — a razão inteira de
/// a máscara existir — leria como quebrado exactamente onde está a funcionar.
#[test]
fn a_half_wrapped_element_is_half_turned() {
    let full = rot_of(&wrapped(1.0, Some(1.0), false)).expect("rot");
    let half = rot_of(&wrapped(1.0, Some(0.5), false)).expect("rot");
    let none = rot_of(&wrapped(1.0, Some(0.0), false)).expect("rot");
    for (i, r) in none.iter().enumerate() {
        assert!(
            r.abs() < 1e-5,
            "falloff 0 nao vira nada (elemento {i}): {none:?}"
        );
    }
    for i in 0..4 {
        assert!(
            (half[i] - full[i] * 0.5).abs() < 1e-4,
            "elemento {i}: metade da mascara e metade da volta -- {} contra {}",
            half[i],
            full[i] * 0.5
        );
    }
}

/// **E a tangente é a MESMA aproximação do irmão que já orienta.**
///
/// ⚠️ Os dois nós que põem coisas ao longo de uma curva são os dois que têm de
/// concordar sobre o que *"seguir a curva"* significa. O `trig.rs` aqui é espelho
/// verbatim do dele; uma segunda aproximação daria dois ângulos para a mesma
/// tangente, e a diferença apareceria como duas metades de uma cena a rodar de
/// formas ligeiramente diferentes — o defeito que ninguém liga a um `atan2`.
#[test]
fn the_tangent_angle_is_the_same_approximation_the_sibling_uses() {
    // Os oito octantes, contra os valores CONHECIDOS — sem chamar o `atan2` do
    // std, que seria um oraculo transcendental num nó que e HR-5.
    for (y, x, want) in [
        (0.0f32, 1.0f32, 0.0f32),
        (1.0, 1.0, 45.0),
        (1.0, 0.0, 90.0),
        (1.0, -1.0, 135.0),
        (0.0, -1.0, 180.0),
        (-1.0, -1.0, -135.0),
        (-1.0, 0.0, -90.0),
        (-1.0, 1.0, -45.0),
    ] {
        let got = trig::deg(trig::atan2_approx(y, x));
        assert!(
            (got - want).abs() < 0.2,
            "atan2({y},{x}) = {want} graus, deu {got}"
        );
    }
}
