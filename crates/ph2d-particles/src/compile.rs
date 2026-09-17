//! **O COMPILADOR** — um `ParticleEmitter` vira um grafo de nós REAIS do Motion (doc 14 §3.3).
//!
//! ```text
//!   [World] value.table(x) ─┐ value.table(y) ─┐          (a trajectória do objecto)
//!                            ▼                 ▼
//!   motion.emitter ──► motion.integrate ──► [drive(Size) ← map_range ← attribute(Life)]
//!                        ▲        │  ⇣ pre      ──► tint(cor) ──► [drive(Falloff) ← attribute(Life)
//!                        └─ drag ◄─ wind ◄┘                        ──► tint(cor final)] ──► output
//! ```
//!
//! ⚠️ **O integrador está SEMPRE lá**, mesmo sem forças: o emissor só dá a posição de nascimento e
//! a velocidade de boca — quem desloca a partícula é o integrador (`P = rest.P + sim_d`). Sem
//! forças o laço é o dele próprio (`out ⇢ forces`), exactamente como o editor o liga.
//!
//! ⚠️ **O tamanho ao longo da vida vem ANTES de a máscara ser a fracção de vida**: o `motion.drive`
//! é mascarado pelo `falloff` como toda modificadora, e com a máscara já a ser a vida o tamanho
//! mudaria duas vezes.

use ph2d_ecs::{ParticleEmitter, ParticleSpace};
use ph2d_node_motion_emitter::schedule::SCHEDULE_KEY;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

use crate::plan::{Emission, emit_params};

/// O ficheiro da TRAJECTÓRIA do objecto, dentro do cozinheiro de cada emissor (os externos são
/// por cozinheiro, então um nome só serve a todos). ⚠️ Com o `$` à frente: fica fora do selector
/// de ficheiros do editor, pela cerca do `table_external_key`.
pub const TRACK_FILE: &str = "$particles/track";

/// As colunas da trajectória.
pub const TRACK_T: &str = "t";
/// A posição `x` de mundo do objecto.
pub const TRACK_X: &str = "x";
/// A posição `y` de mundo do objecto.
pub const TRACK_Y: &str = "y";

/// `channel` do `motion.drive`: o tamanho.
const DRIVE_SIZE: f32 = 3.0;
/// `channel` do `motion.drive`: a máscara.
const DRIVE_FALLOFF: f32 = 5.0;
/// `mode` do `motion.drive`: multiplicar.
const DRIVE_MULTIPLY: f32 = 2.0;
/// `mode` do `motion.drive`: pôr.
const DRIVE_SET: f32 = 1.0;
/// `mode` do `value.attribute`: a fracção de vida.
const ATTR_LIFE_FRACTION: f32 = ph2d_node_value_attribute::MODE_LIFE_FRACTION as f32;
/// `emitter_motion` do emissor: a partícula fica onde nasceu.
const MOTION_LEAVE: f32 = 1.0;

/// O grafo de um emissor, e os nós que a corrida volta a tocar.
pub struct Compiled {
    /// O grafo.
    pub graph: Graph,
    /// O `motion.emitter`.
    pub emitter: NodeId,
    /// O `motion.output`.
    pub sink: NodeId,
}

fn wire(g: &mut Graph, from: NodeId, to: NodeId, port: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, 0),
        to: (to, port),
        delayed,
    })
    .expect("o compilador liga só portas que existem");
}

