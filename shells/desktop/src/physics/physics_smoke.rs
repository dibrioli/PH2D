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

/// **As cenas que abrem PARADAS.** Uma cena que espera um gesto do artista
/// (adicionar um corpo, assar, arrastar um rig) não pode ter meio caído antes de
/// ele chegar ao mouse.
///
/// Uma TABELA e não uma cadeia de `|`: com vinte e poucas entradas o `matches!`
/// gastava uma linha por cena e comia o teto de LOC deste arquivo — e a lista é
/// exatamente o tipo de coisa que só cresce.
/// ⚠️ **Uma cena que pede um GESTO DE ALÇA tem de estar aqui.** As alças de ponto
/// (âncora de joint, centro/aro de roldana) são publicadas **rest-only** — durante
/// o play o overlay desenha a geometria do SOLVER, e elas autoram a AUTORADA —,
/// então uma cena que nasce tocando simplesmente **não tem alça nenhuma**, e o
/// artista relata a feature como quebrada (foi o que aconteceu com a 63).
///
/// ⚠️ **E isto é uma ENUMERAÇÃO escrita à mão**, ou seja exatamente a forma que a
/// próxima cena nasce fora. O gate `handle_scenes_start_paused` varre as mensagens
/// das cenas e exige que quem manda arrastar uma alça esteja nesta lista.
const PAUSED_SCENES: &[&str] = &[
    "3", "7", "14", "15", "16", "17", "21", "22", "23", "24", "37", "38", "39", "40", "41", "43",
    "44", "45", "46", "47", "51", "54", "58", "63", "64", "65", "66", "67", "68", "69", "70", "71",
    "72", "73", "74", "75",
    // ⚠️ A cena 82 (W5) nasce PAUSADA pela razão da 3: ela espera o artista
    // fazer alguma coisa, e um corpo que já caiu meio metro é um corpo cujo
    // gesto de autoria começa no lugar errado.
    "82",
    // ⚠️ A cena 96 (W17) nasce PAUSADA pela razão exata da 95: ela pede uma
    // corrida JOGADA, e com o relógio já a andar o começo da fita descreveria
    // segundos em que ninguém tinha o teclado.
    "96",
    // ⚠️ A cena 95 (W16) nasce PAUSADA pela razão da 7, e mais uma: ela pede ao
    // artista que JOGUE uma corrida, e é essa corrida que vai ser assada. Com o
    // relógio já a andar, o começo da fita descreve segundos em que ninguém
    // tinha o teclado — o bake gravaria um personagem parado antes de gravar o
    // que o artista fez.
    "95",
];

