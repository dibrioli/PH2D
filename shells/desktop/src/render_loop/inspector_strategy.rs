//! **A CONVERSÃO ENTRE ESTRATÉGIAS DE ORIGEM** do Inspector (`Render Source` → `Strategy`).
//!
//! Irmão de [`super::inspector_commits`], e o corte é o que o marcador de exceção de LOC daquele
//! arquivo prescreve por escrito desde 2026-06-02 (*"splitting into per-field sibling modules is a
//! focused Sprite-Inspector follow-up"*). O corte é por responsabilidade: lá ficam os *drains* de
//! campo do Inspector (Transform / Visibility / Name / Sprite / Reimport); aqui fica a única coisa
//! que **muda de onde os pixels vêm**.
//!
//! ## O estado que este módulo encontrou (medido 2026-08-19)
//!
//! Só **Atlas → Individual** funcionava. `Individual → Atlas` devolvia *"M14.C+ (atlas re-insert
//! path)"* desde o M14.C, e o resultado prático era uma **rua sem saída**: toda ferramenta de
//! imagem converte para Individual (`commit_edited_texture`), então bastava recortar um sprite
//! para nunca mais o poder devolver ao atlas.
//!
//! ## As duas regras que este módulo carrega
//!
//! 1. **Libertar um recurso partilhado exige CONTAR quem o usa.** A célula de atlas e a textura
//!    individual são ambas endereçadas por um número que mais de um sprite pode nomear (duplicar
//!    um sprite copia a componente `Sprite`, e nada chama `retain` no store). Libertar sem contar
//!    apaga a imagem de um sprite que ninguém tocou — por isso [`sprites_referencing_atlas_key`] e
//!    [`sprites_referencing_texture_id`] correm antes de qualquer `remove`/`release`.
//! 2. **O alfa é um facto sobre os BYTES.** O atlas guarda alfa **reto**; um sprite que passou
//!    pelo BG-Removal traz bytes **premultiplicados**. Converter sem desfazer isso escureceria a
//!    borda anti-serrilhada — é a mesma classe de bug que o funil `texture_edit` existe para
//!    tornar impossível, e a razão de a leitura passar por ele e não por um `readback` local.

use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::scene::{
    ComponentRegistry, EditorCommand, EditorCommandQueue, apply_editor_commands,
};
use ph2d_ecs::{SimWorld, SpritePixels};
use ph2d_editor::screens::hero::HeroScreen;
use ph2d_editor::{RequestedSpriteStrategy, Toast, ToastQueue};
use ph2d_render::{Sprite, SpriteRenderer, SpriteSource};
use std::collections::BTreeMap;

use crate::hero_intents::texture_edit;

