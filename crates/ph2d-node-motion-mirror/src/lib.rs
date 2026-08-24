#![forbid(unsafe_code)]
//! `motion.mirror` — **reflect and duplicate** the layout across an axis: a symmetry /
//! kaleidoscope modifier (Motion Nodes M3, distributions — doc 01 §3 / doc 25). The
//! "Symmetry"/"Mirror" of every 2D/3D package: take a layout and make it symmetric.
//!
//! **Algorithm — an axis reflection through the centroid.** Each element is kept, and a
//! reflected copy is added: for a **vertical** axis, `(x, y) → (2·cx − x, y)`; for a
//! **horizontal** axis, `(x, y) → (x, 2·cy − y)`, where `(cx, cy)` is the layout's
//! centroid. So `count → 2·count`, the two halves mirror-images. Only the **position**
//! `P` is reflected; every other column (`size`, `tint`, `id`, …) is copied onto the
//! twin — a mirror of the *layout*, which is exact for a positional distribution (a
//! moving sim's `vel`/`rot` are duplicated, not flipped). Transcendental-free (HR-5):
//! reflection is arithmetic — no trig, no `sqrt`. `Effect::Pure`.
//!
//! ## Onde a LINHA de espelho fica (doc 88 §B3 — a varredura PRO da família TRANSFORM)
//!
//! O eixo era **pregado no centroide**: o layout só sabia espelhar contra si mesmo, e
//! encostar a simetria numa parede — a metade das composições que pedem um espelho —
//! não era difícil, era **inexprimível**. O `offset` move a linha.
//!
//! ⚠️ **Ele é medido A PARTIR DO CENTROIDE, e é isso que faz o default ser byte-idêntico
//! ao que já shipava.** Um offset em coordenada de mundo absoluta não teria como
//! exprimir "no centroide" — o número certo dependeria do conteúdo —, então o zero
//! deixaria de ser o comportamento antigo e todo grafo autorado saltaria. O mesmo
//! raciocínio do *Relative Offset* do Array do Blender: **um número redondo tem de
//! significar alguma coisa** sem o artista saber onde a nuvem está.
//!
//! ⚠️ **CONSIDERADO E NÃO CONSTRUÍDO, com o motivo** (para ninguém o reconstruir cego):
//! um **ângulo** livre de eixo. O `axis` já responde *qual eixo*, e um ângulo por cima
//! dele seriam duas portas para a mesma pergunta (`Vertical + 90°` é `Horizontal`); a
//! simetria de ângulo arbitrário já tem dono — o `motion.kaleidoscope`, que é N-fold
//! por construção.
//!
//! ## O GÊMEO passa a espelhar-se por inteiro (doc 89 folha 05)
//!
//! O parágrafo acima dizia que refletir `rot`/`vel` no gêmeo *"é mudança de
//! COMPORTAMENTO de uma sim espelhada, não um param: ela merece o seu próprio
//! smoke"*. A cerca **precificava**, não recusava — e o preço foi pago: o
//! [`FLIP_ROT`] é o param, com `0` no comportamento antigo, e a cena `=69` é o
//! smoke que ela pedia. Sem ele um espelho de peixes a nadar produzia metade do
//! cardume a nadar de costas, e não havia composição que o corrigisse (nada a
//! jusante sabe QUAIS elementos são gêmeos).
//!
//! ⚠️ **E as colunas de IDENTIDADE descreviam a lista de ANTES** — a mesma lei que
//! o `reindex` do `motion.sort` fixou nos nós que PERMUTAM, aqui num que CRESCE.
//! Medido: uma grelha 3×3 (`Index = 0..8`, `Count = 9`) saía deste nó com **n = 18**
//! e `Index = [0..8, 0..8]`, `Count = 9`. Ver [`REINDEX`].

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);

