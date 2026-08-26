#![forbid(unsafe_code)]
//! `motion.sort` — **reorder the instance stream by a key**: the Houdini / TouchDesigner
//! "Sort SOP" (Motion Nodes M3, structure — doc 01 §3 / doc 27). Every other node
//! transforms values in place or grows the count; this is the first that **reorders**.
//! On its own it looks like nothing changed — its purpose is to set the *order* a
//! downstream index-based effector reveals in: `sort` (radial) → `cull` (a growing
//! fraction) is a wipe that fills from the centre out; `sort` (random) → `cull` is a
//! dissolve.
//!
//! **Algorithm — a stable sort of the element permutation by a computed key.** Each
//! element gets a key from `key` — **Radial** (squared distance from `(center_x,
//! center_y)`; squared, so no `sqrt` and the same order), **X**, **Y**, **Random** (a
//! splitmix hash of the index → a deterministic shuffle), or **Index** (identity) — and
//! the permutation is stably sorted ascending (`descending` reverses it). Every column
//! (`P`, `size`, `tint`, `id`, …) is reordered by that permutation, so the whole
//! instance travels together. The count is unchanged. **`Index` is the one column that
//! is a fact about the LIST and not about the element, so `reindex` (on by default)
//! republishes it as the new rank** — without that the ramp downstream keeps painting the
//! order the stream had before ([`REINDEX`]). Transcendental-free (HR-5):
//! comparison keys are arithmetic (Radial uses squared distance) + the integer hash.
//! `Effect::Pure`.

