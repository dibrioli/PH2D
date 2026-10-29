//! ⭐ **O USO de uma imagem num objecto** (planos [GEF 13](../../../docs/GameEngineFuncional/13_isolamento_por_uso_plano.md)
//! D4 e [GEF 15](../../../docs/GameEngineFuncional/15_uso_da_imagem_a_vista_plano.md) D3): parada · repetida (bloco,
//! fundo) · animada · região ([GEF 16](../../../docs/GameEngineFuncional/16_uso_region_plano.md)). O seletor «Use» do
//! Inspector escolhe-o; um uso exclui os outros, e por isso «imagem animada num bloco» deixou de ser um estado que se
//! possa escrever.

use crate::image_choice::{ImageFacts, Slicing, library_facts};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{
    AnimationTag, Entity, ImageRegion, ImageUseLook, ImageUseMemory, ImageUseState, RegionLook,
    RepeatMode, Resource, ScrollRepeat, SpriteAnimCursor, SpriteAnimFrames, SpriteAnimations,
    SpriteAnimator, SpriteGrid, SpritePixels, SpriteRegion, TextureRepeat, TiledDraw, UvTransform,
    World,
};
use ph2d_editor_core::{ImageUseEdit, InspectorDraw, InspectorImageUse};
use ph2d_render::Sprite;
use ph2d_render::texture_of;
use std::collections::BTreeMap;

/// Para que serve a imagem neste objecto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageUse {
    /// Uma imagem (ou uma célula dela) parada.
    Still,
    /// A imagem repete-se: um bloco (*Repeat*, Tiled) ou uma camada de fundo (`ScrollRepeat`).
    Repeated,
    /// As células da imagem tocam como animação.
    Animated,
    /// Só uma área da imagem ([`ImageRegion`] + a área no [`SpriteRegion`]).
    Region,
}

/// O uso de `e`; `None` se não é uma sprite. Animada ganha (as animações mandam no que se desenha), e um
/// animador sem animações já é animada: é o que a secção Animation mostra.
#[must_use]
pub fn image_use(world: &World, e: Entity) -> Option<ImageUse> {
    world.get::<Sprite>(e)?;
    if world.get::<SpriteAnimations>(e).is_some() || world.get::<SpriteAnimator>(e).is_some() {
        return Some(ImageUse::Animated);
    }
    if world.get::<ImageRegion>(e).is_some() {
        return Some(ImageUse::Region);
    }
    let repete = world.get::<ScrollRepeat>(e).is_some()
        || world.get::<TiledDraw>(e).is_some()
        || matches!(
            world.get::<TextureRepeat>(e),
            Some(TextureRepeat(RepeatMode::Enabled | RepeatMode::Mirror))
        );
    Some(if repete {
        ImageUse::Repeated
    } else {
        ImageUse::Still
    })
}

/// Quantas células tem a imagem.
#[must_use]
pub fn cells(facts: ImageFacts) -> u32 {
    facts
        .slicing
        .map_or(1, |s| s.grid[0].max(1) * s.grid[1].max(1))
}

/// O nome da animação que [`make_animated`] cria.
pub const LOOP_TAG: &str = "loop";

/// ⭐ **Passar a animada uma imagem com corte** (planos GEF 13 D5 e 15 D3): o objecto deixa de repetir e toca
/// todas as células da imagem em laço; o tamanho é UMA célula à [`display_scale`] de antes (nada cresce nem
/// encolhe). `false` se a imagem tem uma célula só, ou o objecto já anima.
pub fn make_animated(world: &mut World, e: Entity, facts: ImageFacts) -> bool {
    let Some(slicing) = facts.slicing else {
        return false;
    };
    let n = cells(facts);
    if n < 2 || image_use(world, e).is_none_or(|u| u == ImageUse::Animated) {
        return false;
    }
    let k = display_scale(world, e, facts);
    let mut tags = SpriteAnimations::new();
    if tags.insert(AnimationTag::new(LOOP_TAG, 0, n - 1)).is_err() {
        return false;
    }
    let mut animator = SpriteAnimator::new(LOOP_TAG);
    animator.autoplay = true;
    let grid = [slicing.grid[0].max(1), slicing.grid[1].max(1)];
    show(world, e, facts, grid, k);
    world.entity_mut(e).insert((tags, animator));
    true
}

