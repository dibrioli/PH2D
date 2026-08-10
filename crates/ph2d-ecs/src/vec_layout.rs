//! **O AUTO LAYOUT, do lado do DOCUMENTO** — uma moldura que empilha os filhos.
//!
//! Dois componentes, e a divisão entre eles é a do CSS: um diz como um nó **DISPÕE** os filhos
//! ([`VecLayout`], no pai) e o outro como um nó **se comporta** dentro do pai ([`VecLayoutItem`],
//! no filho). Quem resolve os números é o motor confinado na `ph2d-vec-layout` (ADR-0153); aqui
//! está só o que o artista autora e o arquivo guarda.
//!
//! # A lei, e ela é a razão de esta metade existir separada
//!
//! > **O resultado do layout é uma POSE DERIVADA. Ele nunca é o `Transform` autorado.**
//!
//! ⚠️ O undo deste editor é por **DIFF do mundo ECS**: um passe que escrevesse `Transform` faria
//! cada frame de um redimensionamento virar um passo de undo, e brigaria com o arrasto do artista
//! dentro do mesmo frame. Por isso estes componentes descrevem uma REGRA, e nunca uma posição — a
//! posição nasce e morre no frame que a desenha.
//!
//! # Vocabulário próprio, e por quê
//!
//! ⚠️ [`LayoutDir`]/[`LayoutAlign`]/[`LayoutJustify`] são os enums do DOCUMENTO; o motor tem os
//! dele. É o mesmo par que o `BoolOp` (motor) e o `PathfinderOp` (artista) já formam nesta linha, e
//! que o `JointKind` forma entre a `ph2d-ecs` e a `ph2d-physics`: a crate do documento não depende
//! do motor, e **uma** porta de tradução os liga (na shell, com `match` exaustivo — variante nova
//! sem tradução não compila).
//!
//! # Componente NOVO não bumpa nada
//!
//! Cada um cunha `stable_type_id = blake3(NOME)[..8]` próprio ⇒ **zero bump de `PROJECT_SCHEMA`** —
//! o precedente literal do `VecBindings` (W4a), do `VecFrame` (W0) e do `PhysicsJoint` (W3 da
//! física). Ausência do componente = o mundo é byte-idêntico ao de antes desta feature.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// Direção do fluxo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LayoutDir {
    /// Em linha, sem quebra.
    #[default]
    Row,
    /// Em coluna, sem quebra.
    Column,
    /// Em linha, quebrando quando não cabe.
    RowWrap,
    /// **Em grade** de [`VecLayout::columns`] colunas iguais; as linhas nascem sozinhas.
    ///
    /// ⚠️ **Variante APENDADA**, e é isso que mantém legível todo arquivo já salvo: o postcard
    /// grava o discriminante, e `Row`/`Column`/`RowWrap` continuam em 0/1/2. O bump de
    /// `PROJECT_SCHEMA` que a acompanha é pelo caminho **INVERSO** — para um leitor velho recusar
    /// com um erro de VERSÃO em vez de estourar longe da causa (o raciocínio do `Cap::Square` do
    /// Flip e do `JointKind::Weld` da física).
    ///
    /// ⚠️ **A contagem NÃO vem no variante**, e a razão é o que o artista vê: com ela aqui, trocar
    /// para `Row` e voltar **destruiria o número** (ele viveria num variante que deixou de
    /// existir). Como campo do [`VecLayout`] ele sobrevive à ida e à volta, exactamente como o vão
    /// e o recuo já sobrevivem — e o `DIRS` da shell continua a ser uma tabela de pares que se lê
    /// nas duas direções por igualdade.
    Grid,
}

/// Alinhamento no eixo TRANSVERSAL.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LayoutAlign {
    #[default]
    Start,
    Center,
    End,
    /// Estica o filho para preencher a travessa.
    Stretch,
}

/// Distribuição no eixo PRINCIPAL.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LayoutJustify {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
}

/// **A moldura DISPÕE os filhos.** Presente ⇒ as posições dos filhos passam a ser derivadas;
/// ausente ⇒ eles ficam onde o artista os pôs.
///
/// ⚠️ Ele mora **ao lado** do [`crate::VecFrame`] em vez de dentro dele, e não é cerimónia: são
/// duas perguntas independentes (*"isto é um contêiner?"* × *"ele empilha o conteúdo?"*), e
/// apender campo a um componente existente **bumparia o schema** — que recusa todo projeto já
/// salvo. Um contentor que só agrupa continua a existir.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecLayout {
    pub dir: LayoutDir,
    /// Vão entre filhos: `[principal, transversal]`, em unidades de MUNDO (as do documento).
    pub gap: [f64; 2],
    /// Recuo interno `[topo, direita, base, esquerda]` — a ordem do CSS.
    pub pad: [f64; 4],
    pub align: LayoutAlign,
    pub justify: LayoutJustify,
    /// **Quantas colunas a grade tem** — lido só pelo [`LayoutDir::Grid`].
    ///
    /// ⚠️ Ele é inerte nas outras três direções, e essa forma **já existe neste struct**: o
    /// `gap[1]` (transversal) só significa alguma coisa no `RowWrap`, e a resposta do repo é a
    /// mesma nos dois casos — *o painel não o pinta onde ele não move um pixel*. Guardá-lo aqui,
    /// em vez de dentro do variante, é o que faz o número sobreviver a uma troca de direção.
    ///
    /// ⚠️ O teto vive no motor (`ph2d_vec_layout::MAX_GRID_TRACKS`, medido) e o piso é `1`; o
    /// motor lê `max(1)`, então um zero autorado é uma coluna e não uma divisão por zero.
    pub columns: u16,
}

