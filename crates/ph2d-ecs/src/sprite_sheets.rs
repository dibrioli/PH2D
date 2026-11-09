//! **As folhas extra de um Animated Sprite** (plano GEF 26 §3.1). A folha `0` é o `Sprite` + [`SpriteGrid`] de
//! sempre; a folha `k ≥ 1` é `SpriteSheets.0[k - 1]`. Cada uma guarda só o NOME durável da imagem (o [`AssetId`],
//! como o [`crate::SpritePixels`]), nunca o id de GPU: a textura resolve-se a cada sessão num cache derivado
//! (`ph2d_render::SheetTextures`), e gravar, abrir e desfazer não re-apontam nada.

use bevy_ecs::component::Component;
use bevy_ecs::prelude::{Entity, World};
use ph2d_asset::AssetId;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use std::collections::BTreeMap;

use crate::{SimComponent, SpriteGrid};

/// Quantas folhas um objeto tem, a `0` incluída: o `u8` do [`crate::FrameRef`] menos o [`SpriteGrid::EMPTY_SHEET`].
/// Não há recurso que limite abaixo (40 bytes por folha; a textura é um asset partilhado).
pub const SHEETS_MAX: usize = SpriteGrid::EMPTY_SHEET as usize;

/// ⭐ **O acerto de cada quadro de UMA folha** (GEF 28, Align): célula → deslocamento do desenho, em metros LOCAIS (o
/// mesmo referencial do Pivot, `y` para cima). Mora na FOLHA, não na animação: o desenho é o mesmo em qualquer animação
/// que use a célula. Esparso: célula sem entrada = `[0, 0]`. Na folha `0` é um componente do objeto (como o `Sprite`);
/// nas outras, um campo da [`ExtraSheet`]. Metros (não píxeis) para o Trim por quadro, que guarda o desenho no lugar
/// em metros, não lhe mexer; a escala da folha ([`CellOffsets::scaled`]) leva-os junto.
///
/// ⚠️ **Vale só no corte em que foi feito** (`cut` = `[colunas, linhas]`): recortar a folha (Cut again, Split, a grade,
/// uma imagem trocada que não se divide) muda o que cada célula É, e nenhum caminho precisa de se lembrar de o apagar —
/// [`CellOffsets::of`] pergunta o corte de agora.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CellOffsets {
    pub cut: [u32; 2],
    pub cells: BTreeMap<u32, [f32; 2]>,
}

impl SimComponent for CellOffsets {}

impl CellOffsets {
    /// O acerto da célula `cell` no corte `cut` (`[0, 0]` sem entrada ou com o corte mudado).
    #[must_use]
    pub fn of(&self, cut: [u32; 2], cell: u32) -> [f32; 2] {
        if cut != self.cut {
            return [0.0, 0.0];
        }
        self.cells.get(&cell).copied().unwrap_or([0.0, 0.0])
    }

    /// Põe o acerto da célula no corte `cut` (outro corte recomeça o mapa); `[0, 0]` apaga a entrada — o mapa fica
    /// esparso e o «quadro acertado» é só quem a tem.
    pub fn set(&mut self, cut: [u32; 2], cell: u32, d: [f32; 2]) {
        if cut != self.cut {
            *self = Self {
                cut,
                cells: BTreeMap::new(),
            };
        }
        if d == [0.0, 0.0] {
            self.cells.remove(&cell);
        } else {
            self.cells.insert(cell, d);
        }
    }

    /// Nenhum acerto.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Os mesmos acertos com a folha escalada por `s` em volta do Pivot.
    #[must_use]
    pub fn scaled(&self, s: f32) -> Self {
        Self {
            cut: self.cut,
            cells: self
                .cells
                .iter()
                .map(|(&c, d)| (c, [d[0] * s, d[1] * s]))
                .collect(),
        }
    }
}

/// Uma folha extra: a imagem, o tamanho de UMA célula na tela, o Pivot, o corte e o acerto de cada quadro.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExtraSheet {
    pub asset: AssetId,
    /// Tamanho de uma célula, em metros (o mesmo metro por pixel da folha `0`).
    pub size: [f32; 2],
    /// O Pivot desta folha (`Sprite.anchor` dela).
    pub anchor: [f32; 2],
    pub hframes: u32,
    pub vframes: u32,
    /// O acerto de cada quadro desta folha.
    pub offsets: CellOffsets,
}

impl ExtraSheet {
    /// Células da folha; satura como [`SpriteGrid::cells`].
    #[must_use]
    pub const fn cells(&self) -> u32 {
        self.grid(0).cells()
    }

    /// O corte `[colunas, linhas]` desta folha (a chave do [`CellOffsets`]).
    #[must_use]
    pub const fn cut(&self) -> [u32; 2] {
        [self.hframes, self.vframes]
    }