/// **Compila** — o grafo do emissor, com a emissão `em` já posta.
#[must_use]
pub fn compile(cfg: &ParticleEmitter, em: &Emission) -> Compiled {
    let mut g = Graph::new();
    let emitter = g.add_node("motion.emitter");
    set_emitter(&mut g, emitter, cfg);
    apply_emission(&mut g, emitter, cfg, em);
    if cfg.space == ParticleSpace::World {
        g.set_param(emitter, ph2d_node_motion_emitter::MOTION, MOTION_LEAVE);
        for (param, col) in [("x", TRACK_X), ("y", TRACK_Y)] {
            let table = g.add_node("value.table");
            g.set_text_param(table, ph2d_node_value_table::FILE_KEY, TRACK_FILE);
            g.set_text_param(table, ph2d_node_value_table::TIME_KEY, TRACK_T);
            g.set_text_param(table, ph2d_node_value_table::VALUE_KEY, col);
            g.drive_param(emitter, param, (table, 0))
                .expect("a tabela conduz o emissor");
        }
    }

    // O integrador e o laço de forças.
    let integrate = g.add_node("motion.integrate");
    wire(&mut g, emitter, integrate, 0, false);
    let mut loop_tail = integrate;
    let mut delayed = true;
    let gravity = libm::hypotf(cfg.gravity[0], cfg.gravity[1]);
    if gravity > 0.0 {
        let wind = g.add_node("force.wind");
        let deg = libm::atan2f(cfg.gravity[1], cfg.gravity[0]).to_degrees();
        g.set_param(wind, "angle", deg.rem_euclid(360.0));
        g.set_param(wind, "strength", gravity);
        g.set_param(wind, "gust", 0.0);
        wire(&mut g, loop_tail, wind, 0, delayed);
        loop_tail = wind;
        delayed = false;
    }
    if cfg.damping > 0.0 {
        let drag = g.add_node("force.drag");
        g.set_param(drag, "coefficient", cfg.damping);
        wire(&mut g, loop_tail, drag, 0, delayed);
        loop_tail = drag;
        delayed = false;
    }
    wire(&mut g, loop_tail, integrate, 1, delayed);
    let mut tail = integrate;

    // O tamanho ao longo da vida — antes da máscara (ver o cabeçalho).
    if (cfg.size_end - 1.0).abs() > f32::EPSILON {
        let life = life_fraction(&mut g, tail);
        let map = g.add_node("value.map_range");
        g.set_param(map, "in_lo", 0.0);
        g.set_param(map, "in_hi", 1.0);
        g.set_param(map, "out_lo", 1.0);
        g.set_param(map, "out_hi", cfg.size_end.max(0.0));
        wire(&mut g, life, map, 0, false);
        let drive = g.add_node("motion.drive");
        g.set_param(drive, "channel", DRIVE_SIZE);
        g.set_param(drive, "mode", DRIVE_MULTIPLY);
        g.set_param(drive, "scale", 1.0);
        wire(&mut g, tail, drive, 0, false);
        wire(&mut g, map, drive, 1, false);
        tail = drive;
    }

    // A cor — a do nascimento, e (se diferir) a da morte misturada pela fracção de vida.
    let start = tint(&mut g, cfg.color);
    wire(&mut g, tail, start, 0, false);
    tail = start;
    if cfg.color_end != cfg.color {
        let life = life_fraction(&mut g, tail);
        let mask = g.add_node("motion.drive");
        g.set_param(mask, "channel", DRIVE_FALLOFF);
        g.set_param(mask, "mode", DRIVE_SET);
        g.set_param(mask, "scale", 1.0);
        wire(&mut g, tail, mask, 0, false);
        wire(&mut g, life, mask, 1, false);
        let end = tint(&mut g, cfg.color_end);
        wire(&mut g, mask, end, 0, false);
        tail = end;
    }

    let sink = g.add_node("motion.output");
    wire(&mut g, tail, sink, 0, false);
    Compiled {
        graph: g,
        emitter,
        sink,
    }
}

/// A fracção de vida de cada partícula do fluxo que sai de `from`.
fn life_fraction(g: &mut Graph, from: NodeId) -> NodeId {
    let n = g.add_node("value.attribute");
    g.set_param(n, "mode", ATTR_LIFE_FRACTION);
    g.set_text_param(n, ph2d_node_value_attribute::ATTR_KEY, "age");
    wire(g, from, n, 0, false);
    n
}

/// Um `motion.tint` que põe `rgba` (modo Mix, máscara inteira).
fn tint(g: &mut Graph, rgba: [f32; 4]) -> NodeId {
    let n = g.add_node("motion.tint");
    for (k, v) in ["r", "g", "b", "a"].into_iter().zip(rgba) {
        g.set_param(n, k, v);
    }
    n
}

/// Os números do emissor que não mudam a meio da corrida.
fn set_emitter(g: &mut Graph, n: NodeId, cfg: &ParticleEmitter) {
    #[expect(clippy::cast_precision_loss, reason = "contagens e sementes pequenas")]
    let (amount, seed) = (cfg.amount as f32, cfg.seed as f32);
    let params = [
        ("life", cfg.life.max(0.0)),
        (
            ph2d_node_motion_emitter::LIFE_RANDOM,
            cfg.life_random.clamp(0.0, 1.0),
        ),
        ("speed", cfg.speed),
        ("speed_random", cfg.speed_random),
        ("angle", cfg.angle),
        ("spread", cfg.spread),
        ("shape_mode", f32::from(cfg.shape.index())),
        ("shape_w", cfg.shape_size[0]),
        ("shape_h", cfg.shape_size[1]),
        ("seed", seed),
        ("max", amount),
        ("size", cfg.size.max(0.0)),
        ("size_random", cfg.size_random),
        ("x", 0.0),
        ("y", 0.0),
    ];
    for (k, v) in params {
        g.set_param(n, k, v);
    }
}

/// **Põe a emissão** — os únicos números que a corrida muda sem recompilar (ver `plan`).
pub fn apply_emission(g: &mut Graph, n: NodeId, cfg: &ParticleEmitter, em: &Emission) {
    let p = emit_params(cfg, em);
    g.set_param(n, "emit_mode", p.mode);
    g.set_param(n, "rate", p.rate);
    g.set_param(n, "burst_count", p.burst_count);
    g.set_param(n, "burst_time", 0.0);
    g.set_param(n, "burst_period", 0.0);
    g.set_text_param(n, SCHEDULE_KEY, p.schedule.format());
}
