//! **O AUTO LAYOUT** — uma moldura empilha os filhos, e as posições passam a ser DERIVADAS.
//!
//! O *Auto Layout* do Figma, o *flexbox* do Rive §7. Esta crate é a **única porta do `taffy`** na
//! árvore: ela recebe uma árvore descrita em números, devolve retângulos, e não conhece ECS, nem
//! documento vetorial, nem tema. A contenção é a mesma que confinou o `realfft` na
//! `ph2d-audio-spectral` e o `tract` na `ph2d-audio-ml` — e corta para os dois lados: nada pesado
//! entra, e nenhuma UI entra.
//!
//! # A lei que decide tudo
//!
//! > **O resultado do layout é uma POSE DERIVADA. Ele nunca é o `Transform` autorado.**
//!
//! ⚠️ Não é preferência de estilo: o undo deste editor é **por DIFF do mundo ECS**
//! (`shells/desktop/src/undo.rs`). Um passe que escrevesse `Transform` faria **cada
//! redimensionamento virar um passo de undo**, e faria o layout brigar com o arrasto do artista
//! dentro do mesmo frame. É a disciplina do ADR-0111 (a pose é publicada) e do `LiveGeometry` (a
//! geometria é derivada) aplicada a um terceiro facto.
//!
//! # A convenção é a do CSS, e a conversão é do chamador
//!
//! ⚠️ Aqui `y` cresce para **BAIXO** e a origem é o canto superior-esquerdo do nó raiz — a
//! convenção do flexbox, que é o que o motor calcula. O documento vetorial é **Y-up**. A conversão
//! é UMA, no chamador (a shell), e está escrita lá; fazê-la aqui esconderia metade de uma troca de
//! eixo dentro de um motor que não conhece o eixo do documento.
//!
//! # O que NÃO está aqui, e por quê
//!
//! ⚠️ **Grid não entra.** Medido: `flexbox` sozinho custa **0,20 s** de build cold e com `grid`
//! **0,63 s** (+ a crate `grid`), e nada a jusante honraria um `dir = grid` hoje — seria o `dir`
//! que o artista escolhe e que não muda um pixel. Ele nasce com a UI que o consumir.
//!
//! ⚠️ **Não há *measure function*.** Uma folha entra com o tamanho que ela JÁ tem (a bbox da
//! geometria cozida, que o texto também tem porque os glifos dele são contornos assados). O que
//! falta ao texto é REFLUIR a uma largura — outra wave (W2a), e ela traz a measure function junto.

use taffy::prelude::*;
use taffy::{Rect as TaffyRect, Size as TaffySize};

/// Direção do fluxo. ⚠️ São as três que o motor honra — ver o cabeçalho sobre o grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Dir {
    /// Em linha, sem quebra.
    #[default]
    Row,
    /// Em coluna, sem quebra.
    Column,
    /// Em linha, quebrando quando não cabe.
    RowWrap,
}

/// Alinhamento no eixo TRANSVERSAL (o *align-items* do CSS).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
    /// Estica o filho para preencher a travessa.
    Stretch,
}

/// Distribuição no eixo PRINCIPAL (o *justify-content* do CSS).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Justify {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
}

/// Como um nó DISPÕE os filhos dele. A presença disto é o que faz de um nó uma moldura com fluxo;
/// sem ele, os filhos ficam onde o artista os pôs.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct FrameStyle {
    pub dir: Dir,
    /// Vão entre filhos: `[principal, transversal]`.
    pub gap: [f64; 2],
    /// Recuo interno `[topo, direita, base, esquerda]` — a ordem do CSS.
    pub pad: [f64; 4],
    pub align: Align,
    pub justify: Justify,
}

/// Como um nó se comporta DENTRO do pai.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemStyle {
    /// Quanto ele toma da sobra (o *flex-grow*). `0` = fica no tamanho dele.
    pub grow: f32,
    /// Quanto ele cede quando falta (o *flex-shrink*).
    pub shrink: f32,
    /// Tamanho de partida no eixo principal; `None` = o tamanho do próprio nó.
    pub basis: Option<f64>,
}

