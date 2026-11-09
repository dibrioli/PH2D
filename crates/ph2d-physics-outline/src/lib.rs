//! **O produtor da forma «Outline»** (W2c, plano `f2_contorno_e_formas_plano.md`, «W2 — desenho
//! fechado»): imagem-fonte → máscara pelo `alpha_cutoff` → `trace_fitted` → `ColliderOutline` e
//! `ColliderShape::Outline` na forma, pela fila do editor (um passo de undo).

use ph2d_ecs::scene::{ComponentRegistry, EditorCommandQueue};
use ph2d_ecs::{
    ChildOf, Entity, FrameRef, SpriteAnimFrames, SpriteAnimations, SpriteGrid, SpriteSheets, World,
};
use ph2d_outline_trace::{Mask, Params, trace_fitted};
use ph2d_physics_ecs::{Collider, ColliderOutline, ColliderShape, RigidBody};
use ph2d_render::Sprite;

/// Os pixels RGBA8 de uma folha, linha a linha.
#[derive(Clone, Debug)]
pub struct Picture<'a> {
    pub w: u32,
    pub h: u32,
    pub rgba: std::borrow::Cow<'a, [u8]>,
}

/// O resultado de um traço: o componente a gravar e se o tecto subiu o Detail (o aviso da W4).
#[derive(Clone, Debug, PartialEq)]
pub struct Traced {
    pub outline: ColliderOutline,
    pub raised: bool,
}

/// **Quem tem a imagem que a forma `e` cobre** (opção B): ela própria, ou o antepassado mais próximo
/// com imagem até ao corpo, inclusive.
#[must_use]
pub fn picture_owner(world: &World, e: Entity) -> Option<Entity> {
    let mut cur = Some(e);
    while let Some(x) = cur {
        if world.get::<Sprite>(x).is_some() {
            return Some(x);
        }
        if world.get::<RigidBody>(x).is_some() {
            return None;
        }
        cur = world.get::<ChildOf>(x).map(ChildOf::parent);
    }
    None
}

/// **Os quadros que o contorno cobre**: os das tags (união, sem repetidos), todas as células sem
/// tags. Uma imagem sem grelha é a célula `0` da folha `0`.
#[must_use]
pub fn traced_frames(world: &World, owner: Entity) -> Vec<(u8, u32)> {
    let sheets = ph2d_ecs::sheet_cells_of(world, owner);
    let lists = world.get::<SpriteAnimFrames>(owner);
    let mut out: Vec<(u8, u32)> = match world.get::<SpriteAnimations>(owner) {
        Some(tags) if !tags.is_empty() => tags
            .iter()
            .filter_map(|t| ph2d_ecs::tag_cells(t, lists.and_then(|l| l.get(&t.name)), &sheets))
            .flatten()
            .filter_map(|f| match f {
                FrameRef::Cell { sheet, cell } => Some((sheet, cell)),
                FrameRef::Empty => None,
            })
            .collect(),
        _ => (0u8..)
            .zip(sheets.iter())
            .flat_map(|(k, &n)| (0..n).map(move |c| (k, c)))
            .collect(),
    };
    out.sort_unstable();
    out.dedup();
    out
}

/// A grelha `[colunas, linhas]` da folha `k` de `owner`.
fn grid_of(world: &World, owner: Entity, k: u8) -> Option<[u32; 2]> {
    if k == 0 {
        let g = world
            .get::<SpriteGrid>(owner)
            .copied()
            .unwrap_or(SpriteGrid::SINGLE);
        return Some([g.hframes.max(1), g.vframes.max(1)]);
    }
    let s = world.get::<SpriteSheets>(owner)?.sheet(k)?;
    Some([s.hframes.max(1), s.vframes.max(1)])
}

/// **A máscara do contorno**: o OR das células `frames`, cada uma recortada da imagem da sua folha
/// (`picture(k)`) e cortada no `cutoff`. `None` sem pixels ou com células de tamanhos diferentes.
pub fn frames_mask<'a>(
    world: &World,
    owner: Entity,
    frames: &[(u8, u32)],
    picture: impl Fn(u8) -> Option<Picture<'a>>,
    cutoff: u8,
) -> Option<Mask> {
    let mut masks = Vec::with_capacity(frames.len());
    for &(k, cell) in frames {
        let (Some(p), Some([cols, rows])) = (picture(k), grid_of(world, owner, k)) else {
            continue;
        };
        if p.rgba.len() < (p.w as usize) * (p.h as usize) * 4 {
            continue;
        }
        // A célula que o quad desenha: a régua do motor (região na folha 0, `floor`/`ceil`).
        let region = (k == 0)
            .then(|| world.get::<ph2d_ecs::SpriteRegion>(owner).map(|r| r.rect))
            .flatten();
        let Some(cells) = ph2d_render::SourceCells::of([p.w, p.h], region, cols, rows) else {
            continue;
        };
        let r = cells.cell_rect_px(cell);
        masks.push(Mask::from_rgba8(r[2], r[3], &crop(&p, r), cutoff));
    }
    Mask::union(&masks)
}

