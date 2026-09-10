//! **O CISALHAMENTO** (ciclo 3, W3 — doc 106 §2.4): o terço do afim que faltava ao grupo.
//!
//! Cortado do `lib.rs` pelo teto de LOC do HR-18, pela mesma costura que o `pivot_tests`.

use super::*;

/// **O default é o nó que shipou, AO BIT — e é estrutural, não aritmético.**
///
/// ⚠️ Com `kx = 0` a conta geral daria `p.x·sx + p.y·0 + ox`, e **`a + 0.0` não é `a` quando
/// `a` é `−0.0`**. É por isso que o ramo neutro fica escrito à parte em vez de sair da
/// expressão geral com o knob a zero — a mesma disciplina que a
/// [`super::folded_offset`] já tinha para o pivô.
#[test]
fn the_neutral_shear_is_the_node_that_shipped_bit_for_bit() {
    for (sx, sy, ox, oy) in [
        (2.0f32, 2.0f32, 1.0f32, -1.0f32),
        (0.5, 1.5, -3.0, 0.0),
        (1.0, 1.0, 0.0, 0.0),
        (-1.0, 0.25, 7.5, -2.5),
    ] {
        for p in [[2.0f32, 3.0], [-0.0, -0.0], [0.0, -0.0], [-4.5, 9.25]] {
            let neutro = xform_masked(p, sx, sy, NO_SHEAR, ox, oy, 1.0);
            let shipou = [p[0] * sx + ox, p[1] * sy + oy];
            assert_eq!(
                [neutro[0].to_bits(), neutro[1].to_bits()],
                [shipou[0].to_bits(), shipou[1].to_bits()],
                "p {p:?} sx {sx} sy {sy}"
            );
        }
    }
    // E a dobra do pivo tambem: com cisalhamento neutro ela e' a expressao por EIXO.
    let (fx, fy) = folded_offset(0.63, 1.4, NO_SHEAR, 2.9, -1.4, [3.7, -2.1]);
    assert_eq!(
        [fx.to_bits(), fy.to_bits()],
        [
            (2.9f32 + 3.7 * (1.0 - 0.63)).to_bits(),
            (-1.4f32 + -2.1 * (1.0 - 1.4)).to_bits()
        ]
    );
}

/// ⭐ **O cisalhamento INCLINA, e a régua é a que o doc promete: `1` é 45°.**
///
/// FALSIFICADO por um `kx` que mova o `y` (um cisalhamento em X é uma translação por linha, e
/// as linhas não mudam de altura), e por um deslocamento que não seja `kx · y`.
#[test]
fn a_unit_skew_leans_forty_five_degrees_and_moves_only_along_its_axis() {
    let sh = Shear { kx: 1.0, ky: 0.0 };
    // Uma coluna vertical em x = 0: cada ponto anda EXACTAMENTE o seu proprio `y`.
    for y in [-2.0f32, 0.0, 1.0, 3.5] {
        let out = xform_masked([0.0, y], 1.0, 1.0, sh, 0.0, 0.0, 1.0);
        assert!(
            (out[0] - y).abs() < 1e-6,
            "kx = 1 leva x de 0 para o proprio y ({y}): {out:?}"
        );
        assert!((out[1] - y).abs() < 1e-6, "e nao mexe no y: {out:?}");
    }
    // Meia inclinacao e' meia altura — linear, que e' o que um cisalhamento E'.
    let meio = xform_masked(
        [0.0, 4.0],
        1.0,
        1.0,
        Shear { kx: 0.5, ky: 0.0 },
        0.0,
        0.0,
        1.0,
    );
    assert!((meio[0] - 2.0).abs() < 1e-6, "{meio:?}");
    // E o eixo Y e' o simetrico: `y' = y + ky·x`, com o `x` INTACTO.
    // ⚠️ A 1.a redaccao pedia `x == 0` sobre um ponto em `x = 3` e reprovou sobre codigo
    // correcto — um cisalhamento nao move o eixo que ele le'. E a fixtura tem `y != x` de
    // proposito: em `(3, 0)` o resultado `[3, 3]` satisfaz as duas afirmacoes por acidente.
    let ky = xform_masked(
        [3.0, 1.0],
        1.0,
        1.0,
        Shear { kx: 0.0, ky: 1.0 },
        0.0,
        0.0,
        1.0,
    );
    assert!(
        (ky[0] - 3.0).abs() < 1e-6 && (ky[1] - 4.0).abs() < 1e-6,
        "{ky:?}"
    );
}

/// ⭐⭐ **O cisalhamento roda em torno do PIVÔ como tudo o resto** — e com pivô a dobra deixa de
/// ser por eixo, porque `c − c·M` mistura os dois.
///
/// A régua é o PONTO FIXO: o elemento que está no pivô não se move, qualquer que seja o
/// cisalhamento. É a única afirmação que separa uma dobra certa de uma que translada a figura.
#[test]
fn the_pivot_is_the_fixed_point_of_a_sheared_transform() {
    let sh = Shear { kx: 0.8, ky: -0.35 };
    let c = [3.7f32, -2.1];
    for (sx, sy) in [(1.0f32, 1.0f32), (0.63, 1.4)] {
        let (ox, oy) = folded_offset(sx, sy, sh, 0.0, 0.0, c);
        let fixo = xform_masked(c, sx, sy, sh, ox, oy, 1.0);
        assert!(
            (fixo[0] - c[0]).abs() < 1e-4 && (fixo[1] - c[1]).abs() < 1e-4,
            "sx {sx} sy {sy}: o pivo moveu-se para {fixo:?}"
        );
    }
}

/// **Os dois knobs estão no painel, e o kernel LÊ-OS.**
///
/// ⚠️ Um param declarado que o kernel não lista é um controlo que funciona na CPU e desaparece
/// no dispositivo — a espécie que só um smoke com a placa ligada revela.
#[test]
fn the_skew_is_paintable_and_the_kernel_reads_it() {
    for k in [SKEW_X, SKEW_Y] {
        assert!(
            PARAM_HINTS.iter().any(|h| h.param == k),
            "{k} sem hint — o cartao nao o pinta"
        );
        assert!(
            MANIFEST.params.iter().any(|p| p.name == k),
            "{k} fora do manifesto"
        );
        assert!(
            GPU_KERNEL.params.contains(&k),
            "{k} fora da lista do kernel — vivo na CPU e morto no dispositivo"
        );
        assert!(
            GPU_KERNEL.wgsl.contains(&format!("params.{k}")),
            "{k} declarado ao kernel e nunca lido pelo corpo"
        );
    }
}
