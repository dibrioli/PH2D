//! GPU-vs-CPU parity para a **aritmética do domínio de VALOR** — os modos que o
//! grupo A da conferência (doc 89) acrescentou a cinco nós irmãos.
//!
//! ## Por que este gate é um SWEEP e não cinco gates
//!
//! Os gates de paridade que já existem cozinham **uma** configuração por nó, e a
//! wave do `value.noise` mediu o preço disso: a fixture dela cozinhava
//! `kernel = 0`, então os kernels novos teriam concordado **por VÁCUO** — o
//! branch que ninguém percorre concorda com qualquer coisa. Um modo novo num
//! `if/else if` de WGSL é exatamente essa forma de defeito: compila, o nó coza,
//! e o device devolve o ramo ERRADO em silêncio.
//!
//! Então cada modo NOVO entra aqui com o seu **CONTROLE** — o modo vizinho do
//! MESMO nó, sobre a MESMA entrada — e o gate afirma duas coisas por par:
//! *o device concorda com a CPU* **e** *os dois modos produzem campos
//! DIFERENTES*. Sem a segunda metade, um kernel que ignorasse o param de modo
//! passaria em todas as doze linhas.
//!
//! ## A entrada é ASSINADA de propósito
//!
//! Três dos seis pares só se distinguem no NEGATIVO — `Modulo` contra
//! `Floored Modulo` (o sinal segue o dividendo ou o divisor) e `Floor` contra
//! `Truncate` (o mesmo acima de zero, opostos abaixo). Uma rampa `[0,1]` os
//! tornaria indistinguíveis, e o gate ficaria verde sobre um device que trocasse
//! os dois. A rampa é esticada para `[−2, 2]` antes de chegar neles.
//!
//! ## A tolerância não pode esconder o defeito que este gate procura
//!
//! ε = 1e-4, o mesmo orçamento derivado dos irmãos. ⚠️ E ele é adequado por uma
//! razão que vale escrever: um ramo ERRADO de `if/else if` não erra por um ulp —
//! um degrau perdido no `Stepped` vale `1/steps` do alcance, um `trunc` lido como
//! `floor` vale um passo inteiro da grade. **A falha que este gate persegue é
//! macroscópica**; a tolerância existe só para o FMA que o WGSL pode contrair.
//!
//! ## ⚠️ A fixture não pode pousar NA descontinuidade — medido, não suposto
//!
//! A primeira versão deste gate nasceu **VERMELHA num modo que já shipava** há
//! meses (`value.quantize` Floor, `max |d| = 5e-1` — exactamente um degrau), e o
//! diagnóstico foi impresso pelo próprio gate: **`i = 575`, UM de 576**, o último
//! elemento da rampa, onde `t = 1` exactamente.
//!
//! O mecanismo: a rampa é `i/(N−1)`, e no device o último elemento sai **um ulp
//! abaixo de 1,0**. Isso é *legítimo* — o ADR-0126 diz que floats de GPU não são
//! bit-reprodutíveis, e todo gate de paridade deste repo o admite com ε. Mas
//! `floor` é **DESCONTÍNUA**: um ulp abaixo de `4,0` devolve `3`, não `4`, e o
//! degrau inteiro atravessa o ε que existia para absorver o ulp. *Uma tolerância
//! de VALOR não sobrevive a uma função que transforma valor em ÍNDICE.*
//!
//! ⚠️ **E a primeira tentativa de dodge falhou também, por erro MEU de
//! aritmética** — vale registar porque a regra geral só apareceu na segunda:
//! trocar o passo de `0,25` para `0,3` pôs a amostra `i = 460` (`t = 0,8`
//! exacto) em cima de outra fronteira. Eu tinha "provado" que `i = 287,5 +
//! 86,25k` nunca é inteiro **conferindo só `k` múltiplo de 4**; em `k = 2` dá
//! 460. *Um dodge escolhido por aritmética de cabeça é um dodge que o gate
//! reprova na corrida seguinte.*
//!
//! **A regra que ficou, e é estrutural em vez de por-fixture: os EXTREMOS do
//! alcance são sempre candidatos a fronteira** (`t = 0` e `t = 1` mapeiam
//! exactamente para `out_lo` e `out_hi`), então basta escolher um alcance cujos
//! extremos **não** sejam múltiplos do passo. Daí `[−1,1 , 1,1]` com passo
//! `0,25`: os extremos caem em `±4,4` e o interior não tem solução inteira.
//! O `Stepped` usa `steps = 3`, onde `t·4` só é inteiro em `i ∈ {0, 575}` e o
//! **segundo clamp** faz os dois lados aterrarem em `1,0` de qualquer modo.
//!
//! ⚠️ E é por isto que o gate irmão de `value.quantize` era verde: a fixture
//! dele dirige um **LFO**, cujos valores não pousam em fronteira nenhuma — ele
//! estava verde por sorte de fixture, não por prova.
//!
//! ⛔ **Não "conserte" isto alargando o ε.** O erro é de um degrau inteiro; um ε
//! que o admitisse deixaria de distinguir um modo do outro, que é o trabalho
//! deste gate. Medido depois da cura: pior delta **3,8e-6** nas doze linhas,
//! contra separações de modo de **1,1e-1 a 1,5e0**.
//!
//! `#[ignore]`: precisa de adapter real. Numa máquina de dev / na lane de GPU:
//!   cargo test -p ph2d-gpu-cook --test gpu_cpu_parity_arith --release -- --ignored --nocapture

