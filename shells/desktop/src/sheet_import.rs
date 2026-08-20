//! **O IMPORT de uma folha hand-packed** — largar `folha.png` + `folha.json` na janela.
//!
//! Irmão de [`crate::image_import`] (que importa uma imagem = um sprite) e de
//! [`crate::input_drop`] (que recebe o gesto). O corte é por responsabilidade: aqui uma imagem
//! vira **N sprites**, e é a única porta do app que lê o formato do artista.
//!
//! ## A porta é o drag & drop porque é a única que existe
//!
//! ⚠️ **Este app não tem diálogo de arquivo** — o `io_menu` é stub e o `Ctrl+S`/`Ctrl+O` usam um
//! caminho fixo. Pendurar o import num botão "Importar folha…" seria construir um botão que não
//! consegue abrir nada. O `handle_dropped_files` já é como toda imagem entra, e até hoje ele
//! filtrava por `is_supported_image_extension` — então um `.json` largado era **ignorado em
//! silêncio**, com um toast a dizer que fora saltado.
//!
//! ## O consumidor que faltava há três meses
//!
//! `ph2d_asset::parse_atlas_meta` (Aseprite "Hash" + TexturePacker) foi escrito em 2026-05-12,
//! tem **um** commit, e **nunca foi chamado por nada**. Era a metade fácil do hand-packed, feita
//! e deixada órfã — exatamente o modo de falha nº 3 da DIRETIVA (*isolamento que fabrica fios
//! órfãos*). Este módulo é o consumidor dele.
//!
//! ## Uma folha é UMA textura
//!
//! Os N sprites partilham uma entrada do `IndividualTextureStore` e diferem só no `region_rect`.
//! É a razão de existir do hand-packed (uma textura ⇒ um draw call para a folha inteira), e sai
//! de graça porque aquele store já tem refcount e o `region_subrect()` do extract já converte um
//! retângulo em pixels para UV — vide `ph2d_ecs::SpriteSheetRef`.

use ph2d_ecs::{Name, SimWorld, SpriteSheetRef, Transform};
use ph2d_render::{Sprite, SpriteRenderer};
use std::path::{Path, PathBuf};

/// O que aconteceu a uma folha largada.
pub(crate) enum SheetImportResult {
    Ok {
        /// Nome legível da folha (o do `.json`).
        name: String,
        /// Quantos sprites nasceram.
        regions: usize,
        /// Bits das entidades criadas, para semear a seleção.
        bits: Vec<u64>,
    },
    Err {
        name: String,
        error: String,
    },
}

/// Separa os `.json` do resto de uma leva largada.
///
/// ⚠️ Feito ANTES do filtro de imagem do `input_drop`, pela mesma razão que as malhas 3D saem
/// antes dele: aquele filtro emite *"Skipped non-image"* por arquivo que não reconhece, e um
/// `.json` de folha largado com a folha produziria um aviso de que foi ignorado — a resposta
/// errada, com a certeza da resposta certa.
pub(crate) fn partition_sheet_metadata(paths: &[PathBuf]) -> (Vec<PathBuf>, Vec<PathBuf>) {
    paths.iter().cloned().partition(|p| {
        p.extension()
            .and_then(|s| s.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case("json"))
    })
}

/// A imagem que uma folha referencia, resolvida contra a pasta do `.json`.
///
/// Exposta para o `input_drop` poder **retirar essa imagem da leva** antes do import normal —
/// senão largar `folha.png` + `folha.json` daria a folha **e** um sprite avulso com a folha
/// inteira desenhada nele.
pub(crate) fn referenced_image(json_path: &Path) -> Option<PathBuf> {
    let bytes = std::fs::read(json_path).ok()?;
    let meta = ph2d_asset::parse_atlas_meta(&bytes).ok()?;
    Some(json_path.parent()?.join(meta.image_filename))
}

