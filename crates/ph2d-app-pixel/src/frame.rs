//! **O quadro da família**: a cena de smoke armada, os documentos que os objectos pedem (cópia e
//! ausência) e o enquadramento ao entrar em Draw.

use ph2d_ecs::SimWorld;
use ph2d_host::WindowSize;
use ph2d_pixel_core::NewSprite;
use ph2d_pixel_render::{px_per_cell, zoom};
use ph2d_render::Camera2d;
use ph2d_render::camera_frame::Area;

use crate::PixelApp;
use crate::input::{set_zoom, to_screen};
use crate::mode::sprites;
use crate::state::Doc;

/// Parte da área livre do desenho que um sprite novo ocupa ao abrir (o *Fit Screen* com margem).
const FIT_FRACTION: f64 = 0.6;

impl PixelApp {
    /// Uma vez por quadro, antes do modo e da entrada. `editing` = o objecto em Pixel Sprite ▸
    /// Draw; `spawn_root` cria o objecto vazio da casa (nome único, ordem de raiz).
    #[allow(clippy::too_many_arguments)]
    pub fn frame(
        &mut self,
        sim: &mut SimWorld,
        camera: &mut Camera2d,
        window: WindowSize,
        ppm: f32,
        editing: Option<u64>,
        spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
    ) {
        let whole = Area {
            x: 0.0,
            y: 0.0,
            w: window.width as f32,
            h: window.height as f32,
        };
        self.frame_in(sim, camera, window, ppm, editing, whole, spawn_root);
    }

    /// O [`PixelApp::frame`] com a área livre do desenho (`area`, px da janela): é nela que um
    /// sprite novo se enquadra.
    #[allow(clippy::too_many_arguments)]
    pub fn frame_in(
        &mut self,
        sim: &mut SimWorld,
        camera: &mut Camera2d,
        window: WindowSize,
        ppm: f32,
        editing: Option<u64>,
        area: Area,
        spawn_root: impl FnOnce(&mut SimWorld, &str) -> u64,
    ) {
        if let Some(level) = self.smoke.take()
            && !crate::smoke::stage(self, sim, level, spawn_root)
        {
            eprintln!("[pixel-smoke] nível {level} desconhecido ou falhou");
        }
        self.reconcile(sim);
        if editing.is_none() {
            // SAIR do Draw assenta as células levantadas (o Aseprite larga-as antes de mudar); a cena
            // de smoke que levanta antes do 1.º Draw não saiu de lado nenhum.
            if self.was_editing.is_some() {
                self.assenta();
                crate::viva::assenta(self);
            }
            self.laco = None;
            // A forma viva da cena de smoke, montada antes do 1.º Draw, também não saiu de lado
            // nenhum (a de quem SAIU já assentou acima).
            if self.gesture.as_ref().is_none_or(|g| g.viva.is_none()) {
                self.gesture = None;
            }
            self.hover = None;
            self.sobre_eixo = false;
            self.space = false;
            self.panning = false;
            self.fit = None;
        }
        if editing != self.was_editing
            && let Some(bits) = editing
        {
            self.frame_on_entry(sim.world(), camera, window, ppm, bits, area);
        } else if let (Some(bits), Some((of, at, view))) = (editing, self.fit) {
            if of != bits || camera.view() != view || self.gesture.is_some() {
                self.fit = None;
            } else if at != area {
                self.fit_in(sim.world(), camera, window, ppm, bits, area);
            }
        }
        self.was_editing = editing;
        self.spray_tick();
        self.zoom = editing.and_then(|bits| {
            let doc = self.docs.get(&crate::input::key_of(sim.world(), bits)?)?;
            let m = to_screen(sim.world(), bits, doc, camera, window, ppm)?;
            Some(zoom::nearest(px_per_cell(m)))
        });
    }

    /// O spray segurado borrifa a cada quadro, mesmo com o ponteiro parado (Aseprite).
    fn spray_tick(&mut self) {
        let held = self
            .gesture
            .as_ref()
            .is_some_and(|g| g.held && g.kind == crate::gesture::Kind::Spray);
        if !held {
            return;
        }
        let (w, n) = (self.options.spray_width, self.options.spray_speed);
        let stamp = self.stamp();
        if let Some(g) = self.gesture.as_mut()
            && let Some(doc) = self.docs.get_mut(&g.key)
        {
            g.stroke.spray(&doc.sprite, g.cursor, w, n);
            doc.stamp = stamp;
        }
    }

