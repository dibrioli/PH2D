//! ⭐ **O palco do jogo** (GameEngineFuncional R0b/R0c, `docs/GameEngineFuncional/04_R0_N0_plano.md`).
//!
//! Em ecrã cheio e numa janela o jogo é visto como o jogador o vê: o canvas ocupa tudo (o zen) e a
//! câmara é a do jogo. Na janela o app MUDA-SE para ela enquanto o jogo corre — a janela, o host e a
//! surface trocam de lugar com os do editor uma vez ao abrir e outra ao fechar, e os alvos seguem pela
//! porta de resize de sempre. O quadro desenha-se uma só vez, lá; o editor fica com a última imagem
//! (a vista dupla ao vivo é o R1 do plano).

use ph2d_editor_core::game_stage::GameStage;
use ph2d_host::PlatformHost;
use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

/// A vista do editor antes do palco — o que a saída repõe (a câmara do jogo escreve na do editor).
#[derive(Clone, Copy, Debug)]
pub(crate) struct VistaAntes {
    camara_do_jogo: bool,
    camara: ph2d_render::Camera2d,
    pub(crate) stage: GameStage,
}

/// O editor, guardado enquanto o app vive na janela do jogo.
pub(crate) struct JanelaDoJogo {
    editor_window: Arc<Window>,
    editor_host: crate::winit_host::WinitHost,
    editor_surface: ph2d_gpu::SurfaceContext,
}

/// O tamanho com que a janela do jogo abre (pontos lógicos, 16:9).
const TAMANHO: winit::dpi::LogicalSize<u32> = winit::dpi::LogicalSize::new(1280, 720);

impl crate::App {
    /// A entrada no palco escolhido no menu *Run*. `None` = no editor: nada muda.
    pub(crate) fn palco_entra(&mut self) -> Option<VistaAntes> {
        let gfx = self.gfx.as_mut()?;
        let hero = gfx.hero_screen.as_mut()?;
        let stage = hero.view.game_stage;
        // A janela liga o palco quando ABRIR (`abre_a_janela_do_jogo`): este quadro ainda é desenhado
        // no editor, e é a imagem com que ele fica parado atrás do jogo.
        hero.view.palco_do_jogo = stage == GameStage::Fullscreen;
        let antes = VistaAntes {
            camara_do_jogo: self.game_camera_preview,
            camara: gfx.camera,
            stage,
        };
        self.game_camera_preview = true;
        Some(antes)
    }

