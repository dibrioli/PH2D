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
    // ⚠️ A lei mudou-se para a moldura 3D partilhada (`ph2d-viewport3d`), que e' consumida
    // pelos DOIS modulos 3D — e o `code_of` desta crate ja' nao lhe chega.
    let field3d = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/ph2d-viewport3d/src/layout.rs"),
    )
    .expect("a moldura 3D partilhada");
    assert!(
        field3d.contains("canvas_area::visible("),
        "o modulo 3D voltou a ter a propria copia da area visivel"
    );
    // E a porta é uma só: ninguém mais lê o `last_content` cru.
    for rel in ["prefab_stage.rs", "chrome_hit.rs"] {
        assert!(
            !code_of(rel).contains("last_content"),
            "{rel} le o `last_content` cru — e' a segunda resposta a` mesma pergunta"
        );
    }
}

/// ⭐⭐⭐ **A SESSÃO SÓ ACABA PELAS DUAS PORTAS** (Enio, 2026-09-07: *«só permita sair da edição
/// apertando Done ou a tecla Enter»*).
///
/// A trava é armada na abertura e servida no fim do quadro. ⚠️ Se o serviço deixar de ser chamado,
/// **nada na suíte fica vermelho**: a trava fecha-se, a barra continua a pintar-se, e o artista fica
/// preso num modo sem saída — que é o pior resultado possível desta feature.
#[test]
fn the_session_only_ends_through_the_two_doors() {
    let frame = code_of("render_loop/mod.rs");
    let at = frame
        .find("self.prefab_editing =")
        .expect("a abertura ja' nao fecha a trava — clicar no vazio volta a fechar a sessao");
    let arm = &frame[at..(at + 200).min(frame.len())];
    // ⛔⛔ **E ela guarda a identidade DURÁVEL.** Um `Ctrl+Z` dentro da sessão respawna tudo com
    // bits novos: uma trava em bits aponta para uma entidade morta e **expulsa o artista da
    // sessão**. Foi assim que a 1.ª versão saiu, e é a lei escrita do módulo do editor.
    assert!(
        arm.contains("StableId"),
        "a trava voltou a guardar BITS — um `Ctrl+Z` dentro da sessao expulsa o artista:\n{arm}"
    );
    let main = code_of("main.rs");
    let serve = main
        .find("self.serve_prefab_exit();")
        .expect("o pedido de saida nunca e' servido — o artista fica preso no modo");
    let undo = main
        .find("self.post_frame_undo();")
        .expect("o passo por diff do quadro desapareceu");
    assert!(
        serve < undo,
        "a saida e' servida DEPOIS do passo por diff — um cancelamento deixaria de ser \
         desfazivel, e seria a unica accao irreversivel do app"
    );
}

/// ⭐⭐⭐ **O `Cancel` repõe o DOCUMENTO, e as duas saídas largam a trava e a selecção.**
///
/// ⚠️ Sem a segunda metade o carimbo do quadro seguinte **reabria a sessão a partir da selecção** —
/// a trava era solta e o modo voltava, o que se lê como *«o botão não funciona»*.
#[test]
fn cancelling_restores_the_document_and_both_exits_let_go() {
    let body = code_of("prefab_stage.rs");
    let at = body
        .find("fn serve_prefab_exit")
        .expect("o servico da saida desapareceu");
    let arm = &body[at..];
    assert!(
        arm.contains("apply_project("),
        "o `Cancel` deixou de repor o documento — ele passaria a sair GUARDANDO o que o artista \
         mandou deitar fora"
    );
    assert!(
        arm.contains("self.prefab_editing = None;") && arm.contains("replace_selection(None)"),
        "uma das saidas nao larga a trava ou a seleccao — a sessao reabre no quadro seguinte"
    );
}

/// ⭐⭐ **As teclas são a MESMA porta dos botões, e devolvem o teclado a quem escreve.**
///
/// ⛔⛔ **A guarda do campo de texto não é cortesia:** a sessão dura minutos, então renomear uma
/// peça dentro da receita e carregar `Enter` para confirmar o nome **fecharia a sessão**, e o `Esc`
/// que desiste do nome **cancelaria tudo o que foi feito**.
#[test]
fn the_keys_are_the_same_door_and_yield_to_a_text_field() {
    let keys = code_of("input_dispatch/keyboard_escapes.rs");
    assert!(
        keys.contains("request_prefab_exit("),
        "as teclas deixaram de passar pela porta dos botoes — dois caminhos para o mesmo fim"
    );
    assert!(
        keys.contains("Exit::Cancel") && keys.contains("Exit::Done"),
        "uma das duas teclas perdeu o fim dela"
    );
    let stage = code_of("prefab_stage.rs");
    let at = stage
        .find("fn request_prefab_exit")
        .expect("a porta do pedido desapareceu");
    let arm = &stage[at..(at + 800).min(stage.len())];
    assert!(
        arm.contains("text_entry_focused()"),
        "a porta deixou de devolver o teclado a quem esta' a escrever:\n{arm}"
    );
}
