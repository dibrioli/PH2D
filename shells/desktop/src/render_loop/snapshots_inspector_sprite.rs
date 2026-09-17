//! **A secção Sprite do Inspector** — a construção do `InspectorSpriteInfo` da seleção (a fonte, a precisão, a
//! autoria de folha, o «Mixed» de uma multi-seleção) e os ajudantes dela. Filho do [`super`] (`snapshots`) por
//! `#[path]` (OBRA 3 da `line/render-bodies`): a `publish` do Inspector chama [`sprite_info`] no sítio do bloco.

use super::*;
use ph2d_i18n::{tr, tr_with};

/// **A autoria de folha que o painel MOSTRA** — a regra, isolada do mundo para poder ser testada.
///
/// Duas fontes, na ordem em que se tornam verdadeiras:
///
/// 1. **assado** (`SpriteSheetRef`) — o `storage` já chega como `HandPacked` e há uma região
///    nomeada; o rótulo é `folha · região`;
/// 2. **arranjado** (filho de uma folha, ainda sem `SpriteSheetRef`) — o armazenamento é mesmo
///    `Individual`/`Atlas`, mas a AUTORIA já é da folha. Mostra `HandPacked` com o rótulo a dizer
///    que ainda não foi assado.
///
/// ⚠️ **Os ids `0/0` do caso 2 não significam nada**, e é o rótulo (sempre presente aí) que
/// impede que alguém os leia: a linha `Storage` prefere-o, e só cai nos números quando ele falta —
/// o que neste caso não pode acontecer. *Um número sem significado é aceitável enquanto for
/// inalcançável; deixar de o ser é a regressão a vigiar.*
pub(super) fn sheet_authorship(
    storage: ph2d_editor_core::InspectorSpriteSource,
    unbaked_sheet: Option<&str>,
    baked_label: Option<String>,
) -> (ph2d_editor_core::InspectorSpriteSource, Option<String>) {
    match unbaked_sheet {
        Some(name) => (
            ph2d_editor_core::InspectorSpriteSource::HandPacked {
                sheet: 0,
                region: 0,
            },
            Some(tr_with(
                "shell.snapshots_inspector_sprite.not_baked_yet",
                &[("name", &name)],
            )),
        ),
        None => (storage, baked_label),
    }
}

/// BulkSelect (T2.0): compute which editable `Sprite` fields diverge
/// across the `selected` entities, relative to `primary`. Exact equality
/// is intentional — "Mixed" means the stored values literally differ, so
/// editing the field would stomp the divergence. `selected` includes the
/// primary (a no-op self-compare); unknown / non-sprite entities are
/// skipped. Returns all-`false` for a single selection.
///
/// ⚠️ **Os três componentes do corte (ADR-0164 F1 passo 6) comparam-se pelo valor EFETIVO**, e é
/// a mesma lei que o [`emissive_of`] abaixo já escrevia: *a ausência **é** o valor neutro*. Uma
/// sprite sem `SpriteGrid` e outra com `1×1` concordam; comparar `Option` diria que divergem, e o
/// chip «Mixed» acenderia sobre duas sprites que têm exatamente a mesma grelha de uma célula.
/// ⭐ A ÚNICA exceção é o `region_enabled`, que **é** a presença — ali `Option::is_some()` é o valor.
#[allow(clippy::float_cmp)] // exact compare: same stored value = not mixed
fn compute_sprite_mixed(
    world: &ph2d_ecs::World,
    selected: &[u64],
    primary_bits: u64,
) -> ph2d_editor_core::InspectorSpriteMixed {
    let mut m = ph2d_editor_core::InspectorSpriteMixed::default();
    let primary_entity = ph2d_ecs::Entity::from_bits(primary_bits);
    let Some(primary) = world.get::<Sprite>(primary_entity) else {
        return m;
    };
    let p_grid = grid_of(world, primary_entity);
    let p_region = world.get::<ph2d_ecs::SpriteRegion>(primary_entity).copied();
    let p_rect = p_region.map_or([0.0; 4], |r| r.rect);
    let p_corner = corner_tint_of(world, primary_entity);
    for &bits in selected {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let Some(s) = world.get::<Sprite>(entity) else {
            continue;
        };
        let grid = grid_of(world, entity);
        let region = world.get::<ph2d_ecs::SpriteRegion>(entity).copied();
        let rect = region.map_or([0.0; 4], |r| r.rect);
        m.flip_x |= s.flip_x != primary.flip_x;
        m.flip_y |= s.flip_y != primary.flip_y;
        m.tint_fill |= s.tint_fill != primary.tint_fill;
        m.centered |= s.centered != primary.centered;
        m.region_enabled |= region.is_some() != p_region.is_some();
        m.region_filter_clip |= region.map(|r| r.filter_clip) != p_region.map(|r| r.filter_clip);
        m.opacity |= s.opacity != primary.opacity;
        m.hframes |= grid.hframes != p_grid.hframes;
        m.vframes |= grid.vframes != p_grid.vframes;
        m.frame |= grid.frame != p_grid.frame;
        m.offset_x |= s.offset[0] != primary.offset[0];
        m.offset_y |= s.offset[1] != primary.offset[1];
        m.region_x |= rect[0] != p_rect[0];
        m.region_y |= rect[1] != p_rect[1];
        m.region_w |= rect[2] != p_rect[2];
        m.region_h |= rect[3] != p_rect[3];
        m.tint |= s.tint != primary.tint;
        m.self_tint |= s.self_tint != primary.self_tint;
        m.per_corner |= corner_tint_of(world, entity) != p_corner;
    }
    m
}

