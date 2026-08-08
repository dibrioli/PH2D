//! **A fiação do MODO DE PREVIEW** (plano UI/UX W7r) — arch-gate sobre as três costuras que
//! nenhum teste de unidade alcança.
//!
//! # Por que este arquivo existe
//!
//! O modelo do preview ([`render_loop::ui_preview`]) é headless e tem gates próprios. O que ele
//! **não** pode ver é se alguém o LIGA no produto: o gesto modal vive dentro do
//! `on_mouse_input`/`on_cursor_moved` (que precisam de janela), a supressão de undo é uma
//! expressão no `render_loop`, e a saída por Esc é um braço numa cadeia de Escapes cuja ORDEM é
//! o que decide quem consome.
//!
//! ⚠️ **É a lição que a `line/anim` mediu em 2026-07-31:** com o `draw` cravado em `true` os vinte
//! gates de unidade do overlay ficaram VERDES e só o arch-gate sangrou. Aqui é igual — os seis
//! gates do modelo passam com o modo inalcançável.
//!
//! ⚠️ **E cada asserção é sobre uma PROPRIEDADE, nunca sobre distância em bytes:** um proxy de
//! janela expira na primeira wave que mete uma linha no meio, que é como dois gates desta linha
//! já morreram (o `the_dispatch_is_handed_the_live_geometry` e o
//! `the_render_loop_wires_the_handle_gesture`, 2026-07-23).

use std::fs;

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn at(src: &str, needle: &str, what: &str) -> usize {
    src.find(needle)
        .unwrap_or_else(|| panic!("{what} nao existe no fonte (procurei `{needle}`)"))
}

/// O corpo da função `fn_sig`, do cabeçalho até a assinatura seguinte.
///
/// ⚠️ **Ela existe porque um `find` no arquivo INTEIRO é um anchor ambíguo, e isso foi MEDIDO:**
/// a agulha `if self.ui_preview.is_on()` casa PRIMEIRO no handler de movimento, ~480 linhas antes
/// do handler de botão, então a mutação *"a guarda modal deixa de consumir o clique"* passava — o
/// intervalo examinado varria meia dúzia de outros `return`. *Um gate que procura no arquivo todo
/// afirma sobre o arquivo todo.*
fn body_of<'a>(src: &'a str, fn_sig: &str, what: &str) -> &'a str {
    let start = at(src, fn_sig, what);
    let rest = &src[start + fn_sig.len()..];
    let end = rest
        .find("\n    pub(crate) fn ")
        .or_else(|| rest.find("\n    fn "))
        .unwrap_or(rest.len());
    &rest[..end]
}

