//! **O ALPHA E O FILTRO** — a seção Alpha vale para o gesto de malha inteira.
//!
//! Report do Enio (2026-08-19): *"filter com alpha ligado desconsidera alpha"*.
//!
//! ⚠️ **A lei desta wave numa linha: o alpha é MAIS UM PESO POR-VÉRTICE,
//! exactamente como a máscara.** O motor já multiplicava o fator do filtro pelo
//! `free_weight` da máscara; o alpha entra no mesmo produto, pela mesma porta
//! ([`crate::Brush::alpha_weight`]), avaliado na **pose CONGELADA**. Não há
//! semântica nova a inventar: [`crate::Alpha`] é um CAMPO avaliado num ponto —
//! ele nunca precisou de um dab.
//!
//! ⚠️ **O ORÁCULO NÃO É A FÓRMULA, é a LINEARIDADE.** Perguntar ao motor se ele
//! multiplicou pelo que eu mandei multiplicar seria o espelho que este repo já
//! pagou várias vezes. O que estes gates medem é uma propriedade das LEIS: todas
//! as oito do laço genérico são **lineares no fator** (`Inflate` anda
//! `nrm·f`, `Smooth` anda `(avg−base)·f`, `Scale` anda `base·f`, …), então
//! ligar o alpha tem de escalar o deslocamento de cada vértice **pelo peso do
//! alpha naquele ponto** — uma corrida com alpha contra uma corrida sem, sem
//! nenhuma constante escrita aqui.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --lib stroke::tests::filter_alpha
//! ```

use super::*;

use crate::{Alpha, FilterKind};

use super::stroke_filter::{deltas, norm};

/// Os dois alphas que este arquivo exercita, e eles não são intercambiáveis:
///
/// - o [`Alpha::Noise`] é **isotrópico** — lido em espaço de OBJETO, sem passar
///   pelo frame. É o caso que prova que o peso chega ao fator.
/// - o [`Alpha::Strata`] é **direcional** — ele PASSA pelo
///   [`crate::AlphaFrame`]. É o caso que prova que o filtro passa o frame, e não
///   um `AlphaFrame` improvisado: um consumidor que construísse o seu próprio
///   frame daria outro campo, e só o direcional o denuncia.
const ALPHAS: [Alpha; 2] = [Alpha::Noise, Alpha::Strata];

/// Um pincel de filtro com o alpha pedido armado.
///
/// ⚠️ **A escala é pequena de propósito:** ela é o tamanho de uma feature em
/// unidades de OBJETO, e a fixture é uma esfera de raio 1 — uma feature do
/// tamanho do modelo daria um campo quase constante, e um campo constante não
/// discrimina *"o alpha chegou"* de *"o alpha chegou errado"*.
fn alpha_brush(verb: Verb, alpha: &Alpha) -> Brush {
    Brush {
        alpha: Some(alpha.clone()),
        alpha_scale: 0.25,
        ..super::stroke_filter::filter_brush(verb)
    }
}

/// O peso do alpha em cada vértice da pose CONGELADA, na ordem dos vértices.
fn weights(brush: &Brush, base: &[[f32; 3]]) -> Vec<f32> {
    let frame = brush.alpha_frame();
    base.iter()
        .map(|p| brush.alpha_weight(*p, &frame))
        .collect()
}

