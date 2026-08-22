//! **O DIAGRAMA da booleana viva** — a metade PURA: onde cada coisa fica, e o que está sob o dedo.
//!
//! O card em si é [`crate::screens::hero::chrome::bool_graph_modal`]; aqui mora a geometria, sem
//! uma linha de pintura e sem tocar no `WidgetStore`. É o corte que torna a disposição e o
//! acerto do clique testáveis por si — e, mais importante, **é o que garante que quem PINTA e
//! quem ACERTA leem o mesmo mapa**: um segundo cálculo de posição divergiria do primeiro no dia
//! em que alguém mudasse um espaçamento, e o artista clicaria ao lado do que vê.
//!
//! # A disposição: uma COLUNA por z, e as ligações em arco à direita
//!
//! ⚠️ **Não é um anel, e a razão é a lei.** Quando várias ligações chegam ao mesmo nó, elas
//! dobram na ordem de **z** de quem opera (`ph2d_vec_boolean::graph`) — então o diagrama tem de
//! mostrar z, senão o resultado depende de uma coisa invisível. Um anel espalha os círculos
//! bonito e apaga exatamente o dado de que a lei depende.
//!
//! A coluna também não inventa convenção nenhuma: é a mesma leitura da lista de camadas que o
//! artista já tem, **o mais ao FUNDO em baixo**.
//!
//! As ligações saem pela direita em arco, com a barriga proporcional à distância entre as duas
//! linhas — assim elas se aninham como colchetes em vez de se sobreporem, e uma ligação que salta
//! três formas é visivelmente mais larga que uma entre vizinhas.

use crate::zones::Rect;

// ── Literais de geometria do card (LITERAL-PX-OK: desenho, não medição). ──
/// Raio do círculo de uma forma.
const NODE_R: f32 = 14.0; // LITERAL-PX-OK: bool-graph node circle radius
/// Distância vertical entre os centros de duas linhas.
const ROW_STEP: f32 = 44.0; // LITERAL-PX-OK: bool-graph row pitch
/// Margem interna do card até o primeiro centro.
const PAD: f32 = 20.0; // LITERAL-PX-OK: bool-graph card padding
/// Largura da coluna de rótulos, à direita do círculo.
const LABEL_W: f32 = 150.0; // LITERAL-PX-OK: bool-graph label column width
/// Quanto a barriga do arco cresce por linha saltada.
const BOW_PER_ROW: f32 = 18.0; // LITERAL-PX-OK: bool-graph link bow per skipped row
/// A barriga mínima — uma ligação entre vizinhas ainda precisa de sair da coluna.
const BOW_MIN: f32 = 26.0; // LITERAL-PX-OK: bool-graph minimum link bow
/// Folga do clique em volta do traço de uma ligação.
const LINK_GRAB: f32 = 9.0; // LITERAL-PX-OK: bool-graph link pick tolerance
/// Altura da banda de título (arrasto + fechar).
pub const TITLE_H: f32 = 28.0; // LITERAL-PX-OK: bool-graph modal title band
/// Altura da faixa de aviso, quando há uma.
pub const WARN_H: f32 = 24.0; // LITERAL-PX-OK: bool-graph warning strip

/// Quantas amostras do arco o acerto do clique mede. ⚠️ É CONTAGEM, não pixel: subir compra
/// precisão no meio do arco e não muda a geometria desenhada.
const LINK_SAMPLES: usize = 24;

/// **Uma forma no diagrama.**
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoolGraphNode {
    /// O `VecPathId` cru — a identidade que atravessa undo e save.
    pub id: u64,
    /// O nome que o artista lê na Hierarquia.
    pub label: String,
    /// Ele foi CONSUMIDO (tem ligação de saída) e portanto desenha nada no canvas.
    pub consumed: bool,
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
/// ⚠️ `nodes` vem em ordem de **z (fundo → topo)**, a MESMA ordem que o resolvedor consome. A
/// coluna inverte para desenhar (o fundo em baixo); inverter aqui faria a lei e o desenho
/// discordarem sobre o que "o primeiro" significa.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BoolGraphView {
    /// As formas, em ordem de z (fundo → topo).
    pub nodes: Vec<BoolGraphNode>,
    /// As ligações vivas.
    pub links: Vec<BoolGraphLink>,
    /// O grafo tem um ciclo? ⚠️ A shell publica isto porque a RECUSA é dela: aqui não se resolve
    /// nada, e um diagrama que decidisse sozinho se há ciclo poderia discordar de quem desenha a
    /// arte.
    pub cycle: bool,
}

