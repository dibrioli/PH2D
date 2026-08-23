//! **O gizmo de canvas dos DEFORMADORES DE QUADRILÁTERO** — as alças que o
//! `motion.four_point_warp` e o `motion.bezier_warp` passam a ter na tela.
//!
//! ## Por que ele existe (Enio, 2026-08-23)
//!
//! > *"não faz sentido um nó com milhões de inputs e sem nada visual. Seremos um app
//! > para artistas. Melhor ter o Bezier Warp desenhado no canvas com a possibilidade de
//! > manipular os seus pontos manualmente no canvas. Mas podemos deixar os inputs como
//! > sliders tb para o caso de utilizar em grafos"* — e, a seguir, *"a mesma coisa pra
//! > four point warp"*.
//!
//! ⚠️ **Os sliders FICAM.** Um param que só existe no canvas não pode ser dirigido por
//! um fio, e é por fio que estes nós se animam — as duas superfícies servem gestos
//! diferentes e nenhuma substitui a outra. O que muda é qual delas o artista alcança
//! primeiro.
//!
//! ## Um gizmo, DOIS nós
//!
//! Os dois deformam o mesmo quadrilátero e nomeiam os cantos com os MESMOS params
//! (`tl_dx`…`bl_dy`) — o `motion.bezier_warp` acrescenta as oito tangentes. Então isto
//! é **uma** tabela com um [`WarpGizmoSpec`] por tipo de nó, no molde exacto do
//! `field_gizmo::spec_for`, e não dois gizmos que se parecem.
//!
//! ## ⚠️ A fronteira desenhada é a que o NÓ computa
//!
//! O contorno vem de [`ph2d_node_motion_bezier_warp::coons`] — a mesma função que o
//! `eval` chama. Um overlay com a sua própria Bézier seria um segundo motor sobre o
//! mesmo estado, e os dois divergiriam no dia em que um mudasse.
//!
//! ## O que este módulo NÃO faz
//!
//! Ele é **puro**: recebe a caixa envolvente e uma porta de leitura de param, e devolve
//! alças em coordenadas de MUNDO, o contorno para desenhar, e as edições de um arrasto.
//! Quem sabe do ponteiro é a costura; quem pinta é o overlay.

use ph2d_node_motion_bezier_warp::coons::{
    self, BL, BOTTOM, BR, Boundary, LEFT, RIGHT, TL, TOP, TR,
};
use ph2d_nodegraph::node::NodeTypeId;

/// Quantas alças o maior dos dois nós oferece: 4 cantos + 8 tangentes.
pub(crate) const MAX_HANDLES: usize = 12;

/// **O RAIO DE AGARRE, em pixels de TELA.**
///
/// ⚠️ Em pixels de tela e não de mundo, e a distinção é o gesto: o dedo do artista tem o
/// mesmo tamanho em todo nível de zoom, então um raio em unidades de mundo ficaria
/// inagarrável ao afastar e agarraria meia tela ao aproximar. É a mesma lei do braço da
/// alça de rotação da âncora, do outro lado (aquele é de MUNDO de propósito, porque
/// descreve uma distância geométrica; este descreve um DEDO).
pub(crate) const GRAB_PX: f32 = 11.0;

/// Que alça é esta.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WarpHandleKind {
    /// Um dos quatro cantos (índice na ordem TL, TR, BR, BL).
    Corner(usize),
    /// Uma tangente: `(lado, qual)` — lado em TOP/RIGHT/BOTTOM/LEFT, qual em 0/1.
    Tangent(usize, usize),
}

/// Uma alça, no MUNDO, com os dois params que ela escreve.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WarpHandle {
    pub(crate) kind: WarpHandleKind,
    pub(crate) world: [f32; 2],
    /// Os nomes dos params `dx` e `dy` desta alça — a porta pela qual o arrasto sai.
    pub(crate) param: [&'static str; 2],
}

/// O que um tipo de nó oferece.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WarpGizmoSpec {
    /// `true` ⇒ o nó tem as oito tangentes (o `motion.bezier_warp`).
    pub(crate) has_tangents: bool,
}

/// Os nomes dos offsets de CANTO, na ordem TL, TR, BR, BL — **os mesmos nos dois nós**,
/// e é isso que faz um gizmo servir os dois.
const CORNER_PARAMS: [[&str; 2]; 4] = [
    ["tl_dx", "tl_dy"],
    ["tr_dx", "tr_dy"],
    ["br_dx", "br_dy"],
    ["bl_dx", "bl_dy"],
];

