//! ⭐ **O despacho do teclado, da composição e do rato até à aba de código** (auditoria do fecho) —
//! pelas portas do hero que a shell chama: a tecla morta (`handle_compose_key`, ANTES de tudo), a
//! composição do IME (`handle_preedit`/`handle_text_input`), a tecla (`board_keys::key`), o rato
//! (`board_view::pointer`, com o `on_canvas` que a shell calcula) e o desfazer (`surface_input::undo`).

use crate::{AREA, Mesa, ctrl};
use ph2d_editor_core::ime::ComposeKey;
use ph2d_editor_core::screens::hero::board_keys::{self, BoardKey};
use ph2d_editor_core::screens::hero::{board_view, document_tabs, surface_input};
use ph2d_editor_core::{InteractiveState, NodeId};
use ph2d_host::{Modifiers, PointerButton, PointerKind};

/// Uma tecla premida pelo caminho da shell (`on_keyboard_input`): a composição vê-a primeiro e, se
/// a tomou, o despacho pára; senão segue para o teclado da aba. `true` = alguém a tomou.
fn tecla_da_shell(m: &mut Mesa, k: BoardKey, mods: Modifiers, text: Option<&str>) -> bool {
    // O Ctrl de um acorde é um evento à parte (`Modifier`), antes da letra (`Other`).
    if mods.ctrl {
        let _ = m.hero.handle_compose_key(ComposeKey::Modifier);
    }
    let ck = match k {
        BoardKey::Escape => ComposeKey::Escape,
        _ => ComposeKey::Other,
    };
    if m.hero.handle_compose_key(ck) {
        return true;
    }
    let mut ts = std::mem::replace(&mut m.ts, ph2d_text::TextSystem::without_system_fonts());
    let out = board_keys::key(&mut m.hero, &mut ts, k, mods, true, text, None);
    m.ts = ts;
    out.consumed
}

fn rato(m: &mut Mesa, kind: PointerKind, x: f32, y: f32, on_canvas: bool) -> bool {
    let mut ts = std::mem::replace(&mut m.ts, ph2d_text::TextSystem::without_system_fonts());
    let input = board_view::Input {
        text: &mut ts,
        mods: Modifiers::default(),
        now_ns: 1,
    };
    let taken = board_view::pointer(
        &mut m.hero,
        input,
        kind,
        PointerButton::Primary,
        x,
        y,
        on_canvas,
    );
    m.ts = ts;
    taken
}

fn rato_move(m: &mut Mesa, x: f32, y: f32) {
    let mut ts = std::mem::replace(&mut m.ts, ph2d_text::TextSystem::without_system_fonts());
    let input = board_view::Input {
        text: &mut ts,
        mods: Modifiers::default(),
        now_ns: 2,
    };
    let _ = board_view::pointer_move(&mut m.hero, input, x, y);
    m.ts = ts;
}

/// ⛔ Item 4: o Esc passava pela composição primeiro e ela tomava-o sem tecla morta nenhuma — o
/// `escape` do editor nunca corria (a seleção ficava).
#[test]
fn o_esc_chega_ao_editor_e_a_composicao_so_toma_o_que_e_dela() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", "abc\n")]);
    m.pede(ids[0]);
    assert!(tecla_da_shell(
        &mut m,
        BoardKey::Char('a'),
        ctrl(),
        Some("a")
    ));
    assert!(tecla_da_shell(
        &mut m,
        BoardKey::Escape,
        Modifiers::default(),
        None
    ));
    assert!(tecla_da_shell(
        &mut m,
        BoardKey::Other,
        Modifiers::default(),
        Some("x")
    ));
    assert_eq!(
        m.docs().text_of(ids[0]).as_deref(),
        Some("abc\nx"),
        "o Esc não desfez a seleção (a composição comeu-o)"
    );
    // Controlo: com uma tecla morta pendente, o Esc é da composição (desfá-la) e o texto fica.
    assert!(m.hero.handle_compose_key(ComposeKey::Dead('´')));
    assert_eq!(m.docs().composing(ids[0]).as_deref(), Some("´"));
    assert!(m.hero.handle_compose_key(ComposeKey::Escape));
    assert_eq!(m.docs().composing(ids[0]), None);
    assert!(
        !m.hero.handle_compose_key(ComposeKey::Modifier),
        "sem tecla morta o Shift não é dela"
    );
}

