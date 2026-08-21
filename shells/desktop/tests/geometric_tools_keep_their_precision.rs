//! **AS TRÊS FERRAMENTAS GEOMÉTRICAS PRESERVAM 16 BITS — e as outras seis dizem que não.**
//!
//! Plano [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md),
//! W4-bis. Enio, 2026-08-20: *"após aplicar algumas das tools a sprite volta para RGBA8 no
//! inspector"*.
//!
//! # A distinção, e por que ela precisa de um gate
//!
//! **Trim Transparency**, **Make Square** e **Padding** nunca calculam o valor de um pixel: elas
//! recortam, copiam e preenchem com transparente. Para elas a perda de precisão era **gratuita** —
//! acontecia só porque o `SpriteImage` que as ferramentas falam é `Vec<u8>`.
//!
//! As outras (Upscale, Rasterize, Real Size, Color Equalization, BG-Removal, Equalize Sizes)
//! **resamplam ou recolorem**. Preservar 16 bits nelas exige um resampler e uma pilha de cor de 16
//! bits — código novo, não plumbing. Elas convertem, com o aviso do funil.
//!
//! ⚠️ **Esta lista é a coisa que apodrece.** Uma ferramenta nova que só mova pixels e não passe
//! pela porta perde precisão em silêncio; uma que resample e passe **mente** (o rótulo diria 16
//! sobre valores que atravessaram 8). O gate abaixo lê o código e obriga cada lado a declarar-se.

use std::fs;
use std::path::{Path, PathBuf};

/// A chamada que compromete um resultado geométrico preservando a precisão.
const GEOMETRIC_DOOR: &str = "commit_geometric_edit(";
/// A chamada do funil normal, que converte para 8 bits e avisa.
const EIGHT_BIT_DOOR: &str = "commit_edited_texture(";

fn handler(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/hero_intents/image_edit")
        .join(format!("{name}.rs"))
}

fn body(name: &str) -> String {
    let p = handler(name);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {p:?}: {e}"))
}

/// **As três que só movem pixels passam pela porta que preserva.**
#[test]
fn the_three_geometric_tools_go_through_the_preserving_door() {
    for name in ["trim_transparency", "make_square", "padding"] {
        let src = body(name);
        assert!(
            src.contains(GEOMETRIC_DOOR),
            "`{name}` nao passa por `{GEOMETRIC_DOOR}` — ela nao calcula valor de pixel nenhum, \
             entao converte 16 bits para 8 POR NADA. Ver `crate::precision_geometry`."
        );
        assert!(
            src.contains("blit_rgba16"),
            "`{name}` chama a porta mas nao constroi o buffer de 16 bits — o `None` faz a porta \
             cair no caminho de 8 bits em silencio, e o gate acima passaria na mesma"
        );
    }
}

/// **As que resamplam ou recolorem NÃO usam a porta que preserva.**
///
/// ⚠️ É a metade que impede a mentira. Se o Upscale passasse por ali, o Inspector diria `RGBA16`
/// sobre uma imagem cujos valores foram calculados em 8 bits — *o rótulo tem de prometer o que o
/// modelo entrega*, e esta é a terceira vez que esta wave paga essa lei.
#[test]
fn the_resampling_tools_do_not_claim_to_preserve_precision() {
    for name in [
        "upscale",
        "rasterize",
        "real_size",
        "color_equalization",
        "bgremoval",
        "equalize_sizes",
    ] {
        let p = handler(name);
        if !p.exists() {
            continue; // um handler que mudou de nome nao e' um handler que mente
        }
        let src = fs::read_to_string(&p).unwrap_or_else(|e| panic!("read {p:?}: {e}"));
        assert!(
            !src.contains(GEOMETRIC_DOOR),
            "`{name}` passa por `{GEOMETRIC_DOOR}`, mas ela RESAMPLA ou RECOLORE: os valores \
             atravessam 8 bits e o Inspector passaria a dizer RGBA16 sobre uma imagem que ja' \
             perdeu a precisao. Se ela deixou de resamplar, mova-a para o gate irmao."
        );
    }
}

/// **Controle positivo:** as duas portas existem mesmo, e são distintas.
///
/// ⚠️ Sem isto, renomear qualquer uma delas faria os dois gates acima passarem por não encontrarem
/// nada — verdes sobre um aparelho morto.
#[test]
fn both_doors_are_real_and_distinct() {
    let funnel = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/hero_intents/texture_edit.rs"),
    )
    .expect("texture_edit.rs");
    assert!(
        funnel.contains(&format!(
            "pub(crate) fn {}",
            GEOMETRIC_DOOR.trim_end_matches('(')
        )),
        "a porta geometrica `{GEOMETRIC_DOOR}` deixou de existir com esse nome"
    );
    assert!(
        funnel.contains(&format!(
            "pub(crate) fn {}",
            EIGHT_BIT_DOOR.trim_end_matches('(')
        )),
        "o funil de 8 bits `{EIGHT_BIT_DOOR}` deixou de existir com esse nome"
    );
    assert_ne!(GEOMETRIC_DOOR, EIGHT_BIT_DOOR);
}