/// Os nomes dos offsets de TANGENTE, por lado (TOP, RIGHT, BOTTOM, LEFT).
const TANGENT_PARAMS: [[[&str; 2]; 2]; 4] = [
    [["top_a_dx", "top_a_dy"], ["top_b_dx", "top_b_dy"]],
    [["right_a_dx", "right_a_dy"], ["right_b_dx", "right_b_dy"]],
    [
        ["bottom_a_dx", "bottom_a_dy"],
        ["bottom_b_dx", "bottom_b_dy"],
    ],
    [["left_a_dx", "left_a_dy"], ["left_b_dx", "left_b_dy"]],
];

/// **A tabela: que nós têm gizmo, e o que cada um oferece.**
///
/// ⚠️ Uma tabela e não um `impl` por nó, pelo motivo do irmão `field_gizmo::spec_for`:
/// um nó que ganhe alças amanhã entra com **uma linha**, e um nó que não esteja aqui
/// não tem hit-region nenhuma — a ausência é o default seguro.
pub(crate) fn spec_for(ty: NodeTypeId) -> Option<WarpGizmoSpec> {
    if ty == NodeTypeId::of("motion.four_point_warp") {
        return Some(WarpGizmoSpec {
            has_tangents: false,
        });
    }
    if ty == NodeTypeId::of("motion.bezier_warp") {
        return Some(WarpGizmoSpec { has_tangents: true });
    }
    None
}

/// A caixa envolvente do layout que ENTRA no nó — o quadro a que os offsets se referem.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WarpBox {
    pub(crate) lo: [f32; 2],
    pub(crate) hi: [f32; 2],
}

impl WarpBox {
    /// A caixa de um conjunto de pontos. `None` quando ela é degenerada (uma linha ou
    /// um ponto) — ali o nó passa o layout verbatim, e um gizmo sobre uma caixa que não
    /// existe desenharia alças que não fazem nada.
    pub(crate) fn of(p: &[[f32; 2]]) -> Option<Self> {
        if p.is_empty() {
            return None;
        }
        let mut lo = [f32::MAX; 2];
        let mut hi = [f32::MIN; 2];
        for q in p {
            for k in 0..2 {
                lo[k] = lo[k].min(q[k]);
                hi[k] = hi[k].max(q[k]);
            }
        }
        const EPS: f32 = 1e-6;
        if hi[0] - lo[0] < EPS || hi[1] - lo[1] < EPS {
            return None;
        }
        Some(Self { lo, hi })
    }

    /// Um ponto do quadrado unitário, no mundo.
    fn at(self, q: [f32; 2]) -> [f32; 2] {
        [
            self.lo[0] + q[0] * (self.hi[0] - self.lo[0]),
            self.lo[1] + q[1] * (self.hi[1] - self.lo[1]),
        ]
    }
}

/// A fronteira em MUNDO, com os offsets já somados — a mesma construção do `eval`.
///
/// ⚠️ `warp` escala todo offset, como no nó. Um gizmo que o ignorasse poria as alças
/// onde elas **não estão** sempre que a porta `warp` estivesse ligada, e o arrasto
/// pareceria não pegar.
pub(crate) fn boundary(
    spec: WarpGizmoSpec,
    bbox: WarpBox,
    warp: f32,
    param: &dyn Fn(&str) -> f32,
) -> Boundary {
    let unit = Boundary::unit();
    let mut b = Boundary {
        corner: [[0.0; 2]; 4],
        tangent: [[[0.0; 2]; 2]; 4],
    };
    for (c, names) in CORNER_PARAMS.iter().enumerate() {
        let w = bbox.at(unit.corner[c]);
        b.corner[c] = [w[0] + warp * param(names[0]), w[1] + warp * param(names[1])];
    }
    for (s, side) in TANGENT_PARAMS.iter().enumerate() {
        for (t, names) in side.iter().enumerate() {
            let w = bbox.at(unit.tangent[s][t]);
            let (dx, dy) = if spec.has_tangents {
                (param(names[0]), param(names[1]))
            } else {
                // ⚠️ **O Corner Pin não tem tangentes, e elas ficam nos TERÇOS DOS
                // CANTOS MOVIDOS** — não nos terços da caixa. Sem isto o contorno
                // desenhado seria o do quadrilátero de origem, e o overlay mentiria
                // sobre onde as arestas dele estão.
                (0.0, 0.0)
            };
            b.tangent[s][t] = [w[0] + warp * dx, w[1] + warp * dy];
        }
    }
    if !spec.has_tangents {
        // As tangentes seguem os cantos: os terços de cada aresta do quad deformado.
        b.tangent[TOP] = thirds(b.corner[TL], b.corner[TR]);
        b.tangent[RIGHT] = thirds(b.corner[TR], b.corner[BR]);
        b.tangent[BOTTOM] = thirds(b.corner[BR], b.corner[BL]);
        b.tangent[LEFT] = thirds(b.corner[BL], b.corner[TL]);
    }
    b
}

