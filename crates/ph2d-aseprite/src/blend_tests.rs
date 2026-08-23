//! **Os gates dos 19 modos** ([`super`]) — todos por IDENTIDADE, porque não há oráculo.
//!
//! Um render de referência exigiria o Aseprite instalado. O que se pode afirmar sem ele é álgebra:
//! multiplicar por branco não muda nada, `screen` com preto não muda nada, `difference` consigo
//! mesmo é preto. ⚠️ **Cada identidade morde um erro diferente** — trocar dois canais, inverter um
//! sinal, esquecer o arredondamento de 8 bits. Uma tabela de números copiados de algum lado não
//! morderia nenhum, porque ninguém saberia dizer de onde ela veio.

use super::*;

const OPAQUE: u8 = 255;

fn on_opaque(mode: BlendMode, back: [u8; 3], src: [u8; 3]) -> [u8; 3] {
    let r = blend(
        mode,
        [back[0], back[1], back[2], 255],
        [src[0], src[1], src[2], 255],
        OPAQUE,
    );
    assert_eq!(r[3], 255, "dois opacos dao um opaco");
    [r[0], r[1], r[2]]
}

/// **Os 19 números do ficheiro dão 19 modos, e nada mais.** O 19 não existe — e é isso que faz um
/// ficheiro de uma versão futura virar uma NOTA em vez de um modo escolhido à sorte.
#[test]
fn the_file_numbers_map_one_to_one() {
    let all: Vec<_> = (0..19).map(|v| BlendMode::from_file(v).unwrap()).collect();
    assert_eq!(all.len(), 19);
    assert_eq!(all[0], BlendMode::Normal);
    assert_eq!(all[18], BlendMode::Divide);
    // Sem duplicados: dois números a dar o mesmo modo seria um braço em falta.
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            assert_ne!(a, b, "dois numeros do ficheiro dao o mesmo modo");
        }
    }
    assert_eq!(BlendMode::from_file(19), None);
    assert_eq!(BlendMode::from_file(9999), None);
}

/// **Os neutros de cada modo separável.** Cada linha é uma lei que o modo tem de obedecer, e uma
/// troca de argumentos ou um sinal invertido parte pelo menos uma delas.
#[test]
fn every_separable_mode_honours_its_neutral() {
    let b = [40_u8, 130, 210];
    let white = [255_u8; 3];
    let black = [0_u8; 3];
    assert_eq!(
        on_opaque(BlendMode::Normal, b, white),
        white,
        "Normal e' a fonte"
    );
    assert_eq!(
        on_opaque(BlendMode::Multiply, b, white),
        b,
        "x branco = base"
    );
    assert_eq!(
        on_opaque(BlendMode::Multiply, b, black),
        black,
        "x preto = preto"
    );
    assert_eq!(
        on_opaque(BlendMode::Screen, b, black),
        b,
        "screen com preto = base"
    );
    assert_eq!(on_opaque(BlendMode::Screen, b, white), white);
    assert_eq!(on_opaque(BlendMode::Darken, b, white), b);
    assert_eq!(on_opaque(BlendMode::Lighten, b, black), b);
    assert_eq!(on_opaque(BlendMode::Difference, b, black), b);
    assert_eq!(
        on_opaque(BlendMode::Difference, b, b),
        black,
        "consigo mesmo = preto"
    );
    assert_eq!(on_opaque(BlendMode::Exclusion, b, black), b);
    assert_eq!(on_opaque(BlendMode::Addition, b, black), b);
    assert_eq!(
        on_opaque(BlendMode::Addition, b, white),
        white,
        "satura, nao roda"
    );
    assert_eq!(on_opaque(BlendMode::Subtract, b, black), b);
    assert_eq!(
        on_opaque(BlendMode::Subtract, white, white),
        black,
        "nao vai abaixo de zero"
    );
    assert_eq!(
        on_opaque(BlendMode::ColorDodge, black, b),
        black,
        "base preta fica preta"
    );
    assert_eq!(
        on_opaque(BlendMode::ColorBurn, white, b),
        white,
        "base branca fica branca"
    );
    assert_eq!(
        on_opaque(BlendMode::Divide, b, white),
        b,
        "dividir por branco = base"
    );
}

/// **`Overlay(b, s) = HardLight(s, b)`** — a única diferença entre os dois é a troca de
/// argumentos, e escrevê-los como duas fórmulas é como uma delas fica errada em silêncio.
#[test]
fn overlay_is_hard_light_with_the_arguments_swapped() {
    for b in (0..=255).step_by(17) {
        for s in (0..=255).step_by(17) {
            let o = on_opaque(BlendMode::Overlay, [b; 3], [s; 3]);
            let h = on_opaque(BlendMode::HardLight, [s; 3], [b; 3]);
            assert_eq!(o, h, "overlay({b},{s}) != hard_light({s},{b})");
        }
    }
}

