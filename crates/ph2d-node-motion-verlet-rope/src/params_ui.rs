//! The node's **param UI metadata** — labels, ranges, widgets, units. Split from
//! `lib.rs` at the HR-18 LOC cap, on the seam the siblings already use
//! (`ph2d-node-motion-soft-body/src/params_ui.rs`): none of this is behaviour, so
//! the rope computes exactly the same catenary whatever a slider looks like.

use ph2d_node_registry::{ParamHardMax, ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};
/// **O teto DURO de `count` — MEDIDO** (doc 88 A1 · §0), enquanto o slider fica nos 200 que cobrem
/// uma corda de autoria confortável.
///
/// A relaxação é Gauss-Seidel por aresta — **sequencial por semântica**, mas LINEAR na contagem.
/// Medido pela porta do produto (`measure_the_count_ceiling`, com a aresta `pre` de estado ligada;
/// sem ela o `eval` semeia e a tabela reporta **300× menos**):
///
/// | partículas | cook |
/// |---|---|
/// | 10.000 | 2,040 ms |
/// | **50.000** | **~10 ms** (interpolado do linear) |
/// | 100.000 | 20,533 ms |
/// | 400.000 | 83,267 ms |
///
/// Cem mil já passa de um quadro de 60 fps; cinquenta mil fica em ~60% dele — 250× o que o slider
/// alcança. O teto é onde a medição parou de caber.
/// **OS TRÊS QUE FALTAVAM, MEDIDOS** (doc 89 folha 03 linha 52 · sonda `measure_rope_ceiling`).
///
/// **`iterations` = 128, e o teto É o clamp** (`.clamp(1, 128)` no `eval`). Acima dele a caixa
/// de texto **aceita e mente** — a cicatriz do `lattice` 400 e do `kaleidoscope` 256. Medido, a
/// queda da ponta ao fim de 60 tiques no pior passo:
///
/// | iterations | 8 | 64 | **128** | 129 | 512 | 100.000 |
/// |---|---|---|---|---|---|---|
/// | queda | 9,6001 | 6,2695 | **6,0885** | 6,0885 | 6,0885 | 6,0885 |
///
/// As três últimas não são *parecidas* com a de 128: são **byte a byte** a de 128.
///
/// **`gravity` e `length` = 1e20 — e é a MESMA parede, alcançada por dois caminhos.** Os dois
/// morrem exactamente em `1e21`, e isso não é coincidência: o que estoura não é o parâmetro, é a
/// **POSIÇÃO** da corda em `f32` (uma gravidade grande e um comprimento grande levam ambos as
/// coordenadas ao mesmo lugar). É o recurso que o §0 admite nomear — *precisão de representação*.
///
/// | gravity | 9 | 40 | 1e3 | 1e12 | 1e18 | **1e20** | 1e21 |
/// |---|---|---|---|---|---|---|---|
/// | queda | 6,78 | 9,49 | 95,6 | 8,97e10 | 8,97e16 | **8,97e18** | **0** |
///
/// | length | 6 | 1e8 | 1e16 | **1e20** | 1e21 |
/// |---|---|---|---|---|---|
/// | queda/repouso | 1,129 | 1,0000 | 1,0000 | **1,0000** | **0** |
///
/// ⚠️ **O modo de falha é o SILENCIOSO, e é por isso que o teto se paga apesar de ficar 19
/// ordens de grandeza acima do slider:** em `1e21` a queda é **exactamente zero** — a corda não
/// explode à vista, ela **desaparece**, sem erro e sem aviso. É o mesmo desenho do `SALTA` do
/// `motion.spring`.
///
/// ⚠️ **E o `length` é escala-invariante até morrer** (`queda/repouso` = `1,0000` de `1e8` a
/// `1e20`), tal como o `radius` do `motion.collide` — a diferença é que ali a invariância ia até
/// onde a medição alcançava e **não havia teto a escrever**, e aqui existe uma parede.
///
/// ⚠️ **Uma armadilha da SONDA, não do produto:** medir a queda com `powi(2)` em `f32` estoura
/// em ~1e19 e reporta `0`/`inf` — a 1ª varredura acusou a corda de morrer em `1e24` por culpa
/// da própria régua. A queda é computada em `f64`.
pub(crate) static PARAM_HARD_MAX: &[ParamHardMax] = &[
    ParamHardMax {
        param: "damping",
        max: 0.5,
    },
    ParamHardMax {
        param: "count",
        max: 50_000.0,
    },
    ParamHardMax {
        param: "iterations",
        max: 128.0,
    },
    ParamHardMax {
        param: "gravity",
        max: 1e20,
    },
    ParamHardMax {
        param: "length",
        max: 1e20,
    },
    // **O teto de `substeps` — MEDIDO, e o recurso é TEMPO** (`what_a_substep_costs`,
    // um tique pela porta do `step`, com as 24 iterações de fábrica). O custo é
    // EXACTAMENTE linear nos sub-passos, então o número honesto é o último que cabe
    // no sub-orçamento de física suave do HR-4 (2,0 ms/tique) numa corda de tamanho
    // sério:
    //
    // | pontos | ×1 | ×4 | ×8 | **×16** | ×32 |
    // |---|---|---|---|---|---|
    // | 24 | 0,014 | 0,027 | 0,053 | **0,134** (7%) | 0,213 |
    // | 256 | 0,073 | 0,302 | 0,611 | **1,232** (62%) | 2,383 (**119%, estoura**) |
    // | 2048 | 0,590 | 2,481 (**124%**) | 4,716 | 9,608 | 19,088 |
    //
    // ⚠️ **O teto NÃO desce quando o `count` sobe, e é deliberado** — o mesmo
    // raciocínio que o `MAX_SIDE` do `motion.soft_body` escreve sobre os `clusters`:
    // um teto que se movesse com o vizinho tiraria do artista uma corda que ele já
    // autorou, e o valor certo passaria a ser função de OUTRO knob, que é a forma que
    // esta casa chama de bug de ergonomia. Os dois números multiplicam, os dois estão
    // no painel, e é aqui que o produto fica escrito.
    ParamHardMax {
        param: "solver_substeps",
        max: 16.0,
    },
];

