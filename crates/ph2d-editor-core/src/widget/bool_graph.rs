//! **O DIAGRAMA da booleana viva** — a metade PURA: onde cada coisa fica, e o que está sob o dedo.
//!
//! O card em si é [`crate::screens::hero::chrome::bool_graph_modal`]; aqui mora a geometria, sem
//! uma linha de pintura e sem tocar no `WidgetStore`. É o corte que torna a disposição e o
//! acerto do clique testáveis por si — e, mais importante, **é o que garante que quem PINTA e
//! quem ACERTA leem o mesmo mapa**: um segundo cálculo de posição divergiria do primeiro no dia
//! em que alguém mudasse um espaçamento, e o artista clicaria ao lado do que vê.
//!
//! # Os círculos são LIVRES no plano (Enio, 2026-08-22)
//!
//! A primeira versão punha-os numa coluna por z, para o diagrama mostrar a ordem de empilhamento
//! — que é o que decide a dobra quando várias ligações chegam ao mesmo nó. O Enio viu e pediu
//! *"liberdade para arrastar os círculos e criar conexões"*.
//!
//! ⚠️ **A lei que a coluna protegia não foi abandonada: ela mudou de veículo.** Cada círculo leva
//! o **número da sua ordem de z** ([`BoolGraphNode::z_badge`]), então o dado de que a lei depende
//! continua legível sem amarrar a disposição. Trocar a coluna pelo plano SEM o número teria
//! apagado a única pista de por que duas ligações que chegam ao mesmo sítio dobram nesta ordem e
//! não na outra.
//!
//! Uma forma sem posição guardada cai no **anel default** — é o que faz abrir a janela pela
//! primeira vez mostrar algo legível sem obrigar ninguém a arrastar nada.
//!
//! # O círculo tem duas zonas, e elas são gestos diferentes
//!
//! | zona | gesto |
//! |---|---|
//! | o **miolo** | clicar SELECIONA a forma no canvas · arrastar MOVE o círculo |
//! | o **aro** (a banda de fora) | arrastar de lá LIGA a outra forma |
//!
//! ⚠️ O clique no miolo existe por um defeito real: um operando consumido desenha VAZIO, e a lei
//! do canvas é *"nada desenhado, nada pego"* — ele fica inalcançável pelo ponteiro. O diagrama
//! passa a ser a porta que o alcança, que é o papel natural dele.

use crate::zones::Rect;

// ── Literais de geometria do card (LITERAL-PX-OK: desenho, não medição). ──
/// Raio do círculo de uma forma. Grande de propósito: o nome vai DENTRO dele.
const NODE_R: f32 = 34.0; // LITERAL-PX-OK: bool-graph node circle radius
/// Onde começa o **aro** — a banda de fora, que é a alça de ligar.
const RING_FRAC: f32 = 0.72; // LITERAL-PX-OK: bool-graph link-handle ring, fraction of the radius
/// Margem interna do card.
const PAD: f32 = 16.0; // LITERAL-PX-OK: bool-graph card padding
/// O menor tamanho do plano onde os círculos vivem.
const MIN_W: f32 = 520.0; // LITERAL-PX-OK: bool-graph canvas minimum width
/// Idem, altura.
const MIN_H: f32 = 380.0; // LITERAL-PX-OK: bool-graph canvas minimum height
/// Quanto uma ligação se desloca da reta, para que `A→B` e `B→A` não se sobreponham.
const LINK_OFFSET: f32 = 7.0; // LITERAL-PX-OK: bool-graph link lateral offset
/// Folga do clique em volta do traço de uma ligação.
const LINK_GRAB: f32 = 9.0; // LITERAL-PX-OK: bool-graph link pick tolerance
/// Altura da banda de título (arrasto + fechar).
pub const TITLE_H: f32 = 28.0; // LITERAL-PX-OK: bool-graph modal title band
/// Altura da faixa de rodapé (dica ou aviso).
pub const FOOTER_H: f32 = 26.0; // LITERAL-PX-OK: bool-graph footer strip

/// **Uma forma no diagrama.**
#[derive(Clone, Debug, PartialEq)]
pub struct BoolGraphNode {
    /// O `VecPathId` cru — a identidade que atravessa undo e save.
    pub id: u64,
    /// O nome que o artista lê na Hierarquia.
    pub label: String,
    /// Ele foi CONSUMIDO (tem ligação de saída) e portanto desenha nada no canvas.
    pub consumed: bool,
    /// Onde o artista pôs este círculo, em px **locais ao plano** do card. `None` = ainda não foi
    /// posto, e cai no anel default.
    pub at: Option<[f32; 2]>,
}