/// ⭐ **A escala a que `e` mostra a imagem** — metros no ecrã por metro nativo, lidos do que ele MOSTRA (a sua
/// grelha ou a sua área; o ladrilho de um Tiled, de um fundo ou de um UV à mão; senão o tamanho). É o invariante do seletor
/// «Use» (dono 27/10, «mudando entre os modos a imagem ficou gigante»): cada uso deriva o tamanho dela, e por
/// isso nenhuma troca, nem uma volta inteira, muda o tamanho a que a imagem se vê.
#[must_use]
pub fn display_scale(world: &World, e: Entity, facts: ImageFacts) -> [f32; 2] {
    let Some(size) = world.get::<Sprite>(e).map(|s| s.size) else {
        return [1.0; 2];
    };
    let g = world
        .get::<SpriteGrid>(e)
        .copied()
        .unwrap_or(SpriteGrid::SINGLE);
    let grelha = [g.hframes.max(1) as f32, g.vframes.max(1) as f32];
    let area = area_fraction(world, e, facts);
    let uv = world.get::<UvTransform>(e);
    // Por EIXO: uma imagem que o artista achatou continua achatada em qualquer uso.
    std::array::from_fn(|i| {
        let nativo = facts.native_m[i] / grelha[i] * area[i];
        let visto = match (world.get::<TiledDraw>(e), world.get::<ScrollRepeat>(e), uv) {
            (Some(t), _, _) => t.tile[i],
            (_, Some(r), _) if r.tile[i] > 0.0 => r.tile[i],
            (_, _, Some(uv)) if uv.scale[i] > 0.0 => size[i] / uv.scale[i],
            _ => size[i],
        };
        if nativo > 0.0 && visto > 0.0 {
            visto / nativo
        } else {
            1.0
        }
    })
}

/// A fracção da imagem que a área ([`SpriteRegion`]) de `e` mostra, por eixo; `1` sem área ou sem os píxeis.
fn area_fraction(world: &World, e: Entity, facts: ImageFacts) -> [f32; 2] {
    let Some(r) = world.get::<SpriteRegion>(e) else {
        return [1.0; 2];
    };
    std::array::from_fn(|i| {
        let w = r.rect[2 + i];
        if facts.px[i] > 0 && w > 0.0 {
            w / facts.px[i] as f32
        } else {
            1.0
        }
    })
}

/// `e` passa a mostrar a grelha `grid` da imagem `facts`, uma célula à escala `k`, sem repetir.
fn show(world: &mut World, e: Entity, facts: ImageFacts, grid: [u32; 2], k: [f32; 2]) {
    let celula = [
        facts.native_m[0] / grid[0] as f32 * k[0],
        facts.native_m[1] / grid[1] as f32 * k[1],
    ];
    let mut ent = world.entity_mut(e);
    ent.remove::<(TextureRepeat, UvTransform, ScrollRepeat, TiledDraw)>();
    if let Some(mut s) = ent.get_mut::<Sprite>() {
        s.size = celula;
    }
    ent.insert(SpriteGrid {
        hframes: grid[0],
        vframes: grid[1],
        frame: 0,
    });
}

/// Tira a animação de `e` (a lista, o animador e o estado de corrida). As regras «quando → animação» ficam:
/// sem animações dormem, e voltar a animada devolve-as.
fn stop_animating(world: &mut World, e: Entity) {
    world.entity_mut(e).remove::<(
        SpriteAnimations,
        SpriteAnimator,
        SpriteAnimFrames,
        SpriteAnimCursor,
    )>();
}

