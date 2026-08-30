//! ⭐⭐ **O que os verbos de catálogo FAZEM** (plano 07, wave A3) — irmão do
//! [`crate::asset_card_verbs`], e pela mesma divisão: o painel transporta o pedido, o shell decide
//! e **fala**.
//!
//! # ⚠️ O nome de um catálogo novo é GERADO, e único
//!
//! Criar e nomear são dois gestos. O primeiro tem de produzir algo que já existe e se vê — senão o
//! artista carrega no `+` e não acontece nada visível até ele escrever. ⇒ o catálogo nasce
//! *«Catalog»*, *«Catalog 2»*, … e o gesto seguinte renomeia. É o que o Finder e o Blender fazem.
//!
//! ⚠️ **A unicidade é por CAMINHO e é obrigatória**, não cosmética: o modelo trata dois caminhos
//! iguais como o **mesmo** catálogo (`create` é idempotente), então sem o sufixo o segundo `+`
//! devolveria o primeiro e o botão pareceria morto.

use ph2d_asset_index::{CatalogId, CatalogTree};
use ph2d_editor::Toast;
use ph2d_editor::action_bus::CatalogVerb;
use ph2d_editor::interaction::drag_payload::DragPayload;

/// O nome com que um catálogo nasce.
const BASE: &str = "Catalog";

/// Um caminho livre dentro de `parent`.
fn free_path(tree: &CatalogTree, parent: Option<CatalogId>) -> String {
    let prefix = parent
        .and_then(|p| tree.get(p))
        .map_or_else(String::new, |c| format!("{}/", c.path));
    for n in 1..1000 {
        let label = if n == 1 {
            BASE.to_string()
        } else {
            format!("{BASE} {n}")
        };
        let path = format!("{prefix}{label}");
        if !tree.catalogs().iter().any(|c| c.path == path) {
            return path;
        }
    }
    format!("{prefix}{BASE}")
}

/// ⭐ **O dreno.** Devolve `true` quando a taxonomia mudou.
///
/// ⚠️ **Ela é do PROJECTO e não do `ProjectState`**, então isto **não** produz um passo de undo —
/// a dívida está declarada no `project_catalogs`. O que devolver `true` compra é o título a marcar
/// o projecto como sujo, que é o que faz o Ctrl+S ter o que gravar.
pub(crate) fn drain(
    verb: &CatalogVerb,
    tree: &mut CatalogTree,
    toasts: &mut ph2d_editor::ToastQueue,
) -> bool {
    match verb {
        CatalogVerb::New { parent } => {
            let path = free_path(tree, parent.map(CatalogId));
            let id = tree.create(&path);
            let label = tree.get(id).map_or(path.clone(), |c| c.label().to_string());
            toasts.push(Toast::success(format!(
                "Catalog \u{201c}{label}\u{201d} created"
            )));
            true
        }
        CatalogVerb::Rename { id, name } => {
            if tree.rename(CatalogId(*id), name) {
                true
            } else {
                // ⚠️ **A recusa NOMEIA a razão.** O modelo recusa um nome vazio ou com separador —
                // este último seria *mover* escondido dentro de *renomear* —, e um silêncio aqui
                // deixaria o artista a olhar para o nome antigo sem saber porquê.
                toasts.push(Toast::warning(
                    "A catalog name cannot be empty or contain \u{201c}/\u{201d}",
                ));
                false
            }
        }
        CatalogVerb::Delete { id } => {
            let label = tree
                .get(CatalogId(*id))
                .map(|c| c.label().to_string())
                .unwrap_or_default();
            if label.is_empty() {
                return false;
            }
            tree.delete(CatalogId(*id));
            // ⚠️ **A voz diz o que NÃO aconteceu**, porque é isso que o artista teme: apagar uma
            // gaveta não apaga o que estava lá dentro.
            toasts.push(Toast::warning(format!(
                "Catalog \u{201c}{label}\u{201d} deleted \u{2014} the assets in it were not"
            )));
            true
        }
        CatalogVerb::Assign { asset, catalog } => {
            let key = match asset {
                DragPayload::Prefab { stable_id } => ph2d_asset_index::AssetRef::Component {
                    stable_id: *stable_id,
                },
                DragPayload::Image { asset } => {
                    ph2d_asset_index::AssetRef::Texture { asset: *asset }
                }
            };
            match catalog {
                Some(c) => {
                    let Some(name) = tree.get(CatalogId(*c)).map(|c| c.label().to_string()) else {
                        return false;
                    };
                    tree.assign(key, CatalogId(*c));
                    toasts.push(Toast::success(format!("Moved to \u{201c}{name}\u{201d}")));
                }
                None => {
                    tree.unassign(&key);
                    toasts.push(Toast::info("Removed from its catalog"));
                }
            }
            true
        }
    }
}

#[cfg(test)]
#[path = "asset_catalog_verbs_tests.rs"]
mod tests;
