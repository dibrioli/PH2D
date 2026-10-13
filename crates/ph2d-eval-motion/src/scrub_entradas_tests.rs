//! As ENTRADAS publicadas no scrub (ciclo 19, doc 128 §3.3): um valor que o jogo publicou num
//! tique tem de ser o MESMO quando o scrub volta a esse tique — senão o replay diverge do que se
//! viu a tocar.
//!
//! O arnês: um acumulador sequencial que soma, a cada tique, o número publicado em `$input:g`.
//! A trajectória depende de TODO o histórico da entrada, logo um replay com o valor de agora (e
//! não o da época) aparece no número.

use crate::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
const ENTRADA: &str = "$input:g";

static ACC_MAN: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.test.acumula"),
    name: "motion.test.acumula",
    inputs: &[PortSpec {
        name: "state",
        ty: INST,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST,
    }],
    effect: ph2d_nodegraph::effect::Effect::Pure,
    clock: Clock::Frame,
    params: &[],
    lowerings: &[LoweringKind::Cpu],
};
struct Acumula;
impl NodeOp for Acumula {
    fn manifest(&self) -> &'static NodeManifest {
        &ACC_MAN
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let prev = match ctx.input(0).get("P") {
            Some(Column::Vec2(v)) if !v.is_empty() => v[0][0],
            _ => 0.0,
        };
        let g = match ctx.external(ENTRADA).get("v") {
            Some(Column::Scalar(v)) if !v.is_empty() => v[0],
            _ => 0.0,
        };
        ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[prev + g, 0.0]])));
    }
}
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        (ty == ACC_MAN.id).then_some(&Acumula as &dyn NodeOp)
    }
}

fn grafo() -> (Graph, NodeId) {
    let mut g = Graph::new();
    let c = g.add_node("motion.test.acumula");
    g.connect(Edge {
        from: (c, 0),
        to: (c, 0),
        delayed: true,
    })
    .unwrap();
    (g, c)
}

const UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const SIZE: [f32; 2] = [1.0, 1.0];
const DT: f64 = 1.0 / 60.0;

fn publica(pump: &mut MotionCookPump, g: f32) {
    pump.cook
        .set_external(ENTRADA, Stream::new(1).with("v", Column::Scalar(vec![g])));
}

/// O que o jogo publica no tique `t` desta corrida: 1 até ao 19, 5 do 20 em diante.
fn entrada_da_corrida(t: u64) -> f32 {
    if t < 20 { 1.0 } else { 5.0 }
}

/// Toca para a frente, como o editor: publica a entrada do quadro e coze o tique. Devolve o
/// que se viu em cada tique.
fn toca(
    pump: &mut MotionCookPump,
    g: &Graph,
    c: NodeId,
    de: u64,
    ate: u64,
    e: impl Fn(u64) -> f32,
) -> Vec<f32> {
    let scopes = TimeScopes::new();
    (de..=ate)
        .map(|t| {
            publica(pump, e(t));
            pump.advance_or_scrub_scoped(g, &Ops, &[c], t, |k| k as f64 * DT, UV, SIZE, &scopes);
            pump.instances[0].world_pos[0]
        })
        .collect()
}

fn scrub(pump: &mut MotionCookPump, g: &Graph, c: NodeId, t: u64) -> f32 {
    let scopes = TimeScopes::new();
    pump.advance_or_scrub_scoped(g, &Ops, &[c], t, |k| k as f64 * DT, UV, SIZE, &scopes);
    pump.instances[0].world_pos[0]
}

/// ⭐ A lei: voltar a um tique já tocado mostra o que se viu nele, com a entrada de agora ainda
/// publicada (o jogo não volta atrás com o editor).
#[test]
fn a_scrub_replays_the_inputs_that_were_published_at_each_tick() {
    let (g, c) = grafo();
    let mut pump = MotionCookPump::new();
    pump.define_a_lei(false);
    let visto = toca(&mut pump, &g, c, 0, 40, entrada_da_corrida);

    for alvo in [10u64, 25, 3, 39, 20, 19] {
        // O editor publica o valor de AGORA antes do quadro, como o jogo faria.
        publica(&mut pump, entrada_da_corrida(40));
        let x = scrub(&mut pump, &g, c, alvo);
        assert_eq!(
            x, visto[alvo as usize],
            "scrub para {alvo}: viu-se {} ao tocar e o replay deu {x}",
            visto[alvo as usize]
        );
    }
}

