//! **Os métodos do despacho: picks e arrastos** — `impl App` que o `on_mouse_input` e o `on_cursor_moved` chamam
//! ([`super`]): o poke da explosão, os picks modais da física e do osso, os arrastos do vetor, e o editor de áudio.
//! Mudados VERBATIM (`line/input-dispatch`, 2026-09-13); os que eram privados passam a `pub(super)` — a única
//! troca, e é o preço de o `impl` morar num módulo irmão.

use super::*;

impl crate::App {
    /// **O press das ferramentas de PONTO** (W-Hand: explosão e atração).
    ///
    /// `true` = consumiu o gesto. A decisão inteira (relógio andando · física
    /// armada · a ferramenta é de ponto) mora em `body_grab::poke_at`, que é
    /// testável sem janela; aqui fica só a projeção tela→mundo e a marca que o
    /// overlay desenha.
    ///
    /// ⚠️ **Recusa quando não há mundo sob o cursor** (`vec_world_at` = `None`,
    /// que é o caso fora do canvas): estourar num ponto que não existe é o gesto
    /// caindo em silêncio, e sem esta linha ele seria consumido de qualquer jeito.
    pub(super) fn poke_press(&mut self, sx: f32, sy: f32) -> bool {
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let window = gfx.surface.size();
        let world = gfx.camera.screen_to_world((sx, sy), window);
        let playing = self.playhead.is_playing();
        let simulating = self.timeline.flags.simulate_physics;
        let Some(hit) = ph2d_app_physics::body_grab::poke_at(
            &mut gfx.physics,
            &gfx.sim,
            &self.physics.interaction,
            world,
            playing,
            simulating,
        ) else {
            return false;
        };
        // A metade VISÍVEL. A explosão é instantânea, então a marca é o único
        // vestígio dela que não é "corpos que se moveram"; a atração é sustentada
        // e o overlay lê o campo VIVO da ponte (`attract_marks`), sem cópia aqui.
        if self.physics.interaction.tool == ph2d_physics_ecs::InteractionTool::Explode {
            let radius = self.physics.interaction.clamped().blast_radius;
            self.blast_flash = Some((
                world,
                radius,
                ph2d_app_physics::body_grab::BLAST_FLASH_TICKS,
            ));
            if hit > 0
                && let Some(gfx) = self.gfx.as_mut()
            {
                gfx.toasts.push(ph2d_editor_core::Toast::info(format!(
                    "Blast: {hit} bodies"
                )));
            }
        }
        true
    }

    /// **O clique do eyedropper de corpo do joint** (§12): com um pick armado,
    /// resolve o CORPO sob o cursor e religa aquela ponta do joint. Clique no
    /// vazio (ou num não-corpo) desiste; clicar o corpo que já está na outra
    /// ponta é RECUSADO e mantém o pick armado (um self-joint fica dormente,
    /// `set_joint_body`). Consome o press (o guard já filtrou por
    /// `joint_body_pick.is_some()`), então nunca cai no picking/gizmo.
    pub(super) fn joint_body_pick_click(&mut self, sx: f32, sy: f32) {
        let Some((joint, slot_b)) = self.joint_body_pick else {
            return;
        };
        self.any_input_this_frame = true;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let window_size = gfx.surface.size();
        let world_pos = gfx.camera.screen_to_world((sx, sy), window_size);
        // O sprite mais ao topo sob o cursor que é um CORPO físico e não é a
        // própria entidade-joint.
        let target = ph2d_render::pick_sprites_at_world(gfx.present.world_mut(), world_pos)
            .into_iter()
            .find(|&bits| {
                bits != joint
                    && gfx
                        .sim
                        .world()
                        .get::<ph2d_physics_ecs::RigidBody>(ph2d_ecs::Entity::from_bits(bits))
                        .is_some()
            })
            .map(ph2d_ecs::Entity::from_bits);
        match target {
            Some(t) => {
                if ph2d_app_physics::joint::set_joint_body(&mut gfx.sim, joint, slot_b, t) {
                    self.joint_body_pick = None; // religado — pronto
                }
                // senão: self-joint recusado, segue armado para outro clique
            }
            None => self.joint_body_pick = None, // vazio / não-corpo = desiste
        }
    }

