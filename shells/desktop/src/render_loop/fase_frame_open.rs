//! **Fases do quadro: o MEIO do prelúdio e o CHÃO do quadro** (`line/render-bodies`, 2026-09-13) — corpos movidos
//! do `run_render_frame` pela ordem de sempre. A única troca é o fim antecipado: onde o quadro escrevia
//! `let Some(p) = f() else { return; };`, a fase escreve `let p = f()?;`, e o quadro acaba quando ela devolve `None`.

use super::*;

/// O que o chão do quadro entrega ao resto dele.
pub(super) struct FrameCanvas {
    /// A cor de limpar da camada das sprites (ver o corpo da `fase_frame_canvas`).
    pub(super) r: f64,
    pub(super) g: f64,
    pub(super) b: f64,
    pub(super) window_size: ph2d_host::WindowSize,
    pub(super) viewport: EditorRect,
    /// Há `HeroScreen`? (Só o `PH2D_M5_DEMO=1` o dispensa.)
    pub(super) hero: bool,
}

impl crate::App {
    /// Os relógios do passo fixo, a timeline, a física, os sinais e o extract das sprites — o meio do prelúdio.
    pub(super) fn fase_frame_simulation(
        &mut self,
        wall_dt: f64,
        player_input: ph2d_physics_ecs::PlayerInput,
    ) -> Option<(ph2d_core::FixedStepReport, [Option<u64>; 3])> {
        let fase_fixed_step_clocks::FrameClocks {
            report,
            tool_preview_bits,
            anim_signals,
            timer_signals,
            deaths,
        } = self.fase_fixed_step_clocks(wall_dt)?;
        self.fase_scene_audio();
        // ⭐ **O rectângulo da câmera do JOGO atravessa a fronteira** (TOP-20 #12): quem o conhece é
        // esta fase, e quem o lê é o dreno das mortes, lá em baixo. ⛔ Uma segunda leitura da câmera
        // no dreno seria a segunda resposta a *«qual é a vista?»*, e as duas divergiriam no dia em
        // que uma delas mudasse.
        let camera_rect = self.fase_game_camera(player_input, report);
        // ⭐⭐⭐ **O HUD** (TOP-20 #20) — a raiz de um placar cola-se a` vista que a linha de
        // cima acabou de calcular. Ver o cabecalho da fase: ela tem de correr DEPOIS da camera
        // e ANTES do extract, e as duas metades sao load-bearing.
        self.fase_hud(camera_rect);
        let fase_extract_inputs::ExtractInputs {
            dt,
            preview_overrides,
            sheet_preview,
            ppm,
            default_filter,
        } = self.fase_extract_inputs(wall_dt)?;
        let fase_timeline_view::TimelineView {
            container,
            maos,
            keys_mode,
            selected_now,
        } = self.fase_timeline_view()?;
        self.fase_timeline_containers(container, keys_mode);
        self.fase_timeline_drain(container, &maos, keys_mode, selected_now);
        // ⭐⭐⭐ **A PARALAXE** (plano 24, W1) — o fundo fica para tra's da mesma vista. Ver o
        // cabecalho da fase: depois da camera e do HUD, DEPOIS do dreno da timeline (e' ele que
        // aplica o scrub e o rebobinar, e a deriva le o relogio — antes dele um scrub chegava um
        // quadro atrasado, auditoria 26 §3) e ANTES do extract.
        self.fase_paralaxe(camera_rect);
        self.fase_physics_step(player_input);
        self.fase_signal_outbox(
            anim_signals,
            timer_signals,
            deaths,
            camera_rect,
            report.ticks,
        );
        self.fase_open_recipe();
        // ⭐⭐⭐ **OS MOTORES DE POSE DO ESQUELETO ANTES DO EXTRACT, e a ordem é load-bearing** (report
        // do dono, 2026-09-14): a malha de uma imagem presa é posta na `fase_sim_extract`, logo o
        // osso inteligente e a âncora de IK têm de já ter escrito a pose. Ao contrário — que era
        // onde eles viviam — o gizmo mostra a pose resolvida e a ARTE mostra a curva que o apply
        // acabou de repor, para sempre.
        self.fase_skeleton_drives();
        self.fase_sim_extract(dt, preview_overrides, sheet_preview, ppm, default_filter);
        Some((report, tool_preview_bits))
    }

    /// O chão do quadro: a cor de limpar, o tamanho da janela, a viewport e a cena de widgets reposta.
    pub(super) fn fase_frame_canvas(&mut self) -> Option<FrameCanvas> {
        let gfx = self.gfx.as_mut()?;
        // O empréstimo do `gfx` do quadro: o padrão EXAUSTIVO do `AppGfx` mora em [`frame_gfx`]; aqui
        // ficam só os campos que o resto do corpo ainda lê (os das fases já extraídas saem da lista).
        let FrameGfx {
            #[cfg(feature = "sculpt3d")]
            surface,
            theme,
            vector_scene,
            hero_screen,
            hero_live,
            ..
        } = FrameGfx::of(gfx);

        // Sprite-layer clear color = backdrop visible in the canvas
        // area through the transparent regions of `vello_rt`. Live
        // editor mode wants a static neutral surface so it doesn't
        // pulse rainbow under the chrome.
        //
        // ⭐⭐ A cor sai da PORTA (`canvas_clear::canvas_clear_rgb`),
        // que a deriva do mesmo token que o painter do canvas e o
        // cartão do navegador de assets lêem — era um literal
        // `(0.047, 0.047, 0.055)`, cópia à mão do `Bg1` do Forge, e
        // enquanto foi cópia mudar a cor do canvas movia o resto do
        // app e deixava o canvas onde estava.
        //
        // ⛔ A conversão sRGB→linear continua deliberadamente por
        // fazer (o byte divide-se por 255): é a regressão dos
        // "pixelated borders" da M14.5 ronda 2, medida e revertida.
        // O mecanismo inteiro está no cabeçalho do `canvas_clear`.
        let (r, g, b) = if hero_live.is_some() {
            crate::canvas_clear::canvas_clear_rgb(*theme)
        } else {
            let t = self.fixed_step.tick_count() as f64 * self.fixed_step.fixed_dt();
            (
                (t.sin() * 0.05 + 0.05).clamp(0.0, 1.0),
                ((t + 2.094).sin() * 0.05 + 0.05).clamp(0.0, 1.0),
                ((t + 4.188).sin() * 0.05 + 0.05).clamp(0.0, 1.0),
            )
        };

        let window_size = surface.size();
        // M11: build the widget scene up-front (no GPU work yet — just
        // VectorScene encoding). Done outside acquire_frame so an
        // Occluded/Timeout doesn't waste the encoder.
        let viewport = EditorRect::new(
            0.0,
            0.0,
            window_size.width as f32,
            window_size.height as f32,
        );
        vector_scene.reset();

        // Default editor mode: AppGfx owns a HeroScreen with a
        // retained WidgetStore (ADR-0024). Paint reads + writes its
        // hit_index each frame; pointer/key events are forwarded to
        // it from window_event handlers via `hero_screen.handle_*`.
        // `hero_screen` is `None` only under `PH2D_M5_DEMO=1`.
        // ⚠️ A pergunta é só «há `HeroScreen`?»: o último leitor do `hero` foi para a `fase_snapshots_publish` (P7c), e
        // cada fase deste bloco re-deriva o seu.
        Some(FrameCanvas {
            r,
            g,
            b,
            window_size,
            viewport,
            hero: hero_screen.is_some(),
        })
    }
}
