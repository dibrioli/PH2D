//! ⭐⭐⭐ **O PAINEL MOSTRA O QUE O DOCUMENTO DIZ** — os gates da lei (2026-08-30).
//!
//! # A lei, numa frase
//!
//! *O painel pinta o que o DOCUMENTO diz; o `WidgetStore` só vence enquanto a mão está no
//! controlo.*
//!
//! # O que estava errado
//!
//! As secções *Pattern* e *Brush* liam o store **primeiro e sempre**:
//!
//! ```ignore
//! let denom = self.store.number_value(id).unwrap_or(p.offset_denom);
//! ```
//!
//! ⚠️ **O `unwrap_or` nunca disparava.** `WidgetStore::number_value` devolve `Some` para **todo**
//! widget registado, e o `populate_*` regista todos ⇒ o valor que a shell publicou a partir do
//! documento era lido e **deitado fora**. O sintoma: escolher uma forma com *Offset 1/4* depois de
//! outra com *1/2* mostrava **1/2**, e o primeiro toque escrevia esse número alheio na forma nova.
//!
//! # ⚠️ A RÉGUA teve de ser deitada fora e refeita
//!
//! A 1.ª versão destes gates media **o store** depois de uma pintura — porque a 1.ª versão da cura
//! era uma *re-semeadura* por quadro. Quando a cura passou a ser a **inversão da leitura** (que não
//! escreve nada), a régua ficou a medir um campo que a cura não toca, e reprovava produto correcto.
//! *Uma régua desenhada para uma cura não mede a lei; mede aquela cura.*

use super::{live_number, live_track};
use ph2d_editor_core::interaction::{InteractiveState, WidgetStore};
use ph2d_editor_core::widget::{SliderOrientation, SliderState};

/// Os dois ids desta folha — quaisquer, porque o que se mede é a LEI e não um controlo em
/// particular. ⚠️ Usar ids REAIS do painel ligaria estes gates à disposição da secção, e eles
/// passariam a reprovar quando alguém acrescentasse uma fileira.
const SLIDER: ph2d_a11y::NodeId = ph2d_a11y::NodeId(9_910);
const NUM: ph2d_a11y::NodeId = ph2d_a11y::NodeId(9_911);

/// Um store com os dois widgets registados — é o estado em que o `populate` os deixa, e é
/// exactamente o que tornava o `unwrap_or(documento)` do código antigo **código morto**.
fn store_registado() -> WidgetStore {
    let mut s = WidgetStore::with_capacity(4);
    s.register(
        SLIDER,
        InteractiveState::Slider {
            state: SliderState::Normal,
            value: 0.0,
            orientation: SliderOrientation::Horizontal,
        },
    );
    s.register(
        NUM,
        InteractiveState::NumberInput {
            state: Default::default(),
            value: 0.0,
            buffer: String::new(),
            caret: 0,
            last_committed: 0.0,
            selection_anchor: None,
        },
    );
    s
}

/// ⭐⭐⭐ **O DOCUMENTO VENCE quando a mão não está no controlo** — o buraco inteiro.
///
/// O store carrega o resto da forma ANTERIOR (é o que uma troca de selecção deixa lá); o que se
/// desenha tem de ser o do documento.
#[test]
fn the_document_wins_when_no_hand_is_on_the_control() {
    let mut s = store_registado();
    // O resíduo da forma anterior.
    if let Some(InteractiveState::Slider { value, .. }) = s.get_mut(SLIDER) {
        *value = 0.25;
    }
    s.set_number_value(NUM, 2.0);

    assert!(
        (live_track(&s, SLIDER, 0.75) - 0.75).abs() < f32::EPSILON,
        "o slider desenhou a pista da forma ANTERIOR"
    );
    assert!(
        (live_number(&s, NUM, 4.0) - 4.0).abs() < f64::EPSILON,
        "o chip mostrou o numero da forma ANTERIOR - ele afirma um valor que o documento nao tem, \
         e o primeiro toque escreve esse valor alheio na forma nova"
    );
}

/// ⭐⭐ **A MÃO VENCE enquanto ela está no controlo** — a outra metade, e sem ela a cura seria pior
/// que o defeito.
///
/// ⚠️ **As duas guardas NÃO são a mesma, e é isso que carrega a lei:** arrastar um slider **não dá
/// foco** ao widget, e digitar **não** põe nada em `Dragging`. Trocá-las é um defeito silencioso —
/// uma caixa guardada por arrasto é reescrita a cada tecla, e um slider guardado por foco salta
/// para trás debaixo do dedo.
#[test]
fn the_hand_wins_while_it_is_on_the_control() {
    let mut s = store_registado();
    if let Some(InteractiveState::Slider { state, value, .. }) = s.get_mut(SLIDER) {
        *state = SliderState::Dragging;
        *value = 0.25;
    }
    assert!(
        (live_track(&s, SLIDER, 0.75) - 0.25).abs() < f32::EPSILON,
        "a pista saltou para o valor do documento DEBAIXO DO DEDO"
    );

    s.set_number_value(NUM, 6.0);
    s.set_focus(Some(NUM));
    assert!(
        (live_number(&s, NUM, 4.0) - 6.0).abs() < f64::EPSILON,
        "o documento apagou o que o artista estava a escrever - o campo em FOCO e' dele"
    );
}

/// ⛔⛔ **AS GUARDAS NÃO SE TROCAM** — o gate que apanha a permuta.
///
/// Um slider **em foco mas parado** tem de mostrar o documento (o foco não é um gesto sobre a
/// pista); uma caixa **num store com um slider a arrastar** tem de mostrar o documento (o arrasto é
/// do vizinho). Sem esta folha, copiar a guarda errada de um para o outro passa despercebido.
#[test]
fn the_two_guards_are_not_interchangeable() {
    let mut s = store_registado();
    // Um slider com FOCO, mas sem arrasto: o documento manda.
    if let Some(InteractiveState::Slider { value, .. }) = s.get_mut(SLIDER) {
        *value = 0.25;
    }
    s.set_focus(Some(SLIDER));
    assert!(
        (live_track(&s, SLIDER, 0.75) - 0.75).abs() < f32::EPSILON,
        "a pista usou a guarda de FOCO - ela ficaria presa no residuo sempre que o slider tivesse \
         sido tocado alguma vez"
    );

    // Uma caixa sem foco, com o vizinho a arrastar: o documento manda.
    s.set_number_value(NUM, 2.0);
    s.set_focus(None);
    if let Some(InteractiveState::Slider { state, .. }) = s.get_mut(SLIDER) {
        *state = SliderState::Dragging;
    }
    assert!(
        (live_number(&s, NUM, 4.0) - 4.0).abs() < f64::EPSILON,
        "o chip usou a guarda de ARRASTO - o arrasto do vizinho congelaria o numero"
    );
}

/// ⚠️ **Um widget NÃO REGISTADO cai no documento** — e não num `0` inventado.
///
/// É o caso de um controlo que ainda não foi pintado neste modo (o *Offset* só existe nos
/// reticulados que desfasam). *Um zero de «não medido» e um de «o artista pôs zero» são o mesmo
/// byte*, e esta linha impede que a lei os confunda.
#[test]
fn an_unregistered_control_falls_back_to_the_document() {
    let s = WidgetStore::with_capacity(1);
    assert!((live_track(&s, SLIDER, 0.75) - 0.75).abs() < f32::EPSILON);
    assert!((live_number(&s, NUM, 4.0) - 4.0).abs() < f64::EPSILON);
}
