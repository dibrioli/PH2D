//! **Arch-gate: as teclas do MODO** (spec/06 §3.2; dono, 03/10 e 05/10) — `Tab` = Object ↔ o modo,
//! `Ctrl+Tab` = a lista (as duas por `mode_drive::mode_key`, com gate na fundação) e `Ctrl+Space` =
//! o zen. O despacho do teclado só se alcança com a `App`; o que se afirma é o FONTE do território
//! do input inteiro.
//!
//! ⛔ O defeito que ele apanhou (05/10): no modo Node do vetor um braço `KeyCode::Tab` percorria os
//! nós e CONSUMIA a tecla — em Edit com o Node, o `Tab` não saía do modo. ⇒ o `Tab` tem UM dono.

/// ⚠️ A ÚNICA excepção é a tradução do QUADRO (`keyboard_board.rs`, MiroClone): com uma aba de quadro
/// activa a cena está escondida e o teclado é do quadro (o `Tab` lá é a nota seguinte, como no Miro) —
/// ou da aba de família (o editor de scripts); sem nenhum dos dois a porta devolve `false` antes.
///
/// Mutação que tem de sangrar: um segundo braço `KeyCode::Tab` em qualquer outro ficheiro do input;
/// o quadro tomar o teclado sem quadro activo; ou o braço do modo deixar de chamar o `mode_key`.
#[test]
fn only_the_mode_owns_the_tab() {
    const BOARD: &str = "input_dispatch/keyboard_board.rs";
    let all = crate::input_text::territory();
    let mut arms: Vec<(&str, usize)> = Vec::new();
    let mut board = "";
    for chunk in all.split("// [[ficheiro ").skip(1) {
        let (name, body) = chunk.split_once("]]\n").expect("o marcador do ficheiro");
        if name == BOARD {
            board = body;
        }
        let n = body.matches("KeyCode::Tab").count();
        if n > 0 {
            arms.push((name, n));
        }
    }
    let scene: Vec<_> = arms.iter().filter(|(f, _)| *f != BOARD).collect();
    assert!(
        scene.len() == 1 && scene[0].1 == 1,
        "o `Tab` tem mais de um dono no input (ou nenhum): {arms:?}"
    );
    assert!(
        board.contains("active().is_none() && hero.documents.surface_active().is_none() {\n"),
        "o quadro toma o teclado (e o `Tab`) sem um quadro activo"
    );
    let handlers = crate::input_text::handlers();
    let arm = handlers
        .find("KeyCode::Tab =>")
        .expect("o braco do `Tab` nao esta no input_handlers");
    let body = &handlers[arm..arm + 240];
    assert!(
        body.contains("mode_drive::mode_key(hero, cmd_chord)"),
        "o `Tab` nao chega ao modo (o `Ctrl+Tab` e a lista pelo mesmo `cmd_chord`)"
    );
}

/// O território acima é a shell; a família trata as suas teclas ANTES dela. O Pixel prendia o `Tab`
/// em Draw, invisível ao censo (dono, 06/11). Mutação: um braço `KeyCode::Tab` em qualquer crate.
#[test]
fn no_crate_takes_the_tab_before_the_mode() {
    fn varre(dir: &std::path::Path, achados: &mut Vec<String>) {
        for e in std::fs::read_dir(dir).expect("a pasta").flatten() {
            let p = e.path();
            if p.is_dir() {
                varre(&p, achados);
            } else if p.extension().is_some_and(|x| x == "rs")
                && std::fs::read_to_string(&p).is_ok_and(|s| s.contains("KeyCode::Tab"))
            {
                achados.push(p.display().to_string());
            }
        }
    }
    let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates");
    let mut achados = Vec::new();
    for c in std::fs::read_dir(crates).expect("crates/").flatten() {
        let src = c.path().join("src");
        if src.is_dir() {
            varre(&src, &mut achados);
        }
    }
    assert!(achados.is_empty(), "o `Tab` tem outro dono: {achados:?}");
}

/// Mutação que tem de sangrar: o `Ctrl+Space` deixar de alternar o zen.
#[test]
fn ctrl_space_toggles_the_zen() {
    let handlers = crate::input_text::handlers();
    let arm = handlers
        .find("KeyCode::Space if cmd_chord =>")
        .expect("o braco do `Ctrl+Space` sumiu");
    // ⚠️ Desde 08/10 (F5) a porta é UMA para o `Ctrl+Space`, *Ver ▸ Modo foco* e o toque de 4
    // dedos: o braço chama-a, e ela alterna o zen. Desde F6 ela vive na crate (`ZenMode`).
    assert!(
        handlers[arm..arm + 160].contains("gfx.zen.toggle_announced(&mut gfx.toasts)"),
        "o `Ctrl+Space` nao chega a porta do modo foco"
    );
    assert!(
        include_str!("../../../../crates/ph2d-editor-core/src/zen.rs")
            .contains("if !self.try_toggle()"),
        "a porta do modo foco nao alterna o zen"
    );
}