/// A grelha efetiva desta entidade — ausente = uma célula ([`ph2d_ecs::SpriteGrid::SINGLE`]).
fn grid_of(world: &ph2d_ecs::World, entity: ph2d_ecs::Entity) -> ph2d_ecs::SpriteGrid {
    world
        .get::<ph2d_ecs::SpriteGrid>(entity)
        .copied()
        .unwrap_or(ph2d_ecs::SpriteGrid::SINGLE)
}

/// O degradê efetivo — ausente = quatro cantos brancos (identidade).
fn corner_tint_of(world: &ph2d_ecs::World, entity: ph2d_ecs::Entity) -> [[f32; 4]; 4] {
    world
        .get::<ph2d_ecs::SpriteCornerTint>(entity)
        .map_or(ph2d_ecs::SpriteCornerTint::IDENTITY.0, |c| c.0)
}

/// **A divergência de EMISSÃO** — comparada à parte porque ela não vive no `Sprite`.
///
/// ⚠️ `SpriteEmissive` é um componente OPCIONAL, e a sua ausência **é** `EMISSIVE_OFF`: uma sprite
/// sem o componente e outra com `0.0` concordam. Comparar `Option<&SpriteEmissive>` diretamente
/// diria que divergem, e o chip branquearia sobre duas sprites que emitem exatamente o mesmo nada.
fn emissive_of(world: &ph2d_ecs::World, entity: ph2d_ecs::Entity) -> f32 {
    world
        .get::<ph2d_ecs::SpriteEmissive>(entity)
        .map_or(ph2d_ecs::EMISSIVE_OFF, |e| e.clamped())
}

fn compute_emissive_mixed(world: &ph2d_ecs::World, selected: &[u64], primary: f32) -> bool {
    selected
        .iter()
        .any(|&bits| emissive_of(world, ph2d_ecs::Entity::from_bits(bits)) != primary)
}

