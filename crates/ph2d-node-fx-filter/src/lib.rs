//! **`fx.filter` — UM FILTRO DA CAMADA do Motion** (ciclo 20 W3, [doc 129](../../../docs/Motion%20Nodes/129_ciclo_20_o_acabamento.md)
//! §3.3, o `fx.adjust` do [doc 94](../../../docs/Motion%20Nodes/94_plano_fx_adjust.md) §1): blur,
//! sombra, brilho, contorno, turbulência, ajuste de cor… — os 15 tipos da `ph2d_fx_op::FxOp`, a
//! mesma tabela que o painel vetorial autora. *Uma lei, dois autores.*
//!
//! Nó passthrough (`out == in`, `Effect::Pure`) com a forma do `fx.glow`: ele DECLARA um degrau, e o
//! passe da placa (`ph2d-app-motion::motion_filter`) corre a pilha sobre a camada do Motion desenhada
//! à parte. ⛔ **NÃO é singleton:** os `fx.filter` no fio de uma `Output`, na ordem do fio, SÃO a
//! pilha ([`from_graph`]). Sem nenhum no fio o quadro é byte-idêntico.
//!
//! ⭐ **A ficha é DERIVADA da `FxOp::SPECS`** em tempo de compilação — cada knob aparece só para os
//! tipos que o declaram (um [`ParamGate`] por knob; a lista tem o comprimento CONTADO na `SPECS`,
//! por genérico `const`, e por isso nenhum sentinela). Os nomes dos tipos e dos modos são chaves do
//! nó, e o gate confere cada palavra contra a tabela.
//!
//! ⚠️ Fora da ficha, declarado: a lei de MISTURA (`blend` fica Normal — os rótulos dela só a shell
//! alcança) e a rampa do Gradient Map tem DOIS stops (as duas cores), não os oito da `FxOp`.

#![forbid(unsafe_code)]

use ph2d_fx_op::FxOp;
use ph2d_node_registry::{
    NodeRegistry, NodeSilhouette, NodeUiCategory, NodeUiManifest, ParamGate, ParamHardMax,
    ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget, RegistryError,
};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::graph::{Graph, NodeId};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};
use std::collections::{BTreeMap, BTreeSet};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// O tipo canónico — quem varre o grafo procura este nome.
pub const TYPE_NAME: &str = "fx.filter";

/// O nó terminal cujo fio define a pilha.
const OUTPUT: &str = "motion.output";

/// **A régua de COMPRIMENTO do nó, em unidades de MUNDO** — raio, deslocamento, tamanho do ruído e
/// crescimento partilham-na (a mesma grandeza, a mesma régua, como no painel vetorial).
///
/// É o alcance do ARRASTO (o máximo suave): até ao ciclo 20 W3 era o raio em que o passe saturava
/// na janela medida (`σ = 32` px a `53,4` px/unidade), e numa janela 4K a mesma régua saturava em
/// ~0,27 (doc 129 §5.5 A8). Com a pirâmide o borrão honra qualquer σ até ao
/// `ph2d_render::fx_stack::SIGMA_ALCANCAVEL`, e o valor DIGITADO vai até [`COMPRIMENTO_DIGITADO`].
pub const COMPRIMENTO_MAX: f32 = 0.6;

/// **O tecto do valor digitado** — a altura que a câmera do produto mostra (`height_world = 10`):
/// um raio maior que a vista inteira já só tinge a camada por igual. O passe honra-o em qualquer
/// janela e zoom (gate `the_typed_length_ceiling_is_honoured_at_8k_and_max_zoom`).
pub const COMPRIMENTO_DIGITADO: f32 = 10.0;

