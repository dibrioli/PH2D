//! **Arch-gate da ESCALA DA SELEÇÃO de nós** (plano 25 §6, W3b).
//!
//! Os motores estão gateados na `ph2d-vec-edit` (`selection_scale_tests.rs`); o que só um gate de
//! FONTE alcança é a costura da shell, porque ela vive dentro do `input_dispatch`/`keyboard`, que
//! exigem janela. Três maneiras de partir a wave deixam a suíte inteira verde:
//!
//! 1. **o marquee volta a exigir Shift** — o ramo do vazio no modo Node desaparece, e o artista
//!    perde o retângulo (e o Shift deixa de poder somar, porque foi tomado para o abrir);
//! 2. **as teclas não chegam** — `Tab` e `Ctrl+A` viram no-ops e a forma de 40 nós continua
//!    clique-a-clique, que era a queixa inteira;
//! 3. **o Up ignora o Shift** — o retângulo soma sempre, ou nunca.
//!
//! ⚠️ As asserções afirmam uma RELAÇÃO ou um CONTEÚDO dentro de uma janela sintática, nunca uma
//! distância em bytes: esta linha já teve dois arch-gates apodrecerem por medirem bytes, e um
//! terceiro por ancorar na *primeira ocorrência* de um nome que uma feature irmã passou a usar
//! antes.

const DISPATCH: &str = include_str!("../src/input_dispatch.rs");
const KEYBOARD: &str = include_str!("../src/input_dispatch/keyboard.rs");

/// A posição da 1ª ocorrência de `needle` em `src`, ou pânico com a razão.
fn at(src: &str, needle: &str) -> usize {
    src.find(needle).unwrap_or_else(|| {
        panic!(
            "`{needle}` sumiu — se foi renomeado, atualize este gate (e confira que a escala da \
             selecao ainda chega ao artista: `PH2D_BUILD_SMOKE=43`)"
        )
    })
}

/// **O press no vazio, no modo Node, abre o retângulo — e ANTES do `on_press_node`.**
///
/// A ordem é load-bearing e não é estética: o `on_press_node` DESSELECIONA quando não acerta nada,
/// então um marquee aberto depois dele somaria (com Shift) a uma seleção que acabou de ser apagada.
#[test]
fn a_press_on_empty_canvas_opens_the_marquee_before_the_node_press() {
    let block = at(
        DISPATCH,
        "// **Modo Node: o press no VAZIO abre o retângulo",
    );
    let arm = at(&DISPATCH[block..], "self.vec_marquee = Some(") + block;
    let node_press = at(DISPATCH, "self.vec_pen.on_press_node(");
    assert!(
        block < node_press && arm < node_press,
        "o ramo do marquee corre DEPOIS do `on_press_node` -- ele ja' desselecionou, e o Shift \
         somaria a uma selecao apagada"
    );
    let window = &DISPATCH[block..arm];
    assert!(
        window.contains("DrawMode::Node"),
        "o ramo do marquee nao e' gateado no modo Node -- ele abriria por cima do desenho"
    );
    assert!(
        window.contains("node_edit_hit_at"),
        "o ramo nao pergunta pela porta que ja' existe se o press ACERTA alguma coisa -- uma \
         segunda busca discordaria da que o press usa, e o retangulo abriria sobre um no'"
    );
    assert!(
        !window.contains("shift_key()"),
        "o marquee voltou a exigir Shift -- e' a queixa do plano 25 §6, e toma o modificador que \
         em todo app de desenho significa SOMAR"
    );
}

/// **O Up honra o Shift** (soma) e trata um retângulo de tamanho zero como CLIQUE (desseleciona).
/// Sem a 2ª metade, clicar no vazio deixaria a seleção intacta — e o artista não teria como largar
/// uma forma sem clicar noutra.
#[test]
fn the_marquee_release_adds_with_shift_and_deselects_on_a_bare_click() {
    let take = at(
        DISPATCH,
        "if let Some((start, cur)) = self.vec_marquee.take()",
    );
    let call = at(&DISPATCH[take..], "box_select_with(") + take;
    let window = &DISPATCH[take..call];
    assert!(
        window.contains("shift_key()"),
        "o Up nao le^ o Shift -- o retangulo somaria sempre, ou nunca"
    );
    assert!(
        window.contains("moved"),
        "o Up nao distingue um ARRASTO de um clique -- um clique no vazio viraria um box-select \
         que nao apanha nada e deixa a selecao como estava"
    );
    // E o clique nu desseleciona: a metade que fecha o gesto.
    let tail = &DISPATCH[take..at(&DISPATCH[take..], "return;") + take];
    assert!(
        tail.contains("self.vec_pen.select(None)"),
        "o clique no vazio nao desseleciona"
    );
}

/// **`Tab` e `Ctrl+A` chegam ao `PenTool`, e só no modo Node.** Noutro modo não há nó selecionado
/// a que estas teclas se refiram, e o `Tab` do app tem outros donos.
#[test]
fn tab_and_select_all_reach_the_pen_in_node_mode() {
    let block = at(KEYBOARD, "// **A ESCALA DA SELEÇÃO DE NÓS**");
    let end = at(&KEYBOARD[block..], "// Arrow keys nudge the selection") + block;
    let window = &KEYBOARD[block..end];
    assert!(
        window.contains("DrawMode::Node"),
        "as teclas da escala nao sao gateadas no modo Node"
    );
    for (needle, why) in [
        ("step_vert_selection(", "o `Tab` nao percorre os nos"),
        ("select_all_verts(", "o `Ctrl+A` nao apanha todos os nos"),
    ] {
        assert!(
            window.contains(needle),
            "{why} -- `{needle}` nao esta no bloco"
        );
    }
    assert!(
        window.contains("KeyCode::Tab") && window.contains("KeyCode::KeyA"),
        "uma das duas teclas nao esta ligada"
    );
}

/// **Os dois botões de alcance são drenados pela shell.** Eles chegam ao bus (há gate de seam no
/// painel) e aqui prova-se que alguém os consome — um `ToolPanelEvent` que ninguém drena é um
/// botão que acende e não faz nada.
#[test]
fn the_selection_reach_buttons_are_drained_by_the_shell() {
    const LOOP_SRC: &str = include_str!("../src/render_loop/mod.rs");
    for (id, call) in [
        ("VECTOR_VERT_SEL_SUBPATH", "select_subpath_verts("),
        ("VECTOR_VERT_SEL_SAME", "select_verts_of_same_kind("),
    ] {
        assert!(
            LOOP_SRC.contains(id),
            "o `{id}` nao e' drenado -- o botao chega ao bus e morre la'"
        );
        assert!(
            LOOP_SRC.contains(call),
            "o dreno do `{id}` nao chama `{call}` -- o clique e' consumido e nao faz nada"
        );
    }
}