/// Aplica um pedido de troca de estratégia. Devolve `true` se algo mudou ou se um toast subiu
/// (o chamador ORa no `title_dirty`).
#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch(
    change: Option<(u64, RequestedSpriteStrategy)>,
    hero: &mut HeroScreen,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
    next_import_cell: &mut u32,
    toasts: &mut ToastQueue,
    editor_queue: &mut EditorCommandQueue,
    component_registry: &ComponentRegistry,
    sprite_type_id: u64,
) -> bool {
    let Some((entity_bits, requested)) = change else {
        return false;
    };
    let entity = ph2d_ecs::Entity::from_bits(entity_bits);
    let Some(current) = sim.world().get::<Sprite>(entity).copied() else {
        // A entidade sumiu entre o commit e o dreno (despawn, delete na hierarquia).
        return false;
    };
    match (current.source, requested) {
        (SpriteSource::Atlas { key }, RequestedSpriteStrategy::Individual) => {
            promote_to_individual(
                entity,
                entity_bits,
                key,
                current,
                hero,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                toasts,
                editor_queue,
                component_registry,
                sprite_type_id,
            )
        }
        (SpriteSource::Individual { texture_id }, RequestedSpriteStrategy::Atlas) => {
            demote_to_atlas(
                entity,
                entity_bits,
                texture_id,
                current,
                hero,
                sim,
                renderer,
                asset_db,
                atlas_asset_map,
                next_import_cell,
                toasts,
                editor_queue,
                component_registry,
                sprite_type_id,
            )
        }
        // Pedido igual ao atual. O `apply_event` já curto-circuita cliques idênticos; este braço
        // existe para que uma publicação fora-de-banda (script, MCP futuro) também não faça nada.
        (SpriteSource::Atlas { .. }, RequestedSpriteStrategy::Atlas)
        | (SpriteSource::Individual { .. }, RequestedSpriteStrategy::Individual) => false,
        // Uma textura cozida vem do pipeline de asset: não tem chave de atlas nem bake legível na
        // CPU, então nenhuma estratégia é autorável a partir do Inspector.
        (SpriteSource::CookedTexture { .. }, _) => {
            toasts.push(Toast::info(
                "Cooked textures come from the asset pipeline — render strategy is read-only",
            ));
            reject_visual_reset(hero, requested);
            true
        }
        // **Hand-packed é um ESTADO a que se CHEGA, não um botão que se aperta** (plano
        // `docs/Sprite_projeto/17` §6-§8). Um sprite torna-se hand-packed por entrar numa folha e
        // ela ser assada — não há aqui a pergunta *"para qual folha, e para qual região?"*, e
        // inventar um seletor neste radio seria uma segunda porta para o que o menu da hierarquia
        // já faz melhor (ele tem a seleção e a linha clicada).
        //
        // ⚠️ O toast nomeia o que FALTA e **por que nome procurá-lo na tela**. A 1ª versão dizia
        // *"M14.C+"*, que não diz a ninguém o que fazer a seguir e manteve este botão morto três
        // meses; a 2ª dizia *"pack one from the selection"* — verdade quando existia um pill
        // `[SHEET]`, e **envelhecida no dia em que ele saiu**. *Uma recusa que nomeia um gesto
        // inexistente é pior que uma recusa muda: ela manda o artista procurar o que não há.*
        (_, RequestedSpriteStrategy::HandPacked) => {
            toasts.push(Toast::info(
                "Hand-packed comes from a sheet: right-click in the Hierarchy \u{00b7} Pack into Sheet, then Bake Sheet",
            ));
            reject_visual_reset(hero, requested);
            true
        }
    }
}

/// Atlas → Individual: os pixels do asset sobem para uma textura própria, e **a célula do atlas é
/// devolvida** se mais nenhum sprite a nomeia.
///
/// ⚠️ A devolução da célula é a correção de um vazamento que existia desde o M14.C:
/// `remove_atlas_sprite` existe no renderer e **não era chamado por ninguém**, então cada
/// conversão deixava a célula ocupada para sempre — e o `atlas_asset_map` mantinha a entrada, o
/// que fazia o arquivo continuar a gravar pixels que nenhum sprite já mostrava.
#[allow(clippy::too_many_arguments)]
fn promote_to_individual(
    entity: ph2d_ecs::Entity,
    entity_bits: u64,
    key: u32,
    current: Sprite,
    hero: &mut HeroScreen,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
    toasts: &mut ToastQueue,
    editor_queue: &mut EditorCommandQueue,
    component_registry: &ComponentRegistry,
    sprite_type_id: u64,
) -> bool {
    let decoded = atlas_asset_map.get(&key).and_then(|aid| {
        asset_db.get(aid).and_then(|asset| match &*asset {
            ph2d_asset::Asset::ImageRgba8 {
                width,
                height,
                pixels,
            } => Some((*width, *height, pixels.to_vec())),
            _ => None,
        })
    });
    let Some((w, h, pixels)) = decoded else {
        toasts.push(Toast::error(
            "Cannot promote to Individual — source asset missing",
        ));
        reject_visual_reset(hero, RequestedSpriteStrategy::Individual);
        return true;
    };
    let texture_id = match renderer.acquire_individual(w, h, &pixels) {
        Ok(id) => id,
        Err(err) => {
            toasts.push(Toast::error(format!("Individual acquire failed: {err}")));
            reject_visual_reset(hero, RequestedSpriteStrategy::Individual);
            return true;
        }
    };
    let mut updated = current;
    updated.source = SpriteSource::Individual { texture_id };
    // Os pixels recém-decodificados são alfa RETO; limpa qualquer flag herdado de um Apply de
    // BG-Removal anterior neste sprite.
    updated.premultiplied = false;
    // ⚠️ `region_filter_clip` NÃO se toca aqui, e é decisão: os construtores usam `false` para
    // Individual, mas o valor só é lido quando `region_enabled`, e mudá-lo numa conversão altera
    // o que o artista vê num sprite que ele não pediu para alterar. A direção oposta é outra
    // história — lá aparecem vizinhos de verdade (vide `demote_to_atlas`).
    if !write_sprite(
        entity_bits,
        updated,
        sim,
        editor_queue,
        component_registry,
        sprite_type_id,
        toasts,
    ) {
        return true;
    }
    // O nome durável dos pixels: os mesmos bytes que subiram para a GPU (plano §3). Sem isto, um
    // save/load devolveria este sprite invisível — a rua sem saída ficaria ainda mais escura.
    let pixels_id = asset_db.insert_image_rgba8(w, h, pixels);
    sim.world_mut()
        .entity_mut(entity)
        .insert(SpritePixels(pixels_id));
    // A célula volta ao atlas — se, e só se, mais nenhum sprite a nomeia.
    release_atlas_key_if_unused(key, sim, renderer, atlas_asset_map);
    toasts.push(Toast::success(format!(
        "Strategy · Individual (texture {texture_id})"
    )));
    true
}