/// ⭐ E um RAMO novo: voltar ao 25, mudar a entrada e tocar de novo reescreve o futuro — um
/// scrub ao 30 mostra o ramo NOVO, nunca o checkpoint velho do primeiro ramo.
#[test]
fn playing_on_after_a_scrub_with_a_new_input_writes_a_new_future() {
    let (g, c) = grafo();
    let mut pump = MotionCookPump::new();
    pump.define_a_lei(false);
    toca(&mut pump, &g, c, 0, 40, entrada_da_corrida);

    publica(&mut pump, 5.0);
    scrub(&mut pump, &g, c, 25);
    let ramo = |t: u64| if t <= 25 { entrada_da_corrida(t) } else { 2.0 };
    let novo = toca(&mut pump, &g, c, 26, 35, ramo);

    for alvo in [30u64, 33, 27] {
        publica(&mut pump, 2.0);
        let x = scrub(&mut pump, &g, c, alvo);
        assert_eq!(
            x,
            novo[(alvo - 26) as usize],
            "scrub para {alvo} depois do ramo novo: o replay deu {x}"
        );
    }
    // E o ramo novo, fechado à mão: até ao 19 soma 1 (20), do 20 ao 25 soma 5 (+30), do 26 ao
    // 30 soma 2 (+10).
    publica(&mut pump, 2.0);
    assert_eq!(scrub(&mut pump, &g, c, 30), 60.0);
}

// ── o gatilho: a contagem vira pulso, tique a tique ─────────────────────────────

const GATILHO: &str = "$input:g";

fn conta(pump: &mut MotionCookPump, n: f32) {
    pump.cook.set_external(
        GATILHO,
        Stream::new(1).with(
            ph2d_nodegraph::external::INPUT_COUNT,
            Column::Scalar(vec![n]),
        ),
    );
}

/// ⭐ O pulso é de UM tique, conte o quadro quantos tiques contar: com o acumulador, a soma é o
/// número de disparos — e um quadro que coze dois tiques não dispara duas vezes.
#[test]
fn a_fire_count_becomes_a_pulse_of_exactly_one_tick() {
    let (g, c) = grafo();
    let mut pump = MotionCookPump::new();
    pump.define_a_lei(false);
    let scopes = TimeScopes::new();
    let tique = |pump: &mut MotionCookPump, t: u64| {
        pump.advance_or_scrub_scoped(&g, &Ops, &[c], t, |k| k as f64 * DT, UV, SIZE, &scopes);
        pump.instances[0].world_pos[0]
    };
    conta(&mut pump, 0.0);
    assert_eq!(tique(&mut pump, 0), 0.0);
    conta(&mut pump, 1.0); // um disparo
    assert_eq!(tique(&mut pump, 1), 1.0, "pulsa no tique em que chega");
    assert_eq!(
        tique(&mut pump, 2),
        1.0,
        "e só nesse: o mesmo quadro coze outro tique"
    );
    assert_eq!(tique(&mut pump, 3), 1.0);
    conta(&mut pump, 3.0); // dois disparos entre tiques: um pulso (o Trigger do Rive)
    assert_eq!(tique(&mut pump, 4), 2.0);
    assert_eq!(tique(&mut pump, 5), 2.0);
    conta(&mut pump, 4.0);
    for t in 6..=10 {
        tique(&mut pump, t);
    }
    let visto_8 = 3.0;
    // O scrub repete os pulsos da fita no tique certo.
    assert_eq!(scrub(&mut pump, &g, c, 8), visto_8);
    assert_eq!(scrub(&mut pump, &g, c, 1), 1.0);
    assert_eq!(scrub(&mut pump, &g, c, 0), 0.0);
}