/// **ESPELHAR A ORIENTAÇÃO E A VELOCIDADE do gêmeo** — `0` (default) copia as duas,
/// que é o nó que sempre shipou.
///
/// ⚠️ **É uma reflexão e não uma negação**, e os dois eixos não usam a mesma
/// fórmula: refletir numa reta VERTICAL leva a direção `θ` a `180° − θ`; numa
/// HORIZONTAL, a `−θ`. Escrever `−θ` para os dois (o erro fácil) daria um espelho
/// certo só metade das vezes, e a metade errada **parece** plausível na tela.
///
/// ⚠️ **`rot` e `vel` viajam juntos porque são a mesma pergunta.** Espelhar só a
/// orientação poria o peixe virado para a esquerda a nadar para a direita — pior
/// que não espelhar nada, porque o defeito passa a estar DENTRO do elemento.
///
/// ⚠️ **Transcendental-free (HR-5) na mesma:** `180 − θ` e `−(vx)` são aritmética.
/// Este nó nunca fez trig e continua a não fazer.
const FLIP_ROT: &str = "flip_rot";

/// **A RENUMERAÇÃO** — `0` (default) mantém as colunas de identidade que sempre
/// saíram daqui.
///
/// ⚠️ **Escreve `Index` E `Count`, e mesmo que nenhuma entrada as trouxesse** — a
/// assimetria com o `motion.sort` (que só REESCREVE o `Index` existente) é medida,
/// não de gosto: aquele preserva a contagem, então o `Count` já está honesto e uma
/// lista sem `Index` cai no atalho posicional do `motion.tint`, que **já é** a
/// resposta certa. Este DOBRA a contagem: um `Count` que continue a dizer `n` mente
/// a quem o ler, e meia cura faria a rampa alcançar metade duas vezes. É a mesma
/// escolha que o `motion.combine` faz, pelo mesmo motivo.
///
/// ⚠️ **O default é `0`, ao contrário do `motion.sort`, e também isso é medido.**
/// Lá, desligado, o nó ficava *invisível ao seu único consumidor* e a promessa do
/// próprio doc-comment era falsa. Aqui o estado de hoje tem **leitura de produto**:
/// o `Index` repetido faz cada metade espelhada ler a rampa INTEIRA, que é
/// plausivelmente o que se quer de um espelho. Ligar é pedir *"uma lista só"*.
const REINDEX: &str = "reindex";

/// **O QUE FICA no fim** (doc 89 folha 05 — o 3.º modo *"Discard original"* do Inkscape, que a
/// `line/Vector` W6.3 nomeou e não construiu; no idioma de layout, espelhar sem duplicar).
///
/// - `0` **Both** (o default): os `2n` de sempre — originais e depois os gêmeos.
/// - `1` **Reflection only**: fica só a metade espelhada, `n` elementos.
///
/// ⚠️ **A célula media a composição e ela EXISTE** — `motion.cull(mode = Fraction, amount = 0.5,
/// invert = on)` fica com a segunda metade. Isto entra na mesma pela razão que fechou metade da
/// folha 04: *dois nós para um estado de um enum*, e o segundo deles obriga o artista a saber
/// que este nó emite **originais primeiro e gêmeos depois** — uma ordem que é detalhe de
/// implementação e que ele teria de decorar para escrever o `invert` no sentido certo.
///
/// ⚠️ **O corte acontece DEPOIS de tudo**, e é isso que o torna barato e correcto: a reflexão é
/// a mesma, o `flip_rot` é o mesmo, e o [`reindex`] renumera **o que sobrou** — que é a resposta
/// certa, porque uma lista de `n` que se diz `0..2n` mente para todo nó a jusante.
///
/// ⚠️ **E o espelho continua a ser em torno do CENTROIDE do que entrou**, não do que sai: o
/// centroide da metade espelhada é outro ponto, e recalculá-lo faria o `offset` significar
/// coisas diferentes nos dois modos.
const KEEP: &str = "keep";
/// Só o reflexo — ver [`KEEP`].
const KEEP_REFLECTION: i32 = 1;
/// As palavras que o painel mostra, na ordem dos números.
const KEEP_LABELS: &[&str] = &["Both", "Reflection Only"];