impl crate::App {
    /// **Corre UMA cena da crate e aplica o que ela PEDIU** (W2/L2).
    ///
    /// As 101 cenas que já vivem em [`ph2d_app_physics`] são funções livres sobre
    /// um [`ph2d_app_physics::SceneCtx`]: elas povoam o mundo e **declaram** o que
    /// querem da shell (enquadrar a câmera, abrir um painel, escolher um objecto,
    /// impor definições de mundo). Quem executa esses pedidos é esta função — uma
    /// vez, para todas.
    ///
    /// ⚠️ **O `ctx` vive num ESCOPO fechado, e isso é load-bearing:** ele empresta
    /// `gfx.sim` mutavelmente, então enquanto existir ninguém toca no resto do
    /// `gfx`. Fechá-lo antes de aplicar o `want` é o que permite escrever na
    /// câmera e no `hero_screen` logo a seguir — e é por isso que as cenas que
    /// ainda são método de `App` (as que precisam da timeline ou do playhead)
    /// entram por OUTRO braço do `match`, nunca por dentro deste.
    fn run_physics_scene(&mut self, build: impl FnOnce(&mut ph2d_app_physics::SceneCtx<'_>)) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let mut ctx = ph2d_app_physics::SceneCtx::new(gfx.sim.world_mut(), gfx.physics.settings());
        build(&mut ctx);
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
        let Some(which) = std::env::var("PH2D_PHYSICS_SMOKE").ok() else {
            return;
        };
        if self.physics.smoke_done {
            return;
        }
        if self.gfx.is_none() {
            return; // world not up yet; retry next frame
        }
        self.physics.smoke_done = true;

        match which.trim() {
            "2" => self.physics_smoke_pile(),
            "3" => self.physics_smoke_author(),
            "4" => self.physics_smoke_world(),
            "5" => self.physics_smoke_layers(),
            "6" => self.physics_smoke_joints(),
            "7" => self.physics_smoke_bake(),
            "8" => self.physics_smoke_parented(),
            "9" => self
                .run_physics_scene(ph2d_app_physics::physics_smoke_collider::physics_smoke_scale),
            "10" => self
                .run_physics_scene(ph2d_app_physics::physics_smoke_collider::physics_smoke_sensor),
            "11" => self.physics_smoke_weld(),
            "12" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_props::physics_smoke_gravity)
            }
            "13" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_props::physics_smoke_capsule)
            }
            "14" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_props::physics_smoke_launch)
            }
            "15" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_props::physics_smoke_ccd)
            }
            "16" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_props::physics_smoke_lock_rotation,
            ),
            "17" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_props::physics_smoke_offset)
            }
            "18" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_props::physics_smoke_freeze_position,
            ),
            "19" => self
                .run_physics_scene(ph2d_app_physics::physics_smoke_collision::physics_smoke_mass),
            "20" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_collision::physics_smoke_dominance,
            ),
            "21" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_collision::physics_smoke_material,
            ),
            "22" => self
                .run_physics_scene(ph2d_app_physics::physics_smoke_damping::physics_smoke_damping),
            "23" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_collision::physics_smoke_one_way,
            ),
            "24" => self
                .run_physics_scene(ph2d_app_physics::physics_smoke_collision::physics_smoke_area),
            "25" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_contacts::physics_smoke_contacts,
            ),
            "26" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_contacts::physics_smoke_area_drag,
            ),
            "27" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_contacts::physics_smoke_buoyancy,
            ),
            "28" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_contacts::physics_smoke_form_drag,
            ),
            "29" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_events::physics_smoke_events)
            }
            "30" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_events::physics_smoke_impact_demolition,
            ),
            "31" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_events::physics_smoke_fast_impact,
            ),
            "32" => self
                .run_physics_scene(ph2d_app_physics::physics_smoke_zones::physics_smoke_spin_zone),
            "33" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_zones::physics_smoke_author_spin,
            ),
            "34" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_zones::physics_smoke_force_frame,
            ),
            "35" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_zones::physics_smoke_falloff)
            }
            "36" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_zones::physics_smoke_mirror)
            }
            "37" => self.physics_smoke_bake_range(),
            "38" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_authoring::physics_smoke_joint_anchor,
            ),
            "39" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_joint_bake::physics_smoke_bake_joint,
            ),
            "40" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_authoring::physics_smoke_author_joint,
            ),
            "41" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_authoring::physics_smoke_anchor_follows,
            ),
            "42" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_authoring::physics_smoke_live_tune,
            ),
            "43" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_joint_glyphs::physics_smoke_joint_glyphs,
            ),
            "44" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_joint_handles::physics_smoke_joint_handles,
            ),
            "45" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_joint_pose::physics_smoke_joint_pose,
            ),
            "46" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_joint_draw::physics_smoke_joint_draw,
            ),
            "47" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_joint_slider::physics_smoke_joint_slider,
            ),
            "48" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_joint_motor::physics_smoke_joint_motor,
            ),
            "49" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_joint_break::physics_smoke_joint_break,
            ),
            "50" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_joint_pair::physics_smoke_joint_pair,
            ),
            "51" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_joint_rig::physics_smoke_joint_rig,
            ),
            "52" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_grab::physics_smoke_grab)
            }
            "53" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_interact::physics_smoke_interact,
            ),
            "54" => self.run_physics_scene(ph2d_app_physics::physics_smoke_ik::physics_smoke_ik),
            "55" => self.run_physics_scene(ph2d_app_physics::physics_smoke_fk::physics_smoke_fk),
            "56" => self.run_physics_scene(ph2d_app_physics::physics_smoke_rod::physics_smoke_rod),
            "57" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_wheel::physics_smoke_wheel)
            }
            "58" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_pulley::physics_smoke_pulley)
            }
            "59" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_pulley::physics_smoke_winch)
            }
            "60" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_pulley_break::physics_smoke_break,
            ),
            "61" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_pulley_tackle::physics_smoke_tackle,
            ),
            "62" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_pulley_diff::physics_smoke_differential,
            ),
            "63" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_pulley_comp::physics_smoke_composition,
            ),
            "64" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_pulley_weston::physics_smoke_weston,
            ),
            "65" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_world_pin::physics_smoke_world_pin,
            ),
            "66" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_joint_copy::physics_smoke_joint_copy,
            ),
            "67" => self.run_physics_scene(ph2d_app_physics::physics_smoke_rig::physics_smoke_rig),
            "68" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_soft_weld::physics_smoke_soft_weld,
            ),
            "69" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_compound::physics_smoke_compound,
            ),
            "70" => self.run_physics_scene(ph2d_app_physics::physics_smoke_part::physics_smoke_part),
            "71" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_foot::physics_smoke_foot)
            }
            "72" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_raft::physics_smoke_raft)
            }
            "73" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_signal::physics_smoke_signal)
            }
            "74" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_lead::physics_smoke_lead)
            }
            "75" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_stop::physics_smoke_stop)
            }
            "76" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_signal_leave::physics_smoke_signal_leave,
            ),
            "77" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_rail_rope::physics_smoke_rail_rope,
            ),
            "78" => self.physics_smoke_joint_anim(),
            "79" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_joint_custom::physics_smoke_joint_custom,
            ),
            "80" => self.physics_smoke_float(),
            "81" => self.physics_smoke_walk(),
            "82" => self.physics_smoke_author_player(),
            "83" => self.physics_smoke_jump(),
            "85" => self.physics_smoke_reaction(),
            "86" => self
                .run_physics_scene(ph2d_app_physics::physics_smoke_player_tape::physics_smoke_tape),
            "87" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_player_forgive::physics_smoke_forgive,
            ),
            "88" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_player_slope::physics_smoke_slope,
            ),
            "89" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_player_carry::physics_smoke_chimney,
            ),
            "90" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_player_carry::physics_smoke_wagon,
            ),
            "91" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_player_drop::physics_smoke_pass_through,
            ),
            "92" => self
                .run_physics_scene(ph2d_app_physics::physics_smoke_player_wall::physics_smoke_well),
            "93" => self
                .run_physics_scene(ph2d_app_physics::physics_smoke_player_dash::physics_smoke_dash),
            "94" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_player_crouch::physics_smoke_crouch,
            ),
            "95" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_player_bake::physics_smoke_bake_run,
            ),
            "96" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_player_run::physics_smoke_recorded_run,
            ),
            "97" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_player_drop::physics_smoke_drop_edges,
            ),
            "98" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_player_flank::physics_smoke_flank,
            ),
            "99" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_player_grab::physics_smoke_wall_grab,
            ),
            "100" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_water::physics_smoke_water)
            }
            "101" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_kinematic::physics_smoke_kinematic,
            ),
            "102" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_kin_push::physics_smoke_kin_push,
            ),
            "103" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_kin_pure::physics_smoke_kin_pure,
            ),
            "104" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_kin_water::physics_smoke_kin_water,
            ),
            "105" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_swim::physics_smoke_swim)
            }
            "106" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_zone_force::physics_smoke_zone_force,
            ),
            "107" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_stone::physics_smoke_stone)
            }
            // ⚠️ **O NUMERO E' CONTADO, nunca escolhido** — a `=105` estava
            // tomada (o mergulho), e a nota que a dava como livre tinha
            // envelhecido. Quem pega o proximo LE' este `match`, e o compilador
            // e' o gate: um segundo braco com o mesmo literal e' `unreachable`.
            "108" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_probes::physics_smoke_probes)
            }
            "109" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_foot_fan::physics_smoke_foot_fan,
            ),
            "110" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_multi_jump::physics_smoke_multi_jump,
            ),
            "111" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_ledge::physics_smoke_ledge)
            }
            "112" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_glide::physics_smoke_glide)
            }
            "113" => self.physics_smoke_out(),
            "114" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_brake::physics_smoke_brake)
            }
            "115" => self
                .run_physics_scene(ph2d_app_physics::physics_smoke_surface::physics_smoke_surface),
            // ⚠️ **O NUMERO E' CONTADO, nunca escolhido** — lido deste `match`,
            // que e' a fonte; a nota da §5 do CLAUDE.md dava a `=105` como a
            // proxima livre e tinha envelhecido em onze cenas.
            "116" => self.run_physics_scene(
                ph2d_app_physics::physics_smoke_terminal::physics_smoke_terminal,
            ),
            "117" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_blast::physics_smoke_blast)
            }
            // ⚠️ **O NUMERO E' CONTADO, nunca escolhido** — lido deste `match`,
            // que e' a fonte. O compilador e' o gate: um segundo braco com o
            // mesmo literal e' `unreachable`.
            "118" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_leave::physics_smoke_leave)
            }
            // ⚠️ **O NUMERO E' CONTADO, nunca escolhido** — lido deste `match`.
            "119" => {
                self.run_physics_scene(ph2d_app_physics::physics_smoke_brink::physics_smoke_brink)
            }
            _ => self.physics_smoke_drop(),
        }
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
        if PAUSED_SCENES.contains(&which.trim()) {
            self.playhead.pause();
        } else {
            self.playhead.play();
        }
    }
}
