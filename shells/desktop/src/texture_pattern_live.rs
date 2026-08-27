//! **O ASSADO dos padrões de textura da cena** (plano 33, W4) — memoizado, irmão do
//! [`crate::fx_live`].
//!
//! ⚠️⚠️ **O NOME É `texture_pattern_live`, e o `pattern_live` ao lado é OUTRA COISA.** Aquele é o
//! *Pattern Along Path* ([plano 23](../../../docs/Vector%20Module/23_plano_pattern_along_path.md)):
//! um MOTIVO copiado ao longo de uma guia, com alças e picker. Este é a TINTA de uma forma. A
//! primeira redacção desta wave chamou-se `pattern_live` e o módulo passou a estar **declarado
//! duas vezes** — *o nome de um módulo carrega um contrato, e este já tinha dono*.
//!
//! O documento guarda a RELAÇÃO (qual arte, que reticulado, que tamanho, onde); isto é o desenho
//! derivado dela, e vive só em runtime.
//!
//! # Porque há um memo, e o que está na chave
//!
//! ⚠️ **Não é o quadro que custa — é o ASSADO.** Desenhar um padrão é uma `fill()` (a repetição é
//! do amostrador do Vello); assar é compor a arte num reticulado. Medido na W1: `1,047 ms` para um
//! ladrilho de `536x1072` em colmeia. Fazê-lo por quadro seria 6% de um quadro de 60 fps por forma
//! com padrão, por nada.
//!
//! A chave é **`(fonte, lei assada, dimensões da arte)`** — exactamente o que muda os pixels do
//! ladrilho, e nada mais. ⛔ **A `quality` NÃO entra nela**, e a ausência é a decisão: ela escolhe o
//! filtro de amostragem na GPU e não toca um byte do assado, então metê-la na chave faria alternar
//! o modo de imagem do projecto re-assar toda a cena para produzir os MESMOS pixels. Ela é
//! actualizada em cada quadro sobre a entrada que já existe.
//!
//! ⚠️ E o [`StableImage`] é construído **UMA vez** e clonado por quadro, pela razão que o
//! `FxImage` já documenta: o Vello indexa o cache de imagem pelo id do `Blob`, então um handle novo
//! por quadro faz a textura ser **re-enviada ao atlas** todo quadro.

use ph2d_asset::AssetDb;
use ph2d_vec_pattern::TileLaw;
use ph2d_vec_render::{PatternTile, PatternTiles};
use ph2d_vec_scene::{Paint, PatternSource, VecPathId, VecScene};
use ph2d_vector::{ImageQuality, StableImage};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Com o que estes pixels foram assados. Ver o cabeçalho para o que NÃO está aqui.
#[derive(Clone, PartialEq, Eq)]
struct Key {
    source: PatternSource,
    law: TileLaw,
    art: [u32; 2],
}

/// Os ladrilhos de padrão da cena, assados e memoizados.
#[derive(Default)]
pub(crate) struct TexturePatternLive {
    tiles: PatternTiles,
    keys: BTreeMap<VecPathId, Key>,
}

impl TexturePatternLive {
    /// Os ladrilhos deste quadro — o que o [`ph2d_vec_render::dispatch`] injecta no z das formas.
    ///
    /// Vazio = nenhum padrão resolvido, e toda forma com `Paint::Pattern` pinta a `fallback` dela.
    pub(crate) fn tiles(&self) -> &PatternTiles {
        &self.tiles
    }

    /// Re-assa o que mudou. Uma passagem pela cena; as formas sem padrão não pagam nada.
    pub(crate) fn recook(&mut self, scene: &VecScene, assets: &AssetDb, quality: ImageQuality) {
        let mut seen = BTreeSet::new();
        for path in scene.paths() {
            let Some(Paint::Pattern(pat)) = path.fill.as_ref() else {
                continue;
            };
            let Some((aw, ah, px)) = art_of(&pat.source, assets) else {
                // A arte ainda não carregou (ou a fonte é uma FORMA, que a W7 resolve): a entrada
                // fica de fora e a forma pinta a `fallback` — desenho certo, não desistência.
                continue;
            };
            let key = Key {
                source: pat.source,
                law: pat.law([aw, ah]),
                art: [aw, ah],
            };
            seen.insert(path.id);
            if self.keys.get(&path.id) == Some(&key) {
                // ⭐ Só o filtro se actualiza: ele não muda um byte do assado.
                if let Some(t) = self.tiles.get_mut(&path.id) {
                    t.quality = quality;
                }
                continue;
            }
            let tile = match ph2d_vec_pattern::bake(&px, aw, ah, &key.law) {
                Ok(t) => t,
                Err(e) => {
                    // ⚠️ **EM VOZ ALTA, e uma vez por lei nova** (o memo só chega aqui quando a
                    // chave muda). O report do Enio de 2026-08-27 — *"em column o pattern some"* —
                    // era exactamente esta recusa, calada: a forma voltava à cor de recurso e nada
                    // dizia porquê. Hoje o assador **reduz** em vez de recusar, e chegar aqui passou
                    // a ser um caso que um `offset_denom: u8` não consegue produzir.
                    eprintln!(
                        "[pattern] o assado da forma {} recusou ({e:?}) - ela vai pintar a cor de \
                         recurso",
                        path.id
                    );
                    self.tiles.remove(&path.id);
                    self.keys.remove(&path.id);
                    continue;
                }
            };
            let Some(image) = StableImage::from_rgba(Arc::new(tile.rgba), tile.width, tile.height)
            else {
                self.tiles.remove(&path.id);
                self.keys.remove(&path.id);
                continue;
            };
            self.tiles.insert(
                path.id,
                PatternTile {
                    image,
                    cells: tile.cells,
                    tile_px: [tile.width, tile.height],
                    quality,
                },
            );
            self.keys.insert(path.id, key);
        }
        // ⚠️ **A varredura tem as DUAS metades.** Marcar sem desmarcar deixaria o ladrilho de uma
        // forma que deixou de ter padrão (ou que foi apagada) a ser desenhado para sempre, e a
        // memória dele viva — é a mesma lei das duas metades do passe do `MasterPiece`.
        self.tiles.retain(|id, _| seen.contains(id));
        self.keys.retain(|id, _| seen.contains(id));
    }
}

/// Os pixels da ARTE de um padrão, RGBA reto.
///
/// ⚠️ Passa pela porta [`ph2d_asset::Asset::image_rgba8`] em vez de casar com `ImageRgba8`: ela
/// converte o caso de 16 bits, e um `match` directo aceitaria a variante de 16 bits **em silêncio**
/// pelo braço `_` (o `Asset` é `#[non_exhaustive]`, e o doc dele avisa exactamente disto).
fn art_of(source: &PatternSource, assets: &AssetDb) -> Option<(u32, u32, Vec<u8>)> {
    match source {
        PatternSource::Image(id) => {
            let asset = assets.get(id)?;
            let (w, h, px) = asset.image_rgba8()?;
            Some((w, h, px.into_owned()))
        }
        // ⏳ **W7**: uma FORMA do documento como fonte (o modelo do Figma). Enquanto ela não existe,
        // um padrão com fonte-forma pinta a `fallback` — visível e explicável, ao contrário de
        // invisível.
        PatternSource::Shape(_) => None,
    }
}

#[cfg(test)]
#[path = "texture_pattern_live_tests.rs"]
mod tests;