use ph2d_gpu::GpuContext;
use ph2d_gpu_cook::CookClock;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::graph::{Edge, Graph, NodeId};

const DEFAULT_UV: [f32; 4] = [0.25, 0.25, 0.75, 0.75];
const DEFAULT_SIZE: [f32; 2] = [0.4, 0.4];
const PLAYHEAD: f64 = 0.37;
/// O orçamento herdado dos gates irmãos (ADR-0126: floats de GPU não são
/// bit-reprodutíveis entre vendedores, e o WGSL pode contrair `a*b + c`).
const EPS: f32 = 1e-4;
/// Quanto dois modos vizinhos têm de diferir para que a comparação signifique
/// alguma coisa. Muito acima de [`EPS`] **de propósito**: se um par diferisse por
/// pouco, o gate estaria a medir ruído e não a lei.
const MODE_GAP: f32 = 1e-2;

fn try_headless_gpu() -> Option<GpuContext> {
    use std::sync::OnceLock;
    static SHARED: OnceLock<Option<GpuContext>> = OnceLock::new();
    SHARED
        .get_or_init(|| GpuContext::new(GpuContext::default_instance(), None).ok())
        .clone()
}

/// O registry MÍNIMO — só os nós que as fixtures montam. Uma cópia da lista
/// inteira seria uma segunda tabela a envelhecer ao lado da do irmão.
fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_motion_grid::register(&mut reg).unwrap();
    ph2d_node_motion_drive::register(&mut reg).unwrap();
    ph2d_node_value_instance_field::register(&mut reg).unwrap();
    ph2d_node_value_lfo::register(&mut reg).unwrap();
    ph2d_node_value_map_range::register(&mut reg).unwrap();
    ph2d_node_value_math::register(&mut reg).unwrap();
    ph2d_node_value_quantize::register(&mut reg).unwrap();
    ph2d_node_value_step::register(&mut reg).unwrap();
    ph2d_node_value_mix::register(&mut reg).unwrap();
    ph2d_node_value_curve::register(&mut reg).unwrap();
    ph2d_node_value_switch::register(&mut reg).unwrap();
    ph2d_node_value_pattern::register(&mut reg).unwrap();
    reg
}

fn connect(g: &mut Graph, a: NodeId, b: NodeId) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, 0),
        delayed: false,
    })
    .unwrap();
}

fn connect_to(g: &mut Graph, a: NodeId, b: NodeId, port: u16) {
    g.connect(Edge {
        from: (a, 0),
        to: (b, port),
        delayed: false,
    })
    .unwrap();
}

/// Uma grade de 24² e a rampa `i/(N−1)` sobre ela.
fn grid_and_ramp(g: &mut Graph) -> (NodeId, NodeId) {
    let grid = g.add_node("motion.grid");
    g.set_param(grid, "rows", 24.0);
    g.set_param(grid, "cols", 24.0);
    g.set_param(grid, "gap_x", 0.35);
    g.set_param(grid, "gap_y", 0.25);
    let ramp = g.add_node("value.instance_field");
    g.set_param(ramp, "mode", 1.0); // Ramp: i/(N-1) em [0,1]
    connect(g, grid, ramp);
    (grid, ramp)
}

