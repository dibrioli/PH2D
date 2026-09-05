//! ⭐⭐⭐ **A SONDA DO UNDO DO MODELADOR** (`PH2D_FIELD_UNDO_PROBE=1`) — o aparelho que faltava ao
//! report *«o undo/redo está completamente destruído»* (Enio, 2026-09-04, a **quarta** vez).
//!
//! # Por que ela existe
//!
//! Três jornadas curaram três defeitos reais do registo de passos e **nenhuma reproduziu** o que o
//! dono vê. A razão é a mesma que o [`crate::fx_undo_smoke`] já tinha escrito para a pilha de
//! efeitos: entre o gesto e o passo há uma máquina — `any_input_this_frame`, `held_button`, o
//! `apply_project` e o quadro **seguinte** a ele — que nenhum gate de estado atravessa, e que um
//! `PH2D_UNDO_LOG` só descreve se alguém estiver com a mão no rato.
//!
//! ⚠️ **Ela abre o MODEL pelo PILL** (sem `PH2D_FIELD_SMOKE`), que é o caminho do dono, e conduz os
//! quatro gestos que ele faz — **criar** pela paleta, **arrastar a seta** do gizmo, **arrastar um
//! slider** do painel e **digitar um número** com Enter —, e depois manda `Ctrl+Z` quatro vezes e
//! `Ctrl+Shift+Z` duas, tudo pelo roteamento real do `winit`.
//!
//! # Como se lê
//!
//! ```text
//! [probe-undo] f=NN undo=<n> redo=<n> nos=<n> mods=<n> x=<pose> y=<pose> sel=<bits> setas=<n> row0=<track>
//! ```
//!
//! - `undo` sobe **uma** vez por gesto: certo. Dois gestos e um só passo é *«pula etapas»*.
//! - Cada `Ctrl+Z` devolve **um** gesto (D, C, B, A por essa ordem): `y` volta, `x` volta, a
//!   forma some. Um `Ctrl+Z` que não muda nada visível é um passo espúrio.
//! - `setas = 0` com `nos` a subir é a mão perdida: a peça está lá e o gizmo não.
//! - `row0` é o que o **painel mostra** para a linha 0: se ele discorda de `x` depois de um
//!   `Ctrl+Z`, o painel está a mentir — e se `x` volta a mudar sozinho a seguir, ele está a
//!   **escrever** por cima do restauro.

use std::cell::Cell;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

use ph2d_ecs::{Entity, With};
use ph2d_field_ecs::{FieldNode, FieldPose};

/// O quadro corrente do roteiro — o hook não pode acrescentar campo à `App`.
static FRAME: AtomicU32 = AtomicU32::new(0);

thread_local! {
    /// Onde a seta do gizmo (ou o slider) foi agarrado — os quadros de arrasto partem DAQUI.
    static GRAB: Cell<(f32, f32)> = const { Cell::new((0.0, 0.0)) };
}

fn on() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FIELD_UNDO_PROBE").is_some())
}

