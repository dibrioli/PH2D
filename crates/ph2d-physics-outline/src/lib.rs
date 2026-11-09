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

mod merge;
pub use merge::{MergeReport, PendingMerge, merged, outline_edited};

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

/// **Onde o contorno mora no dono** (metros locais, y para cima): a meia-extensão e o centro da UNIÃO
/// dos quads onde as folhas dos quadros se desenham (folha `0` no Pivot resolvido, as extra no delas).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Layout {
    pub half: [f32; 2],
    pub centre: [f32; 2],
}

/// O quad da folha `k` de `owner`: centro e lado, como o renderer o põe.
fn quad_of(world: &World, owner: Entity, k: u8, ppm: f32) -> Option<([f32; 2], [f32; 2])> {
    if k == 0 {
        let s = world.get::<Sprite>(owner)?;
        return Some((s.resolve_anchor(ppm), s.size));
    }
    let s = world.get::<SpriteSheets>(owner)?.sheet(k)?;
    Some((s.anchor, s.size))
}

/// O [`Layout`] dos `frames` de `owner`; `None` sem quad com área.
#[must_use]
pub fn layout(world: &World, owner: Entity, frames: &[(u8, u32)], ppm: f32) -> Option<Layout> {
    let (mut lo, mut hi) = ([f32::INFINITY; 2], [f32::NEG_INFINITY; 2]);
    for &(k, _) in frames {
        let Some((c, s)) = quad_of(world, owner, k, ppm) else {
            continue;
        };
        for a in 0..2 {
            lo[a] = lo[a].min(c[a] - s[a] * 0.5);
            hi[a] = hi[a].max(c[a] + s[a] * 0.5);
        }
    }
    (lo[0] < hi[0] && lo[1] < hi[1]).then(|| Layout {
        half: [0, 1].map(|a| (hi[a] - lo[a]) * 0.5),
        centre: [0, 1].map(|a| (hi[a] + lo[a]) * 0.5),
    })
}

/// A régua das células da folha `k` sobre a imagem `p` (a do motor: região só na folha `0`).
fn cells_of(
    world: &World,
    owner: Entity,
    k: u8,
    p: &Picture<'_>,
) -> Option<ph2d_render::SourceCells> {
    let [cols, rows] = grid_of(world, owner, k)?;
    let region = (k == 0)
        .then(|| world.get::<ph2d_ecs::SpriteRegion>(owner).map(|r| r.rect))
        .flatten();
    ph2d_render::SourceCells::of([p.w, p.h], region, cols, rows)
}

/// **A máscara do contorno**: o OR das células `frames` cortadas no `cutoff`, cada uma reamostrada
/// (vizinho mais próximo) para o quad onde se DESENHA, nos pixels da folha `0`. Os limites são os do
/// [`Layout`], que cobrem a arte de todas as folhas — nada se corta. `None` sem pixels.
pub fn frames_mask<'a>(
    world: &World,
    owner: Entity,
    frames: &[(u8, u32)],
    picture: impl Fn(u8) -> Option<Picture<'a>>,
    cutoff: u8,
    ppm: f32,
) -> Option<(Mask, Layout)> {
    let lay = layout(world, owner, frames, ppm)?;
    // A resolução: px por metro da célula da folha `0` (sem imagem na `0`, a da 1.ª folha usada).
    let res_of = |k: u8| {
        let p = picture(k)?;
        let r = cells_of(world, owner, k, &p)?.cell_rect_px(0);
        let (_, s) = quad_of(world, owner, k, ppm)?;
        Some([
            f64::from(r[2]) / f64::from(s[0]),
            f64::from(r[3]) / f64::from(s[1]),
        ])
    };
    let res = res_of(0).or_else(|| frames.first().and_then(|&(k, _)| res_of(k)))?;
    let n = |m: f32, r: f64| (f64::from(m) * 2.0 * r - 1e-3).ceil().max(1.0) as u32;
    let (w, h) = (n(lay.half[0], res[0]), n(lay.half[1], res[1]));
    let left = f64::from(lay.centre[0] - lay.half[0]);
    let top = f64::from(lay.centre[1] + lay.half[1]);
    let span = |o: f64, d: f64, n: u32| (o.floor().max(0.0) as u32)..((o + d).ceil() as u32).min(n);
    let mut bits = vec![0u8; (w as usize) * (h as usize)];
    let mut any = false;
    for &(k, cell) in frames {
        let (Some(p), Some((c, s))) = (picture(k), quad_of(world, owner, k, ppm)) else {
            continue;
        };
        if p.rgba.len() < (p.w as usize) * (p.h as usize) * 4 {
            continue;
        }
        let Some(r) = cells_of(world, owner, k, &p).map(|cs| cs.cell_rect_px(cell)) else {
            continue;
        };
        any = true;
        let x0 = (f64::from(c[0] - s[0] * 0.5) - left) * res[0];
        let y0 = (top - f64::from(c[1] + s[1] * 0.5)) * res[1];
        let (dw, dh) = (f64::from(s[0]) * res[0], f64::from(s[1]) * res[1]);
        for my in span(y0, dh, h) {
            let v = (f64::from(my) + 0.5 - y0) / dh;
            if !(0.0..1.0).contains(&v) {
                continue;
            }
            let sy = r[1] + ((v * f64::from(r[3])) as u32).min(r[3] - 1);
            for mx in span(x0, dw, w) {
                let u = (f64::from(mx) + 0.5 - x0) / dw;
                if !(0.0..1.0).contains(&u) {
                    continue;
                }
                let sx = r[0] + ((u * f64::from(r[2])) as u32).min(r[2] - 1);
                let a = ((sy as usize) * (p.w as usize) + sx as usize) * 4 + 3;
                if sx < p.w && sy < p.h && p.rgba[a] >= cutoff {
                    bits[(my * w + mx) as usize] = 1;
                }
            }
        }
    }
    any.then_some((Mask { w, h, bits }, lay))
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
    let up = f.params.detail_q4 > want.detail_q4;
    Traced {
        raised: f.raised(want),
        outline: ColliderOutline {
            rings: f.outline.rings.into(),
            picture_px: [mask.w, mask.h],
            detail_q4: asked.detail_q4,
            alpha_cutoff: asked.alpha_cutoff,
            min_island_px: asked.min_island_px,
            raised_q4: if up { f.params.detail_q4 } else { 0 },
            custom: false,
        },
    }
}

