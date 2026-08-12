//! **A FORMA DO PESO NO BARRO** — irmão do `verb_tests.rs`, cortado por
//! ASSUNTO.
//!
//! Lá cada gate mede *o que o verbo faz* (o Fill sobe, o Pinch aperta); aqui,
//! **quanto peso um dab deposita e ONDE** — os knobs que reescrevem o perfil
//! antes de qualquer verbo o ler: hoje o `hardness`, e o front-face contínuo
//! (E12) quando ele chegar.
//!
//! ⚠️ **O que estes gates têm em comum é o ORÁCULO:** eles medem o
//! deslocamento na MALHA, não a função. Os irmãos de `brush_tests.rs`
//! interrogam a porta direto e por isso **passariam com a chamada removida do
//! laço** — uma função certa que ninguém chama é o modo de falha que esta casa
//! varre a cada wave, e é para ele que este arquivo existe.
//!
//! Este módulo é FILHO de `tests`, então `use super::*` alcança as fixtures
//! compartilhadas.

use super::*;

/// **A DUREZA CHEGA AO DAB — o platô aparece no BARRO, não só na fórmula.**
///
/// ⚠️ **Este gate existe porque os três de `brush_tests.rs` passariam com a
/// chamada REMOVIDA do laço**: eles interrogam a [`Brush::shaped_distance`]
/// direto, e uma função certa que ninguém chama é exatamente o modo de falha que
/// esta casa varre a cada wave. Aqui o oráculo é o deslocamento medido na malha.
///
/// A curva é a `Linear` de propósito: o deslocamento é `1 − t`, então *"o
/// interior virou platô"* tem resposta binária. Com a `Constant` a dureza seria
/// invisível (tudo já é 1), e com uma curva macia ela seria uma questão de grau.
#[test]
fn the_hardness_flattens_the_dab_into_a_plateau_on_the_clay() {
    let c = [0.0, 0.0, 1.0];
    // O deslocamento por vértice, indexado pela distância normalizada ao centro.
    let profile = |hardness: f32| {
        let mut mesh = sphere();
        let base = snapshot(&mesh);
        let b = Brush {
            verb: Verb::Draw,
            radius: 0.5,
            strength: 1.0,
            falloff: Falloff::Linear,
            hardness,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        s.dab(&mut mesh, &b, &dab_at(c, b.radius), Symmetry::default());
        let mut out: Vec<(f32, f32)> = Vec::new();
        for (p, q) in base.iter().zip(mesh.positions()) {
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            let r = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
            let t = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt() / b.radius;
            if t < 1.0 {
                out.push((t, len));
            }
        }
        out
    };
    // O maior deslocamento (o miolo) e o deslocamento a `t ≈ 0,25`.
    let at = |rows: &[(f32, f32)], want_t: f32| {
        rows.iter()
            .min_by(|a, b| (a.0 - want_t).abs().total_cmp(&(b.0 - want_t).abs()))
            .map(|r| (r.0, r.1))
            .expect("a pegada não pode ser vazia")
    };

    let soft = profile(0.0);
    let hard = profile(0.5);
    assert!(soft.len() > 20, "fixture magra ({})", soft.len());
    let peak = soft.iter().map(|r| r.1).fold(0.0f32, f32::max);
    let (t_soft, d_soft) = at(&soft, 0.25);
    let (t_hard, d_hard) = at(&hard, 0.25);
    assert!((t_soft - t_hard).abs() < 1e-6, "o mesmo vértice nos dois");
    // ⚠️ **Sem dureza, `t = 0,25` já perdeu um quarto do peso** — é a `Linear`.
    assert!(
        d_soft < peak * 0.85,
        "sem dureza o dab devia decair já em t={t_soft} ({d_soft} contra {peak})"
    );
    // **Com dureza `0,5`, tudo abaixo de `t = 0,5` está no platô**: o
    // deslocamento a `0,25` é o MESMO do miolo.
    let peak_hard = hard.iter().map(|r| r.1).fold(0.0f32, f32::max);
    assert!(
        (d_hard - peak_hard).abs() < peak_hard * 0.01,
        "com dureza 0,5 o t={t_hard} tinha de estar no platô ({d_hard} contra \
         o pico {peak_hard})"
    );
}
