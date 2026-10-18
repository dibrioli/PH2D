//! `PH2D_MACRO_SMOKE` — as cenas de [`ph2d_app_macros::CENAS_DO_SMOKE`]. A família monta o objecto e
//! grava a macro; aqui monta-se o clipe (`add_macro_clip`, o laço `[0, D]`, o Autoplay) e a cena corre.

use ph2d_editor_core::macro_edits::MacroRequest;
use ph2d_editor_core::panel::PanelHostInternal;
use ph2d_timeline::TimelineIntent;

static FEITO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// A tela da cena (`=6` conduz-se sobre ela).
static TELA: std::sync::OnceLock<u64> = std::sync::OnceLock::new();

thread_local! {
    /// `=6`: o relógio da cena e o condutor do roteiro.
    static CAMINHO: std::cell::RefCell<Option<(std::time::Instant, ph2d_app_painter::macro_track::caminho::Caminho)>> =
        const { std::cell::RefCell::new(None) };
}

/// A montagem de uma cena do Painter: a tela nasce na próxima célula de importação.
type MontaTela<T> = fn(
    &mut ph2d_ecs::SimWorld,
    &mut ph2d_render::SpriteRenderer,
    &ph2d_asset::AssetDb,
    u32,
    f32,
    &mut std::collections::BTreeMap<u32, ph2d_asset::AssetId>,
) -> Option<T>;

impl crate::App {
    fn com_tela<T>(&mut self, monta: MontaTela<T>) -> Option<T> {
        let gfx = self.gfx.as_mut()?;
        let ppm = gfx.hero_screen.as_ref()?.project.pixels_per_meter;
        let r = monta(
            &mut gfx.sim,
            &mut gfx.renderer,
            &gfx.asset_db,
            gfx.next_import_cell,
            ppm,
            &mut gfx.atlas_asset_map,
        );
        gfx.next_import_cell += u32::from(r.is_some());
        r
    }

