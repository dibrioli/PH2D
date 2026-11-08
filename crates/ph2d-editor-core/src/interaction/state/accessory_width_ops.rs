//! ⭐ **A LARGURA de uma janela ACESSÓRIA** — as que flutuam ao lado da coluna do Inspector (Slices,
//! Tilemap). Ordem do dono, 08/11: *«O painel Slices e outros painéis acessórios devem ter a
//! alcinha de redimensionamento como o inspector»*.
//!
//! - **A escolha** — só o que o artista ARRASTOU; ausente = a largura de fábrica do painel. Ela
//!   grava-se em `~/.ph2d/sections.txt` pela porta de
//!   [`super::section_prefs_ops::section_prefs_text`], como a posição do seletor de cor: o formato é
//!   do editor e a shell só sabe onde fica o disco.
//! - **O arrasto** — ABSOLUTO: a largura é a de partida menos o caminho do ponteiro desde o Down.
//!   O tecto e o piso moram na pintura, que conhece a área; como o arrasto nunca soma ao limitado,
//!   passar do tecto e voltar devolve a borda para debaixo do ponteiro (incremental, escorregava).
//!
//! ⚠️ A borda é a de FORA (a esquerda): a direita fica presa à coluna, logo puxar para a esquerda
//! ALARGA.

use super::WidgetStore;
use ph2d_a11y::NodeId;
use std::collections::BTreeMap;

/// A chave de uma largura no ficheiro (`accessory_w.<id>=<px>`).
const CHAVE: &str = "accessory_w.";

/// O estado — um campo só no [`WidgetStore`].
#[derive(Clone, Debug, Default)]
pub struct AccessoryWidths {
    widths: BTreeMap<NodeId, f32>,
    /// `(painel, x do Down, largura no Down)` enquanto a borda está agarrada.
    drag: Option<(NodeId, f32, Option<f32>)>,
}

impl WidgetStore {
    /// A largura que o artista escolheu para esta janela; `None` = a de fábrica.
    #[must_use]
    pub fn accessory_width(&self, panel: NodeId) -> Option<f32> {
        self.accessory_widths.widths.get(&panel).copied()
    }

    /// Grava uma largura (só número finito). Marca a gravação só quando muda.
    pub fn set_accessory_width(&mut self, panel: NodeId, w: f32) {
        if !w.is_finite() || self.accessory_width(panel) == Some(w) {
            return;
        }
        self.accessory_widths.widths.insert(panel, w);
        self.mark_section_prefs_dirty();
    }

    /// Down na borda de fora: arma o arrasto.
    pub fn begin_accessory_width_drag(&mut self, panel: NodeId, x: f32) {
        let w = self.accessory_width(panel);
        self.accessory_widths.drag = Some((panel, x, w));
    }

    /// O painel cuja borda está agarrada, se algum.
    #[must_use]
    pub fn accessory_width_drag(&self) -> Option<NodeId> {
        self.accessory_widths.drag.map(|(p, _, _)| p)
    }

    /// Move: a borda segue o dedo — `x` a descer ALARGA (a direita está presa). `false` sem arrasto.
    /// `base` é a largura de fábrica, para o primeiro arrasto de quem nunca escolheu.
    pub(crate) fn move_accessory_width_drag(&mut self, x: f32, base: f32) -> bool {
        let Some((panel, x0, w0)) = self.accessory_widths.drag else {
            return false;
        };
        self.set_accessory_width(panel, w0.unwrap_or(base) - (x - x0));
        true
    }

    /// Up: larga a borda.
    pub fn end_accessory_width_drag(&mut self) {
        self.accessory_widths.drag = None;
    }
}

/// Escreve as larguras escolhidas.
pub(super) fn escreve(store: &WidgetStore, s: &mut String) {
    use std::fmt::Write as _;
    for (id, w) in &store.accessory_widths.widths {
        let _ = writeln!(s, "{CHAVE}{}={w}", id.0);
    }
}

/// Lê uma linha `k=v`; `true` se era uma largura (mesmo ilegível — ela não é de mais ninguém).
pub(super) fn le(store: &mut WidgetStore, k: &str, v: &str) -> bool {
    let Some(id) = k.strip_prefix(CHAVE) else {
        return false;
    };
    if let (Ok(id), Ok(w)) = (id.parse(), v.trim().parse::<f32>())
        && w.is_finite()
    {
        store.accessory_widths.widths.insert(NodeId(id), w);
    }
    true
}
