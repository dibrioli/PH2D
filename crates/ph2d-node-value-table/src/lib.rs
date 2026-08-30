//! ⭐⭐⭐ **`value.table` — uma linha do ficheiro é um MOMENTO, e sai UM número que anda.**
//!
//! É a metade do padrão-ouro que a Adobe teve de **inventar um formato** para ter. O
//! [manual dela](https://helpx.adobe.com/after-effects/using/data-driven-animations.html) diz o
//! porquê sem rodeios: *"JSON, CSV e TSV só podem conter valores estáticos"* — o `.mgjson`
//! existe porque um ficheiro de dados, lido como tabela, **não sabe dizer o que muda com o
//! tempo**. A resposta deles é o `dataValue()`, que amostra no instante corrente.
//!
//! ⇒ aqui a resposta é: **o artista NOMEIA a coluna do tempo**. Com ela, a tabela deixa de ser
//! uma lista de coisas e passa a ser uma curva — e um CSV de telemetria (GPS, sensor, cotação)
//! conduz a animação sem formato nenhum pelo meio.
//!
//! # ⚠️ Por que este nó não é um MODO do [`ph2d_node_source_table`]
//!
//! Porque a taxonomia deste motor já respondeu: os `source.*` entregam ELEMENTOS e os `value.*`
//! entregam um NÚMERO. Um enum dentro de um nó faria a **cardinalidade** da saída mudar por
//! baixo do consumidor — e nenhum nó a jusante pode ser escrito para os dois.
//!
//! # ⚠️ `Effect::Temporal`, e não é detalhe
//!
//! Este nó lê `ctx.playhead()`. Um nó que o lê e se declara `Pure` **congela** — foi
//! exactamente isso que aconteceu ao `motion.sub_uv`, que esteve parado desde que existe.

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// O caminho do ficheiro — ver a nota do `source.table`: é um CAMINHO, nunca o conteúdo.
pub const FILE_KEY: &str = "file";
/// O nome da coluna que é o TEMPO, em segundos.
pub const TIME_KEY: &str = "time";
/// O nome da coluna cujo valor sai.
pub const VALUE_KEY: &str = "value";

pub mod param {
    /// `0` = degrau (o valor da linha anterior) · `1` = recta entre as duas linhas.
    pub const INTERP: &str = "interp";
    /// O que acontece fora do intervalo do ficheiro: `0` = segura a ponta · `1` = repete.
    pub const OUTSIDE: &str = "outside";
}

pub const INTERP_STEP: i32 = 0;
pub const OUTSIDE_HOLD: i32 = 0;

/// ⭐ **A LEI DA AMOSTRAGEM — porta única, e `pub` para um gate lhe poder falar directamente.**
///
/// `times` tem de ser não-decrescente (é o que todo ficheiro de telemetria exporta, e o que o
/// `.mgjson` também exige). ⚠️ Fora de ordem ela não entra em pânico e não mente sobre o que
/// fez: devolve o resultado da busca binária, que é determinístico.
///
/// ⚠️ **Uma amostra só devolve-a sempre** — sem isto, uma tabela de uma linha (o caso de quem
/// está a montar o ficheiro) devolveria zero e leria-se como *"o vínculo não funciona"*.
#[must_use]
pub fn sample(times: &[f32], values: &[f32], t: f32, linear: bool, loop_outside: bool) -> f32 {
    let n = times.len().min(values.len());
    if n == 0 {
        return 0.0;
    }
    if n == 1 {
        return values[0];
    }
    let (first, last) = (times[0], times[n - 1]);
    // ⚠️⚠️ **`first > last` é alcançável e `f32::clamp` ENTRA EM PÂNICO nesse caso** — foi o
    // gate `nothing_makes_it_panic_and_a_missing_column_reads_zero` que o apanhou, com uma
    // coluna de tempo fora de ordem (que é um ficheiro que existe: basta alguém ordenar a folha
    // por outra coluna antes de exportar). Uma tabela mal ordenada tem de dar um número
    // esquisito, **nunca derrubar o app**.
    let t = if last <= first {
        t
    } else if loop_outside {
        // ⚠️ `rem_euclid` e não `%`: antes do início, o `%` do Rust devolve NEGATIVO e a
        // repetição saltaria para o fim do ficheiro em vez de continuar o ciclo.
        first + (t - first).rem_euclid(last - first)
    } else {
        t.clamp(first, last)
    };
    // O índice da primeira amostra DEPOIS de `t`.
    let hi = times[..n].partition_point(|&x| x <= t);
    if hi == 0 {
        return values[0];
    }
    if hi >= n {
        return values[n - 1];
    }
    if !linear {
        return values[hi - 1];
    }
    let (t0, t1) = (times[hi - 1], times[hi]);
    let span = t1 - t0;
    if span.abs() < f32::EPSILON {
        return values[hi];
    }
    let u = ((t - t0) / span).clamp(0.0, 1.0);
    values[hi - 1] + (values[hi] - values[hi - 1]) * u
}

pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("value.table"),
    name: "value.table",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: VALUE,
    }],
    // ⚠️ Lê o playhead ⇒ **Temporal**. Ver a nota do cabeçalho.
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: param::INTERP,
            default: 1.0,
        },
        ParamSpec {
            name: param::OUTSIDE,
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

struct ValueTable;

impl NodeOp for ValueTable {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let file = ctx.text_param(FILE_KEY).unwrap_or("").to_string();
        let time_col = ctx.text_param(TIME_KEY).unwrap_or("").to_string();
        let value_col = ctx.text_param(VALUE_KEY).unwrap_or("").to_string();
        let linear = ctx.param(param::INTERP).round() as i32 != INTERP_STEP;
        let loop_outside = ctx.param(param::OUTSIDE).round() as i32 != OUTSIDE_HOLD;
        let t = ctx.playhead() as f32;
        let table = ctx.external(&ph2d_node_registry::table_external_key(&file));
        let scalar = |name: &str| match table.get(name) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let (times, values) = (scalar(&time_col), scalar(&value_col));
        let v = sample(&times, &values, t, linear, loop_outside);
        // A cardinalidade segue a geometria, como todo `value.*`.
        let n = ctx.input(0).count().max(1);
        ctx.emit(Stream::new(n).with(VALUE_COL, Column::Scalar(vec![v; n])));
    }
}

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: FILE_KEY,
        label: "Table File",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::File {
            kind: ph2d_node_registry::FileKind::Table,
        },
    },
    ParamUiHint {
        param: TIME_KEY,
        label: "Time Column",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Text,
    },
    ParamUiHint {
        param: VALUE_KEY,
        label: "Value Column",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Text,
    },
    ParamUiHint {
        param: param::INTERP,
        label: "Interpolation",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Step", "Linear"],
        },
    },
    ParamUiHint {
        param: param::OUTSIDE,
        label: "Outside",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Hold", "Loop"],
        },
    },
];

/// Register this node with the runtime registry.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(ValueTable))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Table Value",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::TrapezoidDown,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
