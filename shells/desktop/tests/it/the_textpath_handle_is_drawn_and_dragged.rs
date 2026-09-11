//! **Arch-gate: a alça do texto em caminho é DESENHADA e ARRASTADA pela `render_loop`.**
//!
//! A alça é uma bolinha pintada no shell. O gate de unidade (`vec_text_ride_tests`) prova o
//! MOTOR — onde a alça está, como um arrasto a move — mas não prova que o shell a **desenha** nem
//! que costura o press/drag no fluxo do ponteiro. Mutar o desenho ou esquecer a costura deixa
//! toda a workspace verde, porque nenhum teste de unidade alcança a `render_loop` (ela precisa de
//! janela e GPU).
//!
//! É o mesmo recurso que a `line/Painter` usou para o anel do pincel
//! (`the_brush_ring_wears_the_live_dab_rotor`) e que esta linha usou para o vínculo
//! (`every_text_on_path_id_is_consumed_by_the_render_loop`): quando o único consumidor é o
//! desenho do shell, a prova é sobre o FONTE.
//!
//! ⚠️ **Controle positivo:** o gate exige encontrar CADA costura, e nomeia qual faltou — um gate
//! que deixa de encontrar por uma mudança de forma passa a guardar nada, e um gate que não vê
//! nada passa sempre.

use std::fs;

fn shell(path: &str) -> String {
    fs::read_to_string(format!("{}/src/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("ler {path}: {e}"))
}

/// O DESENHO da alça está costurado no overlay, e gateado pela política de modo (Select-only).
#[test]
fn the_render_loop_draws_the_handle_gated_on_select_mode() {
    let rl = shell("render_loop/mod.rs");
    // Desenha pelo renderer dedicado…
    assert!(
        rl.contains("draw_text_handle"),
        "a `render_loop` não chama `draw_text_handle` — a alça existe no motor e não na tela"
    );
    // …atrás do flag de modo do `VecOverlayPlan` (Select-only, testado em `vec_overlay`)…
    assert!(
        rl.contains("overlay.textpath_handle"),
        "o desenho da alça não é gateado por `overlay.textpath_handle` — apareceria fora do Select"
    );
    // …e o ponto vem da porta única, não de uma re-derivação que divergiria do hit-test.
    assert!(
        rl.contains("vec_text_ride::handle::world"),
        "o desenho não lê o ponto da alça de `handle::world` — uma 2ª derivação divergiria do \
         hit-test, e a alça seria pintada num sítio e agarrada noutro"
    );
    // ⚠️ **E FORA do bloco `if overlay.edit` — o bug do 1º smoke do Select.** Aquele bloco é
    // FALSO no modo Select (as âncoras não aparecem lá, ADR-0112), então um desenho lá dentro
    // NUNCA roda no único modo em que a alça existe. A prova: o `draw_text_handle` vem DEPOIS do
    // `draw_connector_handles` — que é a região comprovadamente fora do `overlay.edit` (as alças
    // do conector são do Select pela mesma razão). Se alguém o puser de volta no bloco de edição,
    // ele passa a preceder o do conector e este gate fica VERMELHO.
    let draw_at = rl.find("draw_text_handle").expect("desenho da alça");
    let conn_at = rl
        .find("draw_connector_handles")
        .expect("desenho das alças do conector");
    assert!(
        draw_at > conn_at,
        "o `draw_text_handle` está ANTES do `draw_connector_handles` — provavelmente dentro do \
         bloco `if overlay.edit`, onde nunca desenha no modo Select (o bug do smoke)"
    );
}

/// O GESTO da alça está costurado no ponteiro: press (arma), move (arrasta), release (limpa).
///
/// A alça é do modo **Select** (Enio, smoke — no Node se confundia com as âncoras), então o
/// press é a porta `vec_textpath_handle_down`, irmã do `conn_handle_down`, e vem ANTES do
/// picking/gizmo. O move e o release seguem a mesma família do conector.
#[test]
fn the_render_loop_wires_the_handle_gesture() {
    let disp = shell("input_dispatch.rs");
    for (needle, what) in [
        (
            "vec_textpath_handle_down",
            "o PRESS (armar) não está no dispatch do ponteiro (modo Select)",
        ),
        (
            "vec_textpath_handle_move",
            "o MOVE (arrastar) não está no dispatch do ponteiro",
        ),
        (
            "self.vec_textpath_handle_drag = false",
            "o RELEASE não limpa o arrasto — a alça ficaria colada ao cursor após soltar",
        ),
    ] {
        assert!(disp.contains(needle), "{what}");
    }
    // ⚠️ E a ORDEM importa: no Select a tool não captura o canvas, então o press da alça tem de
    // ser hit-testado ANTES do picking/gizmo — senão o clique selecionaria a forma atrás dela em
    // vez de a arrastar (a mesma razão da alça do conector). O `over_canvas_or_gizmo` é o que
    // deixa o clique passar mesmo com o gizmo por cima do texto.
    let handle_at = disp
        .find("self.vec_textpath_handle_down(w)")
        .expect("press da alça (Select)");
    // O picking/gizmo do Select roda no fluxo de sempre, que começa no guard ADR-0112 abaixo
    // (`no modo **Select** a ferramenta não captura o canvas`). O arm da alça TEM de vir antes
    // dele, senão o clique selecionaria a forma atrás em vez de arrastar a alça.
    let generic_dispatch_at = disp
        .find("no modo **Select** a ferramenta não captura")
        .expect("o guard ADR-0112 do dispatch genérico");
    assert!(
        handle_at < generic_dispatch_at,
        "o press da alça (Select) tem de ser hit-testado ANTES do picking/gizmo genérico"
    );
    // E é irmão do press do conector — ambos no mesmo cluster de early-return do Select.
    //
    // ⚠️ Este par era afirmado por uma DISTÂNCIA EM BYTES (`< 1200`), e o Picker de caminho-guia
    // — que é do mesmo cluster, também modal, e tem de preceder as duas alças — entrou entre eles
    // e empurrou a distância para 1810: o gate ficou **VERMELHO sobre código correto**. Uma
    // distância é proxy de *"mesmo cluster"*, e todo bloco legítimo inserido no meio a invalida.
    //
    // A propriedade REAL nunca foi métrica, é POSICIONAL: o que faz das duas alças irmãs é que
    // **as duas correm antes do picking/gizmo genérico** — é isso, e só isso, que impede o clique
    // de selecionar a forma atrás em vez de arrastar a alça. Então cada press ganha a MESMA
    // asserção, e o par deixa de depender de quantos bytes há entre eles.
    //
    // ⚠️ Uma 1ª tentativa afirmou também *"o que está entre os dois é gateado em `DrawMode::Select`"*
    // — e a MUTAÇÃO a matou: o span `[conn, textpath)` termina DENTRO da cadeia de guards do
    // próprio bloco do textpath, então o `DrawMode::Select` dele está sempre no span e a asserção
    // **não podia falhar**. Verde por construção; removida em vez de shipada.
    let conn_at = disp
        .find("self.conn_handle_down(w)")
        .expect("press da alça do conector");
    assert!(
        conn_at < generic_dispatch_at,
        "o press da alça do CONECTOR caiu depois do picking/gizmo genérico — o clique passaria a \
         selecionar a forma atrás em vez de arrastar a alça"
    );
}
