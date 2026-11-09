//! Os eventos do cartão Composite (enable, reordenar, sliders por posição) — irmão de
//! [`super::composite`], partido pelo tecto de LOC (movimento puro).

use super::composite::{MAX_TAMANHO_DA_CAMADA, N_ESCOPOS_DA_BORRACHA};
use crate::tool::PainterTool;
use ph2d_editor_core::tool::PanelEvent;

impl PainterTool {
    /// Route the Composite-card panel events (enable checkbox, per-position reorder buttons, per-position
    /// Strength sliders). Returns `true` iff consumed — chained ahead of the big `handle_panel_event` match.
    pub(crate) fn route_composite_event(&mut self, event: &PanelEvent) -> bool {
        match event {
            PanelEvent::Click(id) => {
                if *id == crate::ids::PAINTER_BRUSH_COMPOSITE_ENABLE {
                    self.toggle_composite();
                    return true;
                }
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_UP
                    .iter()
                    .position(|x| x == id)
                {
                    self.move_composite_layer_up(p);
                    return true;
                }
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_DOWN
                    .iter()
                    .position(|x| x == id)
                {
                    self.move_composite_layer_down(p);
                    return true;
                }
                // ⛔⛔ **O chip da OPERAÇÃO saiu em 2026-09-21.** Ele ciclava as quatro, e com
                // as QUOTAS do dono (`3 Brush · 2 Erase · 1 Blur · 1 Smear`) um ciclo livre
                // tornaria a quota uma mentira: dois cliques punham dois Blurs na pilha. A
                // operação passa a ser escolhida na CRIAÇÃO, no menu ao lado do `+`, e trocá-la é
                // retirar a camada e criar outra. ⚠️ *É uma capacidade que sai — declarada, e
                // reversível numa linha se o dono a quiser de volta com o ciclo limitado à
                // quota.*
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_REMOVE
                    .iter()
                    .position(|x| x == id)
                {
                    self.retira_camada(p);
                    return true;
                }
                if *id == crate::ids::PAINTER_BRUSH_COMPOSITE_ADD {
                    self.acrescenta_camada(self.paint.composite_add_op.to_u8());
                    return true;
                }
                if let Some(op) = crate::ids::PAINTER_BRUSH_COMPOSITE_ADD_OPTION
                    .iter()
                    .position(|x| x == id)
                {
                    self.set_composite_add_op(op as u8);
                    return true;
                }
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_COLOR_CLEAR
                    .iter()
                    .position(|x| x == id)
                {
                    self.clear_composite_layer_color(p);
                    return true;
                }
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_HARDNESS_CLEAR
                    .iter()
                    .position(|x| x == id)
                {
                    self.clear_composite_layer_hardness(p);
                    return true;
                }
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_ERASE_SCOPE
                    .iter()
                    .position(|x| x == id)
                {
                    // O chip CICLA os escopos — o mesmo gesto do chip da operação, e pela mesma
                    // razão: o `PanelEvent` é contrato congelado (§6) e não tem clique-direito.
                    let proximo = (usize::from(self.paint.composite[p].erase_scope.to_u8() + 1)
                        % N_ESCOPOS_DA_BORRACHA) as u8;
                    self.set_composite_layer_erase_scope(p, proximo);
                    return true;
                }
                false
            }
            // ⭐ O menu do `+` — a porta do contrato congelado para uma escolha com payload
            // (§6: o `PanelEvent` tem quatro variantes e a `SelectOption` é o canal de texto; o
            // doc dela avisa por escrito contra gastar um ADR numa variante que ela já dá).
            PanelEvent::SelectOption(id, v)
                if *id == crate::ids::PAINTER_BRUSH_COMPOSITE_ADD_KIND =>
            {
                if let Ok(op) = v.parse::<u8>() {
                    self.set_composite_add_op(op);
                }
                true
            }
            PanelEvent::SetValue(id, v) => {
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_STRENGTH
                    .iter()
                    .position(|x| x == id)
                {
                    self.set_composite_layer_strength(p, *v as f32);
                    return true;
                }
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_HARDNESS
                    .iter()
                    .position(|x| x == id)
                {
                    // A pista e o campo são a MESMA grandeza (`0..1`) — sem projecção nenhuma.
                    self.set_composite_layer_hardness(p, *v as f32);
                    return true;
                }
                if let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_SIZE
                    .iter()
                    .position(|x| x == id)
                {
                    // ⚠️ A pista é `0..1` e o campo é um MULTIPLICADOR até `MAX_TAMANHO_DA_CAMADA`
                    // (o chip mostra px de diâmetro; a projecção px ↔ pista é do painel).
                    self.set_composite_layer_size(p, *v as f32 * MAX_TAMANHO_DA_CAMADA);
                    return true;
                }
                false
            }
            // ── A cor de uma camada, pelo picker partilhado: `"r,g,b"` em 8 bits nativos, a mesma
            //    rota (e o mesmo formato) do `PAINTER_COLOR_THUMB` do pincel. ──
            PanelEvent::SelectOption(id, value) => {
                let Some(p) = crate::ids::PAINTER_BRUSH_COMPOSITE_COLOR
                    .iter()
                    .position(|x| x == id)
                else {
                    return false;
                };
                let mut it = value.split(',');
                if let (Some(r), Some(g), Some(b)) = (it.next(), it.next(), it.next())
                    && let (Ok(r), Ok(g), Ok(b)) =
                        (r.parse::<u8>(), g.parse::<u8>(), b.parse::<u8>())
                {
                    self.set_composite_layer_color(p, [r, g, b].map(|v| f32::from(v) / 255.0));
                }
                true
            }
            _ => false,
        }
    }
}
