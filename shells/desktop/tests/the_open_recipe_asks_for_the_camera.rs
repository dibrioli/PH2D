//! ⛔⛔ **A CÂMERA VAI À RECEITA, e as duas metades vivem em pontas opostas do quadro** (Enio,
//! 2026-09-07: *«o prefab não apareceu no centro relativo ao canvas visível»*).
//!
//! Quem sabe que uma receita ABRIU é o carimbo do `master_editing`, que corre antes do extract;
//! quem sabe ONDE ela está é a caixa do gizmo, publicada depois do `snapshots`. ⚠️ Cada metade tem
//! gate próprio e verde — e o que **nenhum dos dois vê** é o fio entre elas: apagar o `apply` do
//! quadro deixa a suíte inteira verde e a receita a abrir fora do ecrã, que é exactamente o report.
//!
//! ⛔ Ele é textual porque o fio vive dentro do laço de quadro da `render_loop`, cuja função tem
//! ~35 argumentos e um `AppGfx` com uma surface de janela real.

use std::path::Path;

/// O corpo do ficheiro sem comentários — senão o censo lê o que o código DIZ sobre si.
fn code_of(rel: &str) -> String {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join(rel);
    let body = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
    body.lines()
        .map(|l| match l.find("//") {
            Some(i) => &l[..i],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// ⭐⭐⭐ **O quadro ARMA o pedido a partir da abertura, e SERVE-o com a câmera.**
///
/// **Mutação que deve sangrar:** apagar qualquer uma das duas linhas — sem a primeira o pedido
/// nunca nasce; sem a segunda ele nunca é servido, e nos dois casos a receita abre onde estava.
#[test]
fn the_frame_arms_the_request_and_serves_it_with_the_camera() {
    let body = code_of("render_loop/mod.rs");
    assert!(
        body.contains("master_editing::mark(") && body.contains(".opened"),
        "o quadro deixou de ler QUEM ABRIU do carimbo — o pedido de camera nunca nasce"
    );
    assert!(
        body.contains("self.prefab_framing = Some("),
        "a abertura ja' nao arma o pedido (`App::prefab_framing`)"
    );
    let at = body
        .find("prefab_framing::apply(")
        .expect("o quadro nao serve o pedido — a receita abre fora do ecra'");
    let arm = &body[at..(at + 400).min(body.len())];
    assert!(
        arm.contains("camera"),
        "o servico do pedido nao recebe a camera — ele nao pode mover vista nenhuma:\n{arm}"
    );
}

/// ⭐⭐ **E a área é a VISÍVEL, com porta única.**
///
/// O canvas é *full-bleed* e os painéis flutuam por cima: centrar na janela põe a receita debaixo
/// de uma coluna docada. ⚠️ A pergunta tem UMA porta (`canvas_area::visible`) desde que o segundo
/// cliente apareceu — este censo é o que impede a terceira cópia de nascer com o `last_content` à
/// mão.
#[test]
fn the_visible_area_has_one_door_and_both_clients_use_it() {
    let framing = code_of("prefab_framing.rs");
    assert!(
        framing.contains("canvas_area::visible("),
        "o enquadramento deixou de perguntar a` porta da area visivel"
    );
    let field3d = code_of("field3d_layout.rs");
    assert!(
        field3d.contains("canvas_area::visible("),
        "o modulo 3D voltou a ter a propria copia da area visivel"
    );
    // E a porta é uma só: ninguém mais lê o `last_content` cru.
    for rel in ["prefab_framing.rs", "field3d_layout.rs"] {
        assert!(
            !code_of(rel).contains("last_content"),
            "{rel} le o `last_content` cru — e' a segunda resposta a` mesma pergunta"
        );
    }
}
