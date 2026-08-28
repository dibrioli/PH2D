#![forbid(unsafe_code)]
//! `value.number` — **um número escrito à mão**, o valor comum que o grafo não tinha.
//!
//! ## O report que o abriu
//!
//! *"Não temos um nó para valores simples comuns. Se eu quiser usar um valor simples comum numa
//! operação matemática, preciso usar LFO:Offset. Isso não é intuitivo."* — Enio, 2026-08-27.
//!
//! ⚠️⚠️ **E esta capacidade já tinha sido RECUSADA, por escrito, com um argumento que o report
//! refuta.** O `debug.const` (que emite `1.0` fixo) foi tirado da paleta em 2026 com a nota:
//! *«a capacidade "uma constante" já existe por duas cadeias de UM nó (`value.pattern` com
//! `steps = 1`, `value.map_range` com `out_lo == out_hi`), então o que faltava não era um nó:
//! era ele deixar de ser OFERECIDO»*.
//!
//! O argumento estava certo sobre a EXPRESSIVIDADE e errado sobre o produto: o artista não achou
//! nenhuma das duas saídas — foi buscar uma **terceira** (o `offset` de uma `value.lfo` com a
//! amplitude a zero) e chamou-lhe não-intuitiva. *Três atalhos existirem e o artista não achar
//! nenhum não é uma capacidade: é a mesma lei que a conferência aplica a todo o catálogo —
//! **exprimível não é alcançável**.*
//!
//! ## O que ele cobre, e por que não cobre texto
//!
//! Medido no registry (2026-08-27): **um fio deste grafo carrega exactamente dois tipos** —
//! `Instances/Scalar` (91 portas de entrada) e `Instances/Vec2` (118). **Não existe canal de
//! TEXTO**: o `Dim` tem `Scalar`/`Vec2`/`Vec3`/`Vec4`/`Mat2..4` e mais nada, e um nome (a imagem
//! do `fx.glow`, a forma do `motion.path`) viaja como **text param do documento**, nunca por
//! aresta. ⇒ um nó de "nome constante" não teria onde ligar, e a pergunta *«nomes também são
//! usados nos grafos?»* responde-se: **sim, mas não por fio**.
//!
//! - **Número** — o caso comum, `Instances/Scalar`.
//! - **Booleano** — medido: **34 dos 757 params do catálogo são `Toggle`**, e um param conduzido
//!   lê um escalar. Um booleano **já é** um número aqui (`0`/`1`); o que faltava era o artista
//!   não ter de saber isso. O modo `Boolean` mostra uma caixa e emite `0.0`/`1.0` exactos.
//! - ⛔ **Vec2 fica de fora, e não é omissão:** `motion.make_point` já constrói posições a
//!   partir de dois escalares (é a razão de ele existir), então um par destes nós faz o vetor
//!   pela composição que já shipa. *Antes de construir um item, meça se a composição já o
//!   exprime.*

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

/// O que este nó emite — o mesmo tipo que todo `value.*` produz.
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// `0` = Número · `1` = Booleano. Ver o cabeçalho.
pub const KIND: &str = "kind";
/// O número, no modo `Number`.
pub const VALUE_PARAM: &str = "value";
/// A caixa, no modo `Boolean`.
pub const STATE: &str = "state";

