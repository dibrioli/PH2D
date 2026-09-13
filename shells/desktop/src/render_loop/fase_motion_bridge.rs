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
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // Motion Nodes M0.T10: same phase as vector_bridge (AFTER the
        // ActivateTool drain, so a freshly-activated tool is seen this frame;
        // BEFORE paint + present, so the split/panel visibility it sets and
        // the instances it cooks both land this frame). Cooks the graph into
        // `motion.instances` (present injects them via `render_with_extra`)
        // and drives the center split + docked-panel visibility.
        // The drawn shapes go into the cook BEFORE it runs (doc 65): every named vector path
        // becomes an external the graph can walk (`motion.path`). Here, because this is the
        // one place the document, the world, the entity map and the transforms are all in
        // hand at once.
        ph2d_app_motion::motion_bridge::publish_shapes(
            motion,
            sim,
            vec_scene,
            &self.vec.entities,
            &vec_xf_ops,
            hero.gizmo.selection,
        );
        // ADR-0154: `source.shape` geometry is NOT published here — it is published
        // by the bridge POST-drain, pre-cook (a param edit is drained inside the
        // bridge, so publishing here would set the pre-edit key while the cook reads
        // the post-edit key ⇒ a 1-frame vanish = flicker). See `motion_bridge`.
        // doc 86 §2: and the engine OBJECTS (named sprites) into the same
        // external table — AFTER shapes (which clears it), so the cook sees
        // both curves and objects. The atlas resolves each sprite's tile.
        // ⚠️ **As DUAS lojas de aparência**, não só o atlas — ver `Appearance`: um sprite
        // KTX2 assado era fonte INVISÍVEL só porque quem o resolvia não estava em mão
        // aqui dentro, e ele está: é o mesmo `renderer` de onde sai o atlas.
        let cooked = |id| renderer.cooked_texture_id(id);
        ph2d_app_motion::motion_bridge::publish_objects(
            motion,
            sim,
            ph2d_app_motion::motion_bridge::Appearance {
                atlas: renderer.atlas(),
                cooked: &cooked,
            },
            // ⚠️ O relógio, porque o canal DESLOCADO resolve um param que pode vir de um
            // fio — e um param conduzido só tem valor num INSTANTE (doc 58).
            self.playhead.time(),
        );
        // ...and the CURSOR, last, into the same table (`ph2d_nodegraph::external`).
        // It is not a document value — it is an editor input that changes every
        // frame — so publishing it is what lets `motion.look_at` aim at the mouse
        // without the node learning what a window or a camera is. Last, because
        // `publish_shapes` CLEARS and the objects append; and in the reserved `$`
        // namespace, which the artist-name publishes above refuse.
        ph2d_app_motion::motion_bridge::publish_cursor(
            motion,
            camera,
            self.last_cursor,
            hero.view.center_split,
            surface.size(),
        );
        ph2d_app_motion::motion_bridge::dispatch(
            hero,
            tools,
            motion,
            &mut self.playhead,
            self.fixed_step.fixed_dt(),
            self.last_pointer,
            toasts,
            surface.gpu(),
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
