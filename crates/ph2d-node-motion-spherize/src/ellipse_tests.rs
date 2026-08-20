//! Os gates do [`super::RADIUS_Y`] — a lente elíptica (doc 89, folha 04).

use super::*;

/// Uma cruz de quatro pontos a distância `d` do centro, nos quatro sentidos.
fn cross(d: f32) -> Vec<[f32; 2]> {
    vec![[d, 0.0], [-d, 0.0], [0.0, d], [0.0, -d]]
}

/// O deslocamento de cada ponto (o quanto a lente o empurrou).
fn push(base: &[[f32; 2]], out: &[[f32; 2]]) -> Vec<f32> {
    base.iter()
        .zip(out)
        .map(|(p, q)| ((q[0] - p[0]).powi(2) + (q[1] - p[1]).powi(2)).sqrt())
        .collect()
}

/// **`0` É O CÍRCULO DE SEMPRE, E PELO CAMINHO LITERAL.**
///
/// ⚠️ A prova é a SIMETRIA: numa lente redonda os quatro braços da cruz são empurrados por
/// exactamente o mesmo número, ao bit — não «quase». Um `radius_y` igual ao `radius` passaria
/// pela métrica nova e daria um número ε diferente, que é precisamente porque o sentinela
/// existe.
#[test]
fn zero_is_the_round_lens_and_it_takes_the_literal_path() {
    let c = cross(1.0);
    let out = spherize(&c, 0.5, 3.0, 0.0, [0.0, 0.0], &[]);
    let d = push(&c, &out);
    assert!(d[0] > 0.0, "a lente tem de empurrar: {d:?}");
    for x in &d {
        assert_eq!(*x, d[0], "os quatro braços têm de ser o MESMO f32: {d:?}");
    }
}

/// **A LENTE ELÍPTICA EMPURRA ANISOTROPICAMENTE** — mais no eixo estreito.
///
/// ⚠️ E é a metade que nenhuma máscara alcança: um `field.box` mascararia a MISTURA, e o
/// deslocamento continuaria radial. Aqui o eixo com o raio menor tem `t` maior, então a rampa
/// `1 − t²` dá-lhe menos — o par horizontal e o vertical saem com empurrões diferentes.
#[test]
fn an_elliptical_lens_pushes_the_two_axes_differently() {
    let c = cross(1.0);
    let out = spherize(&c, 0.5, 3.0, 1.5, [0.0, 0.0], &[]);
    let d = push(&c, &out);
    assert!(
        (d[0] - d[1]).abs() < 1e-6,
        "o par horizontal é simétrico: {d:?}"
    );
    assert!((d[2] - d[3]).abs() < 1e-6, "o par vertical também: {d:?}");
    assert!(
        d[0] > d[2] + 1e-3,
        "o eixo LARGO (radius 3) tem de ser empurrado mais que o estreito (1,5): {d:?}"
    );
}

/// **O CONTORNO DA LENTE É UMA ELIPSE** — dentro/fora deixa de ser um círculo.
///
/// ⚠️ Este é o gate que separa *"o empurrão mudou"* de *"a REGIÃO mudou"*. Um ponto a `2` no
/// eixo Y está DENTRO de uma lente redonda de raio `3` e FORA de uma elipse `3 × 1,5`.
#[test]
fn the_lens_boundary_becomes_an_ellipse() {
    // ⚠️ **A fixture é SIMÉTRICA de propósito.** O centro da lente é o CENTROIDE do layout: com
    // um ponto só, o centroide é esse ponto e ele nunca se move — o gate ficaria verde sobre
    // nada. Aqui os quatro pontos põem o centroide na origem, que é onde a lente tem de estar.
    let p = cross(2.0);
    let round = spherize(&p, 0.5, 3.0, 0.0, [0.0, 0.0], &[]);
    let d = push(&p, &round);
    assert!(d[2] > 0.0, "redonda: o ponto a (0, 2) está dentro: {d:?}");
    let flat = spherize(&p, 0.5, 3.0, 1.5, [0.0, 0.0], &[]);
    let e = push(&p, &flat);
    assert_eq!(
        e[2], 0.0,
        "elíptica: o mesmo ponto está FORA e não pode mover-se"
    );
    assert!(e[0] > 0.0, "…e o do eixo largo continua dentro: {e:?}");
}

/// **O KNOB ESTÁ NO PAINEL, o `0` é alcançável, e o device sabe dele.**
#[test]
fn the_knob_is_painted_and_uploaded() {
    let h = PARAM_HINTS
        .iter()
        .find(|h| h.param == RADIUS_Y)
        .expect("o Radius Y tem de estar pintado");
    assert_eq!(h.min, 0.0, "o sentinela do círculo tem de caber no curso");
    assert!(
        GPU_KERNEL.params.contains(&RADIUS_Y),
        "o device tem de receber o raio vertical: {:?}",
        GPU_KERNEL.params
    );
}
