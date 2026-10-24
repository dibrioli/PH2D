//! [`BitmaskGrid32`] — a 32-bit mask rendered as an 8-column × 4-row grid of TILES, bit `n` labeled
//! `n+1` INSIDE its tile (lit = set). The law is Godot's `EditorPropertyLayers`, run as an oracle
//! (`docs/CodeEditor/ferramentas/oraculo_grade_de_camadas/`): the number lives in the cell, so no
//! panel width can elide it.
//!
//! Sprite Inspector v2 W6 (spec §15.7, T6.4). Extracted from the inline
//! grid in the §8 Visibility section so it can be reused by both
//! `VisibilityLayer` and the camera `VisibilityCullMask`. The widget
//! owns geometry (`cell_rects`) + paint + a11y; the consuming section
//! registers each bit's hit rect and toggles the mask on click.

use crate::paint::{fill_rounded_rect, paint_text_centered_com_orcamento, resolve};
use crate::zones::Rect;
use ph2d_a11y::{Node, NodeBuilder, NodeId, Role};
use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Radius, Theme, TypeToken};
use ph2d_vector::VectorScene;

/// Columns in the bitmask grid — eight, so a cell of a 220 px dock (~25 px) still holds «32» whole.
pub const BITMASK_COLS: usize = 8;
/// Rows in the bitmask grid (`32 / BITMASK_COLS`).
pub const BITMASK_ROWS: usize = 32 / BITMASK_COLS;

#[derive(Clone, Debug)]
pub struct BitmaskGrid32 {
    pub id: NodeId,
    /// Caller-provided (localized) a11y label for the whole grid.
    pub label: String,
    /// The 32 per-bit hit ids, indexed by bit (0..32).
    pub bit_ids: [NodeId; 32],
    /// Current mask; bit `n` set → cell `n` checked.
    pub mask: u32,
    /// When false, all cells render in the disabled (unchecked-looking)
    /// state and the a11y node is non-interactive. The grid is still
    /// laid out so a parent can show it greyed.
    pub enabled: bool,
}

impl BitmaskGrid32 {
    pub fn new(id: NodeId, label: impl Into<String>, bit_ids: [NodeId; 32], mask: u32) -> Self {
        Self {
            id,
            label: label.into(),
            bit_ids,
            mask,
            enabled: true,
        }
    }

    pub fn enabled(mut self, yes: bool) -> Self {
        self.enabled = yes;
        self
    }

    pub fn is_set(&self, bit: usize) -> bool {
        bit < 32 && (self.mask >> bit as u32) & 1 == 1
    }

    /// Total height occupied by the grid given a per-cell height.
    pub fn grid_height(cell_h: f32) -> f32 {
        cell_h * BITMASK_ROWS as f32
    }

    /// Hit rect for a single bit cell. `origin` is the grid's top-left;
    /// `w` is the full grid width (split into [`BITMASK_COLS`] columns);
    /// `cell_h` is the per-row height. Column-major within a row:
    /// bit `n` sits at `(col = n % COLS, row = n / COLS)`.
    pub fn cell_rect(origin_x: f32, origin_y: f32, w: f32, cell_h: f32, bit: usize) -> Rect {
        let cell_w = w / BITMASK_COLS as f32;
        let col = bit % BITMASK_COLS;
        let row = bit / BITMASK_COLS;
        Rect::new(
            origin_x + cell_w * col as f32,
            origin_y + cell_h * row as f32,
            cell_w,
            cell_h,
        )
    }

    /// All 32 cell rects, indexed by bit.
    pub fn cell_rects(&self, origin_x: f32, origin_y: f32, w: f32, cell_h: f32) -> [Rect; 32] {
        std::array::from_fn(|bit| Self::cell_rect(origin_x, origin_y, w, cell_h, bit))
    }

    pub fn build_a11y(&self, x: f64, y: f64, w: f64, h: f64) -> Node {
        let mut b = NodeBuilder::new(Role::Group)
            .label(&self.label)
            .bounds(x, y, w, h);
        for id in self.bit_ids {
            b = b.child(id);
        }
        b.build()
    }
}

/// ⭐ **O LADRILHO de um bit dentro da célula**: a célula menos o vão que separa os ladrilhos.
#[must_use]
pub fn tile_rect(cell: Rect) -> Rect {
    let vao = ph2d_tokens::control_gap_px() * 0.5;
    Rect::new(
        cell.x + vao,
        cell.y + vao,
        (cell.w - 2.0 * vao).max(0.0),
        (cell.h - 2.0 * vao).max(0.0),
    )
}

/// Quanto do ladrilho o número pode ocupar: a largura dele menos um `Xs` (o texto é CENTRADO, sobra
/// meio `Xs` de cada lado). ⛔ Não o `label_budget` de um rótulo: ele tira `Md` de cada lado e, num
/// ladrilho de ~30 px, elidia «20», «23», «24» para «…» (foto do 24/10).
#[must_use]
pub fn tile_budget(tile: Rect) -> f32 {
    (tile.w - ph2d_tokens::Spacing::Xs.px()).max(0.0)
}

/// As tintas de um ladrilho: `(fundo, número)`. Ligado = o acento; desligado = escuro com o número
/// apagado; a grelha inteira desactivada = tudo apagado.
#[must_use]
pub fn tile_tokens(enabled: bool, set: bool) -> (ColorToken, ColorToken) {
    match (enabled, set) {
        (false, _) => (ColorToken::Bg2, ColorToken::TextDisabled),
        (true, true) => (ColorToken::Accent, ColorToken::AccentFg),
        (true, false) => (ColorToken::Bg3, ColorToken::Text3),
    }
}

