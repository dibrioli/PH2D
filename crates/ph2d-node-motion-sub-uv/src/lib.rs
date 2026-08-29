#![forbid(unsafe_code)]
//! `motion.sub_uv` — **que PEDAÇO da textura cada elemento mostra** (doc 89, folha 17).
//!
//! O *SubImage* do Sprite Renderer do Niagara (`SubImageSize` + `SubImageIndex`) e a
//! sprite-sheet do Stardust/Cavalry: uma imagem cortada em `cols × rows` células, e cada
//! instância a mostrar uma delas. É o que faz um flipbook — uma explosão, uma faísca a
//! piscar, uma folha de personagem percorrida por partícula.
//!
//! ## A coluna é RELATIVA, e a decisão inteira está nisso
//!
//! ⚠️ **A `uv_rect` não servia, e a medição diz porquê:** ela é o rectângulo **ABSOLUTO**
//! no atlas, e quem sabe qual é o ladrilho de um objecto é o `source.object` — ou, quando
//! não há objecto nenhum, a **shell**, que só o fornece no instante do lowering. Um nó a
//! meio do grafo não pode escrever UV absoluta sem inventar o ladrilho.
//!
//! Este nó escreve a coluna **`uv_cell`** — `[escala_u, escala_v, desloc_u, desloc_v]`,
//! uma fracção do rectângulo que a linha tiver —, que os dois lowerings depositam no
//! `RenderInstance::uv_xform`. **O shader já aplica esse transform DENTRO do sub-rect da
//! própria sprite** (`mix(atlas_uv.xy, atlas_uv.zw, local)`), então a célula compõe com o
//! ladrilho em vez de o substituir, e o mesmo grafo funciona sobre o atlas partilhado e
//! sobre a textura individual de um objecto.
//!
//! ⚠️ **A ordem das células é a da CASA** (`sprite_sheet_subrect` da shell, e portanto a
//! do Inspector e a do importador de Aseprite): **linha-maior**, `col = k % cols`,
//! `row = k / cols`, com a linha `0` no topo. Por colunas dava uma folha bonita e todas
//! as animações trocadas.
//!
//! ## O índice: um param, uma taxa, um escalonamento — e uma PORTA
//!
//! ⚠️ **`speed` e `stagger` existem para o flipbook ser UM nó.** Com só um índice o
//! artista teria de montar `value.time → value.wrap → sub_uv.cell` para a coisa mais
//! comum que este nó faz, que é *avançar as células no tempo*. Os dois somam-se ao
//! índice base e valem `0` por omissão, então quem quiser a rota explícita continua a
//! tê-la — a porta soma-se a eles, não os substitui.
//!
//! ⚠️ **A porta `cell` é a ESCADA de sempre** (a `time` do `motion.oscillator`):
//! desligada ⇒ o param · comprimento 1 ⇒ **broadcast** · comprimento `n` ⇒ um índice por
//! elemento.

mod holds;
pub use holds::HOLDS_KEY;

use ph2d_node_registry::{NodeRegistry, ParamUiHint, ParamWidget, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::gpu::{ColumnAccess, ColumnBinding, GpuKernel};
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// O tipo da porta de índice — o mesmo `VALUE` que `value.*` emite.
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);

/// A coluna que este nó escreve, e que os DOIS lowerings depositam no `uv_xform`.
///
/// ⚠️ O nome é convenção de stream, copiado à mão em três sítios que não se alcançam
/// (aqui, o lowering da CPU e a geradora da GPU) — exactamente como o `blend`. O que os
/// mantém em acordo é um gate na shell, o único lugar que vê os três.
pub const CELL_COLUMN: &str = "uv_cell";

