//! **O tracejado em PARES** na secção Stroke (F16c) — o *Dashed Line* do painel Stroke do
//! Illustrator, no padrão do Inspector («um nome, N caixas»): uma linha por par, **Dash / Gap 1**
//! [traço][vão] até ao 3. O 1.º traço `0` é o contínuo, e o 1.º par `[0, 0]` depois do 1.º fecha o
//! padrão (`ph2d_vec_scene::DashPattern`).
//!
//! ⛔ **Duas linhas de TRÊS caixas (Dash [1][2][3] / Gap [1][2][3]) foram a 1.ª redacção, e a FOTO
//! da cena 24 recusou-a:** no painel do traço solto (a largura do painel docado) as três não cabem, a
//! fila parte e a 3.ª caixa cai debaixo da 1.ª — a «coluna = par» deixa de se ler. Duas caixas por
//! linha cabem (o *Head Size* ao lado prova-o).
//!
//! Mesmo ciclo das caixas da ponta ([`crate::paint_markers`]): semeadas com o valor EFECTIVO da
//! tool a cada quadro (menos a que tem o foco), e o `ValueChanged` volta como `SetValue` no id da
//! própria caixa — a tool é a dona do Style e o bridge leva-o a todas as selecionadas.

use crate::paint_sections::BodyCtx;
use crate::state;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::interaction::{WidgetEvent, WidgetStore};
use ph2d_editor_core::math::safe_clamp_f64;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_editor_core::property_row::Seccao;
use ph2d_editor_core::tool::PanelEvent;
use ph2d_i18n::tr_with;
use ph2d_tool_vector::ids::{VECTOR_DASH_IDS, VECTOR_GAP_IDS};
use ph2d_tool_vector::VectorStyleSnapshot;
use ph2d_tool_vector::params::{DASH_MAX, DASH_MIN, GAP_MAX, GAP_MIN};

/// O passo do arrasto/teclas das seis caixas — um quarto da largura do traço.
pub(crate) const DASH_STEP: f64 = 0.25; // LITERAL-PX-OK: passo em múltiplos da largura do traço, não uma medida de ecrã

/// A caixa `id` é de um vão? (`None` = não é caixa do tracejado.)
fn is_gap(id: ph2d_a11y::NodeId) -> Option<bool> {
    if VECTOR_DASH_IDS.contains(&id) {
        Some(false)
    } else if VECTOR_GAP_IDS.contains(&id) {
        Some(true)
    } else {
        None
    }
}

fn range(gap: bool) -> (f64, f64) {
    if gap {
        (GAP_MIN, GAP_MAX)
    } else {
        (DASH_MIN, DASH_MAX)
    }
}

/// Semeia as seis caixas com o tracejado EFECTIVO da tool (o da forma selecionada, adoptado).
pub(crate) fn seed(store: &mut WidgetStore) {
    let snap = state::current_snapshot();
    let focus = store.focus_id();
    for (k, [d, g]) in snap.dash.iter().enumerate() {
        for (id, gap, v) in [(VECTOR_DASH_IDS[k], false, *d), (VECTOR_GAP_IDS[k], true, *g)] {
            let (lo, hi) = range(gap);
            store.set_number_range(id, lo, hi, DASH_STEP);
            if focus != Some(id) {
                store.set_number_value(id, safe_clamp_f64(v, lo, hi));
            }
        }
    }
}

/// O evento de volta de uma caixa do tracejado.
pub(crate) fn apply_event(host: &mut dyn PanelHostInternal, ev: WidgetEvent) -> bool {
    let WidgetEvent::ValueChanged(id) = ev else {
        return false;
    };
    let Some(gap) = is_gap(id) else {
        return false;
    };
    let (lo, hi) = range(gap);
    let v = safe_clamp_f64(host.store().number_value(id).unwrap_or(lo), lo, hi);
    host.bus_mut()
        .push(EditorAction::ToolPanelEvent(PanelEvent::SetValue(id, v)));
    true
}

impl BodyCtx<'_> {
    /// As linhas do tracejado, uma por par. Devolve o `y` avançado.
    ///
    /// ⭐ **A linha do par seguinte aparece quando o anterior traça alguma coisa** — a MESMA lei do
    /// [`ph2d_vec_scene::DashPattern::active`]: um traço contínuo mostra só o 1.º par, um tracejado
    /// mostra o 2.º, e o 3.º chega quando o 2.º deixa de ser `[0, 0]`. É o que a casa vazia do
    /// Illustrator diz sem ocupar altura, e mantém a abertura do painel abaixo da de antes da F16c
    /// (o gate `a_altura_de_abertura_de_um_painel_so_encolhe` reprovou as três linhas sempre à vista).
    /// ⚠️ A coluna de nomes mede os TRÊS nomes: ela não salta quando uma linha aparece.
    pub(crate) fn dash_rows(&mut self, snap: &VectorStyleSnapshot, mut y: f32) -> f32 {
        let nomes: Vec<String> = (1..=VECTOR_DASH_IDS.len())
            .map(|n| tr_with("panel.vector.stroke.dash_gap", &[("n", &n)]))
            .collect();
        let refs: Vec<&str> = nomes.iter().map(String::as_str).collect();
        let sec = Seccao::medida(self.text_system, 2, &refs);
        let visiveis = visible_dash_rows(&snap.dash);
        for (k, nome) in refs.iter().enumerate().take(visiveis) {
            y = self.fields_row(nome, &[VECTOR_DASH_IDS[k], VECTOR_GAP_IDS[k]], None, y, sec);
        }
        y
    }
}

/// Quantas linhas do tracejado o painel mostra — ver [`BodyCtx::dash_rows`].
#[must_use]
pub(crate) fn visible_dash_rows(pairs: &[[f64; 2]; ph2d_vec_scene::MAX_DASH_PAIRS]) -> usize {
    let p = ph2d_vec_scene::DashPattern { pairs: *pairs };
    if p.is_dashed() {
        (p.active().len() + 1).min(ph2d_vec_scene::MAX_DASH_PAIRS)
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::visible_dash_rows;

    #[test]
    fn the_next_pair_row_appears_when_the_previous_one_draws() {
        assert_eq!(visible_dash_rows(&[[0.0, 1.0], [0.0, 0.0], [0.0, 0.0]]), 1, "contínuo");
        assert_eq!(visible_dash_rows(&[[0.0, 1.0], [2.0, 2.0], [0.0, 0.0]]), 1, "1.º traço 0");
        assert_eq!(visible_dash_rows(&[[3.0, 1.0], [0.0, 0.0], [0.0, 0.0]]), 2);
        assert_eq!(visible_dash_rows(&[[3.0, 1.0], [0.0, 1.0], [0.0, 0.0]]), 3, "um ponto conta");
        assert_eq!(visible_dash_rows(&[[3.0, 1.0], [1.0, 1.0], [1.0, 1.0]]), 3);
    }
}
