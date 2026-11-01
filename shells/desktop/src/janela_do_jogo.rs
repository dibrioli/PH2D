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
            use ph2d_app_components::{component_attach, object_add};
            let (reg, seeds) = (cx.registry, ph2d_app_physics::physics_seed::COMPONENT_SEEDS);
            let anexa = |s: &mut ph2d_ecs::SimWorld, b, n: &str| {
                component_attach::attach_by_name(s, reg, seeds, b, n)
            };
            let camera = |s: &mut ph2d_ecs::SimWorld| {
                object_add::add(object_add::CAMERA, s, reg, seeds).expect("é dos componentes")
            };
            let menu = ph2d_app_game::object_add::Portas {
                raiz: object_add::spawn_empty_root,
                anexa: &anexa,
            };
            let portas = ph2d_app_game::smoke::Portas {
                menu,
                camera: &camera,
            };
            let jogador = ph2d_app_game::smoke::montar(cx.sim, portas, nivel).to_bits();
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

    /// `PH2D_ORDEM_SMOKE=1|2` (GEF 19): cada tipo nasce pela porta da família dele, sobre a «Pile» e o
    /// «Swarm» do smoke do Motion (espera-os); a pose e a ordem são do `ph2d_app_game::ordem_smoke`.
    pub(crate) fn ordem_smoke(&mut self) {
        use ph2d_app_game::ordem_smoke::{self as o, Peca};
        let (Some(nivel), Some(gfx)) = (o::nivel(), self.gfx.as_mut()) else {
            return;
        };
        let w = gfx.sim.world_mut();
        let (Some(pilha), Some(enxame)) =
            (o::por_nome(w, Peca::Pilha), o::por_nome(w, Peca::Enxame))
        else {
            return;
        };
        if !o::monta_agora() {
            return;
        }
        use ph2d_app_components::{component_attach, object_add};
        let raiz = object_add::spawn_empty_root;
        let (reg, seeds) = (
            &gfx.component_registry,
            ph2d_app_physics::physics_seed::COMPONENT_SEEDS,
        );
        let anexa = |s: &mut ph2d_ecs::SimWorld, b, n: &str| {
            component_attach::attach_by_name(s, reg, seeds, b, n)
        };
        let sim = &mut gfx.sim;
        let imagem = o::imagem(
            sim,
            ph2d_app_game::object_add::Portas {
                raiz,
                anexa: &anexa,
            },
        );
        let flip = ph2d_app_flip::object_add::FLIP;
        let traco = ph2d_app_flip::object_add::add(flip, sim, &mut gfx.flip, &mut self.flip_state);
        if let Some(oid) = self.flip_state.born {
            ph2d_app_flip::ordem_smoke::traco(&mut gfx.flip, oid);
        }
        let rosto = ph2d_app_pixel::smoke::rosto(&mut self.pixel, sim, raiz);
        let macro_ =
            ph2d_app_macros::object_add::add(ph2d_app_macros::object_add::MACRO, sim, raiz);
        let camera = object_add::add(object_add::CAMERA, sim, reg, seeds);
        let pecas: Vec<(Peca, u64)> = [
            (Peca::Imagem, imagem),
            (Peca::Pilha, Some(pilha)),
            (Peca::Traco, traco.and_then(Result::ok)),
            (Peca::Rosto, rosto),
            (Peca::Enxame, Some(enxame)),
            (Peca::Macro, macro_.and_then(Result::ok)),
            (Peca::Camera, camera.and_then(Result::ok)),
        ]
        .into_iter()
        .filter_map(|(p, b)| Some((p, b?)))
        .collect();
        for &(p, b) in &pecas {
            o::arruma(sim.world_mut(), b, p);
        }
        o::ordena(sim.world_mut(), nivel, &pecas);
        // O Pixel nasce em Draw e com zoom nele: volta-se a Object e enquadra-se o cartão (a cena).
        if let (Some(hero), Some(cartao)) = (gfx.hero_screen.as_mut(), imagem) {
            use ph2d_editor_core::{action_bus::EditorAction, object_mode as m};
            hero.bus
                .push(EditorAction::ObjectMode(m::ModeRequest::Enter(
                    m::ObjectMode::Object,
                )));
            hero.gizmo.replace_selection(Some(cartao));
            let kind = ph2d_editor_core::ViewFocusKind::FrameSelected;
            hero.bus.push(EditorAction::SetViewFocus { kind });
        }
        eprintln!("{}", o::mensagem(nivel, pecas.len()));
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

    /// As fotos-gesto da câmera (17b, [`ph2d_app_game::smoke_mao`]): aperta, arrasta e solta pelas portas reais
    /// do ponteiro; o `Up` imprime o que ficou (`[camera-mao]`).
    pub(crate) fn mao_da_camera_do_smoke(&mut self) {
        use ph2d_app_game::smoke_mao::{Mao, mao, proximo_quadro_da_mao, relato};
        let foto = ph2d_app_game::smoke::foto();
        let chave = foto == Some(ph2d_app_game::smoke::Foto::CameraChave);
        if chave && !self.timeline.flags.auto_key {
            self.timeline.flags.auto_key = true;
            // O Auto Key só arma com o painel Timeline à vista — é lá que o botão mora.
            if let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                ph2d_editor_core::panel::PanelHostInternal::set_panel_visible(h, "timeline", true);
            }
            self.playhead.pause();
            self.playhead.seek(ph2d_app_game::smoke::tempo_da_chave());
        }
        if foto == Some(ph2d_app_game::smoke::Foto::CameraDesfaz) {
            self.formato_desfaz_do_smoke(proximo_quadro_da_mao());
            return;
        }
        let Some(m) = foto.and_then(|f| mao(f, proximo_quadro_da_mao())) else {
            return;
        };
        let Some(gfx) = self.gfx.as_ref() else {
            return;
        };
        // A mão anda na TELA: com o «Look through» a vista anda com a câmera, e a mão não.
        let vista = self
            .undo
            .camera_no_canvas
            .vista_do_down
            .unwrap_or(gfx.camera);
        let ecra = |w: [f32; 2]| vista.world_to_screen(w, gfx.scene_window());
        match m {
            Mao::Desce(w) => {
                let (x, y) = ecra(w);
                self.smoke_pointer_down(x, y);
            }
            Mao::Move(w) => {
                let (x, y) = ecra(w);
                self.smoke_pointer_move(x, y);
            }
            Mao::Sobe => {
                self.smoke_pointer_up();
                if let Some(gfx) = self.gfx.as_mut() {
                    let sel = gfx.hero_screen.as_ref().and_then(|h| h.gizmo.selection);
                    eprintln!("[camera-mao] {}", relato(&mut gfx.sim, sel));
                    if chave {
                        eprintln!(
                            "[camera-chave] t={:.2} · {}",
                            self.playhead.time(),
                            crate::janela_do_jogo::chaves_da_camera(&mut gfx.sim, &self.timeline)
                        );
                    }
                }
            }
        }
    }

    /// `PH2D_GAME_SMOKE_USOS`: os cliques do seletor «Use» sobre o Player, um por vez, pelo barramento do Inspector
    /// (a porta do clique); o estado do Player sai no `stderr` antes de cada clique e no fim.
    pub(crate) fn game_smoke_usos(&mut self) {
        use ph2d_app_game::smoke as s;
        let (Some(usos), Some(_)) = (s::usos_do_dono(), s::armed_level()) else {
            return;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        gfx.camera.center = s::CENTRO_DOS_USOS;
        let Some(i) = s::uso_no_quadro(usos.len() + 1) else {
            return;
        };
        eprintln!(
            "[usos] antes do passo {}: {}",
            i + 1,
            s::estado_do_player(&mut gfx.sim)
        );
        let (Some(&passo), Some(p)) = (usos.get(i), s::player(&mut gfx.sim)) else {
            return;
        };
        let Some(hero) = gfx.hero_screen.as_mut() else {
            return;
        };
        use ph2d_app_sprite_anim::region as r;
        match passo {
            s::PassoDoDono::Uso(u) => {
                hero.bus
                    .push(ph2d_editor_core::action_bus::EditorAction::ImageUseEdit {
                        entity_bits: p.to_bits(),
                        edit: ph2d_editor_core::ImageUseEdit::Use(u),
                    })
            }
            s::PassoDoDono::Celula(c) => _ = r::click_cell(hero, c),
            s::PassoDoDono::Ok => _ = r::click_ok(hero),
            s::PassoDoDono::Snap => _ = r::click_snap(hero),
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
        let formato = gfx.hero_screen.as_ref().map(|h| h.project.game_format);
        let [w, h] = ph2d_app_game::encaixe::tamanho_da_janela(formato.unwrap_or_default());
        let attrs = Window::default_attributes()
            .with_title(ph2d_i18n::tr("shell.janela_do_jogo.title"))
            .with_inner_size(winit::dpi::LogicalSize::new(w, h));
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

/// As chaves que a timeline guarda para a «Camera» da cena: `propriedade=n` por ligação (a foto `camera_chave`).
pub(crate) fn chaves_da_camera(
    sim: &mut ph2d_ecs::SimWorld,
    tl: &ph2d_timeline::TimelineState,
) -> String {
    let Some(cam) = ph2d_app_game::smoke::por_nome(sim, "Camera") else {
        return "sem câmera".to_owned();
    };
    let clip = tl.doc.active_clip();
    let v: Vec<String> = tl
        .doc
        .bindings()
        .iter()
        .filter(|b| b.entity == cam.to_bits())
        .map(|b| {
            let n = clip.track(b.target).map_or(0, |t| t.len());
            format!("{:?}={n}", b.prop)
        })
        .collect();
    if v.is_empty() {
        "nenhuma ligação".to_owned()
    } else {
        v.join(" · ")
    }
}

impl crate::App {
    /// A foto `camera_desfaz` (onda 18): no quadro 40 o formato vai a Game Boy pela porta do Inspector (com a
    /// entrada que um clique dá), e depois Ctrl+Z e Ctrl+Y — `[camera-desfaz]` diz o formato a cada passo.
    fn formato_desfaz_do_smoke(&mut self, quadro: u32) {
        use ph2d_editor_core::{CameraFieldEdit, GamePreset};
        let relata = |app: &Self, quando: &str| {
            if let Some(h) = app.gfx.as_ref().and_then(|g| g.hero_screen.as_ref()) {
                eprintln!(
                    "[camera-desfaz] {quando}: {:?}",
                    h.project.game_format.preset
                );
            }
        };
        match quadro {
            40 => {
                relata(self, "antes");
                self.any_input_this_frame = true;
                let cam = self
                    .gfx
                    .as_mut()
                    .and_then(|g| ph2d_app_game::smoke::por_nome(&mut g.sim, "Camera"));
                let i = GamePreset::ALL
                    .iter()
                    .position(|p| *p == GamePreset::GameBoy)
                    .unwrap_or(0);
                if let (Some(cam), Some(h)) =
                    (cam, self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()))
                {
                    h.bus.push(
                        ph2d_editor_core::action_bus::EditorAction::InspectorCameraEdit {
                            entity_bits: cam.to_bits(),
                            edit: CameraFieldEdit::Format(i as u8),
                        },
                    );
                }
            }
            // O clique do artista é entrada nesses quadros (a fila só fecha passos com entrada).
            41..=43 => self.any_input_this_frame = true,
            50 => {
                relata(self, "trocado");
                self.smoke_undo(false);
            }
            60 => {
                relata(self, "depois do Ctrl+Z");
                self.smoke_undo(true);
            }
            70 => relata(self, "depois do Ctrl+Y"),
            _ => {}
        }
    }
}
