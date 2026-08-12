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

/// **O FRONT-FACE CONTÍNUO — em `B` um vértice de PERFIL pesa zero, e a
/// transição não tem degrau.**
///
/// ⚠️ **A fixture TEM de atravessar o terminador**, senão o gate mede o vácuo:
/// no miolo de um dab pequeno todo vértice olha quase direto para o olho, e
/// `max(n · olho, 0)` vale ~1 em toda parte — a lei ficaria indistinguível de
/// não existir. O raio é grande de propósito.
///
/// ⚠️ E o falloff é `Constant` **pelo mesmo motivo do gate da dureza**: com uma
/// curva macia o deslocamento cairia com a distância de qualquer forma, e o
/// número não separaria *a curva* de *a orientação*. Com a `Constant`, toda
/// variação que sobrar é do front-face.
#[test]
fn in_b_mode_a_grazing_vertex_weighs_nothing_and_the_ramp_has_no_step() {
    // O dab olha para o centro da esfera a partir do +Z.
    //
    // ⚠️ **O raio é 1,2 e ele foi MEDIDO, não escolhido bonito:** a pegada é uma
    // consulta por esfera, então uma corda `r` numa esfera unitária varre um
    // ângulo central de `2·asin(r/2)` — com `0,8` o pior vértice ainda olha a
    // 47° (cosseno `0,683`) e a lei do front-face mal se distingue de não
    // existir. Com `1,2` são 74°, cosseno `0,28`: a pegada de fato atravessa o
    // terminador, que é a premissa deste gate.
    let c = [0.0, 0.0, 1.0];
    let radius = 1.2;
    // (cosseno com a direção do olho, deslocamento) por vértice da pegada.
    let profile = |mode: crate::RefMode| {
        let mut mesh = sphere();
        let base = snapshot(&mesh);
        let normals: Vec<[f32; 3]> = mesh.normals().to_vec();
        let d = dab_at(c, radius);
        let b = Brush {
            verb: Verb::Draw,
            radius,
            strength: 1.0,
            falloff: Falloff::Constant,
            mode,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        s.dab(&mut mesh, &b, &d, Symmetry::default());
        let mut rows: Vec<(f32, f32)> = Vec::new();
        for (v, (p, q)) in base.iter().zip(mesh.positions()).enumerate() {
            let delta = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            let len = (delta[0] * delta[0] + delta[1] * delta[1] + delta[2] * delta[2]).sqrt();
            let r = [p[0] - c[0], p[1] - c[1], p[2] - c[2]];
            if (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt() >= radius {
                continue;
            }
            let n = normals[v];
            // ⚠️ O `eye` aponta do OLHO para a superfície: um vértice de frente
            // tem produto NEGATIVO, e é por isso que a régua leva o sinal.
            let facing = -(n[0] * d.eye[0] + n[1] * d.eye[1] + n[2] * d.eye[2]);
            rows.push((facing, len));
        }
        rows.sort_by(|a, b| a.0.total_cmp(&b.0));
        rows
    };

    let s_rows = profile(crate::RefMode::S);
    let b_rows = profile(crate::RefMode::B);
    // A fixture contém o fenômeno? Ela tem de ir do perfil (facing ~ 0) à
    // frente (facing ~ 1) — senão as duas leis dão o mesmo número.
    let (min_face, max_face) = (s_rows[0].0, s_rows[s_rows.len() - 1].0);
    assert!(
        min_face < 0.35 && max_face > 0.95,
        "a pegada não atravessa o terminador ({min_face} .. {max_face})"
    );

    // **Em `S` a `Constant` é constante**: todo vértice anda o mesmo.
    let s_min = s_rows.iter().map(|r| r.1).fold(f32::MAX, f32::min);
    let s_max = s_rows.iter().map(|r| r.1).fold(0.0f32, f32::max);
    assert!(
        (s_max - s_min) < s_max * 0.01,
        "em `S` o dab não pesa por orientação ({s_min} .. {s_max})"
    );

    // **Em `B` o deslocamento SEGUE o cosseno** — e o oráculo é a razão, não um
    // piso escolhido: `deslocamento / facing` tem de ser o MESMO em toda a
    // pegada, que é a lei `factors *= max(dot, 0)` escrita como propriedade.
    let ratios: Vec<f32> = b_rows
        .iter()
        .filter(|r| r.0 > 0.05)
        .map(|r| r.1 / r.0)
        .collect();
    let (r_min, r_max) = (
        ratios.iter().copied().fold(f32::MAX, f32::min),
        ratios.iter().copied().fold(0.0f32, f32::max),
    );
    assert!(
        (r_max - r_min) < r_max * 0.02,
        "em `B` o deslocamento tinha de ser proporcional ao cosseno ({r_min} .. {r_max})"
    );
    // E o de PERFIL não anda: é a metade que dá nome à lei.
    let grazing = b_rows[0];
    assert!(
        grazing.1 < s_max * 0.35,
        "o vértice de perfil (cos {}) andou {} contra os {s_max} de `S`",
        grazing.0,
        grazing.1
    );
}
