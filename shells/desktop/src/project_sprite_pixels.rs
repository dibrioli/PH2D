//! **OS PIXELS PRÓPRIOS de um sprite** dentro do arquivo de projeto (irmão de
//! [`crate::project`], no espírito exato de [`crate::project_painter`]).
//!
//! ## O que estava quebrado
//!
//! O projeto salvava os pixels dos sprites *importados* (o atlas, via
//! [`crate::project_assets`]) e — desde a v3 e a W8.7 — os documentos do Painter e os canais do
//! bake 3D. Tudo o resto ficava de fora. Um sprite tocado por **qualquer** ferramenta de imagem
//! (trim · bgremoval · make-square · padding · upscale · rasterize · equalize) é
//! `SpriteSource::Individual { texture_id }`, e esse `texture_id` é um id de **alocação da GPU**:
//! o `IndividualTextureStore` recomeça a numerar em `1` a cada processo. Recortar, `Ctrl+S`,
//! reabrir devolvia o sprite **invisível** (o `bind_group` não resolve e o run é pulado) — ou, se
//! outro restore tivesse tomado aquele id, **a exibir os pixels de outro sprite**.
//!
//! ## Por que este módulo não é um terceiro remédio
//!
//! O Painter e o bake 3D resolveram isto **cada um para si**, e tinham razão: um guarda camadas
//! (não achatáveis), o outro guarda canais de G-buffer + rig de luz. Nenhum dos dois é *pixels
//! chapados* — o caso base nunca existiu, e é o que este módulo é. Ele cobre o funil que **todas**
//! as ferramentas atravessam ([`crate::hero_intents::texture_edit::commit_edited_texture`], que
//! um gate prova ser porta única), e por isso fecha as oito de uma vez.
//!
//! ## A identidade é o CONTEÚDO
//!
//! O `ph2d_ecs::SpritePixels` carimbado no sprite guarda o `AssetId` (blake3, HR-6) que o
//! `AssetDb` cunhou no momento do upload. Não há contador de ids a manter, e dois sprites com os
//! mesmos pixels custam **uma** entrada no arquivo.
//!
//! ## Quem manda quando há mais de um dono
//!
//! Um sprite pintado carrega `PaintedDoc` **e** passaria por aqui (o Painter também sai pelo
//! funil). A colheita **salta** esses — e os do bake — porque o documento deles é mais rico que a
//! fotografia achatada, e guardar as duas coisas seria gravar duas verdades sobre o mesmo sprite.
//! No load a precedência é a mesma pela ORDEM: este restore corre primeiro, e
//! `restore_painted_docs` / `restore_baked_forms` correm depois e escrevem por cima.

use ph2d_ecs::{Entity, SpritePixels};
use ph2d_render::{Sprite, SpriteSource};
use ph2d_sprite_sheet::SpritePixelDoc;
use std::collections::BTreeMap;

