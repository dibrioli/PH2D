//! **Fase do quadro: OS GIZMOS DO MOTION QUE LEEM O COZIDO** — o retrato do colisor da forma e o
//! dos deformadores de quadrilátero, resolvidos DEPOIS de o Motion cozinhar (report do dono,
//! 2026-09-18: *«melhorou em relação à colisão mas tem um atraso antigo do gizmo em relação à
//! imagem»*).
//!
//! ## Porque esta fase existe
//!
//! Os dois retratos saem das TOMADAS (`pump.tap_streams()`), e as tomadas só existem depois do
//! cook. Enquanto eles eram publicados na `fase_gizmo_views_and_prefab` — o prólogo — liam o cozido
//! do quadro **ANTERIOR**, enquanto a arte é encodada no fim e mostra o cozido deste. *Um quadro
//! inteiro de atraso, visível só com a cena em movimento.*
//!
//! Medido no texto do quadro emendado (`frame_text::render_frame`), antes da cura:
//!
//! | literal | posição |
//! |---|---|
//! | `collider_gizmo::resolve_at(` (o retrato) | **190 082** |
//! | `motion_bridge::dispatch(` (o cook) | 482 136 |
//! | `collider_gizmo_overlay::draw(` (o desenho) | 662 982 |
//!
//! ⚠️ **O DESENHO já estava no sítio certo** — a leitura estrutural que eu fiz primeiro (*«o
//! overlay é construído antes do cook»*) foi **refutada pela medição**: só o retrato era cedo. É
//! por isso que a cura é mover o RESOLVE e não o desenho, e é por isso que a régua imprime os três
//! números em vez de dois.
//!
//! ## Quem NÃO se mudou, e porquê
//!
//! O gizmo do **field espacial** fica no prólogo: ele lê os params do nó, nunca uma tomada
//! (medido — nenhum ficheiro `field_gizmo*` menciona `tap_streams`). *Mover o que não tem o defeito
//! só alarga o diff.*

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_motion_gizmos(&mut self, window_size: ph2d_host::WindowSize) {
        // ⭐ **A modalidade vem da PORTA que já existe** (`App::motion_tool_active`), e é lida
        // ANTES do empréstimo do `gfx` — é isso que a torna chamável aqui. O prólogo deriva-a de
        // um local (`tools`) porque ali o `gfx` já está emprestado, e o comentário dele di-lo.
        // *Duas respostas à mesma pergunta divergem no dia em que uma mudar; esta é a mesma.*
        let motion_tool_active = self.motion_tool_active();
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            camera,
            hero_screen,
            motion,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // **O gizmo dos DEFORMADORES DE QUADRILÁTERO** (Corner Pin + Bezier Warp).
        // ⚠️ Publicar de novo SUBSTITUI, então largar a selecção limpa as alças em vez de as
        // deixar a pairar.
        ph2d_app_motion::warp_gizmo::publish(ph2d_app_motion::warp_gizmo::resolve(
            motion,
            motion_tool_active,
        ));
        // ⭐⭐⭐ **O gizmo de uma corrente de POSIÇÕES** — a mesma janela e a mesma modalidade.
        // Ele lê as tomadas deste cozimento e pergunta, por sink, a MESMA coisa que o lowering
        // pergunta (`tem_aparencia`): o que não veio de uma forma não vira pixel, vira gizmo.
        ph2d_app_motion::ponto_gizmo::publish(ph2d_app_motion::ponto_gizmo::resolve(
            motion,
            motion_tool_active,
            // ⭐ A PORTA DO PRODUTO, lida num sítio só — ver `ponto_gizmo::resolve`: ou se vêem
            // as peças, ou se vê o gizmo.
            ph2d_eval_motion::so_com_forma_por_ordem(),
        ));
        // ⭐⭐ **O gizmo do PIVÔ da forma** (ordem do dono, 2026-09-19: *«permita visualizar o
        // ponto do pivot ao arrastar os parâmetros de pivot»*) — a mesma janela e a mesma
        // modalidade. Ele acende só enquanto a mão arrasta o knob, e o que desenha é o `P` de
        // cada peça: *o pivô de uma instância É a posição dela.*
        ph2d_app_motion::pivot_gizmo::publish(ph2d_app_motion::pivot_gizmo::resolve(
            motion,
            motion_tool_active,
        ));
        // **O gizmo do COLISOR da forma** (doc 109 §5) — a mesma modalidade.
        ph2d_app_motion::collider_gizmo::publish(ph2d_app_motion::collider_gizmo::resolve_at(
            motion,
            motion_tool_active,
            camera,
            hero.view.center_split,
            window_size,
            self.last_pointer,
        ));
    }
}
