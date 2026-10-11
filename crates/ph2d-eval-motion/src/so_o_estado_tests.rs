//! Os portões de `so_o_estado`: um quadro que recupera `N` tiques coze a cadeia `Pure` UMA vez, e
//! desenha os mesmos bits que compor todo tique — com estado por `pre` e por `memo`, e no scrub.

use crate::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::{EvalCtx, Memo};
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::graph::{Edge, Graph};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use std::cell::Cell;

const INST: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
const UV: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
const SIZE: [f32; 2] = [1.0, 1.0];
const DT: f64 = 1.0 / 60.0;

thread_local! {
    /// Quantas vezes o nó `Pure` caro (o papel do `motion.collide`) foi avaliado.
    static PURO: Cell<u32> = const { Cell::new(0) };
}

const fn man(name: &'static str, inputs: &'static [PortSpec], effect: Effect) -> NodeManifest {
    NodeManifest {
        id: NodeTypeId::of(name),
        name,
        inputs,
        outputs: &[PortSpec {
            name: "out",
            ty: INST,
        }],
        effect,
        clock: Clock::Frame,
        params: &[],
        lowerings: &[LoweringKind::Cpu],
    }
}
const UMA: &[PortSpec] = &[PortSpec {
    name: "in",
    ty: INST,
}];
/// O relógio (o papel do `value.lfo`): `P.x = playhead`, sem estado.
static RELOGIO: NodeManifest = man("t.relogio", &[], Effect::Temporal);
/// O caro (o papel do `motion.collide`): `P.x · 2`, conta as avaliações.
static PURO_MAN: NodeManifest = man("t.puro", UMA, Effect::Pure);
/// Fonte de `pre` e `Pure` no manifesto (o contador do `scrub_tests`): `P.x` sobe 1 por tique.
static CONTADOR: NodeManifest = man("t.contador", UMA, Effect::Pure);
/// Estado no `memo`, sem `pre` (como o `sim.step`): soma `1` por avaliação.
static ACUMULA: NodeManifest = man("t.acumula", &[], Effect::Temporal);

fn x(s: &Stream) -> f32 {
    match s.get("P") {
        Some(Column::Vec2(v)) if !v.is_empty() => v[0][0],
        _ => -1.0,
    }
}
fn emite(ctx: &mut EvalCtx<'_>, v: f32) {
    ctx.emit(Stream::new(1).with("P", Column::Vec2(vec![[v, 0.0]])));
}
#[derive(Clone)]
struct Soma(f32);
impl Memo for Soma {
    fn approx_bytes(&self) -> usize {
        4
    }
}

struct Op(&'static NodeManifest);
impl NodeOp for Op {
    fn manifest(&self) -> &'static NodeManifest {
        self.0
    }
    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        match self.0.name {
            "t.relogio" => emite(ctx, ctx.playhead() as f32),
            "t.puro" => {
                PURO.with(|c| c.set(c.get() + 1));
                let v = x(ctx.input(0)) * 2.0;
                emite(ctx, v);
            }
            "t.contador" => {
                let v = x(ctx.input(0)) + 1.0;
                emite(ctx, v);
            }
            _ => {
                let s = ctx.take_memo::<Soma>().map_or(0.0, |m| m.0) + 1.0;
                ctx.keep_memo(Soma(s));
                emite(ctx, s);
            }
        }
    }
}
static TODOS: [Op; 4] = [Op(&RELOGIO), Op(&PURO_MAN), Op(&CONTADOR), Op(&ACUMULA)];
struct Ops;
impl OpResolver for Ops {
    fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
        TODOS
            .iter()
            .find(|o| o.0.id == ty)
            .map(|o| o as &dyn NodeOp)
    }
}

#[derive(Clone, Copy)]
enum Fio {
    /// `fonte → t.puro`.
    Directo,
    /// `fonte → t.puro` e a fonte com laço de `pre` próprio (o contador).
    Laco,
    /// `fonte -pre-> t.puro`: o `Pure` CONSOME o `pre` sem ser fonte dele.
    Atrasado,
}

/// `fonte → t.puro (sink)`, ligado como diz o `fio`.
fn grafo(fonte: &str, fio: Fio) -> (Graph, NodeId) {
    let mut g = Graph::new();
    let f = g.add_node(fonte);
    let p = g.add_node("t.puro");
    g.connect(Edge {
        from: (f, 0),
        to: (p, 0),
        delayed: matches!(fio, Fio::Atrasado),
    })
    .unwrap();
    if matches!(fio, Fio::Laco) {
        g.connect(Edge {
            from: (f, 0),
            to: (f, 0),
            delayed: true,
        })
        .unwrap();
    }
    (g, p)
}