/// Os nomes dos params (a ordem do manifesto é a ordem das linhas do cartão).
pub mod param {
    pub const KIND: &str = "kind";
    pub const SCOPE: &str = "scope";
    pub const RADIUS: &str = "radius";
    pub const OFFSET_X: &str = "offset_x";
    pub const OFFSET_Y: &str = "offset_y";
    pub const COLOR: [&str; 4] = ["color_r", "color_g", "color_b", "color_a"];
    pub const COLOR2: [&str; 4] = ["color2_r", "color2_g", "color2_b", "color2_a"];
    pub const OPACITY: &str = "opacity";
    pub const FALLOFF: &str = "falloff";
    pub const NOISE_MODE: &str = "noise_mode";
    pub const RAMP_MODE: &str = "ramp_mode";
    pub const SIZE: &str = "size";
    pub const DETAIL: &str = "detail";
    pub const SEED: &str = "seed";
    pub const GROW: &str = "grow";
    pub const HUE: &str = "hue";
    pub const SATURATION: &str = "saturation";
    pub const BRIGHTNESS: &str = "brightness";
}

const fn p(name: &'static str, default: f32) -> ParamSpec {
    ParamSpec { name, default }
}

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("fx.filter"),
    name: "fx.filter",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // Os defaults: o `FxOp::new(BLUR)` (gate `the_dropped_filter_is_the_vectors_blur`), cada forma
    // por si, e os knobs dos outros tipos nos valores da `FxOp::new` deles — o que o tipo lê ANTES de
    // um gesto do cartão (o [`switch_kind`] escreve-os ao trocar de tipo).
    params: &[
        p(param::KIND, 0.0),
        p(param::SCOPE, 0.0),
        p(param::RADIUS, 0.12),
        p(param::OFFSET_X, 0.12),
        p(param::OFFSET_Y, -0.12),
        p(param::COLOR[0], 0.0),
        p(param::COLOR[1], 0.0),
        p(param::COLOR[2], 0.0),
        p(param::COLOR[3], 1.0),
        p(param::COLOR2[0], 1.0),
        p(param::COLOR2[1], 1.0),
        p(param::COLOR2[2], 1.0),
        p(param::COLOR2[3], 1.0),
        p(param::OPACITY, 1.0),
        p(param::FALLOFF, 0.0),
        p(param::NOISE_MODE, 0.0),
        p(param::RAMP_MODE, 0.0),
        p(param::SIZE, 0.25),
        p(param::DETAIL, 3.0),
        p(param::SEED, 0.0),
        p(param::GROW, 0.06),
        p(param::HUE, 0.0),
        p(param::SATURATION, 0.0),
        p(param::BRIGHTNESS, 0.0),
    ],
    effect: Effect::Pure,
    clock: Clock::Frame,
    lowerings: &[LoweringKind::Cpu],
};

/// Os 15 tipos, na ordem dos códigos. ⚠️ O comprimento é `FxOp::KINDS`: um tipo novo não compila
/// até ganhar a sua chave, e o gate confere a palavra contra a `SPECS`.
pub static KIND_LABELS: [&str; FxOp::KINDS] = [
    "node.opts.node_fx_filter.kind_labels.0",
    "node.opts.node_fx_filter.kind_labels.1",
    "node.opts.node_fx_filter.kind_labels.2",
    "node.opts.node_fx_filter.kind_labels.3",
    "node.opts.node_fx_filter.kind_labels.4",
    "node.opts.node_fx_filter.kind_labels.5",
    "node.opts.node_fx_filter.kind_labels.6",
    "node.opts.node_fx_filter.kind_labels.7",
    "node.opts.node_fx_filter.kind_labels.8",
    "node.opts.node_fx_filter.kind_labels.9",
    "node.opts.node_fx_filter.kind_labels.10",
    "node.opts.node_fx_filter.kind_labels.11",
    "node.opts.node_fx_filter.kind_labels.12",
    "node.opts.node_fx_filter.kind_labels.13",
    "node.opts.node_fx_filter.kind_labels.14",
];

