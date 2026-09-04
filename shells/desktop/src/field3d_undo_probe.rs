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
//! ⚠️ **Esta sonda não mede um gesto** — ela mede a costura **depois** dele. Ela faz uma edição
//! (uma pose de nó, que é o caso que o dono nomeia: *«principalmente se transformação»*), marca o
//! quadro como tendo entrada, deixa o passo nascer, e depois manda um `Ctrl+Z` **pelo roteamento
//! real do teclado**. O que interessa é o que acontece nos quadros **a seguir** ao restauro: se o
//! documento voltar a diferir do baseline sem ninguém tocar em nada, o próximo quadro com entrada
//! empilha um passo espúrio — e `push_undo` **limpa a pilha de redo**.
//!
//! # Como se lê
//!
//! ```text
//! [probe-undo] f=NN undo=<profundidade> redo=<profundidade>
//! ```
//!
//! - `undo` sobe **uma** vez por edição: certo.
//! - `redo` sobe no `Ctrl+Z` e **volta a zero sozinho**: passo espúrio depois do restauro — é o
//!   report, e a causa está no quadro a seguir ao `apply_project`.
//! - as linhas `[undo] ⛔ o documento MUDOU em [...]` do `PH2D_UNDO_LOG` nomeiam **qual parte**.

use std::cell::Cell;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

use ph2d_ecs::{Entity, With};
use ph2d_field_ecs::FieldNode;

/// O quadro corrente do roteiro — o hook não pode acrescentar campo à `App`.
static FRAME: AtomicU32 = AtomicU32::new(0);

thread_local! {
    /// Onde a seta do gizmo foi agarrada — os quadros de arrasto partem DAQUI.
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
            // ⛔ **O painel MODEL tem de estar ABERTO** — sem ele os chips não são pintados, logo
            // não estão no índice de acerto e um roteiro de cliques mede o vazio. Foi o que a 1.ª
            // corrida devolveu: *«o chip Add[0] NAO esta' no indice de acerto»*.
            5 => crate::field3d_smoke::ask_open_panel(),
            20 => self.probe_select_first_node(),
            // ⭐ **CRIAR** — o primeiro gesto de todo artista, e o que nenhuma sonda desta linha
            // tinha exercido: um clique real no chip *Add*.
            //
            // ⚠️ **Pela porta do PRODUTO, e sem marcar entrada** — o pick da paleta chega num quadro
            // **sem evento nenhum** (o modal consumiu o clique noutro), e quem tem de o declarar é o
            // `mark_authored_change` da W115. Marcar entrada aqui esconderia exactamente o defeito
            // que esta sonda procura.
            30 => {
                eprintln!("[probe-undo] f={f} a PALETA escolheu a forma 0");
                crate::field3d_smoke::ask_shape(0);
            }
            // ⭐ **UM MODIFICADOR** — pelo intent real do chip, que é o que o clique produz.
            40 => {
                eprintln!("[probe-undo] f={f} o chip do modificador 0");
                ph2d_panel_model3d::state::push_intent_for_test(
                    ph2d_panel_model3d::ModelIntent::ToggleMod { slot: 0 },
                );
                self.any_input_this_frame = true;
            }
            50 | 58 => {
                eprintln!("[probe-undo] f={f} Ctrl+Z (down)");
                self.smoke_key_z(false, true);
            }
            51 | 59 => self.smoke_key_z(false, false),
            66 | 74 => {
                eprintln!("[probe-undo] f={f} Ctrl+Shift+Z (down) — o REDO");
                self.smoke_key_z(true, true);
            }
            67 | 75 => self.smoke_key_z(true, false),
            _ => {}
        }
        if (25..=90).contains(&f) {
            let (sel, setas) = self.probe_gizmo_state();
            let (nos, mods) = self.probe_doc_state();
            eprintln!(
                "[probe-undo] f={f} undo={} redo={} nos={nos} mods={mods} sel={sel:?} setas={setas}",
                self.undo.depth(),
                self.undo.redo_depth(),
            );
        }
    }

    /// Quantos nós de modelagem existem, e quantos modificadores há no total — as duas grandezas
    /// que um `Ctrl+Z` sobre *criar* e sobre *pôr um modificador* tem de devolver.
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
}
