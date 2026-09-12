//! `PH2D_PHYSICS_SMOKE=<n>` — READY-TO-SEE scenes for the global rigid
//! physics (ADR-0131).
//!
//! The scene index (`n` -> wave -> what you see) is the wave table in
//! [`docs/Physics/00_plano_waves.md`], the maintained map. It is NOT copied here:
//! a duplicate only rots — the inline table stopped at 28 while the dispatch grew
//! to 40 — so the `match` below is the source of truth for which `n` runs what,
//! and the plan doc is the source of truth for what each shows.
//!
//! The sprites are plain ECS entities carrying `RigidBody` + `Collider`.
//! **Nothing here touches the rapier world** — the bridge
//! (`render_loop::physics_bridge`) builds the bodies from the components,
//! steps at the `Playhead` tick, and reads the pose back into `Transform`.
//! That is deliberate ([[feedback_ready_to_smoke_example]] + the impasto
//! smoke scar): if the bridge were dead, the sprites would hang in the air
//! instead of falling — the honest failure, not a hidden pre-step.

impl crate::App {
    /// **Corre a cena que o roteador da CRATE escolheu, e aplica o que ela PEDIU.**
    ///
    /// As 118 cenas vivem em [`ph2d_app_physics`] e são funções livres sobre um
    /// [`ph2d_app_physics::SceneCtx`]: elas povoam o mundo e **declaram** o que
    /// querem da shell (enquadrar a câmera, abrir um painel, escolher um objecto,
    /// impor definições, armar o laço). Quem escolhe QUAL é
    /// [`ph2d_app_physics::smoke::scene`]; quem executa os pedidos é esta função.
    ///
    /// ⚠️ **O `ctx` vive num ESCOPO fechado, e isso é load-bearing:** ele empresta
    /// `gfx.sim` mutavelmente, então enquanto existir ninguém toca no resto do
    /// `gfx`. Fechá-lo antes de aplicar o `want` é o que permite escrever na
    /// câmera e no `hero_screen` logo a seguir.
    fn run_physics_scene(&mut self, which: &str) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        // ⚠️ `self.gfx` e `self.timeline` são campos DISJUNTOS da `App`: o
        // empréstimo mutável de ambos no mesmo escopo é o que torna a timeline
        // alcançável de uma cena sem o roteador a ter de correr duas vezes.
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
                // ⚠️ O `clear()` anda COLADO ao `selection`: uma cena que escolhe
                // alguém está a dizer *«é este»*, e deixar a selecção extra de
                // trás entregaria o gizmo a dois donos.
                hero.gizmo.extra_selection.clear();
            }
        }
    }

    /// Frame prologue, once. No-op without the env.
    pub(crate) fn physics_smoke(&mut self) {
        // ⭐⭐ **A env é lida DENTRO da crate** (`ph2d_app_physics::smoke::armed_scene`),
        // e não aqui. É isso que torna verdadeiro o que o `crate::FAMILY` declara ao
        // registo: quem diz responder por `PH2D_PHYSICS_SMOKE` é quem a lê.
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
        // ⭐ **A resolução NOME → IDENTIDADE, uma vez, para TODA cena** (ADR-0164 F1).
        //
        // As cenas autoram uma junta escrevendo *"prende ao Poste"* (`stable_name_id("Post")`)
        // — que é a forma legível e a que um humano quer escrever. A ponte da física, desde
        // esta wave, resolve por `StableId`. Esta chamada é a costura entre as duas.
        //
        // ⚠️ **AQUI e não em cada cena, e a diferença é o modo de falha.** São 35 cenas; uma
        // que esquecesse a chamada guardaria um hash onde a ponte espera uma identidade, a
        // junta **não prenderia, e nada avisaria** — o defeito calado que esta linha inteira
        // existe para tornar impossível. Num roteador, esquecer não é uma opção.
        if let Some(gfx) = self.gfx.as_mut() {
            let n = ph2d_physics_ecs::resolve_body_names(gfx.sim.world_mut());
            if n.total() > 0 {
                eprintln!(
                    "[physics-smoke] {} junta(s) e {} roldana(s) passaram a apontar por identidade",
                    n.joints, n.wheels
                );
            }
        }

        // Play so the bridge steps the world forward — except in the scenes
        // that must sit STILL until the artist has done something. Scene 3
        // waits for a body to be added; scene 7 waits for a Bake, and a bake
        // taken while the clock runs would be a bake of a scene that has
        // already half-fallen. (A `match`, not another `!=`: with two of them
        // the next scene to want a paused clock would have to notice both.)
        // ⚠️ ARM the transport's Physics toggle. It is off by default — Play
        // means "play my animation" until an artist opts in — and every scene
        // here exists to show the solver working, so shipping them disarmed
        // would demo a frozen world and read as "physics is broken". Scene 7
        // then asks the artist to turn it back OFF, which is the point of Bake.
        self.timeline.flags.simulate_physics = true;

        // ⚠️ **E a TIMELINE ABERTA, para TODA cena de física.** O prólogo já é o
        // dono do relógio destas cenas — ele arma o toggle acima, rebobina e
        // decide play/pause —, e o transporte é o instrumento de todas elas: quem
        // toca, quem rebobina e quem faz scrub mora lá.
        //
        // ⚠️ **Isto falhou em produto** (Enio, smoke da cena 67): a mensagem dizia
        // *"rebobine a régua"* e a resposta foi ***"que régua?"***. Medido depois,
        // **17 cenas** mandam usar a régua ou o transporte e nenhuma os mostrava —
        // a minha 66 inclusive. E o modo de falha é pior que uma instrução vaga:
        // o painel de física tem um botão **"Reset to Defaults"** que reseta a
        // gravidade e os sub-passos, então um artista procurando "Reset" sem régua
        // na tela acha o controle errado e conclui que o Reset está quebrado.
        //
        // Uma linha no PRÓLOGO em vez de dezessete nas cenas: a lista por-cena
        // seria a enumeração de que a próxima cena nasce fora, e o `Espaco` já
        // toca sem painel nenhum — o que só a timeline oferece é a RÉGUA.
        if let Some(hero) = self.gfx.as_mut().and_then(|g| g.hero_screen.as_mut()) {
            hero.panel_visibility.insert("timeline", true);
        }

        self.playhead.rewind();
        if ph2d_app_physics::smoke::PAUSED_SCENES.contains(&which.trim()) {
            self.playhead.pause();
        } else {
            self.playhead.play();
        }
    }
}