/// Os modos de QUEDA (Glow, Inner Shadow, Inner Glow), na ordem dos códigos.
pub static FALLOFF_LABELS: [&str; 2] = [
    "node.opts.node_fx_filter.falloff_labels.0",
    "node.opts.node_fx_filter.falloff_labels.1",
];
/// Os modos do RUÍDO (Turbulence).
pub static NOISE_LABELS: [&str; 2] = [
    "node.opts.node_fx_filter.noise_labels.0",
    "node.opts.node_fx_filter.noise_labels.1",
];
/// Os modos da RAMPA (Gradient Map).
pub static RAMP_LABELS: [&str; 2] = [
    "node.opts.node_fx_filter.ramp_labels.0",
    "node.opts.node_fx_filter.ramp_labels.1",
];

/// Onde o degrau corre ([`Scope`]), na ordem dos códigos.
pub static SCOPE_LABELS: [&str; 2] = [
    "node.opts.node_fx_filter.scope_labels.0",
    "node.opts.node_fx_filter.scope_labels.1",
];

/// **Onde um degrau corre** (doc 129 §3.4, decisão do dono 10/10) — o idioma do Illustrator: um
/// efeito em CADA objecto corre por objecto; no grupo, corre na união.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Cada instância do Motion filtrada sozinha (a sombra interna, o bevel, o contorno veem só a
    /// SUA forma) e devolvida no seu lugar, na ordem de z.
    EachShape,
    /// A camada inteira (a união das formas) — o estilo de camada do After Effects.
    WholeLayer,
}

impl Scope {
    fn of(code: f32) -> Self {
        if code.round() >= 1.0 {
            Self::WholeLayer
        } else {
            Self::EachShape
        }
    }
}

/// Os três params de modo e a lista de modos que cada um escolhe — a lista é a do tipo que a
/// declara na `SPECS`, e o gate de cada param são os tipos com a MESMA lista.
pub const MODE_PARAMS: [(&str, &[&str]); 3] = [
    (param::FALLOFF, FxOp::SPECS[FxOp::GLOW as usize].modes),
    (
        param::NOISE_MODE,
        FxOp::SPECS[FxOp::TURBULENCE as usize].modes,
    ),
    (
        param::RAMP_MODE,
        FxOp::SPECS[FxOp::GRADIENT_MAP as usize].modes,
    ),
];

// ── A ficha derivada da `SPECS` ──────────────────────────────────────────────────────────────

/// Um knob da ficha, como a `SPECS` o declara.
#[derive(Clone, Copy)]
pub enum Knob {
    Radius,
    Offset,
    /// A 1.ª cor — e a 1.ª ponta da rampa.
    Color,
    /// A 2.ª cor — e a 2.ª ponta da rampa.
    Color2,
    Noise,
    Grow,
    Adjust,
}

/// O tipo `kind` lê o knob?
#[must_use]
pub const fn takes(kind: usize, knob: Knob) -> bool {
    let s = &FxOp::SPECS[kind];
    match knob {
        Knob::Radius => s.radius_label.is_some(),
        Knob::Offset => s.offset_labels.is_some(),
        Knob::Color => s.color_label.is_some() || s.takes_ramp,
        Knob::Color2 => s.color_b_label.is_some() || s.takes_ramp,
        Knob::Noise => s.noise_labels.is_some(),
        Knob::Grow => s.grow_label.is_some(),
        Knob::Adjust => s.adjust_labels.is_some(),
    }
}

const fn sim(k: usize, knob: Option<Knob>, modes: &[&str]) -> bool {
    match knob {
        Some(knob) => takes(k, knob),
        None => same_list(FxOp::SPECS[k].modes, modes),
    }
}

/// Quantos tipos leem o knob (ou têm a lista de modos `modes`).
const fn count(knob: Option<Knob>, modes: &[&str]) -> usize {
    let (mut n, mut k) = (0, 0);
    while k < FxOp::KINDS {
        if sim(k, knob, modes) {
            n += 1;
        }
        k += 1;
    }
    n
}

/// Os tipos que leem o knob, compactos — `N` é o [`count`] (um erro de compilação se divergir).
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    reason = "k < FxOp::KINDS = 15"
)]
const fn kinds_where<const N: usize>(knob: Option<Knob>, modes: &[&str]) -> [i32; N] {
    let mut out = [0; N];
    let (mut j, mut k) = (0, 0);
    while k < FxOp::KINDS {
        if sim(k, knob, modes) {
            out[j] = k as i32;
            j += 1;
        }
        k += 1;
    }
    assert!(j == N, "o comprimento do gate diverge da SPECS");
    out
}