/// As duas colunas de identidade — *quem é este elemento na lista*. Os mesmos
/// nomes que o `motion.sort` e o `motion.combine` usam.
const INDEX: &str = "Index";
const COUNT: &str = "Count";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.mirror"),
    name: "motion.mirror",
    inputs: &[PortSpec {
        name: "in",
        ty: INST_VEC2,
    }],
    outputs: &[PortSpec {
        name: "out",
        ty: INST_VEC2,
    }],
    effect: Effect::Pure,
    clock: Clock::Frame,
    params: &[
        // 0 = Vertical axis (reflect x); 1 = Horizontal axis (reflect y).
        ParamSpec {
            name: "axis",
            default: 0.0,
        },
        // Deslocamento da LINHA de espelho a partir do centroide, ao longo da normal
        // do eixo escolhido. `0` = a linha do centroide, o que sempre shipou.
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        // Apendados (doc 89 folha 05). Os dois em `0` = o nó que sempre shipou.
        ParamSpec {
            name: "flip_rot",
            default: 0.0,
        },
        ParamSpec {
            name: "reindex",
            default: 0.0,
        },
        // **O QUE FICA** — ver [`KEEP`]. `0` (Both) ⇒ os `2n` de sempre.
        ParamSpec {
            name: "keep",
            default: 0.0,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// Reflect + duplicate the positions across the mirror line — the axis through the
/// centroid, **deslocada de `offset`** ao longo da normal do eixo. Returns the `2n`
/// positions (originals then their mirror images).
///
/// A reflexão de `q` na reta que passa por `c` com normal unitária `n` é
/// `q − 2·((q − c)·n)·n`; com `c = centroide + offset·n` e `n` unitária isso vira
/// `q − 2·((q − centroide)·n − offset)·n`. Com `offset = 0` a expressão reduz
/// LITERALMENTE a `2·cx − qx` (ou `2·cy − qy`) — o código que já shipava —, e é por
/// isso que o default não pode mover um bit. Segue transcendental-free (HR-5): a
/// normal é um eixo, não um ângulo.
fn mirror_positions(p: &[[f32; 2]], vertical: bool, offset: f32) -> Vec<[f32; 2]> {
    let n = p.len();
    if n == 0 {
        return Vec::new();
    }
    let mut c = p
        .iter()
        .fold([0.0f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
    c = [c[0] / n as f32, c[1] / n as f32];
    let mut out = p.to_vec();
    out.extend(p.iter().map(|q| {
        if vertical {
            [2.0 * (c[0] + offset) - q[0], q[1]]
        } else {
            [q[0], 2.0 * (c[1] + offset) - q[1]]
        }
    }));
    out
}

struct MotionMirror;

impl NodeOp for MotionMirror {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let vertical = ctx.param("axis").round() as i64 == 0;
        let offset = ctx.param("offset");
        let flip = ctx.param(FLIP_ROT) >= 0.5;
        let renumber = ctx.param(REINDEX) >= 0.5;
        let input = ctx.input(0);
        let n = input.count();
        let p: Vec<[f32; 2]> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        let keep_reflection = ctx.param(KEEP).round() as i32 == KEEP_REFLECTION;
        let mirrored = mirror_positions(&p, vertical, offset);
        // Every column is duplicated onto the twin; `P` is reflected, and with
        // [`FLIP_ROT`] the two ORIENTED channels are reflected too.
        let mut out = Stream::new(mirrored.len());
        for (name, col) in input.columns() {
            if name == "P" {
                continue;
            }
            out.set(name.clone(), twin(name.as_str(), col, vertical, flip));
        }
        out.set("P", Column::Vec2(mirrored));
        // ⚠️ O corte vem ANTES do [`reindex`] — ver [`KEEP`]: renumerar `2n` e depois deitar
        // fora metade deixaria a lista a dizer `0..2n` sobre `n` elementos.
        let mut out = if keep_reflection {
            keep_second_half(&out, n)
        } else {
            out
        };
        if renumber {
            reindex(&mut out);
        }
        ctx.emit(out);
    }
}

/// A coluna do par `[original…, gêmeo…]`. Sem [`FLIP_ROT`] o gêmeo é uma cópia;
/// com ele, os dois canais ORIENTADOS são refletidos no mesmo eixo que `P`.
///
/// ⚠️ **A lista é fechada de propósito** (`rot` e `vel`, e nada mais): uma regra
/// por-tipo — *"todo `Vec2` é refletido"* — apanharia `size` e faria metade do
/// cardume nascer com largura negativa.
fn twin(name: &str, col: &Column, vertical: bool, flip: bool) -> Column {
    match (flip, name, col) {
        (true, "rot", Column::Scalar(v)) => Column::Scalar(
            [
                v.clone(),
                v.iter().map(|d| mirror_angle(*d, vertical)).collect(),
            ]
            .concat(),
        ),
        (true, "vel", Column::Vec2(v)) => Column::Vec2(
            [
                v.clone(),
                v.iter().map(|q| mirror_vec(*q, vertical)).collect(),
            ]
            .concat(),
        ),
        _ => dup(col),
    }
}

/// A direção `deg` refletida na reta do espelho. ⚠️ As duas fórmulas diferem —
/// ver [`FLIP_ROT`].
fn mirror_angle(deg: f32, vertical: bool) -> f32 {
    if vertical { 180.0 - deg } else { -deg }
}

/// O vetor refletido: o componente NORMAL à reta troca de sinal, o tangencial fica.
fn mirror_vec(q: [f32; 2], vertical: bool) -> [f32; 2] {
    if vertical {
        [-q[0], q[1]]
    } else {
        [q[0], -q[1]]
    }
}

/// **Reescreve as duas colunas de identidade** para a lista dobrada: `Index = 0..2n−1`
/// e `Count = 2n` em todas as linhas. Ver [`REINDEX`] — escreve as duas mesmo em
/// branco, porque a contagem MUDOU.
/// **Fica só a metade espelhada** — os últimos `n` de cada coluna (ver [`KEEP`]).
///
/// ⚠️ Percorre TODA coluna, não só o `P`: um `size`/`tint`/`id` que ficasse com `2n` linhas
/// sobre um `P` de `n` seria um stream mal-formado, e o modo de falha é a coluna a ler o
/// elemento errado em silêncio.
/// ⚠️ Devolve um stream NOVO em vez de encolher o que entrou: a contagem de um `Stream` é
/// fixada na construção (`Stream::new`), e um stream cujas colunas encolheram sem a contagem
/// os acompanhar seria mal-formado de uma maneira que só um consumidor a jusante veria.
fn keep_second_half(src: &Stream, n: usize) -> Stream {
    fn tail<T: Clone>(v: &[T], n: usize) -> Vec<T> {
        v[v.len().saturating_sub(n)..].to_vec()
    }
    let mut out = Stream::new(n);
    for (name, col) in src.columns() {
        out.set(
            name.clone(),
            match col {
                Column::Scalar(v) => Column::Scalar(tail(v, n)),
                Column::Vec2(v) => Column::Vec2(tail(v, n)),
                Column::Vec3(v) => Column::Vec3(tail(v, n)),
                Column::Vec4(v) => Column::Vec4(tail(v, n)),
            },
        );
    }
    out
}

fn reindex(out: &mut Stream) {
    let n = out.count();
    #[expect(clippy::cast_precision_loss, reason = "uma contagem de elementos")]
    let idx: Vec<f32> = (0..n).map(|i| i as f32).collect();
    #[expect(clippy::cast_precision_loss, reason = "uma contagem de elementos")]
    let total = n as f32;
    out.set(INDEX, Column::Scalar(idx));
    out.set(COUNT, Column::Scalar(vec![total; n]));
}

/// Duplicate a column onto its mirror twin (`[a, b] → [a, b, a, b]`).
fn dup(col: &Column) -> Column {
    match col {
        Column::Scalar(v) => Column::Scalar([v.clone(), v.clone()].concat()),
        Column::Vec2(v) => Column::Vec2([v.clone(), v.clone()].concat()),
        Column::Vec3(v) => Column::Vec3([v.clone(), v.clone()].concat()),
        Column::Vec4(v) => Column::Vec4([v.clone(), v.clone()].concat()),
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionMirror))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Mirror",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    Ok(())
}

use ph2d_node_registry::{ParamUiHint, ParamUnit, ParamUnitDecl, ParamWidget};

static PARAM_HINTS: &[ParamUiHint] = &[
    ParamUiHint {
        param: "axis",
        label: "Axis",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: &["Vertical", "Horizontal"],
        },
    },
    ParamUiHint {
        param: "offset",
        label: "Axis Offset",
        // Simetrico em torno do centroide: a linha anda para os DOIS lados, e o
        // teto e o piso sao a folga de autoria confortavel, nao um recurso.
        min: -400.0,
        max: 400.0,
        step: 1.0,
        widget: ParamWidget::Slider,
    },
    // ⚠️ Toggles, não sliders: espelhar meia orientação e renumerar meia lista não
    // querem dizer nada, e um slider convidaria a procurar o meio.
    ParamUiHint {
        param: FLIP_ROT,
        label: "Flip Orientation",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    ParamUiHint {
        param: REINDEX,
        label: "Reindex",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Toggle,
    },
    // ⚠️ Um `Enum` e não um Toggle: *"Reflection Only"* diz o que fica, enquanto *"descartar o
    // original"* pedia para se adivinhar qual dos dois é o original. É a mesma escolha que o
    // `mode` do `motion.spline_wrap` fez pelo vocabulário da referência.
    ParamUiHint {
        param: KEEP,
        label: "Keep",
        min: 0.0,
        max: 1.0,
        step: 1.0,
        widget: ParamWidget::Enum {
            labels: KEEP_LABELS,
        },
    },
];

/// O deslocamento e uma DISTANCIA de mundo — a fronteira de display (doc 88, Wave A)
/// a mostra em px ou m conforme a escolha do projeto, e o store fica em metros.
static PARAM_UNITS: &[ParamUnitDecl] = &[ParamUnitDecl {
    param: "offset",
    unit: ParamUnit::Length,
}];

#[cfg(test)]
mod tests {
    use super::*;

    /// A vertical mirror doubles the count and reflects each element's x across the
    /// centroid, keeping y. FALSIFIED if the twin were a plain copy (x unchanged).
    #[test]
    fn vertical_mirror_reflects_x_and_doubles() {
        // Two points, centroid x = 1.
        let p = vec![[0.0, 2.0], [2.0, -1.0]];
        let out = mirror_positions(&p, true, 0.0);
        assert_eq!(out.len(), 4, "count doubled");
        assert_eq!(&out[0..2], &p[..], "originals kept");
        // Reflected across cx=1: (0,2)→(2,2), (2,−1)→(0,−1).
        assert_eq!(out[2], [2.0, 2.0]);
        assert_eq!(out[3], [0.0, -1.0]);
    }

    /// A horizontal mirror reflects y instead.
    #[test]
    fn horizontal_mirror_reflects_y() {
        let p = vec![[1.0, 0.0], [-1.0, 4.0]]; // centroid y = 2
        let out = mirror_positions(&p, false, 0.0);
        assert_eq!(out[2], [1.0, 4.0]); // (1,0) → (1, 4)
        assert_eq!(out[3], [-1.0, 0.0]); // (−1,4) → (−1, 0)
    }

    /// The reflected set is symmetric: its centroid equals the original's (the mirror
    /// adds no net drift).
    #[test]
    fn the_mirrored_layout_is_symmetric() {
        let p = vec![[0.5, 1.0], [3.0, -2.0], [1.5, 0.5]];
        let out = mirror_positions(&p, true, 0.0);
        let c = out
            .iter()
            .fold([0.0f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
        let c = [c[0] / out.len() as f32, c[1] / out.len() as f32];
        let orig_cx = p.iter().map(|q| q[0]).sum::<f32>() / p.len() as f32;
        assert!((c[0] - orig_cx).abs() < 1e-4, "centroid preserved");
    }

    /// Cooks through the registry: every column is duplicated (length `2n`) and `P`
    /// is reflected.
    #[test]
    fn registers_and_mirrors_through_the_cook() {
        use ph2d_nodegraph::cook::{Cook, OpResolver};
        use ph2d_nodegraph::graph::{Edge, Graph};

        static SRC: NodeManifest = NodeManifest {
            id: NodeTypeId::of("motion.mirror.test.src"),
            name: "motion.mirror.test.src",
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
                ctx.emit(
                    Stream::new(2)
                        .with("P", Column::Vec2(vec![[0.0, 0.0], [4.0, 0.0]]))
                        .with("size", Column::Vec2(vec![[0.4, 0.4], [0.4, 0.4]])),
                );
            }
        }
        struct Ops;
        impl OpResolver for Ops {
            fn resolve(&self, ty: NodeTypeId) -> Option<&dyn NodeOp> {
                match ty {
                    t if t == SRC.id => Some(&Src),
                    t if t == MANIFEST.id => Some(&MotionMirror),
                    _ => None,
                }
            }
        }
        let mut reg = NodeRegistry::new();
        register(&mut reg).unwrap();
        assert!(reg.resolve(MANIFEST.id).is_some());

        let mut g = Graph::new();
        let src = g.add_node("motion.mirror.test.src");
        let m = g.add_node("motion.mirror");
        g.connect(Edge {
            from: (src, 0),
            to: (m, 0),
            delayed: false,
        })
        .unwrap();
        let mut cook = Cook::new();
        let out = cook.cook(&g, &Ops, m, 0.0).unwrap();
        let s = out[0].as_stream();
        assert_eq!(s.count(), 4, "doubled");
        match s.get("size").unwrap() {
            Column::Vec2(v) => assert_eq!(v.len(), 4, "size duplicated onto the twin"),
            _ => panic!("size"),
        }
        // Sem offset autorado, a linha e a do centroide (cx = 2): (0,0) -> (4,0).
        match s.get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(v[2], [4.0, 0.0]),
            _ => panic!("P"),
        }

        // ⚠️ E o param AUTORADO tem de ATRAVESSAR o cook. O gate do kernel prova a
        // matematica e e CEGO a um `ctx.param` que ninguem chamou: uma capacidade sem
        // porta passa em todo gate que so olha para a funcao pura.
        g.set_param(m, "offset", 3.0);
        let out = cook.cook(&g, &Ops, m, 0.0).unwrap();
        match out[0].as_stream().get("P").unwrap() {
            Column::Vec2(v) => assert_eq!(
                v[2],
                [10.0, 0.0],
                "o offset autorado no grafo nao chegou ao no"
            ),
            _ => panic!("P"),
        }
    }

    /// **A LINHA DE ESPELHO ANDA** (doc 88 §B3).
    ///
    /// ⚠️ Nasceu VERMELHO: o eixo era pregado no centroide, entao espelhar contra
    /// qualquer outra linha era inexprimivel. O oraculo e a POSICAO da linha, derivada
    /// do resultado — o ponto medio entre um original e o seu gemeo — e nao os valores
    /// crus: e ela que o param nomeia, e ela nao se move em offset nenhum se o numero
    /// for descartado.
    #[test]
    fn the_axis_offset_moves_the_mirror_line() {
        let p = vec![[0.0, 0.0], [4.0, 0.0]]; // centroide x = 2
        let out = mirror_positions(&p, true, 3.0);
        // A linha esta em cx + offset = 5.
        let line = (out[0][0] + out[2][0]) / 2.0;
        assert!(
            (line - 5.0).abs() < 1e-5,
            "a linha tinha de estar em 5.0 e o resultado a poe em {line} —              um offset descartado a deixaria em 2.0 para sempre"
        );
        assert_eq!(out[2], [10.0, 0.0]);
        assert_eq!(out[3], [6.0, 0.0]);
        // E o eixo escolhido continua sendo o unico que o offset move.
        let h = mirror_positions(&p, false, 3.0);
        assert_eq!(h[2], [0.0, 6.0], "no eixo horizontal o offset anda em y");
    }

    /// **O DEFAULT E O MUNDO QUE JA SHIPAVA, AO BIT.**
    ///
    /// A regressao que importa: todo grafo autorado antes desta wave nao declara
    /// `offset`, e o `ctx.param` devolve o default. O gate compara as DUAS rotas —
    /// o zero explicito e o helper — contra os mesmos numeros que os gates de sempre.
    #[test]
    fn the_zero_offset_is_the_centroid_line_to_the_bit() {
        let p = vec![[0.5, 1.0], [3.0, -2.0], [1.5, 0.5]];
        for vertical in [true, false] {
            let out = mirror_positions(&p, vertical, 0.0);
            let cx = p.iter().map(|q| q[0]).sum::<f32>() / p.len() as f32;
            let cy = p.iter().map(|q| q[1]).sum::<f32>() / p.len() as f32;
            for (i, q) in p.iter().enumerate() {
                let want = if vertical {
                    [2.0 * cx - q[0], q[1]]
                } else {
                    [q[0], 2.0 * cy - q[1]]
                };
                assert_eq!(out[p.len() + i], want, "vertical={vertical} i={i}");
            }
        }
    }
}

#[cfg(test)]
#[path = "twin_tests.rs"]
mod twin_tests;

#[cfg(test)]
#[path = "keep_tests.rs"]
mod keep_tests;
