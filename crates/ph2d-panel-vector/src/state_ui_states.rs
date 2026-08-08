//! **Os ESTADOS de UI da seleção** — a projeção que o painel lê (plano UI/UX W7).
//!
//! Irmão do [`crate::state_widget`], com a mesma divisão de donos: a verdade mora no documento
//! (`ph2d_ui_state::StateSets`, dentro do `ProjectState`) e isto é o que a shell publica por
//! frame. O painel não alcança o documento — se alcançasse, a resposta que decide QUE botão pintar
//! divergiria da que HONRA o clique.

use std::cell::RefCell;

/// O que a seleção tem, do ponto de vista dos estados.
#[derive(Clone, Debug, PartialEq)]
pub struct UiStatesState {
    /// Que papéis já foram gravados, na ordem de `StateRole::ALL`.
    ///
    /// ⚠️ Um array e não um mapa: os papéis são um catálogo FIXO, e a única coisa que varia é
    /// *qual deles tem pose*.
    pub recorded: [bool; 4],
    /// Os rótulos dos papéis, na ordem de `StateRole::ALL`.
    ///
    /// ⚠️ **Publicados pela shell, não constantes do painel** — o catálogo de papéis mora na
    /// `ph2d-ui-state`, e uma segunda lista aqui envelheceria no dia em que um papel novo
    /// nascesse: as linhas diriam uma coisa e o clique faria outra. É o mesmo protocolo do
    /// `WidgetSkinState::kinds`.
    pub role_labels: [String; 4],
    /// Qual papel a cena mostra AGORA, se alguma máquina está viva.
    ///
    /// ⚠️ Ele é o único jeito de o artista saber que a pré-visualização está ligada — sem isto,
    /// uma cena parada num hover pareceria a pose de repouso, e ele regravaria o Default por cima
    /// dela.
    pub live: Option<usize>,
    pub duration_s: f32,
    /// **O MODO DE PREVIEW** (W7r): `None` = não oferecer, `Some(on)` = o interruptor e se ele
    /// está ligado.
    ///
    /// ⚠️ **É um fato da CENA dentro de uma seção da SELEÇÃO**, e de propósito: a preview entrega
    /// o rato a *todos* os hospedeiros, não ao selecionado. O `None` é o que impede um botão que
    /// não faz nada — a shell só o oferece quando existe alguma pose autorada em algum lugar, que
    /// é exatamente a condição em que ligar a preview tem efeito.
    pub preview: Option<bool>,
    /// **Mover o widget carrega TODOS os estados** (Enio, 2026-08-07): `None` = não oferecer
    /// (nada gravado neste hospedeiro, logo não há o que carregar), `Some(on)` = o interruptor.
    ///
    /// ⚠️ **Ele qualifica o próximo ARRASTO, não o documento** — é por isso que ele não viaja no
    /// arquivo: é como o gesto se comporta, a mesma classe do `BakeChannels` da física.
    pub move_all: Option<bool>,
}

/// ⚠️ **A CURVA não é oferecida aqui, e a ausência é decisão MEDIDA, não esquecimento.** O
/// catálogo de easing tem **11 famílias × 3 modos = 33 combinações** e a crate que o possui
/// (`ph2d-anim`) **não dá nome a nenhuma** — não há `i18n_key` nem `label`. Um dropdown hoje
/// pintaria `Quad/In`, `Expo/Out`… em identificador inglês cru, que é exatamente o que o HR-15
/// proíbe; e uma tabela de rótulos vivendo NESTE painel seria a segunda lista a envelhecer ao
/// lado do enum. O default (`Cubic Out`) é o que a indústria converge, e o knob nasce quando o
/// catálogo ganhar nomes — no lugar onde as curvas moram.
const _CURVE_IS_DEFERRED_NOT_FORGOTTEN: () = ();

thread_local! {
    static STATES: RefCell<Option<UiStatesState>> = const { RefCell::new(None) };
}

/// Publica o estado da seleção (shell → painel). `None` = não oferecer a seção.
pub fn set_ui_states_state(state: Option<UiStatesState>) {
    STATES.with(|s| *s.borrow_mut() = state);
}

/// O estado da seleção — `None` = não oferecer a seção.
#[must_use]
pub(crate) fn ui_states_state() -> Option<UiStatesState> {
    STATES.with(|s| s.borrow().clone())
}
