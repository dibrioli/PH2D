//! **A cena pronta para o smoke do `value.attribute`** (`PH2D_ATTR_SMOKE=1`).
//!
//! Três exemplos de uso FUNCIONAIS e simples do nó de canais, lado a lado numa
//! ÚNICA cena — cada um é uma pequena fonte de partículas que LÊ uma coluna por
//! partícula (o que o picker do painel escolhe) e a usa para dirigir um canal
//! VISUAL (`motion.drive`). É a frase que a doc do nó promete — *"colorir as
//! faíscas por quão velhas elas são"* — feita em três variações que se veem.
//!
//! O caminho de render é sempre o mesmo, com POUCOS nós:
//!
//! ```text
//! emitter → [force?] → integrate → tint → value.attribute → [map_range?] → drive → output
//! ```
//!
//! - **`emitter`** cospe partículas com colunas por-partícula reais (`age`, `vel`,
//!   `life`, `size`); **`integrate`** as move (sem ele ficam empilhadas na origem);
//!   **`tint`** dá uma cor sólida para a fonte ser distinguível.
//! - **`value.attribute`** lê UMA coluna como um campo de valor — é o nó em foco.
//! - **`motion.drive`** escreve esse valor num canal (Tamanho / Opacidade).
//! - **`motion.output`** é o sink de render; a cena tem TRÊS (a shell compõe vários
//!   sinks num desenho só), então as três fontes coexistem.
//!
//! As três (da esquerda para a direita no canvas):
//!
//! 1. **IDADE → TAMANHO** (esquerda, quente): a partícula nasce minúscula e INCHA
//!    conforme envelhece — `attribute(Age)` dirige o Tamanho. O canal mais direto:
//!    uma coluna que a partícula CARREGA vira o tamanho dela.
//! 2. **VELOCIDADE → OPACIDADE** (centro, ciano, com gravidade): a fonte arqueia
//!    sob gravidade, então a velocidade VARIA — lenta no ápice, rápida na descida.
//!    `attribute(Speed)` (a MAGNITUDE de `vel`) dirige a Opacidade: rápido = vivo,
//!    lento = apagado. Mostra o modo *Length* do picker (Vec2 → escalar).
//! 3. **IDADE → OPACIDADE, invertida** (direita, verde): `attribute(Age)` →
//!    `value.map_range` (0..life → 1..0) → Opacidade. A fonte DESVANECE nas pontas.
//!    Mostra o `value.attribute` alimentando OUTRO nó de valor — a composição que é
//!    a razão de existir do domínio de valor.
//!
//! Sem a env (ou entre portas do mesmo tipo) nada disto acontece. Abra o painel do
//! grafo (o botão ARRANGE arruma os nós numa linha) e o painel de params para ver o
//! picker de canais em cada `Attribute`.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId, Pos};

/// Ligado? Lido UMA vez (o prólogo do frame não paga um `getenv` por quadro).
fn on() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var("PH2D_ATTR_SMOKE").is_ok_and(|v| v != "0"))
}

static FRAME: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// `motion.drive` channel ids (`0 X · 1 Y · 2 Rotation · 3 Size · 4 Opacity`).
const CH_SIZE: f32 = 3.0;
const CH_OPACITY: f32 = 4.0;
/// `value.attribute` read mode (`0` = a coluna escalar · `1` = a MAGNITUDE de um Vec2).
const READ_SCALAR: f32 = 0.0;
const READ_LENGTH: f32 = 1.0;

/// One demo chain's recipe (a fountain that reads a column and drives a channel).
struct Chain {
    /// Rótulo humano (só para o log do smoke).
    label: &'static str,
    /// Origem no MUNDO — as três fontes ocupam faixas distintas do canvas.
    x: f32,
    /// Semente do emitter (fontes distintas não se espelham).
    seed: f32,
    /// Vida da partícula em segundos (também o teto do `map_range` da fade).
    life: f32,
    /// Velocidade de lançamento.
    speed: f32,
    /// A coluna que o `value.attribute` lê + o modo (escalar × magnitude).
    read_attr: &'static str,
    read_mode: f32,
    /// O canal que o `motion.drive` escreve + o multiplicador do valor.
    drive_channel: f32,
    drive_scale: f32,
    /// Arqueia sob gravidade (faz a VELOCIDADE variar — o exemplo do Speed).
    gravity: bool,
    /// Inverte o valor via `value.map_range` (0..life → 1..0) — a fade da idade.
    invert: bool,
    /// Cor sólida do tint (RGBA), para a fonte ser distinguível.
    color: [f32; 4],
}

