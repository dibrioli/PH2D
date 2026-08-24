//! **A SEQUÊNCIA do Input Map leva a algum lugar** — a quarta condição de costura, e a única que
//! fica verde com a feature inalcançável.
//!
//! As três primeiras (*o componente existe · é pintado e registrado · o clique chega ao
//! barramento*) podem estar todas verdes e o artista continuar sem conseguir ligar uma tecla: basta
//! que o gesto **acabe** num sítio que não escreve no mapa. Este ficheiro percorre o caminho todo,
//! do item de menu até a acção ter a ligação.
//!
//! ⚠️ **E percorre-o pelas MESMAS portas que o produto usa** — `apply_event` e `dispatch_key`. Um
//! teste que chamasse `hero.input_map.get_mut(..).bindings.push(..)` provaria que uma `Vec` aceita
//! um `push`, que é uma afirmação sobre o Rust, não sobre esta feature.

use bumpalo::Bump;
use ph2d_a11y::NodeId;
use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::interaction::dispatch::dispatch_key;
use ph2d_editor_core::screens::hero::{HeroScreen, ids};
use ph2d_host::{KeyEvent, KeyKind, Modifiers};

fn hero() -> HeroScreen {
    HeroScreen::new(NodeId(1))
}

/// **A linha de uma acção na janela, pelo NOME.**
///
/// ⚠️ **Escrita depois de a W5 semear o mapa de fábrica**, e a lição é sobre gates: estes testes
/// nasceram a assumir `input_map_listen_id(0)` porque um `HeroScreen` novo tinha o mapa **vazio**.
/// Quando o projecto passou a nascer com os seis verbos do jogador, a linha `0` deixou de ser a
/// acção que o teste tinha acabado de criar — e eles reprovaram sobre produto **correcto**.
/// *Um índice literal é uma âncora na implementação; o nome é a âncora na lei.*
fn row_of(h: &HeroScreen, name: &str) -> usize {
    h.input_map
        .actions()
        .iter()
        .position(|a| a.name == name)
        .unwrap_or_else(|| panic!("a accao `{name}` tem de existir para ter linha"))
}

fn key(code: u32) -> KeyEvent {
    KeyEvent {
        keycode: code,
        kind: KeyKind::Down,
        modifiers: Modifiers::default(),
        timestamp_ns: 0,
    }
}

/// A tecla `Z` do exemplo — o botão de pulo do `player_input.rs` de hoje.
const KEY_Z: u32 = 0x5A;
const KEY_ESCAPE: u32 = 0x1B;

/// **O item de menu ABRE a janela.** Sem isto ela é código que ninguém alcança.
#[test]
fn the_settings_item_opens_the_window() {
    let mut h = hero();
    assert!(h.store.input_map_pos().is_none(), "nasce fechada");
    assert!(
        h.apply_event(WidgetEvent::Click(ids::CTX_MENU_SETTINGS_INPUT_MAP)),
        "o item de menu tem de ser CONSUMIDO -- se ninguem o consome, ele e' um botao mudo"
    );
    assert!(h.store.input_map_pos().is_some(), "a janela abriu");
}

