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

/// F4: quanto dura um piscar — TRÊS períodos do token `Slow`: o tecto de três da WCAG 2.3.1, a um
/// ritmo de 1,25 Hz (bem abaixo dos 3 por segundo da mesma regra).
pub const FLASH_PULSES: u32 = 3;

impl WidgetStore {
    /// O relógio do chrome — chamado pelo `dispatch_tick` em todo quadro.
    pub fn note_clock(&mut self, now_ns: u128) {
        self.tool_bars.clock_ns = now_ns;
    }

    /// O último relógio anotado (ns) — o que uma cena de smoke dá aos contactos sintéticos, para
    /// que o reconhecedor do toque meça o tempo com o MESMO relógio que a shell.
    #[must_use]
    pub fn clock_ns(&self) -> u128 {
        self.tool_bars.clock_ns
    }

    /// ⭐ **Faz PISCAR onde `id` mora** (F4, a paleta que ensina): todo pintor que mostra uma porta
    /// para `id` — o botão, o `⋯` que o guarda, o pulldown que o contém — desenha o anel.
    ///
    /// ⚠️ O pulldown que o contém lê-se AGORA: uma linha-interruptor (Snap ▸ …) troca de id quando é
    /// executada, e no quadro seguinte o id escolhido já não está em lado nenhum.
    pub fn flash(&mut self, id: ph2d_a11y::NodeId) {
        let pill = (0..crate::ids::MAX_AREA_MENUS)
            .find(|&s| {
                u8::try_from(s).is_ok_and(|s| {
                    self.area_menu_rows(s)
                        .iter()
                        .any(|e| e.node_id() == Some(id))
                })
            })
            .map(crate::ids::area_menu_button);
        self.tool_bars.flash = Some((id, pill, self.tool_bars.clock_ns));
    }

    /// O pulldown que continha o comando a piscar, lido na escolha.
    #[must_use]
    pub fn flash_pill(&self) -> Option<ph2d_a11y::NodeId> {
        self.tool_bars.flash.and_then(|(_, p, _)| p)
    }

    /// O comando a piscar e a intensidade agora (`0..=1`); `None` fora do piscar. `steady` = o
    /// movimento reduzido: anel fixo durante o mesmo tempo, sem piscar.
    #[must_use]
    pub fn flashing(&self, steady: bool) -> Option<(ph2d_a11y::NodeId, f32)> {
        let (id, _, t0) = self.tool_bars.flash?;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let period = (f64::from(ph2d_tokens::Duration::Slow.ms()) * 1e6) as u128;
        let t = self.tool_bars.clock_ns.saturating_sub(t0);
        if t >= period * u128::from(FLASH_PULSES) {
            return None;
        }
        if steady {
            return Some((id, 1.0));
        }
        #[allow(clippy::cast_precision_loss)]
        let phase = (t % period) as f32 / period as f32;
        // Cada período acende DE GOLPE e esmorece até apagar: três acesas, e acaba apagado.
        Some((id, (phase * std::f32::consts::FRAC_PI_2).cos()))
    }
}

impl WidgetStore {
    /// F5: o estado do toque.
    #[must_use]
    pub fn touch(&self) -> &crate::interaction::TouchGestures {
        &self.tool_bars.touch
    }

    pub fn touch_mut(&mut self) -> &mut crate::interaction::TouchGestures {
        &mut self.tool_bars.touch
    }

    /// F5: a fila ② está EMBAIXO da área (*Ver ▸ Opções da ferramenta embaixo*)?
    #[must_use]
    pub fn tool_row_bottom(&self) -> bool {
        self.tool_bars.row_bottom
    }

    pub fn set_tool_row_bottom(&mut self, bottom: bool) {
        self.tool_bars.row_bottom = bottom;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_a11y::NodeId;

    /// ⭐ **O piscar acende JÁ, pisca TRÊS vezes e apaga**; com movimento reduzido é um anel fixo
    /// pelo mesmo tempo. Os números são os do token `Slow` (400 ms) e do tecto da WCAG 2.3.1.
    #[test]
    fn the_flash_lights_now_pulses_three_times_and_ends() {
        let ms = 1_000_000u128;
        let mut s = WidgetStore::default();
        s.note_clock(5_000 * ms);
        s.flash(NodeId(7));
        assert_eq!(s.flashing(false), Some((NodeId(7), 1.0)), "não acendeu já");
        let mut acesos = 0;
        let mut antes = 1.0_f32;
        for t in 1..1_200u128 {
            s.note_clock(5_000 * ms + t * ms);
            let (_, l) = s.flashing(false).expect("apagou antes dos 1 200 ms");
            if l > 0.99 && antes <= 0.99 {
                acesos += 1;
            }
            antes = l;
            assert_eq!(
                s.flashing(true),
                Some((NodeId(7), 1.0)),
                "o reduzido piscou"
            );
        }
        assert_eq!(acesos + 1, 3, "pulsos ≠ 3");
        s.note_clock(5_000 * ms + 1_200 * ms);
        assert_eq!(s.flashing(false), None, "não apagou aos 1 200 ms");
        assert_eq!(s.flashing(true), None, "o reduzido não apagou");
    }
}
