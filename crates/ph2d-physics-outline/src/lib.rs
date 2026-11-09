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
#[derive(Clone, Copy, Debug)]
pub struct Picture<'a> {
    pub w: u32,
    pub h: u32,
    pub rgba: &'a [u8],
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
        let (cw, ch) = (p.w / cols, p.h / rows);
        if cw == 0 || ch == 0 || p.rgba.len() < (p.w * p.h * 4) as usize {
            continue;
        }
        let (x0, y0) = ((cell % cols) * cw, (cell / cols).min(rows - 1) * ch);
        let mut rgba = Vec::with_capacity((cw * ch * 4) as usize);
        for y in y0..y0 + ch {
            let at = ((y * p.w + x0) * 4) as usize;
            rgba.extend_from_slice(&p.rgba[at..at + (cw * 4) as usize]);
        }
        masks.push(Mask::from_rgba8(cw, ch, &rgba, cutoff));
    }
    Mask::union(&masks)
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
