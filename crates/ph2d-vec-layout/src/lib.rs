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
//! ⚠️ **Não há *track sizing* na grade.** As colunas são iguais (`repeat(N, 1fr)`) e as linhas
//! nascem do conteúdo, e isso não é modéstia: é o que o único consumidor MEDIDO deste repo faz à
//! mão (`paint_catalog.rs`, `cell_w = (inner_w − gap·(N−1)) / N`) e é o default do Figma. Uma
//! coluna de tamanho autorado é um vocabulário próprio (`fr` × `auto` × comprimento) e nasce com o
//! primeiro consumidor que o peça — hoje seria um controlo que ninguém move.
//!
//! ⚠️ **Não há *measure function*, e isso NÃO impede o `hug`.** Uma folha entra com o tamanho que
//! ela JÁ tem (a bbox da geometria cozida, que o texto também tem porque os glifos dele são
//! contornos assados). Uma *measure function* é precisa para uma **FOLHA** cujo tamanho depende do
//! espaço que lhe oferecem (texto que reflui); um **CONTENTOR** que se ajusta ao conteúdo é
//! *intrinsic sizing* puro, e o flexbox resolve-o sem perguntar nada a ninguém — ver [`Len::Hug`].
//!
//! ⚠️ A frase anterior deste cabeçalho concluía o contrário, e a medição derrubou-a
//! (`hug_probe.rs`, 2026-08-10): uma moldura `Hug` com três filhos de 10, vão 4 e recuo 2 mede
//! **42,000 × 10,000** exactos.

use taffy::prelude::*;
use taffy::{Rect as TaffyRect, Size as TaffySize};

/// Direção do fluxo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Dir {
    /// Em linha, sem quebra.
    #[default]
    Row,
    /// Em coluna, sem quebra.
    Column,
    /// Em linha, quebrando quando não cabe.
    RowWrap,
    /// **Grade de `columns` colunas IGUAIS**; as linhas nascem sozinhas, cada uma com a altura do
    /// seu conteúdo mais alto.
    ///
    /// ⚠️ **O que uma grade dá que o `RowWrap` não dá são duas coisas, e só estas duas:** colunas
    /// **alinhadas entre linhas** quando os conteúdos têm larguras diferentes (no wrap cada faixa
    /// se arruma sozinha, então nada fica em coluna), e **refluxo automático** ao inserir ou
    /// remover um item. Se nenhuma das duas é pedida, o wrap já responde — e é mais barato.
    ///
    /// ⚠️ **A contagem vem no variante** porque, do lado do motor, uma direção sem contagem
    /// deixaria o `style_of` a perguntar por um campo que a `FrameStyle` não tem. Do lado do
    /// DOCUMENTO ela é um campo (`VecLayout::columns`), e a assimetria é a mesma — e pelo mesmo
    /// motivo — que a do [`Len::Fixed`]: lá o número sobrevive a uma troca de direção, aqui não há
    /// troca nenhuma, há uma chamada.
    Grid { columns: u16 },
}

/// **Quantas faixas o motor sabe indexar num eixo da grade** — e este limite NÃO é um palpite: é a
/// largura do inteiro com que o `taffy` numera as linhas da grade (`OriginZeroLine(i16)`), medida
/// por bisseção contra o motor em três fixtures independentes (`grid_probe.rs`).
///
/// | fixture | última contagem que resolve |
/// |---|---|
/// | 1 coluna, N filhos | 32767 filhos = **32767 linhas** |
/// | 2 colunas, N filhos | 65534 filhos = **32767 linhas** |
/// | 3 colunas, N filhos | 98301 filhos = **32767 linhas** |
/// | N colunas, N filhos | **32767** |
///
/// ⚠️ **O teto morde pelas LINHAS, que ninguém autora** — elas nascem da contagem de filhos. Um cap
/// só na contagem de COLUNAS teria sido taste E não teria evitado o pânico; por isso a regra é
/// verificada nos dois eixos, no [`solve`], ao lado das outras recusas.
///
/// ⚠️ E ele está longe de qualquer uso: com 12 filhos, **16384 colunas custam 0,33 ms** e 65535
/// custam 1,43 — o que aperta primeiro é o desenho, não o número.
///
/// ⚠️ **No eixo das COLUNAS a guarda é mais estrita do que a falha medida, de propósito.** O motor
/// só materializa as faixas que os filhos de facto OCUPAM, então 65535 colunas com 12 filhos
/// resolvem (1,43 ms) — mas isso é um detalhe interno do `taffy`, e o modo de falha do outro lado
/// dele é um **pânico**, não um erro. Um número derivado da largura do inteiro é seguro por
/// construção e sobrevive a um upgrade da dep; um que codifique *"só as ocupadas contam"* deixa de
/// ser verdade em silêncio no dia em que ele mudar.
pub const MAX_GRID_TRACKS: u16 = i16::MAX as u16;

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