use ph2d_node_registry::{NodeRegistry, ParamUnit, ParamUnitDecl, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod hash;
mod trig;
use hash::rand01;

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// O tipo do domínio de VALOR — o campo escalar por-instância na coluna `v` (o espelho
/// local que toda folha desta casa escreve; o vocabulário partilhado é a PORTA, não um
/// símbolo comum).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
/// A coluna do domínio de valor.
const VALUE_COL: &str = "v";

/// Sort keys (the `key` param).
const KEY_RADIAL: i64 = 0;
const KEY_X: i64 = 1;
const KEY_Y: i64 = 2;
const KEY_RANDOM: i64 = 3;
// KEY_INDEX (4) = identity — any other value falls through to it.
/// **A CHAVE ARBITRÁRIA** (doc 89 folha 08 — *"a chave é um ENUM de 5 respostas onde toda
/// referência a faz um FIELD"*: Blender *Sort Elements* tem `Sort Weight` como socket,
/// Houdini o *Sort SOP* aceita expressão).
///
/// ⚠️ **Um MODO do enum, e não uma porta que vence em silêncio.** A alternativa —
/// *"weight ligado sobrepõe o enum"* — deixaria o dropdown a mostrar `Radial` enquanto a
/// ordenação é outra coisa, que é o botão morto desta casa visto do avesso. Aqui o enum
/// continua a ser a resposta única a *"o que é a chave?"*, e a porta só é lida quando ele
/// aponta para ela.
///
/// ⚠️ Desligada no modo `Weight`, a coluna lê **zeros** ⇒ a ordenação é estável ⇒ a
/// identidade. Nada explode, e a lista sai como entrou.
const KEY_WEIGHT: i64 = 5;

/// A coluna de IDENTIDADE — *quem é este elemento na lista* (o irmão `motion.combine` usa o
/// mesmo nome pela mesma razão).
const INDEX: &str = "Index";

/// **A RENUMERAÇÃO** — e o defeito que ela cura foi visto num smoke, não por um gate.
///
/// A `eval` permuta TODAS as colunas, e é isso que mantém a instância inteira junta. Mas o
/// `Index` não é um atributo da peça: é **a posição dela na lista**, e uma lista reordenada
/// cujo `Index` viajou com as peças descreve a lista de ANTES. O sintoma medido (cena `=63`,
/// 2026-08-19): um `motion.grid(7×7) → motion.sort(X) → motion.tint(Gradient)` pintava a
/// grelha **por linhas, de baixo para cima** — a ordem de nascimento — com **6 reinícios** do
/// degradê, um por coluna. A ordenação não chegava ao pixel.
///
/// ⚠️ **O censo é de UM** (medido, `crates/`): o único consumidor da COLUNA `Index` é o
/// `motion.tint` em gradiente. Todos os outros efectores indexados — `motion.oscillator`
/// (`i · phase_stagger`), `value.instance_field` (`KeyBy::Index => i`), `motion.cull`
/// (Fraction guarda os primeiros `amount·n`) — chaveiam pela POSIÇÃO no vector, e por isso
/// sempre seguiram a ordenação. Era um nó a discordar de cinco, e o que ele lê é a coluna que
/// este nó estragava.
///
/// ⚠️ **O default é `1`, ao contrário do `reindex` do `motion.combine`, e a diferença é
/// MEDIDA e não de gosto.** Naquele o estado desligado é o que sempre shipou e há arte
/// autorada por trás; aqui o estado desligado torna o nó **invisível ao seu único
/// consumidor**, e a promessa do doc-comment lá em cima (*"its purpose is to set the order a
/// downstream index-based effector reveals in"*) é falsa. Medido nesta árvore: os únicos dois
/// grafos autorados com um `motion.sort` são esta cena e o `motion_state_gpu_demos`, cuja
/// jusante (`oscillator` + `scale`) é **posicional** — nenhuma arte muda. E a referência
/// concorda: no Houdini o *Sort SOP* renumera os pontos, e o que sobrevive a uma ordenação é
/// o `@id`, que aqui é a coluna `id` e continua a viajar com a peça.
///
/// ⚠️ **Ligado, ele REESCREVE o `Index` que existe; não INVENTA um que não existia** — e a
/// assimetria com o irmão é a mesma medição. O `combine` MUDA a contagem, então tem de
/// escrever `Index` **e** `Count` mesmo em branco (senão o `Count` mente). Este preserva a
/// contagem: `Count` já está honesto, e uma lista sem `Index` faz o `motion.tint` cair no seu
/// próprio atalho posicional `i/(n−1)` — que **já é** a ordem ordenada. Cunhar a coluna ali
/// daria exactamente o mesmo número, e uma coluna que não muda resposta nenhuma é peso.
const REINDEX: &str = "reindex";

/// **A DIREÇÃO ARBITRÁRIA** (doc 89 folha 08 — o *Sort SOP* do Houdini ordena ao longo de um
/// VETOR).
///
/// A célula media a composição que já existia: `motion.rotate(θ) → motion.sort(X) →
/// motion.rotate(−θ)` — **três nós para um knob**, que é o critério de `P1` da §7 daquele
/// plano.
///
/// ⚠️ **O ângulo ROTACIONA o eixo escolhido, em vez de substituí-lo**, e é isso que o torna
/// literal nos dois modos: em `0` o `X` continua a ser `p.x` e o `Y` continua a ser `p.y`
/// (o `Y` é o `X` mais um quarto de volta, que é o que o enum já significava). Um ângulo que
/// definisse a direção **absoluta** faria `Y` e `axis_angle = 0` discordarem, e um dos dois
/// teria de ganhar em silêncio.
const AXIS_ANGLE: &str = "axis_angle";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.sort"),
    name: "motion.sort",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // **A CHAVE COMO CAMPO** (doc 89 folha 08). Lida só no modo `Weight`; ver
        // [`KEY_WEIGHT`]. Apendada, então todo grafo salvo continua a resolver as
        // portas pelos mesmos índices.
        PortSpec {
            name: "weight",
            ty: VALUE,
        },
        // **O GRUPO** (doc 89 folha 08, linha 34: *"ordena DENTRO de grupos"* — Blender
        // *Sort Elements*). Apendada pelo mesmo motivo da `weight`: um grafo salvo resolve
        // as portas por ÍNDICE, e uma porta no meio renumeraria as ligações de todos eles.
        // Desligada ⇒ um grupo só ⇒ a permutação de sempre, ao bit. Ver [`permutation`].
        PortSpec {
            name: "group",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // 0 Radial · 1 X · 2 Y · 3 Random · 4 Index (identity).
        ParamSpec {
            name: "key",
            default: 0.0,
        },
        // 0 ascending · 1 descending.
        ParamSpec {
            name: "descending",
            default: 0.0,
        },
        // Centre for the Radial key (world units).
        ParamSpec {
            name: "center_x",
            default: 0.0,
        },
        ParamSpec {
            name: "center_y",
            default: 0.0,
        },
        // Seed for the Random key.
        ParamSpec {
            name: "seed",
            default: 0.0,
        },
        // **A DIREÇÃO do eixo**, em graus. Ver [`AXIS_ANGLE`].
        ParamSpec {
            name: "axis_angle",
            default: 0.0,
        },
        // **A RENUMERAÇÃO** — `1` (o default) reescreve `Index` para o posto na lista
        // ordenada; `0` deixa a identidade viajar com a peça, que é o que sempre
        // aconteceu. Ver [`REINDEX`] para o censo e o porquê do default.
        ParamSpec {
            name: "reindex",
            default: 1.0,
        },
        // ⚠️ **Apendado**: a ROTAÇÃO da ordem (folha 08 linha 35). `0` = a
        // permutação de sempre, literalmente. Ver [`permutation`].
        ParamSpec {
            name: "shift",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// The sort key for each element, given the positions and params.
fn keys(
    p: &[[f32; 2]],
    key: i64,
    center: [f32; 2],
    seed: u32,
    axis_deg: f32,
    weight: &[f32],
) -> Vec<f32> {
    // A direção do eixo: `X` corre em `axis_deg`, `Y` um quarto de volta depois. Em
    // `axis_deg = 0` os dois colapsam em `p.x` e `p.y`, ao bit.
    //
    // ⚠️ **Ciclos e a senoide local, nunca o `f32::sin_cos` da `std`** (HR-5): aquele é a
    // libm da PLATAFORMA, e uma ordenação é justamente onde um ulp de diferença vira uma
    // PERMUTAÇÃO diferente — o replay cross-OS deixaria de bater. Ver o `trig.rs` ao lado.
    let (cos, sin) = crate::trig::cos_sin_cycles(axis_deg / 360.0);
    (0..p.len())
        .map(|i| match key {
            KEY_RADIAL => {
                let (dx, dy) = (p[i][0] - center[0], p[i][1] - center[1]);
                dx * dx + dy * dy
            }
            // ⚠️ **O caminho de `0°` é LITERAL, e a mutação que o apaga SOBREVIVE — ele
            // fica documentado em vez de escondido** (o precedente do `substeps = 1` da
            // `motion.verlet_rope`). A aritmética já é exacta ali: `cos_sin_cycles(0)` dá
            // `(1, 0)` ao bit, e `x·1 + y·0 = x` em IEEE-754 para todo `x`/`y` FINITO. O
            // ramo existe pelo caso degenerado: com uma posição `±inf` — o que uma sim
            // divergida produz — `0 · inf` é **NaN**, e a chave de ordenação de uma peça
            // viraria NaN enquanto o caminho literal devolve o `inf` que ela de facto tem.
            KEY_X if axis_deg == 0.0 => p[i][0],
            KEY_Y if axis_deg == 0.0 => p[i][1],
            KEY_X => p[i][0] * cos + p[i][1] * sin,
            KEY_Y => -p[i][0] * sin + p[i][1] * cos,
            KEY_RANDOM => rand01(seed, i as u32, 0),
            // A chave como CAMPO: a coluna do domínio de valor, com a regra 1→N desta casa
            // (um valor só vale para todos; ausente é zero ⇒ a ordem estável, a identidade).
            KEY_WEIGHT => match weight.len() {
                0 => 0.0,
                1 => weight[0],
                _ => weight.get(i).copied().unwrap_or(0.0),
            },
            _ => i as f32, // Index (identity)
        })
        .collect()
}

/// The stable-sorted permutation of `0..n` by `k` (ascending; reversed if
/// `descending`), **rotated** by `shift`.
///
/// ⚠️ **O `shift` ROTACIONA, não desloca** (Houdini *Sort SOP* «Shift», Cavalry
/// `Shuffle`): a lista continua a ser uma permutação de `0..n`, com toda peça
/// exactamente uma vez. Um deslocamento que empurrasse as pontas para fora
/// **perderia** peças e a saída deixaria de ter a contagem da entrada — que é a
/// coisa que um nó de ORDEM nunca pode fazer.
///
/// A rotação é `rem_euclid` e não `%`: um shift negativo é o caso normal do knob
/// (o artista arrasta para a esquerda), e o `%` de Rust devolve negativo.
///
/// `shift = 0` ⇒ a permutação de sempre, literalmente (`rotate_left(0)` é um no-op).
fn permutation(k: &[f32], groups: &[f32], descending: bool, shift: i64) -> Vec<usize> {
    // O grupo de um elemento: a coluna quando ela existe, e **zero** quando não —
    // *«desligada ⇒ um grupo só»*, que é a linha do default da célula.
    let g = |i: usize| -> f32 {
        match groups.len() {
            0 => 0.0,
            1 => groups[0],
            _ => groups.get(i).copied().unwrap_or(0.0),
        }
    };
    let mut perm: Vec<usize> = (0..k.len()).collect();
    // ⚠️ **A chave primária é o GRUPO, a secundária é a chave de sempre** — e o
    // `sort_by` é estável, então empates nas duas mantêm a ordem de chegada, tal como
    // antes desta porta existir.
    perm.sort_by(|&a, &b| g(a).total_cmp(&g(b)).then(k[a].total_cmp(&k[b])));
    // ⚠️ **O `descending` e o `shift` valem DENTRO de cada grupo, nunca sobre a lista
    // toda.** A alternativa — inverter/rodar o todo — moveria elementos entre grupos, o
    // que é a negação de *"ordena dentro de grupos"*; e num grupo só as duas leituras
    // colapsam, que é o que mantém o caminho de sempre byte a byte.
    let mut start = 0usize;
    while start < perm.len() {
        let gv = g(perm[start]);
        let mut end = start + 1;
        while end < perm.len() && g(perm[end]).total_cmp(&gv) == core::cmp::Ordering::Equal {
            end += 1;
        }
        let run = &mut perm[start..end];
        if descending {
            run.reverse();
        }
        let n = run.len() as i64;
        run.rotate_left(shift.rem_euclid(n) as usize);
        start = end;
    }
    perm
}

/// **Reescreve o `Index` da lista ordenada para o posto de cada peça** (`0..n−1`), *se* a
/// lista trouxer um. Ver [`REINDEX`] — a coluna ausente não é cunhada, de propósito.
fn renumber(out: &mut Stream) {
    if !matches!(out.get(INDEX), Some(Column::Scalar(_))) {
        return;
    }
    let n = out.count();
    #[expect(clippy::cast_precision_loss, reason = "uma contagem de elementos")]
    let idx: Vec<f32> = (0..n).map(|i| i as f32).collect();
    out.set(INDEX, Column::Scalar(idx));
}

/// Reorder a column by the permutation (`out[i] = col[perm[i]]`).
fn permute(col: &Column, perm: &[usize]) -> Column {
    fn gather<T: Clone>(v: &[T], perm: &[usize]) -> Vec<T> {
        perm.iter().map(|&i| v[i].clone()).collect()
    }
    match col {
        Column::Scalar(v) => Column::Scalar(gather(v, perm)),
        Column::Vec2(v) => Column::Vec2(gather(v, perm)),
        Column::Vec3(v) => Column::Vec3(gather(v, perm)),
        Column::Vec4(v) => Column::Vec4(gather(v, perm)),
    }
}

struct MotionSort;

impl NodeOp for MotionSort {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let key = ctx.param("key").round() as i64;
        let descending = ctx.param("descending").round() as i64 != 0;
        let center = [ctx.param("center_x"), ctx.param("center_y")];
        let seed = ctx.param("seed").round().max(0.0) as u32;
        let axis = ctx.param(AXIS_ANGLE);
        // ⚠️ O peso é lido ANTES do input 0 e clonado: os dois `ctx.input` não podem
        // coexistir emprestados, e a coluna é `Vec<f32>` (barata, e só no modo que a lê).
        let weight: Vec<f32> = if key == KEY_WEIGHT {
            match ctx.input(1).get(VALUE_COL) {
                Some(Column::Scalar(v)) => v.clone(),
                _ => Vec::new(),
            }
        } else {
            Vec::new()
        };
        // ⚠️ Lido ANTES do input 0 e clonado, pela mesma razão do peso: dois `ctx.input`
        // emprestados não coexistem. Ao contrário do peso, **não** é gateado por modo — o
        // grupo é ortogonal à chave (ordena-se DENTRO dele, seja qual for a chave).
        let groups: Vec<f32> = match ctx.input(2).get(VALUE_COL) {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let input = ctx.input(0);
        let n = input.count();
        let p: Vec<[f32; 2]> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        let perm = permutation(
            &keys(&p, key, center, seed, axis, &weight),
            &groups,
            descending,
            ctx.param("shift").round() as i64,
        );
        let mut out = Stream::new(n);
        for (name, col) in input.columns() {
            out.set(name.clone(), permute(col, &perm));
        }
        if ctx.param(REINDEX) >= 0.5 {
            renumber(&mut out);
        }
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionSort))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Sort",
            category: ph2d_node_registry::NodeUiCategory::Utility,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    reg.register_param_gates(MANIFEST.id, PARAM_GATES);
    Ok(())
}

use ph2d_node_registry::{ParamGate, ParamUiHint, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "key",
        label: "Key",
        min: 0.0,
        max: 4.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Radial", "X", "Y", "Random", "Index", "Weight"],
        },
    },
    ParamUiHint {
        param: "descending",
        label: "Descending",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Ascending", "Descending"],
        },
    },
    ParamUiHint {
        param: "center_x",
        label: "Center X",
        min: -20.0,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "center_y",
        label: "Center Y",
        min: -20.0,
        max: 20.0,
        step: 0.05,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **Gateado ao eixo**: num `Radial`/`Random`/`Index` o ângulo não muda um número, e
    // pintá-lo ali seria o botão morto que esta casa recusa.
    ParamUiHint {
        param: AXIS_ANGLE,
        label: "Axis Angle",
        min: 0.0,
        max: 360.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "seed",
        label: "Seed",
        min: 0.0,
        max: 9999.0,
        step: 1.0,
        widget: ParamWidget::Seed,
    },
    // ⚠️ **Um `Toggle`, como no irmão `motion.combine`**: renumerar acontece ou não
    // acontece, e meia renumeração não quer dizer nada. Não é gateado a chave nenhuma —
    // no modo `Index` a permutação é a identidade e o knob é um no-op, que é diferente de
    // um botão morto: ele continua a dizer a verdade sobre o que faria.
    ParamUiHint {
        param: REINDEX,
        label: "Reindex",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    // A ROTAÇÃO da ordem. `0` = a permutação de sempre. ⚠️ Ela ROTACIONA — a lista
    // continua a ter toda peça exactamente uma vez.
    ParamUiHint {
        param: "shift",
        label: "Shift",
        min: -32.0,
        max: 32.0,
        step: 1.0,
        widget: ParamWidget::IntSlider,
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
static PARAM_UNITS: &[ParamUnitDecl] = &[
    ParamUnitDecl {
        param: "center_x",
        unit: ParamUnit::Length,
    },
    ParamUnitDecl {
        param: "center_y",
        unit: ParamUnit::Length,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    const O: [f32; 2] = [0.0, 0.0];

    fn r2(p: [f32; 2]) -> f32 {
        p[0] * p[0] + p[1] * p[1]
    }

    /// Radial sort orders elements by distance from the centre (ascending), and the
    /// result is a permutation of the input (same multiset). FALSIFIED if the output
    /// radii weren't monotone.
    #[test]
    fn radial_sort_orders_by_distance() {
        let p = vec![[3.0, 0.0], [0.0, 1.0], [-2.0, 0.0], [0.5, 0.5]];
        let perm = permutation(&keys(&p, KEY_RADIAL, O, 0, 0.0, &[]), &[], false, 0);
        let sorted: Vec<[f32; 2]> = perm.iter().map(|&i| p[i]).collect();
        for w in sorted.windows(2) {
            assert!(r2(w[0]) <= r2(w[1]), "radii non-decreasing: {sorted:?}");
        }
        // Nearest to the centre (0.5,0.5) is first; farthest (3,0) is last.
        assert_eq!(sorted[0], [0.5, 0.5]);
        assert_eq!(*sorted.last().unwrap(), [3.0, 0.0]);
    }

    /// X sort orders by x; descending reverses it.
    #[test]
    fn x_sort_and_descending() {
        let p = vec![[2.0, 0.0], [-1.0, 0.0], [0.5, 0.0]];
        let asc = permutation(&keys(&p, KEY_X, O, 0, 0.0, &[]), &[], false, 0);
        assert_eq!(asc, vec![1, 2, 0], "ascending by x");
        let desc = permutation(&keys(&p, KEY_X, O, 0, 0.0, &[]), &[], true, 0);
        assert_eq!(desc, vec![0, 2, 1], "descending reverses");
    }

    /// The Random key is a deterministic shuffle: a permutation of the input, the same
    /// every run, and (for a non-trivial set) not the identity order.
    #[test]
    fn random_is_a_deterministic_shuffle() {
        let p: Vec<[f32; 2]> = (0..20).map(|i| [i as f32, 0.0]).collect();
        let a = permutation(&keys(&p, KEY_RANDOM, O, 42, 0.0, &[]), &[], false, 0);
        let b = permutation(&keys(&p, KEY_RANDOM, O, 42, 0.0, &[]), &[], false, 0);
        assert_eq!(a, b, "deterministic");
        assert_ne!(a, (0..20).collect::<Vec<_>>(), "actually shuffled");
        let mut sorted = a.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..20).collect::<Vec<_>>(), "a permutation");
    }

    /// Deterministic + cooks through the registry: every column is reordered by the same
    /// permutation (P and size travel together).
    #[test]
    fn registers_and_reorders_all_columns() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.sort.test.src"),
            name: "motion.sort.test.src",
            inputs: &[],
            outputs: &[PortSpec {
                name: "out",
                ty: INST_VEC2,
            }],
            effect: Effect::Pure,
            clock: Clock::Frame,
            params: &[],
            lowerings: &[LoweringKind::Cpu],
        };
        struct Src;
        impl NodeOp for Src {
            fn manifest(&self) -> &'static NodeManifest {
                &SRC
            }
            fn eval(&self, ctx: &mut EvalCtx<'_>) {
                // Two elements, far one first; each tagged by a distinct size.
                ctx.emit(
                    Stream::new(2)
                        .with("P", Column::Vec2(vec![[5.0, 0.0], [1.0, 0.0]]))
                        .with("size", Column::Scalar(vec![9.0, 1.0])),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionSort),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.sort.test.src");
        let s = g.add_node("motion.sort");
        g.set_param(s, "key", KEY_RADIAL as f32);
        g.connect(Edge {
            from: (src, 0),
            to: (s, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, s, 0.0).unwrap();
        let st = out[0].as_stream();
        // The near element (P=(1,0), size 1) sorts first, dragging its size with it.
        match (st.get("P").unwrap(), st.get("size").unwrap()) {
            (Column::Vec2(pv), Column::Scalar(sv)) => {
                assert_eq!(pv[0], [1.0, 0.0], "near element first");
                assert_eq!(sv[0], 1.0, "its size travelled with it");
            }
            _ => panic!("columns"),
        }
    }
}

/// **Que knob aparece em que chave.** Os que já existiam nunca foram gateados — o `center_*`
/// só vale no `Radial` e o `seed` só no `Random`, e os dois eram pintados em todas —, e a
/// wave do eixo é a ocasião de os pôr no sítio: um painel que mostra cinco números dos quais
/// dois não fazem nada ensina que os knobs deste nó são decorativos.
static PARAM_GATES: &[ParamGate] = &[
    ParamGate {
        param: AXIS_ANGLE,
        when: "key",
        #[expect(clippy::cast_possible_truncation, reason = "índices de enum, 0..5")]
        values: &[KEY_X as i32, KEY_Y as i32],
    },
    ParamGate {
        param: "center_x",
        when: "key",
        #[expect(clippy::cast_possible_truncation, reason = "índices de enum, 0..5")]
        values: &[KEY_RADIAL as i32],
    },
    ParamGate {
        param: "center_y",
        when: "key",
        #[expect(clippy::cast_possible_truncation, reason = "índices de enum, 0..5")]
        values: &[KEY_RADIAL as i32],
    },
    ParamGate {
        param: "seed",
        when: "key",
        #[expect(clippy::cast_possible_truncation, reason = "índices de enum, 0..5")]
        values: &[KEY_RANDOM as i32],
    },
];

#[cfg(test)]
#[path = "axis_weight_tests.rs"]
mod axis_weight_tests;

#[cfg(test)]
#[path = "reindex_tests.rs"]
mod reindex_tests;

#[cfg(test)]
#[path = "group_tests.rs"]
mod group_tests;