impl BoolGraphNode {
    /// **O número da ordem de z** que o círculo mostra (1 = o mais ao FUNDO).
    ///
    /// ⚠️ É o que sobrou da coluna, e é o essencial dela: quando várias ligações chegam ao mesmo
    /// nó, elas dobram na ordem de z de quem opera. Sem este número no plano livre, o resultado
    /// dependeria de uma coisa que o diagrama não mostra.
    #[must_use]
    pub fn z_badge(index: usize) -> usize {
        index + 1
    }
}

/// **Uma ligação dirigida**, como o diagrama a mostra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoolGraphLink {
    /// Quem OPERA.
    pub from: u64,
    /// Quem RECEBE.
    pub to: u64,
    /// O discriminante de `PathfinderOp` (só as quatro de conjunto são válidas).
    pub op: u8,
}

/// **O que o diagrama mostra neste frame** — publicado pela shell, nunca calculado aqui.
///
/// ⚠️ `nodes` vem em ordem de **z (fundo → topo)**, a MESMA ordem que o resolvedor consome. É dela
/// que sai o número no círculo.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoolGraphView {
    /// As formas, em ordem de z (fundo → topo).
    pub nodes: Vec<BoolGraphNode>,
    /// As ligações vivas.
    pub links: Vec<BoolGraphLink>,
    /// O grafo tem um ciclo? ⚠️ A shell publica isto porque a RECUSA é dela.
    pub cycle: bool,
}

/// **O que o artista pediu ao diagrama** — drenado pela shell, que é quem escreve o documento.
///
/// ⚠️ O diagrama não muta nada: ele publica intenções. É a mesma lei do painel de vetor (*"a
/// verdade mora no ECS, não aqui"*).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoolGraphIntent {
    /// Ligar `from → to` com esta operação (substitui, se a ligação já existia).
    Link { from: u64, to: u64, op: u8 },
    /// Cortar a ligação `from → to`.
    Unlink { from: u64, to: u64 },
    /// Pôr o círculo desta forma aqui (px locais ao plano).
    Move { id: u64, at: [f32; 2] },
    /// **Selecionar esta forma no canvas** — a porta que alcança um operando consumido, que o
    /// ponteiro não pega (ele desenha vazio, e *"nada desenhado, nada pego"*).
    Select { id: u64 },
}

/// **Um arrasto em curso no diagrama.**
///
/// ⚠️ O movimento de um círculo é **PRÉ-VISTO aqui e só escrito no documento ao SOLTAR**. Escrever
/// a cada frame do arrasto criaria um passo de undo por frame — o Ctrl+Z andaria pixel a pixel
/// para trás e o artista precisaria de cem deles para desfazer um gesto.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoolGraphDrag {
    /// De que forma o arrasto partiu.
    pub from: u64,
    /// `true` = está a puxar uma LIGAÇÃO (partiu do aro); `false` = está a MOVER o círculo.
    pub link: bool,
    /// A posição de pré-visualização, local ao plano (só vale quando `!link`).
    pub at: [f32; 2],
    /// Já saiu do ponto de partida? É o que separa um **clique** (seleciona a forma no canvas) de
    /// um **arrasto** (move o círculo).
    pub moved: bool,
}

/// **A vista com o arrasto já aplicado** — o que o painter desenha E o que o acerto do clique lê.
///
/// ⚠️ Uma porta só. Se o painter aplicasse a pré-visualização e o acerto lesse a vista crua, o
/// círculo apareceria debaixo do cursor e responderia no sítio antigo.
#[must_use]
pub fn with_drag(view: &BoolGraphView, drag: Option<BoolGraphDrag>) -> BoolGraphView {
    let Some(d) = drag.filter(|d| !d.link && d.moved) else {
        return view.clone();
    };
    let mut out = view.clone();
    if let Some(n) = out.nodes.iter_mut().find(|n| n.id == d.from) {
        n.at = Some(d.at);
    }
    out
}

impl BoolGraphView {
    /// Quantas formas o diagrama tem.
    #[must_use]
    pub fn rows(&self) -> usize {
        self.nodes.len()
    }

    /// O índice em `nodes` (ordem de z) da forma com este id.
    #[must_use]
    pub fn index_of(&self, id: u64) -> Option<usize> {
        self.nodes.iter().position(|n| n.id == id)
    }