impl crate::App {
    /// Colhe os pixels de cada sprite que tem os seus próprios, para embutir no arquivo.
    ///
    /// Devolve o documento já serializado (o campo é um blob opaco que carrega a própria versão).
    /// Vazio quando não há sprite individual — um projeto só de atlas não paga byte nenhum.
    pub(super) fn collect_sprite_pixels(&mut self) -> Vec<u8> {
        let Some(gfx) = self.gfx.as_mut() else {
            return Vec::new();
        };
        // 1. Quem tem pixels próprios? Recolhido numa passagem que SOLTA o mundo antes de tocar o
        //    `AssetDb` — os dois são campos irmãos do `AppGfx`, e segurar os dois empréstimos ao
        //    mesmo tempo não compila.
        //
        //    ⚠️ `Option<&PaintedDoc>` / `Option<&BakedForm>` vão na QUERY: a pergunta *"quem mais
        //    é dono destes pixels?"* faz parte do filtro, e deixá-la de fora é exatamente como a
        //    fotografia achatada acabaria por competir com o documento.
        let mut wanted: Vec<(ph2d_asset::AssetId, bool)> = Vec::new();
        {
            let world = gfx.sim.world_mut();
            let mut q = world.query::<(
                &Sprite,
                &SpritePixels,
                Option<&ph2d_ecs::PaintedDoc>,
                Option<&ph2d_ecs::BakedForm>,
            )>();
            for (sprite, pixels, painted, baked) in q.iter(world) {
                if !should_collect(&sprite.source, painted.is_some(), baked.is_some()) {
                    continue;
                }
                wanted.push((pixels.0, sprite.premultiplied));
            }
        }
        // 2. Os bytes.
        let mut docs: Vec<SpritePixelDoc> = Vec::new();
        for (id, premultiplied) in wanted {
            let Some(asset) = gfx.asset_db.get(&id) else {
                // Os bytes saíram do `AssetDb`. Salvar sem eles é o bug que este módulo existe
                // para fechar, então isto GRITA em vez de calar.
                eprintln!(
                    "[proj] pixels {} de um sprite individual nao estao no AssetDb — \
                     o sprite sera salvo SEM imagem",
                    id.to_hex()
                );
                continue;
            };
            let ph2d_asset::Asset::ImageRgba8 {
                width,
                height,
                pixels: rgba,
            } = &*asset
            else {
                continue;
            };
            docs.push(SpritePixelDoc {
                id,
                width: *width,
                height: *height,
                rgba: rgba.to_vec(),
                premultiplied,
            });
        }
        // O `encode` ordena e deduplica — dois sprites com os mesmos pixels custam uma entrada.
        match ph2d_sprite_sheet::encode(&docs) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("[proj] pixels de sprite nao serializaram: {e}");
                Vec::new()
            }
        }
    }

    /// Re-materializa os pixels e re-aponta cada sprite que os nomeia.
    ///
    /// Corre DEPOIS de `apply_project` (o mundo já foi restaurado, com bits novos) e **antes** de
    /// `restore_painted_docs` / `restore_baked_forms`, que têm documentos mais ricos para o mesmo
    /// sprite e escrevem por cima — a ordem é a precedência.
    ///
    /// ⚠️ **Não toca no `size`.** Ele é a pose em unidades de MUNDO (metros) e já veio correta no
    /// snapshot; escrever ali as dimensões da textura (pixels) fez um canvas de 1024 px reabrir
    /// com 1024 **metros** de lado — a textura gigante do 1º smoke da persistência do Painter. Um
    /// documento diz o que está *desenhado* num objeto; ele não redimensiona o objeto.
    pub(super) fn restore_sprite_pixels(&mut self, docs: Vec<SpritePixelDoc>) {
        if docs.is_empty() {
            return;
        }
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // 1. Os bytes voltam ao `AssetDb` e sobem para uma textura NOVA. O `texture_id` do save
        //    morreu com o processo que o criou; o que sobrevive é o `AssetId`.
        let mut by_id: BTreeMap<ph2d_asset::AssetId, (u32, bool)> = BTreeMap::new();
        for d in docs {
            gfx.asset_db
                .insert_image_rgba8(d.width, d.height, d.rgba.clone());
            match gfx.renderer.acquire_individual(d.width, d.height, &d.rgba) {
                Ok(texture_id) => {
                    by_id.insert(d.id, (texture_id, d.premultiplied));
                }
                Err(e) => eprintln!("[proj] textura dos pixels {} falhou: {e}", d.id.to_hex()),
            }
        }
        if by_id.is_empty() {
            return;
        }
        // 2. Que sprite (bits NOVOS) nomeia que pixels?
        let mut targets: Vec<(u64, ph2d_asset::AssetId)> = Vec::new();
        {
            let world = gfx.sim.world_mut();
            let mut q = world.query::<(Entity, &SpritePixels)>();
            for (e, p) in q.iter(world) {
                targets.push((e.to_bits(), p.0));
            }
        }
        // 3. Reata. O `premultiplied` vem do DOCUMENTO — é a única cópia que existe, porque no
        //    `Sprite` ele é `#[serde(skip)]` e volta sempre `false` do snapshot.
        for (bits, id) in targets {
            let Some(&(texture_id, premultiplied)) = by_id.get(&id) else {
                continue; // carimbo sem pixels no arquivo — nada a devolver
            };
            if let Some(mut sprite) = gfx.sim.world_mut().get_mut::<Sprite>(Entity::from_bits(bits))
            {
                reattach_pixels(&mut sprite, texture_id, premultiplied);
            }
        }
    }
}

/// **Quem entra no arquivo** — a regra de posse, isolada porque é onde os bugs de lógica moram.
///
/// Duas exclusões, e as duas por motivo próprio:
/// - **dono mais rico** (`PaintedDoc` / `BakedForm`): o documento deles reconstrói estes pixels e
///   muito mais; gravar os dois seria gravar duas verdades sobre o mesmo sprite, e a que
///   ganhasse seria a que corresse por último.
/// - **carimbo obsoleto**: um sprite que voltou ao Atlas mantém o `SpritePixels` até alguém o
///   retirar. Guardar pixels que ninguém mostra engorda o arquivo em silêncio.
fn should_collect(source: &SpriteSource, has_painted: bool, has_baked: bool) -> bool {
    if has_painted || has_baked {
        return false;
    }
    matches!(source, SpriteSource::Individual { .. })
}

