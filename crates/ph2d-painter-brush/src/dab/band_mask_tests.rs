//! **A máscara do cap atravessa a divisão por linhas sem mudar um byte.**
//!
//! Irmão do [`super::tests`], e o corte é de responsabilidade: lá se prova *o que um dab PINTA*
//! (cobertura, falloff, strength, alpha-lock, ramp); aqui *que dividir as linhas não muda o que ele
//! pinta* quando o cap de Accumulate viaja junto.
//!
//! # O que mudou, e por quê
//!
//! Até 2026-08-04 o caminho do cap rodava numa **banda só**, com a razão escrita no `dab.rs`: *"lê+
//! escreve a máscara por-traço compartilhada … dabs pequenos e macios de qualquer jeito, onde o cap é
//! observável"*. ⚠️ **A premissa era verdadeira e deixou de ser** — o cap disparava só em
//! `strength < 1`, e o **AA do filme** do impasto passou a ligá-lo para TODO pincel de impasto,
//! inclusive os maiores do app, que são exatamente os que cruzam o piso de `PARALLEL_MIN_AREA`.
//! Medido pela porta do artista: com o dab **abaixo** do piso o cap custa `0,99×`; **acima**, `4,15×`
//! — o vão inteiro era o paralelismo perdido, e **zero** a aritmética do cap.
//!
//! # O oráculo é a rota que SHIPAVA
//!
//! [`stamp_dab_textured_masked_with`] com `min_area = usize::MAX` roda a banda única de antes, chamando
//! o MESMO `stamp_band` com os mesmos argumentos — então o gate compara o produto contra o produto, e
//! não contra uma segunda implementação escrita para o teste.
//!
//! ⚠️ **A fixture tem de conter o fenômeno em DUAS frentes:** o dab precisa **cruzar o piso** (senão as
//! duas rotas são literalmente o mesmo código e o verde é vácuo — asserido, não presumido) e a máscara
//! precisa chegar **já escrita** ao dab medido, porque a razão de o cap existir é LER a cobertura
//! anterior; sobre uma máscara zerada um erro de fatia pode passar despercebido.

use super::*;
use crate::blend::BrushBlend;
use crate::falloff::Falloff;

fn solid(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
    rgba.iter()
        .copied()
        .cycle()
        .take((width * height * 4) as usize)
        .collect()
}

/// Um pincel cujo cap é **observável**: `strength < 1` com disco duro, então o teto por texel é
/// exatamente `strength` e um segundo dab tem o que ler.
fn capped_brush(radius: f32) -> BrushSpec {
    BrushSpec {
        radius_px: radius,
        color: [0.0, 0.0, 0.0],
        blend: BrushBlend::Mix,
        falloff: Falloff::Smooth,
        hardness: 0.5,
        strength: 0.5,
        ..Default::default()
    }
}

/// `(canvas, mask)` depois de dois dabs sobrepostos, com o piso dado.
fn two_dabs(w: u32, h: u32, radius: f32, min_area: usize) -> (Vec<u8>, Vec<u8>) {
    let spec = capped_brush(radius);
    let mut buf = solid(w, h, [255, 255, 255, 255]);
    let mut mask = vec![0u8; (w * h) as usize];
    for center in [[290.0, 300.0], [310.0, 300.0]] {
        let _ = stamp_dab_textured_masked_with(
            &mut buf,
            w,
            h,
            center,
            &spec,
            1.0,
            false,
            Some(&mut mask),
            [1.0, 0.0],
            min_area,
        );
    }
    (buf, mask)
}