/// O pedido de `cur` com a edição `e` do cartão (px → Q4, `0..=1` → byte).
#[must_use]
pub fn asked_with(cur: &ColliderOutline, e: ph2d_editor_core::OutlineEdit) -> ColliderOutline {
    use ph2d_editor_core::OutlineEdit as E;
    let to_u16 = |v: f32| v.round().clamp(0.0, f32::from(u16::MAX)) as u16;
    let mut next = cur.clone();
    match e {
        E::Detail(px) => next.detail_q4 = to_u16(px * 16.0).max(1),
        E::AlphaCutoff(v) => next.alpha_cutoff = (v.clamp(0.0, 1.0) * 255.0).round() as u8,
        E::Specks(px) => next.min_island_px = to_u16(px),
        E::Merge(_) => {}
    }
    next
}

/// **«Follows the picture»** da forma `shape` (`None` sem imagem): a caixa compara-se à imagem do
/// dono; o «Outline», à UNIÃO dos quadros traçados ([`OutlineSeen`]), nunca só à folha `0`.
#[must_use]
pub fn follows(world: &World, shape: Entity) -> Option<bool> {
    let owner = picture_owner(world, shape)?;
    let c = world.get::<Collider>(shape)?;
    let size = world.get::<Sprite>(owner)?.size;
    let same = |a: f32, b: f32| (a - b).abs() <= 1e-4 * b.abs().max(1.0);
    if world
        .get::<ColliderOutline>(shape)
        .is_some_and(|o| o.custom)
    {
        return Some(false);
    }
    Some(match (c.shape, world.get::<OutlineSeen>(shape)) {
        (ColliderShape::Outline { half_x, half_y }, Some(s)) => {
            same(half_x, s.layout.half[0]) && same(half_y, s.layout.half[1])
        }
        _ => c.follows_picture(size),
    })
}

/// O que o cartão mostra de um «Outline» (`None` noutra forma). Sem imagem a traçar, `traced` é falso: aí Detail,
/// Alpha cutoff e Ignore specks under não mudariam nada — controlos mortos —, mas o «Merge» serve.
#[must_use]
pub fn card_info(world: &World, shape: Entity) -> Option<ph2d_editor_core::InspectorOutlineInfo> {
    let c = world.get::<Collider>(shape)?;
    if !matches!(c.shape, ColliderShape::Outline { .. }) {
        return None;
    }
    let o = world
        .get::<ColliderOutline>(shape)
        .cloned()
        .unwrap_or_default();
    let max = u32::try_from(ph2d_outline_trace::MAX_PIECES).unwrap_or(u32::MAX);
    Some(ph2d_editor_core::InspectorOutlineInfo {
        detail_px: f32::from(o.detail_q4) / 16.0,
        // O u8 guardado como fração de 2 casas (128 → 0,5, o padrão do dono); ida e volta estáveis.
        alpha_cutoff: (f32::from(o.alpha_cutoff) / 2.55).round() / 100.0,
        specks_px: f32::from(o.min_island_px),
        raised: (o.raised_q4 > o.detail_q4).then(|| (f32::from(o.raised_q4) / 16.0, max)),
        traced: picture_owner(world, shape).is_some(),
        merged: merged(world, shape, &o),
    })
}

