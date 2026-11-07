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
        if let Some((a, b)) = want.playhead_loop {
            self.playhead.set_loop(a, b);
        }
        self.physics.player_readout_log =
            want.player_readout_log.or(self.physics.player_readout_log);
        let hero = gfx.hero_screen.as_mut();
        ph2d_app_physics::smoke::aplica_o_pedido(&want, &mut gfx.camera, &mut gfx.physics, hero);
        ph2d_app_physics::smoke::resolve_os_nomes(gfx.sim.world_mut());
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

        // O Physics do transporte vem desligado; toda cena daqui existe para mostrar o solver.
        self.timeline.flags.simulate_physics = true;

        self.playhead.rewind();
        if ph2d_app_physics::smoke::nasce_parada(which.trim()) {
            self.playhead.pause();
        } else {
            self.playhead.play();
        }
    }
}
