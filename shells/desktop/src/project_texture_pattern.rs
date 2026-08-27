//! **A ARTE DOS PADRÕES dentro do ficheiro de projecto** (plano 33, W4) — irmão do
//! [`crate::project_sprite_pixels`], e pela mesma razão que ele existe.
//!
//! # O que quebraria sem isto
//!
//! O documento guarda **qual** imagem um padrão usa (um `AssetId`), nunca os pixels. Reabrir o
//! projecto noutra sessão — ou noutra máquina — encontraria o `AssetDb` vazio, a fonte não
//! resolveria, e toda forma com padrão pintaria a `fallback` **para sempre e sem erro nenhum a que
//! agarrar**. É literalmente o defeito que o `project_sprite_pixels` curou para as sprites.
//!
//! # A identidade é o CONTEÚDO, e ela tem de SOBREVIVER
//!
//! ⚠️⚠️ **É por isto que a autoria usa `insert_image_rgba8` e não `insert_image_bytes`.** O
//! primeiro cunha `blake3(dims + RGBA)`; o segundo, `blake3(bytes do ficheiro)`. Aqui embutimos
//! **pixels** (o ficheiro do artista pode ter mudado de sítio, ou nunca mais existir), então o
//! restore re-insere RGBA — e só o id dos pixels volta **igual**. Com o outro, o round-trip daria
//! um id novo e a fonte do documento apontaria para o nada.
//!
//! O gate `a_saved_pattern_reopens_with_the_same_asset_id` prende exactamente isso.
//!
//! # Dois padrões com a mesma arte custam UMA entrada
//!
//! A colheita é chaveada pelo `AssetId` (um `BTreeMap`), então a arte partilhada não se duplica no
//! ficheiro — a mesma propriedade que o `project_sprite_pixels` documenta.

use ph2d_asset::AssetId;
use ph2d_vec_scene::{Paint, PatternSource};
use std::collections::BTreeMap;

/// A versão do blob. ⚠️ Ele carrega a própria, como o `timeline` e o `sculpt` — é isso que faz um
/// campo novo AQUI dentro não voltar a bumpar o `PROJECT_SCHEMA`.
const PATTERN_ART_DOC_VERSION: u32 = 1;

/// A arte de um padrão embutida no projecto.
#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct SavedPatternArt {
    /// O `AssetId` que o documento nomeia — e que o restore tem de reproduzir.
    pub(crate) id: AssetId,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rgba: Vec<u8>,
}

impl crate::App {
    /// Colhe a arte de cada padrão da cena, para embutir no ficheiro.
    ///
    /// Vazio quando não há padrão nenhum — um projecto sem padrões não paga byte nenhum.
    pub(super) fn collect_texture_pattern_art(&self, scene: &ph2d_vec_scene::VecScene) -> Vec<u8> {
        let Some(gfx) = self.gfx.as_ref() else {
            return Vec::new();
        };
        let mut by_id: BTreeMap<AssetId, SavedPatternArt> = BTreeMap::new();
        for path in scene.paths() {
            let Some(Paint::Pattern(p)) = path.fill.as_ref() else {
                continue;
            };
            // ⏳ Uma fonte-FORMA (W7) não tem arte a embutir: ela É o documento, e o documento já
            // viaja. Só a imagem precisa dos pixels.
            let PatternSource::Image(id) = p.source else {
                continue;
            };
            if by_id.contains_key(&id) {
                continue;
            }
            let Some(asset) = gfx.asset_db.get(&id) else {
                continue;
            };
            let Some((width, height, rgba)) = asset.image_rgba8() else {
                continue;
            };
            by_id.insert(
                id,
                SavedPatternArt {
                    id,
                    width,
                    height,
                    rgba: rgba.into_owned(),
                },
            );
        }
        if by_id.is_empty() {
            return Vec::new();
        }
        let arts: Vec<SavedPatternArt> = by_id.into_values().collect();
        postcard::to_allocvec(&(PATTERN_ART_DOC_VERSION, arts)).unwrap_or_default()
    }

    /// Devolve a arte ao `AssetDb`, **sob o mesmo id**.
    pub(super) fn restore_texture_pattern_art(&mut self, blob: &[u8]) {
        if blob.is_empty() {
            return;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Ok((ver, arts)) = postcard::from_bytes::<(u32, Vec<SavedPatternArt>)>(blob) else {
            eprintln!("[proj] arte de padrao: blob ilegivel, ignorado");
            return;
        };
        if ver != PATTERN_ART_DOC_VERSION {
            eprintln!("[proj] arte de padrao: versao {ver} desconhecida, ignorada");
            return;
        }
        for a in arts {
            let back = gfx
                .asset_db
                .insert_image_rgba8(a.width, a.height, a.rgba.clone());
            // ⚠️ **Em voz alta.** Um id que não volta igual é uma fonte que nunca mais resolve, e o
            // sintoma seria *"o meu padrão virou uma cor chapada"* sem nada no log.
            if back != a.id {
                eprintln!(
                    "[proj] arte de padrao: o id nao voltou igual ({} != {}) - o padrao vai pintar \
                     a cor de recurso",
                    back.to_hex(),
                    a.id.to_hex()
                );
            }
        }
    }
}

#[cfg(test)]
#[path = "project_texture_pattern_tests.rs"]
mod tests;
