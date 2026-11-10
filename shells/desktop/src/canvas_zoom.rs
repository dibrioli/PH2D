//! **O zoom do canvas obedece à roda, e CHEGA lá.**
//!
//! A roda sobre o canvas multiplicava `height_world` na hora (`camera.zoom(factor)`), então o
//! gesto que o artista repete o dia inteiro era o único movimento da tela que SALTA. Este módulo
//! põe o mesmo escalar no substrato da UI viva: a roda escreve um **destino**, e o quadro publica
//! o **vivo**.

use ph2d_editor_core::NodeId;
use ph2d_editor_core::motion::{Role, UiMotion};
use ph2d_render::Camera2d;

/// O id do escalar no substrato. Hash de string ⇒ **nenhum contador de gate** se move.
pub(crate) const CANVAS_ZOOM: NodeId = ph2d_tool_registry::hash_node_id("canvas.zoom");
/// O escalar do voo de um enquadramento (F8, `ph2d_render::camera_frame::ViewFlight`).
pub(crate) const VIEW_FLIGHT: NodeId = ph2d_tool_registry::hash_node_id("canvas.view_flight");

/// O destino do zoom do canvas, e a testemunha que devolve a câmera aos outros donos.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct CanvasZoom {
    /// O destino AUTORADO, em `height_world`. `None` = **a câmera não é minha**.
    target: Option<f32>,
    /// O último `height_world` que ESTE módulo publicou — a testemunha de escrita estrangeira.
    published: f32,
}

impl CanvasZoom {
    /// **Um entalhe de roda.** `live` é o `height_world` de agora; `factor < 1` aproxima.
    ///
    /// ⚠️ **Compõe no DESTINO, nunca no vivo** — a lição que a rolagem de painel já pagou: cinco
    /// voltas compostas sobre um valor a meio caminho somam bem menos do que cinco voltas.
    pub(crate) fn wheel(&mut self, live: f32, factor: f32) {
        if !factor.is_finite() || factor <= 0.0 || !live.is_finite() || live <= 0.0 {
            return;
        }
        let base = match self.target {
            Some(t) => t,
            None => {
                // O gesto começa: a testemunha nasce igual ao vivo, senão o primeiro tique acha
                // que alguém escreveu por fora e larga a câmera no mesmo quadro.
                self.published = live;
                live
            }
        };
        // ⚠️ **O clamp é o da CÂMERA, perguntado — nunca copiado.** Uma segunda cópia divergiria
        // no dia em que um dos dois limites se mover, e o artista veria a roda parar num sítio que
        // o resto do app não conhece.
        self.target = Some(Camera2d::zoomed(base, factor));
    }

    /// **O tique do quadro.** Devolve o `height_world` a publicar, ou `None` quando a câmera não
    /// é deste módulo.
    ///
    /// ⚠️ **A SEMEADURA é obrigatória e é a lei do substrato:** *a primeira vista de um id chega
    /// ao alvo*. Sem plantar a track no valor VIVO antes de alvejar, o primeiro entalhe de cada
    /// gesto saltaria — o defeito de uma-vez-por-gesto que a estreia das secções já pagou.
    pub(crate) fn tick(&mut self, live: f32, motion: &mut UiMotion) -> Option<f32> {
        let target = self.target?;
        if live != self.published {
            // Escrita estrangeira (fit-to-view, load, cena de smoke): ela é a verdade.
            self.target = None;
            return None;
        }
        let goal = target.ln();
        if motion.get(CANVAS_ZOOM).is_none() {
            let _ = motion.animate(CANVAS_ZOOM, live.ln(), Role::Surface);
        }
        let now = motion.animate(CANVAS_ZOOM, goal, Role::Surface);
        // ⚠️ **O repouso publica o DESTINO, não `exp(ln(destino))` — e esta linha carrega DUAS
        // promessas.** (1) O `height_world` assentado é o que os ~90 leitores da câmera veem e o
        // que um entalhe futuro compõe; deixá-lo com o resíduo do round-trip (~1e-6 relativo)
        // seria a régua a mentir na quarta casa, para sempre. (2) É **ela** que torna o *reduced
        // motion* byte-idêntico ao mundo pré-wave: com a lei instantânea o `animate` devolve o
        // alvo no mesmo quadro, e este ramo publica-o **verbatim**.
        //
        // ⚠️ E o (2) foi aprendido por uma mutação que SOBREVIVEU: eu tinha escrito um atalho
        // `if law(Surface).is_none() { return Some(target) }` e um doc a dizer que ele existia
        // para evitar a ida-e-volta pelo log. Removê-lo não sangrou gate nenhum — porque este
        // ramo já entregava exactamente a mesma coisa. *Um caso especial que nenhuma medição
        // distingue do caso geral é um caso especial que não existe.*
        let next = if now == goal {
            // Assentou (a track põe o valor EXACTO e larga o voo) — a câmera volta a ser de quem
            // a escrever.
            self.target = None;
            target
        } else {
            // ⚠️ **Sem clamp no VIVO, de propósito.** O destino já é cravado, e a mola de
            // `Surface` é criticamente amortecida ⇒ não ultrapassa: o vivo está no intervalo por
            // construção. Cravá-lo aqui só saltaria a imagem quando o gesto parte de uma câmera
            // que outro dono deixou fora da faixa.
            now.exp()
        };
        self.published = next;
        Some(next)
    }

    /// O destino em voo, para os gates e para o diagnóstico. `None` = ninguém está a dar zoom.
    #[cfg(test)]
    pub(crate) fn target(self) -> Option<f32> {
        self.target
    }

    /// Onde o zoom VAI estar: o destino em voo, ou o vivo quando a câmera não é deste módulo. É
    /// sobre isto que a pinça (F6) mede o que o entalhe dela mudou.
    pub(crate) fn goal(self, live: f32) -> f32 {
        self.target.unwrap_or(live)
    }
}

/// ⭐ **A câmera do quadro** — a roda e o voo de um enquadramento (F8), e o gesto mais NOVO ganha.
pub(crate) fn publish(
    zoom: &mut CanvasZoom,
    flight: &mut ph2d_render::camera_frame::ViewFlight,
    camera: &mut Camera2d,
    motion: &mut UiMotion,
) {
    if flight.arbitrate(zoom.target.is_some()) {
        zoom.target = None;
    }
    if let Some(next) = zoom.tick(camera.height_world, motion) {
        camera.height_world = next;
    }
    let step = |g| motion.animate(VIEW_FLIGHT, g, Role::Viewpoint);
    if let Some(v) = flight.tick(camera.view(), step) {
        camera.set_view(v);
    }
}

#[cfg(test)]
#[path = "canvas_zoom_tests.rs"]
mod tests;