/// As duas tangentes que põem a cúbica exactamente sobre o segmento `a → b`.
fn thirds(a: [f32; 2], b: [f32; 2]) -> [[f32; 2]; 2] {
    let d = [(b[0] - a[0]) / 3.0, (b[1] - a[1]) / 3.0];
    [[a[0] + d[0], a[1] + d[1]], [b[0] - d[0], b[1] - d[1]]]
}

/// **As alças deste nó, no mundo.** Devolve o array e quantas valem.
pub(crate) fn handles(
    spec: WarpGizmoSpec,
    bbox: WarpBox,
    warp: f32,
    param: &dyn Fn(&str) -> f32,
) -> ([WarpHandle; MAX_HANDLES], usize) {
    let b = boundary(spec, bbox, warp, param);
    let blank = WarpHandle {
        kind: WarpHandleKind::Corner(0),
        world: [0.0; 2],
        param: CORNER_PARAMS[0],
    };
    let mut out = [blank; MAX_HANDLES];
    let mut n = 0;
    for (c, names) in CORNER_PARAMS.iter().enumerate() {
        out[n] = WarpHandle {
            kind: WarpHandleKind::Corner(c),
            world: b.corner[c],
            param: *names,
        };
        n += 1;
    }
    if spec.has_tangents {
        for (s, side) in TANGENT_PARAMS.iter().enumerate() {
            for (t, names) in side.iter().enumerate() {
                out[n] = WarpHandle {
                    kind: WarpHandleKind::Tangent(s, t),
                    world: b.tangent[s][t],
                    param: *names,
                };
                n += 1;
            }
        }
    }
    (out, n)
}

/// Quantos segmentos por aresta o contorno desenhado usa.
///
/// ⚠️ **É uma resolução de DESENHO e não de produto:** o nó avalia a curva
/// analiticamente por elemento; isto é só quantas linhas o overlay traça. `16` põe o
/// erro de corda abaixo de meio pixel numa aresta que ocupe a tela inteira, e uma
/// aresta recta (o Corner Pin, ou o Bezier no neutro) sai exacta com qualquer número —
/// a cúbica dos terços É o segmento.
pub(crate) const OUTLINE_SEGMENTS: usize = 16;

/// **O contorno para desenhar**, como um anel fechado de pontos de mundo.
///
/// Percorre as quatro arestas na ordem TL → TR → BR → BL → TL, avaliando a MESMA
/// cúbica que o `eval` do nó avalia.
pub(crate) fn outline(b: &Boundary) -> Vec<[f32; 2]> {
    let mut out = Vec::with_capacity(4 * OUTLINE_SEGMENTS + 1);
    let edges = [
        (
            b.corner[TL],
            b.tangent[TOP][0],
            b.tangent[TOP][1],
            b.corner[TR],
        ),
        (
            b.corner[TR],
            b.tangent[RIGHT][0],
            b.tangent[RIGHT][1],
            b.corner[BR],
        ),
        (
            b.corner[BR],
            b.tangent[BOTTOM][0],
            b.tangent[BOTTOM][1],
            b.corner[BL],
        ),
        (
            b.corner[BL],
            b.tangent[LEFT][0],
            b.tangent[LEFT][1],
            b.corner[TL],
        ),
    ];
    for (p0, p1, p2, p3) in edges {
        for k in 0..OUTLINE_SEGMENTS {
            let t = k as f32 / OUTLINE_SEGMENTS as f32;
            out.push(coons::bezier(p0, p1, p2, p3, t));
        }
    }
    out.push(b.corner[TL]);
    out
}

/// **De que CANTO cada tangente sai** — o braço que o overlay liga.
///
/// ⚠️ Sem o braço, oito pontos soltos à volta de um quadrilátero não dizem a que aresta
/// pertencem, e o artista arrasta o errado. O braço é o que torna a alça legível, e é
/// por isso que ele é uma função aqui e não um detalhe do pintor.
pub(crate) fn tangent_arm(kind: WarpHandleKind) -> Option<usize> {
    match kind {
        WarpHandleKind::Corner(_) => None,
        WarpHandleKind::Tangent(side, which) => Some(match (side, which) {
            (TOP, 0) => TL,
            (TOP, 1) => TR,
            (RIGHT, 0) => TR,
            (RIGHT, 1) => BR,
            (BOTTOM, 0) => BR,
            (BOTTOM, 1) => BL,
            (LEFT, 0) => BL,
            _ => TL,
        }),
    }
}

