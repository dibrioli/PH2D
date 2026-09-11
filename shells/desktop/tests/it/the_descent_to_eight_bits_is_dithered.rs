//! **A descida para 8 bits do Inspector passa pela porta com dither** — W6 do plano
//! [`docs/Sprite_projeto/18`](../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md).
//!
//! # As duas portas, e por que é fácil trocá-las
//!
//! `ph2d_color` tem duas conversões de 16 → 8 bits, a um sufixo de distância:
//!
//! | porta | para quê | promessa |
//! |---|---|---|
//! | `rgba16_to_rgba8` | **ler** (inspecionar, gravar, reenviar) | o mesmo valor dá sempre o mesmo byte |
//! | `rgba16_to_rgba8_dithered` | **converter** a sprite, a pedido do autor | as faixas de um degradê desaparecem |
//!
//! ⚠️ **Um refactor bem-intencionado colapsa as duas em segundos** — o ramo do dither tem um `match`
//! a mais, e `image_rgba8()` faz «a mesma coisa» numa linha. O que se perde não dá erro nenhum: o
//! botão continua a funcionar, a sprite continua a converter, e as faixas voltam sem que nada fique
//! vermelho. É por isso que isto é um gate e não um comentário.

use std::path::{Path, PathBuf};

/// A porta que a descida deliberada tem de usar.
const DITHERED_DOOR: &str = "rgba16_to_rgba8_dithered";

fn convert_site() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("precision_convert.rs")
}

/// O corpo sem comentários de linha — este ficheiro **explica** a regra citando os dois nomes, e um
/// comentário não converte nada.
fn code(src: &str) -> String {
    src.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_inspector_conversion_uses_the_dithered_door() {
    let path = convert_site();
    let body =
        code(&std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}")));
    assert!(
        body.contains(DITHERED_DOOR),
        "`precision_convert.rs` deixou de nomear `{DITHERED_DOOR}`.\n\n\
         O botao `RGBA8` do Inspector nao e' uma LEITURA: e' o autor a pedir 8 bits para sempre. \
         Sem o dither, um degrade' limpo ou o halo de um brilho descem para faixas visiveis, e nada \
         no app fica vermelho a dizer isso.\n\n\
         Se a intencao foi mesmo trocar de porta, o sitio a mudar e' o plano \
         `docs/Sprite_projeto/18` W6 -- e este gate com ele."
    );
}

/// ⚠️ **Controle positivo: as duas portas existem MESMO, e são duas.**
///
/// Sem isto, renomear a função dithered faria o gate acima ficar vermelho de forma legível — mas
/// apagá-la e deixar só a fiel com o nome antigo passaria despercebido para sempre.
#[test]
fn the_faithful_door_and_the_dithered_door_are_two_different_functions() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("crates")
        .join("ph2d-color")
        .join("src");
    let precision = std::fs::read_to_string(src.join("precision.rs")).expect("precision.rs");
    let dither = std::fs::read_to_string(src.join("dither.rs")).expect("dither.rs");
    assert!(
        code(&precision).contains("pub fn rgba16_to_rgba8("),
        "a porta FIEL desapareceu de `precision.rs` — leituras passariam a dither*ar*"
    );
    assert!(
        code(&dither).contains(&format!("pub fn {DITHERED_DOOR}(")),
        "a porta com dither desapareceu de `dither.rs`"
    );
}
