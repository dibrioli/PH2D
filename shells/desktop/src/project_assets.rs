//! **Os PIXELS que o undo não guarda** — irmão de [`crate::project`] pelo teto de LOC
//! (HR-18), e o corte é por responsabilidade: o `project.rs` diz *o que um arquivo de
//! projeto É* e *quando ele é escrito*; aqui vive *como os pixels saem do atlas e como
//! voltam para ele*. É a única metade do save que conversa com o `AssetDb` e com o
//! empacotador — o resto do módulo nunca precisa saber que existe uma célula de atlas.
//!
//! O `SavedAsset` fica no PAI de propósito: ele é campo do `ProjectFile`, logo é parte da
//! FORMA do arquivo, não do gesto de colhê-lo. (Um filho enxerga os privados do pai, então
//! o corte não move visibilidade nenhuma.)

use super::SavedAsset;

impl crate::App {
    /// Coleta os pixels de cada imagem importada, para embutir no arquivo.
    ///
    /// A lista canônica é o `atlas_asset_map` (`key → AssetId`) — o `AssetDb` não
    /// itera. Cobre só `SpriteSource::Atlas` (o caminho comum de import); `Individual`
    /// (Painter/Apply) e `CookedTexture` (KTX2) ficam fora do 1º corte.
    pub(super) fn collect_assets(&self) -> Vec<SavedAsset> {
        let Some(gfx) = self.gfx.as_ref() else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for (&key, asset_id) in &gfx.atlas_asset_map {
            // PRECISION-BYPASS: caminho de ESCRITA — não passa pela porta `Asset::image_rgba8`, e
            // é deliberado (plano `docs/Sprite_projeto/18`, auditoria da W2).
            //
            // ⚠️ **CORRIGIDO na W3:** a nota anterior dizia que este sítio *"tem de ganhar um
            // irmão de 16 bits"*. **Não tem, e a razão é estrutural:** este laço percorre o
            // `atlas_asset_map`, e o atlas é de 8 bits por construção — uma textura, um formato.
            // Uma sprite de 16 bits é obrigatoriamente `Individual` (§3.3), e os pixels dela
            // gravam-se pelo `project_sprite_pixels.rs`, que **já** tem o ramo.
            //
            // ⛔ **A regra de que 16 bits ⇒ `Individual` é o que torna esta linha verdadeira**, e
            // ela é imposta onde a precisão nasce: na conversão (W5) e na importação (W2.4). Se
            // alguém alguma vez deixar um asset de 16 bits entrar no `atlas_asset_map`, é ali que
            // se corrige — **não** acrescentando um ramo aqui, que só gravaria o rebaixamento com
            // mais passos.
            //
            // O `_ => ()` implícito continua a existir para prefabs e texturas cozidas, que nunca
            // foram pixels de atlas.
            if let Some(asset) = gfx.asset_db.get(asset_id)
                && let ph2d_asset::Asset::ImageRgba8 {
                    width,
                    height,
                    pixels,
                } = &*asset
            {
                out.push(SavedAsset {
                    key,
                    width: *width,
                    height: *height,
                    rgba: pixels.to_vec(),
                });
            }
        }
        out
    }

    /// Re-insere os pixels no `AssetDb` e re-empacota o atlas **nas mesmas células**
    /// (`key`) que os sprites restaurados referenciam. O `key` é caller-supplied, então
    /// os `Sprite.source = Atlas { key }` do WorldSnapshot resolvem de novo.
    pub(super) fn materialize_assets(&mut self, assets: &[SavedAsset]) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // Campos irmãos de `AppGfx` — refs disjuntas para o closure `fetch` (só lê o
        // mapa/db) coexistir com o `&mut renderer`.
        let renderer = &mut gfx.renderer;
        let asset_db = &gfx.asset_db;
        let atlas_asset_map = &mut gfx.atlas_asset_map;

        // 1. Pixels no AssetDb + o vínculo key→AssetId, ANTES dos inserts (um regrow
        //    disparado no meio precisa ver todos os keys já mapeados).
        for a in assets {
            let id = asset_db.insert_image_rgba8(a.width, a.height, a.rgba.clone());
            atlas_asset_map.insert(a.key, id);
        }
        // 2. Empacota cada um no atlas (upload GPU + mips internos). O `fetch` é o
        //    mesmo de `image_import::pack_image`: re-materializa as regiões num regrow.
        for a in assets {
            let fetch = |key: u32| -> Option<Vec<u8>> {
                let aid = atlas_asset_map.get(&key)?;
                // ⚠️ `image_rgba8`: o atlas é de 8 bits por construção, e o `match` na variante
                // devolvia `None` para 16 bits — regrow com célula vazia (plano 18, auditoria W2).
                let asset = asset_db.get(aid)?;
                asset.image_rgba8().map(|(_, _, px)| px.into_owned())
            };
            if let Err(e) =
                renderer.insert_atlas_sprite_with_regrow(a.key, a.width, a.height, &a.rgba, fetch)
            {
                eprintln!("[proj] atlas insert key={}: {e}", a.key);
            }
        }
        // 3. Imports futuros nesta sessão não podem colidir com os keys do projeto.
        if let Some(max_key) = assets.iter().map(|a| a.key).max() {
            gfx.next_import_cell = gfx.next_import_cell.max(max_key.saturating_add(1));
        }
    }
}
