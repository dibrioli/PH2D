//! **A metade do gizmo de warp que fala com o DOCUMENTO** — cortada do
//! [`super::warp_gizmo`] no teto de LOC (HR-18), pela costura que aquele arquivo já
//! anunciava.
//!
//! Lá em cima é geometria PURA: dá-se uma caixa e uma porta de param, e saem alças em
//! coordenadas de mundo. Aqui pergunta-se ao grafo quem alimenta quem, à bomba o que ela
//! reteve, e à tela em que janela isto vive. *A fronteira é o que mantém doze gates de
//! geometria a correr sem construir um app.*

use super::{
    MAX_HANDLES, WarpBox, WarpGizmoSpec, WarpHandle, boundary, handles, outline, spec_for,
};
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
    let nid = super::super::motion_bridge::params::selected_motion_node().map(NodeId)?;
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
pub(crate) fn taps_for(motion: &MotionState) -> Vec<NodeId> {
    let Some((nid, _)) = selected_warp(motion) else {
        return Vec::new();
    };
    let g = &motion.doc.graph;
    // A ENTRADA dá a caixa; a SAÍDA e o SINK dão, por correspondência de elemento, o que
    // a cadeia de jusante faz. ⚠️ As três, e não a primeira: sem a saída não há de onde
    // medir, e sem o sink não há para onde.
    let mut out = Vec::with_capacity(3);
    if let Some(up) = upstream_of(g, nid) {
        out.push(up);
    }
    out.push(nid);
    if let Some(sink) = sink_of(g, nid) {
        out.push(sink);
    }
    out
}

/// As posições `P` de um nó tapado, se a tomada disparou.
fn tapped_points(motion: &MotionState, node: NodeId) -> Option<Vec<[f32; 2]>> {
    let s = motion
        .pump
        .tap_streams()
        .iter()
        .find(|(n, _)| *n == node)
        .map(|(_, s)| s)?;
    match s.get("P") {
        Some(ph2d_nodegraph::attr::Column::Vec2(p)) => Some(p.clone()),
        _ => None,
    }
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
    move |name: &str| super::super::motion_bridge::params::param_value(motion, node, name)
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

/// **A TRANSFORMAÇÃO DE JUSANTE** — o que os nós DEPOIS do warp fazem às peças, como um
/// afim.
///
/// ## Por que ela existe (Enio, 2026-08-23)
///
/// > *"se o nó Bezier Warp é colocado antes de Transform, a grade é desenhada na posição
/// > (0,0)"*
///
/// E o gizmo estava **certo** e inútil ao mesmo tempo: os params dele são offsets sobre a
/// caixa do que ENTRA, e com o `motion.transform` a jusante o que entra é a grelha crua na
/// origem. O gizmo desenhava no espaço do NÓ; o artista olha para o espaço do que se VÊ.
/// *Um gizmo correcto no frame errado é um gizmo errado.*
///
/// ## ⚠️ Ela é MEDIDA, não presumida
///
/// A shell não pode saber o que uma cadeia arbitrária de nós faz — e não precisa. O que
/// ela tem é **correspondência por elemento**: a peça `i` da saída do warp é a peça `i` do
/// sink. Ajustar um afim por mínimos quadrados sobre 81 pontos e depois **verificar que
/// ele reproduz TODOS** não é um palpite: se reproduz, o composto **é** aquele afim
/// naqueles pontos, e o gizmo desenhado através dele cai exactamente onde o artista vê.
///
/// ⛔ E quando não reproduz — outro deformador a jusante, um `motion.cull` que muda a
/// contagem —, a resposta é a IDENTIDADE, ou seja o comportamento de antes: o gizmo fica
/// no frame do nó. Recusar em silêncio um caso que a medição não cobre é melhor que
/// desenhar uma mentira, e a fronteira fica nomeada em vez de descoberta.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Downstream {
    /// A parte linear, em ordem de linha: `[[a, b], [c, d]]`.
    pub(crate) lin: [[f32; 2]; 2],
    pub(crate) tr: [f32; 2],
}

impl Downstream {
    /// O afim que não faz nada — a resposta quando a medição não cobre a cadeia.
    pub(crate) const IDENTITY: Self = Self {
        lin: [[1.0, 0.0], [0.0, 1.0]],
        tr: [0.0, 0.0],
    };

    /// Um ponto, empurrado para o espaço do que se vê.
    pub(crate) fn apply(self, p: [f32; 2]) -> [f32; 2] {
        [
            self.lin[0][0] * p[0] + self.lin[0][1] * p[1] + self.tr[0],
            self.lin[1][0] * p[0] + self.lin[1][1] * p[1] + self.tr[1],
        ]
    }

