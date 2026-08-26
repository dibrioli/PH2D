//! **A projeção das SETAS do Morph** (shell → painel, plano 32 W4) — irmã do
//! [`crate::state_ui_states`], com a mesma divisão de donos: a verdade mora no documento
//! (`ph2d_ecs::VecMorphMachine`) e isto é o que a shell publica por frame.
//!
//! ⚠️ **O painel não alcança o mundo** — e não pode: se alcançasse, a resposta que decide QUE
//! linha pintar divergiria da que HONRA o clique.

use ph2d_editor_core::zones::Rect;
use std::cell::{Cell, RefCell};

/// Uma linha da lista — **uma seta**.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MorphArrowRow {
    /// O nome da forma de onde ela parte. ⚠️ **Nome e não id**: o artista escolheu formas, e um
    /// número não lhe diz qual delas é. Sem `Name` no documento, a shell cai num rótulo genérico —
    /// o nome é dado do documento, nunca copy de UI.
    pub from: String,
    pub to: String,
    /// A acção que a dispara. **Vazio = sem condição** — a seta só corre pela pré-visualização.
    pub when: String,
    /// **Esta é a seta que a cena está a percorrer AGORA.**
    pub live: bool,
}

/// O que a seleção tem, do ponto de vista da máquina de Morph.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct MorphStatesState {
    /// As setas, na ordem em que o artista as desenhou.
    ///
    /// ⚠️ **Vazio NÃO é "não há máquina"**: é *"há um Morph e ele ainda não tem setas"*, e as duas
    /// pintam faces diferentes — a segunda diz **como** desenhar a primeira.
    pub rows: Vec<MorphArrowRow>,
    /// **As acções do Input Map**, para o menu da condição.
    ///
    /// ⭐ Publicadas, nunca lidas pelo painel: elas são conteúdo autorado do projecto, e uma
    /// segunda leitura aqui envelheceria no dia em que o artista criasse uma acção nova.
    pub actions: Vec<String>,
    /// Em que forma a máquina está agora — o readout que diz *"a pré-visualização está a correr"*.
    pub current: Option<String>,
    /// ⭐ **Quantas formas a seleção tem prontas a virar um conjunto** (plano 32 W8). `0` = não há
    /// nada a oferecer.
    ///
    /// ⚠️ **Uma CONTAGEM, e não um `bool`**, porque a face de criação diz três coisas diferentes
    /// com ela: *escolha mais formas* (`< 2`) · *escolheu formas a mais* (`> MAX_MORPH_STATES`) ·
    /// *o botão, prometendo `n(n-1)` transições*. Um `bool` colapsaria as três em duas, e o artista
    /// que escolheu doze formas leria *"escolha duas ou mais"*.
    pub can_make: usize,
    /// ⭐⭐ **A PRÉ-VISUALIZAÇÃO está ligada** (plano 32 W9) — o modo em que o teclado é da máquina.
    ///
    /// ⚠️ Publicado pela shell, como tudo o resto: quem sabe se o modo corre é ela, e uma segunda
    /// memória disto no painel daria um botão aceso sobre um modo desligado.
    pub preview: bool,
}

thread_local! {
    static MORPH: RefCell<Option<MorphStatesState>> = const { RefCell::new(None) };
    /// **O menu da CONDIÇÃO que está ABERTO** — irmão do `PENDING_BLEND_DD` do `state.rs`, e pela
    /// mesma razão: o card vive dentro do scroll da seção, então sem o passe diferido a lista de
    /// acções seria cortada na borda dele.
    ///
    /// ⚠️ **Mora AQUI, e não com os irmãos**, porque o `state.rs` bateu no teto de 600 LOC no dia
    /// em que este slot lá entrou — e o corte por ASSUNTO já era o certo: este slot fala de setas,
    /// e é neste ficheiro que as setas vivem.
    static PENDING_WHEN_DD: Cell<Option<(usize, Rect)>> = const { Cell::new(None) };
}

pub(crate) fn set_pending_morph_when_dd(row_rect: Option<(usize, Rect)>) {
    PENDING_WHEN_DD.with(|c| c.set(row_rect));
}

pub(crate) fn take_pending_morph_when_dd() -> Option<(usize, Rect)> {
    PENDING_WHEN_DD.with(|c| c.take())
}

/// Publica o estado da seleção (shell → painel). `None` = a seleção não é um Morph.
pub fn set_morph_states_state(state: Option<MorphStatesState>) {
    MORPH.with(|s| *s.borrow_mut() = state);
}

/// O estado da seleção — `None` = não oferecer a lista.
#[must_use]
pub(crate) fn morph_states_state() -> Option<MorphStatesState> {
    MORPH.with(|s| s.borrow().clone())
}
