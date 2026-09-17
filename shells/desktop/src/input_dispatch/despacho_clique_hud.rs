//! **O clique num BOTÃO DO HUD** (TOP-20 #20) — ramo do `on_mouse_input` ([`super`]).
//!
//! # A lei é a do ORÁCULO, e não uma escolha nossa
//!
//! Medida em `docs/Components/ferramentas/godot_hud_probe.gd` (bloco L3), sobre o Godot 4.7.2
//! corrido sem interface:
//!
//! | o gesto | o que acontece |
//! |---|---|
//! | carregar DENTRO, largar DENTRO | ⭐ publica **uma vez, ao LARGAR** |
//! | carregar DENTRO, largar FORA | **não** publica |
//! | carregar FORA, largar DENTRO | **não** publica |
//! | `disabled` | **nunca** |
//!
//! ⚠️ **A 1.ª redacção da sonda dizia que largar FORA publicava** — faltava-lhe um MOUSE MOTION, e
//! sem ele o controlo do alvo nunca soube que o cursor tinha saído. *Um evento em falta mede outro
//! programa.*
//!
//! # ⛔ Só durante a CORRIDA, e é isso que o mantém fora do caminho do artista
//!
//! Fora da corrida (relógio parado) este ramo não existe: clicar num botão do HUD **selecciona-o**,
//! como qualquer outra forma — que é o que o artista quer ao montar o menu. É a mesma fronteira que
//! a fábrica e as mortes usam (`playhead.is_playing()`), e sem ela o HUD tornaria as próprias
//! formas dele ineditáveis.

use super::*;

impl crate::App {
    /// A entidade do botão do HUD sob o cursor, se houver um alcançável.
    ///
    /// ⚠️ **`disabled` e o nome em branco respondem `None` AQUI**, no mesmo sítio: são as duas
    /// maneiras de um botão não ser um botão, e separá-las daria dois lugares para decidir.
    fn hud_button_at(&self, screen: (f32, f32)) -> Option<ph2d_ecs::Entity> {
        let gfx = self.gfx.as_ref()?;
        let world = self.vec_world_at(screen)?;
        let tol = 10.0 * self.vec_px_to_world();
        let id = self.vec.pen.path_at(&gfx.vec_scene, world, tol)?;
        let bits = *self.vec.entities.get(&id)?;
        let e = ph2d_ecs::Entity::from_bits(bits);
        let b = gfx.sim.world().get::<ph2d_ecs::UiButton>(e)?;
        (!b.disabled && b.name().is_some()).then_some(e)
    }

    /// Ver o cabeçalho do módulo. `true` = o clique foi consumido pelo HUD.
    pub(super) fn ramo_botao_do_hud(
        &mut self,
        kind: PointerKind,
        mapped_button: ph2d_host::PointerButton,
        on_canvas: bool,
    ) -> bool {
        if mapped_button != ph2d_host::PointerButton::Primary || !self.playhead.is_playing() {
            return false;
        }
        match kind {
            PointerKind::Down if on_canvas => {
                let Some(e) = self.hud_button_at(self.last_pointer) else {
                    return false;
                };
                self.hud_press = Some(e);
                true
            }
            PointerKind::Up => {
                // ⚠️ O aperto é consumido **mesmo quando o largar cai fora**: quem carregou num
                // botão não estava a começar uma selecção, e deixar o `Up` passar faria o editor
                // ver metade de um gesto que não é dele.
                let Some(premido) = self.hud_press.take() else {
                    return false;
                };
                if self.hud_button_at(self.last_pointer) == Some(premido)
                    && let Some(gfx) = self.gfx.as_ref()
                    && let Some(b) = gfx.sim.world().get::<ph2d_ecs::UiButton>(premido)
                    && let Some(nome) = b.name()
                {
                    let sinal = ph2d_runtime::Signal::from_ui_button(nome, premido.to_bits());
                    self.signals.publish(sinal);
                }
                true
            }
            PointerKind::Down | PointerKind::Move => false,
        }
    }
}