const fn same_str(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

const fn same_list(a: &[&str], b: &[&str]) -> bool {
    if a.len() != b.len() || a.is_empty() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if !same_str(a[i], b[i]) {
            return false;
        }
        i += 1;
    }
    true
}

const RADIUS_KINDS: [i32; count(Some(Knob::Radius), &[])] = kinds_where(Some(Knob::Radius), &[]);
const OFFSET_KINDS: [i32; count(Some(Knob::Offset), &[])] = kinds_where(Some(Knob::Offset), &[]);
const COLOR_KINDS: [i32; count(Some(Knob::Color), &[])] = kinds_where(Some(Knob::Color), &[]);
const COLOR2_KINDS: [i32; count(Some(Knob::Color2), &[])] = kinds_where(Some(Knob::Color2), &[]);
const NOISE_KINDS: [i32; count(Some(Knob::Noise), &[])] = kinds_where(Some(Knob::Noise), &[]);
const GROW_KINDS: [i32; count(Some(Knob::Grow), &[])] = kinds_where(Some(Knob::Grow), &[]);
const ADJUST_KINDS: [i32; count(Some(Knob::Adjust), &[])] = kinds_where(Some(Knob::Adjust), &[]);
const FALLOFF_KINDS: [i32; count(None, MODE_PARAMS[0].1)] = kinds_where(None, MODE_PARAMS[0].1);
const NOISE_MODE_KINDS: [i32; count(None, MODE_PARAMS[1].1)] = kinds_where(None, MODE_PARAMS[1].1);
const RAMP_MODE_KINDS: [i32; count(None, MODE_PARAMS[2].1)] = kinds_where(None, MODE_PARAMS[2].1);

const fn gate(param: &'static str, values: &'static [i32]) -> ParamGate {
    ParamGate {
        param,
        when: param::KIND,
        values,
    }
}

/// Um gate por knob, todos sobre o `kind`. A opacidade não tem gate: todo tipo a lê.
pub static PARAM_GATES: &[ParamGate] = &[
    gate(param::RADIUS, &RADIUS_KINDS),
    gate(param::OFFSET_X, &OFFSET_KINDS),
    gate(param::OFFSET_Y, &OFFSET_KINDS),
    gate(param::COLOR[0], &COLOR_KINDS),
    gate(param::COLOR2[0], &COLOR2_KINDS),
    gate(param::FALLOFF, &FALLOFF_KINDS),
    gate(param::NOISE_MODE, &NOISE_MODE_KINDS),
    gate(param::RAMP_MODE, &RAMP_MODE_KINDS),
    gate(param::SIZE, &NOISE_KINDS),
    gate(param::DETAIL, &NOISE_KINDS),
    gate(param::SEED, &NOISE_KINDS),
    gate(param::GROW, &GROW_KINDS),
    gate(param::HUE, &ADJUST_KINDS),
    gate(param::SATURATION, &ADJUST_KINDS),
    gate(param::BRIGHTNESS, &ADJUST_KINDS),
];

// ── O grafo lido ─────────────────────────────────────────────────────────────────────────────

/// Um degrau da pilha: o nó e o filtro que ele declara.
#[derive(Clone, Debug, PartialEq)]
pub struct NoDoFiltro {
    pub no: NodeId,
    pub op: FxOp,
    pub scope: Scope,
}

/// ⭐ **A pilha partida em DOIS estágios pela ordem do fio** — porta única de *«que degraus correm
/// por forma?»*: o prefixo `Each Shape` corre em cada forma; do primeiro `Whole Layer` em diante,
/// tudo corre na camada (as formas já foram juntas).
#[must_use]
pub fn partition(pilha: &[NoDoFiltro]) -> (&[NoDoFiltro], &[NoDoFiltro]) {
    let n = pilha
        .iter()
        .position(|d| d.scope == Scope::WholeLayer)
        .unwrap_or(pilha.len());
    pilha.split_at(n)
}