/// The three examples, laid out left→right across the canvas.
fn chains() -> [Chain; 3] {
    [
        Chain {
            label: "IDADE -> TAMANHO (incha com a idade)",
            x: -10.0,
            seed: 11.0,
            life: 2.0,
            speed: 6.0,
            read_attr: "age",
            read_mode: READ_SCALAR,
            drive_channel: CH_SIZE,
            drive_scale: 0.16, // size = 0.16 * age  →  ~0 no nascimento, 0.32 na morte
            gravity: false,
            invert: false,
            color: [1.0, 0.72, 0.30, 1.0], // quente
        },
        Chain {
            label: "VELOCIDADE -> OPACIDADE (rapido = vivo)",
            x: 0.0,
            seed: 22.0,
            life: 3.0,
            speed: 12.0,
            read_attr: "vel",
            read_mode: READ_LENGTH, // a MAGNITUDE de vel = speed
            drive_channel: CH_OPACITY,
            drive_scale: 0.035, // opacidade = 0.035 * speed  →  lento apaga, rapido acende
            gravity: true,      // arqueia: velocidade varia ao longo da parabola
            invert: false,
            color: [0.40, 0.72, 1.0, 1.0], // ciano
        },
        Chain {
            label: "IDADE -> OPACIDADE invertida (desvanece nas pontas)",
            x: 10.0,
            seed: 33.0,
            life: 2.0,
            speed: 6.0,
            read_attr: "age",
            read_mode: READ_SCALAR,
            drive_channel: CH_OPACITY,
            drive_scale: 1.0, // o map_range ja normalizou para 0..1
            gravity: false,
            invert: true,                  // age → map_range(0..life → 1..0) → opacidade
            color: [0.55, 1.0, 0.55, 1.0], // verde
        },
    ]
}

