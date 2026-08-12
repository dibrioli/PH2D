//! **A LEI DE KERNEL que um MODO escolhe** — os gates que perguntam *qual
//! referência este verbo está seguindo*, e não *o que este verbo faz*.
//!
//! Irmão do `verb_tests.rs`, cortado por ASSUNTO: lá cada gate mede o verbo
//! (*o Fill sobe, o Scrape desce, o Pinch aperta*); aqui cada gate mede a
//! **escolha** — a [`crate::KernelLaw`] que o [`crate::RefMode`] manda, com o
//! outro modo servindo de CONTROLE. Sem o controle, os dois lados poderiam ser
//! o mesmo e o chip não escolheria nada.
//!
//! ⚠️ **Eles são o que torna o chip `S` uma afirmação em vez de um rótulo.**
//! Antes desta wave o app dizia s-mode e rodava, em três verbos, uma lei que
//! nenhuma das duas referências tem — a mesma doença que a W0 curou nos
//! DEFAULTS, aqui na LEI. Os números vivem no atlas
//! (`tests/measure_reference_divergence.rs`): `1,717e-3` no Flatten e
//! `5,776e-4` no Pinch, contra um piso de `5,96e-8`.
//!
//! Este módulo é FILHO de `tests`, então `use super::*` alcança as fixtures
//! compartilhadas — duplicá-las seria como os dois arquivos passam a medir
//! malhas diferentes sem ninguém notar.

use super::*;

/// **EM `S`, O FLATTEN É O SCRAPE — ao vértice.**
///
/// Não é uma aproximação nem um piso escolhido: o `Flatten.js:11` nasce com
/// `_negative = true`, o `:57` faz `comp = -1` e o `:64` faz
/// `if (distToPlane * comp > 0.0) continue` — ou seja, o *Flatten* da
/// referência **é** o nosso `Scrape`, e o outro lado dela é o nosso `Fill`.
/// Quem quiser os dois lados escolhe o modo `B`, que é a leitura do `plane.cc`.
///
/// ⚠️ **Este gate é o que torna o chip `S` uma afirmação em vez de um rótulo.**
/// Antes desta wave o app dizia s-mode e rodava um Flatten que nenhuma das duas
/// referências tem — a mesma doença que a W0 curou nos DEFAULTS, aqui na LEI.
#[test]
fn in_s_mode_the_flatten_is_the_scrape_vertex_by_vertex() {
    let mut bumpy = sphere();
    let poke = Brush {
        verb: Verb::Draw,
        radius: 0.15,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&bumpy);
    s.dab(
        &mut bumpy,
        &poke,
        &dab_at([0.0, 0.0, 1.0], poke.radius),
        Symmetry::default(),
    );

    let c = [0.1, 0.0, 1.0];
    let run = |verb: Verb, mode: crate::RefMode| {
        let mut mesh = bumpy.clone();
        let b = Brush {
            verb,
            radius: 0.5,
            strength: 1.0,
            falloff: Falloff::Constant,
            mode,
            ..Brush::default()
        };
        let mut st = SculptStroke::default();
        st.begin(&mesh);
        st.dab(&mut mesh, &b, &dab_at(c, b.radius), Symmetry::default());
        snapshot(&mesh)
    };
    let flat_s = run(Verb::Flatten, crate::RefMode::S);
    let scrape = run(Verb::Scrape, crate::RefMode::S);
    let flat_b = run(Verb::Flatten, crate::RefMode::B);

    let gap = |a: &[[f32; 3]], b: &[[f32; 3]]| {
        a.iter()
            .zip(b)
            .map(|(p, q)| (0..3).map(|k| (p[k] - q[k]).abs()).fold(0.0f32, f32::max))
            .fold(0.0f32, f32::max)
    };
    assert_eq!(
        gap(&flat_s, &scrape),
        0.0,
        "em `S` o Flatten tem de ser o Scrape AO VÉRTICE"
    );
    // ⚠️ **O CONTROLE, e sem ele o gate passaria com os dois modos em `S`:** em
    // `B` o mesmo dab tem de divergir do Scrape, senão o chip não escolhe nada.
    let b_gap = gap(&flat_b, &scrape);
    assert!(
        b_gap > 1e-4,
        "o modo `B` tem de morder o outro lado também ({b_gap})"
    );
}