/// Os `Each Shape` DEPOIS de um `Whole Layer`: já não veem as formas, correm na camada — e o cartão
/// di-lo.
#[must_use]
pub fn na_camada_apesar_do_scope(pilha: &[NoDoFiltro]) -> Vec<NodeId> {
    partition(pilha)
        .1
        .iter()
        .filter(|d| d.scope == Scope::EachShape)
        .map(|d| d.no)
        .collect()
}

fn default_of(name: &str) -> f32 {
    MANIFEST
        .params
        .iter()
        .find(|s| s.name == name)
        .map_or(0.0, |s| s.default)
}

fn read(ov: Option<&BTreeMap<String, f32>>, name: &str) -> f32 {
    ov.and_then(|m| m.get(name))
        .copied()
        .unwrap_or_else(|| default_of(name))
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "clampado à faixa do u8 antes do cast"
)]
fn read_u8(ov: Option<&BTreeMap<String, f32>>, name: &str, max: u8) -> u8 {
    read(ov, name).round().clamp(0.0, f32::from(max)) as u8
}

/// ⚠️ **O cartão guarda a cor em RGBA LINEAR** (o seletor do Motion, `motion_bridge_color`); a
/// `FxOp` é sRGB RETO (o shader lineariza-a). A fronteira é AQUI, nos dois sentidos — sem ela toda
/// cor intermédia era linearizada duas vezes (doc 129 §5.5 A4: um cinzento sRGB 128 chegava a 55).
fn read_color(ov: Option<&BTreeMap<String, f32>>, names: [&str; 4]) -> [f32; 4] {
    let [r, g, b, a] = names.map(|n| read(ov, n));
    let srgb = [r, g, b].map(ph2d_color::srgb::linear_to_srgb_unit);
    [srgb[0], srgb[1], srgb[2], a]
}

/// A inversa de [`read_color`]: a cor sRGB reta da `FxOp` como o cartão a guarda.
fn card_color(c: [f32; 4]) -> [f32; 4] {
    let lin = [c[0], c[1], c[2]].map(ph2d_color::srgb::srgb_to_linear_unit);
    [lin[0], lin[1], lin[2], c[3]]
}

