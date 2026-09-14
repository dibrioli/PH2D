//! Os gates da porta da deformação do canvas — ver [`super`].

use super::{WarpedDab, warped_dab};

const I: [[f32; 2]; 2] = [[1.0, 0.0], [0.0, 1.0]];

/// ⭐⭐⭐ **Em repouso a porta é o NO-OP, e não «quase»** — é isso que mantém toda pincelada deste app
/// byte a byte como era. O controlo é uma `warp` que não é a identidade: ali ela TEM de mexer.
#[test]
fn at_rest_the_door_is_a_bit_exact_no_op() {
    for (flatten, angle) in [(0.0, 0u16), (0.4, 30), (0.9, 271)] {
        assert_eq!(
            warped_dab(I, flatten, angle),
            WarpedDab {
                radius_scale: 1.0,
                flatten,
                angle_deg: angle
            },
            "em repouso a porta devolve o que entrou"
        );
    }
    // ⛔ O CONTROLO: sem ele, uma porta que devolvesse sempre a entrada passaria no gate acima.
    let esticada = warped_dab([[2.0, 0.0], [0.0, 1.0]], 0.0, 0);
    assert!(
        esticada != WarpedDab::identity(),
        "uma deformacao real TEM de mover os numeros: {esticada:?}"
    );
}

/// ⭐⭐⭐ **O caso da foto: a arte comprimida num eixo.** Com o ecrã a ver `⅓` da textura em `x`, o
/// disco de ecrã é, na textura, uma elipse **3× mais larga** em `x` — logo o raio cresce `3×` e o
/// achatamento é `1 − ⅓`, com o eixo MAIOR em `x`.
#[test]
fn a_squeezed_axis_paints_a_stretched_ellipse() {
    let w = warped_dab([[1.0 / 3.0, 0.0], [0.0, 1.0]], 0.0, 0);
    assert!(
        (w.radius_scale - 3.0).abs() < 1e-5,
        "o raio segue o eixo MAIOR: {w:?}"
    );
    assert!(
        (w.flatten - (1.0 - 1.0 / 3.0)).abs() < 1e-5,
        "o menor mede um terco do maior: {w:?}"
    );
    assert!(
        w.angle_deg == 0 || w.angle_deg == 180,
        "o eixo maior e' o x: {w:?}"
    );
}

/// ⭐⭐ **A resposta é a elipse que o ecrã vê REDONDA** — a régua é o produto, não os três números:
/// levar a elipse pintada pela deformação tem de devolver um círculo.
#[test]
fn the_painted_ellipse_comes_back_round_on_screen() {
    // Uma deformação com corte (o que um triângulo dobrado de facto faz).
    let warp = [[0.5, 0.25], [0.0, 1.5]];
    let d = warped_dab(warp, 0.0, 0);
    let [c, s] = crate::texture::rotate_by_degrees(d.angle_deg);
    let (maior, menor) = (d.radius_scale, d.radius_scale * (1.0 - d.flatten));
    // Os dois semi-eixos da elipse pintada, levados ao ecrã pela deformação.
    let leva = |v: [f32; 2]| {
        [
            warp[0][0] * v[0] + warp[0][1] * v[1],
            warp[1][0] * v[0] + warp[1][1] * v[1],
        ]
    };
    let a = leva([c * maior, s * maior]);
    let b = leva([-s * menor, c * menor]);
    let (la, lb) = (
        (a[0] * a[0] + a[1] * a[1]).sqrt(),
        (b[0] * b[0] + b[1] * b[1]).sqrt(),
    );
    assert!(
        (la - 1.0).abs() < 2e-2 && (lb - 1.0).abs() < 2e-2,
        "os dois eixos tinham de chegar ao ecra' com o mesmo comprimento 1: {la} e {lb} ({d:?})"
    );
    // ⚠️ A folga de `2e-2` é a QUANTIZAÇÃO do ângulo em graus inteiros (o `dab_angle_deg` é `u16` e
    // escolhe uma entrada da tabela cozida): meio grau sobre um eixo de comprimento 1 vale `~9e-3`.
}

/// ⛔ **Uma malha dobrada sobre si mesma não pede um dab infinito** — determinante nulo devolve a
/// entrada, e o traço continua a ser o que o artista pediu.
#[test]
fn a_collapsed_triangle_is_refused_not_amplified() {
    let d = warped_dab([[1.0, 2.0], [0.5, 1.0]], 0.3, 45);
    assert_eq!(
        d,
        WarpedDab {
            radius_scale: 1.0,
            flatten: 0.3,
            angle_deg: 45
        }
    );
}
