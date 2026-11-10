//! A cena 5 do jogo (`ph2d_sprite_anim_scene`): a shell diz se ela está armada e responde às portas da
//! [`Mesa`]; o guião e o executor moram na folha.

use ph2d_ecs::Entity;
use ph2d_editor_core::action_bus::EditorAction;
use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_editor_core::{ImageUseEdit, NodeId};
use ph2d_sprite_anim_scene::mesa::{AlvosDoEditor, Cartao, Mesa, Rato};

impl crate::App {
    /// Um por quadro, na fase das cenas.
    pub(crate) fn animado_do_smoke(&mut self) {
        if ph2d_app_game::smoke::armed_level() == Some(ph2d_sprite_anim_scene::CENA) {
            ph2d_sprite_anim_scene::mesa::quadro(self);
        }
    }

    fn heroi_da_cena(&mut self) -> Option<&mut HeroScreen> {
        self.gfx.as_mut()?.hero_screen.as_mut()
    }
}

impl Mesa for crate::App {
    fn mundo(&mut self) -> Option<&mut ph2d_ecs::SimWorld> {
        self.gfx.as_mut().map(|g| &mut g.sim)
    }

    fn por_nome(&mut self, nome: &str) -> Option<u64> {
        ph2d_app_game::smoke::por_nome(self.mundo()?, nome).map(Entity::to_bits)
    }

    fn bus(&mut self, acao: EditorAction) {
        if let Some(h) = self.heroi_da_cena() {
            h.bus.push(acao);
        }
    }

    fn uso_da_imagem(&mut self, bits: u64, edit: ImageUseEdit) {
        self.image_use_edit(bits, edit);
    }

    fn escolhe_imagem(&mut self, bits: u64, imagem: [u8; 32]) {
        self.choose_image(bits, imagem);
    }

    fn fileiras(&mut self, bits: u64) -> Vec<Vec<u32>> {
        let Some(g) = self.gfx.as_ref() else {
            return Vec::new();
        };
        let e = Entity::from_bits(bits);
        ph2d_app_sprite_anim::sprite_card::fileiras_de(
            g.sim.world(),
            e,
            &g.asset_db,
            &g.atlas_asset_map,
        )
    }

    fn rato(&mut self, gesto: Rato) {
        match gesto {
            Rato::Abaixo(x, y) => self.smoke_pointer_down(x, y),
            Rato::Move(x, y) => self.smoke_pointer_move(x, y),
            Rato::Acima => self.smoke_pointer_up(),
        }
    }

    fn widget(&self, id: NodeId) -> Option<(f32, f32)> {
        self.smoke_find_widget(id)
    }

    fn ctrl(&mut self, seguro: bool) {
        use winit::keyboard::ModifiersState as M;
        crate::touch_input::set_keyboard(if seguro { M::CONTROL } else { M::empty() });
        self.apply_modifiers(self.effective_modifiers());
    }

    fn ao_ecra(&self, ponto: [f32; 2]) -> Option<(f32, f32)> {
        let g = self.gfx.as_ref()?;
        Some(
            g.camera
                .world_to_screen(ponto, crate::field_gizmo_host::scene_window_of(g)),
        )
    }

    fn alvos(&mut self, bits: u64) -> Option<AlvosDoEditor> {
        let docs = self.heroi_da_cena()?.documents.surfaces_mut();
        let d = docs
            .find_mut::<ph2d_app_sprite_anim::editor::AnimDocs>()?
            .doc(bits)?;
        let (l, p) = (&d.layout.align, &d.align.painted);
        Some(AlvosDoEditor {
            toggle: l.toggle,
            by_feet: l.by_feet,
            grade: l.pivot_grid,
            by_hand: l.by_hand,
            follow: l.pivot_follow,
            previa: l.preview,
            pivo: p.pivot,
            caixa: p.all_frames,
            pega: p.handles[3],
            barra: d.layout.strip_bar,
            ranhura: d.layout.playhead,
            ultimo: d.layout.frames.last().map(|f| f.0),
        })
    }

    fn camara(&mut self, (centro, altura): ([f32; 2], f32)) {
        if let Some(g) = self.gfx.as_mut() {
            (g.camera.center, g.camera.height_world) = (centro, altura);
        }
    }

    fn seleciona(&mut self, bits: Option<u64>) {
        if let Some(h) = self.heroi_da_cena() {
            h.gizmo.replace_selection(bits);
        }
    }

    fn selecionado(&self) -> Option<u64> {
        self.gfx.as_ref()?.hero_screen.as_ref()?.gizmo.selection
    }

    fn filtro_pixel(&mut self) {
        if let Some(h) = self.heroi_da_cena() {
            h.project.image_filter = ph2d_render::ImageFilterMode::PixelArt;
        }
    }

    fn cartao(&mut self, cartao: Cartao, dobrado: bool) {
        let id = match cartao {
            Cartao::Mais => ph2d_panel_inspector::ids::INSP_ANIM_ADVANCED,
            Cartao::Aparencia => ph2d_panel_inspector::CARD_LOOK_ANIMADA,
            Cartao::Imagem => ph2d_panel_inspector::CARD_TRANSFORM_IMAGE,
            Cartao::Animacao => ph2d_editor_core::ids::INSP_LIVE_ANIM_SECTION,
        };
        if let Some(h) = self.heroi_da_cena() {
            h.store.set_collapsed(id, dobrado);
        }
    }

    fn painel_estreito(&mut self) {
        if let Some(h) = self.heroi_da_cena() {
            let direita = ph2d_editor_core::screens::layout::DockSide::Right;
            h.store.set_dock_width(direita, Some(0.0));
        }
    }

    fn desfaz(&mut self) {
        self.smoke_undo(false);
    }

    fn entrada(&mut self) {
        self.any_input_this_frame = true;
    }

    fn regra_acesa(&mut self, bits: u64) -> Option<u8> {
        let a_correr = self.playhead.is_playing();
        ph2d_app_game::personagem::regra_acesa(self.gfx.as_ref()?.sim.world(), bits, a_correr)
    }
}