/// ⭐ **Parado, arrastar a fita para a FRENTE repete** — o quadro do produto marcha tique a tique
/// (`ticks_owed`), e cada tique já gravado tem de mostrar a entrada da época, não a de agora; a
/// TOCAR, o mesmo caminho grava o que se publica (o ramo novo do teste acima).
#[test]
fn with_the_clock_stopped_dragging_forward_replays_the_recorded_inputs() {
    let (g, c) = grafo();
    let mut pump = MotionCookPump::new();
    pump.define_a_lei(false);
    let visto = toca(&mut pump, &g, c, 0, 40, entrada_da_corrida);
    publica(&mut pump, 99.0);
    scrub(&mut pump, &g, c, 10);
    pump.define_a_tocar(false);
    for t in 11..=35u64 {
        publica(&mut pump, 99.0);
        let x = scrub(&mut pump, &g, c, t);
        assert_eq!(x, visto[t as usize], "parado, tique {t}");
    }
    // E editar no MESMO tique, parado, vale o publicado (um ramo): o re-cozimento do mesmo tique
    // parte do estado já avançado (a lei de sempre da bomba) e soma o publicado — `0`, não o `5`
    // da fita.
    publica(&mut pump, 0.0);
    pump.mark_dirty_keeping_ring();
    assert_eq!(
        scrub(&mut pump, &g, c, 35),
        visto[35],
        "a edição parada pesa o publicado"
    );
    assert!(
        pump.fita()
            .estado_em(35)
            .is_some_and(|e| e[ENTRADA].value.get("v") == Some(&Column::Scalar(vec![0.0]))),
        "a fita guardou o ramo"
    );
}

/// ⭐ **Derivar duas vezes no MESMO tique dá o MESMO pulso** — as sub-passadas da placa e um
/// re-cozimento do mesmo tique voltam a derivar; o pulso compara com as contagens de ANTES do tique,
/// nunca com as que a própria derivação acabou de guardar.
#[test]
fn a_second_derivation_in_the_same_tick_keeps_the_pulse() {
    let (g, c) = grafo();
    let mut pump = MotionCookPump::new();
    pump.define_a_lei(false);
    conta(&mut pump, 0.0);
    scrub(&mut pump, &g, c, 0);
    conta(&mut pump, 1.0);
    assert_eq!(
        scrub(&mut pump, &g, c, 1),
        1.0,
        "o disparo pulsa no tique 1"
    );
    pump.mark_dirty_keeping_ring();
    // O mesmo tique outra vez: o estado já somou o tique 1 (a lei do re-cozimento), e o pulso
    // continua a ser 1 — senão o disparo seria engolido por quem coze duas vezes.
    assert_eq!(
        scrub(&mut pump, &g, c, 1),
        2.0,
        "o pulso do tique 1 continua 1"
    );
}

/// ⭐ **O REPLAY QUE MOSTRA A BASE DIZ-O** (doc 132 §2): com a fita dobrada, o scrub para antes da
/// base marca a bomba `Dobrada { desde: base }`; para depois dela, `Verdade`; e tocar de novo grava
/// e volta a `Verdade`.
#[test]
fn o_scrub_para_antes_da_base_marca_a_bomba_como_dobrada() {
    let (g, c) = grafo();
    let mut pump = MotionCookPump::new();
    pump.define_a_lei(false);
    pump.define_o_razao_da_fita(crate::Razao::new(4 << 10));
    #[expect(clippy::cast_precision_loss, reason = "um tique")]
    toca(&mut pump, &g, c, 0, 300, |t| t as f32);
    assert_eq!(
        pump.historia_mostrada(),
        Historia::Verdade,
        "a tocar, grava"
    );
    let (base, ultimo) = pump.fita().faixa().expect("gravou");
    assert!(
        base > 20 && ultimo == 300,
        "o tecto pequeno dobrou: {base}..{ultimo}"
    );
    scrub(&mut pump, &g, c, base / 2);
    assert_eq!(pump.historia_mostrada(), Historia::Dobrada { desde: base });
    scrub(&mut pump, &g, c, base + 5);
    assert_eq!(pump.historia_mostrada(), Historia::Verdade, "o CONTROLO");
}
