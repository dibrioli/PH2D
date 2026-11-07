//! **As cores das animações sobre a folha** ([estudo](../../../../docs/GameEngineFuncional/estudo_editor_de_animacoes_2026-11-07.md)
//! §4): cada célula sabe quem a usa, a animação aberta primeiro. Até [`BANDS_MAX`] faixas, o resto vira «+N».

use super::doc::AnimDoc;
use ph2d_editor_core::row_cells;
use ph2d_tokens::ColorToken;

/// Quantas faixas empilhadas uma célula mostra antes do «+N».
pub const BANDS_MAX: usize = 3;

/// A paleta das animações, de `ph2d_ecs::ANIM_COLORS` cores.
const PALETTE: [ColorToken; ph2d_ecs::ANIM_COLORS as usize] = [
    ColorToken::KanbanBlueBold,
    ColorToken::KanbanOrangeBold,
    ColorToken::KanbanGreenBold,
    ColorToken::KanbanPinkBold,
    ColorToken::KanbanYellowBold,
    ColorToken::KanbanPurpleBold,
    ColorToken::KanbanSkyBold,
    ColorToken::KanbanRedBold,
];

/// O token da cor gravada; fora da paleta (arquivo adulterado) dá a volta.
#[must_use]
pub fn anim_color(color: u8) -> ColorToken {
    PALETTE[usize::from(color) % PALETTE.len()]
}

/// As faixas à vista de uma célula e quantas ficam no «+N».
#[must_use]
pub fn shown_bands(users: &[usize]) -> (&[usize], usize) {
    let n = users.len().min(BANDS_MAX);
    (&users[..n], users.len() - n)
}

impl AnimDoc {
    /// Por célula, as animações que a usam (índices na lista), a aberta primeiro e as outras pela ordem da lista.
    /// A aberta lê o arrasto em curso, para a faixa nascer sob a mão.
    #[must_use]
    pub fn cell_users(&self) -> Vec<Vec<usize>> {
        let cells = self.snap.hframes.max(1) * self.snap.vframes.max(1);
        let mut users = vec![Vec::new(); cells as usize];
        let rows = self.snap.info.as_ref().map_or(&[][..], |i| &i.rows);
        let mut mark = |i: usize, frames: &[u32]| {
            for c in frames {
                if let Some(u) = users.get_mut(*c as usize)
                    && !u.contains(&i)
                {
                    u.push(i);
                }
            }
        };
        if self.selected < rows.len() {
            mark(self.selected, &self.shown_frames());
        }
        for (i, r) in rows.iter().enumerate() {
            if i != self.selected {
                mark(i, &row_cells(r, cells));
            }
        }
        users
    }

    /// Os nomes do que a célula sob o rato usa (a lista completa, a do «+N» incluída).
    #[must_use]
    pub fn hovered_names(&self, users: &[Vec<usize>]) -> Vec<String> {
        let rows = self.snap.info.as_ref().map_or(&[][..], |i| &i.rows);
        self.hover
            .and_then(|(x, y)| self.cell_at(x, y))
            .and_then(|c| users.get(c as usize))
            .map(|u| {
                u.iter()
                    .filter_map(|i| rows.get(*i).map(|r| r.name.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }
}
