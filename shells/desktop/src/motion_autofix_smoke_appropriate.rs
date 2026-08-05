//! **A cena da FORMA APROPRIADA** (`PH2D_AUTOFIX_SMOKE=7`, ADR-0155) — separada de
//! `motion_autofix_smoke.rs` no teto de 600 LOC do shell, por RESPONSABILIDADE: o
//! pai HEALA um gesto (insert/reorder/quick-fix) e marca os avisos das famílias
//! derivada/declarada; aqui mora a cena da configuração CORRETA que a foto do Enio
//! montou (`The Shape × Boids -> Oscillator -> Output`), agora SEM o ⚠ falso na
//! fonte-com-estado.
//!
//! O Boids é uma FONTE COM ESTADO: lê o próprio `P` do frame anterior pelo `pre`
//! self-loop e semeia a nuvem sozinho, então NÃO precisa de fonte a montante. Antes
//! da correção o diagnoser o marcava com um `MissingSource("P")` FALSO; a isenção
//! `ph2d_motion_diagnose::seeds_own_state` (o self-loop delayed, sinal que um
//! deformer nunca carrega) o isenta. 48 estrelas nítidas voando em bando e ondulando.

use ph2d_nodegraph::graph::{Edge, Pos};

impl crate::App {
    /// O corpo da cena `=7`, delegado de [`crate::App::motion_autofix_smoke`] pelo
    /// braço `_` (o pai já avançou o `FRAME`). `mode`/`f` vêm resolvidos; só a
    /// combinação `(7, 3)` age.
    pub(super) fn motion_autofix_smoke_appropriate(&mut self, mode: u32, f: u32) {
        if (mode, f) != (7, 3) {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let out = {
            let g = &mut gfx.motion.doc.graph;
            let shape = g.add_node("source.shape");
            let boids = g.add_node("motion.boids");
            let dup = g.add_node("motion.duplicator");
            let osc = g.add_node("motion.oscillator");
            let out = g.add_node("motion.output");
            g.set_param(shape, "kind", 5.0); // Star
            g.set_param(shape, "size", 0.4);
            g.set_param(shape, "sides", 6.0);
            g.set_param(boids, "seed", 3.0);
            // Ondula o Y com uma onda viajante pelas 48 copias (o mesmo Oscillator
            // da foto): um bando VIVO em vez de uma nuvem parada.
            g.set_param(osc, "channel", 1.0); // Y
            g.set_param(osc, "amplitude", 0.6);
            g.set_param(osc, "frequency", 0.5);
            g.set_pos(
                shape,
                Pos {
                    x: -220.0,
                    y: -280.0,
                },
            );
            g.set_pos(
                boids,
                Pos {
                    x: -220.0,
                    y: -60.0,
                },
            );
            g.set_pos(dup, Pos { x: 40.0, y: -200.0 });
            g.set_pos(
                osc,
                Pos {
                    x: 300.0,
                    y: -200.0,
                },
            );
            g.set_pos(
                out,
                Pos {
                    x: 560.0,
                    y: -200.0,
                },
            );
            g.set_label(shape, "The Shape");
            // O `pre` self-loop que o editor auto-plumba num no' com estado.
            g.connect(Edge {
                from: (boids, 0),
                to: (boids, 2),
                delayed: true,
            })
            .expect("boids pre self-loop");
            g.connect(Edge {
                from: (shape, 0),
                to: (dup, 0), // Shape -> duplicator.shape (porta 0)
                delayed: false,
            })
            .expect("shape -> dup.shape");
            g.connect(Edge {
                from: (boids, 0),
                to: (dup, 1), // Boids -> duplicator.points (porta 1)
                delayed: false,
            })
            .expect("boids -> dup.points");
            g.connect(Edge {
                from: (dup, 0),
                to: (osc, 0),
                delayed: false,
            })
            .expect("dup -> oscillator");
            g.connect(Edge {
                from: (osc, 0),
                to: (out, 0),
                delayed: false,
            })
            .expect("oscillator -> output");
            out
        };
        gfx.motion.sinks.push(out);
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
        let warnings =
            ph2d_motion_diagnose::diagnose(&gfx.motion.doc.graph, &gfx.motion.registry).len();
        eprintln!(
            "[autofix smoke =7] montei a FORMA APROPRIADA (a cena da foto): `The Shape \
             (estrela) -> duplicator.shape` + `Boids -> duplicator.points` (com o `pre` \
             self-loop) + `duplicator -> oscillator -> output`. {warnings} aviso(s) do \
             diagnoser (esperado 0). SE NAO FOR 0, PARE. O Boids e' uma FONTE COM ESTADO: \
             le o proprio P do frame anterior pelo self-loop e semeia a nuvem sozinho, \
             entao NAO precisa de fonte a montante — antes desta correcao ele ganhava um \
             ⚠ MissingSource(P) FALSO (le P, sem aresta nao-delayed entrando); agora \
             `seeds_own_state` (o self-loop delayed, sinal que um deformer nunca carrega) \
             o isenta. Na tela: 48 estrelas NITIDAS voando em bando e ondulando no Y, com \
             ZERO badge. (Enriquecimento opcional: ligar um `value.lfo` em \
             target_x/target_y do Boids conduz o bando inteiro pelo canvas — e tambem \
             silenciaria o aviso, mas a correcao acima ja o faz sem tocar no grafo.)"
        );
    }
}