    /// Todo objecto aponta para um documento SEU: uma cópia do objecto (duplicar) ganha uma cópia
    /// do documento; um objecto sem documento (ficheiro partido) ganha um vazio.
    fn reconcile(&mut self, sim: &mut SimWorld) {
        let mut seen = std::collections::BTreeSet::new();
        for (bits, key) in sprites(sim.world()) {
            let fresh = !seen.insert(key);
            let missing = !self.docs.contains_key(&key);
            if !fresh && !missing {
                continue;
            }
            let stamp = self.stamp();
            let doc = match self.docs.get(&key) {
                Some(d) if !missing => {
                    let mut c = Doc::new(d.sprite.clone(), stamp);
                    c.fresh = false;
                    c
                }
                _ => match ph2d_pixel_core::Sprite::new(NewSprite::default()) {
                    Ok(s) => Doc::new(s, stamp),
                    Err(_) => continue,
                },
            };
            let new_key = if missing {
                key
            } else {
                self.docs.keys().next_back().map_or(1, |k| k + 1)
            };
            self.docs.insert(new_key, doc);
            seen.insert(new_key);
            if new_key != key
                && let Some(e) = ph2d_ecs::Entity::try_from_bits(bits)
                && let Ok(mut ent) = sim.world_mut().get_entity_mut(e)
            {
                ent.insert(ph2d_ecs::PixelSprite(new_key));
            }
        }
    }

    /// Ao entrar em Draw: um sprite novo enquadra-se na área livre ([`Self::fit_in`]); um já aberto
    /// antes só prende o zoom ao degrau inteiro mais próximo.
    fn frame_on_entry(
        &mut self,
        world: &ph2d_ecs::World,
        camera: &mut Camera2d,
        window: WindowSize,
        ppm: f32,
        bits: u64,
        area: Area,
    ) {
        self.fit = None;
        let Some(key) = crate::input::key_of(world, bits) else {
            return;
        };
        let Some(doc) = self.docs.get_mut(&key) else {
            return;
        };
        if std::mem::take(&mut doc.fresh) {
            self.fit_in(world, camera, window, ppm, bits, area);
            return;
        }
        let Some(m) = to_screen(world, bits, doc, camera, window, ppm) else {
            return;
        };
        let center = (window.width as f32 * 0.5, window.height as f32 * 0.5);
        set_zoom(
            world,
            bits,
            doc,
            camera,
            window,
            ppm,
            zoom::nearest(px_per_cell(m)),
            center,
        );
    }

    /// O *Fit Screen*: o sprite no centro de `area` (px da janela) com o maior degrau inteiro que
    /// cabe em [`FIT_FRACTION`] dela. Fica a seguir a área (`self.fit`) até o artista mexer.
    fn fit_in(
        &mut self,
        world: &ph2d_ecs::World,
        camera: &mut Camera2d,
        window: WindowSize,
        ppm: f32,
        bits: u64,
        area: Area,
    ) {
        let Some(doc) = crate::input::key_of(world, bits).and_then(|k| self.docs.get(&k)) else {
            return;
        };
        if let Some(e) = ph2d_ecs::Entity::try_from_bits(bits)
            && let Some(tr) = ph2d_ecs::world_transform(world, e)
        {
            camera.center = [tr.translation.x, tr.translation.y];
        }
        let mid = (area.x + area.w * 0.5, area.y + area.h * 0.5);
        let here = camera.screen_to_world(
            (window.width as f32 * 0.5, window.height as f32 * 0.5),
            window,
        );
        let there = camera.screen_to_world(mid, window);
        camera.center[0] += here[0] - there[0];
        camera.center[1] += here[1] - there[1];
        let target = zoom::fit(
            doc.sprite.width(),
            doc.sprite.height(),
            f64::from(area.w) * FIT_FRACTION,
            f64::from(area.h) * FIT_FRACTION,
        );
        set_zoom(world, bits, doc, camera, window, ppm, target, mid);
        self.fit = Some((bits, area, camera.view()));
    }
}