impl Default for ItemStyle {
    /// O neutro é **INERTE**: não cresce, não encolhe, sem base própria — o item não participa da
    /// repartição, e o tamanho que ele traz é o tamanho que ele fica com.
    ///
    /// ⚠️ Diverge do CSS de propósito (lá `flex-shrink` nasce em `1`). O default do CSS é uma
    /// POLÍTICA — *encolher é gentil com texto* —, e aqui o conteúdo é o desenho do artista:
    /// espremer as formas dele sem ninguém pedir é o oposto de um neutro. Neste repo `Default`
    /// significa *não faz nada*, que é a mesma lei do ponto neutro de cada efeito da rack de áudio.
    fn default() -> Self {
        Self {
            grow: 0.0,
            shrink: 0.0,
            basis: None,
        }
    }
}

/// Um nó da árvore a resolver.
///
/// ⚠️ A fatia tem de vir em ordem de **PAI ANTES DOS FILHOS** (o DFS que a hierarquia do editor já
/// produz), e a ordem entre irmãos é a ordem em que eles aparecem na fatia — é ela que o fluxo
/// segue. [`solve`] **recusa** uma fatia mal ordenada em vez de produzir posições silenciosamente
/// erradas: um layout errado não falha, ele desenha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Node {
    /// Índice do pai na MESMA fatia. `None` = a raiz (tem de ser o índice 0).
    pub parent: Option<usize>,
    /// `Some` ⇒ este nó dispõe os filhos dele em fluxo.
    pub frame: Option<FrameStyle>,
    /// Como ele se comporta dentro do pai.
    pub item: ItemStyle,
    /// O tamanho autorado (uma moldura) ou intrínseco (uma folha), `[w, h]`.
    pub size: [f64; 2],
}

/// O retângulo resolvido de um nó, **relativo ao canto superior-esquerdo da raiz**, com `y` para
/// baixo: `[x, y, w, h]`.
pub type Solved = [f64; 4];

/// Por que uma árvore foi recusada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutError {
    /// A fatia está vazia.
    Empty,
    /// O índice 0 não é a raiz, ou há outra raiz depois dele.
    NotRooted,
    /// Um pai aparece DEPOIS do filho (ou aponta para fora da fatia).
    OutOfOrder,
    /// Um nó pendurado num pai que **não dispõe em fluxo**.
    ///
    /// ⚠️ Recusar em vez de acomodar: o default do motor é `Display::Flex`, então um pai sem fluxo
    /// disporia os filhos na mesma — silenciosamente, e com o artista a ver as formas dele saltarem
    /// para um canto. Quem monta a fatia recolhe só sub-árvores que fluem (é o que mantém
    /// byte-intocado todo documento em que ninguém pediu layout).
    ParentDoesNotFlow,
    /// O motor recusou a árvore (só alcançável com uma árvore que esta função não constrói).
    Engine,
}

