//! Os gates da paleta — a gramática que a CPU e o device partilham.

use super::*;

/// **A PALETA NUNCA SAI VAZIA**, e os quatro caminhos que poderiam esvaziá-la
/// caem todos na de fábrica.
///
/// ⚠️ Este é o contrato de que `cycle` e o WGSL dependem para indexar sem `get`.
/// FALSIFICADO se algum deles devolvesse `[]`: a CPU entraria em pânico e o
/// device leria lixo do slot 1.
#[test]
fn the_palette_is_never_empty_whatever_the_string_says() {
    let factory = DEFAULT_PALETTE_FALLBACK.to_vec();
    assert!(!factory.is_empty(), "a de fábrica tem de ter cor");
    for (text, why) in [
        (None, "nada autorado"),
        (Some(""), "string vazia"),
        (Some("p1"), "a paleta explicitamente VAZIA"),
        (Some("g1 0:1,0,0"), "isso é um gradiente, não uma paleta"),
        (Some("p1 lixo"), "malformada"),
    ] {
        assert_eq!(palette_of(text), factory, "{why}");
    }
}

/// **UMA PALETA AUTORADA CHEGA INTEIRA** — o caminho que não é fallback.
#[test]
fn an_authored_palette_survives_the_round_trip() {
    let p = vec![[0.1, 0.2, 0.3, 1.0], [0.4, 0.5, 0.6, 0.5]];
    assert_eq!(
        palette_of(Some(&ph2d_color::serialize_palette(&p))),
        p,
        "as cores autoradas, exactamente"
    );
}

/// **O TETO É O DO BUFFER, e ele TRUNCA nos dois caminhos.**
///
/// ⚠️ A metade que importa é a segunda: a CPU e a LUT têm de concordar sobre o
/// COMPRIMENTO. Uma truncagem só de um lado seria a peça `1024` a pegar a cor
/// `0` num caminho e a cor `1024` no outro — divergência silenciosa, visível só
/// numa paleta que ninguém edita à mão.
#[test]
fn the_ceiling_is_the_device_buffers_and_both_paths_truncate_alike() {
    let big: Vec<[f32; 4]> = (0..MAX_COLORS + 7)
        .map(|k| {
            #[expect(clippy::cast_precision_loss, reason = "1031 cabe exato em f32")]
            let v = k as f32;
            [v, 0.0, 0.0, 1.0]
        })
        .collect();
    let text = ph2d_color::serialize_palette(&big);
    let cpu = palette_of(Some(&text));
    assert_eq!(cpu.len(), MAX_COLORS, "a CPU trunca no teto");

    let mut lut = vec![0.0_f32; LUT_LEN as usize];
    fill_lut(&text, &mut lut);
    #[expect(clippy::cast_precision_loss, reason = "1024 cabe exato em f32")]
    let want = MAX_COLORS as f32;
    assert_eq!(lut[0], want, "o cabeçalho conta o mesmo que a CPU");
    // A última cor sobrevivente, nos dois caminhos.
    let last = MAX_COLORS - 1;
    assert_eq!(cpu[last], big[last]);
    assert_eq!(lut[1 + 4 * last], big[last][0], "a LUT guarda a mesma cor");
}

/// **O BUFFER É `[len, RGBA…]`, e cada cor ocupa quatro slots consecutivos.**
///
/// FALSIFICADO por um `stride` errado (3 em vez de 4, ou um off-by-one no
/// cabeçalho) — o modo de falha que pintaria a peça `k` com o verde da peça
/// `k−1` e ninguém saberia dizer porquê olhando a tira de swatches.
#[test]
fn the_lut_layout_is_a_count_header_then_four_floats_per_colour() {
    let p = vec![[0.25, 0.5, 0.75, 1.0], [0.125, 0.0, 1.0, 0.5]];
    let mut lut = vec![9.0_f32; LUT_LEN as usize];
    fill_lut(&ph2d_color::serialize_palette(&p), &mut lut);
    assert_eq!(lut[0], 2.0, "duas cores");
    assert_eq!(&lut[1..5], &p[0], "a cor 0 no slot 1");
    assert_eq!(&lut[5..9], &p[1], "a cor 1 no slot 5");
    assert_eq!(lut[9], 9.0, "o resto fica como o chamador o entregou");
}

/// **A CONTA DO TETO É A QUE O DOC ESCREVE** — 1 cabeçalho + 4 floats por cor.
///
/// ⚠️ Um teto cujo número não se deriva do recurso é um palpite (§0). Este gate
/// é o que impede o doc e a const de divergirem no dia em que alguém mexer numa.
#[test]
fn the_buffer_length_is_the_header_plus_four_floats_per_colour() {
    assert_eq!(LUT_LEN as usize, 1 + 4 * MAX_COLORS);
    assert_eq!(LUT_LEN, 4097, "16.388 bytes por nó — o número do doc");
}
