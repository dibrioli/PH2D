//! `PH2D_MACRO_SMOKE` — as cenas de [`ph2d_app_macros::CENAS_DO_SMOKE`]. A família monta o objecto e
//! grava a macro; aqui monta-se o clipe (`add_macro_clip`, o laço `[0, D]`, o Autoplay) e a cena corre.

use ph2d_editor_core::panel::PanelHostInternal;

static FEITO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

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
        if self.gfx.as_ref().is_none_or(|g| g.hero_screen.is_none())
            || FEITO.swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            return; // ainda sem ecrã; tenta no próximo quadro
        }
        let montada = match nivel.as_str() {
            "1" => self.com_tela(ph2d_app_painter::macro_track::smoke::monta),
            "5" => self.com_tela(ph2d_app_painter::macro_track::smoke::monta_artista),
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
}