/// ⭐ **O ALPHA ESCALA O DESLOCAMENTO DE CADA VÉRTICE, LEI A LEI.**
///
/// Duas corridas idênticas — uma com o alpha armado, outra sem — sobre a mesma
/// pose congelada. Como toda lei do laço genérico é linear no fator, o
/// quociente entre as duas TEM de ser o peso do alpha naquele ponto.
///
/// ⚠️ **Antes desta wave o quociente era `1` em toda parte**: o `filter`
/// recebia o `brush` e nunca lhe perguntava pelo alpha, então a seção inteira
/// do painel não tinha efeito nenhum sobre o gesto de malha inteira.
///
/// ⚠️ **O `Sharpen` está FORA deste laço e tem gate próprio** — ele encadeia
/// sub-passos a partir da pose VIVA, então a composição dele com um peso não é
/// um produto: é um produto por sub-passo. Metê-lo aqui seria afirmar
/// linearidade sobre a única lei que não a tem.
#[test]
fn the_alpha_scales_every_law_of_the_generic_loop() {
    for alpha in &ALPHAS {
        for kind in FilterKind::ALL {
            if kind == FilterKind::Sharpen {
                continue;
            }
            let brush = alpha_brush(Verb::Smooth, alpha);
            let plain = super::stroke_filter::filter_brush(Verb::Smooth);

            // Sem alpha — a régua.
            let mut m0 = mesh_for(Verb::Smooth);
            let base = snapshot(&m0);
            let mut s0 = SculptStroke::default();
            s0.filter_begin(&m0);
            s0.filter(&mut m0, &plain, kind, 0.4);
            let d0 = deltas(&base, &m0);

            // Com alpha — a mesma pose, o mesmo arrasto.
            let mut m1 = mesh_for(Verb::Smooth);
            let mut s1 = SculptStroke::default();
            s1.filter_begin(&m1);
            s1.filter(&mut m1, &brush, kind, 0.4);
            let d1 = deltas(&base, &m1);

            let w = weights(&brush, &base);

            // A premissa: a régua move alguma coisa, e o campo do alpha VARIA
            // sobre esta malha. Sem as duas, o resto não afirma nada.
            let biggest = d0.iter().copied().map(norm).fold(0.0f32, f32::max);
            assert!(
                biggest > 1e-4,
                "{kind:?}/{alpha:?}: a regua sem alpha nao moveu a malha ({biggest:e})"
            );
            let (wlo, whi) = w
                .iter()
                .fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(*x), b.max(*x)));
            assert!(
                whi - wlo > 0.2,
                "{alpha:?}: o campo do alpha e' quase constante sobre esta malha \
                 ({wlo:.3}..{whi:.3}) -- a fixture nao contem o fenomeno"
            );

            for (i, (a, b)) in d0.iter().zip(&d1).enumerate() {
                let want = [a[0] * w[i], a[1] * w[i], a[2] * w[i]];
                let err = norm([b[0] - want[0], b[1] - want[1], b[2] - want[2]]);
                assert!(
                    err <= 1e-5 * biggest.max(1.0),
                    "{kind:?}/{alpha:?}: o vertice {i} andou {:?}, e o alpha ali vale {:.4} \
                     sobre um passo de {:?} -- esperado {want:?} (erro {err:e})",
                    b,
                    w[i],
                    a
                );
            }
        }
    }
}

/// **SEM ALPHA ARMADO, o filtro é BYTE-IDÊNTICO.**
///
/// ⚠️ **É a metade que impede a cura de custar o mundo pré-alpha.** A porta
/// devolve `1.0` EXATO quando não há alpha, e `x * 1.0 == x` ao bit no IEEE-754
/// — então o produto novo no fator não pode mover um único bit de quem não
/// armou nada. O gate compara a corrida contra a **posição de cada vértice**, e
/// não contra uma tolerância: um `1.0` que virasse `0.999999` passaria num
/// epsilon e falha aqui.
#[test]
fn an_unarmed_alpha_moves_no_bit_of_the_filter() {
    for kind in FilterKind::ALL {
        let plain = super::stroke_filter::filter_brush(Verb::Smooth);

        let mut a = mesh_for(Verb::Smooth);
        let mut sa = SculptStroke::default();
        sa.filter_begin(&a);
        sa.filter(&mut a, &plain, kind, 0.4);

        // A MESMA corrida, com o campo do alpha explicitamente vazio: é o
        // caminho que o produto percorre quando ninguém armou nada.
        let mut b = mesh_for(Verb::Smooth);
        let empty = Brush {
            alpha: None,
            ..super::stroke_filter::filter_brush(Verb::Smooth)
        };
        let mut sb = SculptStroke::default();
        sb.filter_begin(&b);
        sb.filter(&mut b, &empty, kind, 0.4);

        assert_eq!(
            a.positions(),
            b.positions(),
            "{kind:?}: o caminho sem alpha deixou de ser byte-identico"
        );
    }
}