/// A grelha que a transparência propõe para `rgba` (`w × h`, RGBA8 reto): colunas e linhas transparentes
/// inteiras entre as figuras. `None` sem calhas (uma figura só, ou figuras juntas) — nunca adivinhar.
#[must_use]
pub fn proposed_grid(rgba: &[u8], w: u32, h: u32) -> Option<[u32; 2]> {
    if rgba.len() != (w as usize) * (h as usize) * 4 {
        return None;
    }
    let alpha: Vec<u8> = rgba.as_chunks::<4>().0.iter().map(|p| p[3]).collect();
    let g = ph2d_sprite_sheet::guess_grid(&alpha, w, h)?;
    (g.cols * g.rows > 1).then_some([g.cols, g.rows])
}

/// Os factos de uma sprite sem imagem da biblioteca (uma pintura, um recorte): a grelha dela e o tamanho.
pub fn own_facts(world: &World, e: Entity) -> Option<ImageFacts> {
    let size = world.get::<Sprite>(e)?.size;
    let g = world
        .get::<SpriteGrid>(e)
        .copied()
        .unwrap_or(SpriteGrid::SINGLE);
    let grid = [g.hframes.max(1), g.vframes.max(1)];
    Some(ImageFacts {
        native_m: [size[0] * grid[0] as f32, size[1] * grid[1] as f32],
        px: [0, 0],
        slicing: (grid[0] * grid[1] > 1).then_some(Slicing {
            grid,
            still_frame: g.frame,
        }),
    })
}

/// → Animada. Uma imagem sem corte recebe o que a transparência propõe; sem proposta fica 1 × 1 com a lista
/// vazia (o artista corta) — nenhuma animação inventada.
fn to_animated(
    world: &mut World,
    e: Entity,
    facts: ImageFacts,
    rgba: Option<(u32, u32, &[u8])>,
) -> bool {
    let mut facts = facts;
    if cells(facts) < 2
        && let Some(grid) = rgba.and_then(|(w, h, px)| proposed_grid(px, w, h))
    {
        facts.slicing = Some(Slicing {
            grid,
            still_frame: 0,
        });
    }
    if cells(facts) >= 2 {
        return make_animated(world, e, facts);
    }
    let k = display_scale(world, e, facts);
    show(world, e, facts, [1, 1], k);
    world
        .entity_mut(e)
        .insert((SpriteAnimations::new(), SpriteAnimator::new("")));
    true
}

/// ⭐ **O seletor «Use»** (plano GEF 15 D3): `e` passa a ser usada como `to`. **Cada uso guarda a sua configuração**
/// (dono 27/10) no [`ImageUseMemory`]: o uso que se deixa fica lá tal e qual, e um uso que já foi usado volta como
/// estava (o tamanho, o corte, o ladrilho, as animações). Um uso novo nasce à MESMA [`display_scale`]: parada = a
/// imagem inteira; animada = uma célula; repetida = UM ladrilho (1 × 1) da imagem inteira. O collider automático segue o tamanho.
/// Um passo de desfazer devolve tudo. `false` se já o era, se não é uma sprite, ou se é uma camada de fundo
/// (`ScrollRepeat`, outra lei: repete sempre). `rgba` é a imagem que ela mostra, para propor a grelha.
pub fn set_use(
    world: &mut World,
    e: Entity,
    facts: Option<ImageFacts>,
    rgba: Option<(u32, u32, &[u8])>,
    to: ImageUse,
) -> bool {
    let Some(cur) = image_use(world, e) else {
        return false;
    };
    if cur == to || world.get::<ScrollRepeat>(e).is_some() {
        return false;
    }
    let Some(facts) = facts.or_else(|| own_facts(world, e)) else {
        return false;
    };
    let Some(antes) = world.get::<Sprite>(e).map(|s| s.size) else {
        return false;
    };
    let mut memory = world.get::<ImageUseMemory>(e).cloned().unwrap_or_default();
    let deixado = capture(world, e);
    let regiao_deixada = capture_region(world, e);
    if let Some(destino) = slot(&mut memory, to) {
        let guardado = destino.take();
        match guardado {
            Some(ImageUseState {
                look: Some(look), ..
            }) => restore(world, e, look, guardado.as_ref()),
            _ => {
                if !fresh(world, e, facts, rgba, to) {
                    return false;
                }
                if let Some(st) = guardado.as_ref().filter(|s| s.animations.is_some()) {
                    insert_animation(world, e, st);
                }
            }
        }
    } else {
        match memory.region.take() {
            Some(look) => restore_region(world, e, look),
            None => fresh_region(world, e, facts, rgba),
        }
    }
    if cur == ImageUse::Region {
        // A área é só da Region: as outras mostram a imagem inteira. Sai DEPOIS de o uso novo medir a escala
        // (com ela, a `display_scale` é a da área; sem ela, a da imagem inteira ao tamanho da área).
        world.entity_mut(e).remove::<(ImageRegion, SpriteRegion)>();
    }
    match slot(&mut memory, cur) {
        Some(lugar) => *lugar = Some(deixado),
        None => memory.region = regiao_deixada,
    }
    world.entity_mut(e).insert(memory);
    if let Some(depois) = world.get::<Sprite>(e).map(|s| s.size) {
        crate::tiled::follow_auto_collider(world, e, antes, depois);
    }
    true
}