/// **Retraça a forma `shape`** pela imagem do dono ([`picture_owner`]); `picture(owner, k)` dá os
/// pixels da folha `k`. Escreve o `Collider` (forma `Outline` do [`Layout`]: meia-extensão e desvio)
/// e o `ColliderOutline` pela fila do editor — um passo de undo; nada se o traço já é o gravado.
/// `None` = nada a traçar (sem imagem, sem `Collider`, sem pixels).
pub fn retrace<'a>(
    world: &World,
    shape: Entity,
    picture: impl Fn(Entity, u8) -> Option<Picture<'a>>,
    ppm: f32,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> Option<Traced> {
    let asked = world
        .get::<ColliderOutline>(shape)
        .cloned()
        .unwrap_or_default();
    retrace_asked(world, shape, &asked, picture, ppm, queue, registry)
}

/// [`retrace`] com o pedido `asked` (Detail, Alpha cutoff, Ignore specks under) no lugar do gravado.
pub fn retrace_asked<'a>(
    world: &World,
    shape: Entity,
    asked: &ColliderOutline,
    picture: impl Fn(Entity, u8) -> Option<Picture<'a>>,
    ppm: f32,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> Option<Traced> {
    retrace_sized(world, shape, asked, picture, ppm, queue, registry, false)
}

/// [`retrace_asked`]; `keep` = o «Custom size» fica (caixa e desvio): os anéis, normalizados à
/// imagem, esticam-se à meia-extensão de agora.
#[allow(clippy::too_many_arguments)]
fn retrace_sized<'a>(
    world: &World,
    shape: Entity,
    asked: &ColliderOutline,
    picture: impl Fn(Entity, u8) -> Option<Picture<'a>>,
    ppm: f32,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
    keep: bool,
) -> Option<Traced> {
    let collider = world.get::<Collider>(shape).copied()?;
    let owner = picture_owner(world, shape)?;
    let frames = traced_frames(world, owner);
    let (mask, lay) = frames_mask(
        world,
        owner,
        &frames,
        |k| picture(owner, k),
        asked.alpha_cutoff,
        ppm,
    )?;
    let traced = trace(&mask, asked);
    let mut next = if keep {
        collider
    } else {
        Collider {
            shape: ColliderShape::Outline {
                half_x: lay.half[0],
                half_y: lay.half[1],
            },
            offset: lay.centre,
            ..collider
        }
    };
    next.sanitize();
    if next == collider && world.get::<ColliderOutline>(shape) == Some(&traced.outline) {
        return Some(traced);
    }
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

/// Os pixels da folha `k` de `owner` no `AssetDb` (folha `0` pelo átlas ou `SpritePixels`).
fn db_picture<'w>(
    world: &'w World,
    db: &'w ph2d_asset::AssetDb,
    atlas: &'w std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
) -> impl Fn(Entity, u8) -> Option<Picture<'static>> + 'w {
    move |owner, k| {
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
    }
}

