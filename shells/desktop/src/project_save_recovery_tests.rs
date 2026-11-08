//! O arranque varre as sessões mortas e a janela em cartões oferece cada cópia (P12 S).

use ph2d_editor_core::{ui_scale::UiScaleMap, zones::Rect};
use ph2d_host::{PointerButton, PointerKind};
use ph2d_recovery::{Quit, Session, card};
use winit::event::{ElementState, MouseButton};
use winit::keyboard::{KeyCode, PhysicalKey};

fn clica(app: &mut crate::App, r: Rect) {
    let (x, y) = (r.x + r.w * 0.5, r.y + r.h * 0.5);
    for k in [PointerKind::Down, PointerKind::Up] {
        let evt = ph2d_pen::ponteiro::do_rato(x, y, k, PointerButton::Primary, 0);
        app.pointer_button(evt, MouseButton::Left);
    }
}

#[test]
fn o_arranque_abre_a_janela_em_cartoes_esc_e_later_restore_pelo_ponteiro() {
    let root = std::env::temp_dir().join(format!("ph2d-shell-recovery-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let path = "/x/Hero.ph2dproj";
    let morta = Session::start(&root, 1, u32::MAX).expect("sessão morta");
    use super::super::tests::{bare_file, empty_state};
    let bytes = postcard::to_allocvec(&(super::PROJECT_SCHEMA, &bare_file(empty_state())));
    morta
        .write_copy(&bytes.expect("bytes"), &ph2d_recovery::meta_of(Some(path)))
        .expect("cópia");
    // O `Esc` é da janela (nenhum atalho o vê) e é `Later`: a cópia fica no disco.
    let mut app = crate::App::new();
    app.recovery_start(Some(&root));
    assert!(app.recovery.prompt.is_open());
    let esc = PhysicalKey::Code(KeyCode::Escape);
    assert!(app.modal_owns_the_keyboard(esc, ElementState::Pressed, false));
    assert!(!app.recovery.is_modal() && app.project_path.is_none());
    app.recovery.close();
    // O clique real no `Restore` do cartão: abre, lembra o caminho; e o `Don't Save` do fecho.
    let mut app = crate::App::new();
    app.recovery_start(Some(&root));
    let (screen, map) = (Rect::new(0.0, 0.0, 1280.0, 800.0), UiScaleMap::default());
    app.recovery.screen = Some((screen, map));
    clica(&mut app, card::layout(screen, 1).rows[0].restore);
    assert_eq!(app.project_path.as_deref(), Some(path));
    assert!(!app.recovery.is_modal(), "um Restore fecha a janela");
    // Sem janela o App não serializa: o «por gravar» prova-se no crate (`abrir_poe_a_linha_de_base…`).
    app.recovery.quit = Quit::Asking("Hero.ph2dproj".into());
    clica(&mut app, card::quit_layout(screen).dont_save);
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
