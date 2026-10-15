//! `PH2D_MACRO_SMOKE=1|2` — **as macros em cena** (`docs/Macros/01_plano_onda1.md` §6): `=1` o Splash
//! (Painter), `=2` o Motion nó a nó. A família monta o objecto e grava a macro; aqui monta-se o clipe
//! que a conduz — `add_macro_clip`, o laço do clipe `[0, D]` e o Autoplay — e a cena corre como pelo
//! Play da barra de cima. Cada volta do laço recua a macro ao início.

use ph2d_editor_core::panel::PanelHostInternal;

static FEITO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

impl crate::App {
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
            "1" => self.gfx.as_mut().and_then(|gfx| {
                let ppm = gfx.hero_screen.as_ref()?.project.pixels_per_meter;
                let r = ph2d_app_painter::macro_track::smoke::monta(
                    &mut gfx.sim,
                    &mut gfx.renderer,
                    &gfx.asset_db,
                    gfx.next_import_cell,
                    ppm,
                    &mut gfx.atlas_asset_map,
                );
                gfx.next_import_cell += u32::from(r.is_some());
                r
            }),
            "2" => self
                .with_motion_scene(ph2d_app_motion::macro_track::smoke::monta)
                .flatten(),
            _ => {
                eprintln!(
                    "[macro smoke] PH2D_MACRO_SMOKE={nivel}: the scenes are 1 (Splash) and 2 (Motion \
                     node by node)"
                );
                None
            }
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
        doc.set_autoplay(clip, true);
        self.timeline_intents
            .push(ph2d_timeline::TimelineIntent::RunScene);
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.gizmo.replace_selection(Some(bits));
            hero.set_panel_visible("timeline", true);
            hero.set_panel_visible("hierarchy", true);
            // As etapas à vista: o painel MACROS (lista e propriedades) e a linha «Macro steps».
            hero.set_panel_visible("macros", true);
        }
    }
}