/// **EM `S`, O PINCH PUXA EM 3D — e o deslocamento aponta para o CENTRO.**
///
/// O `Pinch.js:52-58` soma `(centro − v) · f · deform` cru, sem projetar. O
/// oráculo aqui não é *"tem componente normal"* (isso é fraco: qualquer erro de
/// sinal também teria) — é **a DIREÇÃO**: cada vértice deslocado tem de andar ao
/// longo da reta que o liga ao centro do dab, ao cosseno.
///
/// ⚠️ **E o gate mede o par, não o absoluto:** o mesmo dab em `B` tem de ficar
/// perpendicular à normal. Sem essa metade ele passaria com os dois modos em
/// `S`, e o chip não escolheria nada.
#[test]
fn in_s_mode_the_pinch_pulls_straight_at_the_centre() {
    let c = [0.0, 0.0, 1.0];
    let run = |mode: crate::RefMode| {
        let mut mesh = sphere();
        let base = snapshot(&mesh);
        let b = Brush {
            verb: Verb::Pinch,
            radius: 0.5,
            strength: 1.0,
            mode,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        s.dab(&mut mesh, &b, &dab_at(c, b.radius), Symmetry::default());
        // O pior cosseno contra a direção `v -> centro`, e o pior |normal|/|d|.
        let (mut worst_cos, mut worst_normal, mut moved) = (1.0f32, 0.0f32, 0usize);
        for (p, q) in base.iter().zip(mesh.positions()) {
            let d = [q[0] - p[0], q[1] - p[1], q[2] - p[2]];
            let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            if len < 1e-5 {
                continue;
            }
            moved += 1;
            let to_c = [c[0] - p[0], c[1] - p[1], c[2] - p[2]];
            let cl = (to_c[0] * to_c[0] + to_c[1] * to_c[1] + to_c[2] * to_c[2]).sqrt();
            let cos = (d[0] * to_c[0] + d[1] * to_c[1] + d[2] * to_c[2]) / (len * cl);
            worst_cos = worst_cos.min(cos);
            // A normal da área no polo é ~+Z.
            worst_normal = worst_normal.max(d[2].abs() / len);
        }
        (worst_cos, worst_normal, moved)
    };

    let (cos_s, normal_s, moved) = run(crate::RefMode::S);
    assert!(
        cos_s > 0.9999,
        "em `S` todo vértice anda RETO para o centro (pior cosseno {cos_s})"
    );
    // O CONTROLE: em `B` a projeção tira a componente normal, e é isso que faz
    // o mesmo dab andar para OUTRO lugar.
    let (cos_b, normal_b, moved_b) = run(crate::RefMode::B);
    // ⚠️ **O anti-vácuo é ESTRUTURAL, e não um piso que eu escolhesse:** a
    // pegada não depende do modo (ela é a esfera do dab), então os dois têm de
    // mover o MESMO conjunto — e um conjunto vazio faria os `worst_*` acima
    // devolverem os valores de identidade e o gate passar sem medir nada.
    assert_eq!(moved, moved_b, "a pegada não pode depender do modo");
    assert!(moved > 0, "a fixture não contém o fenômeno");
    assert!(
        normal_s > normal_b * 10.0,
        "o puxão cru tem de carregar a normal que a projeção corta          (`S` {normal_s} contra `B` {normal_b})"
    );
    assert!(
        cos_b < 0.9999,
        "em `B` o puxão NÃO é reto para o centro (cosseno {cos_b})"
    );
}
