//! **Fase do quadro: O MOTION: PUBLICAR, DESPACHAR E OS SINAIS** — as formas, os objectos e o cursor publicados ao grafo, o despacho do Motion e os sinais
//! que ele grita (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_motion_bridge(&mut self, vec_xf_ops: ph2d_vec_scene::VecXforms) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            surface,
            renderer,
            sim,
            camera,
            toasts,
            tools,
            vec_scene,
            hero_screen,
            motion,
            text_system,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // ⭐ O Motion como OBJECTO (doc ObjectsAndComponents 01 §13): donos, migração, purga, cópias.
        let feito = ph2d_app_motion::motion_objeto::upkeep(
            motion,
            sim.world_mut(),
            hero.gizmo.selection,
            self.playhead.is_playing(),
        );
        self.title_dirty |= feito != Default::default();
        // Motion Nodes M0.T10: same phase as vector_bridge (AFTER the
        // ActivateTool drain, so a freshly-activated tool is seen this frame;
        // BEFORE paint + present, so the split/panel visibility it sets and
        // the instances it cooks both land this frame). Cooks the graph into
        // `motion.instances` (present injects them via `render_with_extra`)
        // and drives the center split + docked-panel visibility.
        // O mundo vai para o cozimento ANTES dele (doc 65, doc 86 §2): os caminhos do Vetor por
        // nome, os objectos (as DUAS lojas de aparência, o mesmo `renderer` do atlas), as entradas
        // do jogo, o cursor e a vista — aqui, o único sítio com o documento, o mundo, o mapa e as
        // afins em mão. A ordem e o retrato que cada palco de uma macro recebe vivem na porta.
        let cooked = |id| renderer.cooked_texture_id(id);
        ph2d_app_motion::motion_mundo::publica_o_mundo(
            motion,
            ph2d_app_motion::motion_mundo::Cena {
                sim,
                vec_scene,
                entidades: &self.vec.entities,
                xforms: &vec_xf_ops,
                selecao: hero.gizmo.selection,
                aparencia: ph2d_app_motion::motion_bridge::Appearance {
                    atlas: renderer.atlas(),
                    cooked: &cooked,
                },
                segundos: self.playhead.time(),
                camera,
                cursor: self.last_cursor,
                split: hero.view.center_split,
                janela: surface.size(),
            },
        );
        ph2d_app_motion::motion_bridge::dispatch(
            hero,
            tools,
            motion,
            &mut self.playhead,
            self.fixed_step.fixed_dt(),
            // ⭐ O relógio de PAREDE — o que anda com a cena pausada, e o único em que o eco de
            // uma largada no grafo pode viver (o playhead pára com o Play).
            self.fixed_step.wall_seconds(),
            self.last_pointer,
            toasts,
            surface.gpu(),
        );
        ph2d_app_motion::motion_demo_enquadra::abre_inteira(
            motion,
            hero,
            camera,
            crate::scene_mapping::janela(hero.view.center_split, surface.size()),
            text_system,
        );
        // O grafo gritou — o shell é quem publica (ADR-0075: o produtor não chama
        // ninguém). ⚠️ **Este produtor pousa UM QUADRO atrás dos outros dois**, e o
        // fato fica NOMEADO em vez de escondido: o dispatch de Motion roda depois de
        // os consumidores lerem, então um `pulse.signal` chega ao toast no quadro
        // seguinte. O duplo-buffer da outbox torna isso *atrasado, nunca perdido* — a
        // rede, não a licença. Fechar o vão é MOVER a leitura dos consumidores para
        // baixo deste dispatch, o que reordena uma sequência gateada e é decisão
        // própria (o `toasts` tem de continuar vivo lá).
        // ⚠️ **A LEITURA acontece AQUI, e não dentro do dispatch, porque este é o ponto
        // onde TODA rota de cook converge.** Ela morava no laço de tiques da bomba de
        // CPU — correto, e mudo: a rota da GPU **híbrida** marcha por outra porta e
        // devolve `Handled`, então aquele laço nem roda. Medido na cena `=26`, que planeja
        // híbrida (boundaries `[5, 4]`, 4 estágios): o grafo cozinhava, desenhava e não
        // gritava nada — com a suíte verde, porque todo gate dirigia a porta de sinks.
        //
        // ⚠️ **A LEI mora aqui junto com a leitura**, pelo mesmo motivo: perguntá-la
        // dentro do dispatch obrigaria cada rota a lembrar-se dela. Um scrub re-cozinha o
        // grafo e não pode gritar — um sinal é travessia de play para a frente.
        if clock_forward::clock_is_playing_forward(&self.playhead, self.timeline_signals.jumped) {
            ph2d_app_motion::motion_bridge::signals::collect_signals(motion);
        }
        for sig in motion.signals_out.drain(..) {
            self.signals.publish(ph2d_runtime::Signal::from_motion(
                &sig.name, sig.tick, sig.rows,
            ));
        }
    }
}