/// O maior número de células por eixo — o **teto de RECURSO**, digitável.
///
/// ⚠️ **O recurso é a RESOLUÇÃO DA TEXTURA, e o número sai dela:** o teto de textura
/// desta casa é `MAX_SHEET_EDGE_PX = 8192` (memória de GPU, `max_texture_dimension_2d`), e
/// uma folha com mais de 256 cortes num eixo dá células de **32 px ou menos** na maior
/// textura que o app aceita — abaixo disso a célula deixa de ter conteúdo antes de o
/// knob deixar de ter curso. Não é conforto de implementação: a aritmética aguenta
/// qualquer inteiro exacto em `f32` (2²⁴).
pub const MAX_CELLS_PER_AXIS: f32 = 256.0;

/// Até onde o SLIDER arrasta (doc 88 §11) — a faixa de AUTORIA, que não é a mesma
/// pergunta que o teto de recurso.
///
/// ⚠️ **O número não foi escolhido: ele foi imposto por um gate que já existia**
/// (`the_slider_drags_where_the_hand_works`). Com o curso a ir até 256, um pixel do track
/// movia **1,7 células** — mais que o próprio default de `1` —, e o gate mediu-o na 1.ª
/// corrida. `16` é a folha típica (o `hframes × vframes` do Inspector, o Aseprite), e o
/// `256` continua **digitável** pelo [`MAX_CELLS_PER_AXIS`] ⇒ nada fica inalcançável.
const SOFT_CELLS_PER_AXIS: f32 = 16.0;

/// Quantas células por segundo o índice anda, no máximo.
///
/// ⚠️ **O recurso é a TAXA DE QUADROS:** acima de 60 células/s um flipbook a 60 fps salta
/// mais de uma célula por quadro, e o que se vê deixa de ser animação e passa a ser
/// ruído — a célula seguinte nunca chega a ser desenhada. O teto é o dobro disso, para o
/// efeito *cintilar* continuar exprimível de propósito.
pub const MAX_CELL_SPEED: f32 = 120.0;

/// **A CÉLULA `k` de uma grelha `cols × rows`**, como `[escala_u, escala_v, desloc_u,
/// desloc_v]` — a lei, e a mesma que o WGSL espelha linha a linha.
///
/// ⚠️ **O embrulho é `rem_euclid`, não `%`**: um índice NEGATIVO (um `stagger` para trás,
/// um `value.*` que passou de zero) tem de dar a célula contada do fim, e o `%` de Rust
/// devolveria um resto negativo ⇒ uma célula fora da folha, que o shader amostraria como
/// o ladrilho vizinho.
///
/// ⚠️ E um índice **não-finito** cai na célula `0` em vez de propagar `NaN` para a UV: um
/// quad com UV `NaN` não desaparece, ele amostra lixo.
#[must_use]
pub fn cell_xform(index: f32, cols: u32, rows: u32) -> [f32; 4] {
    let cols = cols.max(1);
    let rows = rows.max(1);
    let cells = i64::from(cols) * i64::from(rows);
    let k = if index.is_finite() {
        (index.floor() as i64).rem_euclid(cells)
    } else {
        0
    };
    let col = (k % i64::from(cols)) as f32;
    let row = (k / i64::from(cols)) as f32;
    let (cw, ch) = (1.0 / cols as f32, 1.0 / rows as f32);
    [cw, ch, col * cw, row * ch]
}

