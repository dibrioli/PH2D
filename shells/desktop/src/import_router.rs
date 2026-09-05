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
    let svg = crate::svg_import::SVG_EXTENSIONS.to_vec();
    let img = ph2d_asset::SUPPORTED_IMAGE_EXTENSIONS.to_vec();
    let mut all = ase.clone();
    all.extend(svg.iter().copied());
    all.extend(img.iter().copied());
    vec![
        ("All supported".to_owned(), all),
        ("Aseprite".to_owned(), ase),
        ("Vector (SVG)".to_owned(), svg),
        ("Images".to_owned(), img),
    ]
}

/// O que uma leva contém, por espécie.
///
/// ⚠️ **A ordem dentro de cada grupo é a da leva** — o artista escolheu-a no diálogo (ou largou-a),
/// e reordenar faria a grelha sair noutra ordem sem ninguém a ter pedido.
#[derive(Debug, Default)]
pub(crate) struct Importables {
    pub(crate) ase: Vec<PathBuf>,
    /// ⭐ Desenhos vectoriais (estudo 42, item 3). ⛔ **Espécie própria, e não uma imagem**: um
    /// `.svg` que entrasse pela grelha de imagens viraria uma sprite de pixels, que é exactamente
    /// o contrário do que ele é.
    pub(crate) svg: Vec<PathBuf>,
    pub(crate) images: Vec<PathBuf>,
    pub(crate) unknown: Vec<PathBuf>,
}

/// Separa a leva em `.ase`, `.svg`, imagens e o que este app não sabe ler.
#[must_use]
pub(crate) fn partition_importables(paths: &[PathBuf]) -> Importables {
    let mut out = Importables::default();
    for p in paths {
        if crate::ase_import::is_ase_file(p) {
            out.ase.push(p.clone());
        } else if crate::svg_import::is_svg_file(p) {
            out.svg.push(p.clone());
        } else if ph2d_asset::is_supported_image_extension(p) {
            out.images.push(p.clone());
        } else {
            out.unknown.push(p.clone());
        }
    }
    out
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

/// O documento VECTORIAL e a ponte dele, que só o ramo do `.svg` toca.
///
/// ⚠️ Um `struct` e não dois parâmetros soltos: os dois viajam sempre juntos (o `sync` precisa dos
/// dois) e esta função já carrega oito argumentos — *dois nomes que nunca se separam são um nome*.
pub(crate) struct VecTarget<'a> {
    pub(crate) scene: &'a mut ph2d_vec_scene::VecScene,
    pub(crate) map: &'a mut crate::vec_entities::VecEntityMap,
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
    vec: VecTarget<'_>,
) -> ImportBatch {
    let Importables {
        ase,
        svg,
        images,
        unknown,
    } = partition_importables(paths);
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
    // ⭐ Os DESENHOS partilham a primeira fileira com os `.ase`, e pela mesma razão: os dois são
    // ficheiros COM AUTORIA (um traz a grelha e as animações, o outro a árvore de grupos), e um
    // deles no meio da grelha de imagens faria uma célula ter o tamanho de um logótipo inteiro.
    for path in &svg {
        match crate::svg_import::import_svg(
            sim,
            vec.scene,
            vec.map,
            path,
            [cursor_x, anchor_world[1]],
            pixels_per_meter,
        ) {
            crate::svg_import::SvgImportResult::Ok {
                name,
                shapes,
                bits,
                size,
                notes,
            } => {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "tamanho de mundo: f64 do documento para o f32 da cena"
                )]
                let (w, h) = (size[0] as f32, size[1] as f32);
                cursor_x += w * (1.0 + crate::image_import::IMPORT_GRID_GAP_FRAC);
                row_h = row_h.max(h);
                out.items.push(ImportItemResult::Ok {
                    label: format!("{name} ({shapes} shapes)"),
                    bits,
                });
                out.notes.extend(notes);
            }
            crate::svg_import::SvgImportResult::Err { name, error } => {
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