/// ⭐⭐ **O CAMINHO INTEIRO**: abrir · nomear · Add · Bind… · carregar numa tecla · a ligação existe.
#[test]
fn the_whole_gesture_reaches_the_map() {
    /// A accao que ESTE teste cria -- e a linha dela procura-se pelo NOME, nunca por um indice.
    ///
    /// ⚠️ **NÃO pode ser `"jump"`:** desde a W5 o mapa de fábrica já a traz, com duas ligações — e
    /// `create` de um nome repetido devolve a que existe, em vez de criar uma segunda (que é a lei
    /// certa). O teste ficaria a medir o mapa de fábrica em vez do gesto que ele acabou de fazer.
    const NAME: &str = "test_action";
    let arena = Bump::new();
    let mut h = hero();
    h.apply_event(WidgetEvent::Click(ids::CTX_MENU_SETTINGS_INPUT_MAP));

    // O artista escreve o nome no campo -- pela mesma porta que o pintor lê.
    if let Some(ph2d_editor_core::interaction::InteractiveState::TextInput { text, .. }) =
        h.store.get_mut(ids::INPUT_MAP_NEW_NAME)
    {
        *text = NAME.to_string();
    }
    assert!(h.apply_event(WidgetEvent::Click(ids::INPUT_MAP_ADD)), "Add consumido");
    let jump = h.input_map.id(NAME).expect("a accao nasceu");
    assert!(
        h.input_map.get(jump).expect("existe").bindings.is_empty(),
        "e nasce SEM ligacao -- declarada e por atribuir"
    );

    // `Bind…` arma a escuta da linha 0.
    let row = row_of(&h, NAME);
    assert!(h.apply_event(WidgetEvent::Click(ids::input_map_listen_id(row))));
    assert_eq!(
        h.store.input_map_listening(),
        Some(jump),
        "a escuta ficou armada NA ACCAO CERTA"
    );

    // ⭐ A tecla: o despacho captura e emite; o chrome liga.
    let evts = dispatch_key(&mut h.store, key(KEY_Z), &arena);
    assert!(
        evts.contains(&WidgetEvent::Click(ids::INPUT_MAP_BIND_CAPTURED)),
        "o despacho de teclado nao emitiu a captura -- a tecla foi para outro lado"
    );
    for e in evts {
        h.apply_event(*e);
    }

    let a = h.input_map.get(jump).expect("existe");
    assert_eq!(a.bindings.len(), 1, "a ligacao chegou ao MAPA");
    assert_eq!(
        a.bindings[0],
        ph2d_input::Binding::Key(ph2d_input::Key(KEY_Z)),
        "e e' a tecla que o artista carregou"
    );
    assert!(
        h.store.input_map_listening().is_none(),
        "e a escuta desarmou sozinha -- um modo que fica armado come' a tecla seguinte"
    );
}

/// ⛔ **A tecla capturada NÃO executa o atalho do editor.**
///
/// ⚠️ É a condição 3 da costura, e o modo de falha é caro: ligar `S` a uma acção **salva o
/// projecto**, e o artista fica com um ficheiro gravado e nenhuma ligação. Por isso o ramo da
/// escuta é o **primeiro** do `dispatch_key`, acima do grafo, do foco e dos widgets de texto.
#[test]
fn a_captured_key_never_fires_the_editors_shortcut() {
    /// A accao que ESTE teste cria -- e a linha dela procura-se pelo NOME, nunca por um indice.
    const NAME: &str = "dash_test";
    let arena = Bump::new();
    let mut h = hero();
    h.apply_event(WidgetEvent::Click(ids::CTX_MENU_SETTINGS_INPUT_MAP));
    if let Some(ph2d_editor_core::interaction::InteractiveState::TextInput { text, .. }) =
        h.store.get_mut(ids::INPUT_MAP_NEW_NAME)
    {
        *text = NAME.to_string();
    }
    h.apply_event(WidgetEvent::Click(ids::INPUT_MAP_ADD));
    let row = row_of(&h, NAME);
    h.apply_event(WidgetEvent::Click(ids::input_map_listen_id(row)));

    // `Tab` é o caso mais visível: fora da escuta ele PERCORRE o foco.
    const KEY_TAB: u32 = 0x09;
    let evts = dispatch_key(&mut h.store, key(KEY_TAB), &arena);
    assert_eq!(
        evts,
        [WidgetEvent::Click(ids::INPUT_MAP_BIND_CAPTURED)],
        "sob escuta, a tecla so' pode produzir a CAPTURA -- e aqui ela percorreu o foco tambem"
    );
}

/// **O `Esc` desarma sem ligar** — um modo sem saída prende o teclado de quem armou por engano.
#[test]
fn escape_disarms_the_listening_without_binding() {
    /// A accao que ESTE teste cria -- e a linha dela procura-se pelo NOME, nunca por um indice.
    const NAME: &str = "grab_test";
    let arena = Bump::new();
    let mut h = hero();
    h.apply_event(WidgetEvent::Click(ids::CTX_MENU_SETTINGS_INPUT_MAP));
    if let Some(ph2d_editor_core::interaction::InteractiveState::TextInput { text, .. }) =
        h.store.get_mut(ids::INPUT_MAP_NEW_NAME)
    {
        *text = NAME.to_string();
    }
    h.apply_event(WidgetEvent::Click(ids::INPUT_MAP_ADD));
    let grab = h.input_map.id(NAME).expect("existe");
    let row = row_of(&h, NAME);
    h.apply_event(WidgetEvent::Click(ids::input_map_listen_id(row)));

    let evts = dispatch_key(&mut h.store, key(KEY_ESCAPE), &arena);
    assert!(evts.is_empty(), "o Esc nao produz captura nenhuma");
    assert!(h.store.input_map_listening().is_none(), "desarmou");
    assert!(
        h.input_map.get(grab).expect("existe").bindings.is_empty(),
        "e NAO ligou o Esc a' accao -- que e' o defeito que um `else` esquecido produziria"
    );
}