/// A grelha autorada, coagida na PORTA: pelo menos `1` em cada eixo, no máximo
/// [`MAX_CELLS_PER_AXIS`]. Um `0` ou um negativo vindo de um param dirigido dá a folha
/// inteira, que é o mundo sem este nó.
fn grid(ctx: &EvalCtx<'_>) -> (u32, u32) {
    let axis = |v: f32| v.round().clamp(1.0, MAX_CELLS_PER_AXIS) as u32;
    (axis(ctx.param("cols")), axis(ctx.param("rows")))
}

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.sub_uv"),
    name: "motion.sub_uv",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // ⚠️ **APENDADA, nunca inserida** — as arestas de um documento salvo guardam o
        // ÍNDICE da porta. Aqui ela nasce com o nó, mas a lei fica escrita para quem
        // acrescentar a terceira.
        PortSpec {
            name: "cell",
            ty: VALUE,
        },
    ],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    // ⚠️⚠️ **`Temporal`, e a troca é a CURA DE UM DEFEITO PRÉ-EXISTENTE** (report do Enio,
    // 2026-08-28: *"não há nenhuma animação ou movimento na cena de smoke"*).
    //
    // Este nó lê `ctx.playhead()` (o `speed` e o `stagger` são um relógio), e a impressão
    // digital do memo só inclui o relógio **se o nó se declarar `Temporal`**
    // (`cook_fingerprint.rs`: *"a playhead bits (if `Temporal`)"*). Declarado `Pure`, ele
    // cozinhava UMA vez e devolvia o mesmo stream para sempre: o flipbook ficava **congelado**
    // na célula do primeiro quadro.
    //
    // ⚠️ **Ele shipou assim desde que existe, e ninguém viu — porque nenhuma cena tinha ligado
    // o relógio dele.** A fileira sub-UV da `=9` deixa o `speed` no default (`0`), então a
    // única coisa que o exercitava era um nó parado. *Um defeito só é visível onde há uma cena
    // que o contenha.*
    //
    // ⚠️ E os gates não o viam por uma razão própria: eles construíam um `Cook::new()` a cada
    // instante, e um memo que nasce vazio nunca devolve nada de velho. Quem reusa o cozinhador
    // é o app — e é assim que o gate `the_flipbook_keeps_moving_under_one_persistent_cook`
    // passou a medir.
    effect: Effect::Temporal,
    clock: Clock::Frame,
    params: &[
        ParamSpec {
            name: "cols",
            default: 1.0,
        },
        ParamSpec {
            name: "rows",
            default: 1.0,
        },
        ParamSpec {
            name: "cell",
            default: 0.0,
        },
        ParamSpec {
            name: "speed",
            default: 0.0,
        },
        ParamSpec {
            name: "stagger",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// GPU compute kernel (ADR-0126) — o espelho linha a linha do [`cell_xform`].
///
/// ⚠️ **`suv_floor_mod` é o `rem_euclid` do WGSL**, que não o tem: `x - n*floor(x/n)` é a
/// definição, e é ela que faz um índice negativo contar do fim da folha. Um `%` do WGSL
/// (que é `frem`, resto com o sinal do dividendo) daria a célula errada exactamente nos
/// casos em que a CPU acerta — divergência de LEI, invisível sem o gate de paridade.
///
/// ⚠️ **O `HAS_cell_v` é constante do módulo gerado** (fixo por pipeline compilado, e a
/// cache é chaveada nessa assinatura), então ramificar nele não custa nada no device — a
/// mesma leitura que a porta `time` do `motion.oscillator` pagou.
const GPU_KERNEL: GpuKernel = GpuKernel {
    wgsl: "\
        let suv_cols = suv_axis(params.cols);\n\
        let suv_rows = suv_axis(params.rows);\n\
        var suv_base = params.cell;\n\
        if (HAS_cell_v) { suv_base = read_cell_v(i); }\n\
        let suv_k = suv_base + params.speed * params.playhead + params.stagger * f32(i);\n\
        write_uv_cell(i, suv_cell(suv_hold(suv_k, suv_cols * suv_rows), suv_cols, suv_rows));\n",
    wgsl_lib: "\
        fn suv_axis(v: f32) -> f32 {\n\
        \x20   // A MESMA coacao da porta da CPU: arredonda, depois clampa em [1, MAX].\n\
        \x20   let r = select(ceil(v - 0.5), floor(v + 0.5), v >= 0.0);\n\
        \x20   return clamp(r, 1.0, 256.0);\n\
        }\n\
        fn suv_hold(k: f32, cells: f32) -> f32 {\n\
        \x20   // A DURACAO DESIGUAL POR QUADRO. A tabela leva `-1` em toda a entrada\n\
        \x20   // quando nada foi autorado, e ai este ramo devolve `k` intacto: o\n\
        \x20   // caminho que sempre shipou, byte a byte. Ver `holds.rs`.\n\
        \x20   var ph = 0.0;\n\
        \x20   if (k == k && abs(k) < 1.0e30 && cells > 0.0) { ph = suv_floor_mod(k / cells, 1.0); }\n\
        \x20   let v = suv_hold_sample(ph);\n\
        \x20   if (v < 0.0) { return k; }\n\
        \x20   return floor(v * cells);\n\
        }\n\
        fn suv_floor_mod(x: f32, n: f32) -> f32 {\n\
        \x20   // O `rem_euclid` que o WGSL nao tem. O `%` dele leva o sinal do\n\
        \x20   // dividendo, entao um indice negativo cairia FORA da folha.\n\
        \x20   return x - n * floor(x / n);\n\
        }\n\
        fn suv_cell(index: f32, cols: f32, rows: f32) -> vec4<f32> {\n\
        \x20   let cells = cols * rows;\n\
        \x20   // Nao-finito cai na celula 0: uma UV `NaN` nao some, ela amostra lixo.\n\
        \x20   var f = floor(index);\n\
        \x20   if (!(f == f) || abs(f) > 1.0e30) { f = 0.0; }\n\
        \x20   let k = suv_floor_mod(f, cells);\n\
        \x20   let col = suv_floor_mod(k, cols);\n\
        \x20   let row = floor(k / cols);\n\
        \x20   let cw = 1.0 / cols;\n\
        \x20   let ch = 1.0 / rows;\n\
        \x20   return vec4<f32>(cw, ch, col * cw, row * ch);\n\
        }\n",
    bindings: &[
        ColumnBinding {
            column: CELL_COLUMN,
            dim: Dim::Vec4,
            access: ColumnAccess::Write,
            identity: [1.0, 1.0, 0.0, 0.0],
            port: 0,
        },
        // A porta de índice: `ReadBroadcast` para herdar a regra 1→N (um `value.*`
        // desligado emite UM valor, e ele vale para a instância inteira).
        ColumnBinding {
            column: "v",
            dim: Dim::Scalar,
            access: ColumnAccess::ReadBroadcast,
            identity: [0.0; 4],
            port: 1,
        },
    ],
    params: &["cols", "rows", "cell", "speed", "stagger"],
    count_law: None,
    variant_by_param: None,
    applicable: None,
};

/// O canal de LUT deste nó — o nome dá o acessor `suv_hold_sample` que o WGSL chama.
static LUTS: &[ph2d_nodegraph::gpu::LutSpec] = &[ph2d_nodegraph::gpu::LutSpec {
    name: holds::HOLD_LUT_NAME,
    text_key: holds::HOLDS_KEY,
    resolution: holds::HOLD_LUT_RESOLUTION,
    fill: holds::fill_hold_lut,
}];

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "cols",
        label: "Columns",
        min: 1.0,
        max: SOFT_CELLS_PER_AXIS,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "rows",
        label: "Rows",
        min: 1.0,
        max: SOFT_CELLS_PER_AXIS,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "cell",
        // A célula EMBRULHA na grelha, então o curso do slider só precisa de alcançar a
        // maior folha que o slider da grelha exprime — e o teto duro segue os irmãos.
        label: "Cell",
        min: 0.0,
        max: SOFT_CELLS_PER_AXIS * SOFT_CELLS_PER_AXIS,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    ParamUiHint {
        param: "speed",
        label: "Cells / Second",
        min: -MAX_CELL_SPEED,
        max: MAX_CELL_SPEED,
        step: 0.5,
        widget: ParamWidget::Slider,
    },
    // ⚠️ **PESOS RELATIVOS, nunca milissegundos** — ver [`holds`]. O tempo do ciclo continua
    // a ser `células / speed`; a lista só o redistribui, e é isso que impede uma segunda
    // resposta a *«quão rápido»* ao lado do `Cells / Second`.
    ParamUiHint {
        param: holds::HOLDS_KEY,
        label: "Frame Holds",
        min: 0.0,
        max: 0.0,
        step: 0.0,
        widget: ParamWidget::Text,
    },
    ParamUiHint {
        param: "stagger",
        label: "Stagger",
        min: -16.0,
        max: 16.0,
        step: 0.25,
        widget: ParamWidget::Slider,
    },
];

/// Os tetos DUROS — o que a caixa numérica alcança, acima do curso do slider.
static HARD_MAX: &[ph2d_node_registry::ParamHardMax] = &[
    ph2d_node_registry::ParamHardMax {
        param: "cols",
        max: MAX_CELLS_PER_AXIS,
    },
    ph2d_node_registry::ParamHardMax {
        param: "rows",
        max: MAX_CELLS_PER_AXIS,
    },
    ph2d_node_registry::ParamHardMax {
        param: "cell",
        // A maior célula endereçável é a última da maior folha.
        max: MAX_CELLS_PER_AXIS * MAX_CELLS_PER_AXIS - 1.0,
    },
];

struct MotionSubUv;

impl NodeOp for MotionSubUv {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let (cols, rows) = grid(ctx);
        let base = ctx.param("cell");
        let speed = ctx.param("speed");
        let stagger = ctx.param("stagger");
        let t = ctx.playhead() as f32;
        let text_holds = ctx.text_param(holds::HOLDS_KEY).unwrap_or("").to_string();
        // A ESCADA da porta: vazia ⇒ o param · 1 ⇒ broadcast · n ⇒ por elemento.
        let port: Vec<f32> = match ctx.input(1).get("v") {
            Some(Column::Scalar(v)) => v.clone(),
            _ => Vec::new(),
        };
        let mut out = {
            let input = ctx.input(0);
            let mut out = Stream::new(input.count());
            for (name, col) in input.columns() {
                out.set(name.clone(), col.clone());
            }
            out
        };
        let n = out.count();
        // A tabela dos *holds*, uma vez por cozimento — ver [`holds`] para por que a CPU
        // AMOSTRA em vez de calcular exacto (a lei é um degrau, e exacto-contra-tabelado
        // difere por uma célula inteira na fronteira).
        let lut = holds::table(&text_holds);
        let cell_count = cols * rows;
        let cells = (0..n)
            .map(|i| {
                let seed = match port.len() {
                    0 => base,
                    1 => port[0],
                    _ => port.get(i).copied().unwrap_or(base),
                };
                let k = seed + speed * t + stagger * i as f32;
                cell_xform(holds::held_index(&lut, k, cell_count), cols, rows)
            })
            .collect();
        out.set(CELL_COLUMN.to_string(), Column::Vec4(cells));
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionSubUv))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Sub UV",
            category: ph2d_node_registry::NodeUiCategory::Fx,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    // Doc 88 §11: o slider arrasta onde a mão trabalha, a CAIXA digita até o teto do
    // recurso. Sem isto o curso de 1..256 movia 1,7 células por pixel de arrasto.
    reg.register_param_hard_max(MANIFEST.id, HARD_MAX);
    reg.register_gpu_kernel(MANIFEST.id, GPU_KERNEL);
    // O canal de LUT dos *holds*: o sequenciador amostra o text param no cozimento e liga a
    // tabela, para o `suv_hold` a ler NO DEVICE. ⛔ A alternativa — `applicable: false` a
    // derrubar o nó para a CPU quando há holds — é proibida por lei do módulo (§5), e aqui
    // seria pior do que noutro sítio: este é o nó de flipbook, o mais barato de correr.
    reg.register_luts(MANIFEST.id, LUTS);
    Ok(())
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