/// **`SoftLight` com meio-cinza não mexe na base** (o ponto neutro da fórmula do W3C), e ele nunca
/// sai do intervalo — a raiz quadrada é o sítio onde isso pode falhar.
#[test]
fn soft_light_is_neutral_at_mid_grey_and_never_leaves_the_range() {
    for b in (0..=255).step_by(5) {
        let r = on_opaque(BlendMode::SoftLight, [b; 3], [128; 3]);
        assert!(
            r[0].abs_diff(b) <= 1,
            "soft light com meio-cinza mudou a base {b} para {}",
            r[0]
        );
        for s in (0..=255).step_by(51) {
            let _ = on_opaque(BlendMode::SoftLight, [b; 3], [s; 3]); // clamp: nao entra em panico
        }
    }
}

/// **As quatro não-separáveis obedecem à definição delas**, que é o que as distingue: `Luminosity`
/// pega a luz da fonte e a cor da base; `Color` faz o contrário; `Hue`/`Saturation` mantêm a luz da
/// base.
///
/// ⚠️ A luz é a do W3C (`0,3·R + 0,59·G + 0,11·B`) — a mesma que a implementação usa, então este
/// gate afirma **consistência**, e o que ele morde é uma troca entre os quatro modos (que é o erro
/// provável: quatro braços quase iguais num `match`).
#[test]
fn the_four_non_separable_modes_do_what_their_names_say() {
    let lum8 = |c: [u8; 3]| 0.3 * f64::from(c[0]) + 0.59 * f64::from(c[1]) + 0.11 * f64::from(c[2]);
    let base = [200_u8, 60, 30];
    let src = [20_u8, 90, 240];

    let l = on_opaque(BlendMode::Luminosity, base, src);
    assert!(
        (lum8(l) - lum8(src)).abs() < 2.0,
        "Luminosity tinha de trazer a LUZ da fonte: {} vs {}",
        lum8(l),
        lum8(src)
    );
    let c = on_opaque(BlendMode::Color, base, src);
    assert!(
        (lum8(c) - lum8(base)).abs() < 2.0,
        "Color tinha de manter a luz da BASE"
    );
    for mode in [BlendMode::Hue, BlendMode::Saturation] {
        let r = on_opaque(mode, base, src);
        assert!(
            (lum8(r) - lum8(base)).abs() < 2.0,
            "{} tinha de manter a luz da base",
            mode.label()
        );
    }
    // E os quatro dão resultados DIFERENTES entre si nesta cor — senão um braço estaria a cair no
    // outro sem que nada reprovasse.
    let all = [
        on_opaque(BlendMode::Hue, base, src),
        on_opaque(BlendMode::Saturation, base, src),
        on_opaque(BlendMode::Color, base, src),
        on_opaque(BlendMode::Luminosity, base, src),
    ];
    for (i, a) in all.iter().enumerate() {
        for b in &all[i + 1..] {
            assert_ne!(a, b, "dois dos quatro modos nao-separaveis dao o mesmo");
        }
    }
}

/// **Sobre o vazio, qualquer modo é a fonte** — e o alfa dela leva a opacidade da camada. É a
/// primeira coisa que qualquer cel faz, porque o quadro começa transparente.
#[test]
fn over_nothing_every_mode_is_the_source() {
    for v in 0..19 {
        let mode = BlendMode::from_file(v).unwrap();
        let r = blend(mode, [0, 0, 0, 0], [10, 20, 30, 200], 255);
        assert_eq!(r, [10, 20, 30, 200], "{} sobre o vazio", mode.label());
        let half = blend(mode, [0, 0, 0, 0], [10, 20, 30, 200], 128);
        assert_eq!(
            [half[0], half[1], half[2]],
            [10, 20, 30],
            "a opacidade nao pode mexer na COR"
        );
        assert!(half[3] < 200, "a opacidade tinha de baixar o alfa");
    }
}

/// **Opacidade zero é invisível, em todos os modos** — a metade oposta do gate acima, e a que
/// impede um modo de «vazar» a cor dele por uma camada desligada.
#[test]
fn zero_opacity_never_shows() {
    let back = [11_u8, 22, 33, 255];
    for v in 0..19 {
        let mode = BlendMode::from_file(v).unwrap();
        assert_eq!(
            blend(mode, back, [250, 250, 250, 255], 0),
            back,
            "{} vazou com opacidade zero",
            mode.label()
        );
    }
}

/// **O alfa compõe como `source-over`, seja qual for o modo** — os 19 escolhem a COR e nada mais.
/// Se algum deles calculasse o próprio alfa, esta lei partia-se nele.
#[test]
fn the_alpha_is_source_over_in_every_mode() {
    let back = [100_u8, 100, 100, 128];
    let src = [200_u8, 50, 25, 128];
    // 128 sobre 128 em alfa reto: 128 + 128 - 128·128/255 ≈ 192.
    for v in 0..19 {
        let mode = BlendMode::from_file(v).unwrap();
        let a = blend(mode, back, src, 255)[3];
        assert!(
            (191..=193).contains(&a),
            "{} deu alfa {a}, e o source-over da' ~192",
            mode.label()
        );
    }
}
