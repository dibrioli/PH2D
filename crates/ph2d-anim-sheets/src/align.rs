//! ⭐ **Align: a geometria de cada folha e de cada quadro** (GEF 28, onda A; decisões do dono 09/11: sem giro). Três
//! verbos sobre o documento, e nenhum campo novo por folha além do acerto das células:
//!
//! - **mover a folha** é o Pivot dela sem a posição a compensar (o [`ph2d_sprite_edit::pivot`] move a origem e deixa o
//!   desenho; aqui o desenho anda e a origem fica) — um deslocamento guardado à parte seria uma segunda grandeza para
//!   o mesmo ponto;
//! - **escalar a folha** multiplica o tamanho, o Pivot e os acertos em volta do pivô: os pés ficam;
//! - **acertar um quadro** escreve o [`CellOffsets`] da célula, que o render, a caixa do gizmo e o encaixe leem
//!   ([`ph2d_render::drawn_geometry`]).

use ph2d_ecs::{CellOffsets, Entity, SpriteGrid, SpriteSheets, World};
use ph2d_render::Sprite;

/// A menor escala que um passo aceita (e o inverso, a maior): abaixo disto o quad some num píxel e o passo seguinte
/// não o traz de volta pela mesma conta.
const ESCALA_MIN: f32 = 1.0e-3;

/// ⭐ A folha `k` de `e` escala por `f` em volta do Pivot: tamanho, Pivot e acertos. `false` sem folha ou com um `f`
/// inválido (não finito, `≤ 0`, ou `1`).
pub fn scale_sheet(world: &mut World, e: Entity, k: u8, f: f32) -> bool {
    if !f.is_finite() || !(ESCALA_MIN..=1.0 / ESCALA_MIN).contains(&f) || f == 1.0 {
        return false;
    }
    let vezes = |v: [f32; 2]| [v[0] * f, v[1] * f];
    if k == 0 {
        let Some(mut s) = world.get_mut::<Sprite>(e) else {
            return false;
        };
        // O Pivot resolvido = `anchor` + o canto (de `size`) + `offset`/ppm: os três escalam e o ponto do pivô fica.
        (s.size, s.anchor, s.offset) = (vezes(s.size), vezes(s.anchor), vezes(s.offset));
        let novos = world.get::<CellOffsets>(e).map(|o| o.scaled(f));
        if let Some(o) = novos {
            world.entity_mut(e).insert(o);
        }
        return true;
    }
    let Some(mut sheets) = world.get_mut::<SpriteSheets>(e) else {
        return false;
    };
    let Some(s) = usize::from(k)
        .checked_sub(1)
        .and_then(|i| sheets.0.get_mut(i))
    else {
        return false;
    };
    (s.size, s.anchor) = (vezes(s.size), vezes(s.anchor));
    s.offsets = s.offsets.scaled(f);
    true
}

/// ⭐ O desenho da folha `k` de `e` anda `d` metros locais; a origem, os filhos e a colisão ficam. `false` sem folha
/// ou com `d` nulo ou não finito.
pub fn move_sheet(world: &mut World, e: Entity, k: u8, d: [f32; 2]) -> bool {
    if d == [0.0, 0.0] || !d.iter().all(|v| v.is_finite()) {
        return false;
    }
    let anda = |a: &mut [f32; 2]| *a = [a[0] + d[0], a[1] + d[1]];
    if k == 0 {
        return world
            .get_mut::<Sprite>(e)
            .map(|mut s| anda(&mut s.anchor))
            .is_some();
    }
    world
        .get_mut::<SpriteSheets>(e)
        .and_then(|mut sheets| {
            let i = usize::from(k).checked_sub(1)?;
            sheets.0.get_mut(i).map(|s| anda(&mut s.anchor))
        })
        .is_some()
}

/// ⭐ A célula `cell` da folha `k` de `e` anda `by` metros locais (soma ao acerto que tinha; voltar a `[0, 0]` apaga a
/// entrada). `false` sem folha, com a célula fora dela, ou com `by` nulo ou não finito.
pub fn nudge_cell(world: &mut World, e: Entity, k: u8, cell: u32, by: [f32; 2]) -> bool {
    if by == [0.0, 0.0] || !by.iter().all(|v| v.is_finite()) {
        return false;
    }
    let cells = ph2d_ecs::sheet_cells_of(world, e);
    if usize::from(k) >= cells.len() || cell >= cells[usize::from(k)] {
        return false;
    }
    let soma = |o: &mut CellOffsets, cut: [u32; 2]| {
        let a = o.of(cut, cell);
        o.set(cut, cell, [a[0] + by[0], a[1] + by[1]]);
    };
    if k == 0 {
        if world.get::<Sprite>(e).is_none() {
            return false;
        }
        let cut = world
            .get::<SpriteGrid>(e)
            .copied()
            .unwrap_or(SpriteGrid::SINGLE)
            .cut();
        let mut o = world.get::<CellOffsets>(e).cloned().unwrap_or_default();
        soma(&mut o, cut);
        let mut ent = world.entity_mut(e);
        if o.is_empty() {
            ent.remove::<CellOffsets>();
        } else {
            ent.insert(o);
        }
        return true;
    }
    world
        .get_mut::<SpriteSheets>(e)
        .and_then(|mut sheets| {
            let i = usize::from(k).checked_sub(1)?;
            sheets.0.get_mut(i).map(|s| {
                let cut = s.cut();
                soma(&mut s.offsets, cut);
            })
        })
        .is_some()
}

/// ⭐ **A geometria de uma folha como o editor a mostra** (GEF 28, onda B): o centro do quad e o tamanho em metros
/// locais (Y para cima, a origem do objecto = o Pivot), e o acerto de cada célula do corte ATUAL.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SheetGeo {
    pub center: [f32; 2],
    pub size: [f32; 2],
    pub offsets: std::collections::BTreeMap<u32, [f32; 2]>,
}

impl SheetGeo {
    /// O centro do quad da célula `cell`, com o acerto dela.
    #[must_use]
    pub fn center_of(&self, cell: u32) -> [f32; 2] {
        let d = self.offsets.get(&cell).copied().unwrap_or_default();
        [self.center[0] + d[0], self.center[1] + d[1]]
    }
}

/// A geometria de cada folha de `e` (a `0` primeiro), com `ppm` a converter o `offset` em pixels da folha `0`.
#[must_use]
pub fn sheet_geos(world: &World, e: Entity, ppm: f32) -> Vec<SheetGeo> {
    let n = ph2d_ecs::sheet_cells_of(world, e).len();
    (0..n)
        .filter_map(|k| {
            let k = u8::try_from(k).ok()?;
            let (s, _) = ph2d_render::sheet_geometry(world, e, k, 0)?;
            let offsets = if k == 0 {
                let cut = world
                    .get::<SpriteGrid>(e)
                    .copied()
                    .unwrap_or(SpriteGrid::SINGLE)
                    .cut();
                world
                    .get::<CellOffsets>(e)
                    .filter(|o| o.cut == cut)
                    .map(|o| o.cells.clone())
            } else {
                world
                    .get::<SpriteSheets>(e)
                    .and_then(|x| x.sheet(k))
                    .filter(|x| x.offsets.cut == x.cut())
                    .map(|x| x.offsets.cells.clone())
            };
            Some(SheetGeo {
                center: s.resolve_anchor(ppm),
                size: s.size,
                offsets: offsets.unwrap_or_default(),
            })
        })
        .collect()
}

#[cfg(test)]
#[path = "align_tests.rs"]
mod tests;
