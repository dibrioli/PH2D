//! **Fase do quadro: O RELÓGIO DO CHROME** — o `wall_dt` do quadro (um dono só), o `ui_dt` que desconta
//! o congelamento de um diálogo modal, e os tiques que andam nele: o Zen, a poeira, a UI viva e o zoom
//! do canvas, os toasts, as barras de trabalho e o passo contínuo do stepper (OBRA 2 da
//! `line/render-loop`, 2026-09-12).
//!
//! ⚠️ **A primeira fase com o `gfx` emprestado, e por isso a que carrega os dois GUARDAS do quadro**:
//! sem `gfx` ou sem `host` ela devolve `None` antes de qualquer tique, e o `run_render_frame` acaba
//! ali — exactamente o que os dois `let Some(..) else { return; }` do topo do bloco faziam.

use super::*;

impl crate::App {
    /// Ver o cabeçalho do módulo. Devolve o `wall_dt` do quadro, que o acumulador da sim lê mais abaixo.
    pub(super) fn fase_chrome_clock(&mut self) -> Option<f64> {
        // Os dois guardas do quadro (ver o cabeçalho): nada abaixo corre sem `gfx` e sem `host`.
        let gfx = self.gfx.as_mut()?;
        let FrameGfx {
            camera,
            canvas_zoom,
            zen,
            toasts,
            jobs,
            hero_screen,
            hero_arena,
            ..
        } = FrameGfx::of(gfx);
        self.host.as_ref()?;

        // M12 per-frame ticks: ZenMode debounce cooldown + ToastQueue
        // TTL decay. Both are pure data-layer (no Vello paint here).
        // ⚠️ **O `wall_dt` é calculado AQUI, uma vez, e tem UM dono.** Ele vivia 400 linhas abaixo,
        // junto do acumulador da sim, e o `ToastQueue` — o único relógio do chrome — contava
        // QUADROS por não o alcançar (um toast de "3 s" durava 6 a 30 fps). Subi-lo é mais barato
        // que dar ao chrome um segundo `Instant::now()`: duas respostas a *"quanto durou o último
        // quadro?"* divergiriam, e a que o artista vê seria a errada.
        let now = Instant::now();
        let wall_dt = now.duration_since(self.last_frame).as_secs_f64();
        self.last_frame = now;
        // ⭐ **O tempo em que um DIÁLOGO MODAL congelou o loop não é tempo de animação.** O
        // `wall_dt` acima mede o quadro INTEIRO, e um `rfd::FileDialog` acontece *dentro* dele — de
        // modo que o quadro seguinte cobrava ao chrome os 20 s em que a tela esteve parada. O
        // sintoma, com as palavras do Enio (2026-08-22): *"não vejo em nenhum lugar a mensagem"* —
        // o toast escrito logo depois do diálogo era pintado UM quadro e morria no `tick` seguinte.
        //
        // ⚠️ **Não é um segundo relógio nem um teto mágico:** é a MESMA medição com a parte parada
        // nomeada por quem a causou (`ph2d_app_host::modal`). O medidor de fps e o acumulador da sim
        // continuam a ler o `wall_dt` inteiro, porque para eles o tempo passou mesmo.
        let ui_dt = ph2d_app_host::modal::chrome_dt(wall_dt, ph2d_app_host::modal::take_stall());

        zen.tick();
        // ⚠️ **A poeira anda no relógio do CHROME** (`ui_dt`), e não no do quadro: um diálogo modal
        // congela o laço, e uma faísca não pode envelhecer enquanto nada é desenhado — a mesma lei
        // que o `ph2d_app_host::modal` já impõe aos toasts.
        #[allow(clippy::cast_possible_truncation)]
        self.ui_burst.tick(ui_dt as f32);
        // ⚠️ **A UI VIVA anda aqui, com o MESMO relógio dos toasts** — um segundo `Instant::now()`
        // para o chrome seria a segunda resposta a *"quanto durou o último quadro?"*, e a que o
        // artista vê seria a errada. O `ui_dt` **não** é esse segundo relógio: é o `wall_dt` com a
        // parte CONGELADA descontada, e as duas leis (animar · medir) leem a mesma medição. Sem consumidor de `hover_t` a chamada é neutra: o mapa fica vazio.
        if let Some(hero) = hero_screen.as_mut() {
            hero.tick_motion(ui_dt);
            // ⚠️ **O zoom do canvas é publicado AQUI, e a posição é load-bearing:** depois do
            // `tick_motion` (que é quem anda o relógio deste quadro) e **antes** de qualquer
            // extract, pintura, gizmo ou picking. A câmera é lida por ~90 sítios; publicá-la no
            // meio do quadro faria a imagem e o cursor discordarem sobre onde o mundo está.
            if let Some(next) = canvas_zoom.tick(camera.height_world, &mut hero.motion) {
                camera.height_world = next;
            }
        }
        let prev_toasts = toasts.len();
        #[allow(clippy::cast_possible_truncation)]
        toasts.tick(ui_dt as f32);
        if toasts.len() != prev_toasts {
            self.title_dirty = true;
        }
        // Long-operation bars: drop the ones whose worker has stopped. Same once-per-frame
        // settle as the toasts above, and the same reason it lives here rather than at the
        // paint site — a queue that only prunes when someone draws it is a queue that leaks
        // on any frame that is skipped.
        jobs.tick();

        // M14.A: drive the NumberInput stepper continuous-hold. Each
        // frame we ask the dispatcher whether a held arrow should
        // fire one more `ValueChanged` (initial 250 ms delay, then
        // 30 ms repeat). Events drained through `hero.apply_event` so
        // a Transform field that's been incrementing flows through
        // the same commit path as a Enter/blur — the EditorCommand
        // pipeline drain below picks it up.
        if let Some(hero) = hero_screen.as_mut() {
            let tick_events: Vec<WidgetEvent> =
                ph2d_editor_core::dispatch_tick(hero_arena, &mut hero.store, Self::timestamp_ns())
                    .to_vec();
            for e in tick_events {
                let _ = hero.apply_event(e);
            }
        }
        Some(wall_dt)
    }
}