/// Corre três quadros de `tiques` tiques; `so_o_ultimo` = a lei da shell. Devolve os bits
/// desenhados no fim de cada quadro e as avaliações do `Pure`.
fn corre(fonte: &str, fio: Fio, so_o_ultimo: bool, tiques: u64) -> (Vec<u32>, u32) {
    let (g, sink) = grafo(fonte, fio);
    let mut pump = MotionCookPump::new();
    pump.define_a_lei(false);
    let scopes = TimeScopes::new();
    PURO.with(|c| c.set(0));
    let mut bits = Vec::new();
    for quadro in 0..3u64 {
        // 0, 4, 8: o 1.º quadro é o tique 0 (a rota para a frente); os outros recuperam 4.
        let alvo = quadro * tiques;
        let de = pump.last_cooked_tick().map_or(alvo, |l| l + 1);
        for t in de..=alvo {
            pump.set_separa_o_desenho(!so_o_ultimo || t == alvo);
            pump.advance_or_scrub_scoped(
                &g,
                &Ops,
                &[sink],
                t,
                |k| k as f64 * DT,
                UV,
                SIZE,
                &scopes,
            );
        }
        bits.push(pump.instances[0].world_pos[0].to_bits());
    }
    (bits, PURO.with(Cell::get))
}

/// ⭐⭐⭐ O defeito da cena `=8` sem placa: um quadro que recupera 4 tiques cozinhava o `Pure`
/// caro 4 vezes; agora 1 — e o desenho é o mesmo ao bit.
#[test]
fn um_quadro_que_recupera_tiques_coze_o_puro_uma_vez() {
    let (bits, puro) = corre("t.relogio", Fio::Directo, true, 4);
    let (bits_todos, puro_todos) = corre("t.relogio", Fio::Directo, false, 4);
    assert_eq!(bits, bits_todos, "saltar a composição mudou o desenho");
    assert_eq!(puro, 3, "o Pure foi cozido em tiques que não se desenham");
    assert_eq!(
        puro_todos,
        1 + 4 + 4,
        "controlo: compor todo tique coze-o em cada um"
    );
}

/// Uma fonte de `pre` (`Pure` no manifesto, o contador) continua a marchar todo tique.
#[test]
fn o_estado_por_pre_marcha_todo_tique() {
    let (bits, puro) = corre("t.contador", Fio::Laco, true, 4);
    let (bits_todos, _) = corre("t.contador", Fio::Laco, false, 4);
    assert_eq!(bits, bits_todos);
    // O contador vale o tique: 0, 4, 8 → o Pure desenha o dobro.
    let x: Vec<f32> = bits.iter().map(|b| f32::from_bits(*b)).collect();
    assert_eq!(x, [0.0, 8.0, 16.0], "o contador saltou tiques");
    assert_eq!(puro, 3);
}

/// Um `Pure` que CONSOME o `pre` sem ser fonte lê o tique anterior certo sem os intermédios.
#[test]
fn quem_consome_o_pre_le_o_tique_anterior_sem_os_intermedios() {
    let (bits, puro) = corre("t.relogio", Fio::Atrasado, true, 4);
    let (bits_todos, _) = corre("t.relogio", Fio::Atrasado, false, 4);
    assert_eq!(bits, bits_todos);
    // O tique 8 lê o relógio do tique 7.
    assert_eq!(f32::from_bits(bits[2]), (7.0 * DT) as f32 * 2.0);
    assert_eq!(puro, 3);
}

/// Um `Temporal` com estado no `memo` e sem `pre` também marcha todo tique (o corte pelo EFEITO).
#[test]
fn o_estado_no_memo_marcha_todo_tique() {
    let (bits, puro) = corre("t.acumula", Fio::Directo, true, 4);
    let (bits_todos, _) = corre("t.acumula", Fio::Directo, false, 4);
    assert_eq!(bits, bits_todos, "o acumulador saltou tiques");
    assert_eq!(puro, 3);
}

