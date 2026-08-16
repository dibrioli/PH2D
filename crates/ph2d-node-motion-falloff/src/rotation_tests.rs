//! **O ÂNGULO do campo** (doc 89 folha 10, o P0 do `field.linear`).
//!
//! A rampa Linear só sabia ir na HORIZONTAL — o kernel ramificava em `dx` cru — e
//! o Rect só existia alinhado aos eixos. A célula media as duas voltas tentadas e
//! reprovadas (`field.box(rotation) + invert` dá platô com rampa **simétrica**,
//! não gradiente monótono; um `field.radial_sweep` gigante e distante para no
//! `radius = 40`) e concluía: *"a cura mais barata não é nó novo, é o
//! `motion.falloff` ganhar o `rotation` que o `field.box` já tem"*.

use super::*;

/// A máscara num ponto, com a rotação dada — pela mesma aritmética do `eval`.
fn mask(shape: i32, rotation: f32, px: f32, py: f32) -> f32 {
    let (rc, rs) = trig::cos_sin_cycles(rotation / 360.0);
    let (dx, dy) = (px * rc + py * rs, -px * rs + py * rc);
    field(shape, dx, dy, 5.0, 0, false)
}

/// **`rotation = 0` é o nó que sempre shipou, AO BIT.**
///
/// ⚠️ `cos_sin_cycles(0)` é `(1, 0)` EXATO, então `px·1 + py·0` é `px` em
/// IEEE-754 — a identidade é estrutural, não uma tolerância.
#[test]
fn zero_rotation_is_the_node_that_always_shipped() {
    for shape in [0, 1, 2] {
        for (px, py) in [(1.7, -0.4), (-3.3, 2.9), (0.0, 0.0), (4.9, 4.9)] {
            let want = field(shape, px, py, 5.0, 0, false);
            assert_eq!(
                mask(shape, 0.0, px, py).to_bits(),
                want.to_bits(),
                "shape {shape} em ({px}, {py})"
            );
        }
    }
}

/// **A rampa vai para onde o ângulo aponta.**
///
/// A 90° a rampa Linear — que era horizontal por construção — passa a variar no
/// eixo **Y** e a ficar constante no X. É o P0 inteiro numa asserção.
#[test]
fn the_linear_ramp_follows_the_angle() {
    const LINEAR: i32 = 2;
    // Horizontal (o nó de sempre): varia em x, constante em y.
    let h0 = mask(LINEAR, 0.0, -4.0, 0.0);
    let h1 = mask(LINEAR, 0.0, 4.0, 0.0);
    assert!((h1 - h0).abs() > 0.5, "a rampa de sempre varia em x");
    assert!(
        (mask(LINEAR, 0.0, 0.0, -4.0) - mask(LINEAR, 0.0, 0.0, 4.0)).abs() < 1e-4,
        "e e constante em y"
    );
    // A 90°: os dois eixos TROCAM.
    let v0 = mask(LINEAR, 90.0, 0.0, -4.0);
    let v1 = mask(LINEAR, 90.0, 0.0, 4.0);
    assert!(
        (v1 - v0).abs() > 0.5,
        "a 90 graus a rampa varia em Y, deu {v0} e {v1}"
    );
    assert!(
        (mask(LINEAR, 90.0, -4.0, 0.0) - mask(LINEAR, 90.0, 4.0, 0.0)).abs() < 1e-4,
        "e passa a ser constante em x"
    );
}

/// **A rampa é MONÓTONA em qualquer ângulo** — a propriedade que separa isto das
/// duas voltas que a célula tentou e reprovou.
///
/// ⚠️ `field.box(rotation) + invert` dá um platô com rampa **simétrica** dos dois
/// lados; um gradiente é monótono ao longo da sua direção, e é isso que se afirma
/// aqui — passo a passo ao longo do eixo do campo, o valor nunca desce.
#[test]
fn the_ramp_is_monotone_along_its_own_axis_at_any_angle() {
    const LINEAR: i32 = 2;
    for deg in [0.0f32, 17.0, 45.0, 90.0, 143.0, -60.0] {
        let (rc, rs) = trig::cos_sin_cycles(deg / 360.0);
        let mut prev = f32::MIN;
        for k in -20..=20i8 {
            let t = f32::from(k) * 0.25;
            // Anda ao longo do EIXO do campo, não do eixo do mundo.
            let (px, py) = (t * rc, t * rs);
            let v = mask(LINEAR, deg, px, py);
            assert!(v >= prev - 1e-5, "a {deg} graus a rampa desceu em t = {t}");
            prev = v;
        }
        // E ela de facto percorre a faixa inteira, senão "monótona" seria vácuo.
        let lo = mask(LINEAR, deg, -6.0 * rc, -6.0 * rs);
        let hi = mask(LINEAR, deg, 6.0 * rc, 6.0 * rs);
        assert!(lo < 0.01 && hi > 0.99, "a {deg} graus a faixa e {lo}..{hi}");
    }
}

