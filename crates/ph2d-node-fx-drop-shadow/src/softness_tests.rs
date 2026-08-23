//! Os gates do `softness` no nó inteiro — o que o disco faz ao STREAM (doc 89,
//! folha 11). A lei do disco em si está em [`super::soft`].

use super::*;

const BLACK_35: [f32; 4] = [0.0, 0.0, 0.0, 0.35];

fn row(n: usize) -> Stream {
    #[expect(clippy::cast_precision_loss, reason = "uma fixture pequena")]
    let p: Vec<[f32; 2]> = (0..n).map(|i| [i as f32, 0.0]).collect();
    Stream::new(n)
        .with("P", Column::Vec2(p))
        .with("size", Column::Vec2(vec![[0.5, 0.5]; n]))
}

fn ps(s: &Stream) -> Vec<[f32; 2]> {
    match s.get("P") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => panic!("P"),
    }
}

fn alphas(s: &Stream) -> Vec<f32> {
    match s.get("tint") {
        Some(Column::Vec4(v)) => v.iter().map(|t| t[3]).collect(),
        _ => panic!("tint"),
    }
}

/// **`softness = 0` É A SOMBRA DURA, AO BIT** — e o gate compara com o próprio nó.
///
/// ⚠️ Um `softness` **não-finito** conta como desligado pela mesma porta: um
/// documento carregado ou uma edição por MCP não pode transformar o único fantasma
/// numa nuvem de `NaN`.
#[test]
fn a_zero_or_junk_softness_is_the_hard_shadow_bit_for_bit() {
    let src = row(3);
    let hard = sink_blend(&src, 315.0, 0.2, BLACK_35, 0.0);
    assert_eq!(hard.count(), 3 * COPIES, "um fantasma por elemento");
    for junk in [f32::NAN, f32::INFINITY, -1.0, -0.0] {
        let out = sink_blend(&src, 315.0, 0.2, BLACK_35, junk);
        assert_eq!(ps(&out), ps(&hard), "softness {junk} tinha de estar off");
        assert_eq!(alphas(&out), alphas(&hard));
    }
}

/// **LIGADO, CADA ELEMENTO VIRA `TAPS + 1` LINHAS** — e os elementos continuam por
/// último, que é o que os mantém à frente das próprias sombras.
#[test]
fn softness_emits_one_row_per_tap_plus_the_element() {
    let out = sink_blend(&row(3), 315.0, 0.2, BLACK_35, 0.3);
    assert_eq!(out.count(), 3 * (soft::TAPS + 1));
    let a = alphas(&out);
    // As últimas `n` linhas são os elementos verbatim (a fixture não tem `tint`,
    // então eles saem no branco opaco que é a identidade).
    assert!(
        a[3 * soft::TAPS..].iter().all(|v| (*v - 1.0).abs() < 1e-6),
        "os elementos têm de vir por último e intactos: {:?}",
        &a[3 * soft::TAPS..]
    );
    // E cada tap leva o alfa reduzido — a UNIÃO é que vale o de sempre.
    assert!(
        a[0] < 0.35 * 0.5,
        "um tap sozinho tem de ser bem mais fraco que a sombra dura: {}",
        a[0]
    );
}

/// **O TETO CONTA AS LINHAS QUE ESTE COOK EMITE** — e por isso a maciez o move.
///
/// ⚠️ Sem isto o portão continuaria a multiplicar por `2` e uma sombra macia sobre
/// um layout grande emitiria **oito vezes** o que o teto autoriza. O gate mede os
/// dois lados da fronteira, porque um teto que só é testado por dentro não é um teto.
#[test]
fn the_ceiling_counts_the_rows_the_soft_shadow_actually_emits() {
    let per = soft::TAPS + 1;
    let just_under = MAX_INSTANCES / per;
    let out = sink_blend(&row(just_under), 315.0, 0.2, BLACK_35, 0.3);
    assert_eq!(
        out.count(),
        just_under * per,
        "abaixo do teto, ele trabalha"
    );

    let over = just_under + 1;
    let off = sink_blend(&row(over), 315.0, 0.2, BLACK_35, 0.3);
    assert_eq!(
        off.count(),
        over,
        "acima, o FX desliga-se e devolve a entrada"
    );
    // E o MESMO layout com a sombra DURA continua a passar — a maciez é que custa.
    let hard = sink_blend(&row(over), 315.0, 0.2, BLACK_35, 0.0);
    assert_eq!(hard.count(), over * COPIES);
}

/// **O DISCO É CENTRADO NO OFFSET, e não na origem** — a maciez não move a sombra.
///
/// ⚠️ É o defeito que um disco somado ao `P` em vez de ao `offset` produziria: a
/// sombra saltaria de sítio ao ligar o knob, e a `distance` deixaria de mandar.
#[test]
fn the_disc_is_centred_on_the_offset_so_the_shadow_does_not_jump() {
    let src = row(1);
    let hard = ps(&sink_blend(&src, 0.0, 1.0, BLACK_35, 0.0))[0];
    let soft_rows = ps(&sink_blend(&src, 0.0, 1.0, BLACK_35, 0.4));
    let taps = &soft_rows[..soft::TAPS];
    #[expect(clippy::cast_precision_loss, reason = "TAPS = 16")]
    let n = soft::TAPS as f32;
    let c = taps
        .iter()
        .fold([0.0f32, 0.0], |a, q| [a[0] + q[0], a[1] + q[1]]);
    let (cx, cy) = (c[0] / n, c[1] / n);
    assert!(
        (cx - hard[0]).abs() < 0.07 && (cy - hard[1]).abs() < 0.07,
        "o centro dos taps ({cx:.3}, {cy:.3}) tem de ficar no offset {hard:?}"
    );
}

/// **O KNOB ESTÁ PINTADO, com unidade e com teto de digitação.**
#[test]
fn the_knob_is_painted_with_its_unit_and_ceiling() {
    let h = PARAM_HINTS
        .iter()
        .find(|h| h.param == "softness")
        .expect("o Softness tem de estar pintado");
    assert_eq!(h.min, 0.0, "o neutro tem de caber no curso");
    assert!(
        PARAM_UNITS
            .iter()
            .any(|u| u.param == "softness" && u.unit == ParamUnit::Length),
        "um raio de mundo é um comprimento"
    );
    assert!(
        PARAM_HARD_MAX.iter().any(|m| m.param == "softness"),
        "o teto da aproximação é nomeado"
    );
}
