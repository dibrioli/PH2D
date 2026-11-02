//! **As prévias do quadro que o vetor desenha por cima do documento** — o overlay do Blend Object, os
//! realces do *Pick Shapes* e a silhueta do caminho-guia sob o cursor. Saiu da `fase_vector_overlays` da
//! shell (integração 02/11): é código da família, e a fase passou do tecto de função.

use ph2d_host::WindowSize;
use ph2d_render::Camera2d;
use ph2d_tool_vector::DrawMode;
use ph2d_vec_render::draw_blend_overlay;
use ph2d_vec_scene::{VecScene, VecXforms};
use ph2d_vector::{Affine, VectorScene};

/// Onde o ponteiro está, em ecrã, e a câmera/janela que o levam ao mundo.
pub struct Ponteiro<'a> {
    pub camera: &'a Camera2d,
    pub janela: WindowSize,
    pub px: (f32, f32),
}

impl crate::state::VecState {
    /// Desenha as prévias deste quadro em `out` e LIMPA os pickers que deixaram de valer.
    pub fn desenha_previas_de_pick(
        &mut self,
        vector_active: bool,
        scene: &VecScene,
        xf: &VecXforms,
        ponteiro: &Ponteiro<'_>,
        cam: Affine,
        out: &mut VectorScene,
    ) {
        // O **overlay** do Blend Object (ADR-0128): os passos virtuais + as fontes de cima
        // reempilhadas, na ordem de z (a última fonte por cima do último passo). Desenha depois
        // do `dispatch` (que já pôs as fontes no z da cena, embaixo); o overlay reestabelece a
        // pilha do blend por cima. O interleaving fino contra o resto da cena é da Fase C.
        draw_blend_overlay(&self.blend_overlay, cam, out);
        // **Pick Shapes** (ADR-0128 C2b): realça as formas escolhidas e costura a ORDEM de
        // clique numa polilinha (a prévia do spine). Fora do modo Pick, a lista não vale —
        // limpa, para não vazar escolhas velhas para o próximo blend.
        if vector_active && self.draw_config.mode == DrawMode::PickBlend {
            let preview = crate::blend_live::pick_preview(scene, xf, &self.blend_picks);
            draw_blend_overlay(&preview, cam, out);
        } else if !self.blend_picks.is_empty() {
            self.blend_picks.clear();
        }
        // **O Picker de caminho-guia** (Enio 2026-07-23): armado, o caminho sob o cursor é o que
        // o clique vai prender — a silhueta dele acende, o idioma do conta-gotas. `path_at` é o
        // MESMO resolvedor do clique, então o realce nunca mente sobre o que será escolhido. Fora
        // do modo Select (ou sem a tool) o pick não faz sentido: limpa, para não ficar armado e
        // invisível — a saída sem compromisso, como o clique no vazio.
        let Some(pick) = self.path_pick else {
            return;
        };
        if !(vector_active && self.draw_config.mode == DrawMode::Select) {
            self.path_pick = None;
            return;
        }
        let Ponteiro { camera, janela, px } = *ponteiro;
        let w = camera.screen_to_world(px, janela);
        let a = camera.screen_to_world((0.0, 0.0), janela);
        let b = camera.screen_to_world((1.0, 0.0), janela);
        // LITERAL-PX-OK: raio de acerto em px, o MESMO do picking de canvas (`path_at`).
        let hit_r = 10.0 * f64::from(((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt());
        if let Some(gid) = self
            .pen
            .path_at(scene, [f64::from(w[0]), f64::from(w[1])], hit_r)
            && gid != pick.source()
            && let Some(outline) = crate::pick::hover_outline(scene, xf, gid)
        {
            draw_blend_overlay(&[outline], cam, out);
        }
    }
}
