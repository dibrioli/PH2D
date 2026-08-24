#![forbid(unsafe_code)]
//! `motion.spline_wrap` — **wrap a layout onto a curve**: the Cinema 4D "Spline Wrap"
//! deformer (Motion Nodes M3, deformers — doc 01 §3 / doc 28). Where `motion.bend` bows
//! a layout along a *circular arc*, this bends it along an **arbitrary cubic Bézier**
//! (authored by four control-point params) — a banner following an S, text running along
//! a path. Strictly more general than `bend`.
//!
//! **Algorithm — map the layout's X onto the curve's arc length, offset by Y along the
//! normal.** Each element's `x` is normalised over the layout's bounding box to
//! `u ∈ [0, 1]`, slid by `offset` (wrapping); the point at arc position `u` gives a
//! frame `(B, tangent, normal)`, and the element lands at `B + normal · (y ·
//! height_scale)` — so the layout's rows follow the curve and its columns stack along the
//! normal. An `amount` **value** input blends flat → wrapped (unconnected → fully
//! wrapped), so a `value.lfo` flattens and re-wraps it. Falloff-masked. Self-contained:
//! the curve is in the params (not the vector document). Transcendental-free (HR-5):
//! polynomial curve + `sqrt` normalisation, no trig. `Effect::Pure`.
//!
//! ## `follow_rotation` (doc 89 folha 04 — o P0, e o mais VISÍVEL da família)
//!
//! Sem ele um sprite embrulhado num S **mantinha a rotação original**, e texto
//! numa curva que não gira lê como quebrado. Ligado, cada elemento soma o ângulo
//! da tangente unitária do frame — o mesmo frame que este nó já computava e
//! **jogava fora** (`frame_at` devolve a tangente; o wrap ligava-a a `_t`).
//!
//! ⚠️ **SOMA, não atribui**, e é onde diverge do irmão de propósito: o
//! `motion.distribute_curve` faz `set` porque é uma FONTE e não há nada com que
//! compor; este é um modificador sobre um layout que já pode estar orientado. As
//! duas são a mesma regra — *a rotação da curva entra no que já existe* — vista
//! dos dois lados.
//!
//! ⚠️ **E a volta honra a MESMA máscara que a posição.** Um elemento
//! meio-embrulhado tem de estar meio-virado: mascarar um e não o outro deixaria
//! um sprite em pé numa curva que ele só monta em parte, e o falloff leria como
//! quebrado exactamente onde está a funcionar.

use ph2d_node_registry::{NodeRegistry, RegistryError};
use ph2d_nodegraph::attr::{Column, SIZE_IDENTITY, Stream};
use ph2d_nodegraph::cook::EvalCtx;
use ph2d_nodegraph::effect::Effect;
use ph2d_nodegraph::node::{LoweringKind, NodeManifest, NodeOp, NodeTypeId, ParamSpec, PortSpec};
use ph2d_nodegraph::port::{Clock, Dim, Domain, PortType};

mod curve;
mod taper;
mod trig;
use curve::{ArcLut, EPS, P2, arc_lut, frame_at};

const INST_VEC2: PortType = PortType::new(Domain::Instances, Dim::Vec2, Clock::Frame);
/// The value type of the `amount` input (mirror of `motion.look_at::VALUE`).
const VALUE: PortType = PortType::new(Domain::Instances, Dim::Scalar, Clock::Frame);
const VALUE_COL: &str = "v";

