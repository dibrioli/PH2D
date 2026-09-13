//! **Fase do quadro: A TIRA E O CURSOR DO FLIP** — o arrasto da tira vira documento, o espelho da ferramenta Flip (activa e estilo) e o anel
//! do pincel no canvas (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_flip_strip_and_cursor(
        &mut self,
        window_size: ph2d_host::WindowSize,
    ) -> Option<(bool, Option<ph2d_tool_flip::FlipStyleSnapshot>)> {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            camera,
            tools,
            vector_scene,
            flip,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let hero = hero_screen.as_mut()?;
        // O ARRASTO da tira (mover a chave / esticar o hold): o painel enfileirou o
        // pedido no pen-up do frame anterior; aqui ele vira documento — ANTES do
        // publish, senão o snapshot deste frame descreveria a tira de antes do gesto e
        // a célula piscaria de volta por um frame.
        // (o retorno diz se o documento mudou; ninguém precisa dele aqui — o undo é
        // GLOBAL e por DIFF: `post_frame_undo` compara o `ProjectState`, do qual o
        // `FlipDoc` faz parte. É o mesmo motivo pelo qual o drain do `PanelEvent`
        // logo acima também ignora o seu.)
        let _ = ph2d_app_flip::strip_drag::apply_strip_intents(
            flip,
            self.flip_state.active_layer,
            &mut self.flip_state.strip,
        );
        let (flip_active, flip_style) = ph2d_app_flip::bridge::publish(
            hero,
            tools,
            flip,
            self.flip_state.active_layer,
            &self.playhead,
            &self.flip_state.strip,
        );
        self.flip_state.active = flip_active;
        self.flip_state.style = flip_style;
        // O anel do pincel (W5): mostra no canvas o tamanho do que vai acontecer.
        // Depois do publish (o estilo do frame já está no cache) e na cena de
        // overlay, como o anel do Painter.
        ph2d_app_flip::cursor::draw_flip_cursor(
            flip_active,
            flip_style,
            hero,
            vector_scene,
            self.last_pointer,
            // §4.C.6: o Size mede o MUNDO — o anel se projeta pelo zoom, como a tinta.
            f64::from(window_size.height as f32 / camera.height_world.max(f32::EPSILON)),
        );
        Some((flip_active, flip_style))
    }
}
