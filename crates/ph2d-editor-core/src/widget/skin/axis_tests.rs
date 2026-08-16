//! Gates do **EIXO do hover na pele** — a cauda da F2 (plano UI viva §6).
//!
//! ⚠️ **Filho de `tests.rs`, e não irmão:** as fixtures (`print`, `rect`, `text`) são as mesmas, e
//! duplicá-las daria duas impressões digitais para a mesma cena — o precedente é o `param_tests`
//! ao lado.
//!
//! O corte é de ASSUNTO: ali mora *a pele é a que o painel nativo pinta* (o gate de bytes, o
//! rótulo, a moldura, o estado vivo); aqui, *quanto da transição já passou chega à tinta*. As duas
//! falham por motivos diferentes — uma quando alguém redesenha um widget à mão, a outra quando um
//! arm copia o estado DISCRETO e deita fora o `t`.

use super::*;

/// ⭐ **A pele INTERPOLA — meio caminho não é nenhum dos dois extremos.**
///
/// Este é o defeito que a wave fechou: a pele copiava o estado DISCRETO do `live` e deixava o
/// `hover_t` no default `1.0`, o valor que faz o `Button::bg_color` cair no token duro. O painel
/// gerado **reagia e SALTAVA** enquanto os vinte e três painéis escritos à mão amaciavam — metade
/// do app numa lei, metade noutra, que é o sintoma que a F2 existe para curar.
///
/// ⚠️ **A lista é a dos tipos que TÊM o eixo, e a versão anterior desta nota ERRAVA:** ela dizia
/// que `Tag` **e** `ListItem` ficavam de fora *"por construção"*, honrando a cerca do estudo §6.5
/// (*«uma row de menu que amacia deixa rasto»*). O estudo isenta a tag **explicitamente** —
/// *"ficam por avaliar `TextInput`/`Dropdown`/`Tag`, que **não têm esse risco** (ninguém varre
/// dropdowns)"* —, e medido, os outros dois já tinham eixo. A cerca é do `ListItem`, e só dele;
/// a tag entrou nesta lista.
///
/// *Mutação que deve sangrar:* trocar `visual((*state, t))` por `state = *state` em qualquer arm
/// ⇒ o meio passa a ser byte-idêntico a um dos extremos.
#[test]
fn half_a_hover_is_neither_end_for_every_kind_that_has_the_axis() {
    let mut ts = text();
    // ⚠️ **O `IconButton` precisa de um GLIFO, e o controlo do gate foi quem o disse.** No estilo
    // `Compact` o eixo do hover vive na TINTA do glifo (`icon_tint_t`), então com o `SkinParam`
    // neutro — que desenha um caminho vazio — os dois extremos pintam o mesmo e o gate falhava a
    // afirmar o próprio pressuposto. *A fixture tem de conter o fenómeno.*
    let g = super::param::glyph(20.0);
    let icon = SkinParam {
        icon: Some(IconGlyph::Path(&g)),
        ..SkinParam::default()
    };
    let cases: [(WidgetKind, crate::interaction::InteractiveState); 7] = [
        (
            WidgetKind::Tag,
            crate::interaction::InteractiveState::Tag {
                state: crate::widget::TagState::Hovered,
            },
        ),
        (
            WidgetKind::Button,
            crate::interaction::InteractiveState::Button {
                state: crate::widget::ButtonState::Hovered,
            },
        ),
        (
            WidgetKind::IconButton,
            crate::interaction::InteractiveState::Button {
                state: crate::widget::ButtonState::Hovered,
            },
        ),
        (
            WidgetKind::Toggle,
            crate::interaction::InteractiveState::Toggle {
                state: crate::widget::ToggleState::Hovered,
                on: false,
            },
        ),
        (
            WidgetKind::Checkbox,
            crate::interaction::InteractiveState::Checkbox {
                state: crate::widget::CheckboxState::Hovered,
                value: crate::widget::CheckboxValue::Unchecked,
            },
        ),
        (
            WidgetKind::Slider,
            crate::interaction::InteractiveState::Slider {
                state: crate::widget::SliderState::Hovered,
                value: 0.5,
                orientation: crate::widget::SliderOrientation::Horizontal,
            },
        ),
        (
            WidgetKind::TextInput,
            crate::interaction::InteractiveState::TextInput {
                state: crate::widget::TextInputState::Hovered,
                text: String::new(),
                caret: 0,
                selection_anchor: None,
            },
        ),
    ];

    for (kind, live) in &cases {
        let mut shot = |t: f32| {
            let mut scene = ph2d_vector::VectorScene::new();
            paint_widget_skin_with(
                *kind,
                "Save",
                NodeId(7),
                Some((live, t)),
                if matches!(kind, WidgetKind::IconButton) {
                    icon
                } else {
                    SkinParam::default()
                },
                rect(),
                &mut scene,
                &mut ts,
                Theme::Forge,
            );
            print(&scene)
        };
        let (cold, mid, hot) = (shot(0.0), shot(0.5), shot(1.0));
        assert_ne!(
            cold, hot,
            "{kind:?}: os dois extremos do eixo pintam o MESMO — a fixture nao contem o fenomeno"
        );
        assert_ne!(
            mid, cold,
            "{kind:?}: meio caminho pinta o extremo FRIO — a pele deitou fora o `t`"
        );
        assert_ne!(
            mid, hot,
            "{kind:?}: meio caminho pinta o extremo QUENTE — a pele deitou fora o `t`"
        );
    }
}

/// **E o par vem do STORE, de um id só.**
///
/// ⚠️ O oráculo é o `WidgetStore` real: registar o botão, aquecê-lo pelo ponteiro e publicar meio
/// hover tem de dar a MESMA cena que passar o par à mão. Sem isto, a porta podia devolver o `t` de
/// outro id e a pele continuaria a interpolar — bonita e errada.
///
/// *Mutação que deve sangrar:* o `skin_live` devolver `SETTLED` em vez de perguntar ao relógio.
#[test]
fn the_pair_the_store_hands_out_is_the_pair_the_skin_paints() {
    let mut ts = text();
    let id = NodeId(7);
    let mut store = crate::interaction::WidgetStore::with_capacity(2);
    store.register(
        id,
        crate::interaction::InteractiveState::Button {
            state: crate::widget::ButtonState::Hovered,
        },
    );
    store.set_hover_live(id, 0.5);

    let mut from_store = ph2d_vector::VectorScene::new();
    paint_widget_skin_with(
        WidgetKind::Button,
        "Save",
        id,
        store.skin_live(id),
        SkinParam::default(),
        rect(),
        &mut from_store,
        &mut ts,
        Theme::Forge,
    );

    let live = crate::interaction::InteractiveState::Button {
        state: crate::widget::ButtonState::Hovered,
    };
    let mut by_hand = ph2d_vector::VectorScene::new();
    paint_widget_skin_with(
        WidgetKind::Button,
        "Save",
        id,
        Some((&live, 0.5)),
        SkinParam::default(),
        rect(),
        &mut by_hand,
        &mut ts,
        Theme::Forge,
    );

    assert_eq!(
        print(&from_store),
        print(&by_hand),
        "a porta do store nao entrega o par que a pele consome"
    );
    assert!(
        store.skin_live(NodeId(999)).is_none(),
        "um id que o populate nunca registou tem de cair na PREVIA, nao num par fabricado"
    );
}
