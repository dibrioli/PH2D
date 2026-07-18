//! Gates do Twist e do Pucker & Bloat. Cada um tem UM botão, então o que há a provar é que ele
//! faz o que diz e que o neutro não custa nada.

use super::*;
use crate::VecPath;

/// Um quadrado de lado 40 na origem — de propósito NÃO centrado no mundo, para os gates
/// distinguirem "o centro da forma" de "a origem".
fn square() -> VecPath {
    VecPath {
        verts: [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

fn ctx() -> FxCtx {
    FxCtx::of(&square())
}

/// A distância de cada âncora ao centro da forma.
fn radii(v: &[VecVertex], c: &FxCtx) -> Vec<f64> {
    v.iter()
        .map(|w| (w.anchor[0] - c.center[0]).hypot(w.anchor[1] - c.center[1]))
        .collect()
}

/// **Os dois neutros são no-op byte-idêntico** — o que a pilha exige de todo efeito.
#[test]
fn both_neutral_points_are_byte_identical_no_ops() {
    let p = square();
    let c = ctx();
    assert_eq!(
        twist_contour(&p.verts, true, &TwistSpec::default(), &c).0,
        p.verts
    );
    assert_eq!(
        bloat_contour(&p.verts, true, &BloatSpec::default(), &c).0,
        p.verts
    );
}

/// **O Twist GIRA e não muda o raio** — cada ponto anda sobre a sua circunferência.
///
/// É o gate que separa um giro de uma escala: se o campo tivesse componente radial, os raios
/// mudariam. E o centro é o da FORMA — num quadrado de `0..40` ele é `(20,20)`, longe da origem.
#[test]
fn the_twist_turns_without_changing_any_radius() {
    let p = square();
    let c = ctx();
    let before = radii(&p.verts, &c);
    let out = twist_contour(&p.verts, true, &TwistSpec { angle: 90.0 }, &c).0;
    let after = radii(&out, &c);
    for (i, (a, b)) in before.iter().zip(after.iter()).enumerate() {
        assert!(
            (a - b).abs() < 1e-9,
            "o vértice {i} mudou de raio: {a} -> {b}. Girar não pode mexer na distância."
        );
    }
    assert!(
        out.iter()
            .zip(&p.verts)
            .any(|(o, q)| (o.anchor[0] - q.anchor[0]).hypot(o.anchor[1] - q.anchor[1]) > 1.0),
        "e alguma coisa tem de se ter MEXIDO"
    );
}

/// **A força do Twist cresce com a DISTÂNCIA** — é isto, e só isto, que o separa de uma rotação
/// rígida.
///
/// ⚠️ A 1ª versão deste gate media um ponto colocado NO centro e exigia que ele não andasse.
/// Ficava verde com o campo errado: o centro é ponto fixo de **qualquer** rotação em torno dele,
/// rígida ou não. A mutação que troca a força variável por uma constante sobreviveu, e o
/// culpado era o fixture — estava no único sítio onde as duas leis concordam.
///
/// O oráculo tem de comparar DOIS raios diferentes, ambos não-nulos: o de fora tem de girar
/// mais. O centro parado fica como asserção secundária, agora que não é a única.
#[test]
fn the_twists_force_grows_with_distance() {
    let c = ctx();
    // Raios 5 e 15 a partir do centro, no mesmo eixo: a razão dos ângulos girados tem de ser a
    // razão dos raios (o campo é linear em `t`), e numa rotação rígida seria 1.
    let pts = vec![
        VecVertex::corner(c.center),
        VecVertex::corner([c.center[0] + 5.0, c.center[1]]),
        VecVertex::corner([c.center[0] + 15.0, c.center[1]]),
    ];
    let out = twist_contour(&pts, false, &TwistSpec { angle: 60.0 }, &c).0;
    let turned = |i: usize| -> f64 {
        let (dx, dy) = (
            out[i].anchor[0] - c.center[0],
            out[i].anchor[1] - c.center[1],
        );
        dy.atan2(dx)
    };
    let d0 = (out[0].anchor[0] - c.center[0]).hypot(out[0].anchor[1] - c.center[1]);
    assert!(d0 < 1e-9, "o ponto no centro andou {d0}");
    let (near, far) = (turned(1), turned(2));
    assert!(near > 1e-6, "o ponto de dentro não girou nada ({near} rad)");
    assert!(
        (far / near - 3.0).abs() < 1e-6,
        "o ponto três vezes mais longe devia girar três vezes mais: {near} e {far} rad          (razão {}). Razão 1 significa rotação RÍGIDA, que não é um twist.",
        far / near
    );
}

/// **O Bloat escala o raio pela percentagem pedida**, e o sinal decide a direção.
#[test]
fn the_bloat_scales_every_radius_by_the_amount() {
    let p = square();
    let c = ctx();
    let before = radii(&p.verts, &c);
    for (amount, k) in [(100.0, 2.0), (-50.0, 0.5)] {
        let out = bloat_contour(&p.verts, true, &BloatSpec { amount }, &c).0;
        for (a, b) in before.iter().zip(radii(&out, &c).iter()) {
            assert!(
                (b - a * k).abs() < 1e-9,
                "com {amount}% o raio {a} devia virar {}, deu {b}",
                a * k
            );
        }
    }
}

/// **O raio de quina do Bloat escala junto** — ele é um comprimento LOCAL, e o campo aqui é uma
/// escala uniforme. Sem isto uma quina arredondada mudaria de proporção ao inflar a forma.
#[test]
fn the_bloat_scales_the_corner_radius_too() {
    let c = ctx();
    let mut v = VecVertex::corner([40.0, 40.0]);
    v.corner_radius = 6.0;
    let out = bloat_contour(&[v], false, &BloatSpec { amount: 100.0 }, &c).0;
    assert!(
        (out[0].corner_radius - 12.0).abs() < 1e-9,
        "o raio devia duplicar com a forma, deu {}",
        out[0].corner_radius
    );
}

/// Uma forma degenerada (sem tamanho) não tem centro nem raio — o Twist é **inerte**, não um
/// `NaN` que se espalha por toda a geometria a jusante.
#[test]
fn a_degenerate_shape_leaves_the_twist_inert() {
    let dot: Vec<VecVertex> = (0..4).map(|_| VecVertex::corner([3.0, 7.0])).collect();
    let c = FxCtx::of(&VecPath {
        verts: dot.clone(),
        closed: true,
        ..VecPath::default()
    });
    assert_eq!(
        twist_contour(&dot, true, &TwistSpec { angle: 90.0 }, &c).0,
        dot
    );
}
