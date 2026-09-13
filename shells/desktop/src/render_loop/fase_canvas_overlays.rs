//! **Fase do quadro: AS SOBREPOSIÇÕES DO CANVAS** — a escultura na cena, a trajectória do objecto seleccionado e as poses-fantasma do onion da
//! timeline (OBRA 2 da `line/render-loop`, 2026-09-13).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_canvas_overlays(&mut self, window_size: ph2d_host::WindowSize) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            #[cfg(feature = "sculpt3d")]
            sculpt3d,
            sim,
            present,
            camera,
            vector_scene,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        // **O ANEL DO PINCEL 3D** (ADR-0150 W12). Ele é desenhado no PONTO DE
        // ACERTO reprojetado, então ele é ao mesmo tempo a mira e o
        // instrumento: se ele não estiver debaixo do mouse sobre o barro, a
        // fiação do pick está errada e dá para VER — que é a única coisa que
        // as sondas headless não alcançam.
        //
        // ⚠️ Sob painel ele não é desenhado: o ponteiro ali não é da cena (o
        // `pointer_down` já recusa pela MESMA porta), e uma mira sobre o
        // chrome prometeria um gesto que o clique não faz.
        #[cfg(feature = "sculpt3d")]
        if let Some(scene) = sculpt3d.as_ref() {
            let (px, py) = self.last_pointer;
            let over_panel = hero
                .store
                .panel_rect(ph2d_editor_core::ids::SCULPT3D_PANEL)
                .is_some_and(|r| r.contains(px, py));
            if !over_panel && let Some(mark) = scene.cursor_mark(px, py) {
                use ph2d_vector::{Affine, Brush, Color, Stroke};
                let rgba = if mark.on_surface {
                    ph2d_app_sculpt3d::ON_SURFACE_RGBA
                } else {
                    ph2d_app_sculpt3d::OFF_SURFACE_RGBA
                };
                vector_scene.inner_mut().stroke(
                    // ⚠️ `Affine::IDENTITY`: no Vello o transform do `stroke`
                    // MULTIPLICA a largura — o caminho já está em pixels.
                    &Stroke::new(1.5), // LITERAL-PX-OK: chrome de overlay, espessura de tela
                    Affine::IDENTITY,
                    &Brush::Solid(Color::new(rgba)),
                    None,
                    &mark.path,
                );
            }
        }
        // A TRAJETÓRIA do objeto selecionado (ADR-0141): um binding Position guarda
        // uma curva, e sem desenhá-la o artista vê o objeto aparecer noutro lugar a
        // cada frame sem ter onde pegar o caminho. Os PONTOS são um por quadro, e o
        // espaçamento entre eles é a velocidade. No-op sem seleção ou sem Position.
        // ⚠️ **Só na aba Keys** (Enio, 2026-07-31) — a trajetória é do CLIP ATIVO, e
        // fora dali quem dirige o objeto é a PILHA; o MESMO booleano que sola o clip
        // (`keys_mode`) decide se há alça a oferecer. Um `true` literal aqui deixaria
        // todo gate do overlay verde com a alça fantasma de volta na tela — daí o
        // arch-gate `the_motion_path_is_offered_only_on_the_keys_tab`.
        ph2d_app_motion::motion_path_overlay::draw(
            self.timeline.keys_mode,
            &self.timeline.doc,
            hero.gizmo.iter_selected().next(),
            camera,
            window_size,
            vector_scene,
        );
        // O ONION da timeline (ADR-0142): as poses-fantasma do objeto selecionado em
        // t±k, cozidas AQUI (temos sim/present/doc/seleção) e desenhadas pelo passe de
        // sprite em `run_present_phase` — o padrão do Motion (cozinha numa fase,
        // desenha noutra). No-op quando desligado / sem seleção animada.
        // Onion settings modal (ADR-0142 W3b): while the card is open, read its slider/swatch
        // values back into the onion each frame — live edits on the canvas (the ghost pass
        // below re-reads `self.timeline.onion`). No-op when closed. The store is the shared
        // blackboard; `enabled`/`mode` stay owned by the transport toggles.
        crate::onion_modal::read_into(&hero.store, &mut self.timeline.onion);
        self.onion_ghosts.clear();
        timeline_onion::collect_onion_ghosts(
            &self.timeline.onion,
            sim.world(),
            present,
            &self.timeline.doc,
            hero.gizmo.iter_selected().next(),
            self.playhead.time(),
            &mut self.onion_ghosts,
        );
    }
}