/// O lugar do uso `u` na memória; `None` para a Region, que guarda um [`RegionLook`] em `memory.region`.
fn slot(memory: &mut ImageUseMemory, u: ImageUse) -> Option<&mut Option<ImageUseState>> {
    match u {
        ImageUse::Still => Some(&mut memory.still),
        ImageUse::Repeated => Some(&mut memory.repeated),
        ImageUse::Animated => Some(&mut memory.animated),
        ImageUse::Region => None,
    }
}

/// O que a Region de agora de `e` tem; `None` se `e` não está em Region.
fn capture_region(world: &World, e: Entity) -> Option<RegionLook> {
    Some(RegionLook {
        size: world.get::<Sprite>(e)?.size,
        area: world.get::<SpriteRegion>(e).copied(),
        cut: *world.get::<ImageRegion>(e)?,
    })
}

/// `e` passa a mostrar só uma área, sem repetir nem animar.
fn enter_region(
    world: &mut World,
    e: Entity,
    size: [f32; 2],
    area: Option<SpriteRegion>,
    cut: ImageRegion,
) {
    stop_animating(world, e);
    let mut ent = world.entity_mut(e);
    ent.remove::<(
        TextureRepeat,
        UvTransform,
        TiledDraw,
        SpriteGrid,
        SpriteRegion,
    )>();
    if let Some(mut s) = ent.get_mut::<Sprite>() {
        s.size = size;
    }
    if let Some(a) = area {
        ent.insert(a);
    }
    ent.insert(cut);
}

/// A Region guardada volta tal e qual.
fn restore_region(world: &mut World, e: Entity, look: RegionLook) {
    enter_region(world, e, look.size, look.area, look.cut);
}

/// Uma Region nova: a imagem INTEIRA à [`display_scale`] de agora (plano GEF 16 D1), e a grelha de snap que a
/// imagem traz ou que a transparência propõe (sem proposta, 1 × 1 = ao píxel).
fn fresh_region(world: &mut World, e: Entity, facts: ImageFacts, rgba: Option<(u32, u32, &[u8])>) {
    let k = display_scale(world, e, facts);
    let grid = facts
        .slicing
        .map(|s| s.grid)
        .or_else(|| rgba.and_then(|(w, h, px)| proposed_grid(px, w, h)))
        .unwrap_or([1, 1]);
    let size = [facts.native_m[0] * k[0], facts.native_m[1] * k[1]];
    enter_region(world, e, size, None, ImageRegion { grid });
}

/// O que o uso de agora de `e` tem.
fn capture(world: &World, e: Entity) -> ImageUseState {
    ImageUseState {
        look: world.get::<Sprite>(e).map(|s| ImageUseLook {
            size: s.size,
            grid: world.get::<SpriteGrid>(e).copied(),
            uv: world.get::<UvTransform>(e).copied(),
            repeat: world.get::<TextureRepeat>(e).copied(),
            tiled: world.get::<TiledDraw>(e).copied(),
        }),
        animations: world.get::<SpriteAnimations>(e).cloned(),
        animator: world.get::<SpriteAnimator>(e).cloned(),
        frames: world.get::<SpriteAnimFrames>(e).cloned(),
        cursor: world.get::<SpriteAnimCursor>(e).copied(),
    }
}