impl Default for VecLayout {
    /// Uma linha justa: sem vão, sem recuo, tudo encostado no começo. É o neutro que deixa o
    /// artista ver o efeito de cada número que ele escreve a seguir.
    fn default() -> Self {
        Self {
            dir: LayoutDir::Row,
            gap: [0.0, 0.0],
            pad: [0.0; 4],
            align: LayoutAlign::Start,
            justify: LayoutJustify::Start,
            // ⚠️ **DUAS colunas, e não uma.** É a menor contagem em que a grade é distinguível de
            // uma `Column` — com uma só, clicar em *Grid* não mudaria um pixel, e o artista teria
            // de descobrir por tentativa que ainda faltava escrever um número.
            columns: 2,
        }
    }
}

/// **Como um filho se comporta dentro do fluxo.** Ausente = o neutro do [`Default`].
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecLayoutItem {
    /// Quanto ele toma da SOBRA. `0` = fica no tamanho dele; é o espaçador de uma barra de
    /// ferramentas que empurra o resto para o fim.
    pub grow: f32,
    /// Quanto ele CEDE quando falta espaço.
    pub shrink: f32,
    /// Tamanho de partida no eixo principal; `None` = o tamanho da própria forma.
    pub basis: Option<f64>,
}

impl Default for VecLayoutItem {
    /// ⚠️ **`shrink = 0`, e é uma divergência DELIBERADA do CSS** (onde o default é 1).
    ///
    /// No CSS o conteúdo é texto e encolher é a coisa gentil a fazer. Aqui o conteúdo é o desenho
    /// do artista, e encolher por omissão significaria **espremer as formas dele** no instante em
    /// que a moldura ficasse pequena demais — sem ninguém ter pedido, e com a moldura a recortar o
    /// transbordo de qualquer maneira (`VecFrame::clip`). É o default do Figma (*Fixed*), e
    /// crescer/encolher é opt-in.
    fn default() -> Self {
        Self {
            grow: 0.0,
            shrink: 0.0,
            basis: None,
        }
    }
}

/// **Como um nó decide o próprio TAMANHO num eixo.**
///
/// ⚠️ Aqui `Fixed` **não carrega número**, e no motor carrega — e a diferença não é descuido: o
/// número de uma forma é a **geometria dela**, que o documento já guarda. Guardá-lo outra vez aqui
/// seria a segunda resposta a *"que tamanho tem esta moldura?"*, e as duas divergiriam no primeiro
/// arrasto da alça de redimensionar.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LayoutSize {
    /// O tamanho que a forma TEM — o *Fixed* do Figma, e o default.
    #[default]
    Fixed,
    /// **Abraça o conteúdo** — o *Hug contents* do Figma: o tamanho sai do que está DENTRO.
    ///
    /// ⚠️ Só faz sentido num nó que DISPÕE os filhos ([`VecLayout`] presente). Quem monta a fatia
    /// para o motor não o oferece a mais ninguém, e o motor recusa se alguém tentar.
    Hug,
}

/// **O tamanho de um nó, e os limites dele** — o vocabulário completo de *sizing* do Figma.
///
/// ⚠️ Componente **próprio** em vez de campos apendados ao [`VecLayout`], e a razão é o preço: um
/// campo novo num componente existente é postcard **posicional** ⇒ bump de `PROJECT_SCHEMA` ⇒
/// **todo projeto já salvo é recusado**. Um componente novo cunha `stable_type_id` próprio e não
/// move número nenhum — o precedente do `VecFrame`, do `VecBindings` e do `PhysicsJoint`.
///
/// ⚠️ E ele é do **NÓ**, não do pai nem do filho: *"que tamanho eu tenho"* é uma pergunta sobre
/// si, enquanto o [`VecLayout`] é *"como disponho os outros"* e o [`VecLayoutItem`] é *"como me
/// comporto dentro do meu pai"*. As três respondem coisas diferentes e mudam por motivos
/// diferentes.
#[derive(Component, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct VecLayoutSize {
    /// Por eixo `[w, h]`.
    pub size: [LayoutSize; 2],
    /// **Piso** por eixo. O par que ele serve é `Hug` + `min`: *cresce com o rótulo, e nunca
    /// colapsa quando o texto é uma letra só*.
    pub min: [Option<f64>; 2],
    /// **Teto** por eixo. O par que ele serve é `grow` + `max`: *estica com a janela até uma
    /// largura de leitura, e para*.
    pub max: [Option<f64>; 2],
}

impl Default for VecLayoutSize {
    /// O neutro é o mundo de antes desta feature: os dois eixos no tamanho da forma, sem limites.
    fn default() -> Self {
        Self {
            size: [LayoutSize::Fixed; 2],
            min: [None; 2],
            max: [None; 2],
        }
    }
}

/// **Este filho NÃO participa do fluxo do pai** — o *Absolute position* do Figma.
///
/// A presença É o booleano (o idioma do `Ccd` e do `LockRotation` da física): um selo sobre a quina
/// de um card, um badge sobre um botão, um brilho por cima de tudo. Ele mantém a pose **autorada** —
/// continua a ser filho na hierarquia, logo continua a andar com o pai e a ser recortado por ele.
///
/// ⚠️ Sem isto, *todo* filho de uma moldura que flui entra no fluxo, e um overlay é **inexprimível**:
/// não há como pôr uma coisa por cima de outra dentro de um contentor com auto layout.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VecLayoutAbsolute;

impl SimComponent for VecLayout {}

impl SimComponent for VecLayoutItem {}

#[cfg(test)]
#[path = "vec_layout_tests.rs"]
mod tests;

impl SimComponent for VecLayoutSize {}

impl SimComponent for VecLayoutAbsolute {}