impl crate::App {
    /// Roda no prólogo do quadro, ao lado das outras sondas. No-op sem a env.
    pub(crate) fn field3d_undo_probe(&mut self) {
        if !on() || self.gfx.is_none() {
            return;
        }
        let f = FRAME.fetch_add(1, Ordering::Relaxed);
        match f {
            // ⭐ **O PILL**, e não a variável de ambiente — é o mesmo `insert(PANEL_ID, true)`.
            5 => {
                eprintln!("[probe-undo] f={f} abro o MODEL pelo pill");
                crate::field3d_smoke::ask_open_panel();
            }
            25 => self.probe_select_first_node(),
            // A — criar pela paleta (pedido servido num quadro sem evento; W115).
            30 => {
                eprintln!("[probe-undo] f={f} A: a PALETA escolheu a forma 0");
                crate::field3d_smoke::ask_shape(0);
            }
            // B — arrastar a seta do gizmo, pelo ponteiro real.
            40 => self.probe_grab_an_arrow(f),
            41..=46 => {
                let (x, y) = GRAB.with(Cell::get);
                #[allow(clippy::cast_precision_loss)]
                self.smoke_pointer_move(x + (f - 40) as f32 * 9.0, y);
            }
            47 => {
                eprintln!("[probe-undo] f={f} B: UP da seta");
                self.smoke_pointer_up();
            }
            // C — arrastar o slider da linha 0 do painel (Position X), pelo ponteiro real.
            55 => {
                self.probe_grab_widget(f, ph2d_editor::ids::model3d_radius_slider(0), "slider[0]")
            }
            56..=60 => {
                let (x, y) = GRAB.with(Cell::get);
                #[allow(clippy::cast_precision_loss)]
                self.smoke_pointer_move(x + (f - 55) as f32 * 8.0, y);
            }
            61 => {
                eprintln!("[probe-undo] f={f} C: UP do slider");
                self.smoke_pointer_up();
            }
            // D — clicar no campo numérico da linha 1 (Position Y), digitar e Enter.
            70 => self.probe_grab_widget(f, ph2d_editor::ids::model3d_radius_chip(1), "chip[1]"),
            71 => self.smoke_pointer_up(),
            72 => {
                eprintln!("[probe-undo] f={f} D: digito 0.5 no chip[1]");
                self.probe_type_digits("0.5");
            }
            73 => self.smoke_key_enter(),
            // Quatro Ctrl+Z, pelo teclado real (press num quadro, release no seguinte).
            80 | 86 | 92 | 98 => {
                eprintln!("[probe-undo] f={f} Ctrl+Z (down)");
                self.smoke_key_z(false, true);
            }
            81 | 87 | 93 | 99 => self.smoke_key_z(false, false),
            106 | 112 => {
                eprintln!("[probe-undo] f={f} Ctrl+Shift+Z (down) — o REDO");
                self.smoke_key_z(true, true);
            }
            107 | 113 => self.smoke_key_z(true, false),
            _ => {}
        }
        if (24..=120).contains(&f) {
            let (sel, setas) = self.probe_gizmo_state();
            let (nos, mods) = self.probe_doc_state();
            let (x, y) = self.probe_selected_pose();
            let row0 = self.probe_row_track(0);
            let modal = self.command_palette_open();
            let (tool, panel) = self.gfx.as_ref().map_or((String::new(), false), |g| {
                (
                    g.tools
                        .active()
                        .map_or(String::new(), |t| format!("{:?}", t.id())),
                    g.hero_screen
                        .as_ref()
                        .is_some_and(|h| h.is_panel_visible(ph2d_panel_model3d::PANEL_ID)),
                )
            });
            let armado = crate::field3d_smoke::with_smoke(|_| ()).is_some();
            eprintln!(
                "[probe-undo] f={f} undo={} redo={} nos={nos} mods={mods} x={x:.3} y={y:.3} \
                 sel={sel:?} setas={setas} row0={row0:.3} modal={modal} tool={tool} \
                 painel={panel} armado={armado} held={:?}",
                self.undo.depth(),
                self.undo.redo_depth(),
                self.held_button,
            );
        }
    }

    /// Escolhe o primeiro nó do modelador — o gizmo só existe com alvo.
    fn probe_select_first_node(&mut self) {
        let Some(gfx) = self.gfx.as_mut() else {
            return;
        };
        let alvo: Option<Entity> = {
            let mut q = gfx
                .sim
                .world_mut()
                .query_filtered::<Entity, With<FieldNode>>();
            q.iter(gfx.sim.world()).next()
        };
        match (alvo, gfx.hero_screen.as_mut()) {
            (Some(e), Some(hero)) => {
                hero.gizmo.clear_all_selection();
                hero.gizmo.add_to_selection(e.to_bits());
                eprintln!("[probe-undo] escolhi {e:?}");
            }
            _ => eprintln!("[probe-undo] ⚠️ sem nó de modelagem ou sem hero — roteiro morto"),
        }
    }

    /// Agarra a primeira seta viva do gizmo, no ponto do meio da haste.
    fn probe_grab_an_arrow(&mut self, f: u32) {
        let ponto = crate::field3d_smoke::with_smoke(|s| {
            let area = s.vp().area?;
            crate::field3d_input::handles(s)
                .into_iter()
                .find_map(|h| match (h.live, &h.shape) {
                    (true, crate::field3d_gizmo::Shape::Arrow { from, to }) => Some((
                        area.x + from[0] + (to[0] - from[0]) * 0.7,
                        area.y + from[1] + (to[1] - from[1]) * 0.7,
                    )),
                    _ => None,
                })
        })
        .flatten();
        match ponto {
            Some((x, y)) => {
                GRAB.with(|c| c.set((x, y)));
                eprintln!("[probe-undo] f={f} B: DOWN na seta do gizmo em ({x:.0}, {y:.0})");
                self.smoke_pointer_down(x, y);
            }
            None => eprintln!(
                "[probe-undo] f={f} ⛔ NENHUMA seta viva no gizmo — o artista nao tem gesto"
            ),
        }
    }