/// Individual → Atlas: os pixels voltam a ser empacotados numa célula do atlas partilhado.
///
/// A leitura passa pelo funil `texture_edit` (o único sítio sancionado a fazer `readback`), que
/// devolve um [`ph2d_render::SpriteImage`] a **carregar o seu `AlphaMode`** — e é isso que torna
/// a conversão correta: o atlas guarda alfa **reto**, e um sprite vindo do BG-Removal traz bytes
/// premultiplicados que precisam de ser desfeitos antes de entrar.
#[allow(clippy::too_many_arguments)]
fn demote_to_atlas(
    entity: ph2d_ecs::Entity,
    entity_bits: u64,
    texture_id: u32,
    current: Sprite,
    hero: &mut HeroScreen,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    asset_db: &AssetDb,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
    next_import_cell: &mut u32,
    toasts: &mut ToastQueue,
    editor_queue: &mut EditorCommandQueue,
    component_registry: &ComponentRegistry,
    sprite_type_id: u64,
) -> bool {
    let Some(read) =
        texture_edit::read_sprite_source(entity, sim, renderer, asset_db, atlas_asset_map)
    else {
        toasts.push(Toast::error(
            "Cannot pack into the atlas — this sprite's pixels are unreadable",
        ));
        reject_visual_reset(hero, RequestedSpriteStrategy::Atlas);
        return true;
    };
    // O atlas guarda alfa RETO. `into_straight` é no-op para quem já o é.
    let straight = read.image.into_straight();
    let (w, h) = (straight.width, straight.height);
    // A célula: o MESMO contador do import, para que uma conversão e um import nunca se sentem na
    // mesma célula (o contrato do `next_import_cell`, que o load do projeto também respeita).
    let key = *next_import_cell;
    let asset_id = asset_db.insert_image_rgba8(w, h, straight.pixels.clone());
    // Registado ANTES do insert: um regrow disparado por ele precisa de ver a chave nova.
    atlas_asset_map.insert(key, asset_id);
    let insert = {
        let map = &*atlas_asset_map;
        let fetch = |k: u32| -> Option<Vec<u8>> {
            let aid = map.get(&k)?;
            match &*asset_db.get(aid)? {
                ph2d_asset::Asset::ImageRgba8 { pixels, .. } => Some(pixels.to_vec()),
                _ => None,
            }
        };
        renderer.insert_atlas_sprite_with_regrow(key, w, h, &straight.pixels, fetch)
    };
    if let Err(e) = insert {
        // Rollback: a chave não pode ficar pendurada apontando para uma região que não existe.
        atlas_asset_map.remove(&key);
        // ⚠️ O toast nomeia **de que recurso** é o limite, e traz os números — um *"não foi
        // possível"* manda o artista adivinhar (`CLAUDE.md` §0).
        toasts.push(Toast::error(format!(
            "Cannot pack {w}×{h} into the shared atlas: {e}. Keep this sprite Individual."
        )));
        reject_visual_reset(hero, RequestedSpriteStrategy::Atlas);
        return true;
    }
    *next_import_cell = next_import_cell.saturating_add(1);
    let mut updated = current;
    updated.source = SpriteSource::Atlas { key };
    // Os bytes que entraram são retos, por construção (`into_straight` acima).
    updated.premultiplied = false;
    // ⚠️ AQUI o clip muda, e ao contrário do sentido oposto ele tem consequência MEDÍVEL: no
    // atlas partilhado há vizinhos de verdade, e sem o clamp a amostragem bilinear puxa os texels
    // da região ao lado pela borda.
    updated.region_filter_clip = true;
    if !write_sprite(
        entity_bits,
        updated,
        sim,
        editor_queue,
        component_registry,
        sprite_type_id,
        toasts,
    ) {
        return true;
    }
    // O carimbo de pixels próprios deixa de fazer sentido: quem guarda estes bytes agora é o
    // `atlas_asset_map`, pelo caminho do `collect_assets`. Deixá-lo faria o arquivo gravar os
    // mesmos pixels duas vezes, por dois donos.
    sim.world_mut().entity_mut(entity).remove::<SpritePixels>();
    release_texture_if_unused(texture_id, sim, renderer);
    toasts.push(Toast::success(format!("Strategy · Atlas (cell {key})")));
    true
}

