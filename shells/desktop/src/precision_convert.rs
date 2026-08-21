//! **CONVERTER A PRECISÃO dos pixels de uma sprite** — o que os dois botões `RGBA8 / RGBA16` do
//! Inspector de facto fazem. Plano
//! [`docs/Sprite_projeto/18`](../../../docs/Sprite_projeto/18_precisao_de_16_bits_nas_sprites.md),
//! W5.
//!
//! Enio, 2026-08-20: *"vamos corrigir a UI da sprite no painel com as duas opções e com os botões
//! que a partir de agora devem converter a imagem."*
//!
//! # Por que a conversão mora no SHELL
//!
//! O painel levanta uma intenção e mais nada: ele não tem o `AssetDb`, não tem o device, e não
//! sabe que converter para 16 bits **muda a estratégia** da sprite. As três coisas vivem aqui.
//!
//! # A fonte dos pixels é o ASSET, nunca a GPU
//!
//! ⚠️ Ler de volta a textura seria o caminho óbvio e é o errado por duas razões: um `readback` de
//! `Rgba16Float` não tem o mesmo passo de linha que um de 8 bits (seria uma segunda lei a manter),
//! e a textura **já pode ter sido mexida** por uma prévia. O `AssetDb` guarda os bytes com a
//! precisão verdadeira e a identidade de conteúdo — é ele o oráculo.
//!
//! # O que a conversão NÃO faz
//!
//! ⛔ **Não toca em premultiplicação.** Ela é um facto sobre os bytes e viaja no
//! `Sprite::premultiplied`; uma troca de precisão que a mudasse escureceria toda borda macia sem
//! ninguém pedir.
//!
//! ⛔ **Não muda o tamanho no mundo.** `8 → 16 → 8` devolve os mesmos bytes (gate exaustivo em
//! [`ph2d_color::precision`]) e tem de devolver a mesma sprite.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_color::Precision;
use ph2d_ecs::{Entity, SimWorld, SpritePixels};
use ph2d_editor::{Toast, ToastQueue};
use ph2d_render::{Sprite, SpriteRenderer, SpriteSource};
use std::collections::BTreeMap;

/// De onde vêm os bytes desta sprite, hoje.
///
/// ⚠️ **Uma sprite `Individual` nomeia os seus bytes pelo `SpritePixels`; uma de atlas, pela chave
/// da célula.** São dois caminhos porque são duas formas de posse — e uma sprite recém-promovida
/// que ainda não foi carimbada não tem bytes nomeados, que é um estado real e não um erro.
fn source_asset(
    entity: Entity,
    sim: &SimWorld,
    sprite: &Sprite,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
) -> Option<AssetId> {
    match sprite.source {
        SpriteSource::Atlas { key } => atlas_asset_map.get(&key).copied(),
        SpriteSource::Individual { .. } => sim.world().get::<SpritePixels>(entity).map(|p| p.0),
        SpriteSource::CookedTexture { .. } => None,
    }
}