    /// Agarra um widget do painel pelo índice de acerto — o mesmo caminho do dedo.
    fn probe_grab_widget(&mut self, f: u32, id: ph2d_editor::NodeId, nome: &str) {
        match self.smoke_find_widget(id) {
            Some((x, y)) => {
                GRAB.with(|c| c.set((x, y)));
                eprintln!("[probe-undo] f={f} DOWN no {nome} em ({x:.0}, {y:.0})");
                self.smoke_pointer_down(x, y);
            }
            None => eprintln!("[probe-undo] f={f} ⛔ o {nome} NAO esta' no indice de acerto"),
        }
    }

    /// Digita dígitos e ponto pelo `key_input` inteiro — o `smoke_type` só sabe letras.
    fn probe_type_digits(&mut self, text: &str) {
        use winit::keyboard::KeyCode as K;
        for ch in text.chars() {
            let code = match ch {
                '0' => K::Digit0,
                '1' => K::Digit1,
                '2' => K::Digit2,
                '3' => K::Digit3,
                '4' => K::Digit4,
                '5' => K::Digit5,
                '6' => K::Digit6,
                '7' => K::Digit7,
                '8' => K::Digit8,
                '9' => K::Digit9,
                '.' => K::Period,
                '-' => K::Minus,
                _ => continue,
            };
            for st in [
                winit::event::ElementState::Pressed,
                winit::event::ElementState::Released,
            ] {
                self.key_input(
                    winit::keyboard::PhysicalKey::Code(code),
                    st,
                    false,
                    (st == winit::event::ElementState::Pressed)
                        .then(|| winit::keyboard::SmolStr::new(ch.to_string())),
                );
            }
        }
    }

    /// O que o artista tem na mão: a selecção do gizmo e quantas setas vivas ele oferece.
    ///
    /// ⚠️ **`setas = 0` depois de um `Ctrl+Z` é o report inteiro**: a peça voltou e o gizmo não —
    /// e daí em diante nenhum gesto de transformação existe.
    fn probe_gizmo_state(&self) -> (Option<u64>, usize) {
        let sel = self
            .gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.gizmo.selection);
        let setas = crate::field3d_smoke::with_smoke(|s| {
            crate::field3d_input::handles(s)
                .iter()
                .filter(|h| h.live && matches!(h.shape, crate::field3d_gizmo::Shape::Arrow { .. }))
                .count()
        })
        .unwrap_or(0);
        (sel, setas)
    }

    /// Quantos nós de modelagem existem, e quantos modificadores há no total.
    fn probe_doc_state(&mut self) -> (usize, usize) {
        let Some(gfx) = self.gfx.as_mut() else {
            return (0, 0);
        };
        let nos: Vec<Entity> = {
            let mut q = gfx
                .sim
                .world_mut()
                .query_filtered::<Entity, With<FieldNode>>();
            q.iter(gfx.sim.world()).collect()
        };
        let mods = nos
            .iter()
            .filter_map(|&e| gfx.sim.world().get::<ph2d_field_ecs::FieldMods>(e))
            .map(|m| m.stack.len())
            .sum();
        (nos.len(), mods)
    }

    /// A pose (x, y) do nó ESCOLHIDO — a grandeza que os gestos B, C e D escrevem.
    fn probe_selected_pose(&self) -> (f32, f32) {
        let Some(gfx) = self.gfx.as_ref() else {
            return (f32::NAN, f32::NAN);
        };
        gfx.hero_screen
            .as_ref()
            .and_then(|h| h.gizmo.selection)
            .map(Entity::from_bits)
            .and_then(|e| gfx.sim.world().get::<FieldPose>(e))
            .map_or((f32::NAN, f32::NAN), |p| {
                (p.xform.translation[0], p.xform.translation[1])
            })
    }

    /// O que o PAINEL mostra na trilha do slider da linha `n` (0..1).
    fn probe_row_track(&self, n: u32) -> f32 {
        self.gfx
            .as_ref()
            .and_then(|g| g.hero_screen.as_ref())
            .and_then(|h| h.store.slider(ph2d_editor::ids::model3d_radius_slider(n)))
            .map_or(f32::NAN, |(_, v)| v)
    }
}