#[test]
fn the_capped_dab_is_byte_identical_whether_its_rows_are_split_or_not() {
    const W: u32 = 600;
    const H: u32 = 600;
    const R: f32 = 250.0;

    // A premissa, ASSERIDA: sem cruzar o piso as duas chamadas são o mesmo código e o gate seria vácuo.
    let bbox = (R * 2.0) as usize * (R * 2.0) as usize;
    assert!(
        bbox >= PARALLEL_MIN_AREA,
        "a fixture TEM de cruzar o piso ({bbox} < {PARALLEL_MIN_AREA}), senão as duas rotas são a mesma"
    );

    let (par_buf, par_mask) = two_dabs(W, H, R, PARALLEL_MIN_AREA);
    let (ser_buf, ser_mask) = two_dabs(W, H, R, usize::MAX);

    // E que o cap de fato ESCREVEU — uma máscara vazia tornaria a comparação verde por vácuo.
    let written = ser_mask.iter().filter(|&&m| m > 0).count();
    assert!(
        written > 10_000,
        "a máscara tem de estar escrita para haver o que comparar (got {written})"
    );

    let canvas_diff = par_buf.iter().zip(&ser_buf).filter(|(a, b)| a != b).count();
    assert_eq!(
        canvas_diff, 0,
        "a tinta divergiu entre a rota em banda e a serial ({canvas_diff} bytes)"
    );
    let mask_diff = par_mask
        .iter()
        .zip(&ser_mask)
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        mask_diff, 0,
        "a MÁSCARA divergiu entre a rota em banda e a serial ({mask_diff} texels)"
    );
}

#[test]
fn a_dab_below_the_floor_still_runs_as_one_band() {
    // O controle: abaixo do piso a divisão não acontece, e é por isso que o cap era grátis lá. Ele
    // existe para que o gate acima não possa ser lido como *"a divisão nunca acontece"*.
    const W: u32 = 300;
    const H: u32 = 300;
    const R: f32 = 60.0;
    let bbox = (R * 2.0) as usize * (R * 2.0) as usize;
    assert!(
        bbox < PARALLEL_MIN_AREA,
        "este controle TEM de ficar abaixo do piso ({bbox} >= {PARALLEL_MIN_AREA})"
    );
    let (a_buf, a_mask) = two_dabs(W, H, R, PARALLEL_MIN_AREA);
    let (b_buf, b_mask) = two_dabs(W, H, R, usize::MAX);
    assert_eq!(
        a_buf, b_buf,
        "abaixo do piso as duas chamadas são a MESMA rota"
    );
    assert_eq!(a_mask, b_mask, "idem para a máscara");
}

#[test]
fn the_cap_is_written_where_the_paint_landed_not_displaced() {
    // ⚠️ **O oráculo dos dois gates acima é uma COMPARAÇÃO entre rotas, e isso tem um limite conhecido:**
    // o recorte da máscara é computado no CORPO compartilhado, antes do ramo serial/paralelo, então um
    // erro ali move as DUAS rotas igual e a comparação fica verde — *razão entre dois doentes*
    // ([[feedback_an_identity_gate_cannot_see_a_defect_in_the_shared_body]]). Este é ABSOLUTO: a
    // máscara tem de ser escrita **onde a tinta caiu**.
    //
    // ⚠️ E ele fecha um buraco PRÉ-EXISTENTE, medido: apagar o deslocamento `y0` do recorte deixava os
    // **282 testes da crate VERDES**. As fixtures antigas do cap usam `Falloff::Constant` + disco duro,
    // onde todo texel de dentro vale o mesmo — num campo CHATO qualquer indexação concorda. Por isso
    // este usa falloff suave e mede a CAIXA do que foi escrito, não um texel.
    const W: u32 = 300;
    const H: u32 = 300;
    const R: f32 = 60.0;
    let spec = capped_brush(R);
    let mut buf = solid(W, H, [255, 255, 255, 255]);
    let mut mask = vec![0u8; (W * H) as usize];
    // Centro FORA da linha 0 de propósito: com `y0 == 0` um recorte esquecido é indistinguível do certo.
    let rect = stamp_dab_textured_masked(
        &mut buf,
        W,
        H,
        [150.0, 150.0],
        &spec,
        1.0,
        false,
        None,
        None,
        None,
        Some(&mut mask),
        [1.0, 0.0],
    )
    .expect("o dab pintou");
    assert!(rect.y > 0, "a fixture TEM de ter o dab longe da linha 0");

    let rows: Vec<u32> = (0..H)
        .filter(|y| (0..W).any(|x| mask[(y * W + x) as usize] > 0))
        .collect();
    let (first, last) = (rows[0], rows[rows.len() - 1]);
    assert!(
        first >= rect.y && last < rect.y + rect.h,
        "a máscara foi escrita nas linhas {first}..={last}, fora da pegada do dab \
         ({}..{}) — o recorte por linha está deslocado",
        rect.y,
        rect.y + rect.h
    );
}