/// Paint the 32 tiles. The caller is responsible for
/// registering each cell's hit rect (via [`BitmaskGrid32::cell_rect`])
/// so clicks toggle the bit.
#[allow(clippy::too_many_arguments)]
pub fn paint_bitmask_grid32(
    grid: &BitmaskGrid32,
    origin_x: f32,
    origin_y: f32,
    w: f32,
    cell_h: f32,
    scene: &mut VectorScene,
    text_system: &mut TextSystem,
    theme: Theme,
) {
    // ⭐ **As 32 células são UM controlo** (uma máscara), e era esta grelha que sozinha punha
    //    `32` comandos na dívida do Inspector — ver [`crate::widget::composto`].
    crate::widget::composto::grupo(grid.bit_ids);
    let fonte = TypeToken::Sm.px();
    let raio = crate::paint::frame_radius(theme, Radius::Xs.px());
    for bit in 0..32usize {
        let tile = tile_rect(BitmaskGrid32::cell_rect(origin_x, origin_y, w, cell_h, bit));
        let (fundo, tinta) = tile_tokens(grid.enabled, grid.is_set(bit));
        fill_rounded_rect(scene, tile, raio, resolve(fundo, theme));
        paint_text_centered_com_orcamento(
            text_system,
            scene,
            &(bit + 1).to_string(),
            tile,
            tile_budget(tile),
            fonte,
            resolve(tinta, theme),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids() -> [NodeId; 32] {
        std::array::from_fn(|i| NodeId(100 + i as u64))
    }

    fn fixture() -> BitmaskGrid32 {
        BitmaskGrid32::new(NodeId(1), "Visibility Layer", ids(), 0b1011)
    }

    /// ⭐⭐ **Todo número cabe INTEIRO no seu ladrilho, em toda largura de painel** — o report do dono
    /// de 24/10 (a grade do «Cull Mask» sem um único número) e a foto do dia seguinte, em que o
    /// orçamento de RÓTULO ainda elidia «20», «23», «24» num ladrilho de ~30 px. A varredura é da
    /// linha útil de um dock de 196 px até 600 px, em passos de ¼ px: um defeito de fronteira não se
    /// apanha amostrando seis larguras.
    ///
    /// **Mutações que devem sangrar:** `tile_budget` = `label_budget(tile.w)` · `BITMASK_COLS = 16`.
    #[test]
    fn todo_numero_cabe_inteiro_no_seu_ladrilho() {
        let mut ts = TextSystem::without_system_fonts();
        let fonte = TypeToken::Sm.px();
        let mut linha = 196.0_f32; // LITERAL-PX-OK: a linha útil do dock mais estreito (220 px)
        while linha <= 600.0 {
            let tile = tile_rect(Rect::new(
                0.0,
                0.0,
                linha / BITMASK_COLS as f32,
                ph2d_tokens::ROW_H_PX,
            ));
            for n in 1..=32 {
                let texto = n.to_string();
                let mostrado = crate::text_elide::fit(&mut ts, &texto, fonte, tile_budget(tile));
                let largura = ts.prefix_width(&texto, fonte);
                assert!(
                    mostrado == texto && largura <= tile.w,
                    "linha {linha}: o bit {n} sai «{mostrado}» ({largura:.1} px) num ladrilho de \
                     {:.1} px",
                    tile.w
                );
            }
            linha += 0.25;
        }
    }

    /// Ligado e desligado NUNCA têm o mesmo fundo — a única pista de estado que o ladrilho tem.
    #[test]
    fn ligado_e_desligado_lem_se_diferentes() {
        assert_ne!(tile_tokens(true, true).0, tile_tokens(true, false).0);
        assert_ne!(tile_tokens(true, true).1, tile_tokens(true, false).1);
    }

    #[test]
    fn dimensions_are_8x4() {
        assert_eq!(BITMASK_COLS, 8);
        assert_eq!(BITMASK_ROWS, 4);
        assert_eq!(BITMASK_COLS * BITMASK_ROWS, 32);
    }

    #[test]
    fn is_set_reads_mask() {
        let g = fixture();
        assert!(g.is_set(0));
        assert!(g.is_set(1));
        assert!(!g.is_set(2));
        assert!(g.is_set(3));
        assert!(!g.is_set(31));
    }

    #[test]
    fn cell_rect_grid_layout() {
        // bit 0 → (col 0, row 0); bit 8 → (col 0, row 1); bit 9 → (col 1, row 1).
        let r0 = BitmaskGrid32::cell_rect(0.0, 0.0, 400.0, 24.0, 0);
        let r8 = BitmaskGrid32::cell_rect(0.0, 0.0, 400.0, 24.0, 8);
        let r9 = BitmaskGrid32::cell_rect(0.0, 0.0, 400.0, 24.0, 9);
        assert_eq!(r0.x, 0.0);
        assert_eq!(r0.y, 0.0);
        assert_eq!(r8.x, 0.0);
        assert_eq!(r8.y, 24.0);
        assert_eq!(r9.x, 50.0);
        assert_eq!(r9.y, 24.0);
    }

    #[test]
    fn grid_height_is_four_rows() {
        assert_eq!(BitmaskGrid32::grid_height(24.0), 24.0 * 4.0);
    }

    #[test]
    fn a11y_role_is_group() {
        let node = fixture().build_a11y(0.0, 0.0, 400.0, 192.0);
        assert_eq!(node.role(), Role::Group);
    }

    fn smoke(g: BitmaskGrid32, theme: Theme) {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_bitmask_grid32(&g, 0.0, 0.0, 400.0, 24.0, &mut scene, &mut text, theme);
    }

    #[test]
    fn paint_smoke_enabled() {
        smoke(fixture(), Theme::Forge);
    }

    #[test]
    fn paint_smoke_disabled() {
        smoke(fixture().enabled(false), Theme::Blueprint);
    }
}
