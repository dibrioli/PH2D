//! **O lado da ESCRITA** do projecto: no pai (`project.rs`) *o que um arquivo É*, aqui *como é
//! ESCRITO* (e a cópia de recuperação, `ph2d_recovery`), no `project_load.rs` *como é lido*.

use super::{PROJECT_SCHEMA, ProjectFile};
use ph2d_i18n::tr_with;

impl crate::App {
    /// Grava o projeto inteiro em `path`. ⚠️ O caminho é PARÂMETRO: quem decide *onde* é o
    /// [`crate::App::project_save_gesture`] (pergunta quando ainda não há ficheiro).
    pub(crate) fn project_save_to(&mut self, path: &str) {
        let Some(bytes) = self.project_bytes() else {
            return;
        };
        match std::fs::write(path, &bytes) {
            Ok(()) => {
                eprintln!("[proj] salvo: {path} ({} bytes)", bytes.len());
                self.recovery.saved(&bytes, std::time::Instant::now());
                let n = self.timeline.doc.bindings().len();
                self.toast(tr_with(
                    "shell.project_save.project_saved_kb",
                    &[("bytes", &(bytes.len() / 1024)), ("n", &n)],
                ));
            }
            Err(e) => {
                eprintln!("[proj] erro ao gravar {path}: {e}");
                self.toast(tr_with(
                    "shell.project_save.project_save_failed",
                    &[("e", &e)],
                ));
            }
        }
    }

    /// Arranque da recuperação: a sessão, a varredura e a janela em cartões (`ph2d_recovery::card`).
    pub(crate) fn recovery_start(&mut self, root: Option<&std::path::Path>) {
        self.recovery = ph2d_recovery::Recovery::start(root, crate::prefs::recovery());
        self.recovery_opened(false);
    }

    /// Abriu-se um projecto: a linha de base da cópia e do «por gravar» são os bytes de agora.
    pub(crate) fn recovery_opened(&mut self, restored: bool) {
        let bytes = self.project_bytes();
        self.recovery.opened(bytes.as_deref(), restored);
    }

    /// A janela da recuperação é modal: o clique é dela. `Restore` abre a cópia por gravar com o
    /// caminho original (o Save grava lá); as outras ficam para depois.
    pub(crate) fn recovery_click(&mut self, evt: &ph2d_host::PointerEvent) -> bool {
        use ph2d_recovery::card::Hit;
        let down = evt.kind == ph2d_host::PointerKind::Down;
        let Some(hit) = ph2d_recovery::card::click(&mut self.recovery, evt.x, evt.y) else {
            return false;
        };
        match hit {
            Hit::Answer(i, choice) if down => {
                if let Some((copy, original)) = self.recovery.answer(i, choice) {
                    self.recovery.prompt.later_all();
                    self.project_load_from(&copy.to_string_lossy());
                    self.project_path = original;
                    self.title_dirty = true;
                    self.recovery_opened(true);
                }
            }
            Hit::Save if down => {
                self.project_save_gesture(false);
                let bytes = self.project_bytes();
                self.recovery.saved_then_quit(bytes.as_deref());
            }
            Hit::DontSave if down => self.recovery.quit = ph2d_recovery::Quit::Go,
            _ => {}
        }
        true
    }

    /// O fecho com alterações por gravar pergunta antes (`true` = não fecha ainda).
    pub(crate) fn recovery_quit_guard(&mut self) -> bool {
        if self.recovery.quit == ph2d_recovery::Quit::Go {
            return false;
        }
        let bytes = self.project_bytes();
        let path = self.project_path.clone();
        self.recovery.ask_quit(bytes.as_deref(), path.as_deref())
    }

    /// ⭐ A cópia de recuperação (P12 S, `ph2d_recovery`): à hora, os MESMOS bytes do Save.
    pub(crate) fn recovery_tick(&mut self) {
        let now = std::time::Instant::now();
        if self.recovery.due(now)
            && let Some(bytes) = self.project_bytes()
        {
            let path = self.project_path.clone();
            self.recovery.offer(bytes, path.as_deref(), now);
        }
    }

