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