/// As tomadas são por TIQUE: o livro-razão de um quadro que recupera é o de compor todo tique.
#[test]
fn a_tomada_regista_cada_tique_recuperado() {
    let livro = |so_o_ultimo: bool| {
        let (g, sink) = grafo("t.relogio", Fio::Directo);
        let fonte = g.nodes()[0].id;
        let mut pump = MotionCookPump::new();
        pump.define_a_lei(false);
        pump.set_taps(&[fonte]);
        let scopes = TimeScopes::new();
        for t in 0..=8u64 {
            pump.set_separa_o_desenho(!so_o_ultimo || t % 4 == 0);
            pump.advance_or_scrub_scoped(
                &g,
                &Ops,
                &[sink],
                t,
                |k| k as f64 * DT,
                UV,
                SIZE,
                &scopes,
            );
        }
        let l: Vec<(u64, u32)> = pump
            .tap_fires()
            .iter()
            .map(|(t, _, s)| (*t, x(s).to_bits()))
            .collect();
        l
    };
    let l = livro(true);
    assert_eq!(l.len(), 9);
    assert_eq!(
        l,
        livro(false),
        "a tomada registou um tique velho num tique recuperado"
    );
}

/// O scrub re-simula do checkpoint até ao alvo e só o ALVO é composto — mesmos bits que tocar.
#[test]
fn o_scrub_compoe_so_o_alvo() {
    let (g, sink) = grafo("t.contador", Fio::Laco);
    let scopes = TimeScopes::new();
    let mut tocado = MotionCookPump::new();
    tocado.define_a_lei(false);
    for t in 0..=10u64 {
        tocado.pump_scoped(&g, &Ops, &[sink], t, t as f64 * DT, UV, SIZE, &scopes);
    }
    let mut pump = MotionCookPump::new();
    pump.define_a_lei(false);
    PURO.with(|c| c.set(0));
    pump.scrub_to_scoped(&g, &Ops, &[sink], 10, |k| k as f64 * DT, UV, SIZE, &scopes);
    assert_eq!(
        pump.instances[0].world_pos[0].to_bits(),
        tocado.instances[0].world_pos[0].to_bits()
    );
    assert_eq!(PURO.with(Cell::get), 1, "o scrub compôs tiques intermédios");
    assert_eq!(pump.cozimentos_so_do_estado(), 10);
}

/// Um scrub que sai cedo (sem sinks nem tomadas) devolve a bandeira: o quadro seguinte compõe.
#[test]
fn o_scrub_que_sai_cedo_devolve_a_bandeira() {
    let (g, sink) = grafo("t.relogio", Fio::Directo);
    let scopes = TimeScopes::new();
    let mut pump = MotionCookPump::new();
    pump.define_a_lei(false);
    pump.scrub_to_scoped(&g, &Ops, &[], 5, |k| k as f64 * DT, UV, SIZE, &scopes);
    pump.advance_or_scrub_scoped(&g, &Ops, &[sink], 6, |k| k as f64 * DT, UV, SIZE, &scopes);
    assert_eq!(
        pump.instances.len(),
        1,
        "o quadro depois do scrub não compôs"
    );
}

/// ⭐⭐⭐ **A causa medida no app** (cena `=8` sem placa): o gizmo de posições pede o SINK como
/// tomada, e a tomada cozia em todo tique. Uma tomada só do desenho coze só no tique desenhado.
#[test]
fn uma_tomada_de_gizmo_so_coze_no_tique_desenhado() {
    let quadros = |por_tique: usize| {
        let (g, sink) = grafo("t.relogio", Fio::Directo);
        let mut pump = MotionCookPump::new();
        pump.define_a_lei(false);
        pump.set_taps(&[sink]);
        pump.set_taps_por_tique(por_tique);
        let scopes = TimeScopes::new();
        PURO.with(|c| c.set(0));
        for t in 0..=8u64 {
            pump.set_separa_o_desenho(t % 4 == 0);
            pump.advance_or_scrub_scoped(
                &g,
                &Ops,
                &[sink],
                t,
                |k| k as f64 * DT,
                UV,
                SIZE,
                &scopes,
            );
        }
        let publicada = pump.tap_streams().iter().any(|(n, _)| *n == sink);
        (PURO.with(Cell::get), publicada)
    };
    assert_eq!(
        quadros(0),
        (3, true),
        "a tomada de gizmo cozeu tiques que não se desenham"
    );
    assert_eq!(
        quadros(1).0,
        9,
        "controlo: uma tomada por tique coze-o em cada tique"
    );
}