/// Reata o sprite à textura recém-materializada — **e não toca em mais nada**.
///
/// ⚠️ Em especial, não toca no `size`: ele é a pose em unidades de MUNDO (metros) e já veio
/// correta no snapshot. Escrever ali as dimensões da textura (pixels) fez um canvas de 1024 px
/// reabrir com 1024 **metros** de lado — a textura gigante do 1º smoke da persistência do
/// Painter, e o irmão exato deste caminho.
///
/// ⚠️ O `premultiplied` vem do DOCUMENTO porque é a única cópia que existe: no `Sprite` ele é
/// `#[serde(skip)]` e volta sempre `false` do snapshot. Reatar sem ele devolveria bytes
/// premultiplicados marcados como alfa reto — a franja escura do BG-Removal, ressuscitada pelo
/// caminho do arquivo.
fn reattach_pixels(sprite: &mut Sprite, texture_id: u32, premultiplied: bool) {
    sprite.source = SpriteSource::Individual { texture_id };
    sprite.premultiplied = premultiplied;
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_asset::AssetId;

    fn individual() -> SpriteSource {
        SpriteSource::Individual { texture_id: 1 }
    }

    #[test]
    fn an_individual_sprite_with_no_richer_owner_is_collected() {
        assert!(should_collect(&individual(), false, false));
    }

    /// O Painter e o bake 3D guardam um documento que RECONSTRÓI estes pixels. Gravar a
    /// fotografia achatada ao lado dele é a duplicação que a ordem do load teria de desempatar.
    #[test]
    fn a_sprite_with_a_richer_owner_is_left_to_that_owner() {
        assert!(!should_collect(&individual(), true, false), "pintado");
        assert!(!should_collect(&individual(), false, true), "assado");
        assert!(!should_collect(&individual(), true, true), "ambos");
    }

    /// Um carimbo que sobreviveu a uma volta ao Atlas não pode arrastar pixels mortos.
    #[test]
    fn a_sprite_that_is_no_longer_individual_is_not_collected() {
        assert!(!should_collect(&SpriteSource::Atlas { key: 0 }, false, false));
        assert!(!should_collect(
            &SpriteSource::CookedTexture {
                logical_id: ph2d_asset::LogicalTextureId::from_digest([0u8; 32]),
            },
            false,
            false
        ));
    }

    /// Reabrir devolve os PIXELS ao sprite, não uma pose nova: o tamanho em mundo e o resto do
    /// `Sprite` ficam onde estavam. É a lição do canvas de 1024 metros.
    #[test]
    fn reattaching_moves_the_texture_and_nothing_else() {
        let mut sprite = Sprite::individual(1, [3.0, 5.0], [1.0, 1.0, 1.0, 1.0]);
        sprite.opacity = 0.25;
        sprite.flip_x = true;
        reattach_pixels(&mut sprite, 42, false);
        assert!(matches!(
            sprite.source,
            SpriteSource::Individual { texture_id: 42 }
        ));
        assert_eq!(sprite.size, [3.0, 5.0], "a pose em METROS nao se toca");
        assert_eq!(sprite.opacity, 0.25);
        assert!(sprite.flip_x);
    }

    /// ⚠️ A única cópia do flag é o documento — no `Sprite` ele é `#[serde(skip)]` e volta
    /// SEMPRE `false`. Sem esta linha, um sprite com fundo removido reabre com a franja escura.
    #[test]
    fn reattaching_restores_the_premultiplied_flag_from_the_document() {
        // O estado que o snapshot de facto entrega: o flag perdido.
        let mut sprite = Sprite::individual(1, [1.0, 1.0], [1.0, 1.0, 1.0, 1.0]);
        assert!(!sprite.premultiplied, "o snapshot devolve `false`");
        reattach_pixels(&mut sprite, 7, true);
        assert!(sprite.premultiplied, "o documento tem de o repor");
    }

    /// A identidade é o CONTEÚDO: os mesmos bytes cunham o mesmo nome, então dois sprites iguais
    /// custam uma entrada no arquivo, e re-salvar sem editar não muda o documento.
    #[test]
    fn the_same_pixels_always_get_the_same_name() {
        let a = AssetId::from_bytes(&[1, 2, 3, 4]);
        let b = AssetId::from_bytes(&[1, 2, 3, 4]);
        let c = AssetId::from_bytes(&[1, 2, 3, 5]);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
