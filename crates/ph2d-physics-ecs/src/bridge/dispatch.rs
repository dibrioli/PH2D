//! **As PORTAS por onde se pede um tique** — módulo irmão pelo cap de 700 LOC,
//! cortado por responsabilidade.
//!
//! O pai responde *o que a ponte É* (os campos, as queries em cache, o
//! nascimento e o `rebuild`); este responde *como se pede a ela que ande*. São
//! quatro portas, e as três primeiras **delegam** à quarta — nenhuma tem laço
//! próprio, que é o que impede um chamador de receber um mundo diferente por ter
//! escolhido a porta mais curta:
//!
//! - [`PhysicsBridge::dispatch`] — sem cena, sem fita (a maioria dos gates).
//! - [`PhysicsBridge::dispatch_with_scene`] — com a cena (o shell antes da W7).
//! - [`PhysicsBridge::dispatch_with_tape`] — com a fita (o C9 e os gates dela).
//! - [`PhysicsBridge::dispatch_with_scene_and_tape`] — o laço, e o único.

use ph2d_ecs::SimWorld;

use super::{FrozenScene, HeldInput, PhysicsBridge, PlayerInputAtTick, SceneAtTick};

impl PhysicsBridge {
    /// The per-frame entry point (mirrors `motion_bridge::dispatch`).
    /// `playing` / `target` come from the shell's `Playhead`
    /// (`target = round(playhead.time() / fixed_dt)`).
    ///
    /// - **Playing:** step the owed ticks (`target - last_stepped`,
    ///   sequential) then read poses back into `Transform`.
    /// - **Not playing:** settle bodies to the authored `Transform` (no
    ///   step) — the artist is posing. Scrub-*back* re-sim is W1.5.
    pub fn dispatch(&mut self, sim: &mut SimWorld, playing: bool, target: u64) {
        self.dispatch_with_scene(sim, playing, target, &mut FrozenScene);
    }

    /// Bind the queries and bring the rapier world's *structure* level with the
    /// components — the prologue every entry point shares.
    ///
    /// Bodies BEFORE joints, always: a joint binds body handles and derives its
    /// local anchors from where those bodies stand, so it cannot be built before
    /// them (`bridge::joints`).
    pub(super) fn prepare(&mut self, sim: &mut SimWorld) {
        if self.query.is_none() {
            self.query = Some(sim.world_mut().query());
        }
        if self.joint_query.is_none() {
            self.joint_query = Some(sim.world_mut().query());
        }
        if self.wheel_query.is_none() {
            self.wheel_query = Some(sim.world_mut().query());
        }
        if self.part_query.is_none() {
            // `query_filtered`, não `query`: o `Without<RigidBody>` É a definição
            // de uma peça, e ele mora no TIPO.
            self.part_query = Some(sim.world_mut().query_filtered());
        }
        self.reconcile_structure(sim);
        self.reconcile_parts(sim);
        self.reconcile_joints(sim);
        self.restamp_damping();
        // A static body has ONE author — the authored `Transform` — so it tracks
        // it on every dispatch, not only on the paused ones. Before this, a wall
        // dragged with the clock running moved the drawing and left the collider
        // behind (`bridge::hold::settle_static`). Runs BEFORE the step, so the
        // tick is solved against the wall the artist can see; and it is a no-op
        // for every body nobody touched, so a settled scene is unaffected.
        self.settle_static(sim);
    }

    /// [`PhysicsBridge::dispatch`], told where the scene's scene-driven bodies
    /// are at each tick it runs (see [`SceneAtTick`]). The plain `dispatch` is
    /// this with nothing to consult; there is ONE implementation so the two
    /// cannot drift.
    pub fn dispatch_with_scene(
        &mut self,
        sim: &mut SimWorld,
        playing: bool,
        target: u64,
        scene: &mut dyn SceneAtTick,
    ) {
        self.dispatch_with_scene_and_tape(sim, playing, target, scene, &mut HeldInput);
    }

    /// [`PhysicsBridge::dispatch`], e mais a **FITA** — o atalho para quem tem
    /// fita e não tem curvas (os gates e o C9).
    pub fn dispatch_with_tape(
        &mut self,
        sim: &mut SimWorld,
        playing: bool,
        target: u64,
        tape: &mut dyn PlayerInputAtTick,
    ) {
        self.dispatch_with_scene_and_tape(sim, playing, target, &mut FrozenScene, tape);
    }

