//! O arranque varre as sessões mortas e a janela oferece cada cópia (P12 S, `ph2d_recovery`).

use ph2d_recovery::{Choice, Session};

#[test]
fn o_arranque_oferece_a_copia_da_sessao_morta_e_restore_lembra_o_caminho() {
    let root = std::env::temp_dir().join(format!("ph2d-shell-recovery-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let (name, path) = ("Hero.ph2dproj", "/x/Hero.ph2dproj");
    let meta = ph2d_recovery::meta_of(Some(path));
    // `u32::MAX` nunca é um pid vivo: é a sessão que morreu.
    let morta = Session::start(&root, 1, u32::MAX).expect("sessão morta");
    morta.write_copy(b"bytes do save", &meta).expect("cópia");
    let (mut app, mut seen) = (crate::App::new(), Vec::new());
    app.recovery_start(Some(&root), |o| {
        seen.push((o.meta.name.clone(), o.size));
        Choice::Later
    });
    assert_eq!(seen, [(name.to_owned(), 13)]);
    assert!(!app.recovery.prompt.is_open() && app.project_path.is_none());
    app.recovery.close();
    // `Later` deixou-a; `Restore` lembra o caminho original e a cópia passa à sessão viva.
    let mut app = crate::App::new();
    app.recovery_start(Some(&root), |_| Choice::Restore);
    assert_eq!(app.project_path.as_deref(), Some(path));
    let copy = app
        .recovery
        .session
        .as_ref()
        .expect("sessão viva")
        .copy_path();
    assert_eq!(
        std::fs::read(copy).expect("cópia adoptada"),
        b"bytes do save"
    );
    app.recovery.close();
    assert_eq!(
        std::fs::read_dir(&root).expect("raiz").count(),
        0,
        "o fecho normal limpa"
    );
}
