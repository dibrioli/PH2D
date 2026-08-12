//! **O TETO DO `motion.spring` É ONDE O KERNEL DEIXA DE HONRÁ-LO** (doc 88 B2 · doc 89 folha 03).
//!
//! Os números e a tabela vivem no doc-comment do `PARAM_HARD_MAX` da crate; a sonda que os
//! mediu é a `measure_spring_ceiling`. Estes gates afirmam a **PROPRIEDADE**, não a constante:
//! dirigem a mola pela porta do produto no teto e ACIMA dele, e exigem que o comportamento
//! mude ali. Um gate que comparasse o número registado com um literal seria um espelho — ele
//! seguiria verde no dia em que o kernel mudasse e o teto passasse a mentir.
//!
//! ⚠️ **O relógio é o do PIOR caso (`MAX_DT = 0,1`), e essa escolha É metade do gate.** A 60 fps
//! um `friction` de 100 funciona; o teto existe porque um quadro perdido entrega `0,1` ao
//! integrador, e um teto que só vale na máquina rápida não é um teto.

use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};
use ph2d_nodegraph::node::NodeTypeId;
use ph2d_nodegraph::value::CookValue;

/// O pior `dt` que o `eval` admite (o clamp `MAX_DT` da crate da mola).
const WORST_DT: f64 = 0.1;
const TARGET_Y: f32 = 100.0;
const SPRING: NodeTypeId = NodeTypeId::of("motion.spring");

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("every node registers");
    reg
}

fn wire(g: &mut Graph, from: NodeId, to: NodeId, to_port: u16, delayed: bool) {
    g.connect(Edge {
        from: (from, 0),
        to: (to, to_port),
        delayed,
    })
    .expect("edge");
}

/// Um degrau e uma mola, com o `pre` self-loop que o editor plumba ao SOLTAR o nó.
///
/// ⚠️ Sem o self-loop o `state` chega vazio todo tique e a mola **nunca integra** — a fixture
/// ficaria verde sobre um nó que não faz nada (a armadilha que o `motion.boids` já pagou).
fn scene(tension: f32, friction: f32) -> (Graph, NodeId, NodeId) {
    let mut g = Graph::new();
    let seed = g.add_node("motion.grid");
    g.set_param(seed, "rows", 1.0);
    g.set_param(seed, "cols", 1.0);
    let step = g.add_node("motion.transform");
    g.set_param(step, "offset_y", 0.0);
    let spring = g.add_node("motion.spring");
    g.set_param(spring, "channel", 1.0);
    g.set_param(spring, "tension", tension);
    g.set_param(spring, "friction", friction);
    wire(&mut g, seed, step, 0, false);
    wire(&mut g, step, spring, 0, false);
    wire(&mut g, spring, spring, 1, true);
    (g, spring, step)
}

fn py(s: &Stream) -> f32 {
    match s.get("P") {
        Some(Column::Vec2(v)) if !v.is_empty() => v[0][1],
        _ => f32::NAN,
    }
}

/// O que a mola FAZ, nos três regimes que a medição achou.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Verdict {
    /// Persegue o alvo e assenta — a física que o artista pediu.
    Sane,
    /// Cresce sem limite: o passo explícito perdeu a estabilidade.
    Explodes,
    /// **Pregada no alvo pelo guard de NaN** — finita, imóvel e instantânea.
    ///
    /// ⚠️ Este braço não é zelo: sem ele o oráculo dá *sadia* para uma tensão de 4 milhões,
    /// cujo pico é exactamente `100,000`. O guard repõe a posição no alvo a cada tique e
    /// **apaga a prova** — o discriminante é o TEMPO, não a magnitude.
    Snaps,
}

/// Marcha a cena e classifica. O degrau cai no tique 10; um elemento recém-nascido fica NO
/// alvo por contrato (`fresh id: stays at its target`), então sem o degrau não há o que medir.
fn run(tension: f32, friction: f32) -> Verdict {
    let reg = registry();
    let (mut g, spring, step) = scene(tension, friction);
    let mut cook = Cook::new();
    let (mut peak, mut just_after) = (0.0f32, f32::NAN);
    for t in 0..600u64 {
        if t == 10 {
            g.set_param(step, "offset_y", TARGET_Y);
        }
        let playhead = t as f64 * WORST_DT;
        cook.advance_tick(&g, &reg, playhead).expect("tick");
        let out = cook.cook(&g, &reg, spring, playhead).expect("cook");
        let CookValue::Instances(s) = &out[0] else {
            panic!("a saida da mola e um stream")
        };
        let y = py(s);
        if y.is_finite() {
            peak = peak.max(y.abs());
        } else {
            peak = f32::INFINITY;
        }
        if t == 11 {
            just_after = y;
        }
    }
    // A resposta ao degrau de uma mola SEM atrito passa em exactamente 2× o alvo, então `10×`
    // deixa uma ordem inteira de folga entre *oscila muito* e *explode*.
    if !peak.is_finite() || peak > TARGET_Y * 10.0 {
        Verdict::Explodes
    } else if (just_after - TARGET_Y).abs() < TARGET_Y * 0.01 {
        Verdict::Snaps
    } else {
        Verdict::Sane
    }
}