    /// Os bytes do projecto inteiro, como o Save os grava (`None` = recusado, e já se disse porquê).
    pub(crate) fn project_bytes(&mut self) -> Option<Vec<u8>> {
        let assets = self.collect_assets();
        // Os documentos pintados carimbam a identidade estável no mundo: ANTES da captura.
        let painted = self.collect_painted_docs();
        // Os pixels de todo sprite Individual, DEPOIS do `collect_painted_docs` (que eles saltam).
        let sprite_pixels = self.collect_sprite_pixels();
        // A arte dos padrões de textura (plano 33 W4), por `AssetId`.
        let pattern_art = self
            .gfx
            .as_ref()
            .map(|g| g.vec_scene.clone())
            .map(|scene| self.collect_texture_pattern_art(&scene))
            .unwrap_or_default();
        // A animação: o `serialize` carimba o `StableId` em cada binding (`&mut`, antes da captura).
        let timeline = match self.gfx.as_mut() {
            Some(gfx) => {
                let world = gfx.sim.world_mut();
                match ph2d_timeline_persist::serialize(&mut self.timeline, world) {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("[proj] timeline nao serializou, projeto salvo SEM ela: {e}");
                        Vec::new()
                    }
                }
            }
            None => Vec::new(),
        };
        // Os QUADROS e os KANBANS (ADR-0195): sem bytes NÃO se grava (apagaria os do disco).
        let boards = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .map_or(Ok(Vec::new()), |h| h.documents.boards().to_bytes());
        let boards = match boards {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[proj] quadros nao serializaram — save RECUSADO: {e}");
                return None;
            }
        };
        let hero = self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref());
        let kanbans = ph2d_app_kanban::project::to_bytes(hero.map(|h| &h.documents))?;
        // A meio do Play o documento é a fotografia (`undo::jogo`): o que o jogo mudou não se grava.
        let state = self
            .documento_do_jogo()
            .or_else(|| self.capture_project())?;
        let file = ProjectFile {
            state,
            assets,
            painted,
            motion: self
                .gfx
                .as_ref()
                .map(|g| g.motion.doc.to_text())
                .unwrap_or_default(),
            timeline,
            physics: self
                .gfx
                .as_ref()
                .map(|g| g.physics.settings())
                .unwrap_or_default(),
            tokens: crate::project_tokens::collect(),
            // Sem janela não há `HeroScreen`: o save headless grava os defaults.
            settings: crate::project_settings::collect(
                self.gfx
                    .as_ref()
                    .and_then(|g| g.hero_screen.as_ref())
                    .map(|h| h.project)
                    .unwrap_or_default(),
            ),
            // A corrida que o artista jogou (W17); o `to_wire` é a única tradução.
            player_tape: self.player_tape.to_wire(),
            sprite_pixels,
            // ⚠️ O contador de identidade (ADR-0164 F1), só fotografado; o load reconcilia-o.
            stable_id_counter: self
                .gfx
                .as_ref()
                .and_then(|g| g.sim.world().get_resource::<ph2d_ecs::StableIdCounter>())
                .map_or(ph2d_ecs::StableId::FIRST, |c| c.next_free()),
            // O Input Map por VALOR (v97); dono: o `HeroScreen` (sem hero, o vazio).
            input_map: self
                .gfx
                .as_ref()
                .and_then(|g| g.hero_screen.as_ref())
                .map(|h| h.input_map.clone())
                .unwrap_or_default(),
            pattern_art,
            boards,
            scripts: self
                .gfx
                .as_ref()
                .map(|g| ph2d_app_components::script_project::save(g.sim.world()))
                .unwrap_or_default(),
            marcas_do_artista: self
                .gfx
                .as_mut()
                .map(|g| {
                    let t = ph2d_app_painter::painter_bridge_queries::o_painter(&mut g.tools);
                    g.painter_macros.marcas_a_gravar(t.map(|t| &*t))
                })
                .unwrap_or_default(),
            jogo: crate::project_settings::collect_jogo(
                self.gfx
                    .as_ref()
                    .and_then(|g| g.hero_screen.as_ref())
                    .map(|h| h.project)
                    .unwrap_or_default(),
            ),
            // O AUTORADO: uma curva da Cena a tocar não entra no ficheiro (F30 §B2).
            mesa: (self.gfx.as_ref())
                .map(|g| ph2d_app_audio::mesa_doc::autorado(&g.sim).to_bytes())
                .unwrap_or_default(),
            kanbans,
            pixel_sprites: self
                .gfx
                .as_ref()
                .map(|g| ph2d_app_pixel::persist::to_bytes(&mut self.pixel, g.sim.world()))
                .unwrap_or_default(),
            sounds: (self.gfx.as_ref())
                .map(|g| ph2d_app_components::sound_project::save(g.sim.world()))
                .unwrap_or_default(),
            sfx_lab: (self.gfx.as_ref())
                .map(|g| ph2d_sfx_recipe::project::save(g.sim.world()))
                .unwrap_or_default(),
            physics_layers: self.gfx.as_ref().map_or_else(Vec::new, |g| {
                ph2d_physics_ecs::LayerNames::of(g.sim.world()).to_bytes()
            }),
            // ⚠️ A taxonomia NÃO está aqui (v105): viaja no `state` ([`crate::project_library`]).
        };
        postcard::to_allocvec(&(PROJECT_SCHEMA, &file))
            .map_err(|e| eprintln!("[proj] falha ao serializar: {e}"))
            .ok()
    }

    /// Um aviso na tela, não só no terminal — a porta única (o *Export SVG…* usa a mesma).
    pub(crate) fn toast(&mut self, msg: String) {
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.toasts.push(ph2d_editor_core::Toast::info(msg));
        }
    }
}

#[cfg(test)]
#[path = "project_save_recovery_tests.rs"]
mod recovery_tests;
