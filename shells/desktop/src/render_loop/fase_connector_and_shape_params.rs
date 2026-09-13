//! **Fase do quadro: O CONECTOR E OS PARÂMETROS DE FORMA** — a selecção dos campos de texto, os campos do conector e os sliders da forma viva (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

/// Os pedidos que o dreno do barramento recolheu neste quadro para esta fase.
pub(super) struct ConnectorAndShapeParamsIntents {
    pub(super) pending_vec_shape_param: Option<(ph2d_editor_core::NodeId, f64)>,
    pub(super) pending_vec_connector: Option<(ph2d_editor_core::NodeId, f64)>,
}

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_connector_and_shape_params(
        &mut self,
        intents: ConnectorAndShapeParamsIntents,
        vec_px_to_world: f64,
    ) -> Option<Vec<ph2d_vec_scene::VecPathId>> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx { sim, vec_scene, .. } = FrameGfx::of(gfx);
        let ConnectorAndShapeParamsIntents {
            pending_vec_shape_param,
            pending_vec_connector,
        } = intents;
        // Configs de texto: aplicam na SESSÃO viva; sem sessão, no objeto de TEXTO
        // SELECIONADO (o texto segue editável no Select até virar curva). O
        // `vec_text_sel` é a seleção corrente para o caminho do objeto.
        let vec_text_sel: Vec<ph2d_vec_scene::VecPathId> = self.vec.pen.selected_paths().to_vec();
        // **O conector, pelo painel.** Editar um campo FIXA o valor (`None` → `Some`) em
        // TODOS os conectores selecionados — é o que permite calibrar o diagrama inteiro
        // de uma vez, em vez de linha por linha. A geometria não é escrita aqui: ela é
        // função pura da relação, e o `connector_live::recook` deste mesmo frame (mais
        // abaixo) a refaz. O undo global pega a mudança pelo diff do mundo ECS.
        if let Some((id, v)) = pending_vec_connector {
            crate::vec_connector_panel::edit_selected_connectors(
                sim,
                &self.vec.entities,
                &vec_text_sel,
                id,
                v,
            );
        }
        // Live Shapes: os sliders de forma editam a forma VIVA selecionada — muda o
        // parâmetro e RE-COZINHA in-place (id/estilo/pose preservados). Sem forma
        // viva na seleção, o slider só moveu o default de desenho (a tool já o
        // guardou) — é o que fecha o ciclo paramétrico: um polígono de 5 lados vira
        // de 7 depois de desenhado.
        if let Some((id, v)) = pending_vec_shape_param {
            crate::vec_shape_params::edit_selected_shape(
                sim,
                vec_scene,
                &self.vec.entities,
                &vec_text_sel,
                // ⚠️ **O MESMO modo que a pintura leu**: armado para desenhar, a caixa move o
                // default do próximo traço e NÃO alcança a forma selecionada — senão digitar
                // *"Pontas"* na Estrela armada poria lados no Polígono que está na tela
                // (os slots são por índice). O espelho é do frame anterior, e isso basta:
                // trocar de modo e digitar não são o mesmo gesto.
                self.vec.draw_config.mode,
                self.vec.shape_armed,
                |kind, values| {
                    crate::vec_shape_params::apply_shape_field(kind, values, id, v, vec_px_to_world)
                },
            );
        }
        Some(vec_text_sel)
    }
}