/// **O gesto do preview PRECEDE toda ferramenta.**
///
/// ⚠️ A âncora de comparação é o **primeiro consumo de ferramenta** do handler (o pen-up do Flip),
/// e não um offset: enquanto a preview corre não existe pincel, traço, seleção, gizmo nem caneta —
/// o clique é da interface que o artista desenhou. É a doutrina dos picks armados, uma família
/// adiante (aqueles são modais sobre o Vector, este é modal sobre o editor inteiro).
#[test]
fn the_preview_consumes_the_click_before_any_tool() {
    let src = read("src/input_dispatch.rs");
    let handler = body_of(
        &src,
        "pub(crate) fn on_mouse_input",
        "o handler de botao do ponteiro",
    );
    let guard = at(
        handler,
        "if self.ui_preview.is_on()",
        "a guarda modal da preview no handler de botao",
    );
    let first_tool = at(
        handler,
        "self.flip_canvas_up()",
        "o primeiro consumo de ferramenta do handler",
    );
    assert!(
        guard < first_tool,
        "a guarda da preview corre DEPOIS de uma ferramenta consumir o clique — dentro do modo o \
         artista pintaria na cena em vez de a operar"
    );
    // ⚠️ E ela CONSOME: sem o `return` o clique cai adiante e abre um arrasto de edição por baixo
    // do modo. A janela é o vão entre a guarda e a primeira ferramenta, que contém só este `if`.
    let body = &handler[guard..first_tool];
    assert!(
        body.contains("self.ui_preview_point(evt.x, evt.y, kind == PointerKind::Down)"),
        "a guarda da preview nao entrega os fatos do rato — o clique nao dirige papel nenhum"
    );
    assert!(
        body.contains("return;"),
        "a guarda da preview nao CONSOME o clique — ele segue para as ferramentas por baixo do \
         modo, e um Down primario abre um arrasto de edicao dentro da apresentacao"
    );
    // ⚠️ **E O PREDICADO É `over_canvas_or_gizmo` — esta metade FALTAVA, e o bug passou por ela.**
    //
    // Reportado pelo Enio (2026-08-07): *"em preview permite que tanto o pai como o filho fossem
    // selecionados"*. A v1 usava `on_canvas`, que exige o `hit_index` **VAZIO** — e o gizmo
    // registra as alças NELE. Entrar na preview **exige o hospedeiro selecionado**, logo o gizmo
    // está sempre lá, logo a guarda **nunca disparava**. As três asserções acima ficaram VERDES
    // sobre uma guarda estruturalmente inalcançável: elas provavam que ela existe, consome e
    // precede as ferramentas, e nenhuma perguntava *ela chega a correr?*.
    assert!(
        body.contains("self.over_canvas_or_gizmo(evt.x, evt.y)"),
        "a guarda da preview usa outro predicado que nao o `over_canvas_or_gizmo` — se ele exigir \
         o `hit_index` vazio (como o `on_canvas`), o gizmo do hospedeiro SELECIONADO a torna \
         inalcancavel e o clique volta a selecionar o filho"
    );
}

/// **O gizmo NÃO é publicado durante a preview.**
///
/// ⚠️ Duas razões, e as duas já estavam escritas no repo: a caixa é derivada da pose **AUTORADA**,
/// então enquanto a máquina move a forma ela fica para trás e passa a descrever um lugar que a
/// forma já não ocupa (é por isso que o ADR-0128 recusou cinco vezes um gizmo sobre geometria que
/// se move); e as alças dela **registram hit-rects**, que é o mesmo motivo pelo qual o ADR-0112 já
/// a suprime nos modos de nó.
///
/// ⚠️ **Ela e a guarda do clique NÃO são a mesma defesa**, e a distinção importa: esta responde
/// *o que se VÊ na apresentação*, e a outra *de quem é o CLIQUE*. Hoje, com o gizmo fora, o
/// `hit_index` do caso vetorial fica vazio e a guarda seria alcançável mesmo com o predicado
/// errado — então **a camada da guarda é gateada aqui pela FONTE**, e não por comportamento
/// ([[feedback_layered_defenses_need_per_layer_gates]]).
#[test]
fn the_gizmo_is_not_published_while_the_preview_runs() {
    let src = read("src/render_loop/mod.rs");
    let flat: String = src.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        flat.contains("==ph2d_editor::ToolId::new(\"vector\"))||self.vec_draw_config.mode==ph2d_tool_vector::DrawMode::Select)&&!self.ui_preview.is_on()"),
        "o `vec_gizmo_on` deixou de excluir a preview — a caixa fica sobre a pose autorada \
         enquanto a forma anima para longe dela, e as alcas dela roubam o clique da apresentacao"
    );
}