    pub(crate) fn macro_smoke(&mut self) {
        let Ok(nivel) = std::env::var("PH2D_MACRO_SMOKE") else {
            return;
        };
        if nivel == "6" {
            self.caminho_do_dono();
        }
        if self.gfx.as_ref().is_none_or(|g| g.hero_screen.is_none())
            || FEITO.swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            return; // ainda sem ecrã; tenta no próximo quadro
        }
        let montada = match nivel.as_str() {
            "1" => self.com_tela(ph2d_app_painter::macro_track::smoke::monta),
            "5" => self.com_tela(ph2d_app_painter::macro_track::smoke::monta_artista),
            // A mesma cena; o caminho do dono conduz-se em `caminho_do_dono`.
            "6" => self.com_tela(ph2d_app_painter::macro_track::smoke::monta_artista),
            "2" => self
                .with_motion_scene(ph2d_app_motion::macro_track::smoke::monta)
                .flatten(),
            // O tutorial «fazer»: sem clipe — a macro não toca, o aluno faz.
            "3" => {
                let bits = self
                    .with_motion_scene(ph2d_app_motion::macro_track::smoke::monta_fazer)
                    .flatten();
                if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
                    hero.gizmo.replace_selection(bits);
                    hero.set_panel_visible("hierarchy", true);
                }
                return;
            }
            // O tutorial «fazer» do Painter: a tela em modo Paint (o Wet Paint arma-se no bind).
            "4" => {
                let bits = self.com_tela(ph2d_app_painter::macro_track::smoke::monta_fazer);
                if let (Some(hero), Some(b)) =
                    (self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()), bits)
                {
                    ph2d_app_painter::composite_smoke::arm_hero(hero, b);
                    hero.set_panel_visible("hierarchy", true);
                }
                return;
            }
            _ => ph2d_app_macros::cena_desconhecida(&nivel),
        };
        let Some((bits, duracao)) = montada else {
            return;
        };
        let _ = TELA.set(bits);
        let doc = &mut self.timeline.doc;
        let Ok(clip) = doc.add_macro_clip("Macro".into(), bits, duracao) else {
            return;
        };
        doc.set_active(clip);
        doc.set_active_loop_for(true, Some((0.0, duracao)));
        let parada = ph2d_app_macros::a_cena_abre_parada(&nivel);
        doc.set_autoplay(clip, !parada);
        if !parada {
            self.timeline_intents
                .push(ph2d_timeline::TimelineIntent::RunScene);
        }
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(Some(bits));
            if parada {
                ph2d_app_painter::composite_smoke::arm_hero(hero, bits);
            }
            hero.set_panel_visible("timeline", true);
            hero.set_panel_visible("hierarchy", true);
            // As etapas à vista: o painel MACROS (lista e propriedades) e a linha «Macro steps».
            hero.set_panel_visible("macros", true);
        }
    }
    /// `PH2D_MACRO_SMOKE=6` — **o caminho do dono** ([`ph2d_app_painter::macro_track::caminho`]): os
    /// gestos pelas portas de sempre (o painel MACROS, o ponteiro da tela, o transporte da Keys) e a
    /// linha de diagnóstico a cada décimo de segundo.
    fn caminho_do_dono(&mut self) {
        use ph2d_app_painter::macro_track::caminho::{Caminho, Gesto, regista};
        let Some(&bits) = TELA.get() else {
            return;
        };
        let (ms, gestos, diag) = CAMINHO.with_borrow_mut(|c| {
            let (t0, cam) = c.get_or_insert_with(|| {
                regista(&ph2d_app_painter::macro_track::caminho::anuncio());
                (std::time::Instant::now(), Caminho::default())
            });
            let ms = u64::try_from(t0.elapsed().as_millis()).unwrap_or(u64::MAX);
            (ms, cam.gestos_ate(ms), cam.hora_do_diagnostico(ms))
        });
        for g in gestos {
            regista(&format!("{ms} ms: {g:?}"));
            match g {
                Gesto::Grava => ph2d_app_macros::com(|m| m.pede(MacroRequest::Grava)),
                Gesto::Para => ph2d_app_macros::com(|m| m.pede(MacroRequest::Para)),
                Gesto::Rewind => {
                    self.timeline_intents.push(TimelineIntent::Scrub(0.0));
                    self.timeline_reveal_after_apply = true;
                }
                Gesto::Play => self.timeline_intents.push(TimelineIntent::TogglePlay),
                Gesto::Desce(uv) | Gesto::Move(uv) => {
                    let Some((px, py)) = self.ponto_da_tela(bits, uv) else {
                        continue;
                    };
                    self.last_pointer = (px, py);
                    let ok = if matches!(g, Gesto::Desce(_)) {
                        self.painter_canvas_down(px, py, 1.0)
                    } else {
                        self.painter_canvas_move(px, py)
                    };
                    if !ok {
                        regista(&format!("the canvas refused {g:?} at ({px:.0}, {py:.0})"));
                    }
                }
                Gesto::Sobe => self.painter_canvas_up(),
                Gesto::MeioWet => {
                    if let Some(p) = self.gfx.as_mut().and_then(|g| {
                        ph2d_app_painter::painter_bridge_queries::o_painter(&mut g.tools)
                    }) {
                        ph2d_app_painter::macro_track::smoke::arm_wet(p);
                    }
                }
            }
        }
        let Some(quadros) = diag else {
            return;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let e = ph2d_ecs::Entity::from_bits(bits);
        let t = gfx
            .sim
            .world()
            .get::<ph2d_ecs::MacroTempo>(e)
            .map(|t| t.segundos);
        let (gravando, cede) = ph2d_app_macros::com(|m| (m.gravando(), m.sprite_gravada()));
        let painter = ph2d_app_painter::painter_bridge_queries::o_painter(&mut gfx.tools);
        let linha = gfx
            .painter_macros
            .diagnostico(bits, cede, painter.as_deref());
        let (ph, clip) = (self.playhead.time(), self.clip_playhead.time());
        regista(&format!(
            "{ms} ms: {quadros} frames playhead={ph:.2} clip={clip:.2} t={t:?} gravando={gravando:?} {linha}"
        ));
    }

    /// O ponto do ecrã de `(u, v)` (fracções da imagem) na tela `bits`, pela pose de MUNDO.
    fn ponto_da_tela(&self, bits: u64, [u, v]: [f32; 2]) -> Option<(f32, f32)> {
        let gfx = self.gfx.as_ref()?;
        let e = ph2d_ecs::Entity::from_bits(bits);
        let tr = ph2d_ecs::world_transform(gfx.sim.world(), e)?;
        let sprite = gfx.sim.world().get::<ph2d_render::Sprite>(e)?;
        let grade = gfx.sim.world().get::<ph2d_ecs::SpriteGrid>(e).copied();
        let lado = ph2d_app_painter::macro_track::smoke::LADO;
        let affine = ph2d_sprite_screen::sprite_image_to_screen_affine(
            lado,
            lado,
            tr,
            sprite,
            grade,
            &gfx.camera,
            gfx.scene_window(),
        );
        let lado = f64::from(lado);
        let p = affine * ph2d_vector::Point::new(f64::from(u) * lado, f64::from(v) * lado);
        #[expect(clippy::cast_possible_truncation, reason = "um ponto do ecrã")]
        Some((p.x as f32, p.y as f32))
    }
}