/// A rampa esticada para `[lo, hi]` — o `map_range` aqui é PLUMBING, com a
/// interpolação no default `Linear`, e é o que dá metade negativa às fixtures
/// que precisam dela.
fn stretch(g: &mut Graph, ramp: NodeId, lo: f32, hi: f32) -> NodeId {
    let mr = g.add_node("value.map_range");
    g.set_param(mr, "out_lo", lo);
    g.set_param(mr, "out_hi", hi);
    connect(g, ramp, mr);
    mr
}

/// Um campo CONSTANTE de comprimento 1 (o oscilador de amplitude zero), que o
/// broadcast `1→N` espalha — é o divisor dos dois módulos e o `b` das misturas.
fn constant(g: &mut Graph, v: f32) -> NodeId {
    let k = g.add_node("value.lfo");
    g.set_param(k, "amplitude", 0.0);
    g.set_param(k, "offset", v);
    k
}

/// O que cada caso constrói: devolve o nó de VALOR terminal.
type Build = fn(&mut Graph, NodeId, NodeId) -> NodeId;

struct Case {
    label: &'static str,
    build: Build,
    /// `true` ⇒ este caso tem de produzir um campo DIFERENTE do anterior (o seu
    /// controle). É esta metade que impede um kernel cego ao param de passar.
    differs_from_previous: bool,
}

fn math_mod(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let signed = stretch(g, ramp, -2.0, 2.0);
    let m = g.add_node("value.math");
    g.set_param(m, "op", 6.0); // Modulo (sinal do dividendo)
    connect(g, signed, m);
    let d = constant(g, 0.75);
    connect_to(g, d, m, 1);
    m
}

fn math_floored_mod(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let signed = stretch(g, ramp, -2.0, 2.0);
    let m = g.add_node("value.math");
    g.set_param(m, "op", 7.0); // Floored Modulo (sinal do divisor)
    connect(g, signed, m);
    let d = constant(g, 0.75);
    connect_to(g, d, m, 1);
    m
}

fn quantize_floor(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let signed = stretch(g, ramp, -1.1, 1.1);
    let q = g.add_node("value.quantize");
    g.set_param(q, "step", 0.25);
    g.set_param(q, "mode", 1.0); // Floor
    connect(g, signed, q);
    q
}

fn quantize_truncate(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let signed = stretch(g, ramp, -1.1, 1.1);
    let q = g.add_node("value.quantize");
    g.set_param(q, "step", 0.25);
    g.set_param(q, "mode", 3.0); // Truncate
    connect(g, signed, q);
    q
}

fn step_smooth(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let s = g.add_node("value.step");
    g.set_param(s, "threshold", 0.5);
    g.set_param(s, "width", 0.8);
    g.set_param(s, "mode", 1.0); // Smooth
    connect(g, ramp, s);
    s
}

fn step_smoother(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let s = g.add_node("value.step");
    g.set_param(s, "threshold", 0.5);
    g.set_param(s, "width", 0.8);
    g.set_param(s, "mode", 2.0); // Smoother
    connect(g, ramp, s);
    s
}

fn map_linear(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let mr = stretch(g, ramp, 0.0, 2.0);
    g.set_param(mr, "interpolation", 0.0);
    mr
}

fn map_stepped(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let mr = stretch(g, ramp, 0.0, 2.0);
    g.set_param(mr, "interpolation", 1.0);
    // 3 e nao 4: com 4 as fronteiras caem em `i` multiplo de 115, e o ulp do
    // ultimo elemento da rampa atravessa uma delas (ver o cabecalho).
    g.set_param(mr, "steps", 3.0);
    mr
}

fn map_smooth(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let mr = stretch(g, ramp, 0.0, 2.0);
    g.set_param(mr, "interpolation", 2.0);
    mr
}

fn map_smoother(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let mr = stretch(g, ramp, 0.0, 2.0);
    g.set_param(mr, "interpolation", 3.0);
    mr
}