/// **Qual alça o ponteiro agarra**, dado o ponto em MUNDO e quantas unidades de mundo
/// vale um pixel de tela.
///
/// ⚠️ **A mais PRÓXIMA dentro do raio, nunca a primeira encontrada.** Num quadrilátero
/// pouco deformado a tangente nasce perto do canto, e um "primeiro que couber" faria o
/// canto roubar o gesto da tangente conforme a ordem do array — um bug que se lê como
/// *"esta alça não pega"*.
pub(crate) fn hit(handles: &[WarpHandle], world: [f32; 2], world_per_px: f32) -> Option<usize> {
    let r = GRAB_PX * world_per_px;
    let mut best: Option<(usize, f32)> = None;
    for (i, h) in handles.iter().enumerate() {
        let d = (h.world[0] - world[0]).hypot(h.world[1] - world[1]);
        if d <= r && best.is_none_or(|(_, bd)| d < bd) {
            best = Some((i, d));
        }
    }
    best.map(|(i, _)| i)
}

/// **As edições de um arrasto**: a alça `h` movida por `delta` (mundo) escreve os dois
/// params dela.
///
/// ⚠️ **O `delta` é dividido pelo `warp`**, e não somado cru: os params são o offset
/// ANTES da escala da porta, então com `warp = 0,5` um arrasto de uma unidade tem de
/// escrever DUAS no param para a alça acompanhar o dedo. Sem isto o gizmo escorrega do
/// cursor sempre que a porta estiver ligada — e a lei é a mesma do `boundary`, do outro
/// lado da conta.
///
/// Um `warp` (quase) nulo não tem inverso: ali o arrasto não pode dizer nada sobre o
/// offset, e a função devolve `None` em vez de gerar um infinito.
pub(crate) fn edits(
    h: &WarpHandle,
    start: [f32; 2],
    delta: [f32; 2],
    warp: f32,
) -> Option<[(&'static str, f32); 2]> {
    const MIN_WARP: f32 = 1e-3;
    if !warp.is_finite() || warp.abs() < MIN_WARP {
        return None;
    }
    Some([
        (h.param[0], start[0] + delta[0] / warp),
        (h.param[1], start[1] + delta[1] / warp),
    ])
}

// ─── A ligação ao DOCUMENTO ───────────────────────────────────────────────────────
//
// Daqui para baixo o módulo deixa de ser puro: ele pergunta ao grafo e ao pump. A
// fronteira é de propósito — tudo o que decide GEOMETRIA está acima e é testável sem
// mundo nenhum.

use crate::motion_state::MotionState;
use ph2d_nodegraph::graph::{Graph, NodeId};

/// **Quem alimenta a porta 0 deste nó** — o layout a cuja caixa os offsets se referem.
///
/// ⚠️ **É o nó de CIMA, e não o próprio.** A saída do deformador já está deformada; a
/// caixa que os params usam é a da ENTRADA. Ler a caixa errada faria as alças nascerem
/// fora do sítio assim que o artista mexesse na primeira — e o erro cresceria com o
/// arrasto, que é o modo de falha que se lê como *"o gizmo está a fugir"*.
pub(crate) fn upstream_of(graph: &Graph, node: NodeId) -> Option<NodeId> {
    graph
        .edges()
        .iter()
        .find(|e| e.to == (node, 0))
        .map(|e| e.from.0)
}

/// O nó de warp seleccionado e o que ele oferece, ou `None`.
pub(crate) fn selected_warp(motion: &MotionState) -> Option<(NodeId, WarpGizmoSpec)> {
    let nid = super::motion_bridge::params::selected_motion_node().map(NodeId)?;
    let ty = motion.doc.graph.node(nid)?.type_id();
    Some((nid, spec_for(ty)?))
}

/// **A TOMADA que este gizmo precisa**, para o `motion_bridge` a armar junto das dos
/// sinais.
///
/// ⚠️ **Uma tomada e não um segundo cozimento.** A shell tem grafo e registry e poderia
/// cozinhar o nó de cima por conta própria — e aí haveria DOIS cozimentos do mesmo
/// estado, que é a lei que esta casa já pagou. A bomba já retém streams por nó; o gizmo
/// entra na lista que existe.
pub(crate) fn tap_for(motion: &MotionState) -> Option<NodeId> {
    let (nid, _) = selected_warp(motion)?;
    upstream_of(&motion.doc.graph, nid)
}

/// A caixa envolvente do layout que entra no nó, lida da tomada. `None` quando a tomada
/// não disparou este quadro ou quando a caixa é degenerada.
pub(crate) fn box_from_tap(motion: &MotionState, upstream: NodeId) -> Option<WarpBox> {
    let stream = motion
        .pump
        .tap_streams()
        .iter()
        .find(|(n, _)| *n == upstream)
        .map(|(_, s)| s)?;
    match stream.get("P") {
        Some(ph2d_nodegraph::attr::Column::Vec2(p)) => WarpBox::of(p),
        _ => None,
    }
}

/// **A porta de leitura de param — a MESMA do painel.**
///
/// Seed = sample: a alça é desenhada a partir do número que o slider mostra, então as
/// duas superfícies não podem discordar. É a lei que o `field_gizmo` já pratica, e o
/// pedido do Enio (*"deixar os inputs como sliders tb"*) torna-a obrigatória aqui: as
/// duas existem ao mesmo tempo, na mesma tela.
pub(crate) fn param_port<'a>(
    motion: &'a MotionState,
    node: NodeId,
) -> impl Fn(&str) -> f32 + use<'a> {
    move |name: &str| super::motion_bridge::params::param_value(motion, node, name)
}

