//! ⭐ MI-8 «Send signal» de ponta a ponta (CPU): um envelope que cai abaixo do nível uma vez manda
//! UM sinal, com o nome autorado e o objecto Motion dono do nó como origem (`quem()`), pelo mesmo
//! caminho do quadro: a bomba marcha a tomada, `collect_signals` decide a borda, `from_motion` publica.

use ph2d_app_motion::motion_bridge::signals::{collect_signals, signal_nodes};
use ph2d_app_motion::motion_objeto::{Dono, IDENTIDADE};
use ph2d_app_motion::motion_state::MotionState;
use ph2d_node_send_signal as send;
use ph2d_nodegraph::cook::TimeScopes;
use ph2d_nodegraph::graph::{Edge, NodeId};
use ph2d_runtime::{EntityBits, Signal, SignalOutbox, SignalReader};

const FPS: f64 = 60.0;
const SID: u64 = 77;
const BITS: u64 = 4242;

/// `grid 1×1 → beat → adsr → Send signal (Falls below 0.5) → drive → output`; o nó é do objecto `SID`.
fn grafo(motion: &mut MotionState) -> NodeId {
    let g = &mut motion.doc.graph;
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 1.0);
    g.set_param(grid, "cols", 1.0);
    let beat = g.add_node("pulse.beat");
    g.set_param(beat, "period", 30.0);
    let env = g.add_node("pulse.adsr");
    for (k, v) in [
        ("attack", 0.05),
        ("decay", 0.1),
        ("sustain", 0.8),
        ("hold", 0.2),
        ("release", 0.3),
    ] {
        g.set_param(env, k, v);
    }
    let s = g.add_node(send::MANIFEST.name);
    g.set_text_param(s, send::NAME_KEY, "caiu");
    g.set_param(s, send::WHEN, send::WHEN_FALLS as f32);
    g.set_param(s, send::LEVEL, 0.5);
    let drive = g.add_node("motion.drive");
    let out = g.add_node("motion.output");
    let fios = [
        (grid, 0, beat, 0),
        (beat, 0, env, 0),
        (env, 0, s, 0),
        (grid, 0, drive, 0),
        (s, 0, drive, 1),
        (drive, 0, out, 0),
    ];
    for (a, ap, b, bp) in fios {
        g.connect(Edge {
            from: (a, ap),
            to: (b, bp),
            delayed: false,
        })
        .expect("fio");
    }
    for n in [beat, env] {
        g.connect(Edge {
            from: (n, 0),
            to: (n, 1),
            delayed: true,
        })
        .expect("pre");
    }
    g.validate(&motion.registry).expect("grafo válido");
    motion.doc.set_owner(s, SID);
    let dono = Dono {
        entity: BITS,
        visible: true,
        afim: IDENTIDADE,
    };
    motion.objeto.donos.insert(SID, dono);
    motion.sinks = vec![out];
    s
}

/// Por qual porta a bomba marcha (as duas do `dispatch`): a da CPU (sinks) ou a HÍBRIDA (o prefixo
/// de CPU até às fronteiras do plano da placa).
#[derive(Clone, Copy)]
enum Rota {
    Cpu,
    Hibrida,
}

/// O relógio do quadro, pela lei REAL (`Playhead::crosses_forward`, a pergunta do shell antes de
/// `collect_signals`).
#[derive(Clone, Copy)]
enum Relogio {
    Toca,
    Parado,
    Salta,
}

/// `segundos` de jogo pela `rota`, com a lei do `relogio`. Devolve o que a outbox recebeu.
fn corre(segundos: f64, rota: Rota, relogio: Relogio) -> Vec<Signal> {
    let mut motion = MotionState::new();
    grafo(&mut motion);
    motion.signal_taps = signal_nodes(&motion.doc.graph);
    motion.pump.set_taps(&motion.signal_taps);
    motion.pump.clear_tap_fires();
    let plano = ph2d_gpu_cook::plan(
        &motion.doc.graph,
        &motion.registry,
        &motion.registry,
        motion.sinks[0],
    );
    let fronteiras: Vec<NodeId> = plano.boundaries.iter().map(|(n, _)| *n).collect();
    let mut playhead = ph2d_core::Playhead::new(1.0 / FPS);
    match relogio {
        Relogio::Parado => playhead.pause(),
        Relogio::Toca | Relogio::Salta => playhead.play(),
    }
    let scopes = TimeScopes::new();
    let mut outbox = SignalOutbox::new();
    for tick in 0..=((segundos * FPS) as u64) {
        let (g, r) = (&motion.doc.graph, &motion.registry);
        match rota {
            Rota::Cpu => motion.pump.advance_or_scrub_scoped(
                g,
                r,
                &motion.sinks,
                tick,
                |t| t as f64 / FPS,
                motion.default_uv_rect,
                motion.default_size,
                &scopes,
            ),
            Rota::Hibrida => motion.pump.advance_or_scrub_to_nodes_scoped(
                g,
                r,
                &fronteiras,
                tick,
                |t| t as f64 / FPS,
                &scopes,
            ),
        };
        if playhead.crosses_forward(matches!(relogio, Relogio::Salta)) {
            collect_signals(&mut motion);
        }
        motion.pump.clear_tap_fires();
        for sig in motion.signals_out.drain(..) {
            outbox.publish(Signal::from_motion(
                &sig.name, sig.source, sig.tick, sig.rows,
            ));
        }
    }
    let mut leitor = SignalReader::new();
    outbox.read(&mut leitor).cloned().collect()
}

/// ⭐ Uma queda abaixo do nível = UM sinal, «caiu», com o objecto Motion como origem — pelas DUAS
/// portas de marcha; relógio parado e salto no play (a lei real) = nada.
#[test]
fn send_signal_falls_below_once_and_speaks_for_the_motion_object() {
    for rota in [Rota::Cpu, Rota::Hibrida] {
        let sinais = corre(2.0, rota, Relogio::Toca);
        assert_eq!(sinais.len(), 1, "uma queda = um sinal: {sinais:?}");
        assert_eq!(&*sinais[0].name, "caiu");
        assert_eq!(
            sinais[0].origin.quem(),
            Some(EntityBits(BITS)),
            "a origem é o objecto Motion"
        );
        assert!(
            corre(2.0, rota, Relogio::Parado).is_empty(),
            "relógio parado: nada"
        );
        assert!(
            corre(2.0, rota, Relogio::Salta).is_empty(),
            "um salto no play: nada"
        );
    }
}
