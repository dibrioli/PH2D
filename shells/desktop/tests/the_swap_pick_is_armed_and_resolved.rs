//! **Arch-gate: o conta-gotas do *Swap Prefab* ARMA e o clique seguinte RESOLVE.**
//!
//! # Porque é um arch-gate, e não um gate de unidade
//!
//! O motor tem os gates dele — que trocar o prefab de uma cópia religa o elo e descarta o que o
//! novo não conhece. Todos passariam com a **fiação arrancada**: o botão que nunca arma o pick, ou
//! o clique que arma num sítio e resolve noutro. Essa metade vive dentro do laço de frame e do
//! despacho de ponteiro, que exigem janela — nenhum teste de unidade a alcança.
//!
//! # ⛔⛔ Este ficheiro ENCOLHEU com o motor velho (F4.6c, 2026-09-07)
//!
//! Ele chamava-se *«a lista de PEÇAS chega ao painel, e a cor escolhida volta»* e tinha mais dois
//! gates: a shell publicar as linhas de peça, e a cor do picker voltar para a peça. **A lista de
//! peças deixou de ser pintada** — ela era a porta de autoria do motor `VecInstance`, e no modelo
//! geral uma peça é uma ENTIDADE com endereço próprio: o olho é o da Hierarquia e a cor é qualquer
//! ferramenta sobre a peça seleccionada, com a régua em `crate::instance_piece_override_tests`.
//!
//! ⚠️ **O nome do ficheiro mudou junto, e isso não é arrumação:** um arch-gate cujo nome promete
//! uma costura que ele já não mede manda o próximo leitor procurar a lei no sítio errado.

use std::fs;

fn src(name: &str) -> String {
    fs::read_to_string(format!("{}/src/{name}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("{name}: {e}"))
}

/// **O Swap ARMA o pick modal, e o clique seguinte é dele.**
///
/// ⚠️ Sem o arm o botão é um clique que não faz nada; e sem a variante entrar no `PathPick` o
/// clique cairia no picking/gizmo — o artista seleccionaria o alvo em vez de trocar por ele.
///
/// ⚠️ **RE-ANCORADO em 2026-09-07**: quem resolve o segundo clique deixou de ser o
/// `vec_component_pieces::swap_main` (motor vetorial) e passou a ser o
/// `vec_component_general::swap_by_pick`. ⭐ **A lei ficou mais forte com a troca** — o alvo do
/// modelo geral é *uma CÓPIA do prefab que se quer*, porque a receita está escondida do canvas e
/// clicar nela é impossível.
#[test]
fn the_swap_arms_the_modal_pick_and_the_click_resolves_it() {
    let s = src("render_loop/mod.rs");
    assert!(
        s.contains("crate::vec_pick::PathPick::InstanceMain("),
        "o Swap deixou de armar o pick modal: o botão acende e não leva a lado nenhum"
    );
    let d = src("input_dispatch.rs");
    let at = d
        .find("crate::vec_pick::PathPick::InstanceMain(")
        .expect("o clique do pick não resolve o Swap — o pick fica armado para sempre");
    assert!(
        d[at..].contains("crate::vec_component_general::swap_by_pick("),
        "o braço do Swap não chama a porta que troca o prefab"
    );
}

/// **O Esc DESISTE de um pick armado.**
///
/// ⚠️ Enio, 2026-08-04: *"Esc não desativa Swap Main checado"* — e estava certo. O abortar existia
/// só no botão DIREITO, e o roteiro do smoke `=56` que eu escrevi **afirmava o contrário**: um
/// gesto modal cuja única saída é uma tecla que o roteiro não nomeia é um gesto de que o artista
/// não sabe sair. O gate mora aqui porque a cadeia de Escapes é uma ORDEM dentro do laço de
/// teclado, e nenhum teste de unidade a alcança.
#[test]
fn escape_gives_up_an_armed_pick() {
    // ⚠️ A cadeia de Escapes mudou de arquivo em 2026-08-07 (o `keyboard.rs` cruzou o cap de LOC
    // com o Esc do modo de preview, W7r); a PROPRIEDADE afirmada continua exactamente a mesma.
    let s = src("input_dispatch/keyboard_escapes.rs");
    let at = s
        .find("self.vec_path_pick.take().is_some()")
        .expect("o Esc deixou de desistir de um pick armado — o artista fica preso no conta-gotas");
    // ⚠️ Ele TEM de consumir: um Esc que desarma e deixa passar daria blur num widget que o
    // artista não estava a editar, no mesmo toque.
    //
    // ⚠️ **A forma de "consumir" MUDOU com o arquivo:** a cadeia atrás de uma porta devolve
    // `true` em vez de `return;` nu, e a janela de 120 bytes deste gate — que procurava o
    // literal antigo — reprovou produto CORRETO na primeira corrida depois do corte. É a
    // armadilha que este repo já nomeou: *uma âncora em distância de bytes é um proxy que
    // expira*. A pergunta é *ele volta daqui?*, e as duas formas a respondem.
    assert!(
        s[at..at + 120].contains("return"),
        "o Esc desarma o pick e deixa o evento seguir"
    );
    // E vem ANTES do Escape do Pen: com um pick armado o Esc é sobre ele, não sobre um caminho.
    //
    // ⚠️ A âncora do Pen é o `finish()`, e não o `is_drawing()`: o segundo aparece TAMBÉM numa
    // guarda muito acima (um atalho que só corre sem caneta em curso), e ancorar nele fez este
    // gate reprovar código correto na primeira corrida — o `at < pen` comparava com o sítio
    // errado. Um anchor tem de ser único no que ele nomeia.
    let pen = s
        .find("self.vec_pen.finish();")
        .expect("o Escape do Pen mudou de forma — reancore este gate");
    assert!(at < pen, "o Esc do pick tem de preceder o do Pen");
}
