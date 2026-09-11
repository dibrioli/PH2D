//! ⭐⭐⭐ **As duas aparências estão a UM interruptor de distância — e o clássico continua vivo.**
//!
//! Enio, 2026-09-03: primeiro *«essa nova UI ainda deve ficar desativada até que esteja
//! concluída»*, e horas depois, com a lista do que falta na mão, *«vamos integrar então com toda a
//! UI nova»*. ⇒ **o redesenho passou a ser o caminho de omissão**; o clássico volta com
//! `PH2D_UI_NEW=0`.
//!
//! ⛔ **O que este ficheiro defende MUDOU DE LADO, e por isso ele mudou de NOME.** Já não afirma
//! *«sai o antigo»* — afirma que **os dois existem e diferem**, que é a parte que sobrevive à
//! decisão: no dia em que alguém apagar o caminho clássico, ninguém consegue responder à pergunta
//! *«isto já era assim antes?»* sem um `git stash`.
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

/// **A thread nasce no REDESENHO** — a decisão do dono, 2026-09-03.
#[test]
fn the_neutral_look_is_the_redesign() {
    assert_eq!(
        ui_look(),
        UiLook::Redesign,
        "o app nasce no classico: o dono mandou integrar com a UI nova"
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

/// **E o que a thread pinta SEM ninguém escolher é o REDESENHO.**
///
/// ⚠️ Não basta *«existe um redesenho»*: o que se afirma é que **é ele que sai quando ninguém pede
/// nada** — a única forma de o dizer sobre o produto e não sobre uma constante.
#[test]
fn what_the_app_paints_without_being_asked_is_the_redesign() {
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
        checkbox_ink(UiLook::Redesign),
        "sem ninguem escolher aparencia, o app pintou algo que NAO e' o redesenho"
    );
}

/// **Só `0` volta ao clássico.**
///
/// ⛔ A assimetria é a mesma de antes, virada ao contrário: *quem desvia do caminho normal tem de o
/// dizer com uma palavra exacta*. Um `PH2D_UI_NEW=talvez` não pode devolver um artista à UI velha
/// sem ele perceber porquê.
#[test]
fn only_the_exact_value_goes_back_to_the_classic() {
    assert_eq!(UiLook::from_env_value(Some("0")), UiLook::Classic);
    for v in [
        None,
        Some(""),
        Some("1"),
        Some("false"),
        Some("no"),
        Some(" 0"),
    ] {
        assert_eq!(
            UiLook::from_env_value(v),
            UiLook::Redesign,
            "o valor {v:?} devolveu o app a' UI velha"
        );
    }
}

/// ⛔⛔ **A PORTA das linhas construídas à mão também obedece — e foi ELA que vazou.**
///
/// Report do Enio, 2026-09-03, sobre a 1.ª versão deste interruptor: sem `PH2D_UI_NEW=1` o app
/// *«abriu o desenho novo»*. ⚠️ **O gate acima estava verde**, porque media os três PINTORES que eu
/// me lembrei — e as linhas do Inspector não passam por nenhum deles: são construídas à mão, com a
/// sua própria aritmética, e chamam a `form_row_columns`. Eram **19 sítios**, e nenhum perguntava
/// a aparência.
///
/// ⇒ *um gate que enumera os sítios que o autor recorda mede a memória dele, não o produto.* A
/// guarda mudou-se para a **porta**, e este teste mede a porta.
///
/// **Mutação que deve sangrar:** tirar o `if !ui_is_redesign()` da `form_row_columns` — o caminho
/// clássico deixa de existir, e a pergunta *«isto já era assim antes?»* fica sem resposta.
#[test]
fn the_hand_rolled_row_door_obeys_the_look_too() {
    use ph2d_editor_core::widget::{DECORATOR_W, form_row_columns};

    set_ui_look(UiLook::Classic);
    let (classic_w, classic_dot) = form_row_columns(10.0, 200.0, 0.0, 22.0);
    set_ui_look(UiLook::Redesign);
    let (new_w, new_dot) = form_row_columns(10.0, 200.0, 0.0, 22.0);
    set_ui_look(UiLook::default());

    assert!(
        (classic_w - 200.0).abs() < 0.001,
        "no classico a porta comeu {} px da linha: a coluna de animacao nao existe ali",
        200.0 - classic_w
    );
    assert!(
        classic_dot.w.abs() < 0.001,
        "no classico a porta devolveu uma coluna de {} px de largura",
        classic_dot.w
    );
    assert!(
        (new_w - (200.0 - DECORATOR_W)).abs() < 0.001,
        "no redesenho a porta nao reserva a coluna"
    );
    assert!((new_dot.w - DECORATOR_W).abs() < 0.001);
}

/// ⛔ **E o ponto não se pinta no clássico, mesmo que alguém lhe passe um rect.**
///
/// ⚠️ É a **segunda metade** da guarda: um chamador que ignore a largura devolvida pela porta ainda
/// assim não consegue pintar a coluna. *Duas metades, para que esquecer uma não chegue.*
#[test]
fn the_dot_paints_nothing_in_the_classic_look() {
    use ph2d_editor_core::widget::paint_decorator_dot;
    let ink = |look: UiLook| {
        set_ui_look(look);
        let mut scene = VectorScene::new();
        paint_decorator_dot(&mut scene, Theme::Forge, Rect::new(0.0, 0.0, 14.0, 22.0));
        set_ui_look(UiLook::default());
        scene.inner().encoding().path_data.clone()
    };
    assert!(
        ink(UiLook::Classic).is_empty(),
        "o ponto foi pintado no classico: um chamador que ignore a largura da porta vaza o \
         redesenho para o caminho de omissao"
    );
    assert!(
        !ink(UiLook::Redesign).is_empty(),
        "o ponto nao foi pintado no redesenho: a guarda apagou a feature em vez de a gatear"
    );
}

// ⛔⛔ **A GALERIA não tem gate aqui, e a ausência está NOMEADA.**
//
// Ela mostra a caixa única, e sob o clássico deixou de a mostrar — mas o ponto de entrada
// (`showcase::paint_slider_section`) é `pub(super)`, logo **não é alcançável de um teste de
// integração**. ⛔ Alargar a visibilidade de uma função de produto para um teste lhe chegar é pagar
// com API o que se quer medir.
//
// ⭐ **O instrumento que a encontrou foi o CENSO, não a leitura:** depois do vazamento, correr
// *«quem chama um pintor do redesenho?»* devolveu doze ficheiros, e este chamava o
// `paint_property_box` **directamente**. *A lista de consumidores é uma pergunta ao repo; a minha
// memória é um palpite* — e foi a memória que produziu o gate verde sobre um app vazado.
//
// ⏳ O gate honesto vive na crate do painel da galeria, com o `MockPanelHost`. Fica nomeado.
