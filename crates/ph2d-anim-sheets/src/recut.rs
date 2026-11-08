//! ⭐ **O corte é da IMAGEM, em qualquer folha** (plano GEF 13 D3 + GEF 26): quem mostra a mesma imagem, como folha `0`
//! ou extra, passa ao mesmo corte. E um corte menor **não mata a animação**: o quadro que sai da grelha nova vira um
//! buraco ([`FrameRef::Empty`]) no lugar, com o alerta e as limpezas de sempre (dono 08/11: nada some sozinho).

use ph2d_asset::AssetId;
use ph2d_ecs::{
    Entity, FrameRef, SpriteAnimFrames, SpriteGrid, SpritePixels, SpriteSheets, World, sheet_cells,
    sheet_cells_of,
};
use ph2d_render::{Sprite, SpriteSource};

/// De onde vêm os píxeis de uma folha, pelos DOIS nomes que a mesma imagem pode ter: o da biblioteca e a célula de
/// átlas. Uma folha `0` de átlas e uma folha extra cuja textura se resolveu para a MESMA célula são a mesma imagem
/// (auditoria 08/11: comparar só um dos nomes deixava o «Cut again» parar entre elas).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Image {
    asset: Option<AssetId>,
    atlas: Option<SpriteSource>,
}

impl Image {
    fn same(self, o: Self) -> bool {
        (self.asset.is_some() && self.asset == o.asset)
            || (self.atlas.is_some() && self.atlas == o.atlas)
    }
}

fn image_of(world: &World, e: Entity, k: u8) -> Option<Image> {
    let atlas = |s: SpriteSource| matches!(s, SpriteSource::Atlas { .. }).then_some(s);
    if k > 0 {
        let asset = world.get::<SpriteSheets>(e)?.sheet(k)?.asset;
        let tex = world
            .get::<ph2d_render::SheetTextures>(e)
            .and_then(|t| t.0.get(usize::from(k) - 1).copied().flatten())
            .filter(|(a, _)| *a == asset)
            .and_then(|(_, s)| atlas(s));
        return Some(Image {
            asset: Some(asset),
            atlas: tex,
        });
    }
    let asset = world.get::<SpritePixels>(e).map(|p| p.0);
    let source = world.get::<Sprite>(e).and_then(|s| atlas(s.source));
    (asset.is_some() || source.is_some()).then_some(Image {
        asset,
        atlas: if asset.is_some() { None } else { source },
    })
}

/// O corte `[colunas, linhas]` da folha `k` (a `0` sem grelha é uma célula).
fn cut_of(world: &World, e: Entity, k: u8) -> Option<[u32; 2]> {
    if k > 0 {
        return world
            .get::<SpriteSheets>(e)?
            .sheet(k)
            .map(|s| [s.hframes, s.vframes]);
    }
    let g = world
        .get::<SpriteGrid>(e)
        .copied()
        .unwrap_or(SpriteGrid::SINGLE);
    Some([g.hframes, g.vframes])
}

fn set_cut(world: &mut World, e: Entity, k: u8, [h, v]: [u32; 2]) {
    if k > 0 {
        if let Some(mut sheets) = world.get::<SpriteSheets>(e).cloned()
            && let Some(s) = sheets.0.get_mut(usize::from(k) - 1)
        {
            (s.hframes, s.vframes) = (h, v);
            world.entity_mut(e).insert(sheets);
        }
        return;
    }
    let mut g = world
        .get::<SpriteGrid>(e)
        .copied()
        .unwrap_or(SpriteGrid::SINGLE);
    (g.hframes, g.vframes) = (h, v);
    world.entity_mut(e).insert(g);
}

/// As listas de `frames` com cada quadro que já não cabe na sua folha (`sheets[j]` = células da folha `j`) trocado por
/// um buraco. `None` = todos cabem.
#[must_use]
pub fn holes_beyond(frames: &SpriteAnimFrames, sheets: &[u32]) -> Option<SpriteAnimFrames> {
    let fora = |f: &FrameRef| !f.fits(sheets);
    if !frames.0.iter().any(|t| t.cells.iter().any(fora)) {
        return None;
    }
    let mut out = frames.clone();
    for t in &mut out.0 {
        for f in &mut t.cells {
            if fora(f) {
                *f = FrameRef::Empty;
            }
        }
    }
    Some(out)
}

/// ⭐ **Prende os quadros de `e` às grelhas de agora**: os de fora viram buraco e a célula na tela volta para dentro da
/// folha dela. Devolve se mudou algo.
pub fn hold_frames(world: &mut World, e: Entity) -> bool {
    let cells = sheet_cells_of(world, e);
    let mut mudou = false;
    if let Some(f) = world
        .get::<SpriteAnimFrames>(e)
        .and_then(|f| holes_beyond(f, &cells))
    {
        world.entity_mut(e).insert(f);
        mudou = true;
    }
    if let Some(mut g) = world.get::<SpriteGrid>(e).copied() {
        let n = sheet_cells(&cells, g.sheet);
        if g.sheet != SpriteGrid::EMPTY_SHEET && n > 0 && g.frame >= n {
            g.frame = n - 1;
            world.entity_mut(e).insert(g);
            mudou = true;
        }
    }
    mudou
}

/// ⭐ **O corte da folha `k` de `e` passa a todos os que mostram a MESMA imagem**, como folha `0` ou extra, e cada um
/// (e o próprio `e`) prende os quadros à grelha nova. Devolve os OUTROS objetos cuja folha `0` mudou de corte (o aviso
/// de corte velho deles é da família).
pub fn recut_users(world: &mut World, e: Entity, k: u8) -> Vec<Entity> {
    hold_frames(world, e);
    let (Some(img), Some(cut)) = (image_of(world, e, k), cut_of(world, e, k)) else {
        return Vec::new();
    };
    let todos: Vec<Entity> = world
        .query_filtered::<Entity, bevy_ecs::query::With<Sprite>>()
        .iter(world)
        .collect();
    let (mut mudou, mut folha0) = (Vec::new(), Vec::new());
    for o in todos {
        let n = world.get::<SpriteSheets>(o).map_or(0, |s| s.0.len());
        for j in 0..=u8::try_from(n).unwrap_or(u8::MAX - 1) {
            let outra = (o, j) != (e, k);
            let mesma = image_of(world, o, j).is_some_and(|i| i.same(img));
            if outra && mesma && cut_of(world, o, j) != Some(cut) {
                set_cut(world, o, j, cut);
                if !mudou.contains(&o) {
                    mudou.push(o);
                }
                if j == 0 {
                    folha0.push(o);
                }
            }
        }
    }
    for o in mudou {
        hold_frames(world, o);
    }
    folha0
}

#[cfg(test)]
#[path = "recut_tests.rs"]
mod tests;
