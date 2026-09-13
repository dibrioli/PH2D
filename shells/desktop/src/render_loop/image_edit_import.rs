//! O pedido de IMPORTAR do [`super::dispatch`]: abre o diálogo e importa pela porta única do
//! `import_router`. Filho por ASSUNTO (por `#[path]`) — as edições de imagem ficam no pai, e esta função
//! corre no sítio exacto onde o bloco estava (depois do dreno do undo).

use crate::image_import::ImportItemResult;
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::SimWorld;
use ph2d_editor_core::{HeroScreen, Toast, ToastQueue};
use ph2d_render::{Camera2d, SpriteRenderer};
use std::collections::BTreeMap;

/// Abre o diálogo de importação que o menu pediu e importa o que o artista escolheu, em grelha;
/// devolve se o título ficou sujo.
#[allow(clippy::too_many_arguments)]
pub(super) fn drain_import(
    hero: &mut HeroScreen,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
    toasts: &mut ToastQueue,
    camera: &Camera2d,
    next_import_cell: &mut u32,
    vec_scene: &mut ph2d_vec_scene::VecScene,
    vec_entities: &mut ph2d_vec_entities::entities::VecEntityMap,
) -> bool {
    let mut title_dirty = false;
    // M14.4c: drain pending import request → open native file picker,
    // import every selected image (PNG/WEBP/JPEG). The batch importer
    // lays them out in a near-square grid anchored at the camera center
    // (first cell's center = `camera.center`; grid grows right + down)
    // instead of stacking every sprite on one point.
    if hero.import_requested {
        hero.import_requested = false;
        // ⚠️ **O filtro é DERIVADO, nunca escrito à mão** (`crate::import_router`, Enio
        // 2026-08-23: *«.ase não aparece no dialog de import»*). A lista que morava aqui tinha
        // quatro extensões e o roteamento do drop aceitava **onze** — o `.gif`, o `.psd` e o
        // `.ora` estavam invisíveis neste diálogo há meses, pelo mesmo mecanismo que escondeu o
        // `.ase`. *Uma lista escrita à mão ao lado de um predicado é duas respostas à mesma
        // pergunta, e a que o artista vê é a que envelhece.*
        let mut dialog = rfd::FileDialog::new();
        for (label, exts) in crate::import_router::dialog_filters() {
            dialog = dialog.add_filter(label, &exts);
        }
        let picked = dialog.pick_files();
        let pixels_per_meter = hero.project.pixels_per_meter;
        if let Some(paths) = picked {
            // A MESMA função que o drag & drop chama: a única diferença entre as duas portas é de
            // onde vêm os caminhos.
            let batch = crate::import_router::import_paths_grid(
                sim,
                &mut *renderer,
                asset_db,
                camera.center,
                next_import_cell,
                &paths,
                pixels_per_meter,
                atlas_asset_map,
                crate::import_router::VecTarget {
                    scene: vec_scene,
                    map: vec_entities,
                },
            );
            for name in &batch.skipped {
                toasts.push(Toast::warning(format!(
                    "Skipped {name}: not an image, an SVG drawing or an Aseprite file"
                )));
                title_dirty = true;
            }
            let results = batch.items;
            // First imported sprite replaces the selection; the rest
            // join it as extras so a multi-pick import ends up fully
            // selected (mirrors the drag-drop path). The per-frame
            // snapshot sync turns this into both the canvas gizmo and
            // the Hierarchy highlight.
            let mut selected_any = false;
            for r in results {
                match r {
                    ImportItemResult::Ok { label, bits } => {
                        if selected_any {
                            hero.gizmo.add_to_selection(bits);
                        } else {
                            hero.gizmo.replace_selection(Some(bits));
                            selected_any = true;
                        }
                        toasts.push(Toast::success(format!("Imported {label}")));
                        title_dirty = true;
                    }
                    ImportItemResult::Err { name, error } => {
                        eprintln!("M14.4c import failed ({name}): {error}");
                        toasts.push(Toast::error(format!("Import failed: {error}")));
                        title_dirty = true;
                    }
                }
            }
            // ⚠️ As notas do `.ase` falam por ÚLTIMO — elas dizem o que ficou por trás, e uma
            // linha dessas escondida entre dez «Imported» não é lida.
            for note in batch.notes {
                toasts.push(Toast::warning(note));
            }
        }
    }
    title_dirty
}
