//! O arranque varre as sessões mortas e a janela em cartões oferece cada cópia (P12 S).

use ph2d_editor_core::{ui_scale::UiScaleMap, zones::Rect};
use ph2d_host::{PointerButton, PointerKind};
use ph2d_recovery::{Quit, Session, card};
use winit::event::{ElementState, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};

/// O clique pelo caminho REAL: o movimento e o botão SEM posição (o `MouseInput` não a traz).
/// ⛔ O clique à mão com a posição no evento escondeu que a janela comia o movimento (08/11).
fn clica(r: Rect, app: &mut crate::App) {
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    let evt = ph2d_pen::ponteiro::do_rato(x, y, PointerKind::Move, PointerButton::Primary, 0);
    app.pointer_moved(evt);
    app.on_mouse_input(ElementState::Pressed, MouseButton::Left);
    app.on_mouse_input(ElementState::Released, MouseButton::Left);
}

#[test]
fn o_arranque_abre_a_janela_em_cartoes_esc_e_later_restore_pelo_ponteiro() {
    let root = std::env::temp_dir().join(format!("ph2d-shell-recovery-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let path = "/x/Hero.ph2dproj";
    use super::super::tests::{bare_file, empty_state};
    let bytes = postcard::to_allocvec(&(super::PROJECT_SCHEMA, &bare_file(empty_state())));
    let (bytes, meta) = (bytes.expect("bytes"), ph2d_recovery::meta_of(Some(path)));
    for pid in [1, 2] {
        let morta = Session::start(&root, u64::from(pid), u32::MAX - pid).expect("sessão morta");
        morta.write_copy(&bytes, &meta).expect("cópia");
    }
    let (screen, map) = (Rect::new(0.0, 0.0, 1280.0, 800.0), UiScaleMap::default());
    let abre = |app: &mut crate::App| {
        app.recovery_start(Some(&root));
        app.recovery.screen = Some((screen, map));
    };
    let mut app = crate::App::new();
    abre(&mut app);
    clica(card::layout(screen, 2).later, &mut app);
    assert!(!app.recovery.is_modal() && app.recovery.project_path.is_none());
    app.recovery.close();
    let mut app = crate::App::new();
    abre(&mut app);
    clica(card::layout(screen, 2).rows[0].discard, &mut app);
    assert_eq!(app.recovery.prompt.entries.len(), 1, "Discard tira a linha");
    clica(card::layout(screen, 1).rows[0].restore, &mut app);
    assert_eq!(app.recovery.project_path.as_deref(), Some(path));
    assert!(!app.recovery.is_modal(), "um Restore fecha a janela");
    // A pergunta do fecho (sem janela o App não serializa: `abrir_poe_a_linha_de_base…` no crate).
    let pergunta = |app: &mut crate::App| assert!(app.recovery.ask_quit(Some(b"novo"), Some(path)));
    pergunta(&mut app);
    clica(card::quit_rects(&app.recovery, screen).cancel, &mut app);
    assert_eq!(app.recovery.quit, Quit::Idle, "Cancel fica");
    pergunta(&mut app);
    let esc = PhysicalKey::Code(KeyCode::Escape);
    assert!(app.modal_owns_the_keyboard(esc, ElementState::Pressed, false));
    assert_eq!(app.recovery.quit, Quit::Idle, "Esc = Cancel");
    pergunta(&mut app);
    clica(card::quit_rects(&app.recovery, screen).dont_save, &mut app);
    assert_eq!(app.recovery.quit, Quit::Go);
    assert!(!app.recovery_quit_guard(), "respondido, fecha");
    app.recovery.close();
}

/// A janela da recuperação é modal também para o hover: mover por baixo dela não chega ao editor.
#[test]
fn o_hover_por_baixo_da_janela_nao_chega_ao_editor() {
    let mut app = crate::App::new();
    app.recovery.quit = Quit::Asking("Hero.ph2dproj".into());
    let antes = app.last_pointer;
    let (x, y) = (antes.0 + 40.0, antes.1 + 30.0);
    let evt = ph2d_pen::ponteiro::do_rato(x, y, PointerKind::Move, PointerButton::Primary, 0);
    app.pointer_moved(evt);
    assert_eq!(app.last_pointer, antes, "o hover passou por baixo");
    app.recovery.quit = Quit::Idle;
    app.pointer_moved(evt);
    assert_eq!(app.last_pointer, (x, y));
}