/// The static contract of this node type (ADR-0031).
pub const MANIFEST: NodeManifest = NodeManifest {
    id: NodeTypeId::of("motion.spline_wrap"),
    name: "motion.spline_wrap",
    inputs: &[
        PortSpec {
            name: "in",
            ty: INST_VEC2,
        },
        // Blend flat → wrapped (animatable). Optional: unconnected reads as 1 (fully
        // wrapped).
        PortSpec {
            name: "amount",
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
        // How far the layout's Y offsets the point along the curve normal.
        ParamSpec {
            name: "height_scale",
            default: 1.0,
        },
        // Slides the whole layout along the arc (wraps 0..1).
        // **A PARAMETRIZAÇÃO** — ver [`MODE_KEEP_LENGTH`]. Apendado; `0` (Fit) ⇒ o de sempre.
        ParamSpec {
            name: "mode",
            default: 0.0,
        },
        ParamSpec {
            name: "offset",
            default: 0.0,
        },
        // The REGION of the curve the layout is laid onto, as fractions of arc
        // (the C4D Spline Wrap `From`/`To`). `0, 1` is the whole curve, which is
        // the node that shipped.
        ParamSpec {
            name: "from",
            default: 0.0,
        },
        ParamSpec {
            name: "to",
            default: 1.0,
        },
        // Whether the wrapped element TURNS with the curve. `0` ⇒ `rot` is passed
        // through untouched, which is the node that shipped.
        ParamSpec {
            name: "follow_rotation",
            default: 0.0,
        },
        // **QUAL EIXO DO LAYOUT CORRE NA CURVA** — ver `taper::DIRECTION`. `0` ⇒ o X de sempre.
        ParamSpec {
            name: "direction",
            default: 0.0,
        },
        // **O AFUNILAMENTO AO LONGO DO ARCO** — ver `taper::SIZE_TAPER`. `1, 1` ⇒ o `size` é
        // COPIADO, que é o nó que shipou.
        ParamSpec {
            name: "size_start",
            default: 1.0,
        },
        ParamSpec {
            name: "size_end",
            default: 1.0,
        },
        ParamSpec {
            name: "size_profile",
            default: 0.0,
        },
        // The four control points (world units). Default: a gentle S-curve.
        ParamSpec {
            name: "p0x",
            default: -3.0,
        },
        ParamSpec {
            name: "p0y",
            default: -1.5,
        },
        ParamSpec {
            name: "p1x",
            default: -1.0,
        },
        ParamSpec {
            name: "p1y",
            default: 2.0,
        },
        ParamSpec {
            name: "p2x",
            default: 1.0,
        },
        ParamSpec {
            name: "p2y",
            default: -2.0,
        },
        ParamSpec {
            name: "p3x",
            default: 3.0,
        },
        ParamSpec {
            name: "p3y",
            default: 1.5,
        },
    ],
    lowerings: &[LoweringKind::Cpu],
};

/// O **text** param que nomeia a forma desenhada (o canal do doc 32 — um
/// `ParamSpec` é f32-only e o manifesto é congelado, então um param de string vive
/// no `Graph` ao lado do manifesto, nunca dentro dele). O MESMO nome de param que
/// o `motion.path` usa: os dois perguntam *"qual forma?"*, e um artista que
/// aprendeu a resposta num não a re-aprende no outro.
const PATH_PARAM: &str = "path";

fn scalar_col(s: &Stream, name: &str) -> Vec<f32> {
    match s.get(name) {
        Some(Column::Scalar(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// O `amount` **do elemento `i`** — desligado (vazio) → `1.0`; um valor → todos; N → o dele.
///
/// ⚠️ **Era `vals.first()`, e isso fazia o gesto óbvio DESLIGAR o nó em silêncio** (doc 90 §5,
/// caça aos knobs mortos, 2026-08-22). A porta é do domínio `Instances` — um CAMPO —, e ligar-lhe
/// um `value.instance_field(Ramp)` para *"o embrulho crescer ao longo do layout"* é exactamente o
/// que um artista tenta primeiro. O elemento 0 de uma rampa é **`0.0`**, e `amount = 0` faz a
/// mistura de [`wrap_with_frame`] devolver o ponto original: **a curva inteira deixa de existir**,
/// e os catorze knobs de geometria do nó ficam mudos ao mesmo tempo. Não havia erro, nem log —
/// só o nó a não fazer nada.
///
/// ⚠️ **A lei é a MESMA do [`falloff_at`], deliberadamente** (`0 → neutro · 1 → broadcast ·
/// N → por-elemento`). Uma segunda lei para a mesma pergunta é como duas colunas discordarem
/// sobre o que é «um valor para todos» — e o `1 → broadcast` é o que torna esta mudança
/// **byte-idêntica** para toda cena que já existe: o que antes era lido por `first()` continua a
/// ser lido, porque uma porta com um elemento só é o caso que o `first()` acertava.
fn amount_at(vals: &[f32], i: usize) -> f32 {
    match vals.len() {
        0 => 1.0,
        1 => vals[0],
        _ => vals.get(i).copied().unwrap_or(1.0),
    }
}

/// **Onde na curva pousa a posição `u` do layout** — a pergunta é feita UMA vez.
///
/// `from`/`to` deitam o layout num TRECHO da curva (o *From/To* do C4D Spline
/// Wrap: *"define the spline region where deformations occur, expressed as
/// percentages"*) e `offset` desliza o resultado ao longo do arco. Animar o `to`
/// de `from` até 1 é a REVELAÇÃO (write-on): os elementos saem todos empilhados
/// no começo do trecho e se abrem ao longo da curva.
///
/// ⚠️ `from > to` é LEGÍTIMO e sai de graça: o layout percorre a curva ao
/// contrário. Não há caso degenerado a guardar — `from == to` colapsa tudo num
/// ponto, que é exatamente o quadro inicial de um write-on, e o `clamp` final
/// cobre qualquer valor fora de `[0, 1]`.
#[derive(Clone, Copy)]
struct ArcMap {
    from: f32,
    to: f32,
    offset: f32,
}

impl ArcMap {
    /// ⚠️ Com `from = 0, to = 1` isto reduz LITERALMENTE ao `(u + offset)` que
    /// shipava — `u * 1.0` é `u` e `0.0 + u` é `u` em IEEE-754 para todo `u` não
    /// negativo, e `u` é `(x − xmin) / w`, que nunca é negativo por construção.
    /// A identidade é MEDIDA por gate, não afirmada aqui.
    fn s_at(self, u: f32) -> f32 {
        (self.from + u * (self.to - self.from) + self.offset).clamp(0.0, 1.0)
    }
}

/// **A curva em que este nó embrulha** — a que o artista DESENHOU, ou a cúbica
/// dos oito params.
///
/// ⚠️ **O que é abstraído é a CURVA, nunca o WRAP.** O embrulho pergunta uma coisa
/// só à curva — *"que ponto, que tangente e que normal há na fração de arco
/// `s`?"* — e tudo o mais (`from`/`to`, `offset`, `height_scale`, o `falloff`, o
/// `follow_rotation`) é aritmética sobre a resposta. Ter dois EMBRULHOS, um por
/// representação, seria duas leis a divergir; ter duas CURVAS sob um embrulho é
/// uma lei com duas fontes, e a forma desenhada herda os cinco controles de
/// graça, sem uma linha por controle.
///
/// ⚠️ **As duas amostragens NÃO são duas respostas à mesma pergunta.** A cúbica é
/// analítica e a desenhada é uma polilinha que o shell já achatou — são
/// representações diferentes da mesma ideia, e cada uma tem o seu amostrador
/// exato. O que seria duas portas é *"onde fica a fração de arco `s`"* respondida
/// duas vezes para a MESMA representação, e é por isso que a polilinha vai à
/// folha [`ph2d_arc_length`], compartilhada com o `motion.path`.
#[expect(
    clippy::large_enum_variant,
    reason = "a LUT da cúbica é uma tabela de tamanho FIXO por desenho (a curva é analítica; amostrá-la exige inverter o comprimento de arco), enquanto a polilinha já chega achatada. A `Curve` é construída UMA vez por eval, na pilha, e nunca entra numa coleção — box-á-la trocaria bytes de pilha por uma alocação no caminho quente"
)]
enum Curve<'a> {
    /// O polígono de controle dos oito params — o nó que sempre shipou.
    Cubic { cp: &'a [P2; 4], lut: ArcLut },
    /// A polilinha da forma que o artista desenhou, publicada pelo shell sob o
    /// nome que ela carrega na Hierarquia.
    Drawn { pts: &'a [P2], lut: Vec<f32> },
}

impl<'a> Curve<'a> {
    fn cubic(cp: &'a [P2; 4]) -> Self {
        Self::Cubic {
            cp,
            lut: arc_lut(cp),
        }
    }

    /// A curva desenhada, ou `None` se não há arco a percorrer (forma ausente,
    /// renomeada, apagada, ou um ponto só). ⚠️ `None` faz o nó cair na cúbica —
    /// um DEFORMADOR sem curva não pode emitir nada, senão apagaria o layout do
    /// artista; o irmão `motion.path` emite vazio porque é uma FONTE.
    fn drawn(pts: &'a [P2]) -> Option<Self> {
        let lut = ph2d_arc_length::lut(pts);
        (!lut.is_empty()).then_some(Self::Drawn { pts, lut })
    }

    /// O COMPRIMENTO total do arco, em unidades de mundo — a última entrada da LUT
    /// cumulativa, nos dois braços. É o divisor do modo [`MODE_KEEP_LENGTH`].
    fn length(&self) -> f32 {
        match self {
            Self::Cubic { lut, .. } => lut.last().copied().unwrap_or(0.0),
            Self::Drawn { lut, .. } => lut.last().copied().unwrap_or(0.0),
        }
    }

    /// O ponto, a tangente unitária e a **normal esquerda** unitária em `s`.
    /// A convenção da normal (`⟂` à esquerda) é a MESMA nos dois braços — trocá-la
    /// num deles espelharia o `height_scale` só na curva desenhada.
    fn frame_at(&self, s: f32) -> (P2, P2, P2) {
        match self {
            Self::Cubic { cp, lut } => frame_at(cp, lut, s),
            Self::Drawn { pts, lut } => {
                let (p, ut) = ph2d_arc_length::at(pts, lut, s);
                (p, ut, [-ut[1], ut[0]])
            }
        }
    }
}

/// **`Keep Length` — a parametrização que NÃO estica o layout** (doc 89 folha 04 — C4D Spline
/// Wrap: **Mode: Fit Spline / Keep Length**).
///
/// O nó **sempre** normalizou o `x` sobre a bounding box do layout (`u = (x − xmin)/w`), o que
/// é o *Fit*: seja qual for o tamanho do layout, ele ocupa a curva INTEIRA. A célula media o
/// que faltava e não achava saída — *"a parametrização não é alcançável de fora"*, e é
/// verdade: nenhum nó a jusante desfaz uma normalização que já aconteceu.
///
/// ⚠️ **A cura é trocar o DIVISOR, e é só isso.** Em `Keep Length` o divisor deixa de ser a
/// largura do layout e passa a ser o **comprimento de arco da curva**: um layout de 3 unidades
/// numa curva de 12 ocupa um quarto dela e mantém a sua escala; dobrar o layout faz metade
/// dele. É o que separa *"embrulhe isto na curva"* de *"ponha isto na curva sem o deformar"*.
///
/// ⚠️ **Um layout mais LONGO que a curva satura na ponta**, e isso não é um erro nem precisa de
/// guarda nova: o `ArcMap::s_at` já clampa em `[0, 1]` (o mesmo clamp que o `offset` usa para
/// não dar a volta). O excesso empilha-se no fim da curva, que é o que *keep length* quer dizer
/// quando não há curva que chegue.
///
/// ⚠️ **`Fit` é o default e corre pelo divisor de sempre**, então todo grafo autorado é
/// byte-idêntico — não por um `if` que devolve o mesmo número por outro caminho, mas porque é
/// literalmente a mesma expressão.
const MODE_KEEP_LENGTH: i32 = 1;

/// The multiplicative falloff for element `i` (empty → 1.0).
fn falloff_at(vals: &[f32], i: usize) -> f32 {
    match vals.len() {
        0 => 1.0,
        1 => vals[0],
        _ => vals.get(i).copied().unwrap_or(1.0),
    }
}

/// Wrap `p` onto the curve `cp`: map each element's x (normalised over the bbox, slid by
/// `offset`) to an arc position, offset by y along the normal, blended by `amount` ·
/// `falloff`. A pure function — the whole node.
/// ⚠️ `#[cfg(test)]`: since the frame started being reported this is a pure
/// PROJECTION of [`wrap_with_frame`], not a second law — and a projection with no
/// production caller is exactly the shape that becomes a second answer the day
/// someone reaches for the shorter name. The gates keep it because a test that
/// only wants positions reads better without a `.0`.
#[cfg(test)]
fn wrap(
    p: &[P2],
    curve: &Curve<'_>,
    height_scale: f32,
    map: ArcMap,
    keep_length: bool,
    amount: &[f32],
    falloff: &[f32],
) -> Vec<P2> {
    wrap_with_frame(
        p,
        curve,
        height_scale,
        map,
        keep_length,
        0.0,
        amount,
        falloff,
    )
    .0
}

/// The same wrap, also returning **how much each element turned** — the angle of
/// the curve's unit tangent, in degrees, under the SAME mask as the position.
///
/// ⚠️ The mask is the load-bearing half. An element that is half-wrapped has to be
/// **half-turned**: masking the position and not the rotation would leave a sprite
/// standing straight up on a curve it is only partly riding, and the falloff — the
/// whole point of the mask — would read as broken exactly where it is working.
///
/// ⚠️ And the tangent was **already computed and thrown away** (`frame_at` returns
/// it; the wrap bound it to `_t`). Nothing about this is new geometry: the node
/// always knew which way the curve was going, and simply never said.
/// ⚠️ **E a POSIÇÃO DE ARCO de cada elemento** — a terceira saída, e a razão de ela existir é
/// o afunilamento ([`taper::SIZE_TAPER`]): o `s` nasce aqui, dentro de um `clamp`, e nenhum nó
/// a jusante o pode reconstruir. Devolvê-lo é o que torna o perfil uma propriedade da CURVA em
/// vez de mais uma rampa sobre o layout.
#[expect(
    clippy::too_many_arguments,
    reason = "cada argumento é um param do MANIFEST que a lei lê; agrupá-los num struct só para contar menos põe uma segunda declaração da mesma lista ao lado do manifesto, que é onde as duas divergem. O `ArcMap` é a excepção porque ele responde a UMA pergunta (onde na curva pousa `u`), não a um agrupamento de conveniência"
)]
fn wrap_with_frame(
    p: &[P2],
    curve: &Curve<'_>,
    height_scale: f32,
    map: ArcMap,
    keep_length: bool,
    direction_deg: f32,
    amount: &[f32],
    falloff: &[f32],
) -> (Vec<P2>, Vec<f32>, Vec<f32>) {
    let n = p.len();
    if n == 0 {
        return (Vec::new(), Vec::new(), Vec::new());
    }
    // O quadro LOCAL do embrulho — ver [`taper::DIRECTION`]. Em `0` a base é `(1, 0)` ao bit e
    // `to_local` devolve o ponto intacto, então tudo abaixo é a expressão de sempre.
    let cs = taper::frame_of(direction_deg);
    let local: Vec<P2> = p
        .iter()
        .map(|q| taper::to_local(*q, direction_deg, cs))
        .collect();
    let (mut xmin, mut xmax) = (f32::MAX, f32::MIN);
    for q in &local {
        xmin = xmin.min(q[0]);
        xmax = xmax.max(q[0]);
    }
    let w = xmax - xmin;
    // O DIVISOR é a parametrização — ver [`MODE_KEEP_LENGTH`]. Em `Fit` é a largura do
    // layout (a expressão de sempre); em `Keep Length`, o comprimento da curva.
    let denom = if keep_length { curve.length() } else { w };
    let (mut out, mut turn, mut arc) = (
        Vec::with_capacity(n),
        Vec::with_capacity(n),
        Vec::with_capacity(n),
    );
    for i in 0..n {
        let u = if denom < EPS {
            0.5
        } else {
            (local[i][0] - xmin) / denom
        };
        // Clamp (not wrap): the layout spans the curve [0,1]; `offset` slides it and
        // clamps at the ends, so the endpoint (u=1) stays on the curve end.
        let s = map.s_at(u);
        let (b, ut, un) = curve.frame_at(s);
        // ⚠️ O desvio perpendicular é o `y` do quadro LOCAL — é o que faz `direction` escolher
        // o eixo em vez de apenas re-ordenar as coisas ao longo da curva.
        let wrapped = [
            b[0] + un[0] * local[i][1] * height_scale,
            b[1] + un[1] * local[i][1] * height_scale,
        ];
        // ⚠️ E a mistura é contra o ponto de MUNDO, nunca contra o local: `amount` mistura
        // *plano ↔ embrulhado*, e o plano é onde o elemento de facto está.
        let a = (amount_at(amount, i) * falloff_at(falloff, i)).clamp(0.0, 1.0);
        out.push([
            p[i][0] + (wrapped[0] - p[i][0]) * a,
            p[i][1] + (wrapped[1] - p[i][1]) * a,
        ]);
        turn.push(trig::deg(trig::atan2_approx(ut[1], ut[0])) * a);
        arc.push(s);
    }
    (out, turn, arc)
}

struct MotionSplineWrap;

impl NodeOp for MotionSplineWrap {
    fn manifest(&self) -> &'static NodeManifest {
        &MANIFEST
    }

    fn eval(&self, ctx: &mut EvalCtx<'_>) {
        let follow = ctx.param("follow_rotation") >= 0.5;
        let height_scale = ctx.param("height_scale");
        let keep_length = ctx.param("mode").round() as i32 == MODE_KEEP_LENGTH;
        let map = ArcMap {
            from: ctx.param("from"),
            to: ctx.param("to"),
            offset: ctx.param("offset"),
        };
        let cp = [
            [ctx.param("p0x"), ctx.param("p0y")],
            [ctx.param("p1x"), ctx.param("p1y")],
            [ctx.param("p2x"), ctx.param("p2y")],
            [ctx.param("p3x"), ctx.param("p3y")],
        ];
        let amount = scalar_col(ctx.input(1), VALUE_COL);
        // **A forma que o artista DESENHOU**, se ele nomeou uma. O shell publica
        // toda forma vetorial nomeada como uma polilinha sob o nome que ela
        // carrega na Hierarquia (`motion_bridge_shapes`) — o mesmo canal, o mesmo
        // widget e o mesmo gesto que o `motion.path` já usava. Vazio ⇒ a cúbica
        // dos oito params, **byte-idêntica** ao nó que shipava.
        //
        // ⚠️ **Pelo canal da GEOMETRIA (`curve_of`), NUNCA pelo nome cru** — o
        // nome cru carrega a APARÊNCIA da forma assada (uma instância na origem),
        // que o publicador de objetos escreve por cima da polilinha todo frame.
        // E **sem fallback para o nome cru**: um fallback seria a segunda porta
        // que reintroduz a colisão, e ela erra devolvendo *um ponto* — que este
        // nó lê como *"sem arco"* e trata como forma ausente. Silêncio outra vez.
        let name = ctx.text_param(PATH_PARAM).unwrap_or_default().to_string();
        let key = ph2d_nodegraph::external::curve_of(&name);
        let drawn: Vec<P2> = match ctx.external(&key).get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        };
        let input = ctx.input(0);
        let n = input.count();
        let p: Vec<P2> = match input.get("P") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => vec![[0.0, 0.0]; n],
        };
        let falloff = scalar_col(input, "falloff");
        let curve = Curve::drawn(&drawn).unwrap_or_else(|| Curve::cubic(&cp));
        let (out_p, turn, arc) = wrap_with_frame(
            &p,
            &curve,
            height_scale,
            map,
            keep_length,
            ctx.param(taper::DIRECTION),
            &amount,
            &falloff,
        );
        // The element's OWN rotation composes with the curve's frame — this is a
        // modifier on a layout that may already be oriented, not a source that
        // mints one (its sibling `motion.distribute_curve` SETS `rot` because
        // there is nothing there to compose with).
        let base = scalar_col(input, "rot");
        // **O AFUNILAMENTO ao longo do arco** — ver [`taper::SIZE_TAPER`]. Com as duas pontas
        // em `1` a coluna `size` é COPIADA pelo laço abaixo, como o `rot` com o follow
        // desligado: byte-idêntico por ESTRUTURA, e um stream sem `size` continua sem ele.
        let taper = taper::Taper {
            start: ctx.param(taper::SIZE_TAPER.0),
            end: ctx.param(taper::SIZE_TAPER.1),
            profile: ctx.param(taper::SIZE_TAPER.2).round() as i32,
        };
        let tapering = !taper.is_identity();
        let size_base: Vec<[f32; 2]> = match input.get("size") {
            Some(Column::Vec2(v)) => v.clone(),
            _ => Vec::new(),
        };
        let mut out = Stream::new(n);
        for (name, col) in input.columns() {
            // ⚠️ With `follow_rotation` off, `rot` is copied through like every
            // other column — not written with an unchanged value, COPIED. So a
            // stream that never had one still does not, and the default is the
            // node that shipped by STRUCTURE rather than by arithmetic.
            if name != "P" && !(follow && name == "rot") && !(tapering && name == "size") {
                out.set(name.clone(), col.clone());
            }
        }
        out.set("P", Column::Vec2(out_p));
        if follow {
            let rot: Vec<f32> = (0..n)
                .map(|i| base.get(i).copied().unwrap_or(0.0) + turn[i])
                .collect();
            out.set("rot", Column::Scalar(rot));
        }
        if tapering {
            // ⚠️ O afunilamento **MULTIPLICA** o que já lá está (a identidade da coluna é
            // `[1, 1]`, e é ela que um stream sem `size` traz): é um modificador sobre um
            // layout que um `motion.scale` a montante pode já ter dimensionado, a mesma lei
            // do `rot` acima. E ele honra a MESMA máscara que a posição — um elemento
            // meio-embrulhado é meio-afunilado, senão o falloff leria como quebrado
            // exactamente onde está a funcionar.
            let size: Vec<[f32; 2]> = (0..n)
                .map(|i| {
                    let b = size_base.get(i).copied().unwrap_or(SIZE_IDENTITY);
                    let a = (amount_at(&amount, i) * falloff_at(&falloff, i)).clamp(0.0, 1.0);
                    let k = 1.0 + (taper.at(arc[i]) - 1.0) * a;
                    [b[0] * k, b[1] * k]
                })
                .collect();
            out.set("size", Column::Vec2(size));
        }
        ctx.emit(out);
    }
}

