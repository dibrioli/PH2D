//! ⭐⭐ **O que a queda EXECUTA** (plano `docs/Components/07`, etapa B) — as três acções.
//!
//! ⛔ **Nenhuma decisão mora aqui.** Quem decide é a lei pura ([`ph2d_app_components::asset_drop::resolve`]);
//! isto é o braço. Uma decisão dentro do braço seria uma decisão sem gate.
//!
//! ⚠️ **E cada acção passa pela PORTA que já existe**, nunca por uma segunda:
//! - re-texturar → [`ph2d_app_sprite_anim::image_choice::choose_library_image`], a porta da escolha de
//!   imagem (referência, plano GEF 12);
//! - nascer uma sprite → [`crate::image_import::spawn_sprite`], a mesma do import de ficheiro;
//! - pôr um prefab → o mesmo `instance_verbs::drain(Verb::Place, …)` do duplo-clique e da
//!   Hierarquia. O que a queda acrescenta é **onde**.

use ph2d_asset::AssetId;
use ph2d_i18n::{tr, tr_with};
use ph2d_render::premul::{AlphaMode, SpriteImage};

impl crate::App {
    /// Os pixels de um asset, na forma que as portas do app consomem.
    ///
    /// ⚠️ `None` para um id que o `AssetDb` não tem, ou que não é uma imagem — e o chamador
    /// **diz** que não deu, em vez de a queda evaporar.
    fn image_of(&self, asset: [u8; 32]) -> Option<SpriteImage> {
        let gfx = self.gfx.as_ref()?;
        let a = gfx.asset_db.get(&AssetId::from_digest(asset))?;
        // ⭐ **Pela PORTA** (`image_rgba8`) — o `ph2d_asset::Asset` é `#[non_exhaustive]`, e um
        // `match` na variante de 8 bits aceitava uma imagem de 16 **em silêncio**: a queda
        // evaporava sobre um ficheiro perfeitamente válido, sem erro nenhum. A porta converte (e
        // não copia quando já é de 8 bits).
        let (width, height, pixels) = a.image_rgba8()?;
        Some(SpriteImage {
            width,
            height,
            pixels: pixels.into_owned(),
            // ⚠️ **`Straight`** — é o que o `AssetDb` guarda para uma imagem importada, e é o que o
            // `insert_image_rgba8` recebeu. Declarar `Premultiplied` aqui escureceria toda borda
            // macia, e o modo de falha seria visual e mudo.
            alpha: AlphaMode::Straight,
        })
    }

    /// **A sprite passa a mostrar esta imagem** — a queda na ranhura e o seletor (plano GEF 12).
    ///
    /// ⚠️ Pela porta [`ph2d_app_sprite_anim::image_choice::choose_library_image`]: REFERENCIA a imagem da
    /// biblioteca (nunca copia píxeis) e a escala do objecto fica — o tamanho segue a imagem, como na Unity
    /// e na Godot (medido, `docs/GameEngineFuncional/ferramentas/textura_godot.gd`).
    pub(crate) fn choose_image(&mut self, entity_bits: u64, asset: [u8; 32]) {
        let img = self.image_of(asset);
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let ppm = gfx
            .hero_screen
            .as_ref()
            .map_or(ph2d_editor_core::project::DEFAULT_PIXELS_PER_METER, |h| {
                h.project.pixels_per_meter
            })
            .max(crate::EPS_PIXELS_PER_METER);
        let renderer = &mut gfx.renderer;
        let r = ph2d_app_sprite_anim::image_choice::choose_library_image(
            &mut gfx.sim,
            &mut |w, h, px| {
                renderer
                    .acquire_individual(w, h, px)
                    .map_err(|e| e.to_string())
            },
            &gfx.asset_db,
            &gfx.atlas_asset_map,
            ppm,
            ph2d_ecs::Entity::from_bits(entity_bits),
            AssetId::from_digest(asset),
        );
        match r {
            Ok(()) => {
                // Uma folha largada propõe colunas × linhas no editor de animações (plano GEF 07).
                if let Some(img) = img {
                    ph2d_app_sprite_anim::editor::propose_after_drop(
                        &mut gfx.sim,
                        entity_bits,
                        &img.pixels,
                        img.width,
                        img.height,
                    );
                }
                gfx.toasts.push(ph2d_editor_core::Toast::success(tr(
                    "shell.asset_drop_apply.texture_applied",
                )));
            }
            Err(e) => {
                gfx.toasts.push(ph2d_editor_core::Toast::warning(tr_with(
                    "shell.asset_drop_apply.could_not_apply",
                    &[("e", &e)],
                )));
            }
        }
    }