/// **A caixa gira junto** — o `Rect` deixa de ser alinhado aos eixos.
#[test]
fn the_rect_turns_too() {
    const RECT: i32 = 1;
    // Um ponto na diagonal, FORA da caixa alinhada (Chebyshev 4.2 > radius 5? não
    // — mas dentro), e o que muda é a distância de Chebyshev sob rotação.
    let p = (4.5, 0.0);
    let a = mask(RECT, 0.0, p.0, p.1);
    let b = mask(RECT, 45.0, p.0, p.1);
    assert!(
        (a - b).abs() > 1e-3,
        "girar a caixa muda a mascara na mesma posicao: {a} contra {b}"
    );
}

/// **O ângulo do `motion.falloff` é o MESMO do `field.box`** — os dois são campos
/// espaciais que o artista gira, e um `30°` que significasse ângulos diferentes
/// seria a falha de duas portas na sua forma mais quieta: nada na tela diria qual
/// está certo.
///
/// ⚠️ **O gate é ESTRUTURAL, não numérico**, e é de propósito: a
/// `cos_sin_cycles` do `field.box` é `pub(crate)`, e expô-la só para um teste
/// abriria superfície pública para provar uma coisa que o TEXTO já diz. Aqui o
/// `trig.rs` deste nó é comparado com o de lá, do qual foi copiado verbatim — um
/// gate numérico ficaria verde com os dois a derivar juntos, este falha alto no
/// dia em que alguém editar um só.
///
/// ⚠️ **E o repo tem 21 cópias deste arquivo.** Medido em 2026-08-12, os CORPOS
/// são idênticos (só testes e docs diferem), então não há divergência hoje — há
/// vinte e uma chances de uma. Unificá-las é wave própria; este gate pina o par
/// que ESTE P0 tornou load-bearing.
#[test]
fn the_angle_means_the_same_thing_as_the_field_box() {
    fn body(src: &str) -> String {
        // Só o código, e só até o módulo de testes (que difere de propósito).
        let code = src.split("#[cfg(test)]").next().unwrap_or(src);
        code.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    }
    let ours = body(include_str!("trig.rs"));
    let theirs = body(include_str!("../../ph2d-node-field-box/src/trig.rs"));
    assert!(!ours.is_empty(), "o controle: o arquivo tem codigo");
    assert_eq!(
        ours, theirs,
        "o `trig.rs` deste no e uma copia verbatim do do `field.box` -- \
         um `30 graus` que significasse coisas diferentes nos dois nao teria sintoma"
    );
}

/// **E o ÂNGULO chega pelo `eval` do nó, não só pela aritmética.**
///
/// ⚠️ **Este gate existe porque a mutação "a CPU não gira" passou nos cinco
/// acima.** Eles medem a máscara por um helper local que faz a rotação ele
/// mesmo — um ESPELHO da sequência, que fica verde com o produto quieto. O que
/// prova o nó é cozinhá-lo.
#[test]
fn the_angle_reaches_the_cooked_column() {
    use ph2d_nodegraph::cook::{Cook, OpResolver};
    use ph2d_nodegraph::graph::{Edge, Graph};

    static SRC: NodeManifest = NodeManifest {
        id: NodeTypeId::of("motion.falloff.rot.src"),
        name: "motion.falloff.rot.src",
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
            // Dois pontos: um no eixo X, um no eixo Y, à mesma distância.
            ctx.emit(Stream::new(2).with("P", Column::Vec2(vec![[4.0, 0.0], [0.0, 4.0]])));
        }
    }
    struct Ops;
    impl OpResolver for Ops {
        fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
            match ty {
                t if t == SRC.id => Some(&Src),
                t if t == MANIFEST.id => Some(&MotionFalloff),
                _ => None,
            }
        }
    }
    let cook_at = |deg: f32| -> Vec<f32> {
        let mut g = Graph::new();
        let src = g.add_node("motion.falloff.rot.src");
        let f = g.add_node("motion.falloff");
        g.set_param(f, "shape", 2.0); // Linear
        g.set_param(f, "radius", 5.0);
        g.set_param(f, "rotation", deg);
        g.connect(Edge {
            from: (src, 0),
            to: (f, 0),
            delayed: false,
        })
        .expect("in");
        let mut c = Cook::new();
        match c.cook(&g, &Ops, f, 0.0).expect("coze")[0]
            .as_stream()
            .get("falloff")
        {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        }
    };
    // A 0° a rampa é horizontal: o ponto em x = 4 é ALTO, o em y = 4 fica no meio.
    let flat = cook_at(0.0);
    assert!(
        flat[0] > 0.85 && (flat[1] - 0.5).abs() < 1e-4,
        "0 graus: {flat:?}"
    );
    // A 90° os dois TROCAM — e é isto que a mutação "a CPU não gira" quebra.
    let turned = cook_at(90.0);
    assert!(
        turned[1] > 0.85 && (turned[0] - 0.5).abs() < 1e-4,
        "90 graus: os eixos trocam, deu {turned:?}"
    );
}
