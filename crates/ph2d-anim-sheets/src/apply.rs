//! ⭐ **As edições das folhas no mundo** (plano GEF 26 §3.3, F10/F11): o dropdown, Load/Delete sheet e as duas
//! limpezas do alerta. As leis são as de [`crate::ops`]; aqui só se lê e escreve os componentes do objeto.

use crate::ops::{self, FirstSheet, NewSheetImage};
use ph2d_asset::{AssetDb, AssetId};
use ph2d_ecs::{
    Entity, SHEETS_MAX, ShownSheet, SpriteAnimCursor, SpriteAnimFrames, SpriteAnimMarkers,
    SpriteAnimations, SpriteAnimator, SpriteGrid, SpritePixels, SpriteSheets, World,
    sheet_cells_of, shown_sheet,
};
use ph2d_editor_core::ImageUseEdit;
use ph2d_render::{SheetTextures, Sprite, SpriteSource, factory_images, texture_of};
use std::collections::BTreeMap;

/// ⭐ Uma edição das folhas de `e`. `None` = não é uma edição das folhas; `Some(mudou)` senão.
pub fn apply(
    world: &mut World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
    edit: ImageUseEdit,
) -> Option<bool> {
    Some(match edit {
        ImageUseEdit::ShowSheet(k) => show_sheet(world, e, k),
        // O painel de Assets abre-se no shell (ou ele diz porque não, [`load_sheet_refusal`]); a resposta volta como
        // `AddSheet`.
        ImageUseEdit::LoadSheet => false,
        ImageUseEdit::AddSheet(id) => add_sheet(world, e, db, atlas, ppm, AssetId::from_digest(id)),
        ImageUseEdit::DeleteSheet => delete_shown_sheet(world, e, atlas),
        ImageUseEdit::RemoveEmptyFrames => remove_empty_frames(world, e),
        ImageUseEdit::DeleteEmptyAnimations => delete_empty_animations(world, e),
        _ => return None,
    })
}

/// ⭐ **Porque o Load sheet não abre** o painel de Assets para `e` (auditoria 08/11: ele calava-se), como chave da
/// tabela de textos; `None` = abre. `has_assets_panel` = há janela do editor para o abrir.
#[must_use]
pub fn load_sheet_refusal(
    world: &World,
    e: Entity,
    has_assets_panel: bool,
) -> Option<&'static str> {
    if !has_assets_panel {
        Some("card.insp.sheet_load_no_assets")
    } else if ph2d_sprite_edit::vazio::sem_imagem(world, e) {
        Some("card.insp.sheet_load_no_image")
    } else if sheet_count(world, e) >= SHEETS_MAX {
        Some("card.insp.sheet_load_full")
    } else {
        None
    }
}

/// Quantas folhas `e` tem, a `0` incluída.
fn sheet_count(world: &World, e: Entity) -> usize {
    1 + world.get::<SpriteSheets>(e).map_or(0, |s| s.0.len())
}

/// A folha `k` passa a ser a visível do painel (estado de UI).
pub fn show_sheet(world: &mut World, e: Entity, k: u8) -> bool {
    if usize::from(k) >= sheet_count(world, e) {
        return false;
    }
    world.entity_mut(e).insert(ShownSheet(k));
    false
}

/// **Cols/Rows** com uma folha EXTRA visível (`k ≥ 1`): o corte dela passa a `h × v`. Com a folha `0` visível devolve
/// `false` e nada muda (o corte do `SpriteGrid` é a porta de sempre, `AnimFieldEdit::Grid`).
pub fn set_shown_grid(world: &mut World, e: Entity, h: u32, v: u32) -> bool {
    let k = shown_sheet(world, e);
    let Some(mut sheets) = world.get::<SpriteSheets>(e).cloned().filter(|_| k > 0) else {
        return false;
    };
    let Some(s) = usize::from(k)
        .checked_sub(1)
        .and_then(|i| sheets.0.get_mut(i))
    else {
        return false;
    };
    (s.hframes, s.vframes) = (h.max(1), v.max(1));
    world.entity_mut(e).insert(sheets);
    true
}

