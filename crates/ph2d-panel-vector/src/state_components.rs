//! **O COMPONENTE da seleção** — a projeção que o painel lê (plano UI/UX W5).
//!
//! Irmão do [`crate::state_anchors`], com a mesma divisão de donos: a verdade mora no ECS
//! (`ph2d_ecs::VecComponentMain` / `VecInstance`) e isto é o que a shell publica por frame. O
//! painel não alcança o mundo — se alcançasse, a resposta que decide QUE botão pintar divergiria
//! da que HONRA o clique.
//!
//! ⚠️ **Os quatro campos são a resposta a *"que verbos fazem sentido agora?"*, e nada mais.** Uma
//! contagem de instâncias, ou o nome do mestre, seriam factos que o painel mostraria e que ninguém
//! usa para decidir — e cada um deles é uma cópia que fica velha.

use std::cell::{Cell, RefCell};

/// **Uma PEÇA do mestre, vista desta instância** (plano UI/UX W5b).
///
/// A lista é a sub-árvore INTEIRA do mestre — nunca só as peças visíveis. Esconder uma peça
/// tirar-lhe-ia a linha, e o interruptor não teria volta: o gesto seria de mão única, que é a
/// forma mais barata de perder trabalho sem um erro.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstancePiece {
    /// O nome que a Hierarquia mostra — a peça é apontada por PALAVRA, não por índice.
    pub name: String,
    /// A cor EFETIVA: o override desta instância se houver, senão a do mestre.
    ///
    /// ⚠️ Efetiva e não autorada: uma swatch que mostrasse a cor do mestre numa peça overridada
    /// afirmaria um valor que o desenho não usa — a rachura que a swatch de Fill já documenta.
    pub colour: [u8; 4],
    /// Esta peça **aparece** nesta instância.
    pub visible: bool,
    /// Esta peça DIFERE do mestre (a marca que separa *"herdado"* de *"meu"*).
    pub overridden: bool,
}

/// **Um eixo de VARIANT e o que ele oferece** (plano UI/UX W5c).
///
/// ⚠️ Os eixos são **derivados dos NOMES** dos mestres irmãos (`Size=Small, State=Idle`, a
/// convenção do Figma) e não existem em lado nenhum como dado — guardá-los seria um segundo lugar
/// onde o mesmo facto envelhece, e o artista já os renomeia na Hierarquia.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct VariantRow {
    /// O nome da propriedade (`Size`). **Vazio** = o modo de nomes crus, e o rótulo é i18n.
    pub name: String,
    /// Os valores ALCANÇÁVEIS a partir da combinação vigente.
    ///
    /// ⚠️ Alcançáveis, e não todos: um valor que não corresponde a nenhum irmão seria um chip
    /// clicável que não faz nada — o botão-morto que este painel persegue.
    pub values: Vec<String>,
    /// Qual deles é o vigente.
    pub selected: usize,
}

/// O que a seleção É, do ponto de vista dos componentes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ComponentState {
    /// A seleção é um MESTRE (oferece *Place Instance*).
    pub is_main: bool,
    /// A seleção é uma INSTÂNCIA (oferece *Detach*).
    pub is_instance: bool,
    /// A instância selecionada tem overrides (oferece *Reset Overrides*).
    ///
    /// ⚠️ Separado de `is_instance` de propósito: um *Reset* sobre uma instância limpa é um clique
    /// que não faz nada, e o artista não tem como saber disso antes de o dar.
    pub has_overrides: bool,
    /// O mestre desta instância **não resolve** — o readout de órfã.
    pub main_missing: bool,
    /// O conta-gotas do *Swap* está ARMADO (o próximo clique no canvas escolhe o mestre).
    ///
    /// ⚠️ Sem isto o botão pareceria não ter feito nada: um pick modal que não se anuncia é
    /// indistinguível de um clique perdido, e o artista carrega uma segunda vez.
    pub swap_armed: bool,
}

thread_local! {
    static COMPONENT: Cell<Option<ComponentState>> = const { Cell::new(None) };
    /// As peças do mestre da instância selecionada — vazio quando não há instância.
    static PIECES: RefCell<Vec<InstancePiece>> = const { RefCell::new(Vec::new()) };
    /// Quantas peças o mestre tem ALÉM das que a lista endereça (`MAX_INSTANCE_PIECES`).
    ///
    /// ⚠️ Publicado, e não derivado do `len` da lista: quem trunca é a shell, e um número que o
    /// painel recalculasse seria a segunda resposta a *"quantas ficaram de fora?"*.
    static PIECES_BEYOND: Cell<usize> = const { Cell::new(0) };
    /// Os eixos de variant do mestre desta instância — vazio quando ele não tem irmãos.
    static VARIANTS: RefCell<Vec<VariantRow>> = const { RefCell::new(Vec::new()) };
    /// Quantas opções de variant a tabela de ids NÃO endereça.
    static VARIANTS_BEYOND: Cell<usize> = const { Cell::new(0) };
}

/// Publica os eixos de variant (shell → painel) e quantas opções ficaram além do teto.
pub fn set_variant_rows(rows: Vec<VariantRow>, beyond: usize) {
    VARIANTS.with(|v| *v.borrow_mut() = rows);
    VARIANTS_BEYOND.with(|b| b.set(beyond));
}

/// Os eixos publicados, para o corpo do painel.
pub(crate) fn variant_rows() -> Vec<VariantRow> {
    VARIANTS.with(|v| v.borrow().clone())
}

/// Quantas opções de variant ficaram fora da tabela de ids.
pub(crate) fn variant_rows_beyond() -> usize {
    VARIANTS_BEYOND.with(Cell::get)
}

/// Publica as peças do mestre (shell → painel) e quantas ficaram além do teto.
pub fn set_instance_pieces(pieces: Vec<InstancePiece>, beyond: usize) {
    PIECES.with(|p| *p.borrow_mut() = pieces);
    PIECES_BEYOND.with(|b| b.set(beyond));
}

/// As peças publicadas, para o corpo do painel.
pub(crate) fn instance_pieces() -> Vec<InstancePiece> {
    PIECES.with(|p| p.borrow().clone())
}

/// Quantas peças do mestre a lista NÃO endereça.
pub(crate) fn instance_pieces_beyond() -> usize {
    PIECES_BEYOND.with(Cell::get)
}

/// Publica o estado da seleção (shell → painel). `None` = não oferecer a seção.
pub fn set_component_state(state: Option<ComponentState>) {
    COMPONENT.with(|c| c.set(state));
}

/// O estado da seleção — `None` = não oferecer a seção.
#[must_use]
pub(crate) fn component_state() -> Option<ComponentState> {
    COMPONENT.with(Cell::get)
}

thread_local! {
    /// **O Z-INDEX da seleção** — `(z, quantos irmãos)`, com maior = mais à FRENTE.
    ///
    /// ⚠️ Mora aqui, no arquivo dos componentes, e não num state próprio: é a mesma classe de
    /// facto (algo que só a shell sabe, projetado do ECS por frame) e um arquivo por escalar seria
    /// o oposto do corte por assunto que este painel segue. `None` = seleção sem resposta (nada
    /// selecionado, ou mais de uma forma).
    static Z_INDEX: Cell<Option<f32>> = const { Cell::new(None) };
}

/// Publica o Z-index AUTORADO da seleção (shell → painel).
pub fn set_z_index(z: Option<f32>) {
    Z_INDEX.with(|c| c.set(z));
}

/// O Z-index autorado da seleção — `None` = não há resposta única.
pub(crate) fn z_index() -> Option<f32> {
    Z_INDEX.with(Cell::get)
}