pub(crate) static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "count",
        label: "Points",
        min: 2.0,
        max: 200.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "length",
        label: "Length",
        min: 0.5,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "gravity",
        label: "Gravity",
        min: 0.0,
        max: 40.0,
        step: 0.1,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "iterations",
        label: "Stiffness",
        min: 1.0,
        max: 128.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "damping",
        label: "Damping",
        min: 0.0,
        max: 0.2,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "pin_tail",
        label: "Pin Tail",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Free", "Pinned"],
        },
    },
    // ⚠️ A pista começa em `0` porque `0` é o DESLIGADO — um piso aqui esconderia
    // o neutro, e é ele que mantém toda corda já autorada byte-idêntica.
    ParamUiHint {
        param: "bend",
        label: "Bend Stiffness",
        min: 0.0,
        max: 1.0,
        step: 0.01,
        widget: ParamWidget::Slider,
    },
    // ⚠️ O slider para em **8** e a caixa digita até **16**: o par soft/hard do
    // doc 88 B2 — a faixa de arrasto é onde a mão trabalha, e o teto digitável é
    // onde o disfuncional começa (a tabela está no `PARAM_HARD_MAX`).
    // ⚠️ A CHAVE é `solver_substeps` e o RÓTULO é "Substeps" — o artista vê a palavra da
    // referência, e a convenção de relógio do grafo (`SUBSTEPS_PARAM`) não é reclamada por um nó
    // folha. O porquê, com a tabela medida, está no `ParamSpec` do `lib.rs`.
    ParamUiHint {
        param: "solver_substeps",
        label: "Substeps",
        min: 1.0,
        max: 8.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
];

/// **What each of this node's numbers IS** (doc 88, Wave A) — never how it is
/// shown. A `Length` is stored in world METRES and the panel resolves the face
/// the artist reads (`px` or `m`) from `ProjectSettings::display_unit`; a node
/// that could pin one would be overriding a setting it does not own.
///
/// Only params whose value is a world COORDINATE or a world DISTANCE are declared
/// here. A weight, a fraction, a rate and a count are left bare on purpose: a unit
/// that is wrong is worse than a unit that is missing, because the artist can read
/// a bare number but a mislabelled one teaches them something false.
pub(crate) static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: "length",
    unit: ParamUnit::Length,
}];