/// Register this node with the runtime registry. Called (via codegen) from
/// `ph2d-node-registry-init::register_all_nodes`.
pub fn register(reg: &mut NodeRegistry) -> Result<(), RegistryError> {
    reg.register(Box::new(MotionSplineWrap))?;
    reg.register_ui(
        MANIFEST.id,
        ph2d_node_registry::NodeUiManifest {
            display_name: "Spline Wrap",
            category: ph2d_node_registry::NodeUiCategory::Transform,
            silhouette: ph2d_node_registry::NodeSilhouette::Rect,
        },
    );
    reg.register_param_ui(MANIFEST.id, PARAM_HINTS);
    reg.register_param_groups(MANIFEST.id, PARAM_GROUPS);
    // ⚠️ **Os oito sliders SOMEM quando há forma** — é o que a queixa pedia. Com
    // uma forma escolhida o polígono de controle não é lido por rota nenhuma, e
    // *um controle que não faz nada não é pintado* (a lei do `amount_y` do
    // `motion.scale`). O `ParamGate` comum não sabe fazer esta pergunta: ele
    // decide pelo valor de outro param **f32**, e um nome de forma vive no canal
    // de TEXTO — daí o irmão `ParamGateText`.
    reg.register_param_gates_text(MANIFEST.id, PARAM_GATES_TEXT);
    reg.register_param_units(MANIFEST.id, PARAM_UNITS);
    // CPU-only: this node reads `falloff` only at eval runtime (no GPU kernel), so the
    // diagnoser cannot derive the role from a `ColumnBinding` — declare it (ADR-0155).
    reg.register_couplings(
        MANIFEST.id,
        &[ph2d_node_registry::Coupling::Consumes("falloff")],
    );
    Ok(())
}

mod ui;
use ui::{PARAM_GATES_TEXT, PARAM_GROUPS, PARAM_HINTS, PARAM_UNITS};
// ⚠️ Re-exportado para os cinco arquivos de teste irmãos, que o alcançam por
// `use super::*` desde antes do split — mover a superfície de painel não pode
// mover o vocabulário que eles já usavam.
#[cfg(test)]
use ph2d_node_registry::ParamWidget;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "follow_tests.rs"]
mod follow_tests;

#[cfg(test)]
#[path = "axis_taper_tests.rs"]
mod axis_taper_tests;

#[cfg(test)]
#[path = "drawn_tests.rs"]
mod drawn_tests;

#[cfg(test)]
#[path = "range_tests.rs"]
mod range_tests;

#[cfg(test)]
#[path = "keep_length_tests.rs"]
mod keep_length_tests;
