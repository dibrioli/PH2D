//! Os gates do [`super::RADIUS`] e do [`super::PROFILE`] — a extensão e o perfil do twist
//! (doc 89, folha 04).

use super::*;

/// Uma fileira radial: pontos a 1, 2, 3 e 4 do pivô, ao longo de +X.
fn spokes() -> Vec<[f32; 2]> {
    (1..=4).map(|i| [i as f32, 0.0]).collect()
}

/// O ângulo (em graus) a que cada ponto de fato girou, medido da saída.
fn turned(out: &[[f32; 2]], base: &[[f32; 2]]) -> Vec<f32> {
    out.iter()
        .zip(base)
        .map(|(q, p)| {
            let (r0, r1) = ((p[0] * p[0] + p[1] * p[1]).sqrt(), 0.0f32.max(q[1]));
            // A fileira parte de +X, então o seno do ângulo é `y / r`.
            (r1 / r0).clamp(-1.0, 1.0).asin().to_degrees()
        })
        .collect()
}

/// **`radius = 0` É O ARO MEDIDO, E UM ARO AUTORADO MENOR SATURA O RESTO.**
///
/// ⚠️ As duas metades. A primeira: com `0` o ponto mais externo leva o ângulo inteiro (é o
/// `r_max` da redução). A segunda: com o aro em `2`, os pontos a `3` e `4` estão FORA dele e
/// levam o ângulo inteiro também — é o clamp, e é a resposta certa para *"o twist acaba aqui"*.
#[test]
fn zero_is_the_measured_rim_and_an_authored_rim_saturates_beyond_it() {
    let s = spokes();
    let auto = twist(&s, [0.0, 0.0], 60.0, 0.0, 0, &[], &[1.0; 4]);
    let a = turned(&auto, &s);
    assert!(
        (a[3] - 60.0).abs() < 0.2,
        "o aro leva o ângulo inteiro: {a:?}"
    );
    assert!((a[0] - 15.0).abs() < 0.3, "e o de dentro, um quarto: {a:?}");

    let tight = twist(&s, [0.0, 0.0], 60.0, 2.0, 0, &[], &[1.0; 4]);
    let t = turned(&tight, &s);
    assert!((t[1] - 60.0).abs() < 0.2, "em r = 2 o aro é ali: {t:?}");
    assert!(
        (t[2] - 60.0).abs() < 0.2 && (t[3] - 60.0).abs() < 0.2,
        "e para lá dele o twist satura em vez de crescer: {t:?}"
    );
}

/// **O PERFIL MOLDA O ÂNGULO, NÃO A POSIÇÃO** — e o aro continua a levar o ângulo inteiro.
///
/// ⚠️ É esta segunda metade que separa um perfil de uma máscara. Um `falloff` sobre a rotação
/// puxaria o ponto para a CORDA (raio `r·cos(θ/2)`) e o layout ENCOLHERIA; aqui o raio de cada
/// ponto é o mesmo nos quatro perfis, e só o quanto ele girou muda.
#[test]
fn the_profile_shapes_the_angle_and_never_the_radius() {
    let s = spokes();
    let radii = |out: &[[f32; 2]]| -> Vec<f32> {
        out.iter()
            .map(|q| (q[0] * q[0] + q[1] * q[1]).sqrt())
            .collect()
    };
    let base = radii(&twist(&s, [0.0, 0.0], 60.0, 0.0, 0, &[], &[1.0; 4]));
    let mut middles = Vec::new();
    for profile in 0..4 {
        let out = twist(&s, [0.0, 0.0], 60.0, 0.0, profile, &[], &[1.0; 4]);
        // ⚠️ **A barra é 0,3% do raio, e isso MEDE um facto da casa:** a senoide parabólica
        // do HR-5 não é norma-preservante (`c² + s² ≠ 1` ao bit), então uma rotação por um
        // ângulo diferente perturba o raio em ~0,1%. O que este gate afirma é que o perfil não
        // mexe no raio de PROPÓSITO — a deriva que sobra é a da trig, e é a mesma que o nó já
        // tinha antes desta wave.
        for (a, b) in radii(&out).iter().zip(&base) {
            assert!(
                (a - b).abs() < 3e-3 * b.max(1.0),
                "o perfil {profile} mexeu num RAIO: {a} contra {b}"
            );
        }
        let t = turned(&out, &s);
        assert!(
            (t[3] - 60.0).abs() < 0.2,
            "o aro leva sempre o inteiro: {t:?}"
        );
        // ⚠️ A amostra é o ponto a `r = 1` (`t = 0,25`), **não** o do meio: o `r_max` é `4`,
        // então o segundo ponto cai exactamente em `t = 0,5` — e o smoothstep e o smootherstep
        // FIXAM o meio. Medido: ali os perfis 0, 2 e 3 dão os mesmos 30° e só o Quad difere,
        // e o gate teria acusado o produto de um defeito que é da amostra.
        middles.push(t[0]);
    }
    // …e os quatro perfis dão quatro meios diferentes, senão o enum é decorativo.
    for (i, a) in middles.iter().enumerate() {
        for b in middles.iter().skip(i + 1) {
            assert!(
                (a - b).abs() > 1.0,
                "dois perfis com o mesmo meio: {middles:?}"
            );
        }
    }
}

/// **`Linear` É `t` E NADA MAIS** — a identidade da família, e é o que faz o default literal.
#[test]
fn linear_is_the_identity_of_the_curve_family() {
    for t in [0.0f32, 0.1, 0.25, 0.5, 0.75, 1.0] {
        assert_eq!(curve(0, t), t, "Linear tem de devolver o próprio t");
    }
    // ⚠️ E o controle é amostrado em `0,25`, **não** em `0,5`: o smoothstep e o smootherstep
    // fixam o meio (`0,5 → 0,5`), então um controle ali daria verde para a identidade em dois
    // dos três perfis. O ponto onde as quatro curvas de facto discordam é fora do centro.
    for k in 1..4 {
        assert_ne!(curve(k, 0.25), 0.25, "o perfil {k} tem de curvar");
    }
}

/// **OS DOIS KNOBS ESTÃO NO PAINEL, e o `0` do raio é alcançável.**
///
/// ⚠️ Um `min` acima de zero tornaria o modo automático — que é o DEFAULT — impossível de
/// repor pelo painel: o artista sairia dele e não voltaria.
#[test]
fn both_knobs_are_painted_and_auto_is_reachable() {
    let r = PARAM_HINTS
        .iter()
        .find(|h| h.param == RADIUS)
        .expect("o Radius tem de estar pintado");
    assert_eq!(r.min, 0.0, "o sentinela de «auto» tem de caber no curso");
    let p = PARAM_HINTS
        .iter()
        .find(|h| h.param == PROFILE)
        .expect("o Profile tem de estar pintado");
    match p.widget {
        ParamWidget::Enum { labels } => assert_eq!(labels.len(), 4, "a família das quatro"),
        _ => panic!("o Profile é um Enum"),
    }
    // E o device NÃO recua por nenhum dos dois — a redução não mudou de expressão.
    assert!(GPU_KERNEL.applicable.is_none(), "nada aqui recusa o device");
    for k in [RADIUS, PROFILE] {
        assert!(
            GPU_KERNEL.params.contains(&k),
            "o device tem de receber {k}"
        );
    }
}