    /// A operação da ligação `from → to`, se ela existe.
    #[must_use]
    pub fn op_of(&self, from: u64, to: u64) -> Option<u8> {
        self.links
            .iter()
            .find(|l| l.from == from && l.to == to)
            .map(|l| l.op)
    }
}

/// **O tamanho do card** — largura × altura, em px de tela.
///
/// O plano tem um mínimo generoso e **cresce** para caber o círculo mais distante que o artista
/// arrastou. ⚠️ Sem o crescimento, arrastar um círculo para longe o poria fora do card e ele
/// ficaria inalcançável — o gesto teria criado um estado de que não se pode voltar.
#[must_use]
pub fn card_size(view: &BoolGraphView) -> (f32, f32) {
    let (mut w, mut h) = (MIN_W, MIN_H);
    for n in &view.nodes {
        if let Some([x, y]) = n.at {
            w = w.max(x + NODE_R + PAD);
            h = h.max(y + NODE_R + PAD);
        }
    }
    (w, TITLE_H + h + FOOTER_H)
}

/// O retângulo do **plano** — o card menos a banda de título e o rodapé.
#[must_use]
pub fn canvas_rect(card: Rect) -> Rect {
    Rect::new(
        card.x,
        card.y + TITLE_H,
        card.w,
        (card.h - TITLE_H - FOOTER_H).max(1.0),
    )
}

/// **O centro do círculo** da forma de índice `i`, dado o canto do card.
///
/// Posição guardada, ou o **anel default** — as formas espalhadas em círculo, na ordem de z,
/// começando no topo. ⚠️ O anel é *default*, não *lei*: assim que o artista arrasta, a posição
/// dele manda, e o anel deixa de ser consultado para aquela forma.
#[must_use]
pub fn node_center(card: Rect, view: &BoolGraphView, i: usize) -> (f32, f32) {
    let plane = canvas_rect(card);
    if let Some(Some([x, y])) = view.nodes.get(i).map(|n| n.at) {
        return (plane.x + x, plane.y + y);
    }
    let n = view.rows().max(1);
    let (cx, cy) = (plane.x + plane.w * 0.5, plane.y + plane.h * 0.5);
    let r = (plane.w.min(plane.h) * 0.5 - NODE_R - PAD).max(NODE_R);
    #[allow(clippy::cast_precision_loss)] // índice de forma, não medida
    let a = std::f32::consts::TAU * i as f32 / n as f32 - std::f32::consts::FRAC_PI_2;
    (r.mul_add(a.cos(), cx), r.mul_add(a.sin(), cy))
}

/// O raio do círculo — publicado para o painter e o gate lerem o mesmo número.
#[must_use]
pub const fn node_radius() -> f32 {
    NODE_R
}

/// Onde começa o **aro** (a alça de ligar), em px do centro.
#[must_use]
pub fn ring_inner_radius() -> f32 {
    NODE_R * RING_FRAC
}

/// **Que zona do círculo está sob o ponto?**
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeZone {
    /// O miolo: clicar SELECIONA a forma, arrastar MOVE o círculo.
    Core,
    /// O aro: arrastar de lá LIGA a outra forma.
    Ring,
}

/// **Qual forma está sob o ponto, e em que zona?** Devolve o índice em ordem de z.
#[must_use]
pub fn node_at(card: Rect, view: &BoolGraphView, p: (f32, f32)) -> Option<(usize, NodeZone)> {
    (0..view.rows()).find_map(|i| {
        let c = node_center(card, view, i);
        let (dx, dy) = (p.0 - c.0, p.1 - c.1);
        let d2 = dx.mul_add(dx, dy * dy);
        if d2 > NODE_R * NODE_R {
            return None;
        }
        let inner = ring_inner_radius();
        Some((
            i,
            if d2 <= inner * inner {
                NodeZone::Core
            } else {
                NodeZone::Ring
            },
        ))
    })
}