/// **Aplica um pedido de troca de precisão.** Devolve `true` se algo mudou ou se um toast subiu.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply(
    request: Option<(u64, Precision)>,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    toasts: &mut ToastQueue,
) -> bool {
    let Some((entity_bits, wanted)) = request else {
        return false;
    };
    let entity = Entity::from_bits(entity_bits);
    let Some(sprite) = sim.world().get::<Sprite>(entity).copied() else {
        // A entidade sumiu entre o clique e o dreno (despawn, delete na hierarquia).
        return false;
    };
    if matches!(sprite.source, SpriteSource::CookedTexture { .. }) {
        toasts.push(Toast::info(
            "Cooked textures come from the asset pipeline — format is read-only",
        ));
        return true;
    }
    let Some(asset_id) = source_asset(entity, sim, &sprite, atlas_asset_map) else {
        toasts.push(Toast::error("Cannot convert — source pixels missing"));
        return true;
    };
    let Some(asset) = asset_db.get(&asset_id) else {
        toasts.push(Toast::error("Cannot convert — source pixels missing"));
        return true;
    };
    if asset.precision() == Some(wanted) {
        // Não é uma edição. O painel já curto-circuita isto; este ramo apanha uma publicação
        // fora-de-banda (script, MCP futuro) e mantém o undo limpo.
        return false;
    }
    let Some((width, height)) = asset.image_dimensions() else {
        toasts.push(Toast::error("Cannot convert — source is not an image"));
        return true;
    };

    // ⚠️ **A conversão parte SEMPRE dos bytes de 8 bits do asset**, nos dois sentidos. Para
    // `→ Rgba16` isso é exatamente o que se quer; para `→ Rgba8` a porta `image_rgba8` já faz a
    // descida com a mesma lei que o resto do app usa. *Uma conversão, uma lei.*
    let uploaded = match wanted {
        Precision::Rgba16 => {
            let Some((_, _, straight)) = asset.image_rgba8() else {
                toasts.push(Toast::error("Cannot convert — source is not an image"));
                return true;
            };
            let halves = ph2d_color::rgba8_to_rgba16(&straight);
            let pixels_id = asset_db.insert_image_rgba16(width, height, halves.clone());
            renderer
                .acquire_individual_16(width, height, &halves)
                .map(|texture_id| (texture_id, pixels_id))
        }
        Precision::Rgba8 => {
            // ⚠️ **A descida é o ÚNICO dos dois sentidos que perde, e é aqui que ela ganha dither**
            // (plano 18 W6). O `image_rgba8` continua a ser a porta **fiel** — ele serve leituras, e
            // uma leitura que devolvesse valores diferentes dos guardados não é uma leitura. Este
            // sítio não está a ler: está a **cumprir um pedido do autor** de ficar com 8 bits para
            // sempre, e é exatamente onde as faixas de um degradê nascem.
            //
            // ⚠️ Quem chegou aqui de 8 bits já saiu no curto-circuito de `precision()` lá em cima,
            // por isso o ramo `None` não é a imagem de 8 bits: é o asset deixar de ser imagem entre
            // duas linhas. O `image_rgba8` apanha esse caso com a mesma lei de sempre.
            let bytes = match asset.image_rgba16() {
                Some((_, _, halves)) => ph2d_color::rgba16_to_rgba8_dithered(halves, width),
                None => {
                    let Some((_, _, straight)) = asset.image_rgba8() else {
                        toasts.push(Toast::error("Cannot convert — source is not an image"));
                        return true;
                    };
                    straight.into_owned()
                }
            };
            let pixels_id = asset_db.insert_image_rgba8(width, height, bytes.clone());
            renderer
                .acquire_individual(width, height, &bytes)
                .map(|texture_id| (texture_id, pixels_id))
        }
    };
    let (texture_id, pixels_id) = match uploaded {
        Ok(v) => v,
        Err(e) => {
            toasts.push(Toast::error(format!("Format conversion failed: {e}")));
            return true;
        }
    };

    // ⚠️ **A MESMA cauda que toda ferramenta de imagem atravessa** — `rebind_to_individual` carrega
    // as cinco invariantes de "esta sprite passou a ter pixels próprios", e duas delas só falham
    // depois de fechar e reabrir o projeto. Reescrevê-las aqui era pedir que duas cópias
    // concordassem para sempre.
    //
    // ⚠️ **A estratégia vira `Individual` nos DOIS sentidos, e é deliberado.** Para 16 bits é
    // obrigatório (o atlas é uma textura com um formato, §3.3). Para 8 bits seria possível voltar
    // ao atlas — mas o caminho de volta já existe e tem dono: o par `Strategy` logo acima, que
    // conta as referências da célula antes de a libertar. *Duas portas para o mesmo destino
    // divergem na primeira que ganhar uma regra.*
    crate::hero_intents::texture_rebind::rebind_to_individual(
        entity,
        sim,
        texture_id,
        pixels_id,
        sprite.size,
        sprite.premultiplied,
        // ⚠️ **A conversão sobe A MESMA imagem noutra precisão** — mesmo conteúdo, mesmas
        // dimensões — por isso o recorte do artista continua a apontar para os mesmos pixels.
        // Este argumento é a correção do defeito de 2026-08-21: a cauda partilhada apagava a
        // janela porque a regra do OUTRO chamador tinha vindo junto na extração.
        crate::hero_intents::texture_rebind::SamplingWindow::Survives,
    );
    toasts.push(Toast::success(format!("Format · {}", wanted.label())));
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db_with_rgba8(w: u32, h: u32) -> (AssetDb, AssetId) {
        let db = AssetDb::new();
        let px: Vec<u8> = (0..(w * h * 4)).map(|i| (i % 251) as u8).collect();
        let id = db.insert_image_rgba8(w, h, px);
        (db, id)
    }

    /// **Uma sprite de atlas nomeia os bytes pela CÉLULA; uma individual, pelo carimbo.**
    ///
    /// ⚠️ Este é o ponto onde uma conversão silenciosamente não faz nada: sem achar o asset, a
    /// função só pode desistir, e desistir sem toast seria um botão que não responde.
    #[test]
    fn the_source_asset_comes_from_the_cell_or_from_the_stamp() {
        let (_, id) = db_with_rgba8(2, 2);
        let mut sim = SimWorld::default();

        let atlas = sim
            .world_mut()
            .spawn((Sprite::atlas(4, [1.0, 1.0], [1.0; 4]),))
            .id();
        let mut map = BTreeMap::new();
        map.insert(4u32, id);
        let sprite = sim.world().get::<Sprite>(atlas).copied().expect("sprite");
        assert_eq!(source_asset(atlas, &sim, &sprite, &map), Some(id));

        let individual = sim
            .world_mut()
            .spawn((
                Sprite::individual(9, [1.0, 1.0], [1.0; 4]),
                SpritePixels(id),
            ))
            .id();
        let sprite = sim
            .world()
            .get::<Sprite>(individual)
            .copied()
            .expect("sprite");
        assert_eq!(
            source_asset(individual, &sim, &sprite, &BTreeMap::new()),
            Some(id),
            "uma individual nomeia os bytes pelo carimbo, e o mapa do atlas nao lhe diz respeito"
        );
    }

    /// **Uma individual SEM carimbo não tem bytes nomeados** — estado real (uma sprite acabada de
    /// promover, antes do carimbo), e a conversão tem de o dizer em vez de calar.
    #[test]
    fn an_unstamped_individual_has_no_named_bytes() {
        let mut sim = SimWorld::default();
        let e = sim
            .world_mut()
            .spawn((Sprite::individual(9, [1.0, 1.0], [1.0; 4]),))
            .id();
        let sprite = sim.world().get::<Sprite>(e).copied().expect("sprite");
        assert_eq!(source_asset(e, &sim, &sprite, &BTreeMap::new()), None);
    }

    /// **Uma textura cozida não tem pixels na CPU para converter.**
    #[test]
    fn a_cooked_texture_names_nothing() {
        let mut sim = SimWorld::default();
        let mut sprite = Sprite::individual(1, [1.0, 1.0], [1.0; 4]);
        sprite.source = SpriteSource::CookedTexture {
            logical_id: ph2d_asset::LogicalTextureId::from_source_bytes(b"cozida"),
        };
        let e = sim.world_mut().spawn((sprite,)).id();
        let sprite = sim.world().get::<Sprite>(e).copied().expect("sprite");
        assert_eq!(source_asset(e, &sim, &sprite, &BTreeMap::new()), None);
    }
}
