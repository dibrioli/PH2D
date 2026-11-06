//! As fotos da câmera do smoke do jogo (`camera_chave`, `camera_desfaz`, onda 18): nomes da cena e
//! linhas de consola para quem corre o smoke.

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
    pub(super) fn fim_da_imagem_do_smoke(&mut self, m: ph2d_app_game::smoke_mao::Mao) {
        use ph2d_app_game::smoke_mao::Mao;
        use ph2d_editor_core::object_mode::{ModeRequest, ObjectMode};
        self.any_input_this_frame = true;
        match m {
            Mao::Desfaz => self.smoke_undo(false),
            Mao::Agulha(t) => self.playhead.seek(t),
            Mao::Desloca | Mao::Pivo(_) => self.mexe_na_sprite_do_smoke(m),
            Mao::Prova => self.prova_da_sprite_do_smoke(),
            Mao::SaiDoDraw => {
                if let Some(h) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                    h.bus
                        .push(ph2d_editor_core::action_bus::EditorAction::ObjectMode(
                            ModeRequest::Enter(ObjectMode::Object),
                        ));
                }
            }
            Mao::Aplica => {
                let alvo = self
                    .gfx
                    .as_mut()
                    .and_then(|g| ph2d_app_game::smoke::por_nome(&mut g.sim, "Platform High"));
                if let (Some(e), Some(h)) =
                    (alvo, self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()))
                {
                    h.bus
                        .push(ph2d_editor_core::action_bus::EditorAction::OneShotImageOp {
                            tool_id: "painter",
                            entity_bits: e.to_bits(),
                        });
                }
            }
            _ => {
                let Some(g) = self.gfx.as_mut() else { return };
                for nome in ["Platform High", "Platform Low"] {
                    let e = ph2d_app_game::smoke::por_nome(&mut g.sim, nome);
                    let id = e.map(|e| {
                        let w = g.sim.world();
                        ph2d_render::texture_of(
                            w.get::<ph2d_ecs::SpritePixels>(e),
                            w.get::<ph2d_render::Sprite>(e),
                            &g.atlas_asset_map,
                        )
                        .map(|a| a.as_bytes()[0])
                    });
                    eprintln!("[sprite-pixel] {nome}: imagem {id:?}");
                }
                eprintln!(
                    "[sprite-pixel] passos de desfazer: {} · Pixel Sprites: {} · marca do Painter: {}",
                    self.undo.depth(),
                    ph2d_app_pixel::mode::sprites(g.sim.world()).len(),
                    ph2d_sprite_edit::imagem::edicao_marcada(),
                );
            }
        }
    }

    /// O clique do Inspector sobre a Platform High: pela mesma ação do barramento.
    fn mexe_na_sprite_do_smoke(&mut self, m: ph2d_app_game::smoke_mao::Mao) {
        use ph2d_editor_core::action_bus::EditorAction;
        let Some(g) = self.gfx.as_mut() else { return };
        let Some(e) = ph2d_app_game::smoke::por_nome(&mut g.sim, "Platform High") else {
            return;
        };
        let t = g.sim.world().get::<ph2d_ecs::Transform>(e).copied();
        let (Some(t), Some(h)) = (t, g.hero_screen.as_mut()) else {
            return;
        };
        if let ph2d_app_game::smoke_mao::Mao::Pivo(ponto) = m {
            h.bus.push(EditorAction::ImageUseEdit {
                entity_bits: e.to_bits(),
                edit: ph2d_editor_core::ImageUseEdit::Pivot(ph2d_editor_core::SpritePivot::Ponto(
                    ponto,
                )),
            });
        } else {
            h.bus.push(EditorAction::InspectorTransformEdit(
                ph2d_editor_core::InspectorTransformInfo {
                    entity_bits: e.to_bits(),
                    translation: [t.translation.x + 1.0, t.translation.y],
                    rotation_rad: t.rotation,
                    scale: [t.scale.x, t.scale.y],
                    skew_rad: [t.skew_x, t.skew_y],
                },
            ));
        }
    }

    /// `[sprite-prova]`: a pose da Platform High, onde o desenho e a colisão caem no mundo, o filho, e as chaves.
    fn prova_da_sprite_do_smoke(&mut self) {
        let t = self.playhead.time();
        let tl = &self.timeline;
        let Some(g) = self.gfx.as_mut() else { return };
        let Some(e) = ph2d_app_game::smoke::por_nome(&mut g.sim, "Platform High") else {
            return;
        };
        let filho = ph2d_app_game::smoke::por_nome(&mut g.sim, "Coin 4");
        let chaves: Vec<String> = tl
            .doc
            .bindings()
            .iter()
            .filter(|b| b.entity == e.to_bits())
            .map(|b| {
                let n = tl.doc.active_clip().track(b.target).map_or(0, |k| k.len());
                match b.prop {
                    ph2d_timeline::PropKind::Field(f) => format!("campo{}={n}", f.campo),
                    p => format!("{p:?}={n}"),
                }
            })
            .collect();
        let w = g.sim.world();
        let p = w
            .get::<ph2d_ecs::Transform>(e)
            .map_or(ph2d_core::Vec2::ZERO, |t| t.translation);
        let s = w.get::<ph2d_render::Sprite>(e).copied();
        let no_mundo = |o: [f32; 2]| [p.x + o[0], p.y + o[1]];
        let desenho = s.map(|s| no_mundo(s.resolve_anchor(100.0)));
        let col = w
            .get::<ph2d_physics_ecs::Collider>(e)
            .map(|c| no_mundo(c.offset));
        let local = filho.and_then(|f| w.get::<ph2d_ecs::Transform>(f).map(|l| l.translation));
        let filho = local.map(|l| no_mundo([l.x, l.y]));
        eprintln!(
            "[sprite-prova] t={t:.2} desfazer={} posicao=[{},{}] anchor={:?} desenho={desenho:?} colisao={col:?} filho={filho:?} chaves=[{}]",
            self.undo.depth(),
            p.x,
            p.y,
            s.map(|s| s.anchor),
            chaves.join(" "),
        );
    }

    pub(super) fn formato_desfaz_do_smoke(&mut self, quadro: u32) {
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