/// **O Move alimenta a preview e NÃO consome** — a assimetria é deliberada.
///
/// ⚠️ Um `Down` primário abriria um arrasto de edição e por isso é da preview; um movimento não
/// abre nada, e consumi-lo mataria o pan e o zoom, que o Figma mantém vivos no modo de
/// apresentação dele pela mesma razão: olhar de perto não é editar.
#[test]
fn the_pointer_move_feeds_the_preview_without_consuming_it() {
    let src = read("src/input_dispatch.rs");
    let moved = at(
        &src,
        "pub(crate) fn on_cursor_moved",
        "o handler de movimento do cursor",
    );
    let tail = &src[moved..];
    let hook = at(
        tail,
        "if self.ui_preview.is_on()",
        "o hook da preview no Move",
    );
    // O bloco vai do `if` até ao fim do braço; um `return` dentro dele seria consumo.
    let end = tail[hook..]
        .find("\n        }\n")
        .expect("o fim do bloco da preview no Move");
    let block = &tail[hook..hook + end];
    assert!(
        block.contains("self.ui_preview_point("),
        "o Move nao entrega a posicao a' preview — o hover nunca acende nada"
    );
    assert!(
        !block.contains("return"),
        "o Move da preview CONSOME o evento — o pan e o zoom morrem dentro do modo"
    );
}

/// **A supressão de undo cobre a preview INTEIRA, e não só as máquinas em voo.**
///
/// ⚠️ Uma máquina PARADA num hover deixa o mundo fora da pose autorada, e o diff registraria esse
/// mundo como trabalho do artista. E o quadro da SAÍDA precisa da supressão porque o `leave`
/// escreve poses de volta: um passo de undo ali diria *"você mexeu na cena"* por ele ter olhado.
#[test]
fn the_undo_is_suppressed_for_the_whole_preview_not_just_the_machines() {
    let src = read("src/render_loop/mod.rs");
    // ⚠️ Sem espaço em branco dos dois lados: o `rustfmt` decide onde quebra a expressão, e uma
    // âncora que inclui indentação afirma a FORMATAÇÃO em vez do produto (a cicatriz que o
    // `the_draw_pass_publishes_the_facts_it_derived` já pagou duas vezes).
    let flat: String = src.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
        flat.contains("self.ui_state_live=preview_frame|self.ui_preview.is_on()|"),
        "a supressao de undo nao inclui os dois termos da preview — um hover parado, ou o quadro \
         em que se sai dela, vira um passo de undo espurio"
    );
    // A leitura tem de ser ANTES de o toggle correr: depois, `preview_frame` responderia sobre o
    // mundo pós-saída e o quadro da saída ficaria descoberto.
    let read_at = at(
        &flat,
        "letpreview_frame=self.ui_preview.is_on()",
        "a leitura",
    );
    let toggle = at(&flat, "self.ui_preview.leave(", "a saida da preview");
    assert!(
        read_at < toggle,
        "`preview_frame` e' lido DEPOIS de a preview sair — o quadro da saida perde a supressao"
    );
}

/// **Esc sai da preview, e vem antes de todos os outros Escapes.**
///
/// ⚠️ A preview toma o rato do editor inteiro, então com ela ligada o Esc é inequivocamente sobre
/// ela; qualquer outro consumidor deixaria o artista preso num modo cuja única outra saída é um
/// botão que a própria preview pode ter tirado de vista.
#[test]
fn escape_leaves_the_preview_before_any_other_escape() {
    // ⚠️ A cadeia inteira mudou de arquivo quando o `keyboard.rs` cruzou o cap de LOC — o gate
    // segue o CÓDIGO, e é a propriedade (*quem vem antes de quem*) que ele afirma, não o endereço.
    let src = read("src/input_dispatch/keyboard_escapes.rs");
    let ours = at(&src, "self.ui_preview.is_on()", "o Esc da preview");
    let next = at(
        &src,
        "self.joint_draw_cancel_key()",
        "o primeiro Esc de gesto modal",
    );
    assert!(
        ours < next,
        "o Esc da preview corre depois de outro Escape — dentro do modo a tecla faria outra coisa"
    );
    assert!(
        src[ours..next].contains("self.ui_preview_leave = true"),
        "o Esc da preview nao pede a saida — a tecla e' consumida e nada acontece"
    );

    // ⚠️ **E a CADEIA é chamada.** Esta metade é a que a integração da `line/physics` pagou em
    // 2026-07-27: com a cadeia atrás de uma porta, um gate que só olha para DENTRO dela fica
    // verde sobre um teclado que nunca a consulta. E a POSIÇÃO da chamada é a lei que o corte
    // preservou — depois dos atalhos da timeline, antes de o Painter reivindicar as teclas.
    let kb = read("src/input_dispatch/keyboard.rs");
    let call = at(
        &kb,
        "self.escape_key(",
        "a chamada da cadeia de encerramento",
    );
    let timeline = at(&kb, "self.timeline_key(", "os atalhos da timeline");
    let painter = at(
        &kb,
        "self.painter_nudge_brush_size(",
        "os atalhos de pincel do Painter",
    );
    assert!(
        timeline < call && call < painter,
        "a cadeia de encerramento saiu do lugar dela na sequencia — o Esc passa a ser de outro \
         dono, e um gesto em curso deixa de poder ser cancelado"
    );
}

