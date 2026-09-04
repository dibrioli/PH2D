//! ⭐⭐⭐ **Por omissão o app veste a UI ANTIGA** — o redesenho é opcional até estar concluído.
//!
//! Enio, 2026-09-03, ao mandar integrar com o `main`: *«essa nova UI ainda deve ficar desativada
//! até que esteja concluída. Por enquanto permanece a antiga.»*
//!
//! ⛔ **Este gate é a condição de a linha poder ser integrada.** Sem ele, «está desligado» é uma
//! afirmação sobre um `enum` que qualquer wave muda sem reparar — e o sintoma seria o `main` a
//! shipar um redesenho a meio, que é exactamente o que o dono recusou.
//!
//! # ⚠️ Ele mede TINTA, não a bandeira
//!
//! Afirmar `UiLook::default() == Classic` seria uma tautologia sobre uma linha de código. O que
//! aqui se mede é o que o artista vê: os três pintores que o redesenho mudou pintam **coisas
//! diferentes** conforme a aparência, e no neutro pintam a de sempre.

use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::{HitIndex, InteractiveState, WidgetStore};
use ph2d_editor_core::paint::{set_ui_look, ui_look};
use ph2d_editor_core::widget::{
    Checkbox, SliderOrientation, SliderState, Toggle, paint_checkbox, paint_slider_with_chip,
    paint_toggle,
};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::{Theme, UiLook};
use ph2d_vector::VectorScene;

const ID: NodeId = NodeId(1);
const CHIP: NodeId = NodeId(2);
const ROW: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 240.0,
    h: 24.0,
};

type Ink = (Vec<u32>, Vec<u32>);

fn ink(look: UiLook, f: impl FnOnce(&mut VectorScene, &mut TextSystem)) -> Ink {
    set_ui_look(look);
    let mut scene = VectorScene::new();
    let mut text = TextSystem::without_system_fonts();
    f(&mut scene, &mut text);
    set_ui_look(UiLook::default());
    let e = scene.inner().encoding();
    (e.path_data.clone(), e.draw_data.clone())
}

fn row_ink(look: UiLook) -> Ink {
    ink(look, |scene, text| {
        let mut store = WidgetStore::with_capacity(4);
        store.register(
            ID,
            InteractiveState::Slider {
                state: SliderState::Normal,
                value: 0.5,
                orientation: SliderOrientation::Horizontal,
            },
        );
        let mut hit = HitIndex::default();
        paint_slider_with_chip(
            ROW,
            "Opacity",
            0.5,
            ID,
            CHIP,
            &store,
            &mut hit,
            scene,
            text,
            Theme::Forge,
        );
    })
}

fn checkbox_ink(look: UiLook) -> Ink {
    ink(look, |scene, text| {
        paint_checkbox(&Checkbox::new(ID, ""), ROW, scene, text, Theme::Forge);
    })
}

fn toggle_ink(look: UiLook) -> Ink {
    ink(look, |scene, _text| {
        paint_toggle(
            &Toggle::new(ID, "").on(true),
            Rect::new(0.0, 0.0, 60.0, 24.0),
            scene,
            Theme::Forge,
        );
    })
}

/// **A thread nasce na aparência CLÁSSICA.**
#[test]
fn the_neutral_look_is_the_old_one() {
    assert_eq!(
        ui_look(),
        UiLook::Classic,
        "o app nasce no REDESENHO: ele ainda nao esta' concluido, e o dono mandou manter a antiga"
    );
}

/// **Os três pintores que o redesenho mudou pintam coisas DIFERENTES nas duas aparências.**
///
/// ⚠️ É esta a metade que impede o gate de ser vácuo: se o `Classic` e o `Redesign` pintassem o
/// mesmo, o interruptor não existiria e o «está desligado» não significaria nada.
///
/// **Mutação que deve sangrar:** apagar o `if !ui_is_redesign()` de qualquer um dos três — aquela
/// tinta passa a ser a mesma nas duas aparências, e o caminho de omissão leva o redesenho consigo.
#[test]
fn the_switch_actually_switches_all_three_painters() {
    assert_ne!(
        row_ink(UiLook::Classic),
        row_ink(UiLook::Redesign),
        "a LINHA DE PROPRIEDADE pinta o mesmo nas duas aparencias: ou o classico foi perdido, ou \
         o redesenho nao esta' atras do interruptor"
    );
    assert_ne!(
        checkbox_ink(UiLook::Classic),
        checkbox_ink(UiLook::Redesign),
        "a CAIXA DE VERIFICACAO pinta o mesmo nas duas aparencias"
    );
    assert_ne!(
        toggle_ink(UiLook::Classic),
        toggle_ink(UiLook::Redesign),
        "o INTERRUPTOR pinta o mesmo nas duas aparencias"
    );
}

/// **E o que a thread pinta SEM ninguém escolher é o clássico** — não o redesenho.
///
/// ⚠️ Este é o teste que de facto responde à instrução do dono: não *«existe um clássico»*, mas
/// *«é ele que sai quando ninguém pede nada»*.
#[test]
fn what_the_app_paints_without_being_asked_is_the_classic() {
    let untouched = {
        let mut scene = VectorScene::new();
        let mut text = TextSystem::without_system_fonts();
        paint_checkbox(
            &Checkbox::new(ID, ""),
            ROW,
            &mut scene,
            &mut text,
            Theme::Forge,
        );
        let e = scene.inner().encoding();
        (e.path_data.clone(), e.draw_data.clone())
    };
    assert_eq!(
        untouched,
        checkbox_ink(UiLook::Classic),
        "sem ninguem escolher aparencia, o app pintou algo que NAO e' o classico"
    );
}

/// **A leitura do ambiente só liga com `1`.**
///
/// ⛔ *Um interruptor que liga com o que não percebe é um interruptor que se liga sozinho* — e este
/// governa se o `main` mostra um redesenho a meio.
#[test]
fn only_the_exact_value_turns_it_on() {
    assert_eq!(UiLook::from_env_value(Some("1")), UiLook::Redesign);
    for v in [
        None,
        Some(""),
        Some("0"),
        Some("true"),
        Some("yes"),
        Some(" 1"),
    ] {
        assert_eq!(
            UiLook::from_env_value(v),
            UiLook::Classic,
            "o valor {v:?} ligou o redesenho"
        );
    }
}
