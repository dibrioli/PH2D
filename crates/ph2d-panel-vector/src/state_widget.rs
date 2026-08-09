//! **A PELE da seleção** — a projeção que o painel lê (plano UI/UX W6.2).
//!
//! Irmão do [`crate::state_components`], com a mesma divisão de donos: a verdade mora no ECS
//! (`ph2d_ecs::VecWidget`) e isto é o que a shell publica por frame. O painel não alcança o mundo
//! — se alcançasse, a resposta que decide QUE chip pintar divergiria da que HONRA o clique.

use std::cell::{Cell, RefCell};

/// O que a seleção É, do ponto de vista da pele.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WidgetSkinState {
    /// Os nomes dos tipos do catálogo, na ordem em que os chips os oferecem.
    ///
    /// ⚠️ **Publicados pela shell, não constantes do painel** — o catálogo mora na
    /// `ph2d-editor-core::widget`, e uma segunda lista aqui envelheceria no dia em que um tipo
    /// novo nascesse: os chips diriam uma coisa e o clique faria outra.
    pub kinds: Vec<String>,
    /// Qual o vigente — `None` = a forma ainda **não** veste (oferece *Wear a Widget*).
    pub selected: Option<usize>,
    /// **O que esta row DIRIGE** (W8b.3) — `None` = este tipo não dirige nada e a linha não é
    /// pintada; `Some(None)` = pode dirigir e ainda não dirige; `Some(Some(nome))` = a forma.
    ///
    /// ⚠️ Três estados e não dois, porque *"não pode"* e *"pode e não está"* pedem UI oposta: o
    /// primeiro não oferece botão nenhum (um `Button` produz evento, não valor), o segundo oferece
    /// exatamente o conta-gotas. Colapsá-los daria um *Bind Shape* que resolve e não faz nada.
    pub drives: Option<Option<String>>,
    /// **O ÍCONE escolhido** (W8b §6.2) — `None` = este tipo não tem ícone e a row não é pintada;
    /// `Some(None)` = o botão desenha a FORMA; `Some(Some(slug))` = o glifo escolhido.
    ///
    /// ⚠️ Três estados e não dois, pela MESMA razão do [`Self::drives`] acima: *"não tem ícone"* e
    /// *"tem ícone e é o desenho"* pedem UI oposta — o primeiro não pinta row nenhuma, o segundo
    /// pinta o chip a dizer *Drawing*. Colapsá-los daria um picker num `Slider`.
    pub icon: Option<Option<String>>,
    /// A forma veste um tipo que este build **não conhece** (o readout de compatibilidade).
    ///
    /// ⚠️ Sem esta linha, um documento do futuro abriria mostrando a arte crua e nada diria por
    /// quê — e o artista concluiria que a pele dele se perdeu.
    pub unknown: bool,
}

thread_local! {
    static SKIN: RefCell<Option<WidgetSkinState>> = const { RefCell::new(None) };
    /// O retângulo do chip de ícone quando ele está ABERTO — o popover é pintado num passe
    /// diferido, e este é o recado que o corpo deixa para ele. Espelha o `PENDING_FONT_DD`.
    static PENDING_ICON_DD: Cell<Option<ph2d_editor_core::zones::Rect>> = const { Cell::new(None) };
    /// Quantos tipos do catálogo a tabela de ids NÃO endereça.
    ///
    /// ⚠️ Publicado, e não derivado do `len` da lista: quem trunca é a shell, e um número que o
    /// painel recalculasse seria a segunda resposta a *"quantos ficaram de fora?"*.
    static KINDS_BEYOND: Cell<usize> = const { Cell::new(0) };
}

/// Publica o estado da seleção (shell → painel). `None` = não oferecer a seção.
pub fn set_widget_skin_state(state: Option<WidgetSkinState>, beyond: usize) {
    SKIN.with(|s| *s.borrow_mut() = state);
    KINDS_BEYOND.with(|b| b.set(beyond));
}

/// O estado da seleção — `None` = não oferecer a seção.
#[must_use]
pub(crate) fn widget_skin_state() -> Option<WidgetSkinState> {
    SKIN.with(|s| s.borrow().clone())
}

/// Quantos tipos ficaram fora da tabela de ids.
pub(crate) fn widget_kinds_beyond() -> usize {
    KINDS_BEYOND.with(Cell::get)
}

/// O corpo do painel anuncia que o chip de ícone está aberto, e onde.
pub(crate) fn set_pending_icon_dd(chip: Option<ph2d_editor_core::zones::Rect>) {
    PENDING_ICON_DD.with(|c| c.set(chip));
}

/// O passe diferido consome o recado. ⚠️ **Consome**: deixá-lo lá pintaria o popover no frame
/// seguinte mesmo com o chip fechado.
pub(crate) fn take_pending_icon_dd() -> Option<ph2d_editor_core::zones::Rect> {
    PENDING_ICON_DD.with(Cell::take)
}