/// **Ligar a mesma tecla duas vezes não a duplica.**
///
/// ⚠️ Duas linhas iguais no painel seriam indistinguíveis ao apagar — o artista carrega no `x` de
/// uma e vê a outra ficar, sem nada dizer porquê.
#[test]
fn binding_the_same_key_twice_does_not_duplicate_it() {
    /// A accao que ESTE teste cria -- e a linha dela procura-se pelo NOME, nunca por um indice.
    const NAME: &str = "jump_test";
    let arena = Bump::new();
    let mut h = hero();
    h.apply_event(WidgetEvent::Click(ids::CTX_MENU_SETTINGS_INPUT_MAP));
    if let Some(ph2d_editor_core::interaction::InteractiveState::TextInput { text, .. }) =
        h.store.get_mut(ids::INPUT_MAP_NEW_NAME)
    {
        *text = NAME.to_string();
    }
    h.apply_event(WidgetEvent::Click(ids::INPUT_MAP_ADD));
    let jump = h.input_map.id(NAME).expect("existe");

    for _ in 0..2 {
        let row = row_of(&h, NAME);
    h.apply_event(WidgetEvent::Click(ids::input_map_listen_id(row)));
        let evts = dispatch_key(&mut h.store, key(KEY_Z), &arena);
        for e in evts {
            h.apply_event(*e);
        }
    }
    assert_eq!(
        h.input_map.get(jump).expect("existe").bindings.len(),
        1,
        "a mesma tecla entrou duas vezes na lista"
    );
}

/// **Fechar a janela desarma a escuta.**
///
/// ⚠️ Uma escuta que sobrevivesse ao fecho comeria a próxima tecla **com a janela já fechada** — e
/// nada na tela diria porquê. *Fechar é largar tudo.*
#[test]
fn closing_the_window_disarms_the_listening() {
    /// A accao que ESTE teste cria -- e a linha dela procura-se pelo NOME, nunca por um indice.
    const NAME: &str = "jump_close";
    let mut h = hero();
    h.apply_event(WidgetEvent::Click(ids::CTX_MENU_SETTINGS_INPUT_MAP));
    if let Some(ph2d_editor_core::interaction::InteractiveState::TextInput { text, .. }) =
        h.store.get_mut(ids::INPUT_MAP_NEW_NAME)
    {
        *text = NAME.to_string();
    }
    h.apply_event(WidgetEvent::Click(ids::INPUT_MAP_ADD));
    let row = row_of(&h, NAME);
    h.apply_event(WidgetEvent::Click(ids::input_map_listen_id(row)));
    assert!(h.store.input_map_listening().is_some());

    h.apply_event(WidgetEvent::Click(ids::INPUT_MAP_CLOSE));
    assert!(h.store.input_map_pos().is_none(), "fechou");
    assert!(
        h.store.input_map_listening().is_none(),
        "e a escuta foi junto -- senao ela comeria a proxima tecla fora da janela"
    );
}

/// **Um nome vazio não cria nada.** Uma acção sem nome é inalcançável por código (a leitura é
/// `pressed("...")`), então criá-la produziria uma linha que nada pode usar.
#[test]
fn an_empty_name_creates_nothing() {
    let mut h = hero();
    h.apply_event(WidgetEvent::Click(ids::CTX_MENU_SETTINGS_INPUT_MAP));
    // ⚠️ Conta ANTES e DEPOIS, em vez de `is_empty`: desde a W5 um projecto novo nasce com os
    // seis verbos do jogador, e `is_empty` mediria isso em vez de medir o Add.
    let before = h.input_map.len();
    assert!(h.apply_event(WidgetEvent::Click(ids::INPUT_MAP_ADD)), "consome o clique");
    assert_eq!(h.input_map.len(), before, "nao pode ter nascido accao nenhuma");
}
