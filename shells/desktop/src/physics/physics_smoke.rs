//! `PH2D_PHYSICS_SMOKE=<n>`: as cenas moram em `ph2d_app_physics::smoke`, o índice em
//! `docs/Physics/00_plano_waves.md`. Aqui só o prólogo: quem decide o quadro é a composição.

impl crate::App {
    /// Corre a cena da crate e aplica o que ela pediu. O `ctx` empresta `gfx.sim`: fecha antes de aplicar.
    fn run_physics_scene(&mut self, which: &str) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let mut ctx = ph2d_app_physics::SceneCtx::new(
            gfx.sim.world_mut(),
            gfx.physics.settings(),
            &mut self.timeline.doc,
        );
        ph2d_app_physics::smoke::scene(which, &mut ctx);
        let want = std::mem::take(&mut ctx.want);
        drop(ctx);

        if let Some(c) = want.camera_center {
            gfx.camera.center = c;
        }
        if let Some(h) = want.camera_height_world {
            gfx.camera.height_world = h;
        }
        if let Some(s) = want.settings {
            gfx.physics.set_settings(s);
        }
        if let Some((a, b)) = want.playhead_loop {
            self.playhead.set_loop(a, b);
        }
        if let Some(bits) = want.player_readout_log {
            self.physics.player_readout_log = Some(bits);
        }
        if let Some(hero) = gfx.hero_screen.as_mut() {
            for k in &want.panels {
                hero.panel_visibility.insert(k, true);
            }
            if let Some(bits) = want.select {
                hero.gizmo.selection = Some(bits);
                hero.gizmo.extra_selection.clear();
            }
        }
    }

    /// Frame prologue, once. No-op without the env.
    pub(crate) fn physics_smoke(&mut self) {
        let Some(which) = ph2d_app_physics::smoke::armed_scene() else {
            return;
        };
        if self.physics.smoke_done {
            return;
        }
        if self.gfx.is_none() {
            return; // world not up yet; retry next frame
        }
        self.physics.smoke_done = true;

        self.run_physics_scene(which.trim());
        // Nome → identidade, uma vez, para toda cena: a ponte resolve juntas por `StableId` (ADR-0164 F1).
        if let Some(gfx) = self.gfx.as_mut() {
            let n = ph2d_physics_ecs::resolve_body_names(gfx.sim.world_mut());
            if n.total() > 0 {
                eprintln!(
                    "[physics-smoke] {} junta(s) e {} roldana(s) passaram a apontar por identidade",
                    n.joints, n.wheels
                );
            }
        }

        // O Physics do transporte vem desligado; toda cena daqui existe para mostrar o solver.
        self.timeline.flags.simulate_physics = true;

        // A régua é o instrumento de toda cena (smoke da cena 67: «que régua?»).
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.panel_visibility.insert("timeline", true);
        }

        self.playhead.rewind();
        if ph2d_app_physics::smoke::nasce_parada(which.trim()) {
            self.playhead.pause();
        } else {
            self.playhead.play();
        }
    }
}
