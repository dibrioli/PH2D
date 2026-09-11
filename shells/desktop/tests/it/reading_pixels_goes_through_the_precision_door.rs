//! **Quem lê pixels de um `Asset` passa pela porta — ou declara, no sítio, por que não.**
//!
//! Plano [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md),
//! auditoria da W2.
//!
//! # O defeito que este gate existe para impedir
//!
//! `ph2d_asset::Asset` é `#[non_exhaustive]` — e o cabeçalho da crate diz que isso é de propósito,
//! para *"adding variants doesn't break downstream matches"*. Ele corta nos dois sentidos: um
//! `match ... { Asset::ImageRgba8 {..} => ..., _ => None }` **aceita a variante nova em silêncio,
//! sem a tratar**, e o compilador nunca abre a boca.
//!
//! A auditoria de 2026-08-20 encontrou **doze** sítios assim no shell, todos com a mesma forma e
//! todos com um sintoma diferente e mentiroso quando a imagem fosse de 16 bits:
//!
//! | sítio | o que o utilizador veria |
//! |---|---|
//! | `image_import` / `project_assets` / `inspector_strategy` (regrow) | a célula do atlas reconstruída **VAZIA** |
//! | `texture_edit` | a ferramenta de imagem abre **sem imagem** |
//! | `painter_bridge_assets` | *"not an RGBA image"* sobre uma imagem que **é** RGBA |
//! | `sheet_import` | *"is not an image"* sobre um `.png` válido |
//! | `inspector_commits` / `snapshots` | o tamanho da sprite **desaparece** do Inspector |
//!
//! A cura foi encaminhá-los por [`ph2d_asset::Asset::image_rgba8`] (que converte, e **não copia**
//! quando já é de 8 bits) ou por `image_dimensions`. Este gate é o que impede o décimo-terceiro.
//!
//! # Por que um MARCADOR NO SÍTIO e não uma allowlist central
//!
//! Há duas excepções legítimas, e as duas são caminhos de **escrita** (`project_sprite_pixels` e
//! `project_assets`): converter para 8 bits ali apagaria a precisão no ficheiro gravado. *Uma
//! conversão de conveniência num caminho de leitura é um atalho; no de escrita é destruição de
//! dados.*
//!
//! ⚠️ A forma óbvia de as acomodar seria uma lista de ficheiros dentro deste teste. **Não é o que
//! isto faz**, e a razão está registada no projeto: uma allowlist central envelhece longe do código
//! que a justifica, e a primeira pessoa apressada acrescenta-lhe uma linha em vez de encaminhar o
//! sítio. Em vez disso, quem contorna a porta escreve `PRECISION-BYPASS:` **no próprio sítio**, com
//! o motivo — e o gate só verifica que o marcador está lá. A justificação viaja com o código, e
//! quem apagar o bypass apaga o marcador com ele.

use std::path::{Path, PathBuf};

/// O marcador que um bypass tem de trazer, no próprio ficheiro.
const MARKER: &str = "PRECISION-BYPASS:";

/// O que denuncia um `match` na variante em vez de uma passagem pela porta.
const VARIANT_MATCH: &str = "Asset::ImageRgba8 {";

fn shell_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Todos os `.rs` sob `shells/desktop/src`, recursivamente.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = std::fs::read_dir(dir).unwrap_or_else(|e| panic!("read_dir {dir:?}: {e}"));
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// O corpo sem comentários de linha — um comentário que **cite** a variante (há vários, a explicar
/// precisamente esta regra) não pode contar como um `match` nela.
fn code_without_comments(src: &str) -> String {
    src.lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn every_pixel_read_goes_through_the_door_or_declares_its_bypass() {
    let mut files = Vec::new();
    rust_files(&shell_src(), &mut files);
    assert!(
        files.len() > 50,
        "so' {} ficheiros varridos — a varredura partiu-se e este gate mede o vazio",
        files.len()
    );

    let mut offenders = Vec::new();
    for path in &files {
        let src = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
        if !code_without_comments(&src).contains(VARIANT_MATCH) {
            continue;
        }
        if src.contains(MARKER) {
            continue;
        }
        offenders.push(format!("  {}", path.display()));
    }

    assert!(
        offenders.is_empty(),
        "estes ficheiros casam `{VARIANT_MATCH}` sem declarar um bypass:\n{}\n\n\
         `ph2d_asset::Asset` e' `#[non_exhaustive]`: um `match` na variante aceita uma imagem de \
         16 bits EM SILENCIO, sem a tratar, e o compilador nao avisa. O sintoma nao e' um erro -- \
         e' uma celula de atlas vazia, uma ferramenta que abre sem imagem, ou o tamanho da sprite \
         a sumir do Inspector.\n\n\
         Encaminhe por `Asset::image_rgba8()` (converte, e nao copia quando ja' e' de 8 bits) ou \
         por `Asset::image_dimensions()`.\n\
         Se o sitio for um caminho de ESCRITA, onde converter apagaria a precisao no ficheiro \
         gravado, escreva `{MARKER} <motivo>` num comentario ao lado -- a justificacao viaja com o \
         codigo, e nao numa lista dentro deste teste.",
        offenders.join("\n")
    );
}

/// **Controle positivo — sem ele o gate acima passaria por não encontrar nada.**
///
/// Se a varredura, o filtro de comentários ou a agulha se partissem, `offenders` ficaria vazio e o
/// teste verde sobre um aparelho morto. Este exige que os **dois** bypasses conhecidos existam
/// mesmo, com o marcador — e que eles sejam os caminhos de escrita, não outra coisa qualquer.
#[test]
fn the_two_known_bypasses_are_both_write_paths_and_are_marked() {
    let src = shell_src();
    for (file, why) in [
        (
            "project_sprite_pixels.rs",
            "os pixels proprios de uma sprite",
        ),
        ("project_assets.rs", "as celulas do atlas"),
    ] {
        let path = src.join(file);
        let body = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {path:?}: {e}"));
        assert!(
            code_without_comments(&body).contains(VARIANT_MATCH),
            "{file} deixou de casar a variante ({why}) — se ele foi encaminhado pela porta, \
             APAGUE esta entrada do controle em vez de a silenciar"
        );
        assert!(
            body.contains(MARKER),
            "{file} contorna a porta e perdeu o `{MARKER}` — o gate irmao ja' o estaria a acusar"
        );
    }
}
