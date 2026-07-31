//! **Arch-gate das três operações de nó da W4** (plano 25 §7) — Join · Reverse · Average.
//!
//! Os motores estão gateados na `ph2d-vec-scene` (`path_join_tests`) e na `ph2d-vec-edit`
//! (`node_ops_tests`); o gate de seam do painel prova que o clique chega ao barramento. O que só
//! um gate de FONTE alcança é a última condição — **a shell CONSUMIR o evento** —, porque ela vive
//! dentro do `render_loop`/`input_dispatch`, que exigem janela.
//!
//! Duas maneiras de partir a wave deixando a suíte inteira verde:
//!
//! 1. **o dreno some** — os três botões acendem, o `PanelEvent::Click` viaja e ninguém o lê;
//! 2. **o `Close Path` volta a virar só o flag** — fechar um laço que o artista acabou de encostar
//!    deixa dois vértices sobrepostos no mesmo ponto, invisível no desenho e presente em todo
//!    Delete/Average/Simplify seguinte.
//!
//! ⚠️ As asserções afirmam uma RELAÇÃO ou um CONTEÚDO dentro de uma janela sintática, nunca uma
//! distância em bytes: esta linha já teve dois arch-gates apodrecerem por medirem bytes.

const LOOP_SRC: &str = include_str!("../src/render_loop/mod.rs");
const DISPATCH: &str = include_str!("../src/input_dispatch.rs");

/// A posição da 1ª ocorrência de `needle` em `src`, ou pânico com a razão.
fn at(src: &str, needle: &str) -> usize {
    src.find(needle).unwrap_or_else(|| {
        panic!(
            "`{needle}` sumiu — se foi renomeado, atualize este gate (e confira que as três \
             operacoes ainda chegam ao artista: `PH2D_BUILD_SMOKE=44`)"
        )
    })
}

/// **Os três botões são drenados pela shell, e cada um chama a SUA porta.** Um `ToolPanelEvent`
/// que ninguém consome é um botão que acende e não faz nada.
#[test]
fn the_three_node_ops_are_drained_by_the_shell() {
    for (id, call, what) in [
        ("VECTOR_PATH_JOIN", "join_selection(", "Join"),
        ("VECTOR_PATH_REVERSE", "reverse_selected_paths(", "Reverse"),
        ("VECTOR_VERT_AVERAGE", "average_selected_verts(", "Average"),
    ] {
        assert!(
            LOOP_SRC.contains(id),
            "o `{id}` nao e' drenado -- o {what} chega ao bus e morre la'"
        );
        assert!(
            LOOP_SRC.contains(call),
            "o dreno do {what} nao chama `{call}` -- o clique e' consumido e nao faz nada"
        );
    }
}

/// **Cada uma abre UM passo de undo, e só se mudou alguma coisa.** Sem o `begin`, desfazer um Join
/// devolve o estado de antes de outro gesto; sem o guard, um clique que não muda nada põe uma
/// linha na fila que o Ctrl+Z não tem o que desfazer.
#[test]
fn each_node_op_opens_exactly_one_undo_step_and_only_when_it_changed_something() {
    let block = at(LOOP_SRC, "// **As três da W4.**");
    let end = at(
        &LOOP_SRC[block..],
        "if let Some(order) = pending_vec_reorder",
    ) + block;
    let window = &LOOP_SRC[block..end];
    assert!(
        window.contains("self.vec_history.begin("),
        "nenhuma das tres abre passo de undo -- o Ctrl+Z saltaria por cima delas"
    );
    assert!(
        window.contains("if changed {") && window.contains("commit_if_changed("),
        "o commit nao e' gateado no resultado -- um clique inerte poria um passo vazio na fila"
    );
    // As três correm no MESMO bloco: uma quarta operação entra na tabela e nasce com undo.
    for call in [
        "join_selection(",
        "reverse_selected_paths(",
        "average_selected_verts(",
    ] {
        assert!(
            window.contains(call),
            "o `{call}` saiu do bloco que da' undo -- ele passou a mudar o documento sem passo"
        );
    }
}