/// Os pixels `[x, y, w, h]` de `p`; o que cai fora da imagem é transparente.
fn crop(p: &Picture<'_>, [x0, y0, w, h]: [u32; 4]) -> Vec<u8> {
    let mut out = vec![0u8; (w as usize) * (h as usize) * 4];
    for y in 0..h.min(p.h.saturating_sub(y0)) {
        let n = w.min(p.w.saturating_sub(x0)) as usize * 4;
        let at = (((y0 + y) * p.w + x0) * 4) as usize;
        let to = (y * w * 4) as usize;
        out[to..to + n].copy_from_slice(&p.rgba[at..at + n]);
    }
    out
}

/// **O traço**: `trace_fitted` com o Detail e o Ignore specks under de `asked`; os anéis e os
/// parâmetros USADOS vão para o componente, e `raised` diz se o tecto os subiu.
#[must_use]
pub fn trace(mask: &Mask, asked: &ColliderOutline) -> Traced {
    let want = Params {
        detail_q4: asked.detail_q4,
        min_island_px: asked.min_island_px,
    };
    let f = trace_fitted(mask, want);
    Traced {
        raised: f.raised(want),
        outline: ColliderOutline {
            rings: f.outline.rings.into(),
            picture_px: [mask.w, mask.h],
            detail_q4: f.params.detail_q4,
            alpha_cutoff: asked.alpha_cutoff,
            min_island_px: f.params.min_island_px,
        },
    }
}

/// **Retraça a forma `shape`** pela imagem do dono ([`picture_owner`]); `picture(owner, k)` dá os
/// pixels da folha `k`. Escreve o `Collider` (forma `Outline` do tamanho da imagem em metros) e o
/// `ColliderOutline` pela fila do editor — um passo de undo. `None` = nada a traçar (sem imagem,
/// sem `Collider`, sem pixels).
pub fn retrace<'a>(
    world: &World,
    shape: Entity,
    picture: impl Fn(Entity, u8) -> Option<Picture<'a>>,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> Option<Traced> {
    let collider = world.get::<Collider>(shape).copied()?;
    let owner = picture_owner(world, shape)?;
    let size = world.get::<Sprite>(owner)?.size;
    let asked = world
        .get::<ColliderOutline>(shape)
        .cloned()
        .unwrap_or_default();
    let frames = traced_frames(world, owner);
    let mask = frames_mask(
        world,
        owner,
        &frames,
        |k| picture(owner, k),
        asked.alpha_cutoff,
    )?;
    let traced = trace(&mask, &asked);
    let mut next = Collider {
        shape: ColliderShape::Outline {
            half_x: size[0] * 0.5,
            half_y: size[1] * 0.5,
        },
        offset: [0.0, 0.0],
        ..collider
    };
    next.sanitize();
    let bits = shape.to_bits();
    use ph2d_inspector_ordering::queue_set;
    queue_set(queue, registry, bits, "ph2d::physics::Collider", &next);
    queue_set(
        queue,
        registry,
        bits,
        "ph2d::physics::ColliderOutline",
        &traced.outline,
    );
    Some(traced)
}

/// **As entradas da app**: depois de aplicadas as edições do cartão, «Outline» escolhido (`Shape(3)`)
/// ou «Fit to picture» numa forma `Outline` retraça-a pelos pixels do `AssetDb` (folha `0` pelo
/// átlas ou `SpritePixels`, folhas extra pelo `SpriteSheets`). Devolve quantas retraçou.
pub fn retrace_edited(
    world: &World,
    edits: &[(u64, ph2d_editor_core::PhysicsFieldEdit)],
    db: &ph2d_asset::AssetDb,
    atlas: &std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> usize {
    use ph2d_editor_core::PhysicsFieldEdit as P;
    let picture = |owner: Entity, k: u8| {
        let id = match k {
            0 => ph2d_render::texture_of_entity(world, owner, atlas)?,
            k => world.get::<SpriteSheets>(owner)?.sheet(k)?.asset,
        };
        let asset = db.get(&id)?;
        let (w, h, px) = asset.image_rgba8()?;
        Some(Picture {
            w,
            h,
            rgba: px.into_owned().into(),
        })
    };
    let mut n = 0;
    for &(bits, edit) in edits {
        if !matches!(edit, P::Shape(3) | P::FitToPicture) {
            continue;
        }
        let pedido = Entity::from_bits(bits);
        // Num corpo, a edição da forma vai à 1.ª forma (a mesma lei do `apply_physics_edit`).
        let shape = match world.get::<RigidBody>(pedido) {
            Some(_) => ph2d_physics_ecs::forma_para_editar(world, pedido).unwrap_or(pedido),
            None => pedido,
        };
        let outline = world
            .get::<Collider>(shape)
            .is_some_and(|c| matches!(c.shape, ColliderShape::Outline { .. }));
        if outline && retrace(world, shape, picture, queue, registry).is_some() {
            n += 1;
        }
    }
    n
}