/// `e` volta a ter o aspecto `look` e a animação de `state`, tal e qual.
fn restore(world: &mut World, e: Entity, look: ImageUseLook, state: Option<&ImageUseState>) {
    stop_animating(world, e);
    let mut ent = world.entity_mut(e);
    ent.remove::<(TextureRepeat, UvTransform, TiledDraw, SpriteGrid)>();
    if let Some(mut s) = ent.get_mut::<Sprite>() {
        s.size = look.size;
    }
    if let Some(g) = look.grid {
        ent.insert(g);
    }
    if let Some(uv) = look.uv {
        ent.insert(uv);
    }
    if let Some(r) = look.repeat {
        ent.insert(r);
    }
    if let Some(t) = look.tiled {
        ent.insert(t);
    }
    if let Some(st) = state {
        insert_animation(world, e, st);
    }
}

/// Põe em `e` a animação guardada em `state` (a lista, o animador, os quadros e o passo), no lugar da de agora.
fn insert_animation(world: &mut World, e: Entity, state: &ImageUseState) {
    let mut ent = world.entity_mut(e);
    if let Some(a) = &state.animations {
        ent.insert(a.clone());
    }
    if let Some(a) = &state.animator {
        ent.insert(a.clone());
    }
    if let Some(f) = &state.frames {
        ent.insert(f.clone());
    }
    if let Some(c) = state.cursor {
        ent.insert(c);
    }
}

/// Um uso que `e` ainda não teve (ou cujo aspecto descrevia outra imagem), à [`display_scale`] de agora.
fn fresh(
    world: &mut World,
    e: Entity,
    facts: ImageFacts,
    rgba: Option<(u32, u32, &[u8])>,
    to: ImageUse,
) -> bool {
    let k = display_scale(world, e, facts);
    match to {
        ImageUse::Region => {
            fresh_region(world, e, facts, rgba);
            true
        }
        ImageUse::Animated => to_animated(world, e, facts, rgba),
        ImageUse::Still => {
            stop_animating(world, e);
            show(world, e, facts, [1, 1], k);
            true
        }
        ImageUse::Repeated => {
            // A IMAGEM INTEIRA repete-se, nunca uma célula da folha (dono 27/10): 1 × 1 é a imagem como a parada a mostra.
            let tile = [facts.native_m[0] * k[0], facts.native_m[1] * k[1]];
            stop_animating(world, e);
            let mut ent = world.entity_mut(e);
            ent.remove::<(TextureRepeat, UvTransform, SpriteGrid)>();
            if let Some(mut s) = ent.get_mut::<Sprite>() {
                s.size = tile;
            }
            ent.insert(TiledDraw { tile });
            true
        }
    }
}

/// A imagem da biblioteca que `e` mostra.
fn shown_id(world: &World, e: Entity, atlas: &BTreeMap<u32, AssetId>) -> Option<AssetId> {
    texture_of(world.get::<SpritePixels>(e), world.get::<Sprite>(e), atlas)
}

/// A imagem da biblioteca que `e` mostra e os factos dela (o corte dos utilizadores incluído).
pub fn shown_facts(
    world: &World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
) -> Option<ImageFacts> {
    library_facts(world, shown_id(world, e, atlas)?, db, atlas, ppm)
}