/// **Fechar passa pela porta que SOLDA.** É a metade que o `Close Path` não tinha: ele virava o
/// flag e deixava as duas pontas coincidentes como dois vértices distintos.
#[test]
fn the_close_button_goes_through_the_welding_door() {
    let f = at(DISPATCH, "pub(crate) fn apply_vec_toggle_closed(");
    let end = at(&DISPATCH[f..], "history.push_undo(pre);") + f;
    let window = &DISPATCH[f..end];
    assert!(
        window.contains("scene.close_path("),
        "o toggle nao chama a porta que solda -- fechar um laco encostado deixa dois vertices \
         sobrepostos, invisiveis ate' o proximo Delete"
    );
    assert!(
        window.contains("set_path_closed(sel, false)"),
        "ABRIR deixou de ser so' o flag -- nao ha' nada a soldar ao abrir, e um `close_path(false)` \
         seria uma porta que nao existe"
    );
    assert!(
        window.contains("pen.select("),
        "o toggle nao larga a selecao de no' -- a costura mudou de sitio (e num fecho soldado um \
         vertice inteiro sumiu), entao todo indice plano guardado descreve outro no'"
    );
}

/// **O gesto da TESOURA corre no canvas, antes das ferramentas de quina, e não cai adiante.**
///
/// A ordem é load-bearing: sem o `return`, o mesmo press seguiria para o roteador do pen/shape e a
/// tesourada viria acompanhada de um segundo gesto que o artista não pediu.
#[test]
fn the_scissors_press_cuts_and_stops_there() {
    let block = at(DISPATCH, "// **Modo Tesoura** (W4)");
    let corner = at(DISPATCH, "if self.vec_draw_config.mode.is_corner_tool() {");
    assert!(
        block < corner,
        "o ramo da tesoura corre DEPOIS das ferramentas de quina -- a ordem dos modos exclusivos \
         nesta cadeia e' o que decide quem ve^ o press"
    );
    let window = &DISPATCH[block..corner];
    assert!(
        window.contains("DrawMode::Scissors"),
        "o ramo nao e' gateado no modo Tesoura -- ele cortaria em todo modo"
    );
    assert!(
        window.contains("scissors_cut("),
        "o ramo nao chama a porta que corta"
    );
    assert!(
        window.contains("self.vec_history.begin(") && window.contains("commit_if_changed("),
        "a tesourada nao abre UM passo de undo -- o Ctrl+Z saltaria por cima dela"
    );
    assert!(
        window.contains("return;"),
        "o press da tesoura cai adiante -- o pen/shape veria o mesmo clique"
    );
}

/// **A FACA arma no press e CORTA no release** — e a lâmina segue o dedo pelo meio.
///
/// Um gesto de três tempos tem três sítios onde morre em silêncio, e cada um deixa a suíte inteira
/// verde: sem o arm nada acontece, sem o `k.1 = …` a lâmina fica com comprimento zero e nunca
/// atravessa nada, sem o corte no release o artista desenha um traço e larga-o no vazio.
#[test]
fn the_knife_arms_on_press_tracks_on_move_and_cuts_on_release() {
    let arm = at(DISPATCH, "// **Modo Faca** (W4)");
    let scissors = at(DISPATCH, "// **Modo Tesoura** (W4)");
    assert!(
        arm < scissors,
        "os dois modos de corte trocaram de ordem na cadeia -- quem ve^ o press muda"
    );
    let arm_window = &DISPATCH[arm..scissors];
    assert!(
        arm_window.contains("DrawMode::Knife") && arm_window.contains("self.vec_knife = Some("),
        "o press nao ARMA a lamina"
    );
    assert!(
        !arm_window.contains("knife_cut("),
        "o press CORTA -- uma faca e' um traco, e um traco nao existe ate' se soltar"
    );

    // O move: a lâmina segue o dedo.
    let track = at(DISPATCH, "if let Some(k) = self.vec_knife.as_mut() {");
    assert!(
        DISPATCH[track..track + 200].contains("k.1 = self.last_pointer"),
        "a lamina nao segue o dedo -- ela ficaria com comprimento zero"
    );

    // O release: corta, com UM passo de undo.
    let rel = at(
        DISPATCH,
        "if let Some((start, cur)) = self.vec_knife.take()",
    );
    let end = at(&DISPATCH[rel..], "return;") + rel;
    let window = &DISPATCH[rel..end];
    for (needle, why) in [
        ("knife_cut(", "o release nao corta"),
        ("self.vec_history.begin(", "sem passo de undo"),
        ("commit_if_changed(", "sem commit do passo"),
        ("vec_world_at(", "a lamina nao e' convertida para MUNDO"),
    ] {
        assert!(window.contains(needle), "{why} -- falta `{needle}`");
    }
}