/// **O tamanho de um nó num eixo.**
///
/// ⚠️ É um enum e não um `f64` com um `bool` ao lado porque as duas respostas são **exclusivas**:
/// um eixo que abraça não tem um número que alguém escreveu, e guardá-lo deixaria no tipo um campo
/// que só significa alguma coisa metade das vezes. A representação apaga o caso especial.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Len {
    /// O número que o nó TRAZ — a bbox da geometria de uma folha, ou o tamanho autorado de uma
    /// moldura.
    Fixed(f64),
    /// **Abraça o conteúdo** (o *Hug contents* do Figma): o tamanho sai do que está DENTRO.
    ///
    /// ⚠️ Só faz sentido num nó que FLUI — uma folha não tem conteúdo a abraçar, ela **é** o
    /// conteúdo, e pedi-lo ali resolveria para **zero** (a forma sumiria da tela, em silêncio).
    /// Por isso [`solve`] o RECUSA com [`LayoutError::HugWithoutFlow`] em vez de o acomodar.
    ///
    /// ⚠️ Num `RowWrap` o abraço **desliga a quebra na prática**, e é o correcto: quebrar é
    /// responder *"não cabe na largura que me deram"*, e quem abraça não recebeu largura nenhuma.
    /// É o que o Figma faz.
    Hug,
}

impl Default for Len {
    /// Zero fixo — o neutro de um nó que ainda não foi descrito, nunca um pedido de abraço.
    fn default() -> Self {
        Self::Fixed(0.0)
    }
}

impl Len {
    /// O número que este eixo traz, se trouxer algum.
    #[must_use]
    pub const fn fixed(self) -> Option<f64> {
        match self {
            Len::Fixed(v) => Some(v),
            Len::Hug => None,
        }
    }
}