/// `a` é a rampa (que CRUZA 0,5, então os dois ramos do Overlay correm), `b` é
/// constante, e o factor é 1 para o modo aparecer inteiro em vez de diluído.
fn mix_mode(g: &mut Graph, ramp: NodeId, mode: f32) -> NodeId {
    let m = g.add_node("value.mix");
    g.set_param(m, "factor", 1.0);
    g.set_param(m, "blend", mode);
    connect(g, ramp, m);
    let b = constant(g, 0.6);
    connect_to(g, b, m, 1);
    m
}

/// **As COMPARAÇÕES** — a rampa contra um limiar constante, dobrada numa máscara.
///
/// ⚠️ **A fixture é escolhida pela regra que o cabeçalho deste arquivo pagou:** uma
/// comparação é DESCONTÍNUA em valor, então um ulp de diferença no device vira um
/// degrau inteiro de máscara e atravessa qualquer ε de valor. A rampa é `[−1, 1]`
/// sobre 576 amostras (`t = i/575`) e o limiar é `0`, então o cruzamento cai em
/// `i = 287,5` — **entre** duas amostras, nunca em cima de uma. As bandas de
/// igualdade seguem a mesma regra: com `eps = 0,3` as fronteiras caem em `201,25`
/// e `373,75`, com `0,5` em `143,75` e `431,25`. ⚠️ `eps = 0,6` foi **descartado
/// por aritmética**: ele põe as fronteiras exatamente em `i = 115` e `i = 460`.
///
/// ⚠️ E o cruzamento é o que torna o par `Less`/`Greater` **não-vazio**: sem ele as
/// duas máscaras seriam constantes e iguais, e o controle passaria sobre um kernel
/// cego ao `op`.
fn math_compare(g: &mut Graph, ramp: NodeId, op: f32, eps: f32) -> NodeId {
    let signed = stretch(g, ramp, -1.0, 1.0);
    let m = g.add_node("value.math");
    g.set_param(m, "op", op);
    g.set_param(m, "epsilon", eps);
    connect(g, signed, m);
    let t = constant(g, 0.0);
    connect_to(g, t, m, 1);
    m
}

fn math_less(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    math_compare(g, ramp, 8.0, 0.0)
}

fn math_greater(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    math_compare(g, ramp, 10.0, 0.0)
}

fn math_equal_narrow(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    math_compare(g, ramp, 12.0, 0.3)
}

fn math_equal_wide(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    math_compare(g, ramp, 12.0, 0.5)
}

fn math_not_equal_wide(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    math_compare(g, ramp, 13.0, 0.5)
}

fn mix_screen(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    mix_mode(g, ramp, 4.0)
}

fn mix_overlay(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    mix_mode(g, ramp, 8.0)
}

// ─────────────────────────────────────────────────────────────────────────────
// A FASE, A INVERSÃO e a MISTURA — os knobs apendados em 2026-08-22 (folha 15).
//
// ⚠️ **Cada um entra em PAR: o nó sem o knob e o nó com ele.** É a única forma
// destas linhas provarem o que dizem provar — um kernel que não lesse o param
// novo produziria os dois membros do par IGUAIS, e `differs_from_previous`
// reprova exatamente isso. Um caso solitário só provaria que o shader compila.
// ─────────────────────────────────────────────────────────────────────────────

fn step_smoother_inverted(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let s = g.add_node("value.step");
    g.set_param(s, "threshold", 0.5);
    g.set_param(s, "width", 0.8);
    g.set_param(s, "mode", 2.0); // Smoother
    g.set_param(s, "invert", 1.0);
    connect(g, ramp, s);
    s
}

fn quantize_round(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let signed = stretch(g, ramp, -1.1, 1.1);
    let q = g.add_node("value.quantize");
    g.set_param(q, "step", 0.25);
    connect(g, signed, q);
    q
}

/// ⚠️ O offset é **meio degrau**, que é onde ele muda TODA amostra: um offset de
/// um degrau inteiro daria a mesma grade e o par nasceria vazio.
fn quantize_round_phased(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let q = quantize_round(g, _grid, ramp);
    g.set_param(q, "offset", 0.125);
    q
}

/// A rampa moldada por uma TENDA — o caminho da LUT no domínio de valor.
fn curve_tent(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let c = g.add_node("value.curve");
    g.set_text_param(c, "curve", "c1 0:0:L 0.5:1:L 1:0:L");
    connect(g, ramp, c);
    c
}