/// Importa uma folha: lê o JSON, carrega o PNG irmão, sobe UMA textura e nasce um sprite por
/// região, numa grade ancorada no ponto onde o artista largou.
#[allow(clippy::too_many_arguments)]
pub(crate) fn import_sheet(
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &ph2d_asset::AssetDb,
    sheets: &mut std::collections::BTreeMap<u32, ph2d_sprite_sheet::AuthoredSheet>,
    sheet_textures: &mut std::collections::BTreeMap<u32, u32>,
    next_sheet_id: &mut u32,
    json_path: &Path,
    anchor_world: [f32; 2],
    pixels_per_meter: f32,
) -> SheetImportResult {
    let name = json_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("sheet")
        .to_owned();
    let fail = |error: String| SheetImportResult::Err {
        name: name.clone(),
        error,
    };
    let bytes = match std::fs::read(json_path) {
        Ok(b) => b,
        Err(e) => return fail(format!("read: {e}")),
    };
    let meta = match ph2d_asset::parse_atlas_meta(&bytes) {
        Ok(m) => m,
        Err(e) => return fail(e.to_string()),
    };
    // O PNG irmão. O parser não toca no disco de propósito (é uma função pura sobre bytes), então
    // resolver o caminho é responsabilidade de quem chama — aqui.
    let Some(dir) = json_path.parent() else {
        return fail("the metadata file has no directory".into());
    };
    let image_path = dir.join(&meta.image_filename);
    let image_bytes = match std::fs::read(&image_path) {
        Ok(b) => b,
        Err(e) => {
            // ⚠️ A mensagem NOMEIA o arquivo que falta: o `.json` aponta para um `.png` por nome,
            // e o modo de falha nº 1 deste formato é o artista mover ou renomear um dos dois.
            return fail(format!(
                "{} not found next to the metadata ({e})",
                meta.image_filename
            ));
        }
    };
    // Decodifica pelo MESMO caminho do import de imagens (`pack_image`): o `AssetDb` detecta o
    // formato, hasheia por conteúdo (HR-6) e devolve RGBA8 justo. Uma segunda porta de decode
    // seria uma segunda resposta a *"que pixels tem este arquivo?"*.
    let asset_id = match asset_db.insert_image_bytes(&image_bytes) {
        Ok(id) => id,
        Err(e) => return fail(format!("decode {}: {e}", meta.image_filename)),
    };
    let Some(asset) = asset_db.get(&asset_id) else {
        return fail(format!("{} vanished after decode", meta.image_filename));
    };
    let ph2d_asset::Asset::ImageRgba8 {
        width,
        height,
        pixels,
    } = &*asset
    else {
        return fail(format!("{} is not an image", meta.image_filename));
    };
    let (width, height, pixels) = (*width, *height, pixels.to_vec());
    // O `.json` declara o tamanho da folha; se o `.png` discorda, os dois divergiram e cada
    // retângulo passa a apontar para o sítio errado. Recusar aqui é a leitura honesta — deixar
    // passar daria N sprites com o desenho trocado, e ninguém saberia porquê.
    if (width, height) != meta.image_size {
        return fail(format!(
            "metadata says {}x{} but {} is {}x{} — re-export both",
            meta.image_size.0, meta.image_size.1, meta.image_filename, width, height
        ));
    }
    if meta.regions.is_empty() {
        return fail("the metadata declares no frames".into());
    }
    let sheet_id = *next_sheet_id;
    let sheet = ph2d_sprite_sheet::AuthoredSheet::new(
        sheet_id,
        name.clone(),
        width,
        height,
        pixels,
        meta.regions
            .iter()
            .map(|(n, r)| (n.clone(), [r.x, r.y, r.w, r.h])),
    );
    let texture_id = match renderer.acquire_individual(sheet.width, sheet.height, &sheet.rgba) {
        Ok(id) => id,
        Err(e) => return fail(format!("GPU upload: {e}")),
    };
    *next_sheet_id = next_sheet_id.saturating_add(1);
    sheet_textures.insert(sheet_id, texture_id);

    // A grade: mesma lei do import de imagens (quase-quadrada, cresce à direita e para BAIXO —
    // o eixo Y cresce para cima, então cada linha desce).
    let ppm = pixels_per_meter.max(crate::EPS_PIXELS_PER_METER);
    let n = sheet.regions.len();
    let cols = (n as f32).sqrt().ceil().max(1.0) as usize;
    let max_w = sheet
        .regions
        .iter()
        .map(|r| r.rect[2] as f32 / ppm)
        .fold(0.0_f32, f32::max);
    let max_h = sheet
        .regions
        .iter()
        .map(|r| r.rect[3] as f32 / ppm)
        .fold(0.0_f32, f32::max);
    let gap = crate::image_import::IMPORT_GRID_GAP_FRAC * max_w.max(max_h);
    let (pitch_x, pitch_y) = (max_w + gap, max_h + gap);

    let needs_clip = sheet.regions_need_filter_clip();
    let mut bits = Vec::with_capacity(n);
    for (k, region) in sheet.regions.iter().enumerate() {
        let world_size = [region.rect[2] as f32 / ppm, region.rect[3] as f32 / ppm];
        let center = ph2d_core::Vec2::new(
            anchor_world[0] + (k % cols) as f32 * pitch_x,
            anchor_world[1] - (k / cols) as f32 * pitch_y,
        );
        // O nome do sprite é o nome que o ARTISTA deu à região, não `sheet_0`: é o que ele vê no
        // Aseprite, e é por ele que ele vai procurar na hierarquia.
        let label = crate::name_unique::unique_name(sim, &region.name);
        let mut sprite = Sprite::individual(texture_id, world_size, [1.0, 1.0, 1.0, 1.0]);
        // ⚠️ MEDIDO na folha, não assumido: um `.png` do Aseprite pode vir com as regiões
        // coladas (aí o recuo defende) ou com padding (aí ele só cortaria borda).
        crate::project_sprite_pixels::bind_sheet_region(
            &mut sprite,
            texture_id,
            region.rect,
            needs_clip,
            // Um `.png` é alfa RETO — aqui a suposição é a verdade, e o `AuthoredSheet::new` (que
            // este caminho usa) já a assume.
            sheet.premultiplied,
        );
        let entity = sim
            .world_mut()
            .spawn((
                Transform::from_translation(center),
                sprite,
                Name::new(label),
                // A AUTORIA — o que sobrevive ao `texture_id` morrer com o processo.
                SpriteSheetRef {
                    sheet: sheet_id,
                    region: k as u32,
                },
            ))
            .id();
        bits.push(entity.to_bits());
    }
    sheets.insert(sheet_id, sheet);
    SheetImportResult::Ok {
        name,
        regions: n,
        bits,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_files_are_partitioned_out_of_the_image_drop() {
        let paths = vec![
            PathBuf::from("/a/hero.png"),
            PathBuf::from("/a/hero.json"),
            PathBuf::from("/a/other.PNG"),
        ];
        let (json, rest) = partition_sheet_metadata(&paths);
        assert_eq!(json, vec![PathBuf::from("/a/hero.json")]);
        assert_eq!(rest.len(), 2);
    }

    /// ⚠️ A extensão vem do sistema de arquivos do utilizador — no Windows e no macOS ela pode
    /// chegar em maiúsculas, e um `.JSON` tratado como imagem daria "Skipped non-image".
    #[test]
    fn the_extension_match_is_case_insensitive() {
        let paths = vec![PathBuf::from("/a/hero.JSON")];
        let (json, rest) = partition_sheet_metadata(&paths);
        assert_eq!(json.len(), 1);
        assert!(rest.is_empty());
    }
}
