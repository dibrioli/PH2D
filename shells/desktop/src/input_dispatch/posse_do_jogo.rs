//! ⭐ **De quem é o teclado e o rato com a cena a correr** (CodeEditor item 7) — só a composição: a lei e a régua moram
//! em [`ph2d_app_components::posse_do_jogo`]; o porquê em `docs/CodeEditor/06_posse_do_teclado.md`.

use super::*;
use ph2d_app_components::posse_do_jogo::{Cena, Tecla, Transporte};
use ph2d_i18n::tr;
use winit::keyboard::KeyCode;

impl crate::App {
    /// A cena corre? Está à vista (nenhuma aba de documento por cima, nenhum campo de texto com o teclado)?
    pub(crate) fn cena_do_jogo(&self) -> Cena {
        // Tapam-na também os modos do editor donos do teclado: a paleta, um menu ou diálogo, a prévia do Morph.
        let a_vista = self.hero_ref().is_some_and(|h| {
            h.documents.active().is_none()
                && h.documents.surface_active().is_none()
                && !h.text_entry_focused()
        }) && !self.menu_aberto()
            && !self.command_palette_open()
            && !self.morph_preview;
        Cena {
            a_correr: self.playhead.is_playing(),
            a_vista,
        }
    }

    /// Uma vez por quadro, ANTES dos dois leitores do teclado (o jogador e os scripts); a cena leva a moldura.
    pub(crate) fn quadro_do_foco_do_jogo(&mut self) {
        let cena = self.cena_do_jogo();
        let tem = self.components.foco_do_jogo.quadro(&mut self.input, cena);
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.view.jogo_com_o_teclado = tem;
            ph2d_app_components::posse_do_jogo_registo::selecao(hero.gizmo.selection);
        }
    }

    /// A tecla física: `true` = consumida (nenhum atalho do editor a vê). O normalizador é o TOTAL, não o do editor.
    pub(super) fn tecla_do_jogo(&mut self, code: KeyCode, desceu: bool, repete: bool) -> bool {
        let m = self.modifiers;
        let t = Tecla {
            desceu,
            acorde: m.control_key() || m.alt_key() || m.super_key(),
            shift: m.shift_key(),
            f5: code == KeyCode::F5,
            repete,
        };
        let chave = crate::keymap::winit_to_input_keycode(code).map(ph2d_input::Key);
        let cena = self.cena_do_jogo();
        let previa = self.morph_preview && !self.surface_owns_the_keyboard();
        let d = self
            .components
            .foco_do_jogo
            .tecla(&mut self.input, cena, t, chave, previa, &code);
        match (d.transporte, self.gfx.as_mut()) {
            (Some(Transporte::Alterna), Some(gfx)) => {
                alterna_o_relogio(
                    self.timeline.containers_list,
                    &mut self.playhead,
                    &mut gfx.toasts,
                );
            }
            // A mesma porta do botão Rebobinar: o relógio volta ao início, parado, e a corrida sai.
            (Some(Transporte::Para), Some(gfx)) => {
                if let Some(hero) = gfx.hero_screen.as_mut() {
                    use ph2d_editor_core::action_bus::{EditorAction, TransportCmd};
                    hero.bus.push(EditorAction::Transport(TransportCmd::Reset));
                }
            }
            _ => {}
        }
        d.consumida
    }

    /// O rato do JOGO: o premido NA CENA (nem painel, nem barra, nem aba, nem menu aberto) entra nele e dá-lhe o foco.
    /// `true` = o clique é SÓ do jogo.
    pub(super) fn rato_do_jogo(&mut self, button: ph2d_host::PointerButton, down: bool) -> bool {
        let (x, y) = self.last_pointer;
        let na_cena = self.pointer_on_canvas(x, y) && !self.menu_aberto();
        let cena = self.cena_do_jogo();
        self.components
            .foco_do_jogo
            .clique(&mut self.input, cena, button, down, na_cena)
    }

    /// Um menu de contexto ou o modal do Fill aberto: o clique é dele (fecha-o), nunca da cena.
    pub(super) fn menu_aberto(&self) -> bool {
        self.hero_ref()
            .is_some_and(|h| h.store.context_menu().is_some() || h.store.fill_modal_pos().is_some())
    }

    /// O clique SÓ do jogo: além do jogo (já servido), só agem o que é DELE — os botões do HUD (a interface do
    /// jogo) e o laboratório de física (a mão, a explosão, a atração: decisão do dono, 20/10, «fica como está»).
    pub(super) fn clique_so_do_jogo(&mut self, button: ph2d_host::PointerButton, down: bool) {
        let kind = if down {
            PointerKind::Down
        } else {
            PointerKind::Up
        };
        if self.ramo_botao_do_hud(kind, button, true)
            || !down
            || button != ph2d_host::PointerButton::Primary
            || !self.timeline.flags.simulate_physics
        {
            return;
        }
        let (x, y) = self.last_pointer;
        if !self.physics.interaction.tool.needs_a_body() {
            self.poke_press(x, y);
            return;
        }
        self.mao_no_jogo(x, y);
    }

    /// A MÃO: o corpo mais à frente sob o rato que ela consegue pegar — e fica selecionado, como no pick de sempre.
    fn mao_no_jogo(&mut self, x: f32, y: f32) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let Some(hero) = gfx.hero_screen.as_mut() else {
            return;
        };
        let window_size = crate::scene_mapping::janela(hero.view.center_split, gfx.surface.size());
        let world = gfx.camera.screen_to_world((x, y), window_size);
        let mut pw = crate::hover_highlight::PickWorld {
            window_size,
            sim: &gfx.sim,
            vec_scene: &gfx.vec_scene,
            flip: &gfx.flip,
            present: &mut gfx.present,
            camera: &gfx.camera,
            pixels_per_meter: hero.project.pixels_per_meter,
        };
        let mut hits = crate::hover_highlight::pick_objects_at(
            &mut pw,
            &self.vec.entities,
            &self.vec.view_derived,
            &self.vec.live_drawn,
            &self.flip_state.entities,
            (x, y),
        );
        crate::pick_order::descendants_first(gfx.sim.world(), &mut hits);
        for bits in hits {
            let entity = ph2d_ecs::Entity::from_bits(bits);
            if !ph2d_ecs::is_locked_for_edit(gfx.sim.world(), entity)
                && ph2d_app_physics::body_grab::take_hold(
                    &mut gfx.physics,
                    &self.physics.interaction,
                    entity,
                    world,
                    true,
                    true,
                )
            {
                hero.gizmo.replace_selection(Some(bits));
                return;
            }
        }
    }
}

/// ⭐ **Pausa ou continua** — a porta única do Espaço e do F5. Na lista de Containers não há reprodução (Enio,
/// 2026-07-22): a ponte voltaria a pausá-la no quadro seguinte, e o aviso anunciaria um Play que não acontece.
pub(crate) fn alterna_o_relogio(
    lista_de_containers: bool,
    playhead: &mut ph2d_core::Playhead,
    toasts: &mut ph2d_editor_core::toast::ToastQueue,
) {
    if lista_de_containers {
        toasts.push(ph2d_editor_core::toast::Toast::info(tr(
            "shell.handlers_teclas_editor.timeline_no_playback",
        )));
        return;
    }
    let a_tocar = playhead.toggle_play();
    toasts.push(ph2d_editor_core::toast::Toast::info(if a_tocar {
        tr("shell.handlers_teclas_editor.timeline_play")
    } else {
        tr("shell.handlers_teclas_editor.timeline_pause")
    }));
}
