//! ⭐⭐ **O que a queda EXECUTA** (plano `docs/Components/07`, etapa B) — as três acções.
//!
//! ⛔ **Nenhuma decisão mora aqui.** Quem decide é a lei pura ([`crate::asset_drop::resolve`]);
//! isto é o braço. Uma decisão dentro do braço seria uma decisão sem gate.
//!
//! ⚠️ **E cada acção passa pela PORTA que já existe**, nunca por uma segunda:
//! - re-texturar → [`crate::hero_intents::texture_edit::commit_edited_texture`], o funil das oito
//!   ferramentas de imagem (ele sobe os pixels, interna o `AssetId` e re-aponta a sprite);
//! - nascer uma sprite → [`crate::image_import::spawn_sprite`], a mesma do import de ficheiro;
//! - pôr um prefab → o mesmo `instance_verbs::drain(Verb::Place, …)` do duplo-clique e da
//!   Hierarquia. O que a queda acrescenta é **onde**.

use ph2d_asset::AssetId;
use ph2d_render::premul::{AlphaMode, SpriteImage};

impl crate::App {
    /// Os pixels de um asset, na forma que as portas do app consomem.
    ///
    /// ⚠️ `None` para um id que o `AssetDb` não tem, ou que não é uma imagem de 8 bits — e o
    /// chamador **diz** que não deu, em vez de a queda evaporar.
    fn image_of(&self, asset: [u8; 32]) -> Option<SpriteImage> {
        let gfx = self.gfx.as_ref()?;
        let a = gfx.asset_db.get(&AssetId::from_digest(asset))?;
        match &*a {
            ph2d_asset::Asset::ImageRgba8 {
                width,
                height,
                pixels,
            } => Some(SpriteImage {
                width: *width,
                height: *height,
                pixels: pixels.to_vec(),
                // ⚠️ **`Straight`** — é o que o `AssetDb` guarda para uma imagem importada, e é o
                // que o `insert_image_rgba8` recebeu. Declarar `Premultiplied` aqui escureceria
                // toda borda macia, e o modo de falha seria visual e mudo.
                alpha: AlphaMode::Straight,
            }),
            _ => None,
        }
    }

    /// **A sprite passa a mostrar esta imagem.**
    ///
    /// ⚠️ **O TAMANHO não muda**, e é uma escolha: re-texturar não é redimensionar. Trocar a
    /// imagem de um objecto posto na cena e vê-lo saltar de tamanho é perder a composição.
    pub(super) fn drop_retexture(&mut self, entity_bits: u64, asset: [u8; 32]) {
        let Some(img) = self.image_of(asset) else {
            self.toast_drop_failed();
            return;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let e = ph2d_ecs::Entity::from_bits(entity_bits);
        let Some(size) = gfx
            .sim
            .world()
            .get::<ph2d_render::Sprite>(e)
            .map(|s| s.size)
        else {
            return;
        };
        let mut toasts = std::mem::take(&mut gfx.toasts);
        let r = crate::hero_intents::texture_edit::commit_edited_texture(
            e,
            &mut gfx.sim,
            &mut gfx.renderer,
            &gfx.asset_db,
            &img,
            size,
            &mut toasts,
        );
        gfx.toasts = toasts;
        match r {
            Ok(_) => {
                gfx.toasts
                    .push(ph2d_editor::Toast::success("Texture applied"));
            }
            Err(e) => {
                gfx.toasts
                    .push(ph2d_editor::Toast::warning(format!("Could not apply: {e}")));
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
        let Ok(texture_id) = gfx
            .renderer
            .acquire_individual(img.width, img.height, &img.pixels)
        else {
            self.toast_drop_failed();
            return;
        };
        // ⚠️ **Os dois travões que toda conversão px→mundo deste shell aplica** — e a 1.ª versão
        // desta era a **única** de oito que não os aplicava. Um `ppm` a zero dá tamanho `inf`; um
        // `ppm` grande com uma imagem pequena dá uma sprite abaixo do mínimo que o resto do código
        // assume não poder existir.
        let ppm = gfx
            .hero_screen
            .as_ref()
            .map_or(ph2d_editor::project::DEFAULT_PIXELS_PER_METER, |h| {
                h.project.pixels_per_meter
            })
            .max(crate::EPS_PIXELS_PER_METER);
        let world_size = [
            (img.width as f32 / ppm).max(crate::MIN_SPRITE_SIZE),
            (img.height as f32 / ppm).max(crate::MIN_SPRITE_SIZE),
        ];
        let (_, bits) = crate::image_import::spawn_sprite(
            &mut gfx.sim,
            crate::image_import::PackedSource::Individual {
                texture_id,
                pixels_id: AssetId::from_digest(asset),
            },
            ph2d_core::Vec2::new(world[0], world[1]),
            world_size,
            "Image",
        );
        if let Some(hero) = gfx.hero_screen.as_mut() {
            hero.gizmo.replace_selection(Some(bits));
        }
        gfx.toasts.push(ph2d_editor::Toast::success("Image placed"));
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
    pub(super) fn drop_place_prefab(&mut self, stable_id: u64, world: [f32; 2]) {
        // ⭐ **O MESMO verbo do duplo-clique**, com o `at` preenchido. O dreno já sabe pôr a cópia
        // no ponto e seleccioná-la — ver `render_loop::hierarchy`. Uma segunda rota de instanciar
        // aqui seria a que diverge no dia em que o verbo ganhar um passo.
        if let Some(gfx) = self.gfx.as_mut()
            && let Some(hero) = gfx.hero_screen.as_mut()
        {
            hero.bus
                .push(ph2d_editor::action_bus::EditorAction::AssetInstantiate {
                    stable_id,
                    at: Some(world),
                });
        }
    }

    /// ⛔ **Uma queda que não deu diz que não deu.** Silêncio faria o artista concluir que colocou.
    fn toast_drop_failed(&mut self) {
        if let Some(gfx) = self.gfx.as_mut() {
            gfx.toasts
                .push(ph2d_editor::Toast::warning("Could not place that asset"));
        }
    }
}
