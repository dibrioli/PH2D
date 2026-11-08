//! O executor do guião da cena 5 do jogo (`ph2d_sprite_anim_scene`): cada passo pelas portas do produto.

use ph2d_editor_core::action_bus::EditorAction;
use ph2d_sprite_anim_scene::{self as a, Passo as P};

impl crate::App {
    /// Um por quadro, na fase das cenas.
    pub(crate) fn animado_do_smoke(&mut self) {
        if ph2d_app_game::smoke::armed_level() != Some(a::CENA) {
            return;
        }
        let Some(q) = a::proximo_quadro() else {
            return;
        };
        for p in a::passos(q, a::foto()) {
            self.passo_do_animado(p);
        }
    }

    fn bits_de(&mut self, nome: &str) -> Option<u64> {
        let g = self.gfx.as_mut()?;
        ph2d_app_game::smoke::por_nome(&mut g.sim, nome).map(ph2d_ecs::Entity::to_bits)
    }

    fn passo_do_animado(&mut self, p: P) {
        let bus = |app: &mut Self, entity_bits, edit| {
            if let Some(h) = app.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                h.bus
                    .push(EditorAction::InspectorAnimEdit { entity_bits, edit });
            }
        };
        let nome = match &p {
            P::Anim(n, _)
            | P::Verbo(n, _)
            | P::Heroi(n)
            | P::Imagem(n, _)
            | P::Folha(n, _)
            | P::Corta(n)
            | P::Ouve(n)
            | P::Relata(_, n) => n,
            P::Foto(f) => f.map_or(a::PLAYER, a::Foto::alvo),
            P::RelataRegra => a::PLAYER,
            P::Entrada | P::Desfaz => "",
        };
        let bits = self.bits_de(nome);
        match (p, bits) {
            (P::Anim(_, edit), Some(b)) => bus(self, b, edit),
            (P::Verbo(_, v), Some(b)) => self.image_use_edit(b, v),
            (P::Heroi(_), Some(b)) => {
                if let Some(g) = self.gfx.as_mut() {
                    let e = ph2d_ecs::Entity::from_bits(b);
                    if let Err(motivo) = ph2d_render::veste_o_heroi(g.sim.world_mut(), e) {
                        eprintln!("[animado] o herói não entrou: {motivo}");
                    }
                }
            }
            (P::Imagem(_, i), Some(b)) => {
                if let Some(id) = a::imagem(i) {
                    self.choose_image(b, id);
                }
            }
            (P::Folha(_, i), Some(b)) => {
                if let Some(id) = a::imagem(i) {
                    self.image_use_edit(b, ph2d_editor_core::ImageUseEdit::AddSheet(id));
                }
            }
            (P::Corta(_), Some(b)) => {
                let rows = self.gfx.as_ref().map(|g| {
                    let e = ph2d_ecs::Entity::from_bits(b);
                    ph2d_app_sprite_anim::sprite_card::fileiras_de(
                        g.sim.world(),
                        e,
                        &g.asset_db,
                        &g.atlas_asset_map,
                    )
                });
                bus(
                    self,
                    b,
                    ph2d_editor_core::AnimFieldEdit::SplitByRows(rows.unwrap_or_default()),
                );
            }
            (P::Ouve(_), Some(b)) => {
                if let Some(g) = self.gfx.as_mut() {
                    a::ouve_o_marcador(&mut g.sim, ph2d_ecs::Entity::from_bits(b));
                }
            }
            (P::Foto(f), b) => self.foto_do_animado(f, b),
            (P::Entrada, _) => self.any_input_this_frame = true,
            (P::Desfaz, _) => self.smoke_undo(false),
            (P::Relata(t, n), Some(b)) => {
                if let Some(g) = self.gfx.as_ref() {
                    let e = ph2d_ecs::Entity::from_bits(b);
                    eprintln!("[animado-prova] {t}: {}", a::relato(&g.sim, n, e));
                }
            }
            (P::RelataRegra, Some(b)) => {
                let a_correr = self.playhead.is_playing();
                let acesa = self.gfx.as_ref().and_then(|g| {
                    ph2d_app_game::personagem::regra_acesa(g.sim.world(), b, a_correr)
                });
                eprintln!("[animado-prova] regra acesa do Player: {acesa:?}");
            }
            _ => {}
        }
    }

    fn foto_do_animado(&mut self, foto: Option<a::Foto>, bits: Option<u64>) {
        let Some(g) = self.gfx.as_mut() else {
            return;
        };
        (g.camera.center, g.camera.height_world) = a::VISTA;
        let Some(h) = g.hero_screen.as_mut() else {
            return;
        };
        h.gizmo.replace_selection(bits);
        match (foto, bits) {
            (Some(a::Foto::More), _) => {
                h.store
                    .set_collapsed(ph2d_panel_inspector::ids::INSP_ANIM_ADVANCED, false);
            }
            // O pedido de zero cai no piso do painel, `DOCK_W_MIN`.
            (Some(a::Foto::Estreito), _) => {
                h.store.set_dock_width(
                    ph2d_editor_core::screens::layout::DockSide::Right,
                    Some(0.0),
                );
            }
            (
                Some(a::Foto::Editor | a::Foto::FolhasEditor | a::Foto::FolhasApagada),
                Some(entity_bits),
            ) => {
                h.bus.push(EditorAction::InspectorAnimEdit {
                    entity_bits,
                    edit: ph2d_editor_core::AnimFieldEdit::OpenEditor,
                })
            }
            _ => {}
        }
    }
}