    /// ⭐⭐⭐ **O clique que escolhe o ALVO de um osso inteligente.** Consome o press — é isso que
    /// impede a ferramenta Bone de criar um osso por baixo do gesto (report do dono, 2026-09-08).
    ///
    /// ⚠️ **Ele procura DUAS coisas, e a ordem é a do desenho:** primeiro uma FORMA vectorial (é o
    /// que o artista aponta num editor de vector, e o que a cena de smoke tem), depois uma SPRITE.
    /// Um alvo pode ser qualquer objecto que a timeline anime, e as duas famílias respondem a
    /// *«o que está debaixo do cursor»* de maneiras diferentes.
    ///
    /// ⛔ **Clique no vazio NÃO desiste** — ao contrário dos eyedroppers de física, e a razão é o
    /// alvo: falhar a forma por três píxeis é comum, e um pick que se perde nisso faz o artista
    /// repetir o botão sem saber porquê. Quem desiste é o `Escape`.
    pub(super) fn smart_pick_click(&mut self, sx: f32, sy: f32) {
        let Some(bits_osso) = self.skeleton.smart_pick else {
            return;
        };
        self.any_input_this_frame = true;
        let hit_r = 10.0 * self.vec_px_to_world(); // LITERAL-PX-OK: o MESMO raio do irmão `vec_path_pick_click`
        let Some(world) = self.vec_world_at((sx, sy)) else {
            return;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let por_forma = self
            .vec
            .pen
            .path_at(&gfx.vec_scene, world, hit_r)
            .and_then(|pid| self.vec.entities.get(&pid).copied());
        let alvo = por_forma.or_else(|| {
            // ⚠️ O mundo do documento é `f64` e o do render `f32` — a conversão vive aqui, na porta
            // entre os dois, e não numa das pontas.
            #[expect(
                clippy::cast_possible_truncation,
                reason = "o picking de sprite fala f32; o documento vectorial fala f64"
            )]
            let p = [world[0] as f32, world[1] as f32];
            ph2d_render::pick_sprites_at_world(gfx.present.world_mut(), p)
                .into_iter()
                .find(|&bits| bits != bits_osso)
        });
        let Some(alvo) = alvo else {
            return; // clique no vazio — segue armado
        };
        if alvo != bits_osso
            && crate::skeleton_smart::set_target(
                &mut gfx.sim,
                ph2d_ecs::Entity::from_bits(bits_osso),
                ph2d_ecs::Entity::from_bits(alvo),
            )
        {
            self.skeleton.smart_pick = None;
        }
    }

    /// **O clique do eyedropper de MONTAGEM** (§13, W3): com um pick armado,
    /// resolve o CORPO sob o cursor e monta o eixo daquela roldana nele. Clique
    /// no vazio (ou num não-corpo) desiste. Consome o press, então nunca cai no
    /// picking/gizmo.
    pub(super) fn wheel_mount_pick_click(&mut self, sx: f32, sy: f32) {
        let Some(wheel) = self.wheel_body_pick else {
            return;
        };
        self.any_input_this_frame = true;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let window_size = gfx.surface.size();
        let world_pos = gfx.camera.screen_to_world((sx, sy), window_size);
        // O sprite mais ao topo sob o cursor que é um CORPO físico e não é a
        // própria roldana — montar uma roldana nela mesma não descreve nada.
        let target = ph2d_render::pick_sprites_at_world(gfx.present.world_mut(), world_pos)
            .into_iter()
            .find(|&bits| {
                bits != wheel
                    && gfx
                        .sim
                        .world()
                        .get::<ph2d_physics_ecs::RigidBody>(ph2d_ecs::Entity::from_bits(bits))
                        .is_some()
            })
            .map(ph2d_ecs::Entity::from_bits);
        if let Some(t) = target {
            ph2d_app_physics::joint_wheel::set_wheel_mount(&mut gfx.sim, wheel, t);
        }
        self.wheel_body_pick = None;
    }

    /// **O clique do eyedropper de CORDA** (§13, W1): com um pick armado, resolve a
    /// corda cuja ROTA passa sob o cursor e religa aquela roldana a ela.
    ///
    /// ⚠️ **A tolerância é a MESMA `SNAP_PX` do ímã de âncora**, convertida em
    /// mundo pelo zoom vigente — um app onde dois alvos de canvas respondem a
    /// distâncias diferentes é um app que se aprende duas vezes. Medido: 14 px valem
    /// 0,052 m a `height_world` 4 e 0,207 m a 16.
    ///
    /// Clique longe de toda corda desiste; um alvo que não é polia é RECUSADO e o
    /// pick segue armado (`set_wheel_rope`).
    pub(super) fn wheel_rope_pick_click(&mut self, sx: f32, sy: f32) {
        let Some(wheel) = self.wheel_rope_pick else {
            return;
        };
        self.any_input_this_frame = true;
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let window_size = gfx.surface.size();
        let world_pos = gfx.camera.screen_to_world((sx, sy), window_size);
        let tol = ph2d_app_physics::joint_anchor_drag::SNAP_PX * gfx.camera.height_world
            / window_size.height as f32;
        match gfx.physics.rope_at_world(world_pos, tol) {
            Some(rope) => {
                if ph2d_app_physics::joint_wheel::set_wheel_rope(&mut gfx.sim, wheel, rope) {
                    self.wheel_rope_pick = None;
                }
                // senão: o alvo não é uma polia, segue armado para outro clique
            }
            None => self.wheel_rope_pick = None, // longe de toda corda = desiste
        }
    }

    /// Um quadro de POSE de osso — no-op sem osso agarrado.
    pub(super) fn vec_bone_pose_move(&mut self) -> bool {
        let Some((bits, parte)) = self.skeleton.bone_pose else {
            return false;
        };
        let Some(world) = self.vec_world_at(self.last_pointer) else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        crate::bone_pose::pose(
            &mut gfx.sim,
            ph2d_ecs::Entity::from_bits(bits),
            world,
            parte,
        )
    }

    pub(super) fn vec_envelope_corner_move(&mut self, x: f32, y: f32) -> bool {
        let Some(active) = self.vec.envelope_drag else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let w = gfx.camera.screen_to_world((x, y), win);
        crate::envelope_gesture::drag(
            &mut gfx.sim,
            Some(active),
            [f64::from(w[0]), f64::from(w[1])],
        )
    }

    pub(super) fn vec_pen_drag_move(&mut self, x: f32, y: f32) -> bool {
        if !self.vector_tool_active() || !self.vec.pen.is_dragging() {
            return false;
        }
        let cfg = self.vec_snap_cfg(self.vec_px_to_world());
        // `take` evita emprestar `self` duas vezes: a closure fica com os alvos e as
        // guias, `self.vec.pen`/`self.gfx` seguem livres. Devolvidos logo abaixo.
        let targets = std::mem::take(&mut self.vec.snap_targets);
        let mut guides = Vec::new();
        let Some(gfx) = self.gfx.as_mut() else {
            self.vec.snap_targets = targets;
            return false;
        };
        let win = gfx.surface.size();
        let w = gfx.camera.screen_to_world((x, y), win);
        // `hero_screen` e `vec_scene` são campos IRMÃOS de `AppGfx`: a grade pode ser
        // consultada enquanto o Pen muta a cena.
        let mut hero = gfx.hero_screen.as_mut();
        let mut snap = |p: [f64; 2]| {
            let mut grid = |q: [f64; 2]| {
                let h = hero.as_mut()?;
                crate::vec_snap::ask_grid(&mut h.grid.snap_state, q)
            };
            let r = ph2d_vec_edit::snap::snap(&[p], &targets, cfg, Some(&mut grid));
            guides = crate::vec_snap::guides_of(&r);
            r.apply(p)
        };
        let consumed =
            self.vec
                .pen
                .on_drag(&mut gfx.vec_scene, [w[0] as f64, w[1] as f64], &mut snap);
        self.vec.snap_targets = targets;
        self.vec.snap_guides = guides;
        consumed
    }

    /// Gradient group: hit-test the selected path's gradient handles (screen `pos`)
    /// → the handle within ~9 px (world-scaled): a multi-point point, or a
    /// linear/radial endpoint. `None` unless the Vector tool is active and the
    /// selected path has a gradient fill.
    pub(super) fn vec_grad_hit(&self, pos: (f32, f32)) -> Option<ph2d_vec_render::GradHandle> {
        if !self.vector_tool_active() {
            return None;
        }
        let gfx = self.gfx.as_ref()?;
        let sel = self.vec.pen.selected()?;
        let path = gfx.vec_scene.paths().iter().find(|p| p.id == sel)?;
        let win = gfx.surface.size();
        let w = gfx.camera.screen_to_world(pos, win);
        let (wx, wy) = (w[0] as f64, w[1] as f64);
        let w0 = gfx.camera.screen_to_world((0.0, 0.0), win);
        let w1 = gfx.camera.screen_to_world((1.0, 0.0), win);
        let px = (((w1[0] - w0[0]).powi(2) + (w1[1] - w0[1]).powi(2)).sqrt()) as f64;
        // ADR-0111: a geometria do gradiente é LOCAL, como a do path. O cursor desce
        // pelo afim, e o raio de captura com ele (a forma pode estar escalada).
        let x = ph2d_vec_entities::transform::xform_of_transform(
            ph2d_vec_entities::transform::world_transform(
                &gfx.sim,
                ph2d_ecs::Entity::from_bits(*self.vec.entities.get(&sel)?),
            ),
        );
        let inv = x.inverse()?;
        let l = inv.apply([wx, wy]);
        ph2d_vec_render::hit_gradient_handle(path, l[0], l[1], 9.0 * px / x.mean_scale())
    }

    /// Gradient group: while a gradient handle is grabbed, move it to the cursor's
    /// world position (a radial edge sets the radius). No-op unless a grad drag is
    /// live. Reuses the pure `drag_gradient_handle` geometry helper.
    pub(super) fn vec_grad_drag_move(&mut self, x: f32, y: f32) -> bool {
        let Some(handle) = self.vec.grad_drag else {
            return false;
        };
        let Some(sel) = self.vec.pen.selected() else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let win = gfx.surface.size();
        let w = gfx.camera.screen_to_world((x, y), win);
        // O ponto do gradiente é guardado no espaço local do path (ADR-0111).
        let w = match self.vec.entities.get(&sel).and_then(|&b| {
            ph2d_vec_entities::transform::xform_of_transform(
                ph2d_vec_entities::transform::world_transform(
                    &gfx.sim,
                    ph2d_ecs::Entity::from_bits(b),
                ),
            )
            .inverse()
        }) {
            Some(inv) => {
                let l = inv.apply([f64::from(w[0]), f64::from(w[1])]);
                [l[0] as f32, l[1] as f32]
            }
            None => w,
        };
        if let Some(path) = gfx.vec_scene.path_mut(sel) {
            return ph2d_vec_render::drag_gradient_handle(path, handle, w[0] as f64, w[1] as f64);
        }
        false
    }

    /// Motion Nodes M1: is the cursor over the docked graph panel? Drives the
    /// cursor-gated graph keyboard focus + middle-pan routing (Blender-style F
    /// acts on the hovered area, graph vs scene).
    pub(crate) fn cursor_over_motion_graph(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| {
                h.store
                    .panel_rect(ph2d_editor_core::ids::MOTION_GRAPH_PANEL)
            })
            .is_some_and(|r| r.contains(self.last_pointer.0, self.last_pointer.1))
    }

    /// W2.E6: is the cursor over the general timeline dock? Mirrors
    /// [`Self::cursor_over_motion_graph`] — a middle-drag there pans the
    /// dope-sheet (via its `TimelineSurface` gesture), not the camera behind it.
    /// Blender-style: the hovered component owns the zoom/pan.
    pub(crate) fn cursor_over_timeline(&self) -> bool {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.store.panel_rect(ph2d_editor_core::ids::TIMELINE_PANEL))
            .is_some_and(|r| r.contains(self.last_pointer.0, self.last_pointer.1))
    }
    /// ADR-0108 Fase 1: while a shape drag is live, resize it to the cursor.
    /// No-op unless the Vector tool is active AND a shape gesture is in progress.
    /// A ferramenta de forma não faz hit-test, então o canto é encaixado direto.
    pub(super) fn vec_shape_drag_move(&mut self, x: f32, y: f32) -> bool {
        if !self.vector_tool_active() || !self.vec.shape.is_active() {
            return false;
        }
        let Some(w) = self
            .gfx
            .as_ref()
            .map(|gfx| gfx.camera.screen_to_world((x, y), gfx.surface.size()))
        else {
            return false;
        };
        let cfg = self.vec_snap_cfg(self.vec_px_to_world());
        let p = self.vec_snap_point([f64::from(w[0]), f64::from(w[1])], cfg);
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        self.vec
            .shape
            .on_drag(&mut gfx.vec_scene, p, shape_constraint(self.modifiers))
    }

    /// **O LÁPIS, enquanto a mão anda**: a amostra entra (se andou o passo mínimo) e a curva é
    /// re-ajustada AO VIVO. No-op sem gesto — a mesma disciplina de early-return do pen.
    ///
    /// ⚠️ **Sem snap, de propósito.** A caneta encaixa porque cada clique é uma DECISÃO; encaixar
    /// cada amostra de um arrasto contínuo quantizaria a mão inteira numa grade, que é o oposto
    /// de desenhar à mão livre. (O snap a caminho/interseção é a W6 do plano, e a pergunta lá é
    /// sobre as PONTAS.)
    /// **O move do Width Tool**: a alça agarrada segue o cursor — a distância à curva vira o
    /// multiplicador e a projeção nela vira a posição. Mesma disciplina de early-return do lápis;
    /// no-op sem alça agarrada.
    pub(super) fn vec_width_drag_move(&mut self, x: f32, y: f32) -> bool {
        let Some(grab) = self.vec.width_grab else {
            return false;
        };
        let Some(w) = self
            .gfx
            .as_ref()
            .map(|gfx| gfx.camera.screen_to_world((x, y), gfx.surface.size()))
        else {
            return false;
        };
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        let scene = &gfx.vec_scene;
        crate::width_handles::drag(
            &mut gfx.sim,
            scene,
            &self.vec.entities,
            grab,
            [f64::from(w[0]), f64::from(w[1])],
        );
        // O dedo MOVEU: a parada deixa de ser "nascida num clique" e o release não a desfaz.
        self.vec.width_grab = Some(ph2d_app_vec::width_grab::Grab {
            created: false,
            ..grab
        });
        // Consome o move: o gesto É do Width enquanto a alça está agarrada, e deixar cair viraria
        // pan da câmera no meio do arrasto.
        true
    }

    pub(super) fn vec_pencil_drag_move(&mut self, x: f32, y: f32) -> bool {
        if !self.vector_tool_active() || !self.vec.pencil.is_active() {
            return false;
        }
        // **O ESTABILIZADOR corre aqui, em px de TELA, antes da conversão para mundo** — o tremor
        // é um fato da mão sobre a mesa, e é em px que ele tem tamanho. Com o slider no mínimo o
        // `lazy_mouse_step` devolve o ponteiro cru, ao bit.
        let (x, y) = self
            .vec
            .pencil_hand
            .filter((x, y), self.vec.draw_config.pencil_stabilizer);
        let Some(w) = self
            .gfx
            .as_ref()
            .map(|gfx| gfx.camera.screen_to_world((x, y), gfx.surface.size()))
        else {
            return false;
        };
        let dyn_in = self.pointer_dynamics();
        let Some(gfx) = self.gfx.as_mut() else {
            return false;
        };
        self.vec.pencil.on_drag(
            &mut gfx.vec_scene,
            [f64::from(w[0]), f64::from(w[1])],
            dyn_in,
        );
        // Consome o move mesmo quando a amostra foi recusada pelo passo mínimo: o gesto É do
        // lápis enquanto ele está vivo, e deixar cair viraria pan da câmera no meio do traço.
        true
    }

    /// The clip frame under `(x, y)` if it's inside the overlay waveform — for
    /// starting a selection drag. `None` if the overlay is hidden or the point is
    /// outside the waveform area.
    #[cfg(feature = "panel-audio-editor")]
    pub(super) fn audio_wave_frame_at(&self, x: f32, y: f32) -> Option<(u64, f32)> {
        let view = ph2d_app_audio::wave_view()?;
        self.audio.as_ref()?.editor_clip()?;
        let r = view.rect;
        (x >= r.x && x <= r.x + r.w && y >= r.y && y <= r.y + r.h)
            .then(|| (ph2d_app_audio::frame_at_x(&view, x), freq_at_y(&view, y)))
    }

    /// Extend the active selection to the cursor. Returns `true` if a selection drag is
    /// live (the caller early-returns so it doesn't also pan).
    ///
    /// In the **spectrogram** the gesture sets a box — a time range AND a frequency band,
    /// which is the only thing spectral repair can act on (W5). In the **waveform** there is
    /// no frequency axis, so the same gesture sets a time range and explicitly CLEARS the
    /// band.
    ///
    /// Clearing it matters. The doc used to promise the waveform "sets a time range and
    /// nothing else" while the code wrote the band on every drag, using the y of a gesture
    /// the user made along x — so a horizontal drag in the waveform overwrote a carefully
    /// drawn box with a degenerate one, and Repair stayed lit and did nothing visible
    /// (audit 2026-07-12).
    #[cfg(feature = "panel-audio-editor")]
    pub(super) fn audio_sel_drag_move(&mut self, x: f32, y: f32) -> bool {
        let Some((anchor, anchor_hz)) = self.audio_sel_drag else {
            return false;
        };
        let spectral = ph2d_panel_audio_editor::spectral_state::view();
        if let Some(view) = ph2d_app_audio::wave_view() {
            let cur = ph2d_app_audio::frame_at_x(&view, x);
            let cur_hz = freq_at_y(&view, y);
            if let Some(a) = self.audio.as_mut() {
                a.editor_set_selection(anchor, cur);
                if spectral {
                    a.editor_set_spectral_band(anchor_hz, cur_hz);
                } else {
                    a.editor_clear_spectral_band();
                }
            }
        }
        true
    }

    /// Update the piece drag (Move / Scale) to the cursor. Returns `true` while one is live, so
    /// the caller early-returns and the drag does not also pan the camera.
    ///
    /// Nothing is committed here — the overlay draws an outline and the release does the edit
    /// (`audio/editor/pieces.rs`). A per-frame WSOLA stretch of a three-minute piece is not a
    /// thing to attempt sixty times a second.
    #[cfg(feature = "panel-audio-editor")]
    pub(super) fn audio_piece_drag_move(&mut self, x: f32) -> bool {
        let Some(view) = ph2d_app_audio::wave_view() else {
            return false;
        };
        let frame = ph2d_app_audio::frame_at_x(&view, x) as usize;
        self.audio
            .as_mut()
            .is_some_and(|a| a.editor_piece_drag_to(frame))
    }

    /// The clip frame under `(x, y)` if it's inside the overlay's time RULER — for
    /// starting a playhead scrub (seek). The ruler is the strip below the waveform;
    /// dragging it scrubs, while the wave body above it makes a selection.
    #[cfg(feature = "panel-audio-editor")]
    pub(super) fn audio_ruler_frame_at(&self, x: f32, y: f32) -> Option<u64> {
        let view = ph2d_app_audio::wave_view()?;
        self.audio.as_ref()?.editor_clip()?;
        let r = view.ruler;
        (x >= r.x && x <= r.x + r.w && y >= r.y && y <= r.y + r.h)
            .then(|| ph2d_app_audio::frame_at_x(&view, x))
    }

    /// Seek the preview to the cursor `x` while a scrub is live. Returns `true` if
    /// scrubbing (the caller early-returns so the drag doesn't also pan).
    #[cfg(feature = "panel-audio-editor")]
    pub(super) fn audio_scrub_move(&mut self, x: f32) -> bool {
        if !self.audio_scrub_drag {
            return false;
        }
        if let Some(view) = ph2d_app_audio::wave_view() {
            let frame = ph2d_app_audio::frame_at_x(&view, x);
            if let Some(a) = self.audio.as_mut() {
                a.editor_scrub_to_frame(frame);
            }
        }
        true
    }
}