/// ⚠️ `factor = 0,5` e não `0`: em `0` a saída seria a ENTRADA, que é a rampa —
/// e a rampa também é o que alimenta o caso anterior, então o par continuaria
/// não-vazio por acidente em vez de por leitura do param.
fn curve_tent_half(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let c = curve_tent(g, _grid, ramp);
    g.set_param(c, "factor", 0.5);
    c
}

/// `Add` sobre valores que TRANSBORDAM — sem transbordo o clamp do resultado não
/// teria o que segurar e o par nasceria vazio.
fn mix_add_overflowing(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let m = g.add_node("value.mix");
    g.set_param(m, "factor", 1.0);
    g.set_param(m, "blend", 1.0); // Add
    connect(g, ramp, m);
    let b = constant(g, 0.9);
    connect_to(g, b, m, 1);
    m
}

fn mix_add_clamped(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let m = mix_add_overflowing(g, _grid, ramp);
    g.set_param(m, "clamp_result", 1.0);
    m
}

/// Um switch cujo `select` é a rampa esticada para `[0, 3]` — ele **atravessa**
/// os quatro índices, que é o que dá matéria ao crossfade.
fn switch_router(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let sel = stretch(g, ramp, 0.0, 3.0);
    let sw = g.add_node("value.switch");
    connect(g, sel, sw);
    for (k, v) in [(1u16, 0.0f32), (2, 1.0), (3, 2.0), (4, 3.0)] {
        let c = constant(g, v);
        connect_to(g, c, sw, k);
    }
    sw
}

fn switch_crossfader(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let sw = switch_router(g, _grid, ramp);
    g.set_param(sw, "blend", 1.0);
    sw
}

/// O padrão por índice — a grade dá a contagem, como o nó pede.
fn pattern_steps(g: &mut Graph, grid: NodeId, _ramp: NodeId) -> NodeId {
    let p = g.add_node("value.pattern");
    g.set_param(p, "steps", 3.0);
    g.set_param(p, "v0", 0.0);
    g.set_param(p, "v1", 1.0);
    g.set_param(p, "v2", 2.0);
    connect(g, grid, p);
    p
}

/// ⚠️ A fase é **fracionária de propósito**: é ela que cria o "entre dois slots"
/// que o `interp` resolve, e é o único sítio onde `Step` e `Linear` divergem.
fn pattern_phased_step(g: &mut Graph, grid: NodeId, ramp: NodeId) -> NodeId {
    let p = pattern_steps(g, grid, ramp);
    g.set_param(p, "offset", 0.5);
    p
}

fn pattern_phased_linear(g: &mut Graph, grid: NodeId, ramp: NodeId) -> NodeId {
    let p = pattern_phased_step(g, grid, ramp);
    g.set_param(p, "interp", 1.0);
    p
}

/// `a·b + c` com os TRÊS termos vivos — a terceira porta cozida no device.
fn math_multiply_add(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let m = g.add_node("value.math");
    g.set_param(m, "op", 14.0); // Multiply Add
    connect(g, ramp, m);
    let b = constant(g, 2.0);
    connect_to(g, b, m, 1);
    let c = constant(g, 0.25);
    connect_to(g, c, m, 2);
    m
}

/// O mesmo `a·b` SEM o `c` — o par que prova que a terceira porta é lida.
fn math_multiply_add_without_c(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let m = g.add_node("value.math");
    g.set_param(m, "op", 14.0);
    connect(g, ramp, m);
    let b = constant(g, 2.0);
    connect_to(g, b, m, 1);
    m
}

/// A rampa contra uma constante, dobrada por `Min` — a quina que a mistura
/// arredonda cai **dentro** da rampa, senão o par nasceria vazio.
fn math_smooth_min_hard(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let m = g.add_node("value.math");
    g.set_param(m, "op", 15.0); // Smooth Min, distance = 0 => Min duro
    connect(g, ramp, m);
    let b = constant(g, 0.5);
    connect_to(g, b, m, 1);
    m
}

fn math_smooth_min_soft(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let m = math_smooth_min_hard(g, _grid, ramp);
    g.set_param(m, "distance", 0.4);
    m
}

fn math_smooth_max_soft(g: &mut Graph, _grid: NodeId, ramp: NodeId) -> NodeId {
    let m = math_smooth_min_soft(g, _grid, ramp);
    g.set_param(m, "op", 16.0); // Smooth Max
    m
}