/// ⭐ O que o Inspector publica de `e` (plano GEF 13 D4): o uso e quantas células tem a imagem (sem imagem da
/// biblioteca — uma pintura, um recorte — conta a grelha do objecto).
#[must_use]
pub fn inspector_use(
    world: &World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
) -> (InspectorImageUse, u32, InspectorDraw) {
    let uso = match image_use(world, e) {
        Some(ImageUse::Animated) => InspectorImageUse::Animated,
        Some(ImageUse::Repeated) => InspectorImageUse::Repeated,
        Some(ImageUse::Region) => InspectorImageUse::Region,
        _ => InspectorImageUse::Still,
    };
    let facts = shown_facts(world, e, db, atlas, ppm);
    let n = match facts {
        Some(f) => cells(f),
        None => world
            .get::<SpriteGrid>(e)
            .map_or(1, |g| g.hframes.max(1) * g.vframes.max(1)),
    };
    let draw = if uso == InspectorImageUse::Region {
        InspectorDraw::Region {
            px: region_px(world, e, facts.map_or([0, 0], |f| f.px)),
        }
    } else {
        crate::tiled::draw_of(world, e)
    };
    (uso, n, draw)
}

/// Os píxeis da área que `e` mostra: o [`SpriteRegion`], senão a imagem inteira (`image_px`).
#[must_use]
pub fn region_px(world: &World, e: Entity, image_px: [u32; 2]) -> [u32; 2] {
    world.get::<SpriteRegion>(e).map_or(image_px, |r| {
        [r.rect[2].round() as u32, r.rect[3].round() as u32]
    })
}

/// ⭐ **O OK do editor de Region** (plano GEF 16 D2): `e` passa a mostrar a área `rect` (`[x, y, w, h]`, px da
/// imagem `facts`) com a grelha de snap `grid`, à MESMA [`display_scale`] (nenhuma área muda a escala). A área fica
/// dentro da imagem e com pelo menos 1 px. O collider automático segue. `false` fora de Region ou se nada mudou.
pub fn set_region(
    world: &mut World,
    e: Entity,
    facts: ImageFacts,
    rect: [u32; 4],
    grid: [u32; 2],
) -> bool {
    if image_use(world, e) != Some(ImageUse::Region) || facts.px[0] == 0 || facts.px[1] == 0 {
        return false;
    }
    let Some(antes) = world.get::<Sprite>(e).map(|s| s.size) else {
        return false;
    };
    let rect = clamp_area(rect, facts.px);
    let k = display_scale(world, e, facts);
    let size: [f32; 2] =
        std::array::from_fn(|i| facts.native_m[i] * rect[2 + i] as f32 / facts.px[i] as f32 * k[i]);
    let area = SpriteRegion::for_atlas(rect.map(|v| v as f32));
    let cut = ImageRegion {
        grid: [grid[0].max(1), grid[1].max(1)],
    };
    let w = world;
    if w.get::<SpriteRegion>(e) == Some(&area) && w.get::<ImageRegion>(e) == Some(&cut) {
        return false;
    }
    enter_region(w, e, size, Some(area), cut);
    crate::tiled::follow_auto_collider(w, e, antes, size);
    true
}

/// `rect` dentro de uma imagem de `px` píxeis, com pelo menos 1 px de lado.
#[must_use]
pub fn clamp_area(rect: [u32; 4], px: [u32; 2]) -> [u32; 4] {
    let x = rect[0].min(px[0].saturating_sub(1));
    let y = rect[1].min(px[1].saturating_sub(1));
    [
        x,
        y,
        rect[2].clamp(1, px[0] - x),
        rect[3].clamp(1, px[1] - y),
    ]
}

/// Por onde o editor de Region começa: os píxeis da imagem, a área (`None` = a imagem inteira) e a grelha.
pub type RegionSeed = ([u32; 2], Option<[f32; 4]>, [u32; 2]);

/// Por onde o editor de Region de `e` começa: os píxeis da imagem, a área de agora (`None` = a imagem inteira) e a
/// grelha. `None` fora de Region ou sem os píxeis da imagem.
#[must_use]
pub fn region_seed(
    world: &World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
) -> Option<RegionSeed> {
    let cut = *world.get::<ImageRegion>(e)?;
    let px = shown_facts(world, e, db, atlas, ppm)?.px;
    (px[0] > 0 && px[1] > 0).then(|| (px, world.get::<SpriteRegion>(e).map(|r| r.rect), cut.grid))
}