/// Os píxeis de uma imagem da biblioteca: `[w, h]` e o RGBA (vazio numa de fábrica sem píxeis no `AssetDb`).
fn pixels_of(
    id: AssetId,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
) -> Option<([u32; 2], Vec<u8>)> {
    if let Some((w, h, px)) = db
        .get(&id)
        .and_then(|a| a.image_rgba8().map(|(w, h, p)| (w, h, p.into_owned())))
    {
        return Some(([w, h], px));
    }
    let art = atlas
        .iter()
        .find(|(_, a)| **a == id)
        .and_then(|(k, _)| factory_images::art_of_key(*k))?;
    Some((art.image_px(), Vec::new()))
}

/// O corte de uma imagem nova: o da fábrica, senão o de quem já a mostra, senão o que a transparência propõe
/// (a porta da 1.ª imagem de um Animated vazio, `image_choice::choose_library_image`).
fn proposed_cut(
    world: &World,
    id: AssetId,
    px: [u32; 2],
    rgba: &[u8],
    atlas: &BTreeMap<u32, AssetId>,
) -> [u32; 2] {
    if let Some(art) = atlas
        .iter()
        .find(|(_, a)| **a == id)
        .and_then(|(k, _)| factory_images::art_of_key(*k))
    {
        return art.frames();
    }
    let usado = world
        .try_query::<(Option<&SpritePixels>, &Sprite, &SpriteGrid)>()
        .and_then(|mut q| {
            q.iter(world)
                .filter(|(_, _, g)| g.cells() > 1)
                .find(|(p, s, _)| texture_of(*p, Some(*s), atlas) == Some(id))
                .map(|(_, _, g)| [g.hframes.max(1), g.vframes.max(1)])
        });
    usado.or_else(|| guess(rgba, px)).unwrap_or([1, 1])
}

/// A grelha que a transparência propõe (`image_use::proposed_grid`).
fn guess(rgba: &[u8], px: [u32; 2]) -> Option<[u32; 2]> {
    if rgba.len() != (px[0] as usize) * (px[1] as usize) * 4 {
        return None;
    }
    let alpha: Vec<u8> = rgba.as_chunks::<4>().0.iter().map(|p| p[3]).collect();
    let g = ph2d_sprite_sheet::guess_grid(&alpha, px[0], px[1])?;
    (g.cols * g.rows > 1).then_some([g.cols, g.rows])
}

/// A folha `0` como a lei do §3.2 a lê.
fn first_sheet(
    world: &World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
) -> Option<FirstSheet> {
    let s = world.get::<Sprite>(e)?;
    let id = texture_of(world.get::<SpritePixels>(e), Some(s), atlas)?;
    let (px, rgba) = pixels_of(id, db, atlas)?;
    let g = world
        .get::<SpriteGrid>(e)
        .copied()
        .unwrap_or(SpriteGrid::SINGLE);
    let grid = [g.hframes.max(1), g.vframes.max(1)];
    let a = s.resolve_anchor(ppm);
    let opaque = (rgba.len() == (px[0] as usize) * (px[1] as usize) * 4)
        .then(|| ph2d_sprite_edit::pivot::fracao_opaca(&rgba, px[0], [0, 0, px[0], px[1]], grid))
        .flatten();
    Some(FirstSheet {
        size: s.size,
        pivot: [-a[0], -a[1]],
        flip: [s.flip_x, s.flip_y],
        cell_px: [px[0] / grid[0], px[1] / grid[1]],
        opaque,
    })
}