/// A LFO como campo (o `in` ligado à grade, para o device a cozer por elemento).
fn lfo_plain(g: &mut Graph, grid: NodeId, _ramp: NodeId) -> NodeId {
    let k = g.add_node("value.lfo");
    g.set_param(k, "period", 0.5);
    g.set_param(k, "phase_stagger", 0.01);
    connect(g, grid, k);
    k
}

/// ⚠️ A rampa é **maior que o playhead da fixture** (`0,37`), senão o envelope já
/// estaria cheio e o par nasceria vazio: o knob só se vê DENTRO da rampa.
fn lfo_fading_in(g: &mut Graph, grid: NodeId, ramp: NodeId) -> NodeId {
    let k = lfo_plain(g, grid, ramp);
    g.set_param(k, "fade_in", 2.0);
    k
}

static CASES: &[Case] = &[
    Case {
        label: "math Modulo (assinado)",
        build: math_mod,
        differs_from_previous: false,
    },
    Case {
        label: "math Floored Modulo (assinado)",
        build: math_floored_mod,
        differs_from_previous: true,
    },
    Case {
        label: "quantize Floor (assinado)",
        build: quantize_floor,
        differs_from_previous: false,
    },
    Case {
        label: "quantize Truncate (assinado)",
        build: quantize_truncate,
        differs_from_previous: true,
    },
    Case {
        label: "step Smooth",
        build: step_smooth,
        differs_from_previous: false,
    },
    Case {
        label: "step Smoother",
        build: step_smoother,
        differs_from_previous: true,
    },
    Case {
        label: "map_range Linear",
        build: map_linear,
        differs_from_previous: false,
    },
    Case {
        label: "map_range Stepped",
        build: map_stepped,
        differs_from_previous: true,
    },
    Case {
        label: "map_range Smooth",
        build: map_smooth,
        differs_from_previous: true,
    },
    Case {
        label: "map_range Smoother",
        build: map_smoother,
        differs_from_previous: true,
    },
    Case {
        label: "mix Screen",
        build: mix_screen,
        differs_from_previous: false,
    },
    Case {
        label: "mix Overlay",
        build: mix_overlay,
        differs_from_previous: true,
    },
    Case {
        label: "math Less (limiar 0)",
        build: math_less,
        differs_from_previous: false,
    },
    // O par que prova que o device LÊ o `op`: mesma entrada, máscaras opostas.
    Case {
        label: "math Greater (limiar 0)",
        build: math_greater,
        differs_from_previous: true,
    },
    Case {
        label: "math Equal (eps 0,3)",
        build: math_equal_narrow,
        differs_from_previous: true,
    },
    // ⚠️ **O par que prova que o device lê o `epsilon`**: MESMO op, MESMA entrada,
    // só a tolerância difere. Um kernel que ignorasse `params.epsilon` passaria em
    // todas as outras linhas desta tabela e falha nesta.
    Case {
        label: "math Equal (eps 0,5)",
        build: math_equal_wide,
        differs_from_previous: true,
    },
    Case {
        label: "math Not Equal (eps 0,5)",
        build: math_not_equal_wide,
        differs_from_previous: true,
    },
    // ── Os knobs apendados em 2026-08-22 (folha 15), cada um com o seu par ──
    Case {
        label: "step Smoother (o controle do invert)",
        build: step_smoother,
        differs_from_previous: true,
    },
    Case {
        label: "step Smoother INVERTIDO",
        build: step_smoother_inverted,
        differs_from_previous: true,
    },
    Case {
        label: "quantize Round (o controle da fase)",
        build: quantize_round,
        differs_from_previous: true,
    },
    Case {
        label: "quantize Round com FASE de meio degrau",
        build: quantize_round_phased,
        differs_from_previous: true,
    },
    Case {
        label: "mix Add transbordando (o controle do clamp)",
        build: mix_add_overflowing,
        differs_from_previous: true,
    },
    Case {
        label: "mix Add com Clamp Result",
        build: mix_add_clamped,
        differs_from_previous: true,
    },
    Case {
        label: "switch roteador (o controle do blend)",
        build: switch_router,
        differs_from_previous: true,
    },
    Case {
        label: "switch CROSSFADER",
        build: switch_crossfader,
        differs_from_previous: true,
    },
    Case {
        label: "pattern com fase de meio slot (Step)",
        build: pattern_phased_step,
        differs_from_previous: true,
    },
    Case {
        label: "pattern com a MESMA fase, Linear",
        build: pattern_phased_linear,
        differs_from_previous: true,
    },
    Case {
        label: "math Multiply Add SEM o c (o controle da 3ª porta)",
        build: math_multiply_add_without_c,
        differs_from_previous: true,
    },
    Case {
        label: "math Multiply Add COM o c",
        build: math_multiply_add,
        differs_from_previous: true,
    },
    Case {
        label: "math Smooth Min a distance 0 (== Min duro)",
        build: math_smooth_min_hard,
        differs_from_previous: true,
    },
    Case {
        label: "math Smooth Min a distance 0,4",
        build: math_smooth_min_soft,
        differs_from_previous: true,
    },
    Case {
        label: "math Smooth MAX a distance 0,4",
        build: math_smooth_max_soft,
        differs_from_previous: true,
    },
    Case {
        label: "lfo sem rampa (o controle do fade_in)",
        build: lfo_plain,
        differs_from_previous: true,
    },
    Case {
        label: "lfo com fade_in de 2 s",
        build: lfo_fading_in,
        differs_from_previous: true,
    },
];

