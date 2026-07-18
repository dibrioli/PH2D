//! O estado publicado da seção **EFFECTS** (ADR-0132) — módulo irmão do [`super`] pelo teto
//! de 600 LOC.
//!
//! # A seção é DIRIGIDA PELA TABELA, e é isso que a torna barata
//!
//! O painel não sabe o que é um Trim nem um Zig Zag: ele recebe **linhas** com um rótulo e uma
//! lista de parâmetros DESCRITOS (nome, faixa, é-caixinha, valor) e as desenha. Acrescentar um
//! efeito ao motor passa a custar **zero mudança de painel** — o padrão que a rack de áudio já
//! provou aqui (*"o painel se auto-popula da tabela `KINDS`"*).
//!
//! Isto foi **medido, não suposto**: o 1º efeito (Trim) custou uma rodada inteira dos 8 sites
//! de costura, e o 2º (Zig Zag) ia custar outra. O gargalo tinha deixado de ser a geometria, e
//! a promessa do ADR só valia para metade do caminho.

use std::cell::{Cell, RefCell};

/// Um parâmetro de efeito, como o painel o desenha.
///
/// Espelha o `FxParam` do motor — o painel **não alcança** o `ph2d-vec-scene` (ele vive de
/// snapshots), e é a shell que traduz na fronteira.
#[derive(Clone, Debug, PartialEq)]
pub struct FxParamView {
    /// O rótulo do controle.
    pub name: &'static str,
    /// A faixa, no domínio do DOCUMENTO.
    pub min: f64,
    /// O topo da faixa.
    pub max: f64,
    /// `true` = caixinha (o valor só é 0 ou 1); `false` = slider.
    pub toggle: bool,
    /// `true` = o valor é uma CONTAGEM: mostra-se sem casas decimais, e o chip arredonda o que
    /// o utilizador escreve. O motor já guarda o inteiro — isto só evita que a tela o contradiga.
    pub integer: bool,
    /// O valor ATUAL.
    pub value: f64,
}

/// Uma linha da pilha: o efeito e os parâmetros dele.
#[derive(Clone, Debug, PartialEq)]
pub struct FxRowView {
    /// O nome do efeito ("Trim Path", "Zig Zag"…).
    pub label: &'static str,
    /// O efeito está LIGADO? Desligado, a pilha o salta e o card é desenhado apagado — mas os
    /// parâmetros continuam lá e editáveis.
    pub enabled: bool,
    /// Os parâmetros, na ordem em que o painel os desenha.
    pub params: Vec<FxParamView>,
}

thread_local! {
    /// A pilha do caminho selecionado, na ordem em que se aplica.
    static CURRENT_STACK: RefCell<Vec<FxRowView>> = const { RefCell::new(Vec::new()) };
    /// Os tipos que o menu "Add" oferece — publicados a partir da tabela do motor, então um
    /// efeito novo aparece no menu sem o painel saber que ele existe.
    static CURRENT_KINDS: RefCell<Vec<&'static str>> = const { RefCell::new(Vec::new()) };
    /// Há exatamente UM caminho selecionado? A seção é por-caminho: sem alvo, *"a pilha"* não
    /// tem referente, e nem os botões de Add são oferecidos.
    static HAS_TARGET: Cell<bool> = const { Cell::new(false) };
}

/// **Publica a seção Effects inteira** — o alvo, os tipos disponíveis e a pilha.
///
/// Os três viajam JUNTOS de propósito: em setters separados haveria um frame em que o painel
/// desenha a pilha de um caminho com os tipos de outro. Mesma razão do
/// `set_current_envelope_presets`.
pub fn set_current_effects(has_target: bool, kinds: &[&'static str], stack: Vec<FxRowView>) {
    HAS_TARGET.with(|c| c.set(has_target));
    CURRENT_KINDS.with(|c| c.borrow_mut().clear());
    CURRENT_KINDS.with(|c| c.borrow_mut().extend_from_slice(kinds));
    CURRENT_STACK.with(|c| *c.borrow_mut() = stack);
}

/// Há um caminho único selecionado?
pub(crate) fn has_target() -> bool {
    HAS_TARGET.with(Cell::get)
}

/// Os tipos do menu "Add".
pub(crate) fn kinds() -> Vec<&'static str> {
    CURRENT_KINDS.with(|c| c.borrow().clone())
}

/// A pilha publicada.
pub(crate) fn stack() -> Vec<FxRowView> {
    CURRENT_STACK.with(|c| c.borrow().clone())
}