/// **O SHARPEN TAMBÉM HONRA O ALPHA** — e ele é o que escapa de toda lista.
///
/// ⚠️ **Ele bifurca do laço genérico** (é o único filtro com pré-passe), então
/// um consumidor novo escrito só no laço deixaria **oito leis honrando o alpha
/// e uma ignorando-o** — a forma de defeito que este repo já nomeou: *um `match`
/// exaustivo não guarda a lista que um laço itera*.
///
/// ⚠️ **O oráculo aqui NÃO é a linearidade** (ele encadeia sub-passos a partir
/// da pose viva, e o peso entra por sub-passo). É a MONOTONIA: onde o alpha é
/// quase zero o vértice quase não anda, e onde ele é quase um o vértice anda
/// como andaria sem alpha nenhum.
#[test]
fn the_sharpen_honours_the_alpha_too() {
    let brush = alpha_brush(Verb::Smooth, &Alpha::Noise);
    let plain = super::stroke_filter::filter_brush(Verb::Smooth);

    let mut m0 = mesh_for(Verb::Smooth);
    let base = snapshot(&m0);
    let mut s0 = SculptStroke::default();
    s0.filter_begin(&m0);
    s0.filter(&mut m0, &plain, FilterKind::Sharpen, 2.0);
    let d0 = deltas(&base, &m0);

    let mut m1 = mesh_for(Verb::Smooth);
    let mut s1 = SculptStroke::default();
    s1.filter_begin(&m1);
    s1.filter(&mut m1, &brush, FilterKind::Sharpen, 2.0);
    let d1 = deltas(&base, &m1);

    let w = weights(&brush, &base);

    // Os dois extremos do campo, escolhidos entre os vértices que a régua de
    // facto MOVEU -- perguntar a um vértice parado o que o alpha lhe fez seria
    // medir vácuo.
    let big = d0.iter().copied().map(norm).fold(0.0f32, f32::max);
    assert!(big > 1e-4, "a regua do Sharpen nao moveu a malha");
    let live: Vec<usize> = (0..d0.len()).filter(|&i| norm(d0[i]) > 0.2 * big).collect();
    assert!(
        live.len() > 8,
        "poucos vertices moveram o bastante para medir ({})",
        live.len()
    );

    let lo = *live
        .iter()
        .min_by(|a, b| w[**a].total_cmp(&w[**b]))
        .expect("ha' vertices vivos");
    let hi = *live
        .iter()
        .max_by(|a, b| w[**a].total_cmp(&w[**b]))
        .expect("ha' vertices vivos");
    assert!(
        w[hi] - w[lo] > 0.2,
        "os dois extremos do alpha estao perto demais ({:.3} vs {:.3})",
        w[lo],
        w[hi]
    );

    let (r_lo, r_hi) = (norm(d1[lo]) / norm(d0[lo]), norm(d1[hi]) / norm(d0[hi]));
    assert!(
        r_hi > r_lo,
        "o Sharpen ignorou o alpha: onde ele vale {:.3} o vertice andou {r_lo:.3} do normal, e \
         onde vale {:.3} andou {r_hi:.3} -- a razao tinha de SUBIR com o peso",
        w[lo],
        w[hi]
    );
    assert!(
        r_lo < 0.9,
        "onde o alpha vale {:.3} o vertice andou {r_lo:.3} do que andaria sem alpha nenhum: o \
         peso nao esta' a frear",
        w[lo]
    );
}