/// **As entradas da app**: depois de aplicadas as edições do cartão, «Outline» escolhido (`Shape(3)`)
/// ou «Fit to picture» numa forma `Outline` retraça-a pelos pixels do `AssetDb`. Devolve quantas.
pub fn retrace_edited(
    world: &World,
    edits: &[(u64, ph2d_editor_core::PhysicsFieldEdit)],
    db: &ph2d_asset::AssetDb,
    atlas: &std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
    ppm: f32,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> usize {
    use ph2d_editor_core::PhysicsFieldEdit as P;
    let picture = db_picture(world, db, atlas);
    let mut n = 0;
    for &(bits, edit) in edits {
        if !matches!(edit, P::Shape(3) | P::FitToPicture | P::Outline(_))
            || matches!(edit, P::Outline(ph2d_editor_core::OutlineEdit::Merge(_)))
        {
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
        let cur = world.get::<ColliderOutline>(shape).cloned();
        let asked = match edit {
            P::Outline(e) => asked_with(&cur.unwrap_or_default(), e),
            _ => cur.unwrap_or_default(),
        };
        // Só o pedido mudou: o «Custom size» fica; escolher «Outline» ou «Fit» mede a imagem.
        let keep = matches!(edit, P::Outline(_)) && follows(world, shape) == Some(false);
        if outline
            && retrace_sized(world, shape, &asked, &picture, ppm, queue, registry, keep).is_some()
        {
            n += 1;
        }
    }
    n
}

/// **O que o contorno viu da imagem** da última vez: a impressão digital ([`picture_key`]) e o
/// [`Layout`] dela. Estado de sessão, como o `ShownSheet`: NÃO registado — fora do gravar e do undo.
#[derive(bevy_ecs::component::Component, Clone, Copy, Debug, PartialEq)]
pub struct OutlineSeen {
    pub key: u64,
    pub layout: Layout,
    /// O pedido (Detail, Alpha cutoff, Ignore specks under) visto: a curva que o muda retraça.
    pub asked: [u16; 3],
}

/// **A impressão digital da imagem** que o contorno de `owner` cobre: as imagens (o `AssetId` é o
/// hash do conteúdo: pixels editados mudam-no), grelhas, região, quadros das tags, lados e Pivots.
#[must_use]
pub fn picture_key(
    world: &World,
    owner: Entity,
    atlas: &std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
    ppm: f32,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::hash::DefaultHasher::new();
    let bits = |v: &[f32], h: &mut std::hash::DefaultHasher| {
        v.iter().for_each(|x| x.to_bits().hash(h));
    };
    ph2d_render::texture_of_entity(world, owner, atlas).hash(&mut h);
    traced_frames(world, owner).hash(&mut h);
    if let Some(s) = world.get::<Sprite>(owner) {
        bits(&s.size, &mut h);
        bits(&s.resolve_anchor(ppm), &mut h);
    }
    grid_of(world, owner, 0).hash(&mut h);
    if let Some(r) = world.get::<ph2d_ecs::SpriteRegion>(owner) {
        bits(&r.rect, &mut h);
    }
    for s in world
        .get::<SpriteSheets>(owner)
        .map_or(&[][..], |s| &s.0[..])
    {
        s.asset.hash(&mut h);
        bits(&s.size, &mut h);
        bits(&s.anchor, &mut h);
        [s.hframes, s.vframes].hash(&mut h);
    }
    h.finish()
}

/// **Retraça os contornos que SEGUEM a imagem quando ela muda** (troca, folhas, grelha, tags,
/// pixels): chamar depois das edições do quadro, para o traço cair no MESMO passo de undo. Segue = a
/// forma ainda é a do [`Layout`] visto, ou a que o `follow_auto_collider` já ajustou; o «Custom
/// size» não segue e fica. O PEDIDO mudado (a curva do Detail, o Undo) retraça sempre. A 1.ª vista só
/// anota. Devolve quantos retraçou.
pub fn retrace_followed(
    world: &mut World,
    db: &ph2d_asset::AssetDb,
    atlas: &std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
    ppm: f32,
    queue: &EditorCommandQueue,
    registry: &ComponentRegistry,
) -> usize {
    let same = |a: f32, b: f32| (a - b).abs() <= 1e-4 * b.abs().max(1.0);
    let near = |h: [f32; 2], l: &Layout| same(h[0], l.half[0]) && same(h[1], l.half[1]);
    let mut q = world.query::<(Entity, &Collider)>();
    let shapes: Vec<(Entity, Collider)> = q
        .iter(world)
        .filter(|(_, c)| matches!(c.shape, ColliderShape::Outline { .. }))
        .map(|(e, c)| (e, *c))
        .collect();
    let mut seen = Vec::with_capacity(shapes.len());
    let mut n = merge::pending(world, queue, registry);
    let w: &World = world;
    let picture = db_picture(w, db, atlas);
    for (shape, c) in shapes {
        let Some(owner) = picture_owner(w, shape) else {
            continue;
        };
        let key = picture_key(w, owner, atlas, ppm);
        let lay = layout(w, owner, &traced_frames(w, owner), ppm).unwrap_or_default();
        let before = w.get::<OutlineSeen>(shape).copied();
        let o = w.get::<ColliderOutline>(shape).cloned().unwrap_or_default();
        let asked = [o.detail_q4, u16::from(o.alpha_cutoff), o.min_island_px];
        if before.is_some_and(|b| b.key == key && b.asked == asked) {
            continue;
        }
        // Um «Outline» ainda sem traço (cena montada à mão) traça-se à 1.ª vista.
        if before.is_none()
            && o.rings.is_empty()
            && retrace(w, shape, &picture, ppm, queue, registry).is_some()
        {
            n += 1;
        }
        if let (Some(b), ColliderShape::Outline { half_x, half_y }) = (before, c.shape) {
            let size = w.get::<Sprite>(owner).map(|s| s.size);
            let follows = !o.custom
                && (near([half_x, half_y], &b.layout)
                    || near([half_x, half_y], &lay)
                    || size.is_some_and(|s| c.follows_picture(s)));
            // O pedido mudado retraça sempre; o «Custom size» guarda a caixa.
            let keep = !follows;
            if (follows || b.asked != asked)
                && retrace_sized(w, shape, &o, &picture, ppm, queue, registry, keep).is_some()
            {
                n += 1;
            }
        }
        seen.push((
            shape,
            OutlineSeen {
                key,
                layout: lay,
                asked,
            },
        ));
    }
    drop(picture);
    for (shape, s) in seen {
        world.entity_mut(shape).insert(s);
    }
    n
}
