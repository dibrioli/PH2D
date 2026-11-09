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
            | P::Ferramenta(n, _)
            | P::Corta(n)
            | P::Ouve(n)
            | P::Edita(n)
            | P::Pivo(n, _)
            | P::Alinha(n, _)
            | P::Relata(_, n) => n,
            P::Foto(f) => f.map_or(a::PLAYER, a::Foto::alvo),
            P::RelataRegra => a::PLAYER,
            P::Entrada | P::Desfaz => "",
        };
        let bits = self.bits_de(nome);
        match (p, bits) {
            (P::Anim(_, edit), Some(b)) => bus(self, b, edit),
            (P::Verbo(_, v), Some(b)) => self.image_use_edit(b, v),
            (P::Ferramenta(_, tool_id), Some(entity_bits)) => {
                if let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                    h.bus.push(EditorAction::OneShotImageOp {
                        tool_id,
                        entity_bits,
                    });
                }
            }
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
            (P::Edita(_), Some(_)) => {
                if let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                    let pedido = ph2d_editor_core::object_mode::ModeRequest::Enter(
                        ph2d_editor_core::object_mode::ObjectMode::Edit,
                    );
                    h.bus.push(EditorAction::ObjectMode(pedido));
                }
            }
            (P::Pivo(_, fase), Some(b)) => self.pivo_do_animado(fase, b),
            (P::Alinha(_, fase), Some(b)) => self.alinha_do_animado(fase, b),
            (P::Entrada, _) => self.any_input_this_frame = true,
            (P::Desfaz, _) => self.smoke_undo(false),
            (P::Relata(t, n), Some(b)) => {
                if let Some(g) = self.gfx.as_ref() {
                    let e = ph2d_ecs::Entity::from_bits(b);
                    let folha0 = g
                        .sim
                        .world()
                        .get::<ph2d_render::Sprite>(e)
                        .map(|s| (s.size, s.anchor));
                    eprintln!(
                        "[animado-prova] {t}: {} folha0={folha0:?}",
                        a::relato(&g.sim, n, e)
                    );
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

    /// O gesto do Pivot com Ctrl pelas portas do rato (`smoke_pointer_*`), uma fase por quadro: um clique humano tem
    /// o botão abaixo e acima em quadros separados.
    fn pivo_do_animado(&mut self, fase: u8, bits: u64) {
        use ph2d_editor_core::gizmo::ids::GIZMO_PIVOT;
        let achar = |app: &Self, id, o_que: &str| {
            let p = app.smoke_find_widget(id);
            if p.is_none() {
                eprintln!("[animado-prova] pivô: {o_que} não está na tela");
            }
            p
        };
        match fase {
            0 => {
                if let Some((x, y)) =
                    achar(self, ph2d_editor_core::ids::TOOL_PIVOT, "o botão Pivot")
                {
                    self.smoke_pointer_down(x, y);
                }
            }
            1 => self.smoke_pointer_up(),
            2 => {
                eprintln!("[animado-prova] pivô antes: {}", self.relato_do_pivo(bits));
                if let Some((x, y)) = achar(self, GIZMO_PIVOT, "o ponto do pivô") {
                    // Ctrl SEGURO pela porta do teclado real: o `sync_touch_modifiers` relê-a em todo quadro.
                    crate::touch_input::set_keyboard(winit::keyboard::ModifiersState::CONTROL);
                    self.apply_modifiers(self.effective_modifiers());
                    self.smoke_pointer_down(x, y);
                }
            }
            3 => {
                let alvo = self.gfx.as_ref().and_then(|g| {
                    let c = centro_do_quad(g.sim.world(), ph2d_ecs::Entity::from_bits(bits))?;
                    let (x, y) = g
                        .camera
                        .world_to_screen(c, crate::field_gizmo_host::scene_window_of(g));
                    Some((x + 5.0, y + 4.0))
                });
                if let Some((x, y)) = alvo {
                    self.smoke_pointer_move(x, y);
                }
            }
            _ => {
                self.smoke_pointer_up();
                crate::touch_input::set_keyboard(winit::keyboard::ModifiersState::empty());
                self.apply_modifiers(self.effective_modifiers());
                eprintln!("[animado-prova] pivô depois: {}", self.relato_do_pivo(bits));
            }
        }
    }

    /// A prévia Align pelas portas do rato, uma fase por quadro: `0`/`1` o botão Align; `2` a pega do canto, `3..=6` o
    /// caminho até `2/3` da distância ao Pivot, `7` soltar.
    fn alinha_do_animado(&mut self, fase: u8, bits: u64) {
        static PEGA: std::sync::Mutex<Option<((f32, f32), (f32, f32))>> =
            std::sync::Mutex::new(None);
        let doc = self.gfx.as_mut().and_then(|g| {
            let d = g.hero_screen.as_mut()?.documents.surfaces_mut();
            let d = d
                .find_mut::<ph2d_app_sprite_anim::editor::AnimDocs>()?
                .doc(bits)?;
            Some((
                d.layout.align.toggle,
                d.layout.align.by_feet,
                d.align.painted,
            ))
        });
        let Some((toggle, pes, p)) = doc else {
            return eprintln!("[animado-prova] align: o editor não está aberto");
        };
        let c = |r: ph2d_editor_core::zones::Rect| (r.x + r.w / 2.0, r.y + r.h / 2.0);
        let mut pega = PEGA
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match (fase, *pega) {
            (0, _) => self.smoke_pointer_down(c(toggle).0, c(toggle).1),
            (8, _) => self.smoke_pointer_down(c(pes).0, c(pes).1),
            (1 | 7 | 9, _) => self.smoke_pointer_up(),
            (2, _) => {
                *pega = p.handles[3].map(c).zip(p.pivot);
                eprintln!(
                    "[animado-prova] align: pega={:?} pivô={:?}",
                    p.handles[3], p.pivot
                );
                if let Some(((x, y), _)) = *pega {
                    self.smoke_pointer_down(x, y);
                }
            }
            (_, Some(((hx, hy), (ox, oy)))) => {
                let s = 1.0 - (1.0 - a::ESCALA_DA_RUN) * f32::from(fase - 2) / 4.0;
                self.smoke_pointer_move(ox + (hx - ox) * s, oy + (hy - oy) * s);
            }
            _ => {}
        }
    }

    /// Selecionado? · a origem · a folha visível e o Pivot dela · o centro do quad dela no mundo.
    fn relato_do_pivo(&self, bits: u64) -> String {
        let Some(g) = self.gfx.as_ref() else {
            return String::new();
        };
        let e = ph2d_ecs::Entity::from_bits(bits);
        let sel = g.hero_screen.as_ref().and_then(|h| h.gizmo.selection) == Some(bits);
        let t = g
            .sim
            .world()
            .get::<ph2d_ecs::Transform>(e)
            .map(|t| t.translation);
        let pivo =
            ph2d_sprite_edit::pivot::sprite_do_pivot(g.sim.world(), e).map(|(k, s)| (k, s.anchor));
        let c = centro_do_quad(g.sim.world(), e);
        format!("selecionado={sel} origem={t:?} folha_e_pivo={pivo:?} desenho={c:?}")
    }

    fn foto_do_animado(&mut self, foto: Option<a::Foto>, bits: Option<u64>) {
        let Some(g) = self.gfx.as_mut() else {
            return;
        };
        (g.camera.center, g.camera.height_world) = foto.map_or(a::VISTA, a::Foto::vista);
        let Some(h) = g.hero_screen.as_mut() else {
            return;
        };
        h.gizmo.replace_selection(bits);
        // `PH2D_GAME_SMOKE_FILTRO=pixel`: as fotos do filtro do app (Preferences ▸ Image Filter ▸ Pixel Art).
        if std::env::var("PH2D_GAME_SMOKE_FILTRO").is_ok_and(|v| v.trim() == "pixel") {
            h.project.image_filter = ph2d_render::ImageFilterMode::PixelArt;
        }
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
                Some(
                    a::Foto::Editor
                    | a::Foto::FolhasEditor
                    | a::Foto::FolhasApagada
                    | a::Foto::Alinhar
                    | a::Foto::AlinharEscala
                    | a::Foto::AlinharDesfaz
                    | a::Foto::AlinharPes
                    | a::Foto::AlinharPesDesfaz,
                ),
                Some(entity_bits),
            ) => h.bus.push(EditorAction::InspectorAnimEdit {
                entity_bits,
                edit: ph2d_editor_core::AnimFieldEdit::OpenEditor,
            }),
            _ => {}
        }
    }
}

/// O centro do quad da folha visível de `e` no espaço do pai (a origem mais o Pivot dela, pela pose do objeto).
fn centro_do_quad(world: &ph2d_ecs::World, e: ph2d_ecs::Entity) -> Option<[f32; 2]> {
    let (_, s) = ph2d_sprite_edit::pivot::sprite_do_pivot(world, e)?;
    let t = *world.get::<ph2d_ecs::Transform>(e)?;
    let mut filho = ph2d_ecs::Transform::default();
    (filho.translation.x, filho.translation.y) = (s.anchor[0], s.anchor[1]);
    let c = ph2d_ecs::Transform::compose(t, filho).translation;
    Some([c.x, c.y])
}