/// ⭐ **Load sheet** (decisão 7): a folha nova entra no fim e fica visível; NENHUMA animação nasce. Num Animated
/// sem imagem a 1.ª entra pela porta de sempre (`ChooseImage`), não por aqui.
pub fn add_sheet(
    world: &mut World,
    e: Entity,
    db: &AssetDb,
    atlas: &BTreeMap<u32, AssetId>,
    ppm: f32,
    id: AssetId,
) -> bool {
    let n = sheet_count(world, e);
    if n >= SHEETS_MAX || ph2d_sprite_edit::vazio::sem_imagem(world, e) {
        return false;
    }
    let (Some(first), Some((px, mut rgba))) = (
        first_sheet(world, e, db, atlas, ppm),
        pixels_of(id, db, atlas),
    ) else {
        return false;
    };
    let grid = proposed_cut(world, id, px, &rgba, atlas);
    let id = straight_id(world, db, id, px, &mut rgba);
    let sheet = ops::new_sheet(
        &first,
        NewSheetImage {
            asset: id,
            rgba: &rgba,
            px,
            grid,
        },
    );
    let mut sheets = world.get::<SpriteSheets>(e).cloned().unwrap_or_default();
    sheets.0.push(sheet);
    world.entity_mut(e).insert((sheets, ShownSheet(n as u8)));
    true
}

/// ⭐ A folha extra desenha-se SEMPRE em alfa recto (`sheet_sprite`), mas um recurso da biblioteca guarda os bytes
/// pré-multiplicados quando uma sprite o mostra assim (o BG removal): o flag vive só no `Sprite`. Então a folha
/// leva uma cópia RECTA, com id próprio; senão o id de sempre (auditoria do fecho 08/11, item D).
fn straight_id(
    world: &mut World,
    db: &AssetDb,
    id: AssetId,
    px: [u32; 2],
    rgba: &mut [u8],
) -> AssetId {
    let pre = !rgba.is_empty()
        && world
            .try_query::<(&SpritePixels, &Sprite)>()
            .is_some_and(|mut q| q.iter(world).any(|(p, s)| p.0 == id && s.premultiplied));
    if !pre {
        return id;
    }
    ph2d_render::unpremultiply_rgba8(rgba);
    db.insert_image_rgba8(px[0], px[1], rgba.to_vec())
}

/// O quadro na tela depois de apagar a folha `k`.
fn grid_after(world: &mut World, e: Entity, k: u8) {
    if let Some(mut g) = world.get_mut::<SpriteGrid>(e) {
        let f = ops::after_deleting(g.shown(), k);
        g.show(f);
    }
}

/// A folha `1` passa a ser a `0`: o `Sprite`, a grelha e os píxeis dela, desenhada como já se desenhava
/// (`ph2d_render::sheet_sprite`). `false` sem a textura dela resolvida.
fn promote_first_extra(world: &mut World, e: Entity, atlas: &BTreeMap<u32, AssetId>) -> bool {
    let Some(x) = world
        .get::<SpriteSheets>(e)
        .and_then(|s| s.0.first())
        .copied()
    else {
        return false;
    };
    let resolvida = world
        .get::<SheetTextures>(e)
        .and_then(|t| t.0.first().copied().flatten())
        .filter(|(a, _)| *a == x.asset)
        .map(|(_, src)| src);
    let atlas_key = atlas.iter().find(|(_, a)| **a == x.asset).map(|(k, _)| *k);
    let Some(source) = atlas_key
        .map(|key| SpriteSource::Atlas { key })
        .or(resolvida)
    else {
        return false;
    };
    let Some(s) = world.get::<Sprite>(e).copied() else {
        return false;
    };
    let nova = Sprite {
        source,
        ..ph2d_render::sheet_sprite(&x, &s)
    };
    let mut ent = world.entity_mut(e);
    ent.insert(nova);
    if atlas_key.is_some() {
        ent.remove::<SpritePixels>();
    } else {
        ent.insert(SpritePixels(x.asset));
    }
    if let Some(mut g) = ent.get_mut::<SpriteGrid>() {
        g.hframes = x.hframes;
        g.vframes = x.vframes;
    }
    true
}