/// As sprites cujo editor de Region alguém pediu neste quadro («Edit Region», ou uma Region nova).
#[derive(Resource, Default, Debug)]
pub struct RegionEditorRequests(pub Vec<u64>);

/// Pede o editor de Region de `e`.
pub fn request_region_editor(world: &mut World, e: Entity) {
    world
        .get_resource_or_init::<RegionEditorRequests>()
        .0
        .push(e.to_bits());
}

/// ⭐ **Uma edição do uso da imagem que `e` mostra** — o seletor «Use» e Width/Height. `false` se nada mudou.
pub fn apply_edit(
    world: &mut World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
    edit: ImageUseEdit,
) -> bool {
    let facts = shown_facts(world, e, db, atlas, ppm);
    match edit {
        ImageUseEdit::Use(u) => {
            let to = match u {
                InspectorImageUse::Still => ImageUse::Still,
                InspectorImageUse::Repeated => ImageUse::Repeated,
                InspectorImageUse::Animated => ImageUse::Animated,
                InspectorImageUse::Region => ImageUse::Region,
            };
            let asset = shown_id(world, e, atlas).and_then(|id| db.get(&id));
            let px = asset.as_ref().and_then(|a| a.image_rgba8());
            let rgba = px.as_ref().map(|(w, h, p)| (*w, *h, &p[..]));
            let nova = to == ImageUse::Region
                && world
                    .get::<ImageUseMemory>(e)
                    .is_none_or(|m| m.region.is_none());
            let mudou = set_use(world, e, facts, rgba, to);
            // Uma Region nova é a imagem inteira: o editor abre-se sozinho para escolher a área.
            if mudou && nova {
                request_region_editor(world, e);
            }
            mudou
        }
        ImageUseEdit::SetRegion { rect, grid } => match facts {
            Some(f) => set_region(world, e, f, rect, grid),
            None => false,
        },
        ImageUseEdit::EditRegion => {
            if image_use(world, e) == Some(ImageUse::Region) {
                request_region_editor(world, e);
            }
            false
        }
        ImageUseEdit::Width(n) => crate::tiled::set_tiled_side(world, e, 0, n),
        ImageUseEdit::Height(n) => crate::tiled::set_tiled_side(world, e, 1, n),
    }
}

/// ⭐ **O corte é da IMAGEM** (plano GEF 13 D3, o *importer* da Unity): depois de editar colunas/linhas em `e`,
/// todo objecto que mostra a MESMA imagem (a mesma célula do atlas, ou os mesmos píxeis próprios) passa ao
/// mesmo corte; a célula de cada um fica dentro da grelha. Devolve quantos mudaram.
pub fn recut_users(world: &mut World, e: Entity) -> usize {
    let Some(s) = world.get::<Sprite>(e).copied() else {
        return 0;
    };
    let g = world
        .get::<SpriteGrid>(e)
        .copied()
        .unwrap_or(SpriteGrid::SINGLE);
    let px = world.get::<SpritePixels>(e).copied();
    let mut q = world.query::<(Entity, &Sprite, Option<&SpritePixels>, Option<&SpriteGrid>)>();
    let outros: Vec<(Entity, u32)> = q
        .iter(world)
        .filter(|(o, os, opx, _)| {
            *o != e
                && match (px, opx) {
                    (Some(a), Some(b)) => a == **b,
                    (None, None) => {
                        matches!(s.source, ph2d_render::SpriteSource::Atlas { .. })
                            && os.source == s.source
                    }
                    _ => false,
                }
        })
        .filter(|(_, _, _, og)| {
            og.map_or((1, 1), |x| (x.hframes, x.vframes)) != (g.hframes, g.vframes)
        })
        .map(|(o, _, _, og)| (o, og.map_or(0, |x| x.frame)))
        .collect();
    let n = (g.hframes.max(1) * g.vframes.max(1)).max(1);
    for (o, frame) in &outros {
        world.entity_mut(*o).insert(SpriteGrid {
            hframes: g.hframes,
            vframes: g.vframes,
            frame: (*frame).min(n - 1),
        });
    }
    outros.len()
}