/// **O deslocamento dos estados não se REALIMENTA** (Enio, 2026-08-07).
///
/// ⚠️ O detector compara o `Transform` do hospedeiro com o do quadro anterior — o que faz o
/// arrasto, a seta do teclado, o campo numérico e o align entrarem todos pela mesma porta. Mas um
/// **Show** também move o `Transform`, e sem duas coisas o mesmo detector leria a pose que a
/// MÁQUINA escreveu como um arrasto do artista e deslocaria todos os estados por uma distância
/// que ninguém percorreu:
///
/// 1. **a guarda `!self.ui_state_live`** — nada é aplicado enquanto algo dirige a pose;
/// 2. **o ancoradouro re-escrito em TODO quadro**, aplique-se ou não — sem isso, o primeiro
///    quadro depois de um Show mediria a diferença acumulada e a aplicaria de uma vez.
///
/// É a doença que o `expr_owed` e o `skip` do autokey já pagaram noutros módulos: *uma pose
/// derivada realimentada vira autoria que ninguém fez*.
#[test]
fn the_state_shift_never_feeds_back_on_a_pose_the_machine_wrote() {
    let src = read("src/render_loop/mod.rs");
    let flat: String = src.chars().filter(|c| !c.is_whitespace()).collect();
    let apply = at(
        &flat,
        "shift_host_in_all_states(ui_states,h,d)",
        "o deslocamento dos estados",
    );
    // A guarda vive na condição que precede a aplicação, no mesmo bloco.
    let guard = at(
        &flat,
        "self.ui_states_move_all&&!self.ui_state_live",
        "a guarda anti-realimentacao",
    );
    assert!(
        guard < apply,
        "a guarda `!ui_state_live` nao precede o deslocamento — a pose que a MAQUINA escreve \
         seria lida como um arrasto do artista"
    );
    // ⚠️ **E o ancoradouro é escrito FORA do `if`**: dentro dele, um quadro que não aplica não
    // re-ancora, e a diferença acumulada de um Show inteiro seria despejada de uma vez no quadro
    // seguinte. É a metade que a guarda sozinha não dá.
    let anchor = at(
        &flat,
        "self.ui_states_anchor=Some((h,now));",
        "a re-ancoragem",
    );
    assert!(anchor > apply, "a re-ancoragem nao vem depois da aplicacao");
    // ⚠️ **E ela está FORA do ramo — a chave que fecha o `if` vem imediatamente antes dela.**
    //
    // Esta linha nasceu de uma mutação SOBREVIVENTE: mover a re-ancoragem para DENTRO do ramo que
    // aplica a mantém *depois* da aplicação, então o `anchor > apply` acima continuava VERDE — e o
    // produto ficava com o defeito inteiro (um Show acumula dívida, e o primeiro quadro que
    // aplicar a despeja de uma vez). *Depois* e *fora* não são a mesma pergunta — a mesma
    // distinção que o arch-gate do memo do Painter pagou em 2026-07-22.
    assert!(
        flat.contains("}self.ui_states_anchor=Some((h,now));"),
        "a re-ancoragem esta' DENTRO do ramo que aplica — um quadro que nao aplica deixa de \
         re-ancorar, e a diferenca acumulada de um Show inteiro e' despejada de uma vez"
    );
}
