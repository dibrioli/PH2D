//! **A cena da RAMIFICAÇÃO MORTA** (`PH2D_AUTOFIX_SMOKE=8`, ADR-0155) — irmã de
//! `motion_autofix_smoke.rs` pelo teto de 600 LOC do shell, e por RESPONSABILIDADE: as cenas
//! `4`/`5`/`6` são sobre um nó que não tem o que precisa; esta é sobre um nó que tem **quase**
//! tudo, e cujo buraco lê um número perfeitamente válido.
//!
//! Um `value.switch` escolhe por índice — `clamp(round(select), 0, N−1)`. Uma porta VAZIA no
//! meio é um índice que existe e devolve **`0.0`**, e `0` é um valor legítimo: nada na tela
//! separa *"esta ramificação está vazia"* de *"ela vale zero"*. Medido no
//! `measure_switch_arity`: com só `in0`/`in1` ligadas, `select = 2` e `select = 3` devolvem
//! `0.000` sem sinal nenhum.
//!
//! ⚠️ **A cena põe o defeito em MOVIMENTO em vez de o descrever**: uma `value.lfo` varre o
//! `select` de 0 a 3 e a fileira sobe em degraus — e **AFUNDA** ao passar pelo índice 1, cuja
//! porta ninguém ligou. O badge diz porquê.
//!
//! ⚠️ **A cauda vazia (`in3`) NÃO é avisada, e é isso que a cena também mostra**: deixar as
//! últimas portas livres é como se escreve um mux estreito. Só o buraco do MEIO é defeito.

use ph2d_nodegraph::graph::{Edge, Pos};

/// As alturas que as DUAS portas ligadas entregam (`in0` e `in2`) — separadas o bastante para o
/// degrau ser óbvio, e as duas longe de zero para o buraco AFUNDAR em vez de se confundir com uma
/// delas.
const LEVELS: [f32; 2] = [1.2, 2.4];

impl crate::App {
    /// O corpo da cena `=8`, delegado de [`crate::App::motion_autofix_smoke`] pelo braço `_`
    /// (o pai já avançou o `FRAME`). Só a combinação `(8, 3)` age.
    pub(super) fn motion_autofix_smoke_dead_branch(&mut self, mode: u32, f: u32) {
        if (mode, f) != (8, 3) {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let (sw, out) = {
            let g = &mut gfx.motion.doc.graph;
            let grid = g.add_node("motion.grid");
            g.set_param(grid, "rows", 1.0);
            g.set_param(grid, "cols", 16.0);
            g.set_param(grid, "gap_x", 0.45);
            g.set_pos(
                grid,
                Pos {
                    x: -460.0,
                    y: -120.0,
                },
            );

            // O `select` varre 0..3 devagar: uma serra de amplitude 1,5 em torno de 1,5.
            let sel = g.add_node("value.lfo");
            g.set_param(sel, "wave", 3.0); // Saw
            g.set_param(sel, "period", 4.0);
            g.set_param(sel, "amplitude", 1.5);
            g.set_param(sel, "offset", 1.5);
            g.set_pos(sel, Pos { x: -220.0, y: 40.0 });

            let sw = g.add_node("value.switch");
            g.set_pos(sw, Pos { x: 40.0, y: -120.0 });
            g.connect(Edge {
                from: (sel, 0),
                to: (sw, 0),
                delayed: false,
            })
            .expect("lfo -> select");

            // `in0` e `in2` ligadas, `in1` VAZIA — o buraco. A `in3` fica livre de
            // propósito: uma cauda vazia é legítima e NÃO leva badge.
            for (k, (port, level)) in [(1u16, LEVELS[0]), (3, LEVELS[1])].into_iter().enumerate() {
                let src = g.add_node("value.pattern");
                g.set_param(src, "steps", 1.0);
                g.set_param(src, "v0", level);
                g.set_pos(
                    src,
                    Pos {
                        x: -220.0,
                        y: -220.0 + k as f32 * 90.0,
                    },
                );
                g.connect(Edge {
                    from: (grid, 0),
                    to: (src, 0),
                    delayed: false,
                })
                .expect("a contagem vem da geometria");
                g.connect(Edge {
                    from: (src, 0),
                    to: (sw, port),
                    delayed: false,
                })
                .expect("uma entrada do switch");
            }

            // O valor escolhido vira ALTURA, para o buraco ser visível e não só marcado.
            let drive = g.add_node("motion.drive");
            g.set_param(drive, "channel", 1.0); // Y
            g.set_pos(
                drive,
                Pos {
                    x: 300.0,
                    y: -120.0,
                },
            );
            let out = g.add_node("motion.output");
            g.set_pos(
                out,
                Pos {
                    x: 540.0,
                    y: -120.0,
                },
            );
            for (from, to) in [
                ((grid, 0), (drive, 0)),
                ((sw, 0), (drive, 1)),
                ((drive, 0), (out, 0)),
            ] {
                g.connect(Edge {
                    from,
                    to,
                    delayed: false,
                })
                .expect("a cadeia da altura");
            }
            (sw, out)
        };
        gfx.motion.sinks.push(out);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
        ph2d_panel_motion_graph::request_graph_selection(vec![sw.0]);

        let dead = ph2d_motion_diagnose::diagnose(&gfx.motion.doc.graph, &gfx.motion.registry)
            .iter()
            .filter(|d| matches!(d.deficit, ph2d_motion_diagnose::Deficit::DeadBranch(_)))
            .count();
        eprintln!(
            "[autofix smoke =8] montei um `value.switch` com `in0`/`in2` ligadas e a `in1` VAZIA, \
             com uma serra a varrer o `select` de 0 a 3: {dead} ramificacao(oes) morta(s) \
             marcada(s) (esperado 1 — a `in1`). SE NAO FOR 1, PARE. ⚠ OLHE A FILEIRA: ela sobe \
             em degraus ({:.1} -> ? -> {:.1}) e AFUNDA a zero no degrau do meio, porque um indice \
             sem porta le' 0.0 — e zero e' um valor legitimo, entao sem o badge nada na tela \
             distingue 'ramificacao vazia' de 'ramificacao vale zero'. CLIQUE o badge do switch: \
             o app so' EXPLICA (toast nomeando a porta `in1`) e SELECIONA — NADA muda no grafo \
             (Offer: o que ligar ali e' escolha sua, e adivinhar seria INVENTAR conteudo). ⚠ A \
             `in3` tambem esta' vazia e NAO leva badge, de proposito: uma cauda livre e' como se \
             escreve um mux estreito; so' o buraco do MEIO e' defeito.",
            LEVELS[0], LEVELS[1]
        );
    }
}

#[cfg(test)]
#[path = "motion_autofix_smoke_dead_branch_tests.rs"]
mod tests;