/// O param de modo que o tipo `kind` lê, se algum.
fn mode_param_of(kind: usize) -> Option<(&'static str, &'static [&'static str])> {
    let modes = FxOp::SPECS[kind].modes;
    MODE_PARAMS
        .iter()
        .find(|(_, lista)| same_list(lista, modes))
        .copied()
}

#[expect(clippy::cast_possible_truncation, reason = "KINDS = 15 cabe num u8")]
fn kind_of(ov: Option<&BTreeMap<String, f32>>) -> u8 {
    read_u8(ov, param::KIND, (FxOp::KINDS - 1) as u8)
}

/// **O degrau que o nó `no` declara** — o `FxOp::new(tipo)` do Vector (a mesma lei, doc 129 §5.5
/// A2/A5), com os knobs que o TIPO lê tirados do cartão. Um knob que o tipo não lê fica no default
/// do tipo, nunca no valor escondido de outro.
#[must_use]
pub fn op_of(graph: &Graph, no: NodeId) -> FxOp {
    let ov = graph.node_param_overrides(no);
    let kind = kind_of(ov);
    let k = usize::from(kind);
    let mut op = FxOp::new(kind);
    op.opacity = read(ov, param::OPACITY).clamp(0.0, 1.0);
    if takes(k, Knob::Radius) {
        op.radius = read(ov, param::RADIUS).max(0.0);
    }
    if takes(k, Knob::Offset) {
        op.offset = [read(ov, param::OFFSET_X), read(ov, param::OFFSET_Y)];
    }
    if takes(k, Knob::Color) {
        op.color = read_color(ov, param::COLOR);
    }
    if takes(k, Knob::Color2) {
        op.color_b = read_color(ov, param::COLOR2);
    }
    if let Some((nome, lista)) = mode_param_of(k) {
        #[expect(clippy::cast_possible_truncation, reason = "listas de 2 modos")]
        let max = (lista.len() - 1) as u8;
        op.mode = read_u8(ov, nome, max);
    }
    if takes(k, Knob::Noise) {
        op.scale = read(ov, param::SIZE).max(0.0);
        op.detail = read_u8(ov, param::DETAIL, FxOp::MAX_DETAIL);
        op.seed = read_u8(ov, param::SEED, u8::MAX);
    }
    if takes(k, Knob::Grow) {
        op.grow = read(ov, param::GROW);
    }
    if takes(k, Knob::Adjust) {
        // O cartão fala em GRAUS; a `FxOp` guarda a matiz em VOLTAS.
        op.hue = read(ov, param::HUE) / 360.0;
        op.sat = read(ov, param::SATURATION);
        op.bright = read(ov, param::BRIGHTNESS);
    }
    if FxOp::SPECS[k].takes_ramp {
        op.stops[0] = op.color;
        op.stops[1] = op.color_b;
        op.stop_pos[0] = 0.0;
        op.stop_pos[1] = 1.0;
        op.stop_count = 2;
    }
    op.enabled = true;
    op.blend = FxOp::BLEND_NORMAL;
    op
}

/// **O que o cartão mostra para o degrau `op`** — a inversa do [`op_of`], só nos knobs que o tipo
/// LÊ (com a opacidade, que todos leem).
#[must_use]
pub fn card_values(op: &FxOp) -> Vec<(&'static str, f32)> {
    let k = usize::from(op.kind);
    let mut v = vec![(param::OPACITY, op.opacity)];
    if takes(k, Knob::Radius) {
        v.push((param::RADIUS, op.radius));
    }
    if takes(k, Knob::Offset) {
        v.extend([(param::OFFSET_X, op.offset[0]), (param::OFFSET_Y, op.offset[1])]);
    }
    let ramp = FxOp::SPECS[k].takes_ramp;
    if takes(k, Knob::Color) {
        let c = card_color(if ramp { op.stops[0] } else { op.color });
        v.extend(param::COLOR.into_iter().zip(c));
    }
    if takes(k, Knob::Color2) {
        let c = card_color(if ramp { op.stops[1] } else { op.color_b });
        v.extend(param::COLOR2.into_iter().zip(c));
    }
    if let Some((nome, _)) = mode_param_of(k) {
        v.push((nome, f32::from(op.mode)));
    }
    if takes(k, Knob::Noise) {
        v.extend([
            (param::SIZE, op.scale),
            (param::DETAIL, f32::from(op.detail)),
            (param::SEED, f32::from(op.seed)),
        ]);
    }
    if takes(k, Knob::Grow) {
        v.push((param::GROW, op.grow));
    }
    if takes(k, Knob::Adjust) {
        v.extend([
            (param::HUE, op.hue * 360.0),
            (param::SATURATION, op.sat),
            (param::BRIGHTNESS, op.bright),
        ]);
    }
    v
}

/// ⭐ **O gesto de TROCAR DE TIPO** (doc 129 §5.5 A2/A5) — o cartão e as cenas passam por aqui, no
/// MESMO passo de undo do `kind` (o precedente do `apply_channel_presets`). Cada knob que o tipo
/// NOVO lê recebe o `FxOp::new(novo)` se ainda vale o default do tipo ANTERIOR (ou se o anterior nem
/// o lia); o que o artista mexeu fica. Assim o Glow nasce branco e em Proximity, como no Vector.
pub fn switch_kind(graph: &mut Graph, no: NodeId, novo: u8) {
    let ov = graph.node_param_overrides(no);
    let velho = kind_of(ov);
    if velho == novo {
        return;
    }
    let antes = card_values(&FxOp::new(velho));
    let escritas: Vec<(&str, f32)> = card_values(&FxOp::new(novo))
        .into_iter()
        .filter(|(nome, _)| {
            antes
                .iter()
                .find(|(n, _)| n == nome)
                .is_none_or(|(_, d)| read(ov, nome).to_bits() == d.to_bits())
        })
        .collect();
    graph.set_param(no, param::KIND, f32::from(novo));
    for (nome, valor) in escritas {
        graph.set_param(no, nome, valor);
    }
}

/// **A pilha do quadro**: os `fx.filter` no fio principal (porta 0) de cada `Output`, em ordem de id
/// das saídas, cada fio de montante para jusante — a ordem em que o artista os encadeou. Vazia sem
/// nenhum no fio.
#[must_use]
pub fn from_graph(graph: &Graph) -> Vec<NoDoFiltro> {
    let mut saidas: Vec<NodeId> = graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == OUTPUT)
        .map(|n| n.id)
        .collect();
    saidas.sort();
    let tipo = |id: NodeId| {
        graph
            .nodes()
            .iter()
            .find(|n| n.id == id)
            .map(|n| n.type_name.as_str())
    };
    let mut vistos = BTreeSet::new();
    let mut pilha = Vec::new();
    for saida in saidas {
        let mut fio = Vec::new();
        let mut cur = saida;
        // ⚠️ Pára numa aresta ATRASADA (o laço de feedback) e num nó já visto: o fio é finito.
        while let Some((acima, _, atrasada)) = graph.input_edge(cur, 0) {
            if atrasada || !vistos.insert(acima) {
                break;
            }
            if tipo(acima) == Some(TYPE_NAME) {
                fio.push(acima);
            }
            cur = acima;
        }
        pilha.extend(fio.into_iter().rev().map(|no| NoDoFiltro {
            no,
            op: op_of(graph, no),
            scope: Scope::of(read(graph.node_param_overrides(no), param::SCOPE)),
        }));
    }
    pilha
}