/// A secção Sprite: o `InspectorSpriteInfo` da seleção primária, com o «Mixed» de uma multi-seleção.
#[allow(clippy::too_many_arguments)]
pub(super) fn sprite_info(
    hero: &HeroScreen,
    sim: &SimWorld,
    inspector_selection: &[u64],
    selected_count: usize,
    atlas_asset_map: &BTreeMap<u32, AssetId>,
    asset_db: &AssetDb,
    renderer: &ph2d_render::SpriteRenderer,
    sheets: &BTreeMap<u32, ph2d_sprite_sheet::AuthoredSheet>,
) -> Option<ph2d_editor_core::InspectorSpriteInfo> {
    hero.gizmo.selection.and_then(|bits| {
        let entity = ph2d_ecs::Entity::from_bits(bits);
        let world = sim.world();
        let sprite = world.get::<Sprite>(entity)?;
        let transform = world.get::<Transform>(entity)?;
        // ⚠️ A emissão é lida pela mesma porta que a comparação usa (`emissive_of`), senão as duas
        // metades — o valor mostrado e a decisão de o mostrar — poderiam discordar sobre o que
        // «ausente» significa.
        let emissive = emissive_of(world, entity);
        // A grelha e a janela desta sprite (ADR-0164 F1 passo 6).
        let grid = grid_of(world, entity);
        let region = world.get::<ph2d_ecs::SpriteRegion>(entity).copied();
        let mut mixed = if inspector_selection.len() > 1 {
            compute_sprite_mixed(world, inspector_selection, bits)
        } else {
            ph2d_editor_core::InspectorSpriteMixed::default()
        };
        if inspector_selection.len() > 1 {
            mixed.emissive = compute_emissive_mixed(world, inspector_selection, emissive);
        }
        let (source_kind, source_pixels, can_reimport) = match sprite.source {
            ph2d_render::SpriteSource::Atlas { key } => {
                // ⚠️ `image_dimensions` e não um `match` na variante — ver o irmão em
                // `inspector_commits.rs`. Aqui a falha silenciosa era o tamanho da sprite sumir do
                // Inspector (plano `docs/Sprite_projeto/18`, auditoria da W2).
                let dims = atlas_asset_map
                    .get(&key)
                    .and_then(|aid| asset_db.get(aid).and_then(|a| a.image_dimensions()));
                (
                    ph2d_editor_core::InspectorSpriteSource::Atlas { key },
                    dims,
                    dims.is_some(),
                )
            }
            ph2d_render::SpriteSource::Individual { texture_id } => {
                // Source dims come from the renderer's individual-texture
                // store (the bake's own size) so the Region UI can show
                // "Source W×H" and seed `region_rect` to the full source —
                // the extract already supports Individual region sampling.
                let dims = renderer.individual().dims(texture_id);
                // ⚠️ **A ESTRATÉGIA é uma pergunta de AUTORIA, não de armazenamento.** No
                // armazenamento um sprite hand-packed É uma textura individual com um retângulo
                // — é essa composição que faz o extract não precisar de saber que ele existe
                // (plano `docs/Sprite_projeto/17` §2.1). Quem sabe de que FOLHA ele é, é o
                // `SpriteSheetRef`, e por isso o painel pergunta ao componente, não ao `source`.
                match world.get::<ph2d_ecs::SpriteSheetRef>(entity) {
                    Some(r) => (
                        ph2d_editor_core::InspectorSpriteSource::HandPacked {
                            sheet: r.sheet,
                            region: r.region,
                        },
                        dims,
                        false,
                    ),
                    None => (
                        ph2d_editor_core::InspectorSpriteSource::Individual { texture_id },
                        dims,
                        // Reimport recomputes world size from an Atlas asset's
                        // px/m; Individual bakes have no atlas asset to re-decode.
                        false,
                    ),
                }
            }
            // W2.T2: a cooked KTX2 source — read-only display marker. Dims
            // come from the W2.T4 loader (logical_id → tier asset); unknown
            // here, so the Region UI shows no "Source W×H" and no reimport.
            ph2d_render::SpriteSource::CookedTexture { .. } => (
                ph2d_editor_core::InspectorSpriteSource::CookedTexture,
                None,
                false,
            ),
        };
        // **ESTAR NUMA FOLHA JÁ É SER HAND-PACKED** (Enio, 2026-08-19: *"ao colocar uma imagem
        // numa sheet, no inspector ainda diz que ela usa a estratégia Individual"*).
        //
        // ⚠️ **O modelo já concordava com ele e o código não seguia** — a nota logo acima diz que
        // *"a estratégia é uma pergunta de AUTORIA, não de armazenamento"*, e a autoria estava a
        // ser lida só do `SpriteSheetRef`, que **nasce no bake**. Entre pôr a peça na folha e
        // assá-la, o painel dizia `Individual` — que é verdade sobre os PIXELS e mentira sobre o
        // que o artista acabou de fazer. *Uma resposta correta à pergunta errada lê-se como um
        // bug, e é.*
        //
        // A autoria passa a ter duas fontes, na ordem em que se tornam verdadeiras: o
        // `SpriteSheetRef` (assado — sabe a região) e, na falta dele, **ser filho de uma folha**
        // (arranjado — ainda não sabe). O rótulo diz qual das duas é.
        // **A precisão MEDIDA, não derivada** (plano `docs/Sprite_projeto/18` W5). O store de
        // texturas é quem sabe: o Inspector recebe o facto pronto, como já recebe o `sheet_label`.
        //
        // ⚠️ Uma célula de atlas é `Rgba8UnormSrgb` por construção; uma individual pode ser
        // qualquer das duas, e é por isso que se **pergunta** em vez de assumir.
        let source_precision = match sprite.source {
            ph2d_render::SpriteSource::Atlas { .. } => Some(ph2d_color::Precision::Rgba8),
            ph2d_render::SpriteSource::Individual { texture_id } => {
                renderer.individual_format(texture_id).map(|f| {
                    if f == ph2d_render::IndividualTextureStore::FORMAT_16 {
                        ph2d_color::Precision::Rgba16
                    } else {
                        ph2d_color::Precision::Rgba8
                    }
                })
            }
            // Cozida: BC/ASTC/ETC2, e o formato concreto depende do tier resolvido.
            ph2d_render::SpriteSource::CookedTexture { .. } => None,
        };
        let unbaked_sheet = if matches!(
            source_kind,
            ph2d_editor_core::InspectorSpriteSource::Individual { .. }
                | ph2d_editor_core::InspectorSpriteSource::Atlas { .. }
        ) {
            world
                .get::<ph2d_ecs::ChildOf>(entity)
                .map(|c| c.parent())
                .filter(|p| world.get::<ph2d_ecs::SpriteSheetFrame>(*p).is_some())
                .map(|p| {
                    world
                        .get::<ph2d_ecs::Name>(p)
                        .map(|n| n.0.clone())
                        .unwrap_or_else(|| {
                            tr("shell.snapshots_inspector_sprite.sprite_sheet").to_string()
                        })
                })
        } else {
            None
        };
        // O rótulo legível de uma origem hand-packed. Derivado AQUI (e não no painel) porque o
        // painel é chrome e não pode depender do documento de folhas sem inverter a seta.
        let baked_label = match source_kind {
            ph2d_editor_core::InspectorSpriteSource::HandPacked { sheet, region } => {
                sheets.get(&sheet).and_then(|s| {
                    s.region(region)
                        .map(|r| format!("{} \u{00b7} {}", s.name, r.name))
                })
            }
            _ => None,
        };
        let (source_kind, sheet_label) =
            sheet_authorship(source_kind, unbaked_sheet.as_deref(), baked_label);
        let world_size = [
            sprite.size[0] * transform.scale.x,
            sprite.size[1] * transform.scale.y,
        ];
        Some(ph2d_editor_core::InspectorSpriteInfo {
            sheet_label,
            entity_bits: bits,
            world_size,
            source_kind,
            source_precision,
            // **Quanto esta sprite emite** (plano `docs/Sprite_projeto/18` W8). Ausente = `0.0`:
            // para o painel, «sem componente» e «componente a zero» são a mesma coisa, e é isso que
            // deixa o slider voltar a zero remover a linha em vez de a deixar morta no ficheiro.
            emissive,
            source_pixels,
            can_reimport,
            flip_x: sprite.flip_x,
            flip_y: sprite.flip_y,
            opacity: sprite.opacity,
            tint_fill: sprite.tint_fill,
            // ⚠️ **Os três do corte (ADR-0164 F1 passo 6) publicam o valor EFETIVO** — ausente =
            // o neutro que o campo v4 tinha. É a mesma lei do `emissive` acima: para o painel,
            // «sem componente» e «componente no neutro» são a mesma coisa, e é isso que deixa a
            // seção desaparecer sem que o widget mude de leitura.
            hframes: grid.hframes,
            vframes: grid.vframes,
            frame: grid.frame,
            tint: sprite.tint,
            self_tint: sprite.self_tint,
            per_corner_tint: corner_tint_of(world, entity),
            // ⭐ A PRESENÇA é o antigo `region_enabled`.
            region_enabled: region.is_some(),
            region_rect: region.map_or([0.0; 4], |r| r.rect),
            region_filter_clip: region.is_some_and(|r| r.filter_clip),
            centered: sprite.centered,
            offset: sprite.offset,
            selected_count,
            mixed,
        })
    })
}