fn worst_delta(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "contagem de elementos");
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0, f32::max)
}

/// Coze UM caso nos dois caminhos e devolve `(cpu_y, gpu_y)` — a plumbing que o
/// laço da tabela e o gate da LUT partilham.
///
/// O valor dirige Y para que ele atravesse o MESMO lowering que o produto usa, e
/// não só a coluna solta.
fn cook_on_both(
    gpu: &GpuContext,
    reg: &NodeRegistry,
    build: Build,
    label: &str,
) -> (Vec<f32>, Vec<f32>) {
    let mut g = Graph::new();
    let (grid, ramp) = grid_and_ramp(&mut g);
    let value = build(&mut g, grid, ramp);
    let drive = g.add_node("motion.drive");
    g.set_param(drive, "channel", 1.0); // Y
    g.set_param(drive, "mode", 0.0); // Add
    g.set_param(drive, "scale", 2.0);
    connect(&mut g, grid, drive);
    connect_to(&mut g, value, drive, 1);

    g.validate(reg).unwrap_or_else(|e| panic!("{label}: {e:?}"));
    let plan = ph2d_gpu_cook::plan(&g, reg, reg, drive);
    assert!(
        plan.is_fully_gpu(),
        "{label}: a cadeia tem de ser reivindicada de ponta a ponta"
    );

    let mut cook = Cook::new();
    let cpu = cook
        .cook(&g, reg, drive, PLAYHEAD)
        .unwrap_or_else(|e| panic!("{label}: cpu cook {e:?}"));
    let mut gc = ph2d_gpu_cook::GpuCook::new();
    gc.retain_streams_for_debug(true);
    gc.cook(
        gpu,
        &g,
        reg,
        reg,
        &plan,
        &[],
        CookClock::at(PLAYHEAD),
        DEFAULT_UV,
        DEFAULT_SIZE,
        0,
    )
    .unwrap_or_else(|e| panic!("{label}: gpu cook {e:?}"));

    let cpu_p: Vec<f32> = match cpu[0].as_stream().get("P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).collect(),
        _ => panic!("{label}: sem coluna P"),
    };
    let gpu_p = gc
        .read_column_vec2(gpu, drive, "P")
        .unwrap_or_else(|| panic!("{label}: P não volta do device"));
    (cpu_p, gpu_p.iter().map(|p| p[1]).collect())
}

/// A ε da LUT — **medida, não escolhida**, e MUITO mais larga que a [`EPS`] da
/// tabela acima.
///
/// A curva viaja para o device como uma tabela de 256 amostras com lerp, e uma
/// tenda tem uma QUINA no pico: a tabela corta-a por cerca de um passo de
/// amostra. Medido nesta fixture: `4,4e-3` sobre um Y que o `drive` amplifica 2×.
/// É o mesmo compromisso que o gêmeo `field.remap` documenta há meses, e a razão
/// de este caso viver fora da tabela em vez de afrouxar a barra dela — *uma barra
/// larga aplicada a todos esconde a divergência de um*.
const LUT_EPS: f32 = 1e-2;

