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

/// A vista do editor antes do palco — o que a saída repõe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VistaAntes {
    zen: bool,
    camara: bool,
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
        let stage = gfx.hero_screen.as_ref()?.view.game_stage;
        if stage == GameStage::Editor {
            return None;
        }
        let antes = VistaAntes {
            zen: gfx.zen.active,
            camara: self.game_camera_preview,
            stage,
        };
        gfx.zen.active = true;
        self.game_camera_preview = true;
        Some(antes)
    }

    /// A saída: a vista do editor volta como estava. A janela fecha em [`Self::palco_janela`].
    pub(crate) fn palco_sai(&mut self, antes: VistaAntes) {
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.zen.active = antes.zen;
        }
        self.game_camera_preview = antes.camara;
    }

    /// Depois de cada quadro, com o event loop na mão: abre a janela que o palco pede e fecha-a quando
    /// o jogo pára.
    pub(crate) fn palco_janela(&mut self, event_loop: &ActiveEventLoop) {
        let quer = self.jogo.foto().and_then(|f| f.vista).map(|v| v.stage) == Some(GameStage::Window);
        match (quer, self.janela_do_jogo.is_some()) {
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
            ph2d_app_game::smoke::montar(cx.sim.world_mut(), nivel);
        }
        self.timeline.flags.simulate_physics = true;
        self.playhead.rewind();
        self.playhead.pause();
    }

    /// O condutor do smoke (`PH2D_GAME_SMOKE_DRIVE`): os gestos pelas portas do produto.
    pub(crate) fn game_smoke_drive(&mut self) {
        use ph2d_app_game::smoke::{self as s, Gesto};
        let Some(stage) = s::conducao().filter(|_| s::armed_level().is_some()) else {
            return;
        };
        match s::gesto_no_quadro(stage, s::proximo_quadro()) {
            Some(Gesto::Joga(stage)) => {
                if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                    use ph2d_editor_core::action_bus::{EditorAction, TransportCmd};
                    hero.view.game_stage = stage;
                    hero.bus.push(EditorAction::Transport(TransportCmd::Play));
                }
            }
            Some(Gesto::Direita(desce)) => {
                let tecla = winit::keyboard::KeyCode::ArrowRight;
                if let Some(k) = crate::keymap::winit_to_input_keycode(tecla).map(ph2d_input::Key) {
                    self.input.apply_event(if desce {
                        ph2d_input::Event::KeyDown(k)
                    } else {
                        ph2d_input::Event::KeyUp(k)
                    });
                }
            }
            Some(Gesto::Para) => self.jogo_para(),
            None => {}
        }
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
        let aberta = event_loop.create_window(attrs).ok().map(Arc::new).and_then(|w| {
            let raw = gpu.instance.create_surface(w.clone()).ok()?;
            let host = crate::winit_host::WinitHost::new(w.clone());
            let surface = ph2d_gpu::SurfaceContext::new(gpu, raw, host.window_size()).ok()?;
            Some((w, host, surface))
        });
        let Some((window, host, surface)) = aberta else {
            gfx.toasts.push(ph2d_editor_core::toast::Toast::info(ph2d_i18n::tr(
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
        self.janela_do_jogo = Some(JanelaDoJogo {
            editor_window,
            editor_host,
            editor_surface,
        });
        self.on_resized(window.inner_size());
        window.focus_window();
        window.request_redraw();
    }

    fn fecha_a_janela_do_jogo(&mut self) {
        let Some(j) = self.janela_do_jogo.take() else {
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