/// **O que o artista pediu ao diagrama** — drenado pela shell, que é quem escreve o documento.
///
/// ⚠️ O diagrama não muta nada: ele publica intenções. É a mesma lei do painel de vetor (*"a
/// verdade mora no ECS, não aqui"*) e é o que impede duas respostas para *"quais são as ligações
/// deste grupo?"*.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoolGraphIntent {
    /// Ligar `from → to` com esta operação (substitui, se a ligação já existia).
    Link { from: u64, to: u64, op: u8 },
    /// Cortar a ligação `from → to`.
    Unlink { from: u64, to: u64 },
}

impl BoolGraphView {
    /// Quantas linhas a coluna tem.
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

/// **O tamanho do card** para uma vista — largura × altura, em px de tela.
///
/// A largura reserva a coluna de rótulos MAIS a maior barriga que alguma ligação vai precisar:
/// um arco que saísse do card seria uma ligação que existe e não se pode clicar.
#[must_use]
pub fn card_size(view: &BoolGraphView) -> (f32, f32) {
    let rows = view.rows().max(1);
    let widest = view
        .links
        .iter()
        .filter_map(|l| {
            let a = view.index_of(l.from)?;
            let b = view.index_of(l.to)?;
            Some(bow(a, b))
        })
        .fold(0.0_f32, f32::max);
    let w = PAD + NODE_R * 2.0 + LABEL_W + widest + PAD;
    #[allow(clippy::cast_precision_loss)] // contagem de linhas, não medida
    let h = TITLE_H
        + PAD
        + ROW_STEP * (rows as f32 - 1.0)
        + NODE_R * 2.0
        + PAD
        + if view.cycle { WARN_H } else { 0.0 };
    (w, h)
}

/// **Quanto o topo do arco passa DA DIREITA DOS RÓTULOS**, para uma ligação entre as linhas `a` e
/// `b` (índices em ordem de z).
///
/// ⚠️ É a medida do ponto mais largo que a curva de facto alcança — **não** o deslocamento do
/// ponto de controlo. A distinção não é acadêmica: numa quadrática o topo fica a **meio caminho**
/// do controlo, então usar o controlo como se fosse o alcance reserva o dobro do necessário e
/// deixa o card largo demais para uma folga que o desenho nunca usa. Foi um mutante sobrevivente
/// que a expôs — o gate do card passava com a reserva REMOVIDA, porque ela nunca era o que
/// apertava.
fn bow(a: usize, b: usize) -> f32 {
    #[allow(clippy::cast_precision_loss)] // distância em LINHAS, não em px
    let d = a.abs_diff(b) as f32;
    BOW_MIN + BOW_PER_ROW * (d - 1.0).max(0.0)
}

/// A borda direita da coluna de rótulos — onde os arcos começam a ter espaço só deles.
fn label_right(card: Rect) -> f32 {
    card.x + PAD + NODE_R * 2.0 + LABEL_W
}

/// **O centro do círculo** da forma de índice `i` (ordem de z), dado o canto do card.
///
/// ⚠️ A inversão está AQUI e em mais lugar nenhum: `i = 0` é o mais ao FUNDO e desenha-se em
/// BAIXO. Repetir esta conta noutro sítio é como o desenho e o clique passam a discordar.
#[must_use]
pub fn node_center(card: Rect, view: &BoolGraphView, i: usize) -> (f32, f32) {
    let rows = view.rows().max(1);
    let row = rows.saturating_sub(1).saturating_sub(i);
    #[allow(clippy::cast_precision_loss)] // índice de linha, não medida
    let y = card.y + TITLE_H + PAD + NODE_R + ROW_STEP * row as f32;
    (card.x + PAD + NODE_R, y)
}

/// O raio do círculo de uma forma — publicado para o painter e o gate lerem o mesmo número.
#[must_use]
pub const fn node_radius() -> f32 {
    NODE_R
}

/// **Qual forma está sob o ponto?** Devolve o índice em ordem de z.
///
/// ⚠️ O alvo é o CÍRCULO, não a linha inteira: o rótulo à direita fica livre para o arrasto de
/// uma ligação passar por cima sem ser engolido.
#[must_use]
pub fn node_at(card: Rect, view: &BoolGraphView, p: (f32, f32)) -> Option<usize> {
    (0..view.rows()).find(|&i| {
        let c = node_center(card, view, i);
        let (dx, dy) = (p.0 - c.0, p.1 - c.1);
        dx.mul_add(dx, dy * dy) <= NODE_R * NODE_R
    })
}

/// **Os pontos do arco** de uma ligação, do centro de quem OPERA ao de quem RECEBE.
///
/// Uma quadrática amostrada: o painter desenha por estes pontos e o acerto do clique mede a
/// distância a eles. **Uma fonte, dois consumidores** — é isso que impede o clique de cair ao lado
/// da linha que se vê.
#[must_use]
pub fn link_points(card: Rect, view: &BoolGraphView, link: BoolGraphLink) -> Vec<(f32, f32)> {
    let (Some(a), Some(b)) = (view.index_of(link.from), view.index_of(link.to)) else {
        return Vec::new();
    };
    let p0 = node_center(card, view, a);
    let p2 = node_center(card, view, b);
    // ⚠️ O arco passa à direita dos RÓTULOS, não da coluna. Curvar a partir da coluna faria cada
    // ligação atravessar o nome das formas que ela salta — e o nome é como o artista sabe qual
    // círculo é qual.
    //
    // O topo de uma quadrática fica a MEIO CAMINHO do controlo, então o controlo é posto ao dobro
    // da distância: assim `bow` é o alcance REAL, e o card pode reservar exatamente ele.
    let apex_x = label_right(card) + bow(a, b);
    let ctrl = (
        2.0f32.mul_add(apex_x, -p0.0.midpoint(p2.0)),
        p0.1.midpoint(p2.1),
    );
    (0..=LINK_SAMPLES)
        .map(|k| {
            #[allow(clippy::cast_precision_loss)] // índice de amostra, não medida
            let t = k as f32 / LINK_SAMPLES as f32;
            let u = 1.0 - t;
            (
                u.mul_add(u * p0.0, 2.0 * u * t * ctrl.0) + t * t * p2.0,
                u.mul_add(u * p0.1, 2.0 * u * t * ctrl.1) + t * t * p2.1,
            )
        })
        .collect()
}

/// **Qual ligação está sob o ponto?** Devolve o índice em `view.links`.
///
/// Quando dois arcos se cruzam, ganha o MAIS PRÓXIMO — nunca o primeiro da lista, que é ordem de
/// armazenamento e não diz nada ao artista sobre o que ele está a apontar.
#[must_use]
pub fn link_at(card: Rect, view: &BoolGraphView, p: (f32, f32)) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for (i, l) in view.links.iter().enumerate() {
        let mut d2 = f32::INFINITY;
        for q in link_points(card, view, *l) {
            let (dx, dy) = (p.0 - q.0, p.1 - q.1);
            d2 = d2.min(dx.mul_add(dx, dy * dy));
        }
        if d2 <= LINK_GRAB * LINK_GRAB && best.is_none_or(|(_, b)| d2 < b) {
            best = Some((i, d2));
        }
    }
    best.map(|(i, _)| i)
}

/// **O que soltar o arrasto em `to` significa**, tendo começado em `from`.
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