fn ceiling(param: &str) -> f32 {
    registry()
        .param_hard_max(SPRING, param)
        .unwrap_or_else(|| panic!("a mola declara um teto digitavel para `{param}`"))
}

/// **A ENTREGA:** os dois params têm teto, e o teto é HONRADO — no valor declarado, com o outro
/// knob no teto DELE, a mola ainda persegue o alvo.
///
/// ⚠️ **O par importa:** medir a tensão com `friction` no default responde sobre um documento
/// que o artista não é obrigado a autorar. O canto é o que um teto tem de sobreviver.
#[test]
fn the_declared_ceilings_are_honoured_at_the_corner() {
    let (t, f) = (ceiling("tension"), ceiling("friction"));
    assert_eq!(
        run(t, f),
        Verdict::Sane,
        "nos dois tetos ao mesmo tempo (tension {t}, friction {f}) a mola ainda tem de perseguir \
         o alvo -- um teto que ja nao e honrado no proprio valor nao e um teto"
    );
}

/// **E o teto do `friction` é o do PIOR caso — a tensão MÍNIMA.**
///
/// Este é o gate load-bearing da escolha: o limite verdadeiro é `2 / sub_dt`, e o `sub_dt`
/// encolhe com a tensão ⇒ sob tensão alta um `friction` de 200 é estável. Escolher aquele
/// número deixaria o artista digitar 200 e ver a mola explodir ao baixar a tensão — *o valor
/// certo seria função de OUTRO knob*, que esta casa trata como bug de desenho. O único número
/// que nunca mente é o do pior caso.
#[test]
fn the_friction_ceiling_holds_at_the_lowest_tension_not_just_the_highest() {
    let f = ceiling("friction");
    let floor_tension = 0.1; // o piso que o `eval` aplica (`ctx.param("tension").max(0.1)`)
    assert_eq!(
        run(floor_tension, f),
        Verdict::Sane,
        "com a tensao no PISO o friction no teto ({f}) ainda tem de ser estavel"
    );
    // E logo acima ele deixa de ser: é isto que torna o número um teto, e não uma folga.
    assert_ne!(
        run(floor_tension, f * 2.0),
        Verdict::Sane,
        "o DOBRO do teto de friction, com a tensao no piso, nao pode ser sadio -- se for, o teto \
         esta baixo demais e esta a roubar faixa util do artista"
    );
}

/// **ACIMA do teto de tensão a mola deixa de ser uma mola** — e o modo de falha é o SILENCIOSO.
///
/// ⚠️ Uma ordem de grandeza acima ela nem explode à vista: o guard de NaN a prega no alvo, e o
/// artista vê um controle que **parece não fazer nada**. É exactamente o que ele vê hoje, sem
/// teto nenhum, ao digitar um número grande — e é o que este teto existe para tornar impossível.
#[test]
fn far_above_the_tension_ceiling_the_guard_pins_the_spring_and_that_is_the_silent_failure() {
    let t = ceiling("tension");
    assert_eq!(
        run(t * 4.0, 20.0),
        Verdict::Snaps,
        "quatro vezes o teto tem de PREGAR a mola no alvo (o guard de NaN), que e o modo de \
         falha que nenhuma medicao de magnitude enxerga"
    );
}

/// O `ParamUiHint.max` do `friction` **é** o teto duro — e isso é um FATO, não um descuido.
///
/// ⚠️ O par slider/chip do doc 88 B2 pressupõe que a faixa confortável seja MENOR que o
/// digitável. Aqui as duas coincidem, e a coincidência é o achado: `2 / MAX_DT = 20`. Este gate
/// existe para que ninguém "conserte" a igualdade abrindo folga que o kernel não tem.
#[test]
fn the_friction_slider_already_sits_on_the_stability_limit() {
    let reg = registry();
    let hint = reg
        .param_ui(SPRING)
        .expect("a mola tem hints de UI")
        .iter()
        .find(|h| h.param == "friction")
        .expect("o friction tem hint de UI");
    assert!(
        (hint.max - ceiling("friction")).abs() < f32::EPSILON,
        "o slider do friction ({}) e o teto digitavel ({}) coincidem de proposito: os dois sao \
         2/MAX_DT, o limite de estabilidade do amortecimento explicito no pior passo",
        hint.max,
        ceiling("friction")
    );
}