/// **Resolve a árvore.** Devolve um retângulo por nó, na mesma ordem da entrada.
///
/// A raiz é resolvida no tamanho DELA (`nodes[0].size`) — é a moldura, e o tamanho dela é autorado
/// (o `w`/`h` do retângulo vivo que a carrega). O que o layout decide é o que está DENTRO.
///
/// ⚠️ **Todo nó da fatia (menos a raiz) tem de pendurar num pai que FLUI** — ver
/// [`LayoutError::ParentDoesNotFlow`]. Uma sub-árvore que não flui não entra aqui; ela fica com a
/// pose autorada, e é isso que mantém byte-intocado todo documento em que ninguém pediu layout.
pub fn solve(nodes: &[Node]) -> Result<Vec<Solved>, LayoutError> {
    if nodes.is_empty() {
        return Err(LayoutError::Empty);
    }
    if nodes[0].parent.is_some() {
        return Err(LayoutError::NotRooted);
    }
    for (i, n) in nodes.iter().enumerate().skip(1) {
        match n.parent {
            None => return Err(LayoutError::NotRooted),
            Some(p) if p >= i => return Err(LayoutError::OutOfOrder),
            Some(p) if nodes[p].frame.is_none() => return Err(LayoutError::ParentDoesNotFlow),
            Some(_) => {}
        }
    }
    if nodes[0].frame.is_none() && nodes.len() > 1 {
        return Err(LayoutError::ParentDoesNotFlow);
    }

    let mut tree: TaffyTree<()> = TaffyTree::with_capacity(nodes.len());
    let mut ids: Vec<NodeId> = Vec::with_capacity(nodes.len());
    for n in nodes {
        ids.push(
            tree.new_leaf(style_of(n))
                .map_err(|_| LayoutError::Engine)?,
        );
    }
    // Os filhos entram na ORDEM da fatia — é ela que o fluxo segue.
    for (i, n) in nodes.iter().enumerate() {
        if let Some(p) = n.parent {
            tree.add_child(ids[p], ids[i])
                .map_err(|_| LayoutError::Engine)?;
        }
    }
    let root_size = TaffySize {
        width: AvailableSpace::Definite(nodes[0].size[0] as f32),
        height: AvailableSpace::Definite(nodes[0].size[1] as f32),
    };
    tree.compute_layout(ids[0], root_size)
        .map_err(|_| LayoutError::Engine)?;

    // O motor dá a posição RELATIVA ao pai; quem chama quer relativa à RAIZ (é ela que ancora a
    // conversão para o mundo). Como a fatia é pai-antes-do-filho, uma passada basta.
    let mut abs: Vec<Solved> = Vec::with_capacity(nodes.len());
    for (i, n) in nodes.iter().enumerate() {
        let l = tree.layout(ids[i]).map_err(|_| LayoutError::Engine)?;
        let (px, py) = match n.parent {
            Some(p) => (abs[p][0], abs[p][1]),
            None => (0.0, 0.0),
        };
        abs.push([
            px + f64::from(l.location.x),
            py + f64::from(l.location.y),
            f64::from(l.size.width),
            f64::from(l.size.height),
        ]);
    }
    Ok(abs)
}

/// O estilo de um nó, traduzido para o vocabulário do motor. Porta ÚNICA: se a tradução vivesse em
/// dois sítios, o que a UI mostra e o que o motor faz divergiriam no primeiro modo novo.
fn style_of(n: &Node) -> Style {
    let mut s = Style {
        size: TaffySize {
            width: length(n.size[0] as f32),
            height: length(n.size[1] as f32),
        },
        flex_grow: n.item.grow,
        flex_shrink: n.item.shrink,
        ..Default::default()
    };
    if let Some(b) = n.item.basis {
        s.flex_basis = length(b as f32);
    }
    let Some(f) = n.frame else {
        // Sem fluxo: para o pai este nó é uma folha. Os filhos dele mantêm a pose autorada, e quem
        // chama não lê o retângulo deles.
        return s;
    };
    s.display = Display::Flex;
    let (dirn, wrap) = match f.dir {
        Dir::Row => (FlexDirection::Row, FlexWrap::NoWrap),
        Dir::Column => (FlexDirection::Column, FlexWrap::NoWrap),
        Dir::RowWrap => (FlexDirection::Row, FlexWrap::Wrap),
    };
    s.flex_direction = dirn;
    s.flex_wrap = wrap;
    s.gap = TaffySize {
        width: length(f.gap[0] as f32),
        height: length(f.gap[1] as f32),
    };
    s.padding = TaffyRect {
        top: length(f.pad[0] as f32),
        right: length(f.pad[1] as f32),
        bottom: length(f.pad[2] as f32),
        left: length(f.pad[3] as f32),
    };
    s.align_items = Some(match f.align {
        Align::Start => AlignItems::FLEX_START,
        Align::Center => AlignItems::CENTER,
        Align::End => AlignItems::FLEX_END,
        Align::Stretch => AlignItems::STRETCH,
    });
    s.justify_content = Some(match f.justify {
        Justify::Start => JustifyContent::FLEX_START,
        Justify::Center => JustifyContent::CENTER,
        Justify::End => JustifyContent::FLEX_END,
        Justify::SpaceBetween => JustifyContent::SPACE_BETWEEN,
        Justify::SpaceAround => JustifyContent::SPACE_AROUND,
    });
    s
}

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