/// O valor da porta `warp` deste nó — desligada lê `1`, como o `eval`.
///
/// ⚠️ Ela é uma PORTA e não um param, então `param_value` não a alcança: um fio ligado
/// ali entrega um número por tique que só o cook conhece. O gizmo lê o caso comum
/// (desligada) e, com fio, o arrasto recusa em vez de escrever um offset que a escala
/// desconhecida tornaria errado — ver [`edits`].
pub(crate) fn warp_amount(graph: &Graph, node: NodeId) -> Option<f32> {
    let wired = graph.edges().iter().any(|e| e.to == (node, 1));
    if wired { None } else { Some(1.0) }
}

/// **O retrato do gizmo deste quadro** — tudo o que o pintor precisa, já resolvido.
///
/// ⚠️ Ele existe porque quem SABE (a tool activa, o nó seleccionado, a tomada) e quem
/// PINTA vivem em escopos diferentes do laço de render, e passar seis argumentos por
/// dez chamadas seria pior. É o mesmo idioma da legenda das cenas de conferência:
/// publica-se uma vez, lê-se onde a tinta sai.
///
/// ⚠️ **E publicar de novo SUBSTITUI.** Se ele acumulasse, largar a selecção deixaria as
/// alças do nó anterior a pairar sobre uma figura que já não é dele.
#[derive(Clone, Copy, Debug)]
pub(crate) struct WarpGizmoView {
    pub(crate) node: NodeId,
    pub(crate) spec: WarpGizmoSpec,
    pub(crate) bbox: WarpBox,
    pub(crate) warp: f32,
}

static VIEW: std::sync::Mutex<Option<WarpGizmoView>> = std::sync::Mutex::new(None);

/// Publica (ou limpa) o retrato deste quadro.
pub(crate) fn publish(v: Option<WarpGizmoView>) {
    if let Ok(mut slot) = VIEW.lock() {
        *slot = v;
    }
}

/// O retrato deste quadro, se houver.
pub(crate) fn view() -> Option<WarpGizmoView> {
    VIEW.lock().ok().and_then(|s| *s)
}

/// **Resolve o retrato a partir do estado** — a porta única, para o laço de render e o
/// gate lerem a MESMA resposta.
///
/// `None` quando: a tool não é a Motion · nenhum nó de warp está seleccionado · não há
/// nó a montante · a tomada ainda não trouxe o stream · a caixa é degenerada · a porta
/// `warp` está ligada por fio (ali o gizmo não sabe a escala e recusa em vez de mentir).
pub(crate) fn resolve(motion: &MotionState, tool_is_motion: bool) -> Option<WarpGizmoView> {
    if !tool_is_motion {
        return None;
    }
    let (node, spec) = selected_warp(motion)?;
    let warp = warp_amount(&motion.doc.graph, node)?;
    let upstream = upstream_of(&motion.doc.graph, node)?;
    let bbox = box_from_tap(motion, upstream)?;
    Some(WarpGizmoView {
        node,
        spec,
        bbox,
        warp,
    })
}

#[cfg(test)]
#[path = "warp_gizmo_tests.rs"]
mod tests;