/// Escreve o `Sprite` pelo caminho canônico de componente (fila + registro), e não por uma escrita
/// direta no mundo: um segundo caminho para *"editar um componente"* é como um deles passa a não
/// aparecer no undo, no MCP e no log de auditoria. Devolve `false` se algo falhou (já toastado).
fn write_sprite(
    entity_bits: u64,
    updated: Sprite,
    sim: &mut SimWorld,
    editor_queue: &mut EditorCommandQueue,
    component_registry: &ComponentRegistry,
    sprite_type_id: u64,
    toasts: &mut ToastQueue,
) -> bool {
    let data = match postcard::to_allocvec(&updated) {
        Ok(d) => d,
        Err(e) => {
            toasts.push(Toast::error(format!("Sprite encode failed: {e}")));
            return false;
        }
    };
    if let Err(e) = editor_queue.push(EditorCommand::SetComponent {
        entity: entity_bits,
        type_id: sprite_type_id,
        data,
    }) {
        toasts.push(Toast::error(format!("Editor queue full: {e}")));
        return false;
    }
    if let Err(e) = apply_editor_commands(sim.world_mut(), editor_queue, component_registry) {
        toasts.push(Toast::error(format!("Strategy commit failed: {e}")));
        return false;
    }
    true
}

/// Quantos sprites do mundo nomeiam esta célula de atlas.
///
/// ⚠️ Existe porque duplicar um sprite copia a componente `Sprite` **inteira**, chave incluída —
/// libertar sem contar apagaria a imagem de um sprite que o utilizador não tocou.
fn sprites_referencing_atlas_key(sim: &mut SimWorld, key: u32) -> usize {
    let world = sim.world_mut();
    let mut q = world.query::<&Sprite>();
    q.iter(world)
        .filter(|s| matches!(s.source, SpriteSource::Atlas { key: k } if k == key))
        .count()
}

/// Quantos sprites do mundo nomeiam esta textura individual.
///
/// ⚠️ O refcount do `IndividualTextureStore` **não** responde a isto: nada chama `retain` quando um
/// sprite é duplicado, então a contagem dele fica em 1 mesmo com dois donos, e um `release` apagaria
/// a textura do outro.
fn sprites_referencing_texture_id(sim: &mut SimWorld, texture_id: u32) -> usize {
    let world = sim.world_mut();
    let mut q = world.query::<&Sprite>();
    q.iter(world)
        .filter(
            |s| matches!(s.source, SpriteSource::Individual { texture_id: t } if t == texture_id),
        )
        .count()
}

/// Devolve a célula ao atlas quando o último sprite a larga.
fn release_atlas_key_if_unused(
    key: u32,
    sim: &mut SimWorld,
    renderer: &mut SpriteRenderer,
    atlas_asset_map: &mut BTreeMap<u32, AssetId>,
) {
    if sprites_referencing_atlas_key(sim, key) > 0 {
        return;
    }
    renderer.remove_atlas_sprite(key);
    atlas_asset_map.remove(&key);
}

/// Liberta a textura individual quando o último sprite a larga.
fn release_texture_if_unused(texture_id: u32, sim: &mut SimWorld, renderer: &mut SpriteRenderer) {
    if sprites_referencing_texture_id(sim, texture_id) > 0 {
        return;
    }
    renderer.individual_mut().release(texture_id);
}

