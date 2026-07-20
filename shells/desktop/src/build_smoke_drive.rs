//! **O DRIVER dos smokes do Build** — módulo irmão de [`crate::build_smoke`] (HR-18).
//!
//! Os helpers que dirigem o app pelo caminho REAL do input (cursor → botão → hit-index →
//! chrome → bus): um roteiro de smoke que injetasse estado por baixo estaria testando outra
//! coisa. A CENA de cada nível mora no `build_smoke.rs`; aqui mora só a MÃO.

impl crate::App {
    /// Um clique no ponto de MUNDO `w`, pelo caminho do winit: cursor → botão → botão.
    pub(crate) fn smoke_click(&mut self, w: [f64; 2]) {
        let Some(gfx) = self.gfx.as_ref() else { return };
        let win = gfx.surface.size();
        let s = gfx.camera.world_to_screen([w[0] as f32, w[1] as f32], win);
        self.on_cursor_moved(winit::dpi::PhysicalPosition::new(
            f64::from(s.0),
            f64::from(s.1),
        ));
        self.on_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        );
        self.on_mouse_input(
            winit::event::ElementState::Released,
            winit::event::MouseButton::Left,
        );
    }

    /// Ctrl+Z (ou Ctrl+Shift+Z) pelo roteamento REAL do teclado — **inclusive o RELEASE**.
    ///
    /// O release não é enfeite: é um evento de input como qualquer outro, e o
    /// `post_frame_undo` faz a varredura de diff **em todo frame com input**. Enquanto o
    /// harness só mandava o `Pressed`, o frame do release nunca existia — e era exatamente
    /// nele que o passo espúrio nascia.
    pub(crate) fn smoke_key_z(&mut self, redo: bool, down: bool) {
        use winit::keyboard::ModifiersState;
        self.modifiers = if redo {
            ModifiersState::CONTROL | ModifiersState::SHIFT
        } else {
            ModifiersState::CONTROL
        };
        self.key_input(
            winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyZ),
            if down {
                winit::event::ElementState::Pressed
            } else {
                winit::event::ElementState::Released
            },
            false,
            None,
        );
        if !down {
            self.modifiers = ModifiersState::empty();
        }
    }

    pub(crate) fn smoke_undo(&mut self, redo: bool) {
        self.smoke_key_z(redo, true);
        self.smoke_key_z(redo, false);
    }

    /// Acha um chip do rail no hit-index e o clica **com o ponteiro** (é o caminho do
    /// usuário: pixel → hit → widget → chrome → bus → shell).
    pub(crate) fn smoke_rail_click(&mut self, id: ph2d_editor::NodeId, label: &str) {
        let Some(hero) = self.gfx.as_ref().and_then(|g| g.hero_screen.as_ref()) else {
            return;
        };
        let mut found = None;
        'scan: for y in (0..1000).step_by(4) {
            for x in (0..120).step_by(4) {
                if hero.hit_index.hit(x as f32, y as f32) == Some(id) {
                    found = Some((x as f32, y as f32));
                    break 'scan;
                }
            }
        }
        let Some((x, y)) = found else {
            eprintln!("[build-smoke] o chip {label} NÃO está no hit-index (não é clicável!)");
            return;
        };
        eprintln!("[build-smoke] clicando no botão {label} em ({x}, {y})");
        self.on_cursor_moved(winit::dpi::PhysicalPosition::new(
            f64::from(x),
            f64::from(y),
        ));
        self.on_mouse_input(
            winit::event::ElementState::Pressed,
            winit::event::MouseButton::Left,
        );
        self.on_mouse_input(
            winit::event::ElementState::Released,
            winit::event::MouseButton::Left,
        );
    }

    pub(crate) fn smoke_state(&mut self, when: &str) {
        let n = self.gfx.as_ref().map_or(0, |g| g.vec_scene.paths().len());
        eprintln!(
            "[build-smoke] {when}: {n} path(s) · undo={} redo={}",
            self.undo.depth(),
            usize::from(self.undo.can_redo()),
        );
        // A ORDEM da cena e o `RootOrder` de cada forma — é o que o restore embaralha.
        if let Some(gfx) = self.gfx.as_ref() {
            let ids: Vec<u64> = gfx.vec_scene.paths().iter().map(|p| p.id).collect();
            let ro: Vec<(u64, String)> = ids
                .iter()
                .map(|id| {
                    let r = self
                        .vec_entities
                        .get(id)
                        .and_then(|&b| {
                            gfx.sim
                                .world()
                                .get::<ph2d_ecs::RootOrder>(ph2d_ecs::Entity::from_bits(b))
                        })
                        .map(|r| {
                            if r.0 == u32::MAX {
                                "MAX".to_string()
                            } else {
                                r.0.to_string()
                            }
                        })
                        .unwrap_or_else(|| "—".to_string());
                    (*id, r)
                })
                .collect();
            eprintln!("[build-smoke]    ordem da cena + RootOrder: {ro:?}");
        }
    }
}
