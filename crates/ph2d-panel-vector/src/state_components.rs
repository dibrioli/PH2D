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

use std::cell::Cell;

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
    /// ⭐⭐⭐ **Esta cópia pode virar uma VERSÃO NOVA do prefab** (report do Enio, 2026-09-06:
    /// *«Make Prefab só aparece no menu da hierarchy e não no painel vector»*).
    ///
    /// ⚠️ **Não é derivável de `is_instance` aqui dentro, e a razão é o OUTRO motor:** promover uma
    /// cópia a variante é lei do modelo geral (ADR-0164), e o produtor vetorial não a tem. Um
    /// painel que a inferisse pintaria, no motor velho, um botão cujo dreno faz outra coisa.
    /// *Quem sabe o que o gesto faz é o produtor; o painel oferece o que lhe é publicado.*
    pub can_make_variant: bool,
    /// ⭐⭐⭐ **A receita desta cópia NÃO está no canvas** — ou seja, é preciso um gesto para lá
    /// chegar (*Edit Prefab*).
    ///
    /// ⚠️ **É um FACTO do modelo, e não o mesmo que [`Self::can_make_variant`]** — hoje os dois
    /// valem o mesmo, e por razões diferentes: no motor vetorial o mestre é uma forma **visível**,
    /// que o artista alcança clicando nela, e por isso ali este verbo não teria sujeito nenhum.
    /// *Colapsá-los faria a próxima mudança num deles mexer no outro sem que ninguém percebesse.*
    pub can_edit_prefab: bool,
    /// O conta-gotas do *Swap* está ARMADO (o próximo clique no canvas escolhe o mestre).
    ///
    /// ⚠️ Sem isto o botão pareceria não ter feito nada: um pick modal que não se anuncia é
    /// indistinguível de um clique perdido, e o artista carrega uma segunda vez.
    pub swap_armed: bool,
}

thread_local! {
    static COMPONENT: Cell<Option<ComponentState>> = const { Cell::new(None) };
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
