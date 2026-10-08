//! As barras de um módulo no store — ver [`crate::interaction::ToolBars`].

use super::WidgetStore;
use crate::interaction::ToolBars;

impl WidgetStore {
    /// ⭐ Publica as barras deste quadro. ⚠️ Em TODO quadro, `None` incluído: um módulo que larga a
    /// mão não pode deixar a coluna dele pintada (e clicável) por cima de outro.
    pub fn set_tool_bars(&mut self, bars: Option<ToolBars>) {
        self.tool_bars.bars = bars;
    }

    /// As barras publicadas neste quadro.
    #[must_use]
    pub fn tool_bars(&self) -> Option<&ToolBars> {
        self.tool_bars.bars.as_ref()
    }

    /// ④ desligada pelo artista?
    #[must_use]
    pub fn selection_actions_off(&self) -> bool {
        self.tool_bars.actions_off
    }

    pub fn set_selection_actions_off(&mut self, off: bool) {
        self.tool_bars.actions_off = off;
    }

    /// ④ fixada no topo da área?
    #[must_use]
    pub fn selection_actions_pinned(&self) -> bool {
        self.tool_bars.actions_pinned
    }

    pub fn set_selection_actions_pinned(&mut self, pinned: bool) {
        self.tool_bars.actions_pinned = pinned;
    }

    /// ⭐ **Abre o seletor de cor partilhado em `id`** — semeado com a cor que a amostra mostra.
    ///
    /// ⚠️ A ÚNICA porta: o Down numa amostra (`dispatch::pointer_down`) e o `X` do vetor (D10)
    /// chamam esta. Só apontar o alvo deixaria o seletor com o valor da amostra ANTERIOR, e quem o
    /// lê de volta (o `vector_bridge`) pintaria essa cor na amostra nova.
    pub fn open_picker_on(&mut self, id: ph2d_a11y::NodeId) {
        let seed = self.widget_color(id).unwrap_or([0x88, 0x88, 0x88, 0xFF]);
        self.set_widget_color(id, seed);
        self.set_picker_target(Some(id));
        self.set_blender_value(
            crate::ids::INSP_BLENDER_PICKER,
            ph2d_tokens::ColorValue::from_rgba8(seed[0], seed[1], seed[2], seed[3]),
        );
    }

    /// ⑤ Regista onde o ponteiro está (chamado pelo despacho de TODO evento de ponteiro).
    pub fn note_pointer(&mut self, x: f32, y: f32) {
        self.tool_bars.pointer = Some([x, y]);
    }

    /// ⑤ A última posição do ponteiro no ecrã — `None` antes do primeiro evento.
    #[must_use]
    pub fn last_pointer(&self) -> Option<[f32; 2]> {
        self.tool_bars.pointer
    }
}