/// Quando uma troca é recusada, devolve o botão clicado a `Normal` para ele não continuar a pintar
/// `Pressed`/`Hovered` ao lado do que continua ativo.
fn reject_visual_reset(hero: &mut HeroScreen, clicked: RequestedSpriteStrategy) {
    let id = match clicked {
        RequestedSpriteStrategy::Atlas => {
            ph2d_editor::screens::hero::ids::INSP_RENDER_STRATEGY_ATLAS
        }
        RequestedSpriteStrategy::Individual => {
            ph2d_editor::screens::hero::ids::INSP_RENDER_STRATEGY_INDIVIDUAL
        }
        RequestedSpriteStrategy::HandPacked => {
            ph2d_editor::screens::hero::ids::INSP_RENDER_STRATEGY_HANDPACKED
        }
    };
    if let Some(ph2d_editor::InteractiveState::Button { state }) = hero.store.get_mut(id) {
        *state = ph2d_editor::widget::ButtonState::Normal;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world_with(sources: &[SpriteSource]) -> SimWorld {
        let mut sim = SimWorld::new();
        for source in sources {
            let mut sprite = Sprite::atlas(0, [1.0, 1.0], [1.0; 4]);
            sprite.source = *source;
            sim.world_mut().spawn(sprite);
        }
        sim
    }

    fn atlas(key: u32) -> SpriteSource {
        SpriteSource::Atlas { key }
    }
    fn individual(texture_id: u32) -> SpriteSource {
        SpriteSource::Individual { texture_id }
    }

    /// A contagem é sobre o MUNDO, não sobre o sprite em mãos: é ela que decide se libertar o
    /// recurso apaga a imagem de outro sprite.
    #[test]
    fn an_atlas_cell_named_once_counts_once() {
        let mut sim = world_with(&[atlas(3), atlas(9)]);
        assert_eq!(sprites_referencing_atlas_key(&mut sim, 3), 1);
        assert_eq!(sprites_referencing_atlas_key(&mut sim, 9), 1);
        assert_eq!(sprites_referencing_atlas_key(&mut sim, 4), 0);
    }

    /// ⚠️ **A regressão que esta contagem existe para impedir.** Duplicar um sprite copia a
    /// componente `Sprite` inteira, chave incluída — converter UM dos dois e libertar a célula
    /// deixaria o irmão a apontar para uma região que já não existe (invisível, em silêncio).
    #[test]
    fn a_shared_atlas_cell_is_seen_by_every_sprite_that_names_it() {
        let mut sim = world_with(&[atlas(7), atlas(7), atlas(7)]);
        assert_eq!(sprites_referencing_atlas_key(&mut sim, 7), 3);
    }

    /// ⚠️ O refcount do `IndividualTextureStore` **não** responde a isto: nada chama `retain`
    /// quando um sprite é duplicado, então ele diria 1 com dois donos — e um `release` apagaria a
    /// textura do outro.
    #[test]
    fn a_shared_individual_texture_is_seen_by_every_sprite_that_names_it() {
        let mut sim = world_with(&[individual(5), individual(5)]);
        assert_eq!(sprites_referencing_texture_id(&mut sim, 5), 2);
        assert_eq!(sprites_referencing_texture_id(&mut sim, 6), 0);
    }

    /// As duas contagens não se confundem: a chave `2` do atlas e a textura `2` são números do
    /// mesmo tipo em espaços diferentes.
    #[test]
    fn the_two_namespaces_do_not_answer_for_each_other() {
        let mut sim = world_with(&[atlas(2), individual(2)]);
        assert_eq!(sprites_referencing_atlas_key(&mut sim, 2), 1);
        assert_eq!(sprites_referencing_texture_id(&mut sim, 2), 1);
    }

    /// Uma textura cozida não é nomeada por nenhuma das duas — libertar a partir dela seria
    /// libertar o recurso de outra pessoa.
    #[test]
    fn a_cooked_source_names_neither_namespace() {
        let mut sim = world_with(&[SpriteSource::CookedTexture {
            logical_id: ph2d_asset::LogicalTextureId::from_digest([0u8; 32]),
        }]);
        assert_eq!(sprites_referencing_atlas_key(&mut sim, 0), 0);
        assert_eq!(sprites_referencing_texture_id(&mut sim, 0), 0);
    }
}