/// Build one chain into `g`, return its `motion.output` node. `row` places the
/// nodes in a readable band in graph space (the ARRANGE button can re-line them).
fn build_chain(g: &mut Graph, c: &Chain, row: usize) -> NodeId {
    let em = g.add_node("motion.emitter");
    g.set_param(em, "rate", 1200.0);
    g.set_param(em, "life", c.life);
    g.set_param(em, "max", 4000.0);
    g.set_param(em, "size", 0.06);
    g.set_param(em, "speed", c.speed);
    g.set_param(em, "angle", 90.0); // para cima (mundo Y-up)
    g.set_param(em, "spread", 30.0);
    g.set_param(em, "x", c.x);
    g.set_param(em, "y", -7.0);
    g.set_param(em, "seed", c.seed);

    let ig = g.add_node("motion.integrate");

    let tint = g.add_node("motion.tint");
    g.set_param(tint, "mode", 0.0); // Solid
    g.set_param(tint, "r", c.color[0]);
    g.set_param(tint, "g", c.color[1]);
    g.set_param(tint, "b", c.color[2]);
    g.set_param(tint, "a", c.color[3]);

    // O nó em FOCO: lê uma coluna por-partícula como campo de valor.
    let attr = g.add_node("value.attribute");
    g.set_text_param(attr, "attr", c.read_attr);
    g.set_param(attr, "mode", c.read_mode);

    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", c.drive_channel);
    g.set_param(drive, "scale", c.drive_scale);
    g.set_param(drive, "mode", 1.0); // Set (o valor VIRA o canal)

    let out = g.add_node("motion.output");

    // Gravidade opcional (o exemplo da Velocidade): força para baixo que arqueia a
    // fonte, fazendo a velocidade variar ao longo da parábola.
    let gravity = c.gravity.then(|| {
        let w = g.add_node("force.wind");
        g.set_param(w, "angle", 270.0); // reto para baixo
        g.set_param(w, "strength", 22.0);
        g.set_param(w, "gust", 0.0);
        w
    });

    // map_range opcional (a fade da idade): inverte 0..life → 1..0.
    let map = c.invert.then(|| {
        let m = g.add_node("value.map_range");
        g.set_param(m, "in_lo", 0.0);
        g.set_param(m, "in_hi", c.life);
        g.set_param(m, "out_lo", 1.0);
        g.set_param(m, "out_hi", 0.0);
        g.set_param(m, "clamp", 1.0);
        m
    });

    // Layout: uma faixa por chain (posições de GRAFO, não de mundo).
    let y = 60.0 + row as f32 * 240.0;
    for (i, n) in [em, ig, tint, attr, drive, out].into_iter().enumerate() {
        g.set_pos(
            n,
            Pos {
                x: 60.0 + i as f32 * 190.0,
                y,
            },
        );
    }
    if let Some(w) = gravity {
        g.set_pos(
            w,
            Pos {
                x: 250.0,
                y: y + 110.0,
            },
        );
    }
    if let Some(m) = map {
        g.set_pos(
            m,
            Pos {
                x: 820.0,
                y: y + 110.0,
            },
        );
    }

    // Fiação. `motion.integrate` tem porta 0 = `rest` (a montante) e porta 1 =
    // `forces` (o laço de feedback, sempre `delayed`). `motion.drive` tem porta 0 =
    // stream e porta 1 = o campo de valor.
    let mut wire = |from: NodeId, fp: u16, to: NodeId, tp: u16, delayed: bool| {
        g.connect(Edge {
            from: (from, fp),
            to: (to, tp),
            delayed,
        })
        .expect("attribute-demo edge");
    };
    wire(em, 0, ig, 0, false);
    match gravity {
        // ig.out --pre(delayed)--> força --> ig.forces (o laço que o editor
        // auto-desenha; aqui explícito).
        Some(w) => {
            wire(ig, 0, w, 0, true);
            wire(w, 0, ig, 1, false);
        }
        // Sem forças: o auto-laço puro out --pre--> forces.
        None => wire(ig, 0, ig, 1, true),
    }
    wire(ig, 0, tint, 0, false);
    // Fan-out do tint: o stream vai para o `attribute` (que lê a coluna) E para o
    // `drive` (que recebe o stream a modular). Portas de saída podem alimentar
    // vários destinos.
    wire(tint, 0, attr, 0, false);
    wire(tint, 0, drive, 0, false);
    match map {
        Some(m) => {
            wire(attr, 0, m, 0, false);
            wire(m, 0, drive, 1, false);
        }
        None => wire(attr, 0, drive, 1, false),
    }
    wire(drive, 0, out, 0, false);
    out
}

/// Build the whole three-example document (a fresh graph, validated).
fn build_attribute_demo(reg: &NodeRegistry) -> MotionDoc {
    let mut doc = MotionDoc::new();
    for (row, c) in chains().iter().enumerate() {
        build_chain(&mut doc.graph, c, row);
    }
    doc.graph
        .validate(reg)
        .expect("attribute-demo document is well-typed");
    doc
}

