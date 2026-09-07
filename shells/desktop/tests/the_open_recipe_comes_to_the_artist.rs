//! ⛔⛔ **A RECEITA VEM AO ARTISTA, e as duas metades vivem em pontas opostas do quadro** (Enio,
//! 2026-09-07: *«o canvas busca a posição inicial do prefab. não deve ser assim. O prefab deve
//! aparecer na posição central do canvas onde o canvas está»*).
//!
//! Quem sabe que uma receita ABRIU é o carimbo do `master_editing`, que corre antes do extract;
//! quem sabe ONDE ela está é a caixa do gizmo, publicada depois do `snapshots`. ⚠️ Cada metade tem
//! gate próprio e verde — e o que **nenhum dos dois vê** é o fio entre elas: apagar o `run` do
//! quadro deixa a suíte inteira verde e a receita a abrir fora do ecrã, que é o primeiro report.
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

/// ⭐⭐⭐ **O quadro ARMA o pedido a partir da abertura, e SERVE-o com o mundo e o ledger.**
///
/// **Mutação que deve sangrar:** apagar qualquer uma das duas linhas — sem a primeira o pedido
/// nunca nasce; sem a segunda ele nunca é servido, e nos dois casos a receita abre onde estava.
#[test]
fn the_frame_arms_the_request_and_serves_it_on_the_stage() {
    let body = code_of("render_loop/mod.rs");
    assert!(
        body.contains("master_editing::mark(") && body.contains(".opened"),
        "o quadro deixou de ler QUEM ABRIU do carimbo — o pedido de palco nunca nasce"
    );
    assert!(
        body.contains("self.prefab_stage_pending = Some("),
        "a abertura ja' nao arma o pedido (`App::prefab_stage_pending`)"
    );
    let at = body
        .find("prefab_stage::run(")
        .expect("o quadro nao serve o pedido — a receita abre fora do ecra'");
    let arm = &body[at..(at + 500).min(body.len())];
    assert!(
        arm.contains("sim") && arm.contains("preview_drive"),
        "o servico do pedido nao recebe o mundo e o ledger — ou ele nao move nada, ou o \
         movimento vira um passo de undo:\n{arm}"
    );
}

/// ⛔⛔⛔ **O PALCO NÃO TOCA NA CÂMERA** — é o report inteiro.
///
/// A primeira versão movia a vista até à receita: cumpria a letra (*«o prefab no centro»*) e
/// falhava o pedido, porque o artista estava a olhar para outro sítio. ⚠️ Uma mutação que
/// devolvesse a câmera a este módulo passaria por todos os gates de unidade dele — a lei pura
/// devolve um deslocamento e nem recebe a câmera mutável.
#[test]
fn the_stage_never_moves_the_view() {
    let body = code_of("prefab_stage.rs");
    for proibido in ["camera.center", "height_world", "zoom"] {
        assert!(
            !body.contains(proibido),
            "o palco voltou a mexer na vista (`{proibido}`) — o artista estava a olhar para \
             outro sitio, e e' isso que o report diz"
        );
    }
}

/// ⭐⭐ **E a área é a VISÍVEL, com porta única.**
///
/// O canvas é *full-bleed* e os painéis flutuam por cima: centrar na janela põe a receita debaixo
/// de uma coluna docada. ⚠️ A pergunta tem UMA porta (`canvas_area::visible`) desde que o segundo
/// cliente apareceu — este censo é o que impede a terceira cópia de nascer com o `last_content` à
/// mão.
#[test]
fn the_visible_area_has_one_door_and_both_clients_use_it() {
    let stage = code_of("prefab_stage.rs");
    assert!(
        stage.contains("canvas_area::visible("),
        "o palco deixou de perguntar a` porta da area visivel"
    );
    let field3d = code_of("field3d_layout.rs");
    assert!(
        field3d.contains("canvas_area::visible("),
        "o modulo 3D voltou a ter a propria copia da area visivel"
    );
    // E a porta é uma só: ninguém mais lê o `last_content` cru.
    for rel in ["prefab_stage.rs", "field3d_layout.rs"] {
        assert!(
            !code_of(rel).contains("last_content"),
            "{rel} le o `last_content` cru — e' a segunda resposta a` mesma pergunta"
        );
    }
}