    /// Um DESLOCAMENTO, de volta ao espaço do nó — a parte linear invertida, sem a
    /// translação.
    ///
    /// ⚠️ **Sem translação, e isso não é um detalhe:** um delta é a diferença de dois
    /// pontos, e a translação cancela-se nela. Aplicá-la faria o arrasto saltar pela
    /// posição do transform a jusante logo no primeiro pixel.
    pub(crate) fn unapply_delta(self, d: [f32; 2]) -> Option<[f32; 2]> {
        let [[a, b], [c, e]] = self.lin;
        let det = a * e - b * c;
        if !det.is_finite() || det.abs() < 1e-9 {
            return None;
        }
        Some([(e * d[0] - b * d[1]) / det, (-c * d[0] + a * d[1]) / det])
    }
}

/// **Ajusta o afim que leva `from` a `to`, e VERIFICA-O** — `None` quando ele não
/// reproduz, que é a resposta honesta para uma cadeia não-afim.
///
/// Mínimos quadrados clássico sobre os pontos centrados: `A = Σ(s−s̄)(q−q̄)ᵀ · [Σ(q−q̄)(q−q̄)ᵀ]⁻¹`.
///
/// ⚠️ **A barra da verificação é RELATIVA à extensão do conjunto**, e não absoluta: um
/// layout de metros e um de milímetros exigiriam números diferentes, e um ε fixo mediria
/// a unidade em vez do ajuste.
pub(crate) fn fit_downstream(from: &[[f32; 2]], to: &[[f32; 2]]) -> Option<Downstream> {
    const MIN_POINTS: usize = 3;
    /// O erro tolerado, como fracção da extensão do conjunto de destino.
    const EPS_REL: f32 = 1e-4;
    if from.len() != to.len() || from.len() < MIN_POINTS {
        return None;
    }
    let n = from.len() as f32;
    let mean = |v: &[[f32; 2]]| {
        let s = v
            .iter()
            .fold([0.0f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
        [s[0] / n, s[1] / n]
    };
    let (mq, ms) = (mean(from), mean(to));
    // Σ(s−s̄)(q−q̄)ᵀ  e  Σ(q−q̄)(q−q̄)ᵀ
    let mut sq = [[0.0f32; 2]; 2];
    let mut qq = [[0.0f32; 2]; 2];
    for (q, s) in from.iter().zip(to) {
        let dq = [q[0] - mq[0], q[1] - mq[1]];
        let ds = [s[0] - ms[0], s[1] - ms[1]];
        for r in 0..2 {
            for c in 0..2 {
                sq[r][c] += ds[r] * dq[c];
                qq[r][c] += dq[r] * dq[c];
            }
        }
    }
    let det = qq[0][0] * qq[1][1] - qq[0][1] * qq[1][0];
    if !det.is_finite() || det.abs() < 1e-12 {
        // Pontos colineares (ou um só) — não determinam um afim.
        return None;
    }
    let inv = [
        [qq[1][1] / det, -qq[0][1] / det],
        [-qq[1][0] / det, qq[0][0] / det],
    ];
    let mut lin = [[0.0f32; 2]; 2];
    for r in 0..2 {
        for c in 0..2 {
            lin[r][c] = sq[r][0] * inv[0][c] + sq[r][1] * inv[1][c];
        }
    }
    let d = Downstream {
        lin,
        tr: [
            ms[0] - (lin[0][0] * mq[0] + lin[0][1] * mq[1]),
            ms[1] - (lin[1][0] * mq[0] + lin[1][1] * mq[1]),
        ],
    };
    // ⚠️ **A VERIFICAÇÃO — é ela que transforma o ajuste numa prova.** Sem esta metade,
    // uma cadeia não-afim receberia o afim "menos errado" e o gizmo cairia num sítio
    // plausível e falso.
    let extent = {
        let mut lo = [f32::MAX; 2];
        let mut hi = [f32::MIN; 2];
        for s in to {
            for k in 0..2 {
                lo[k] = lo[k].min(s[k]);
                hi[k] = hi[k].max(s[k]);
            }
        }
        (hi[0] - lo[0]).max(hi[1] - lo[1]).max(1e-6)
    };
    let worst = from
        .iter()
        .zip(to)
        .map(|(q, s)| {
            let p = d.apply(*q);
            (p[0] - s[0]).hypot(p[1] - s[1])
        })
        .fold(0.0f32, f32::max);
    (worst <= EPS_REL * extent).then_some(d)
}

/// **O SINK a que este nó chega** — andando pela porta 0 para jusante.
///
/// ⚠️ Pela porta 0 e não por qualquer aresta: é o caminho do STREAM, e é o que decide
/// onde as peças deste nó acabam desenhadas. Um limite de passos guarda contra um ciclo
/// que a validação do grafo não devesse deixar existir — mas um gizmo não é o sítio de
/// descobrir isso a travar.
pub(crate) fn sink_of(graph: &Graph, node: NodeId) -> Option<NodeId> {
    const MAX_STEPS: usize = 64;
    let mut cur = node;
    for _ in 0..MAX_STEPS {
        if graph
            .node(cur)
            .is_some_and(|n| n.type_name == "motion.output")
        {
            return Some(cur);
        }
        cur = graph.edges().iter().find(|e| e.from.0 == cur)?.to.0;
    }
    None
}

/// **A JANELA em que este gizmo vive** — a da CENA, nunca a janela cheia.
///
/// ⚠️ **É a porta ÚNICA, e ela existe porque eu errei exactamente isto** (Enio,
/// 2026-08-23: *"grade fora do lugar. drift. Não consegui manipular pontos e alças no
/// canvas"*). Sob o split a cena renderiza num sub-retângulo, e o `field_gizmo` já tinha
/// a lei escrita: *"a vector shape projected with the FULL window drifts off them —
/// shifted and shrunk"*, com o precedente nomeado (os caminhantes de um `motion.path`
/// sobre uma cópia deslocada da curva). Eu li aquele comentário e mesmo assim passei
/// `surface.size()` ao overlay.
///
/// ⚠️ **E o erro deu DOIS sintomas de uma causa só**: o desenho saiu deslocado *e* o
/// arrasto deixou de pegar — porque o hit-test usava a janela certa e a tinta a errada,
/// então a alça que se via não era a alça que existia. *Quando duas superfícies projectam
/// o mesmo mundo, elas têm de dividir a MESMA porta* — e é por isso que esta função
/// existe em vez de o chamador escolher.
pub(crate) fn scene_window(
    center_split: ph2d_editor::screens::layout::CenterSplit,
    full: ph2d_host::WindowSize,
) -> ph2d_host::WindowSize {
    crate::field_gizmo::scene_camera_window(center_split, full)
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
    /// O que a cadeia DEPOIS deste nó faz às peças — ver [`Downstream`]. Identidade
    /// quando o nó já está no fim, ou quando a medição não cobre a cadeia.
    pub(crate) down: Downstream,
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
    // ⚠️ A cadeia de jusante, MEDIDA por correspondência de elemento. Quando ela não é
    // afim (ou o nó já é o último), a resposta é a identidade — o frame do próprio nó,
    // que é o comportamento anterior.
    let down = sink_of(&motion.doc.graph, node)
        .filter(|s| *s != node)
        .and_then(|sink| {
            let q = tapped_points(motion, node)?;
            let sv = tapped_points(motion, sink)?;
            fit_downstream(&q, &sv)
        })
        .unwrap_or(Downstream::IDENTITY);
    Some(WarpGizmoView {
        node,
        spec,
        bbox,
        warp,
        down,
    })
}

/// **As alças no espaço do que se VÊ** — a porta única do overlay e do ponteiro.
///
/// ⚠️ Ela existe pela mesma razão que a `scene_window`: duas superfícies que projectam o
/// mesmo mundo têm de dividir a porta. Aqui a projecção extra é a cadeia de jusante.
pub(crate) fn view_handles(
    v: &WarpGizmoView,
    param: &dyn Fn(&str) -> f32,
) -> ([WarpHandle; MAX_HANDLES], usize) {
    let (mut hs, n) = handles(v.spec, v.bbox, v.warp, param);
    for h in &mut hs[..n] {
        h.world = v.down.apply(h.world);
    }
    (hs, n)
}

/// O contorno no espaço do que se vê, e os quatro cantos já empurrados (o overlay liga os
/// braços a eles).
pub(crate) fn view_outline(
    v: &WarpGizmoView,
    param: &dyn Fn(&str) -> f32,
) -> (Vec<[f32; 2]>, [[f32; 2]; 4]) {
    let b = boundary(v.spec, v.bbox, v.warp, param);
    let ring = outline(&b).into_iter().map(|p| v.down.apply(p)).collect();
    let mut corners = [[0.0f32; 2]; 4];
    for (i, c) in b.corner.iter().enumerate() {
        corners[i] = v.down.apply(*c);
    }
    (ring, corners)
}