/// Os `fx.filter` do grafo que NÃO estão no fio de uma `Output` — não filtram nada, e o cartão
/// diz-lo.
#[must_use]
pub fn fora_do_fio(graph: &Graph, pilha: &[NoDoFiltro]) -> Vec<NodeId> {
    graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == TYPE_NAME && !pilha.iter().any(|d| d.no == n.id))
        .map(|n| n.id)
        .collect()
}

struct FxFilter;

impl NodeOp for FxFilter {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        // Passthrough: o nó DECLARA o degrau; o fio passa intacto.
        let out = ctx.input(0).clone();
        ctx.emit(out);
    }
}

/// Regista o nó (chamado por `ph2d-node-registry-init`).
///
/// # Errors
/// O [`RegistryError`] do registo.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(FxFilter))?;
    // O kernel é o PASSTHROUGH (ADR-0126): sem ele a cadeia inteira cairia na CPU.
    reg.register_gpu_kernel(MANIFEST.id, ph2d_nodegraph::gpu::GpuKernel::PASSTHROUGH);
    reg.register_ui(
        MANIFEST.id,
        NodeUiManifest {
            display_key: "node.fx.filter.name",
            category: NodeUiCategory::Fx,
            silhouette: NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_hard_max(MANIFEST.id, PARAM_HARD_MAX);
    Ok(())
}

const fn hint(
    param: &'static str,
    label: &'static str,
    min: f32,
    max: f32,
    step: f32,
    widget: ParamWidget,
) -> ParamUiHint {
    ParamUiHint {
        param,
        label,
        min,
        max,
        step,
        widget,
    }
}

const fn slider(param: &'static str, label: &'static str, min: f32, max: f32) -> ParamUiHint {
    hint(param, label, min, max, 0.005, ParamWidget::Slider)
}

const fn modos(
    param: &'static str,
    label: &'static str,
    labels: &'static [&'static str],
) -> ParamUiHint {
    hint(param, label, 0.0, 1.0, 1.0, ParamWidget::Enum { labels })
}

const L: f32 = COMPRIMENTO_MAX;