    /// [`PhysicsBridge::dispatch_with_scene`], e mais a **FITA** de entrada do
    /// player (W7, ver `bridge::tape`).
    ///
    /// ⚠️ Há UMA implementação e as outras delegam, pelo motivo que o
    /// `dispatch_with_scene` já dá: duas não podem divergir se só existe uma.
    pub fn dispatch_with_scene_and_tape(
        &mut self,
        sim: &mut SimWorld,
        playing: bool,
        target: u64,
        scene: &mut dyn SceneAtTick,
        tape: &mut dyn PlayerInputAtTick,
    ) {
        self.prepare(sim);
        // Transitions are a fresh list per dispatch; the forward loop appends to it,
        // tick by tick. (Flashes are NOT cleared here — they outlive their tick.)
        self.contact_events.clear();
        self.joint_breaks.clear();
        match target.cmp(&self.last_stepped) {
            // The clock went BACKWARDS — Reset, or a scrub. rapier has no
            // rewind, so replay from the rest state (see `rewind_to`).
            std::cmp::Ordering::Less => self.rewind_to(sim, target, scene, tape),
            // Forward: advance the owed ticks, sequentially. This is play,
            // and it is equally a scrub FORWARD while paused — the sim state
            // is a function of the tick, not of the play button.
            std::cmp::Ordering::Greater => {
                // A kinematic body's pose is an INPUT, and the scene supplies
                // it once per FRAME while a frame may owe several ticks. So the
                // move is spread across them — the same slicing the wrapper
                // does across sub-steps, one level up, through the same law
                // (`PhysicsWorld::slice_pose`).
                //
                // ⚠️ Neither half of that is optional, and both were wrong once.
                // Aiming once per DISPATCH feeds the first step and leaves the
                // rest with none (an aim is spent by the step it is for), so
                // the body crosses the whole span in one tick at N× speed:
                // measured on a platform carrying a box, stepping to tick 60
                // one tick at a time leaves the cargo riding at x = 1.049,
                // while asking for tick 60 in ONE dispatch FLINGS it to
                // x = -0.520 and off the edge. Aiming at the same TARGET every
                // tick is no better — the aim is absolute, so the body arrives
                // on the first step and then has zero velocity for the rest.
                let owed = target - self.last_stepped;
                self.capture_kinematic_start();
                // The handle→entity map for the per-tick event diff. Built ONCE here:
                // the forward branch never respawns bodies (only a rewind does), so the
                // handles are stable across the loop.
                let by_handle = self.handle_map();
                let mut tick = self.last_stepped;
                for i in 0..owed {
                    // Ask the scene for THIS tick first. When it answers, the
                    // pose is exact and there is nothing to interpolate; when it
                    // does not, spread the frame's move across the ticks owed.
                    let exact = scene.put(sim, self.last_stepped + i + 1);
                    let f = if exact {
                        1.0
                    } else {
                        (i + 1) as f32 / owed as f32
                    };
                    // E os PARÂMETROS que o documento dá a este tick — a outra
                    // metade da mesma frase que a linha acima diz das poses
                    // (`bridge::joint_drive`). Sem ela um `motor_target`
                    // keyframado chega uma vez por QUADRO, com o valor do quadro
                    // aplicado a todos os ticks devidos.
                    self.drive_joint_params(sim, false);
                    self.drive_kinematic(sim, f);
                    // E o DEDO daquele tick (W7). Antes da perna, porque é ela
                    // que o consome.
                    self.take_taped_input(sim, tape, self.last_stepped + i + 1);
                    // E os PLAYERS (W2): o sensor pergunta ao BVH que o step
                    // ANTERIOR deixou e a mola escreve o motor deste tick. Por
                    // TICK, como tudo aqui — uma perna que agisse por FRAME
                    // seguraria o personagem mais alto em máquina rápida.
                    self.drive_players(sim);
                    self.world.step();
                    self.steps_taken += 1;
                    // Diff this tick's touching union against the standing set — the
                    // only place the clock stepped through the transitions, and the one
                    // that catches a touch shorter than a whole tick (W-TickContacts).
                    self.accumulate_contact_events(&by_handle);
                    // And the joints that parted during that same tick (W-J7) —
                    // the wrapper clears its own list every `step`, so a break in
                    // an early tick of a multi-tick dispatch is gone by the last.
                    self.accumulate_joint_breaks();
                    self.accumulate_pulley_breaks();
                    // And the high-water mark of the RUN (the tuning signal): the
                    // wrapper's peak is per-TICK and a yank is over before it can
                    // be read.
                    self.accumulate_joint_peaks();
                    self.accumulate_pulley_peaks();
                    // E o GIRO das roldanas, que é o que torna uma roda uma roda
                    // na tela (`bridge::rope`). Por TICK, como tudo aqui: o
                    // ângulo é a integral de uma taxa, e integrá-lo por FRAME
                    // deixaria a roda girar mais rápido em máquina rápida.
                    self.spin_rope_wheels();
                    tick += 1;
                    // Asking is free; capturing costs about one step, which
                    // is why the ring is sparse (see `PhysicsCheckpointRing`).
                    //
                    // ⚠️ **Nada é gravado enquanto um CUTUCÃO está em voo** (a
                    // mão do W-Grab ou o campo de atração do W-Hand; regra 1 de
                    // `bridge::grab`): um checkpoint tirado sob o
                    // cutucão descreve uma corrida que nenhum replay reproduz,
                    // então semear um scrub com ele faria a resposta para um tick
                    // depender de o cache tê-lo ou não. O `grab` já limpou o que
                    // havia; isto impede que a janela se re-encha.
                    if !self.is_poking() && self.ring.should_record(tick) {
                        self.ring.record(tick, self.world.checkpoint());
                        // E a memória do controlador do MESMO tique — ver
                        // `bridge::tape`.
                        self.record_jump_states(tick);
                    }
                }
                self.readback(sim);
                self.last_stepped = target;
            }
            // The clock is standing still. While PAUSED, let bodies follow a
            // Transform the artist moved (authoring). While PLAYING, do
            // nothing: a frame faster than the tick must not touch the world
            // — `settle` would zero the velocity and the fall would stutter.
            std::cmp::Ordering::Equal => {
                if !playing {
                    self.settle(sim);
                }
            }
        }
        // W-Pulley W3: e os eixos das roldanas MONTADAS onde os corpos delas
        // ficaram — **incondicional, e é essa a correção**.
        //
        // ⚠️ A arena é reinstalada por `prepare` a cada dispatch com o centro
        // derivado da pose de REPOUSO (o único que a colheita do ECS conhece), e
        // até aqui só o laço de sub-passos a refrescava. Um quadro mais rápido
        // que o tique não dá passo nenhum ⇒ ele publicava a roldana **onde ela
        // foi autorada**: medido em **1,27 m** de salto num bloco que viaja, com
        // a simulação correta o tempo todo. Era o tremor do smoke da talha, e o
        // fato de ser só-desenho é exatamente o que se espera de uma lista que o
        // solver refresca e o pintor lê.
        //
        // Aqui, e não junto da instalação: este é o único ponto por onde as
        // QUATRO saídas passam (replay, laço de tiques, `settle` pausado, e o
        // quadro que não deve tique nenhum), então a arena publicada descreve
        // onde as roldanas ESTÃO sem ninguém ter de enumerar os ramos. Nos que
        // dão passo é idempotente com o que o `step` já fez.
        self.world.refresh_mounted_wheels();
        // Read the sensor overlaps of the world in its final state for this
        // frame, whichever branch produced it (`bridge::triggers`). No-op (and
        // no alloc) when the scene has no sensors.
        self.rebuild_triggers();
        // And the SOLID half of the same question (`bridge::contacts`): who is
        // actually touching whom, where, and under how much load — the STANDING set,
        // read from the same final world state as the triggers, for the same reason.
        // The transitions and flashes are the forward loop's job above; this only
        // publishes what touches AT the tick the artist is now looking at, which is a
        // question even a scrub must answer. No-op (and no alloc) when nothing touches.
        self.rebuild_standing_contacts();
        // Sync each joint's DISPLAY pivot (its `Transform.translation`) to where
        // its authored body-local anchor now sits — so the anchor dot and the
        // Inspector Position follow the body when a body moves. Rest-only: during
        // play the dot is not shown and the overlay draws the live solver anchors
        // (`joint_anchors`), so a per-frame write there would be a stale display
        // value for no reader. `!playing` also covers a scrub that lands paused.
        if !playing {
            self.sync_joint_pivots(sim);
            // W-Pulley W3: e o mesmo para o EIXO de uma roldana montada, pela
            // razão idêntica — o dot de centro e a §2 Position leem o
            // `Transform`, então mover o BLOCO tem de levar a roldana junto. Em
            // play a arena já carrega o centro vivo (`refresh_mounts`) e é dela
            // que o desenho lê, então escrever aqui seria trabalho sem leitor.
            self.sync_mounted_wheels(sim);
        }
    }
}