/// **O `factor` do `value.curve` é LIDO PELO DEVICE** — dentro da ε da LUT, e com
/// o controle que prova que a fixture contém a diferença.
///
/// ⚠️ **`factor = 0,5` e não `0`:** em `0` a saída seria a ENTRADA (a rampa), que
/// já é o que alimenta metade desta suíte — o par continuaria não-vazio por
/// acidente em vez de por leitura do param.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_curve_factor_is_read_on_the_device_within_the_lut_epsilon() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let (cpu_full, gpu_full) = cook_on_both(&gpu, &reg, curve_tent, "curve tenda");
    let (cpu_half, gpu_half) = cook_on_both(&gpu, &reg, curve_tent_half, "curve tenda a meio");
    for (label, cpu, gpu_y) in [
        ("curve tenda", &cpu_full, &gpu_full),
        ("curve tenda a meio", &cpu_half, &gpu_half),
    ] {
        let worst = worst_delta(cpu, gpu_y);
        eprintln!("{label:<34} max |d| = {worst:e} (ε da LUT)");
        assert!(worst < LUT_EPS, "{label}: max |d| = {worst:e}");
    }
    // ⚠️ O CONTROLE: as duas metades TÊM de diferir, e por muito mais que a ε da
    // LUT — senão isto estaria a medir um kernel cego ao `factor`.
    let gap = worst_delta(&cpu_full, &cpu_half);
    eprintln!("{:<34}   contra o controle: {gap:e}", "");
    assert!(gap > MODE_GAP, "o factor nao muda nada ({gap:e})");
    // E o device vê a MESMA diferença que a CPU vê — não só o mesmo valor.
    let gap_gpu = worst_delta(&gpu_full, &gpu_half);
    assert!(
        (gap - gap_gpu).abs() < LUT_EPS,
        "a diferenca do device ({gap_gpu:e}) nao e' a da CPU ({gap:e})"
    );
}

/// **Os doze modos da aritmética de valor, no device, cada um com o seu
/// controle.** Ver o cabeçalho para o porquê da forma.
#[test]
#[ignore = "requires a GPU adapter; run with --ignored on a dev machine"]
fn the_new_arithmetic_modes_match_the_cpu_on_the_device() {
    let Some(gpu) = try_headless_gpu() else {
        eprintln!("no GPU adapter — skipping");
        return;
    };
    let reg = registry();
    let mut previous: Option<Vec<f32>> = None;

    for case in CASES {
        let (cpu_p, gpu_y) = cook_on_both(&gpu, &reg, case.build, case.label);
        let worst = worst_delta(&cpu_p, &gpu_y);
        eprintln!("{:<34} max |d| = {worst:e}", case.label);
        if worst >= EPS {
            let n_bad = cpu_p
                .iter()
                .zip(&gpu_y)
                .filter(|(a, b)| (*a - *b).abs() >= EPS)
                .count();
            for (i, (a, b)) in cpu_p.iter().zip(&gpu_y).enumerate() {
                if (a - b).abs() >= EPS {
                    eprintln!(
                        "    i={i} cpu={a:.9} gpu={b:.9}  ({n_bad} de {})",
                        cpu_p.len()
                    );
                    break;
                }
            }
        }
        assert!(worst < EPS, "{}: max |d| = {worst:e}", case.label);

        // A fixture CONTÉM o fenômeno: um campo constante concordaria com
        // qualquer kernel.
        let spread = cpu_p.iter().fold(f32::NEG_INFINITY, |m, v| m.max(*v))
            - cpu_p.iter().fold(f32::INFINITY, |m, v| m.min(*v));
        assert!(
            spread > MODE_GAP,
            "{}: o campo é chato ({spread:e}) — a fixture não exercita o modo",
            case.label
        );

        if case.differs_from_previous {
            let prev = previous.as_ref().expect("um controle precede cada par");
            let gap = worst_delta(&cpu_p, prev);
            eprintln!("{:<34}   contra o controle: {gap:e}", "");
            assert!(
                gap > MODE_GAP,
                "{}: NÃO se distingue do modo anterior ({gap:e}) — ou o param é \
                 ignorado, ou a fixture não contém a diferença",
                case.label
            );
        }
        previous = Some(cpu_p);
    }
}