impl crate::App {
    /// Roda no prólogo do frame, ao lado do `adapter_smoke`/`build_smoke`. No-op
    /// sem a env; caso contrário, substitui o documento do Motion pela cena dos três
    /// exemplos e entra na ferramenta Motion (que auto-toca na entrada).
    pub(crate) fn attribute_demo_smoke(&mut self) {
        use std::sync::atomic::Ordering;
        if !on() || self.gfx.is_none() || FRAME.fetch_add(1, Ordering::Relaxed) != 4 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        gfx.motion.doc = build_attribute_demo(&gfx.motion.registry);
        // A ponte recomputa os sinks a partir dos nós `motion.output` do documento;
        // limpar aqui é higiene (o boot doc pode ter deixado outros).
        gfx.motion.sinks.clear();
        let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
        eprintln!(
            "[attribute smoke] TRES exemplos de `value.attribute` numa cena so, da esquerda para a direita:"
        );
        for (i, c) in chains().iter().enumerate() {
            eprintln!("  {}. {}", i + 1, c.label);
        }
        eprintln!(
            "  Cada fonte LE uma coluna por-particula (age / vel) e a usa para dirigir \
             um canal visual (Tamanho / Opacidade). O exemplo 2 mostra o modo Length do \
             picker (Vec2 -> escalar) e o 3 alimenta OUTRO no de valor (map_range).\n  \
             Abra o painel do grafo (o botao ARRANGE arruma os nos numa linha) e o \
             painel de params: cada `Attribute` mostra o PICKER DE CANAIS. (Sem a env, \
             nada muda.)"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> NodeRegistry {
        let mut reg = NodeRegistry::new();
        ph2d_node_registry_init::register_all_nodes(&mut reg).expect("node registry builds");
        reg
    }

    /// A cena inteira é bem-tipada e monta os TRÊS sinks — se qualquer aresta
    /// (fan-out, o laço `delayed` do integrate, a porta de valor do drive) estivesse
    /// errada, `validate` reprovaria e o produto renderizaria vazio em silêncio.
    #[test]
    fn the_three_example_scene_is_well_typed_with_three_sinks() {
        let doc = build_attribute_demo(&registry());
        let sinks = doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == "motion.output")
            .count();
        assert_eq!(sinks, 3, "one render sink per example");
    }

    /// Cada exemplo carrega um `value.attribute` cuja coluna lida é o que a receita
    /// pediu — o texto no canal de text-param, não um default. (Uma coluna trocada
    /// leria zeros e o drive não modularia nada: verde no grafo, morto na tela.)
    #[test]
    fn each_example_reads_the_column_its_recipe_names() {
        let doc = build_attribute_demo(&registry());
        let attrs: Vec<&str> = doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == "value.attribute")
            .map(|n| {
                doc.graph
                    .node_text_param_overrides(n.id)
                    .and_then(|m| m.get("attr"))
                    .map(String::as_str)
                    .unwrap_or("<none>")
            })
            .collect();
        assert_eq!(attrs.len(), 3, "one Attribute node per example");
        // As três receitas: age, vel (Speed via Length), age (fade).
        assert!(
            attrs.contains(&"vel"),
            "the Speed example reads `vel`: {attrs:?}"
        );
        assert_eq!(
            attrs.iter().filter(|a| **a == "age").count(),
            2,
            "two examples read `age`: {attrs:?}"
        );
    }

    /// The whole read→drive path is exercised END TO END through the real registry,
    /// not just type-checked. Cook the IDADE→TAMANHO example at a mid-tick: the
    /// emitter must produce particles, and `value.attribute(age)` → `drive(Size)`
    /// must leave the `size` column NON-constant (older particles bigger). A broken
    /// read (wrong column → zeros) or a dropped drive would leave size flat — green
    /// in the graph, dead on the screen — and this gate is what catches it.
    #[test]
    fn the_age_to_size_example_actually_drives_size_end_to_end() {
        use crate::motion_state::MotionState;
        use ph2d_nodegraph::attr::Column;
        use ph2d_nodegraph::cook::Cook;

        let motion = MotionState::new(); // registry = register_all_nodes
        let mut g = Graph::new();
        let out = build_chain(&mut g, &chains()[0], 0); // IDADE -> TAMANHO
        g.validate(&motion.registry).expect("chain is well-typed");

        let mut cook = Cook::new();
        let streams = cook
            .cook(&g, &motion.registry, out, 0.5)
            .expect("the example cooks end to end");
        let s = streams[0].as_stream();
        assert!(s.count() > 0, "the emitter produced particles at t=0.5");
        match s.get("size") {
            Some(Column::Vec2(v)) => {
                let mn = v.iter().map(|p| p[0]).fold(f32::INFINITY, f32::min);
                let mx = v.iter().map(|p| p[0]).fold(f32::NEG_INFINITY, f32::max);
                assert!(
                    mx - mn > 1e-3,
                    "attribute(age) -> drive(Size) modulated the size column: min={mn} max={mx}"
                );
            }
            other => panic!("expected a Vec2 `size` column, got {other:?}"),
        }
    }
}