/// **O CURSO DA MÃO — medido, não escolhido** (§0.0).
///
/// Este valor pode conduzir **qualquer** param do catálogo, então a faixa útil é a união das
/// faixas deles. Medido no registry em 2026-08-27, sobre os **491 sliders** que o catálogo
/// declara:
///
/// ```text
///   faixa          cobre (de 492 sliders)
///   [-10, 10]      348   (71%)
///   [-50, 50]      435   (88%)
///   [-75, 75]      447   (91%)   <- este
///   [-100, 100]    458   (93%)
/// ```
///
/// ⚠️⚠️ **O `75` NÃO é o joelho da cobertura — é o TECTO DA MÃO, e ele é DERIVADO.** A 1.ª
/// redacção pôs `100`, e o gate `the_slider_drags_where_the_hand_works` reprovou-a com o
/// mecanismo: o track do painel mede `154 px` e o mapeamento é linear, então `span / 154` é o
/// **menor passo que um arrasto consegue** — a `±100` isso dá `1,3` por pixel, ou seja *um pixel
/// movia mais que o próprio default (`1`)*. O maior curso simétrico que a convenção admite é
/// `±77`.
///
/// ⭐ E baixar **não custou cobertura**: `±75` cobre os mesmos `91%` que `±100`, porque a cauda
/// do catálogo salta de `~60` para os milhares sem nada pelo meio. Os extremos reais vão a `−720`
/// e a `22 000`, e nenhum curso de mão os alcança com passo utilizável — é para isso que existe
/// o tecto DIGITÁVEL abaixo.
///
/// ⚠️ **Isto é o default de quem NÃO conduz nada.** Desde a cura do `ParamUnit::FromWire`
/// (2026-08-27), um `Number` ligado a um param veste a faixa DAQUELE param — ligado ao
/// `source.shape::size` ele arrasta em `5..1000 px`, exactamente como o slider do destino. A
/// união das faixas do catálogo só manda enquanto o nó está solto, que é o único momento em que
/// ninguém sabe para que ele serve.
const HAND_SPAN: f32 = 75.0;
/// **O TECTO DA MÁQUINA — e o recurso é a PRECISÃO do `f32`**, não uma opinião.
///
/// O passo do slider é `0,01`, e a pergunta é onde `v + 0,01 == v`. **MEDIDO** (2026-08-27):
///
/// ```text
///   131 072 (2¹⁷)  ->  131 072,01562   o passo ainda MOVE   <- o tecto
///   262 144 (2¹⁸)  ->  262 144         o passo EVAPORA
/// ```
///
/// ⚠️ **A 1.ª redacção desta nota deduziu `65 536` e a medição corrigiu-a uma potência de dois
/// acima.** A álgebra dizia `ULP(v) ≤ passo`; o `f32` arredonda **ao mais próximo**, então basta
/// `passo ≥ ULP/2` — e a diferença entre as duas contas é exactamente um expoente. *Uma derivação
/// que não se corre é uma hipótese com cara de resultado.*
///
/// ⚠️ Ele cobre com folga a maior faixa que o catálogo declara (`22 000`, o `sim.spawn::rate`).
const TYPED_LIMIT: f32 = 131_072.0;

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.number"),
    name: "value.number",
    // ⚠️ **Sem entradas, de propósito.** Um número autorado que aceitasse uma entrada seria um
    // `value.math` com um operando escondido — e esse nó já existe.
    inputs: &[],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: KIND,
            default: 0.0,
        },
        // ⚠️ **O default é `1.0` e não `0.0`.** Um nó recém-largado tem de fazer alguma coisa
        // visível: `0` é o elemento neutro da soma e o absorvente do produto, então um `Number`
        // acabado de largar num `value.math` não mudaria nada e leria como partido. `1` é o
        // neutro do produto e move a soma — o mesmo raciocínio do `debug.const`, que emitia `1`.
        ParamSpec {
            name: VALUE_PARAM,
            default: 1.0,
        },
        ParamSpec {
            name: STATE,
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// O valor que este nó emite, dados os params — a lei, separada do `eval` para ser testável.
#[must_use]
pub fn value_of(kind: f32, value: f32, state: f32) -> f32 {
    if kind.round() as i32 == 1 {
        // ⚠️ **`0.0`/`1.0` EXACTOS**, nunca o `state` cru: o destino é um param que outro nó lê
        // como número, e um booleano que chegasse `0,7` seria um terceiro estado que a caixa não
        // consegue exprimir nem desfazer.
        f32::from(state >= 0.5)
    } else {
        value
    }
}

struct ValueNumber;

impl NodeOp for ValueNumber {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let v = value_of(ctx.param(KIND), ctx.param(VALUE_PARAM), ctx.param(STATE));
        // Um elemento: a regra 1→N do substrato difunde-o por qualquer contagem a jusante.
        ctx.emit(Stream::new(1).with(VALUE_COL, Column::Scalar(vec![v])));
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueNumber))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            // ⚠️ **«Number», não «Value»** — o artista procura o que quer PÔR no grafo, e a
            // família inteira já se chama `value.*`. Um nó chamado *Value* na paleta seria o
            // nome da prateleira, e a busca por *"number"* não o acharia.
            display_name: "Number",
            // Cinzento de utilidade: ele não transforma nada, é encanamento — a mesma escolha
            // do `value.math`, que é o consumidor que o report nomeou.
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    reg.register_param_hard_max(MANIFEST.id, HARD_MAX);
    reg.register_param_hard_min(MANIFEST.id, HARD_MIN);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

use ph2d_node_registry::{
    ParamGate, ParamHardMax, ParamHardMin, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget,
};

/// **O NÚMERO É O QUE ELE CONDUZ** (doc 88 + doc 58) — o report do Enio, 2026-08-27:
/// *"number em 0,94 imprime em shape:size 94px."*
///
/// Ele estava certo sobre o defeito e o número estava certo sobre a física: o
/// `source.shape::size` é um comprimento guardado em **metros**, e `0,94 m` **são** `94 px`.
/// As duas rows diziam a verdade sobre a própria unidade — e por isso o artista via **um
/// fio e dois números**, sem nada na tela que os ligasse.
///
/// ⚠️ **`state` e `kind` ficam de fora, e não é omissão:** um índice de enum e uma caixa não
/// são grandezas. Marcá-los faria a caixa de um booleano herdar a face de um comprimento e
/// mostrar `100 px` para *ligado*.
static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: VALUE_PARAM,
    unit: ParamUnit::FromWire,
}];

/// **Cada modo mostra o seu controle, e só o seu.** Sem isto o painel pintaria a caixa e o
/// slider ao mesmo tempo, e um deles seria um controle sobre nada — o defeito que o
/// `ParamGate` existe para não ter (doc 90).
static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: VALUE_PARAM,
        when: KIND,
        values: &[0],
    },
    ParamGate {
        param: STATE,
        when: KIND,
        values: &[1],
    },
];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: KIND,
        label: "Kind",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Number", "Boolean"],
        },
    },
    ParamUiHint {
        param: VALUE_PARAM,
        label: "Value",
        min: -HAND_SPAN,
        max: HAND_SPAN,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: STATE,
        label: "State",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
];

static HARD_MAX: &[ParamHardMax] = &[ParamHardMax {
    param: VALUE_PARAM,
    max: TYPED_LIMIT,
}];

static HARD_MIN: &[ParamHardMin] = &[ParamHardMin {
    param: VALUE_PARAM,
    min: -TYPED_LIMIT,
}];

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