impl Default for Node {
    /// Uma folha de tamanho zero pendurada na raiz. Existe para o `..Default::default()` de quem
    /// constrói um nó de que só dois campos interessam — nunca como um nó útil por si.
    fn default() -> Self {
        Self {
            parent: None,
            frame: None,
            item: ItemStyle::default(),
            size: [Len::default(); 2],
            min: [None; 2],
            max: [None; 2],
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
    /// O tamanho autorado (uma moldura) ou intrínseco (uma folha), `[w, h]` — ou o pedido de
    /// abraçar o conteúdo. Ver [`Len`].
    pub size: [Len; 2],
    /// **Piso** por eixo, `[w, h]`. `None` = sem piso.
    ///
    /// ⚠️ Vale com qualquer [`Len`], e o par que ele existe para servir é `Hug` + `min`: *"cresce
    /// com o rótulo, mas nunca fica mais estreito do que isto"* — o botão que não colapsa quando o
    /// texto é uma letra só.
    pub min: [Option<f64>; 2],
    /// **Teto** por eixo. `None` = sem teto.
    pub max: [Option<f64>; 2],
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
    /// Uma **grade** pediu mais faixas do que o motor sabe indexar — ver [`MAX_GRID_TRACKS`].
    ///
    /// ⚠️ Recusar em vez de acomodar, e aqui o preço de acomodar não é uma pose errada: é o
    /// **motor a PANICAR** (`taffy/compute/grid/types/cell_occupancy.rs`, um `unwrap` sobre a
    /// célula fora da grade), o que derruba o app. A recusa deixa a sub-árvore com a pose autorada,
    /// que é o que toda outra recusa desta lista já faz.
    GridTooManyTracks,
    /// Um nó que **não flui** pediu [`Len::Hug`].
    ///
    /// ⚠️ Recusar em vez de acomodar, e aqui o preço de acomodar é a arte a **desaparecer**: sem
    /// filhos, o conteúdo a abraçar mede zero, e o motor devolveria um retângulo de tamanho nulo
    /// sem erro nenhum. Quem monta a fatia só oferece o abraço a uma moldura.
    HugWithoutFlow,
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
    // O abraço é uma propriedade de CONTENTOR — ver `LayoutError::HugWithoutFlow`.
    for n in nodes {
        if n.frame.is_none() && n.size.contains(&Len::Hug) {
            return Err(LayoutError::HugWithoutFlow);
        }
    }
    // ⚠️ **O teto da grade, e ele é medido — ver [`MAX_GRID_TRACKS`].** As COLUNAS o artista
    // escreve; as LINHAS nascem da contagem de filhos, e é por elas que o motor estoura primeiro.
    // A pergunta é feita aqui, ao lado das outras recusas, e não no `style_of` — ali um nó não
    // conhece os filhos dele.
    for (i, n) in nodes.iter().enumerate() {
        let Some(FrameStyle {
            dir: Dir::Grid { columns },
            ..
        }) = n.frame
        else {
            continue;
        };
        let cols = usize::from(columns.max(1));
        if cols > usize::from(MAX_GRID_TRACKS) {
            return Err(LayoutError::GridTooManyTracks);
        }
        let kids = nodes.iter().filter(|k| k.parent == Some(i)).count();
        if kids.div_ceil(cols) > usize::from(MAX_GRID_TRACKS) {
            return Err(LayoutError::GridTooManyTracks);
        }
    }

    let mut tree: TaffyTree<()> = TaffyTree::with_capacity(nodes.len());
    // ⚠️ **O ARREDONDAMENTO SAI, e não é afinação — é uma diferença de UNIDADE.**
    //
    // O `taffy` arredonda o resultado para inteiros por default, e para um navegador isso está
    // certo: a unidade dele é o PIXEL de dispositivo, e meio pixel não existe. A nossa é a unidade
    // de MUNDO do documento, onde uma moldura inteira mede 9 e um filho mede 1,1 — arredondar ali
    // é apagar a geometria.
    //
    // Medido, com a cena de smoke `=50` (moldura 9,0 · cinco botões de 1,1 · um espaçador de 0,7
    // com `grow`): os botões saíam a **1,0** e o espaçador a **4,0** em vez de 1,1 e 3,5. A soma
    // continuava a fechar em 9,0, que é precisamente o que torna o defeito difícil de ver.
    //
    // ⚠️ E ele atravessou a wave inteira porque **todas as fixtures usavam números inteiros**
    // (molduras de 100×40, filhos de 10, vãos de 4 e 2): sob elas as duas versões são idênticas.
    // O gate `a_fractional_layout_is_not_rounded_to_whole_units` é o que contém o fenómeno.
    tree.disable_rounding();
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
    // ⚠️ **O espaço oferecido à raiz É a pergunta do abraço.** `Definite` diz *"tens esta
    // largura"*; `MaxContent` diz *"decide-a tu"*, e é o que faz o `Dimension::AUTO` do estilo
    // resolver pelo conteúdo em vez de por zero. Os dois eixos são independentes: uma barra que
    // ocupa a largura toda e tem a altura do que está dentro é `Definite` num e `MaxContent` no
    // outro.
    let avail = |l: Len| match l {
        Len::Fixed(v) => AvailableSpace::Definite(v as f32),
        Len::Hug => AvailableSpace::MaxContent,
    };
    let root_size = TaffySize {
        width: avail(nodes[0].size[0]),
        height: avail(nodes[0].size[1]),
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
    // `Hug` é literalmente o `auto` do CSS: *"o teu tamanho sai do teu conteúdo"*. Não há nada a
    // calcular aqui — a tradução é o trabalho todo.
    let dim = |l: Len| match l {
        Len::Fixed(v) => length(v as f32),
        Len::Hug => Dimension::AUTO,
    };
    let bound = |v: Option<f64>| v.map_or_else(auto, |x| length(x as f32));
    let mut s = Style {
        size: TaffySize {
            width: dim(n.size[0]),
            height: dim(n.size[1]),
        },
        min_size: TaffySize {
            width: bound(n.min[0]),
            height: bound(n.min[1]),
        },
        max_size: TaffySize {
            width: bound(n.max[0]),
            height: bound(n.max[1]),
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
    // ⚠️ **O vão é MAIN e CROSS; quem os traduz para x/y é a DIREÇÃO.** O `taffy` fala em eixos
    // FÍSICOS (`width` = entre colunas, `height` = entre linhas), e escrever `[main, cross]` neles
    // direto está certo em `Row`/`RowWrap` e INVERTIDO em `Column` — onde o principal é vertical.
    //
    // Foi o report do Enio (2026-08-02), e ele chegou como DOIS: *"Gap não funciona em Column"* e
    // *"Cross, que só aparece para Wrap, afeta Column"*. É um só, visto pelas duas pontas — o
    // número do artista ia para o eixo errado, e o do eixo errado governava o dele.
    //
    // ⚠️ Na GRADE o principal é horizontal (entre colunas) e o transversal vertical (entre linhas),
    // exactamente como no `Row` — é a mesma frase, e por isso é o mesmo braço.
    let (main, cross) = (length(f.gap[0] as f32), length(f.gap[1] as f32));
    s.gap = match f.dir {
        Dir::Column => TaffySize {
            width: cross,
            height: main,
        },
        Dir::Row | Dir::RowWrap | Dir::Grid { .. } => TaffySize {
            width: main,
            height: cross,
        },
    };
    s.padding = TaffyRect {
        top: length(f.pad[0] as f32),
        right: length(f.pad[1] as f32),
        bottom: length(f.pad[2] as f32),
        left: length(f.pad[3] as f32),
    };
    // O alinhamento transversal diz a MESMA coisa nos dois motores — *onde o filho senta na
    // travessa* —, então ele é resolvido uma vez, antes do ramo.
    s.align_items = Some(match f.align {
        Align::Start => AlignItems::FLEX_START,
        Align::Center => AlignItems::CENTER,
        Align::End => AlignItems::FLEX_END,
        Align::Stretch => AlignItems::STRETCH,
    });
    if let Dir::Grid { columns } = f.dir {
        s.display = Display::Grid;
        // ⚠️ **`max(1)` e não uma recusa.** Uma grade de zero colunas é um degenerado com UMA
        // leitura sã, e recusá-la faria o motor devolver `Err` para a fatia INTEIRA — a moldura
        // pararia de dispor, em silêncio, por causa de um número que o painel já impede. O clamp
        // mora aqui, e só aqui, porque este é o sítio que não pode dividir por zero.
        s.grid_template_columns = evenly_sized_tracks(columns.max(1));
        // ⚠️ **O `align_content` ESPELHA o `align`, e sem esta linha a grade herdaria o CSS.**
        // Numa grade `align-content: normal` comporta-se como `stretch`, então as linhas seriam
        // ESTICADAS para encher a moldura — a mesma cena que um `Row` encosta no topo saía com as
        // faixas espalhadas, e o filho a flutuar no meio de uma linha alta demais. O gate vivo
        // nasceu vermelho exactamente aí (topo 17,5 onde a aritmética pede 25).
        //
        // Espelhá-lo é o que mantém a palavra HONESTA: com uma linha só, `align` numa grade dá o
        // MESMO resultado que `align` num `Row` (o `Center` centra na moldura, o `Stretch` faz o
        // filho encher a moldura); com várias, ele diz *onde o bloco de linhas senta*, que é a
        // leitura que generaliza. Um controlo, duas propriedades do CSS, um significado.
        s.align_content = Some(match f.align {
            Align::Start => AlignContent::FLEX_START,
            Align::Center => AlignContent::CENTER,
            Align::End => AlignContent::FLEX_END,
            Align::Stretch => AlignContent::STRETCH,
        });
        // ⚠️ **`justify_items`, não `justify_content`** — e a diferença é o que o controlo passa a
        // significar. Com colunas `1fr` não sobra espaço nenhum no eixo horizontal, então
        // `justify_content` seria INERTE: cinco chips que não movem um pixel. `justify_items`
        // responde *onde o filho senta DENTRO da célula dele*, que é a pergunta que uma grade de
        // conteúdos de larguras diferentes de facto faz.
        //
        // ⚠️ E as duas DISTRIBUIÇÕES não têm resposta aqui (repartir sobra não é posicionar dentro
        // de uma caixa), então caem em `Start`. O painel **não as oferece** na grade; o que sobra é
        // o documento que já as trazia de um `Row` anterior, e ele é PRESERVADO de propósito — o
        // valor volta intacto quando o artista volta ao `Row`, como o vão e o recuo já voltam.
        s.justify_items = Some(match f.justify {
            Justify::Center => AlignItems::CENTER,
            Justify::End => AlignItems::FLEX_END,
            Justify::Start | Justify::SpaceBetween | Justify::SpaceAround => AlignItems::FLEX_START,
        });
        return s;
    }
    s.display = Display::Flex;
    let (dirn, wrap) = match f.dir {
        Dir::Column => (FlexDirection::Column, FlexWrap::NoWrap),
        Dir::RowWrap => (FlexDirection::Row, FlexWrap::Wrap),
        Dir::Row | Dir::Grid { .. } => (FlexDirection::Row, FlexWrap::NoWrap),
    };
    s.flex_direction = dirn;
    s.flex_wrap = wrap;
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

#[cfg(test)]
#[path = "hug_probe.rs"]
mod hug_probe;

#[cfg(test)]
#[path = "overflow_probe.rs"]
mod overflow_probe;

#[cfg(test)]
#[path = "grid_tests.rs"]
mod grid_tests;

#[cfg(test)]
#[path = "grid_probe.rs"]
mod grid_probe;