    /// A grelha desta folha na célula `frame`.
    #[must_use]
    pub const fn grid(&self, frame: u32) -> SpriteGrid {
        SpriteGrid {
            hframes: self.hframes,
            vframes: self.vframes,
            frame,
            sheet: 0,
        }
    }
}

/// As folhas `1..` do objeto, por ordem.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpriteSheets(pub SmallVec<[ExtraSheet; 2]>);

impl SimComponent for SpriteSheets {}

impl SpriteSheets {
    /// A folha `k` (`k ≥ 1`); `0` e as que não existem dão `None`.
    #[must_use]
    pub fn sheet(&self, k: u8) -> Option<&ExtraSheet> {
        usize::from(k).checked_sub(1).and_then(|i| self.0.get(i))
    }
}

/// **A folha visível do painel** (plano GEF 26 §3.3, decisão 8): a do cartão Image e a do Pivot. Estado de UI, como
/// a aba aberta: NÃO registado, não entra no gravar nem no undo.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ShownSheet(pub u8);

/// A folha visível de `entity`, presa às que existem (um undo pode ter tirado a que estava à vista).
#[must_use]
pub fn shown_sheet(world: &World, entity: Entity) -> u8 {
    let k = world.get::<ShownSheet>(entity).map_or(0, |s| s.0);
    let n = world.get::<SpriteSheets>(entity).map_or(0, |s| s.0.len());
    if usize::from(k) <= n { k } else { 0 }
}

/// **As células de cada folha** do objeto: `[0]` = a grelha dele (uma célula sem grelha), `[k]` = a folha extra `k`.
/// É o `sheets` que [`crate::tag_cells`] e o tique pedem.
#[must_use]
pub fn sheet_cells_of(world: &World, entity: Entity) -> SmallVec<[u32; 4]> {
    cells_per_sheet(
        world.get::<SpriteGrid>(entity),
        world.get::<SpriteSheets>(entity),
    )
}

/// **As imagens das folhas extra** de `entity`, por ordem (a folha `0` responde-se por `ph2d_render::texture_of`).
/// Quem pergunta «que imagens este objeto usa» (uso, gravar, macro, Painter) soma-as às da folha `0`.
#[must_use]
pub fn sheet_assets(world: &World, entity: Entity) -> SmallVec<[AssetId; 4]> {
    world
        .get::<SpriteSheets>(entity)
        .map(|s| s.0.iter().map(|x| x.asset).collect())
        .unwrap_or_default()
}

/// ⭐ **O nome de cada imagem usada como folha**, o da biblioteca (`door`, `door (2)`): a fonte ÚNICA do cartão Image e
/// do editor de animações (plano GEF 26). Derivado, não registado; `ids` é a chave do recálculo.
#[derive(bevy_ecs::prelude::Resource, Clone, Debug, Default, PartialEq)]
pub struct SheetNames {
    pub ids: Vec<AssetId>,
    pub names: std::collections::BTreeMap<AssetId, String>,
}

/// O nome de biblioteca de `id`, se [`SheetNames`] o tem.
#[must_use]
pub fn sheet_name(world: &World, id: AssetId) -> Option<&str> {
    world
        .get_resource::<SheetNames>()?
        .names
        .get(&id)
        .map(String::as_str)
}

/// ⭐ **O acerto da célula `cell` da folha `sheet`** de `entity` (`[0, 0]` sem acerto, folha inexistente ou vazia).
#[must_use]
pub fn cell_offset(world: &World, entity: Entity, sheet: u8, cell: u32) -> [f32; 2] {
    match sheet {
        0 => world.get::<CellOffsets>(entity).map(|o| {
            let g = world
                .get::<SpriteGrid>(entity)
                .copied()
                .unwrap_or(SpriteGrid::SINGLE);
            o.of(g.cut(), cell)
        }),
        k => world
            .get::<SpriteSheets>(entity)
            .and_then(|s| s.sheet(k))
            .map(|s| s.offsets.of(s.cut(), cell)),
    }
    .unwrap_or([0.0, 0.0])
}

/// [`sheet_cells_of`] sobre os componentes já em mão (o tique, numa query).
#[must_use]
pub fn cells_per_sheet(
    grid: Option<&SpriteGrid>,
    sheets: Option<&SpriteSheets>,
) -> SmallVec<[u32; 4]> {
    let mut out = SmallVec::new();
    out.push(grid.map_or(1, |g| g.cells().max(1)));
    if let Some(s) = sheets {
        out.extend(s.0.iter().map(ExtraSheet::cells));
    }
    out
}

#[cfg(test)]
#[path = "sprite_sheets_tests.rs"]
mod tests;