/// ⛔ Item 5: clicar na aba (um `Button`, que ganha o foco) deixava o IME a ir para o botão.
#[test]
fn o_ime_depois_de_clicar_na_aba_chega_ao_documento() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", "")]);
    m.pede(ids[0]);
    let tab = m.hero.documents.surface_active().expect("aba activa");
    m.hero
        .store
        .set_focus(Some(document_tabs::surface_tab_node_id(tab)));
    m.hero.handle_preedit("ç", None);
    assert_eq!(
        m.docs().composing(ids[0]).as_deref(),
        Some("ç"),
        "a pré-edição perdeu-se"
    );
    let arena = bumpalo::Bump::new();
    let _ = m.hero.handle_text_input('ç', &arena);
    assert_eq!(
        m.docs().text_of(ids[0]).as_deref(),
        Some("ç"),
        "o carácter confirmado perdeu-se"
    );
    // Controlo: um CAMPO DE TEXTO focado fica com a composição.
    m.hero.store.register(
        NodeId(77),
        InteractiveState::TextInput {
            state: ph2d_editor_core::widget::TextInputState::Focused,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    m.hero.store.set_focus(Some(NodeId(77)));
    let _ = m.hero.handle_text_input('x', &arena);
    assert_eq!(m.docs().text_of(ids[0]).as_deref(), Some("ç"));
}

/// ⛔ Item 6: os acordes que o editor não usa (Ctrl+K — a paleta, Ctrl+O — abrir) eram tomados e
/// morriam; não tomados, a shell segue para o atalho (o `match` dos atalhos vem depois). Ctrl+Espaço
/// é do editor só no TEXTO (a lista de completação, I4 da Onda 2); num campo é da shell (o modo foco).
#[test]
fn os_acordes_que_o_editor_nao_usa_seguem_para_a_shell() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", "local a = 1\n")]);
    m.pede(ids[0]);
    for (k, t) in [
        (BoardKey::Char('k'), Some("k")),
        (BoardKey::Char('o'), Some("o")),
    ] {
        assert!(
            !tecla_da_shell(&mut m, k, ctrl(), t),
            "Ctrl+{k:?} foi tomado pelo editor"
        );
    }
    assert!(
        tecla_da_shell(&mut m, BoardKey::Space, ctrl(), Some(" ")),
        "Ctrl+Espaço no texto é a lista de completação"
    );
    // Com a barra de busca focada também — e aí Ctrl+Espaço é da shell.
    assert!(tecla_da_shell(
        &mut m,
        BoardKey::Char('f'),
        ctrl(),
        Some("f")
    ));
    assert!(!tecla_da_shell(
        &mut m,
        BoardKey::Char('k'),
        ctrl(),
        Some("k")
    ));
    assert!(
        !tecla_da_shell(&mut m, BoardKey::Space, ctrl(), Some(" ")),
        "Ctrl+Espaço num campo é o modo foco"
    );
    assert_eq!(m.docs().text_of(ids[0]).as_deref(), Some("local a = 1\n"));
    // Controlo: os do editor continuam dele.
    assert!(tecla_da_shell(
        &mut m,
        BoardKey::Escape,
        Modifiers::default(),
        None
    ));
    for c in ['a', 'z', 'd', 's', 'f', 'g', 'h'] {
        assert!(
            tecla_da_shell(&mut m, BoardKey::Char(c), ctrl(), Some("x")),
            "Ctrl+{c}"
        );
        let _ = tecla_da_shell(&mut m, BoardKey::Escape, Modifiers::default(), None);
    }
}

/// ⛔ Item 7: um carregar sobre o chrome desenhado POR CIMA do código (o menu File, um popover, a
/// paleta) era engolido pelo editor. ⭐ Um gesto em curso continua dele até largar.
#[test]
fn um_carregar_sobre_o_chrome_por_cima_do_codigo_nao_e_do_editor() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", "abc\nlinha dois\n")]);
    m.pede(ids[0]);
    m.pinta(ids[0]);
    let antes = m.docs().cursor_of(ids[0]);
    let (x, y) = (AREA.x + AREA.w - 10.0, AREA.y + 40.0);
    assert!(
        !rato(&mut m, PointerKind::Down, x, y, false),
        "o carregar no chrome foi tomado"
    );
    assert!(
        !rato(&mut m, PointerKind::Up, x, y, false),
        "e o largar dele também"
    );
    assert_eq!(m.docs().cursor_of(ids[0]), antes);
    // Controlo: no código é do editor, e o largar fora (sobre o chrome) também — o gesto é dele.
    assert!(rato(&mut m, PointerKind::Down, x, y, true));
    assert!(rato(&mut m, PointerKind::Up, x, y, false));
}