/// **Os dois extremos do traço** de uma ligação — da borda de quem OPERA à de quem RECEBE.
///
/// ⚠️ O traço vai de BORDA a BORDA, não de centro a centro: uma linha que entra no círculo passaria
/// por cima do nome que está lá dentro, e o nome é como o artista sabe qual círculo é qual.
///
/// ⚠️ E ele desloca-se lateralmente por um valor FIXO, sempre para o mesmo lado da direção. É o
/// que faz `A→B` e `B→A` serem duas linhas paralelas em vez de uma só riscada duas vezes — sem
/// isso, um par que opera nos dois sentidos seria indistinguível de um que opera num só.
#[must_use]
pub fn link_points(card: Rect, view: &BoolGraphView, link: BoolGraphLink) -> Vec<(f32, f32)> {
    let (Some(a), Some(b)) = (view.index_of(link.from), view.index_of(link.to)) else {
        return Vec::new();
    };
    let p0 = node_center(card, view, a);
    let p1 = node_center(card, view, b);
    let (dx, dy) = (p1.0 - p0.0, p1.1 - p0.1);
    let len = dx.hypot(dy);
    if len <= f32::EPSILON {
        return Vec::new();
    }
    let (ux, uy) = (dx / len, dy / len);
    // A perpendicular, sempre à esquerda da direção de marcha.
    let (nx, ny) = (-uy * LINK_OFFSET, ux * LINK_OFFSET);
    let start = (NODE_R.mul_add(ux, p0.0) + nx, NODE_R.mul_add(uy, p0.1) + ny);
    let end = (
        NODE_R.mul_add(-ux, p1.0) + nx,
        NODE_R.mul_add(-uy, p1.1) + ny,
    );
    vec![start, end]
}

/// **Qual ligação está sob o ponto?** Devolve o índice em `view.links`.
///
/// Quando dois traços se cruzam, ganha o MAIS PRÓXIMO — nunca o primeiro da lista, que é ordem de
/// armazenamento e não diz nada ao artista sobre o que ele está a apontar.
#[must_use]
pub fn link_at(card: Rect, view: &BoolGraphView, p: (f32, f32)) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (i, l) in view.links.iter().enumerate() {
        let pts = link_points(card, view, *l);
        let [a, b] = pts[..] else { continue };
        let d = dist_to_segment(p, a, b);
        if d <= LINK_GRAB && best.is_none_or(|(_, bd)| d < bd) {
            best = Some((i, d));
        }
    }
    best.map(|(i, _)| i)
}

/// A distância de `p` ao segmento `a—b`.
fn dist_to_segment(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (vx, vy) = (b.0 - a.0, b.1 - a.1);
    let (wx, wy) = (p.0 - a.0, p.1 - a.1);
    let len2 = vx.mul_add(vx, vy * vy);
    let t = if len2 <= f32::EPSILON {
        0.0
    } else {
        (wx.mul_add(vx, wy * vy) / len2).clamp(0.0, 1.0) // CLAMP-OK: limites ordenados, len2 > 0
    };
    (wx - t * vx).hypot(wy - t * vy)
}

/// **Onde um círculo arrastado até `p` fica**, em px locais ao plano — já preso ao plano.
///
/// ⚠️ A prisão é o que impede o gesto de criar um estado irreversível: um círculo largado fora do
/// card ficaria fora do alcance do ponteiro para sempre.
#[must_use]
pub fn clamp_to_plane(card: Rect, p: (f32, f32)) -> [f32; 2] {
    let plane = canvas_rect(card);
    let hi_x = (plane.w - NODE_R).max(NODE_R);
    let hi_y = (plane.h - NODE_R).max(NODE_R);
    [
        (p.0 - plane.x).clamp(NODE_R, hi_x), // CLAMP-OK: limites ordenados (hi ≥ NODE_R), sem NaN
        (p.1 - plane.y).clamp(NODE_R, hi_y), // CLAMP-OK: idem
    ]
}

/// **O que soltar o arrasto de LIGAÇÃO em `to` significa**, tendo começado em `from`.
///
/// `None` = o gesto não é uma ligação (soltou fora, ou no mesmo círculo). ⚠️ O laço de um nó
/// consigo mesmo é recusado **aqui**, e não só no resolvedor: um gesto que produz uma recusa é um
/// gesto que não devia ter sido aceite.
#[must_use]
pub fn drop_intent(view: &BoolGraphView, from: u64, to: u64, op: u8) -> Option<BoolGraphIntent> {
    if from == to || view.index_of(from).is_none() || view.index_of(to).is_none() {
        return None;
    }
    Some(BoolGraphIntent::Link { from, to, op })
}

// Os gates vivem num irmão de pasta (a convenção do `command_palette`) para continuarem um módulo
// FILHO — só assim eles alcançam os itens privados que a disposição usa.
#[cfg(test)]
mod tests;