    /// **Nasce uma sprite nova, com esta imagem, no ponto.**
    pub(super) fn drop_spawn_image(&mut self, asset: [u8; 32], world: [f32; 2]) {
        let Some(img) = self.image_of(asset) else {
            self.toast_drop_failed();
            return;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let ppm = gfx
            .hero_screen
            .as_ref()
            .map_or(ph2d_editor_core::project::DEFAULT_PIXELS_PER_METER, |h| {
                h.project.pixels_per_meter
            });
        let renderer = &mut gfx.renderer;
        let placed = ph2d_app_sprite_anim::image_choice::place_library_image(
            &mut gfx.sim,
            &mut |w, h, px| {
                renderer
                    .acquire_individual(w, h, px)
                    .map_err(|e| e.to_string())
            },
            &gfx.asset_db,
            &gfx.atlas_asset_map,
            ppm,
            world,
            tr("shell.asset_drop_apply.image"),
            AssetId::from_digest(asset),
        );
        let Ok(e) = placed else {
            self.toast_drop_failed();
            return;
        };
        let bits = e.to_bits();
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.gizmo.replace_selection(Some(bits));
        }
        ph2d_app_sprite_anim::editor::propose_after_drop(
            &mut gfx.sim,
            bits,
            &img.pixels,
            img.width,
            img.height,
        );
        gfx.toasts.push(ph2d_editor_core::Toast::success(tr(
            "shell.asset_drop_apply.image_placed",
        )));
    }

    /// **Uma cópia do prefab nasce ONDE a mão largou.**
    ///
    /// ⚠️ **A pose escreve-se DEPOIS do verbo**, e é o precedente da casa (o `cascade` do
    /// *Instantiate* faz o mesmo): o `instantiate_master` copia o `Transform` da receita verbatim,
    /// de propósito — uma prova de mutação já matou uma versão que o reescrevia lá dentro.
    ///
    /// ⛔ **A pose é LOCAL.** Sob uma receita que tem pai com escala, o ponto de queda chega
    /// escalado — é a mesma cerca que o `duplicate_subtree` já declara para o passo dele. Curá-lo
    /// pede a inversa do mundo do pai, e isso é wave própria: hoje o caso comum (receita na raiz)
    /// é exacto.
    pub(super) fn drop_place_prefab(&mut self, stable_id: u64, world: [f32; 2], over: Option<u64>) {
        // ⭐ **O MESMO verbo do duplo-clique**, com o `at` preenchido. O dreno já sabe pôr a cópia
        // no ponto e seleccioná-la — ver `render_loop::hierarchy`. Uma segunda rota de instanciar
        // aqui seria a que diverge no dia em que o verbo ganhar um passo.
        // W18: uma receita de MACRO aplica-se — por cima do objecto debaixo da mão, se ele a recebe.
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
            && !ph2d_app_macros::aplica_se_receita(&mut gfx.sim, stable_id, Some(world), over)
        {
            hero.bus.push(
                ph2d_editor_core::action_bus::EditorAction::AssetInstantiate {
                    stable_id,
                    at: Some(world),
                },
            );
        }
    }

    /// ⛔ **Uma queda que não deu diz que não deu.** Silêncio faria o artista concluir que colocou.
    fn toast_drop_failed(&mut self) {
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.toasts.push(ph2d_editor_core::Toast::warning(tr(
                "shell.asset_drop_apply.could_not_place_that",
            )));
        }
    }
}

impl crate::App {
    /// A cena 4 do jogo (plano GEF 12): as imagens dela entram na biblioteca; `PH2D_GAME_SMOKE_FOTO` conduz
    /// o instante que a foto quer (o seletor sobre os montes, ou as duas imagens trocadas).
    pub(crate) fn troca_de_imagem_do_smoke(&mut self, nivel: u32) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let mut ids = Vec::new();
        for path in ph2d_app_game::smoke::imagens_da_cena(nivel) {
            match ph2d_app_sprite_anim::image_choice::bring_into_library(
                &gfx.asset_db,
                &mut gfx.renderer,
                &mut gfx.next_import_cell,
                &mut gfx.atlas_asset_map,
                std::path::Path::new(path),
            ) {
                Ok(id) => ids.push(id),
                Err(e) => eprintln!("[game-smoke] {path}: {e}"),
            }
        }
        let ids: Vec<[u8; 32]> = ids.iter().map(|i| *i.as_bytes()).collect();
        let Some((foto, escolhas, alvo)) = ph2d_app_game::smoke::plano_da_foto(&mut gfx.sim, &ids)
        else {
            return;
        };
        if foto == ph2d_app_game::smoke::Foto::Seletor
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            ph2d_panel_asset_browser::choose_image_for(
                alvo,
                ph2d_app_game::smoke::CAMADAS_2[1].0.into(),
            );
            <_ as ph2d_editor_core::panel::PanelHostInternal>::set_panel_visible(
                hero,
                ph2d_panel_asset_browser::PANEL_ID,
                true,
            );
        }
        for (bits, asset) in escolhas {
            self.choose_image(bits, asset);
        }
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(Some(alvo));
        }
    }
}