/// ⭐ **Delete sheet** (decisão 6): apaga a folha visível; os quadros dela ficam VAZIOS no lugar, as seguintes descem
/// uma. A `0` apagada promove a `1`. Só com ≥ 2 folhas.
pub fn delete_shown_sheet(world: &mut World, e: Entity, atlas: &BTreeMap<u32, AssetId>) -> bool {
    if sheet_count(world, e) < 2 {
        return false;
    }
    let k = shown_sheet(world, e);
    let cells = sheet_cells_of(world, e);
    if k == 0 && !promote_first_extra(world, e, atlas) {
        return false;
    }
    let lib = world.get::<SpriteAnimations>(e).cloned();
    let mut frames = world
        .get::<SpriteAnimFrames>(e)
        .cloned()
        .unwrap_or_default();
    if let Some(mut lib) = lib {
        ops::delete_sheet(&mut lib, &mut frames, &cells, k);
        world.entity_mut(e).insert(lib);
        put_frames(world, e, frames);
    }
    let mut sheets = world.get::<SpriteSheets>(e).cloned().unwrap_or_default();
    sheets.0.remove(usize::from(k.max(1)) - 1);
    grid_after(world, e, k);
    let mut ent = world.entity_mut(e);
    if sheets.0.is_empty() {
        ent.remove::<SpriteSheets>();
    } else {
        ent.insert(sheets);
    }
    ent.insert(ShownSheet(k.saturating_sub(1)));
    true
}

/// As listas de quadros de volta; vazias tiram o componente e o cursor (como `card::follow_remove`).
fn put_frames(world: &mut World, e: Entity, frames: SpriteAnimFrames) {
    let mut ent = world.entity_mut(e);
    if frames.is_empty() {
        ent.remove::<(SpriteAnimFrames, SpriteAnimCursor)>();
    } else {
        ent.insert(frames);
    }
}

/// ⭐ **Remove empty frames** (F11).
pub fn remove_empty_frames(world: &mut World, e: Entity) -> bool {
    let (Some(lib), Some(mut frames)) = (
        world.get::<SpriteAnimations>(e).cloned(),
        world.get::<SpriteAnimFrames>(e).cloned(),
    ) else {
        return false;
    };
    if !ops::remove_empty_frames(&lib, &mut frames) {
        return false;
    }
    put_frames(world, e, frames);
    true
}

/// ⭐ **Delete empty animations** (F11): as TODAS vazias saem com o que a porta `AnimFieldEdit::Remove` leva — os
/// marcadores, a lista e o início (o tocador que estava nela cai na primeira).
pub fn delete_empty_animations(world: &mut World, e: Entity) -> bool {
    let Some(mut lib) = world.get::<SpriteAnimations>(e).cloned() else {
        return false;
    };
    let nomes = ops::empty_animations(&lib, world.get::<SpriteAnimFrames>(e));
    if nomes.is_empty() {
        return false;
    }
    let mut frames = world
        .get::<SpriteAnimFrames>(e)
        .cloned()
        .unwrap_or_default();
    let mut markers = world.get::<SpriteAnimMarkers>(e).cloned();
    for n in &nomes {
        lib.remove(n);
        frames.remove(n);
        if let Some(m) = markers.as_mut() {
            m.remove_tag(n);
        }
    }
    let primeira = lib.iter().next().map(|t| t.name.clone());
    let animator = world.get::<SpriteAnimator>(e).cloned().map(|mut p| {
        if p.start.as_ref().is_some_and(|s| nomes.contains(s)) {
            p.start.clone_from(&primeira);
        }
        if nomes.contains(&p.current) {
            p.current = primeira.clone().unwrap_or_default();
        }
        p
    });
    world.entity_mut(e).insert(lib);
    put_frames(world, e, frames);
    let mut ent = world.entity_mut(e);
    match markers {
        Some(m) if m.0.is_empty() => {
            ent.remove::<SpriteAnimMarkers>();
        }
        Some(m) => {
            ent.insert(m);
        }
        None => {}
    }
    if let Some(p) = animator {
        ent.insert(p);
    }
    true
}

#[cfg(test)]
#[path = "apply_tests.rs"]
mod tests;
