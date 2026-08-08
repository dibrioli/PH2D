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
    /// **A MOLA**: `None` = o par duração+curva, `Some((rigidez, amortecimento))` = ela.
    ///
    /// ⚠️ Uma `Option` de PAR, e não um bool ao lado de dois números: *ter mola* e *que mola* são
    /// a mesma decisão, e separá-los daria ao painel dois estados para manter de acordo — o
    /// checkbox a dizer uma coisa e as linhas a mostrar outra.
    pub spring: Option<(f32, f32)>,
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
    /// **A CURVA da transição** — a que o hospedeiro selecionado usa agora.
    ///
    /// ⚠️ **É o `Easing` inteiro, não um par de índices**, e a escolha evita a classe de defeito
    /// que este repo já pagou várias vezes: um índice publicado por um lado e resolvido por outro
    /// obriga as duas pontas a concordarem sobre a ORDEM de `EasingFamily::ALL`, e no dia em que
    /// alguém reordenasse o catálogo o painel acenderia um chip e o documento guardaria outra
    /// curva. Com o enum, a comparação é do próprio valor.
    ///
    /// Este campo **já viajava no arquivo** desde o v56 (`HostStates.easing`) e não tinha porta
    /// nenhuma — o seletor é a porta, e é por isso que a wave não custa schema.
    pub easing: ph2d_anim::Easing,
}

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
