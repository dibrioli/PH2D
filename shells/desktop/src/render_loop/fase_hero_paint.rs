//! **Fase do quadro: A PINTURA DO ECRÃ HERO** — o `paint_hero_screen` (painéis, chrome, gizmos), a arrumação
//! gravada quando muda, o relógio do `hero-paint` do perfilador e o rectângulo de selecção por cima (OBRA 2 da `line/render-loop`, 2026-09-12).

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo.
    pub(super) fn fase_hero_paint(&mut self, viewport: EditorRect) {
        // O `gfx` re-derivado; os guardas do quadro já correram na `fase_chrome_clock`.
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let FrameGfx {
            theme,
            vector_scene,
            text_system,
            hero_screen,
            ..
        } = FrameGfx::of(gfx);
        // O bloco do quadro só chama esta fase com o `HeroScreen` vivo.
        let Some(hero) = hero_screen.as_mut() else {
            return;
        };
        let paint_ctx = PaintCtx {
            theme: *theme,
            viewport,
            text: text_system,
        };
        // Frame profiler: panel/chrome Vello encode (includes the painter panel's Paper preview).
        let hero_t0 = frame_prof_on().then(Instant::now);
        paint_hero_screen(hero, viewport, vector_scene, paint_ctx.text);
        // ⭐⭐ **A ARRUMAÇÃO é detectada no QUADRO, não no hook de ponteiro** (decisão D4).
        //
        // ⛔⛔ Ela viveu no `forward_to_hero` durante uma entrega, com os outros dois
        // inquilinos da persistência — e **não funcionava para a largura da coluna**: o
        // arrasto da borda faz `return` no Move E no Up (`input_dispatch`), então nunca
        // alcançava o detector. O mesmo valia para a largada de uma aba, que é resolvida
        // DENTRO do `paint`. *Um detector no caminho de um gesto só vê os gestos que passam
        // por ele; o quadro vê todos, porque é onde o estado assenta.*
        crate::layout_persist::save_if_changed(hero);
        if let Some(t0) = hero_t0 {
            FRAME_PROF_HERO_US.with(|c| c.set(t0.elapsed().as_micros() as u64));
        }
        // Audio Editor floating waveform overlay (docs/Audio/, W1) — painted
        // after the hero chrome, in the Hierarchy↔Inspector gap. Reads the
        // loaded clip from the audio system; no-op when the panel is closed
        // or no clip is loaded.
        #[cfg(feature = "panel-audio-editor")]
        if let Some(audio) = self.audio.as_mut() {
            audio_overlay::draw_audio_overlay(
                hero,
                audio,
                ph2d_editor_core::zones::Rect::new(viewport.x, viewport.y, viewport.w, viewport.h),
                vector_scene,
                paint_ctx.text,
            );
        }
        // Fase 0f: overlay the active rubber-band rect on top of
        // everything (panels, gizmo, hero chrome). Pure shell
        // concern — coords stay in screen space so the rect
        // doesn't shift if the camera pans mid-drag. Semi-
        // transparent fill + 4 thin border rects (no stroke API
        // on VectorScene yet; the 4-fills idiom matches the rest
        // of the shell's overlay painters).
        if let Some(rb) = self.rubber_band {
            let (ax, ay) = rb.anchor_screen;
            let (cx, cy) = rb.current_screen;
            let x0 = ax.min(cx) as f64;
            let y0 = ay.min(cy) as f64;
            let x1 = ax.max(cx) as f64;
            let y1 = ay.max(cy) as f64;
            use ph2d_vector::{Color, Rect as VRect};
            // Selection accent — design tokens use OKLCH; the
            // sRGB approximation here is the canonical Selection
            // color from `ColorToken::Selection` (~#3a8ee6 @ 25%
            // fill, 100% border) baked at boot. Keeping it inline
            // avoids threading the theme into render_loop just
            // for one overlay; a follow-up can swap to a token
            // lookup if the rubber-band needs theme parity.
            let fill = Color::new([0.23, 0.56, 0.90, 0.18]);
            let border = Color::new([0.23, 0.56, 0.90, 1.0]);
            vector_scene.fill_rect(VRect::new(x0, y0, x1, y1), fill);
            vector_scene.fill_rect(VRect::new(x0, y0, x1, y0 + 1.0), border);
            vector_scene.fill_rect(VRect::new(x0, y1 - 1.0, x1, y1), border);
            vector_scene.fill_rect(VRect::new(x0, y0, x0 + 1.0, y1), border);
            vector_scene.fill_rect(VRect::new(x1 - 1.0, y0, x1, y1), border);
        }
    }
}
