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
use ph2d_vec_render::{PatternTile, PatternTiles};
use ph2d_vec_scene::{Paint, PatternSource, VecPath, VecPathId, VecScene};
use ph2d_vector::{ImageQuality, StableImage};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Com o que estes pixels foram assados. Ver o cabeçalho para o que NÃO está aqui.
///
/// ⚠️⚠️ **Ela é feita SEM tocar na arte, e isso é o assunto.** A 1.ª versão punha aqui as dimensões
/// em pixels e a lei já convertida — e para as saber tinha de **resolver a arte primeiro**, o que
/// com uma fonte-FORMA significa um render + readback de GPU **por quadro**. Um gate apanhou-o
/// (`editing_the_source_shape_rebakes_the_tile`, a metade do *"não re-assou o que não mudou"*).
///
/// ⇒ a chave guarda o que o **artista autorou** mais a **identidade/conteúdo da fonte**, e as
/// dimensões em pixels são derivadas disso. Uma imagem é endereçada por conteúdo (HR-6), então o
/// mesmo `AssetId` são os mesmos pixels; uma forma entra pelo conteúdo dela.
#[derive(Clone, PartialEq)]
struct Key {
    source: PatternSource,
    kind: ph2d_vec_pattern::TileKind,
    offset_denom: u8,
    /// ⚠️ O `size` entra porque o vão em PIXELS deriva de `gap/size` — mexer no tamanho muda o
    /// assado. Já o `origin` e o `angle` são COLOCAÇÃO: eles não tocam um byte do ladrilho, e
    /// metê-los aqui faria arrastar a alça de mover re-assar a cena.
    size: [f64; 2],
    gap: [f64; 2],
    /// ⭐⭐ **A FORMA-FONTE, quando a arte vem do documento** (W7) — e é ela que torna o padrão
    /// **VIVO**: editar a forma re-assa o ladrilho em toda forma que a usa, que é o *"pattern fills
    /// are dynamic"* do Figma.
    ///
    /// ⚠️ Sem ela na chave, a `PatternSource::Shape(id)` seria estável e mexer nos nós da fonte não
    /// mudaria a tela — o defeito EXACTO que o `FxKey` da crate irmã documenta (*"a chave era
    /// `(pilha, w, h)` e a forma não entrava nela, então mudar a cor do fill de uma forma filtrada
    /// não mudava a tela"*).
    shape: Option<VecPath>,
}

/// A ARTE que uma fonte nomeia, no que ela tem de identidade — a metade PURA da resolução.
///
/// ⚠️ Extraída da que assa porque assar precisa de GPU e **isto não**: é aqui que vive a recusa do
/// ciclo, e ela tem gate.
#[must_use]
fn source_shape(scene: &VecScene, host: VecPathId, source: &PatternSource) -> Option<VecPath> {
    let PatternSource::Shape(id) = source else {
        return None;
    };
    // ⛔⛔ **UMA FORMA NÃO PODE SER O PRÓPRIO PADRÃO.** Assá-la exigiria desenhá-la, desenhá-la
    // exigiria o ladrilho, e o ladrilho exigiria assá-la. ⚠️ E o sintoma não seria um erro: seria o
    // app a parar, ou um ladrilho de uma versão anterior de si mesmo a cada quadro.
    if *id == host {
        return None;
    }
    scene.paths().iter().find(|p| p.id == *id).cloned()
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
    ///
    /// ⭐⭐ **O assador de FORMA vem INJECTADO** (`bake_shape`), e não cablado.
    ///
    /// Assar uma forma em pixels é render + readback — GPU. Cablá-lo aqui poria uma `GpuContext` na
    /// assinatura e tornaria **todo** gate deste memo dependente de uma placa; com a injecção, o
    /// quadro passa a porta única [`crate::motion_object_bake::bake_rgba`] e os gates passam um
    /// bitmap sintético. *Um memo que só se pode medir com GPU é um memo que não se mede.*
    pub(crate) fn recook(
        &mut self,
        scene: &VecScene,
        assets: &AssetDb,
        quality: ImageQuality,
        bake_shape: &mut dyn FnMut(VecPathId) -> Option<(u32, u32, Vec<u8>)>,
    ) {
        let mut seen = BTreeSet::new();
        for path in scene.paths() {
            let Some(Paint::Pattern(pat)) = path.fill.as_ref() else {
                continue;
            };
            let shape = source_shape(scene, path.id, &pat.source);
            // ⚠️ Uma fonte-FORMA que não resolve (inexistente, ou a própria forma) nunca chega ao
            // assador: a recusa é PURA e mora na `source_shape`.
            if matches!(pat.source, PatternSource::Shape(_)) && shape.is_none() {
                continue;
            }
            let key = Key {
                source: pat.source,
                kind: pat.kind,
                offset_denom: pat.offset_denom,
                size: pat.size,
                gap: pat.gap,
                shape,
            };
            seen.insert(path.id);
            if self.keys.get(&path.id) == Some(&key) {
                // ⭐ Só o filtro se actualiza: ele não muda um byte do assado. E — mais importante —
                // **não se resolve a arte**: com uma fonte-FORMA isso seria um readback de GPU por
                // quadro.
                if let Some(t) = self.tiles.get_mut(&path.id) {
                    t.quality = quality;
                }
                continue;
            }
            let Some((aw, ah, px)) = art_of(&pat.source, assets, key.shape.is_some(), bake_shape)
            else {
                // A arte ainda não carregou: a entrada fica de fora e a forma pinta a `fallback` —
                // desenho certo, não desistência. ⚠️ E a chave NÃO se grava, senão o próximo quadro
                // acharia que já estava assado.
                self.tiles.remove(&path.id);
                self.keys.remove(&path.id);
                continue;
            };
            let law = pat.law([aw, ah]);
            let tile = match ph2d_vec_pattern::bake(&px, aw, ah, &law) {
                Ok(t) => t,
                Err(e) => {
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
fn art_of(
    source: &PatternSource,
    assets: &AssetDb,
    shape_ok: bool,
    bake_shape: &mut dyn FnMut(VecPathId) -> Option<(u32, u32, Vec<u8>)>,
) -> Option<(u32, u32, Vec<u8>)> {
    match source {
        PatternSource::Image(id) => {
            let asset = assets.get(id)?;
            let (w, h, px) = asset.image_rgba8()?;
            Some((w, h, px.into_owned()))
        }
        // ⭐⭐ **W7 — uma FORMA do documento como arte** (o modelo do Figma). O `shape_ok` é o
        // veredito da [`source_shape`]: ela é que diz se a fonte existe e **não é a própria forma**,
        // e é lá que a recusa do ciclo vive — pura, e com gate.
        PatternSource::Shape(id) => {
            if !shape_ok {
                return None;
            }
            bake_shape(*id)
        }
    }
}

#[cfg(test)]
#[path = "texture_pattern_live_tests.rs"]
mod tests;
