//! **UMA lei sobre o que este app importa** — e as DUAS portas leem-na daqui.
//!
//! ⚠️ Enio, 2026-08-23: *«`.ase` não aparece no dialog de import»*. O defeito não era o `.ase`: era
//! haver **duas portas** para a mesma pergunta. O drag & drop roteava por um predicado
//! (`is_supported_image_extension`, onze extensões) e o botão **Import…** oferecia uma lista
//! **escrita à mão** com quatro. O `.ase` entrou por uma e não pela outra — mas o `.gif`, o `.psd`
//! e o `.ora` já estavam invisíveis no diálogo **há meses**, pelo mesmo mecanismo.
//!
//! > *Uma lista escrita à mão ao lado de um predicado é duas respostas à mesma pergunta, e a que o
//! > artista vê é a que envelhece.*
//!
//! ⇒ Aqui vive **a resposta**: [`partition_importables`] decide, [`dialog_filters`] enumera, e
//! [`import_paths_grid`] importa. As duas portas chamam a mesma função — a diferença entre elas
//! passa a ser só **de onde vêm os caminhos**, que é a única diferença que elas de facto têm.
//!
//! # A colocação, e porque ela é uma lei e não um acidente
//!
//! Um `.ase` vira **uma** sprite com grelha; uma imagem vira uma sprite avulsa e N imagens formam
//! uma grelha. Misturar os dois num só arranjo daria uma grelha em que uma célula é uma tira de 12
//! quadros. ⇒ **os `.ase` ocupam a primeira LINHA, um à direita do outro, e a grelha de imagens
//! começa abaixo dela** ([`images_anchor`]).

use crate::image_import::{ImportItemResult, import_images_grid};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_render::SpriteRenderer;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// O que uma leva de ficheiros produziu, pela porta que for.
pub(crate) struct ImportBatch {
    pub(crate) items: Vec<ImportItemResult>,
    /// As linhas que um formato **com autoria** (o `.ase`) produziu: o que ficou por trás, nomeado.
    pub(crate) notes: Vec<String>,
    /// Os nomes dos ficheiros que este app não sabe importar.
    pub(crate) skipped: Vec<String>,
}

/// **O que o diálogo de ficheiro OFERECE**, derivado das mesmas listas que a
/// [`partition_importables`] consulta.
///
/// A primeira linha é o «tudo», que é a que o artista quer 99% das vezes; as seguintes existem para
/// ele poder estreitar. ⚠️ **Nenhuma delas é escrita à mão** — foi assim que o `.gif` desapareceu.
#[must_use]
pub(crate) fn dialog_filters() -> Vec<(String, Vec<&'static str>)> {
    let ase = crate::ase_import::ASE_EXTENSIONS.to_vec();
    let img = ph2d_asset::SUPPORTED_IMAGE_EXTENSIONS.to_vec();
    let mut all = ase.clone();
    all.extend(img.iter().copied());
    vec![
        ("All supported".to_owned(), all),
        ("Aseprite".to_owned(), ase),
        ("Images".to_owned(), img),
    ]
}

/// Separa a leva em `.ase`, imagens e o que este app não sabe ler.
///
/// ⚠️ **A ordem dentro de cada grupo é a da leva** — o artista escolheu-a no diálogo (ou largou-a),
/// e reordenar faria a grelha sair noutra ordem sem ninguém a ter pedido.
#[must_use]
pub(crate) fn partition_importables(
    paths: &[PathBuf],
) -> (Vec<PathBuf>, Vec<PathBuf>, Vec<PathBuf>) {
    let mut ase = Vec::new();
    let mut images = Vec::new();
    let mut unknown = Vec::new();
    for p in paths {
        if crate::ase_import::is_ase_file(p) {
            ase.push(p.clone());
        } else if ph2d_asset::is_supported_image_extension(p) {
            images.push(p.clone());
        } else {
            unknown.push(p.clone());
        }
    }
    (ase, images, unknown)
}

/// **Onde a grelha de imagens começa**, depois de os `.ase` terem ocupado a primeira linha.
///
/// Sem `.ase` na leva (o caso comum) ela começa **exactamente** na âncora — nada muda para quem
/// larga só imagens. O eixo Y cresce para cima, então descer é subtrair.
#[must_use]
pub(crate) fn images_anchor(anchor: [f32; 2], ase_row_height: f32) -> [f32; 2] {
    if ase_row_height <= 0.0 {
        return anchor;
    }
    let gap = crate::image_import::IMPORT_GRID_GAP_FRAC * ase_row_height;
    [anchor[0], anchor[1] - ase_row_height - gap]
}

/// **Importa uma leva**, seja ela largada na janela ou escolhida no diálogo.
#[allow(clippy::too_many_arguments)]
pub(crate) fn import_paths_grid(
    sim: &mut crate::SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    anchor_world: [f32; 2],
    next_cell: &mut u32,
    paths: &[PathBuf],
    pixels_per_meter: f32,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) -> ImportBatch {
    let (ase, images, unknown) = partition_importables(paths);
    let mut out = ImportBatch {
        items: Vec::new(),
        notes: Vec::new(),
        skipped: unknown
            .iter()
            .map(|p| {
                p.file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("(unnamed)")
                    .to_owned()
            })
            .collect(),
    };
    // Os `.ase`, um à direita do outro. O passo sai do tamanho que a sprite de facto ficou — usar
    // um número fixo faria duas folhas de tamanhos diferentes ficarem coladas ou longe.
    let mut cursor_x = anchor_world[0];
    let mut row_h = 0.0_f32;
    for path in &ase {
        match crate::ase_import::import_ase(
            sim,
            renderer,
            asset_db,
            next_cell,
            atlas_asset_map,
            path,
            [cursor_x, anchor_world[1]],
            pixels_per_meter,
        ) {
            crate::ase_import::AseImportResult::Ok {
                name,
                frames,
                animations,
                bits,
                notes,
            } => {
                let size = sim
                    .world()
                    .get::<ph2d_render::Sprite>(ph2d_ecs::Entity::from_bits(bits))
                    .map_or([1.0, 1.0], |s| s.size);
                cursor_x += size[0] * (1.0 + crate::image_import::IMPORT_GRID_GAP_FRAC);
                row_h = row_h.max(size[1]);
                out.items.push(ImportItemResult::Ok {
                    label: format!("{name} ({frames} frames, {animations} animations)"),
                    bits,
                });
                out.notes.extend(notes);
            }
            crate::ase_import::AseImportResult::Err { name, error } => {
                out.items.push(ImportItemResult::Err { name, error });
            }
        }
    }
    if !images.is_empty() {
        out.items.extend(import_images_grid(
            sim,
            renderer,
            asset_db,
            images_anchor(anchor_world, row_h),
            next_cell,
            &images,
            pixels_per_meter,
            atlas_asset_map,
        ));
    }
    out
}

#[cfg(test)]
#[path = "import_router_tests.rs"]
mod tests;