/// ⛔ Item 16: perder a aba activa a meio (a sessão ao vivo chama para um quadro) deixava a
/// pré-edição e o arrasto pendurados — o rato seguinte ainda seleccionava.
#[test]
fn perder_a_aba_activa_larga_a_composicao_e_o_arrasto() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", "abc\nlinha dois\nlinha tres\n")]);
    m.pede(ids[0]);
    m.pinta(ids[0]);
    m.hero.handle_preedit("ã", None);
    let (x, y) = (AREA.x + AREA.w - 10.0, AREA.y + 10.0);
    assert!(rato(&mut m, PointerKind::Down, x, y, true));
    m.hero.documents.activate(None);
    m.quadro();
    m.pede(ids[0]);
    assert_eq!(
        m.docs().composing(ids[0]),
        None,
        "a pré-edição ficou pendurada"
    );
    let antes = m.docs().cursor_of(ids[0]);
    rato_move(&mut m, x, AREA.y + 60.0);
    assert_eq!(
        m.docs().cursor_of(ids[0]),
        antes,
        "o arrasto ficou pendurado"
    );
}

/// ⛔ Item 18: com um campo do Inspector focado e a aba activa, o Ctrl+Z ia ao código.
#[test]
fn com_um_campo_focado_o_desfazer_nao_e_do_codigo() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", "")]);
    m.pede(ids[0]);
    m.escreve(ids[0], "x");
    m.hero.store.register(
        NodeId(77),
        InteractiveState::TextInput {
            state: ph2d_editor_core::widget::TextInputState::Focused,
            text: String::new(),
            caret: 0,
            selection_anchor: None,
        },
    );
    m.hero.store.set_focus(Some(NodeId(77)));
    assert_eq!(
        surface_input::undo(&mut m.hero, false),
        None,
        "o desfazer foi ao código"
    );
    assert_eq!(m.docs().text_of(ids[0]).as_deref(), Some("x"));
    // Controlo: sem campo focado, desfaz o código.
    m.hero.store.set_focus(None);
    assert_eq!(surface_input::undo(&mut m.hero, false), Some(true));
    assert_eq!(m.docs().text_of(ids[0]).as_deref(), Some(""));
}

/// ⛔ Item 17: com a aba de código activa, nem o F9 (o chrome legado) nem o modo do preview do Morph
/// tomam teclas. A shell é um binário (os testes não a importam): a decisão mora na porta do hero
/// (`surface_input::owns_the_keyboard`) e este gate lê os dois sítios da shell que a têm de chamar.
#[test]
fn a_shell_cede_o_f9_e_o_modo_do_morph_a_aba_activa() {
    let (mut m, ids) = Mesa::new(&[("bob.luau", "")]);
    assert!(!surface_input::owns_the_keyboard(Some(&m.hero)));
    m.pede(ids[0]);
    assert!(surface_input::owns_the_keyboard(Some(&m.hero)));
    let modal = include_str!("../../../../shells/desktop/src/input_dispatch/keyboard_modal.rs");
    let teclado = include_str!("../../../../shells/desktop/src/input_dispatch/keyboard.rs");
    assert!(
        modal.contains("fn modal_owns_the_keyboard"),
        "controlo: o ficheiro certo"
    );
    assert!(
        teclado.contains("KeyCode::F9"),
        "controlo: o ficheiro certo"
    );
    let corpo = &modal[modal.find("fn modal_owns_the_keyboard").expect("fn")..];
    assert!(
        corpo.contains("surface_owns_the_keyboard()"),
        "o modo do Morph come as teclas da aba"
    );
    let f9 = teclado.find("KeyCode::F9").expect("F9");
    assert!(
        teclado[f9.saturating_sub(200)..f9 + 200].contains("surface_owns_the_keyboard()"),
        "o F9 alterna o chrome com a aba activa"
    );
}