static PARAM_HINTS: &[ParamUiHint] = &[
    hint(
        param::KIND,
        "node.fx.filter.param.kind",
        0.0,
        (FxOp::KINDS - 1) as f32,
        1.0,
        ParamWidget::Enum {
            labels: &KIND_LABELS,
        },
    ),
    modos(param::SCOPE, "node.fx.filter.param.scope", &SCOPE_LABELS),
    slider(param::RADIUS, "node.fx.filter.param.radius", 0.0, L),
    slider(param::OFFSET_X, "node.fx.filter.param.offset_x", -L, L),
    slider(param::OFFSET_Y, "node.fx.filter.param.offset_y", -L, L),
    hint(
        param::COLOR[0],
        "node.fx.filter.param.color_r",
        0.0,
        1.0,
        0.01,
        ParamWidget::Color {
            channels: param::COLOR,
        },
    ),
    hint(
        param::COLOR2[0],
        "node.fx.filter.param.color2_r",
        0.0,
        1.0,
        0.01,
        ParamWidget::Color {
            channels: param::COLOR2,
        },
    ),
    hint(
        param::OPACITY,
        "node.fx.filter.param.opacity",
        0.0,
        1.0,
        0.01,
        ParamWidget::Slider,
    ),
    modos(
        param::FALLOFF,
        "node.fx.filter.param.falloff",
        &FALLOFF_LABELS,
    ),
    modos(
        param::NOISE_MODE,
        "node.fx.filter.param.noise_mode",
        &NOISE_LABELS,
    ),
    modos(
        param::RAMP_MODE,
        "node.fx.filter.param.ramp_mode",
        &RAMP_LABELS,
    ),
    slider(param::SIZE, "node.fx.filter.param.size", 0.0, L),
    hint(
        param::DETAIL,
        "node.fx.filter.param.detail",
        1.0,
        FxOp::MAX_DETAIL as f32,
        1.0,
        ParamWidget::IntSlider,
    ),
    hint(
        param::SEED,
        "node.fx.filter.param.seed",
        0.0,
        u8::MAX as f32,
        1.0,
        ParamWidget::Seed,
    ),
    slider(param::GROW, "node.fx.filter.param.grow", -L, L),
    // Meia volta para cada lado cobre todas as matizes: é o PERÍODO, não um tecto.
    hint(
        param::HUE,
        "node.fx.filter.param.hue",
        -180.0,
        180.0,
        1.0,
        ParamWidget::Slider,
    ),
    // `-1` = cinzento / preto exacto, `+1` = croma dobrado / branco exacto: a faixa da LEI.
    hint(
        param::SATURATION,
        "node.fx.filter.param.saturation",
        -1.0,
        1.0,
        0.01,
        ParamWidget::Slider,
    ),
    hint(
        param::BRIGHTNESS,
        "node.fx.filter.param.brightness",
        -1.0,
        1.0,
        0.01,
        ParamWidget::Slider,
    ),
];

const fn digitado(param: &'static str) -> ParamHardMax {
    ParamHardMax {
        param,
        max: COMPRIMENTO_DIGITADO,
    }
}

/// O valor digitado vai além do arrasto nos comprimentos (o passe honra-o, [`COMPRIMENTO_DIGITADO`]).
static PARAM_HARD_MAX: &[ParamHardMax] = &[
    digitado(param::RADIUS),
    digitado(param::OFFSET_X),
    digitado(param::OFFSET_Y),
    digitado(param::SIZE),
    digitado(param::GROW),
];

const fn unidade(param: &'static str, unit: ParamUnit) -> ParamUnitDecl {
    ParamUnitDecl { param, unit }
}

static PARAM_UNITS: &[ParamUnitDecl] = &[
    unidade(param::RADIUS, ParamUnit::Length),
    unidade(param::OFFSET_X, ParamUnit::Length),
    unidade(param::OFFSET_Y, ParamUnit::Length),
    unidade(param::SIZE, ParamUnit::Length),
    unidade(param::GROW, ParamUnit::Length),
    unidade(param::HUE, ParamUnit::Angle),
];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