    /// O palco está aberto? Então os auxiliares do editor (física, selecção, gizmos) não se desenham.
    pub(crate) fn no_palco(&self) -> bool {
        let hero = self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref());
        hero.is_some_and(|h| h.view.palco_do_jogo)
    }

    /// A saída: a vista do editor volta como estava. A janela fecha em [`Self::palco_janela`].
    pub(crate) fn palco_sai(&mut self, antes: VistaAntes) {
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.camera = antes.camara;
            if let Some(hero) = gfx.hero_screen.as_mut() {
                hero.view.palco_do_jogo = false;
            }
        }
        self.game_camera_preview = antes.camara_do_jogo;
    }

    /// Depois de cada quadro, com o event loop na mão: abre a janela que o palco pede e fecha-a quando
    /// o jogo pára.
    pub(crate) fn palco_janela(&mut self, event_loop: &ActiveEventLoop) {
        let quer =
            self.undo.jogo.foto().and_then(|f| f.vista).map(|v| v.stage) == Some(GameStage::Window);
        match (quer, self.undo.janela_do_jogo.is_some()) {
            (true, false) => self.abre_a_janela_do_jogo(event_loop),
            (false, true) => self.fecha_a_janela_do_jogo(),
            _ => {}
        }
    }

    /// `PH2D_GAME_SMOKE` (o jogo de plataforma do plano 03): a cena e a física ligada, com o relógio
    /// PARADO — o Play é do dono.
    pub(crate) fn game_smoke(&mut self) {
        let Some(nivel) = ph2d_app_game::smoke::armed_level() else {
            return;
        };
        if self.gfx.is_none() || !ph2d_app_game::smoke::monta_agora() {
            return;
        }
        if let Some(cx) = self.components_ctx() {
            let jogador = ph2d_app_game::smoke::montar(cx.sim, cx.registry, nivel).to_bits();
            if let Some(hero) = cx.hero_screen.filter(|_| matches!(nivel, 2 | 4)) {
                hero.gizmo.replace_selection(Some(jogador)); // o Inspector abre no personagem
                hero.store
                    .set_collapsed(ph2d_editor_core::ids::INSP_LIVE_PLAYER_SECTION, false);
            }
        }
        self.troca_de_imagem_do_smoke(nivel);
        self.timeline.flags.simulate_physics = true;
        self.playhead.rewind();
        self.playhead.pause();
    }

    /// O condutor do smoke (`PH2D_GAME_SMOKE_DRIVE`): os gestos pelas portas do produto.
    pub(crate) fn game_smoke_drive(&mut self) {
        use ph2d_app_game::smoke::{self as s, Gesto};
        let (Some(stage), Some(nivel)) = (s::conducao(), s::armed_level()) else {
            return;
        };
        match s::gesto_no_quadro(nivel, stage, s::proximo_quadro()) {
            Some(Gesto::Joga(stage)) => {
                if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                    use ph2d_editor_core::action_bus::{EditorAction, TransportCmd};
                    hero.view.game_stage = stage;
                    hero.bus.push(EditorAction::Transport(TransportCmd::Play));
                }
            }
            Some(Gesto::Para) => self.jogo_para(),
            Some(g) => {
                use winit::keyboard::KeyCode;
                let Some((seta, desce)) = g.seta() else {
                    return;
                };
                #[rustfmt::skip] // pela ordem das variantes de `Seta`
                let tecla = [KeyCode::ArrowRight, KeyCode::ArrowUp, KeyCode::ArrowLeft, KeyCode::ArrowDown][seta as usize];
                if let Some(k) = crate::keymap::winit_to_input_keycode(tecla).map(ph2d_input::Key) {
                    self.input.apply_event(if desce {
                        ph2d_input::Event::KeyDown(k)
                    } else {
                        ph2d_input::Event::KeyUp(k)
                    });
                }
            }
            None => {}
        }
    }

    /// Só a janela onde o app vive conduz; a outra (o editor parado atrás da janela do jogo) só sabe
    /// fechar — e fechar pára o jogo. `true` = o evento era da outra janela.
    pub(crate) fn da_outra_janela(
        &mut self,
        id: winit::window::WindowId,
        event: &winit::event::WindowEvent,
    ) -> bool {
        if self.window.as_ref().is_none_or(|w| w.id() == id) {
            return false;
        }
        if matches!(event, winit::event::WindowEvent::CloseRequested) {
            self.jogo_para();
        }
        true
    }

    /// O Stop por fora do transporte (fechar a janela do jogo): a MESMA porta do botão.
    pub(crate) fn jogo_para(&mut self) {
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            use ph2d_editor_core::action_bus::{EditorAction, TransportCmd};
            hero.bus.push(EditorAction::Transport(TransportCmd::Reset));
        }
    }

    fn abre_a_janela_do_jogo(&mut self, event_loop: &ActiveEventLoop) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let attrs = Window::default_attributes()
            .with_title(ph2d_i18n::tr("shell.janela_do_jogo.title"))
            .with_inner_size(TAMANHO);
        let gpu = gfx.surface.gpu().clone();
        let aberta = event_loop
            .create_window(attrs)
            .ok()
            .map(Arc::new)
            .and_then(|w| {
                let raw = gpu.instance.create_surface(w.clone()).ok()?;
                let host = crate::winit_host::WinitHost::new(w.clone());
                let surface = ph2d_gpu::SurfaceContext::new(gpu, raw, host.window_size()).ok()?;
                Some((w, host, surface))
            });
        let Some((window, host, surface)) = aberta else {
            // Joga no editor, e não tenta outra vez a cada quadro.
            if let Some(v) = self.undo.jogo.foto_mut().and_then(|f| f.vista.as_mut()) {
                v.stage = GameStage::Editor;
            }
            gfx.toasts
                .push(ph2d_editor_core::toast::Toast::info(ph2d_i18n::tr(
                    "shell.janela_do_jogo.no_window",
                )));
            return;
        };
        window.set_ime_allowed(true);
        let editor_surface = std::mem::replace(&mut gfx.surface, surface);
        let (Some(editor_window), Some(editor_host)) =
            (self.window.replace(window.clone()), self.host.replace(host))
        else {
            return;
        };
        self.undo.janela_do_jogo = Some(JanelaDoJogo {
            editor_window,
            editor_host,
            editor_surface,
        });
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.view.palco_do_jogo = true;
        }
        self.on_resized(window.inner_size());
        window.focus_window();
        window.request_redraw();
    }

    fn fecha_a_janela_do_jogo(&mut self) {
        let Some(j) = self.undo.janela_do_jogo.take() else {
            return;
        };
        // A surface do jogo cai ANTES da janela dela.
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.surface = j.editor_surface;
        }
        let size = j.editor_window.inner_size();
        j.editor_window.request_redraw();
        self.host = Some(j.editor_host);
        self.window = Some(j.editor_window);
        self.on_resized(size);
        self.title_dirty = true;
    }
}
