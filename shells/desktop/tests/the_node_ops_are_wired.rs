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
/// A PONTE do vetor, INTEIRA — o pai mais os irmãos que o teto de 600 LOC (HR-18) obriga a
/// nascer. As duas publicações abaixo já viveram no pai e hoje moram no `_publish`.
///
/// ⚠️ **Ler UM arquivo aqui seria ler um ENDEREÇO** — quando o dono se muda, o gate falha por um
/// motivo que não é o dele, ou pior, fica verde por vácuo. O `concat!` de `include_str!` é a
/// pergunta certa em tempo de COMPILAÇÃO: apagar ou renomear um irmão não deixa o gate varrer o
/// vazio, quebra o build do teste; e uma publicação que se mude para um irmão ainda não listado
/// derruba o `expect` ao lado, alto.
const BRIDGE: &str = concat!(
    include_str!("../src/render_loop/vector_bridge.rs"),
    "\n",
    include_str!("../src/render_loop/vector_bridge_publish.rs"),
);

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

/// **O modo Corte NÃO tem ramo próprio no press — ele cai na CANETA.**
///
/// Esta é a decisão inteira do modo, e é a que um gate de fonte tem de proteger: a linha de corte
/// é desenhada pela caneta, e um ramo próprio aqui seria uma segunda resposta a *"como se desenha
/// uma curva?"* — divergiria dela no primeiro refino (handles, fechamento, snap, continuar por um
/// endpoint), e o artista teria duas canetas com comportamentos diferentes.
///
/// O que o modo muda é só o que a caneta PRODUZ: o caminho começado nele fica pendente até o
/// `sync` lhe dar entidade.
#[test]
fn the_cut_mode_draws_with_the_pen_and_owns_no_press_branch() {
    // Nenhum ramo exclusivo de modo intercepta o Cut antes do roteador do pen/shape.
    assert!(
        !DISPATCH.contains("mode == ph2d_tool_vector::DrawMode::Cut {"),
        "o modo Corte ganhou um ramo de press proprio -- ele deixaria de desenhar pela caneta"
    );
    // E o press da caneta ARMA a adoção da lâmina.
    let pen = at(DISPATCH, "let click = self.vec_pen.on_press(");
    let window = &DISPATCH[pen..at(&DISPATCH[pen..], "Some(kind) => {") + pen];
    for (needle, why) in [
        ("DrawMode::Cut", "a adocao nao e' gateada no modo Corte"),
        (
            "PenClick::Started",
            "adota em todo clique, nao so' no que CRIA um caminho",
        ),
        (
            "self.vec_cut_pending = Some(",
            "o caminho novo nunca vira lamina",
        ),
    ] {
        assert!(window.contains(needle), "{why} -- falta `{needle}`");
    }
}

/// **A lâmina é adotada DEPOIS do `sync` e ANTES do `settle`** — a mesma posição do conector e do
/// blend, e pela mesma razão em cada ponta.
///
/// Antes do `sync` a entidade não existe e o componente não tem onde pousar; depois do `settle` o
/// caminho já teria sido assentado como arte comum.
#[test]
fn the_cut_line_is_adopted_between_the_sync_and_the_settle() {
    let sync = at(LOOP_SRC, "crate::vec_entities::sync(");
    let adopt = at(LOOP_SRC, "crate::vec_cut_line::upkeep(");
    let settle = at(LOOP_SRC, "settle_origins(");
    assert!(
        sync < adopt && adopt < settle,
        "a adocao da lamina saiu da janela entre o sync e o settle"
    );
}

/// **Os dois botões do corte são drenados, com UM passo de undo cada.**
///
/// Um `Cut` sem `begin`/`commit_if_changed` é um corte que o Ctrl+Z salta por cima; e sem o
/// `select(None)` a seleção continuaria a apontar formas que as peças substituíram.
#[test]
fn the_two_cut_buttons_are_drained_by_the_shell() {
    let apply = at(LOOP_SRC, "if pending_vec_cut {");
    let discard = at(LOOP_SRC, "if pending_vec_cut_discard {");
    assert!(apply < discard, "os dois ramos trocaram de ordem");
    let a = &LOOP_SRC[apply..discard];
    for (needle, why) in [
        (
            "vec_cut_line::apply_cut(",
            "o botao Cut nao chama a porta que corta",
        ),
        ("self.vec_history.begin(", "sem passo de undo"),
        ("commit_if_changed(", "sem commit do passo"),
        (
            "self.vec_pen.select(None)",
            "a selecao sobrevive as formas que ela apontava",
        ),
    ] {
        assert!(a.contains(needle), "{why} -- falta `{needle}`");
    }
    let d = &LOOP_SRC[discard..discard + 400];
    assert!(
        d.contains("vec_cut_line::discard("),
        "o botao Discard nao chama a porta que apaga a lamina"
    );
}

/// **A shell publica se existe LÂMINA.** Sem isso o painel lê sempre `false` e os dois botões do
/// corte nunca aparecem — a feature ficaria completa e inalcançável.
#[test]
fn the_shell_publishes_whether_a_cut_line_exists() {
    let pos = at(BRIDGE, "set_cut_line_exists(");
    let window = &BRIDGE[pos..pos + 220];
    assert!(
        window.contains("cut_line(") && window.contains("is_some()"),
        "a publicacao nao pergunta ao ECS -- ela e' o unico caminho ate' o painel"
    );
}

/// **A shell publica a CONTAGEM de nós selecionados.** Sem ela o painel lê sempre `0` e o botão
/// **Average** nunca aparece — a feature ficaria completa e invisível.
///
/// ⚠️ É publicação de shell, então nenhum teste de unidade do painel a alcança: o gate de seam
/// prova que o botão aparece *dada a contagem*, e só este prova que a contagem CHEGA.
#[test]
fn the_shell_publishes_how_many_nodes_are_selected() {
    let f = BRIDGE
        .find("set_current_vertex_count(")
        .expect("a contagem de nós não é publicada -- o Average nunca aparece");
    let window = &BRIDGE[f..f + 200];
    assert!(
        window.contains("pen.selected_verts().len()"),
        "a contagem publicada não vem da seleção de nós do pen -- duas fontes divergem"
    );
    assert!(
        window.contains("vector_active"),
        "a contagem não é zerada fora da ferramenta -- o painel de outra tool leria a nossa"
    );
}

/// **A lâmina é desenhada FORA do guard de modo** (Enio, 2026-07-31: *"quando movido com Select a
/// aparência rachurada deve permanecer"*).
///
/// O bloco `if overlay.edit { … }` é **falso no Select** — e a lâmina, que o render de arte não
/// desenha, desaparecia por completo justamente no modo em que se a move. A hachura não é feedback
/// de um MODO: é a aparência do objeto.
///
/// ⚠️ O oráculo é a **PROFUNDIDADE DE CHAVES**, não a ordem: `draw_cut_line` estar *depois* de
/// `if overlay.edit {` é verdade dentro E fora do bloco, então uma asserção de ordem não podia
/// falhar pelo motivo que alega. Contar chaves diz de que lado do `}` a chamada está.
#[test]
fn the_cut_line_is_drawn_outside_the_edit_mode_guard() {
    // Comentários fora do caminho: a região é densa em prosa, e prosa em português tem chaves.
    let code: String = LOOP_SRC
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    let guard = at(&code, "if overlay.edit {");
    let call = at(&code, "draw_cut_line(");
    assert!(guard < call, "a chamada mudou de sitio -- reveja este gate");

    // ⚠️ A pergunta é *o bloco FECHOU antes da chamada?* — e não *a chamada está em profundidade
    // zero?*: ela está legitimamente aninhada no próprio `for`/`if`, então exigir zero seria um
    // gate que falha sobre produto correto (foi o que ele fez na 1ª escrita).
    let mut depth = 0i32;
    let mut closed_before_call = false;
    for c in code[guard..call].chars() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    closed_before_call = true;
                }
            }
            _ => {}
        }
    }
    assert!(
        closed_before_call,
        "o desenho da lamina esta' DENTRO do `if overlay.edit` -- ela sumiria no modo Select, \
         que e' onde o artista a move"
    );
}

/// **Os dois interruptores de POSIÇÃO não estão cruzados** (plano 25 §9, a W6).
///
/// ⚠️ Este gate existe porque o seam do painel **não pode** provar isto: `painted_rect` devolve
/// geometria, e as duas opções de uma linha ocupam as mesmas posições esteja qual estiver acesa.
/// A fiação de facto mora aqui — cada id despacha para o seu `pending`, e trocá-los daria um
/// painel em que ligar "Path" acende "Cross" e vice-versa, sem que teste de unidade nenhum
/// enxergasse.
#[test]
fn the_snap_toggles_are_not_crossed() {
    let code = std::fs::read_to_string("src/render_loop/mod.rs").expect("render_loop");
    for (id, slot) in [
        (
            "VECTOR_SNAP_PATH_OFF",
            "pending_vec_snap_path = Some(false)",
        ),
        ("VECTOR_SNAP_PATH_ON", "pending_vec_snap_path = Some(true)"),
        (
            "VECTOR_SNAP_CROSS_OFF",
            "pending_vec_snap_cross = Some(false)",
        ),
        (
            "VECTOR_SNAP_CROSS_ON",
            "pending_vec_snap_cross = Some(true)",
        ),
        (
            "VECTOR_SNAP_GUIDES_OFF",
            "pending_vec_snap_guides = Some(false)",
        ),
        (
            "VECTOR_SNAP_GUIDES_ON",
            "pending_vec_snap_guides = Some(true)",
        ),
        ("VECTOR_RULERS_OFF", "pending_rulers = Some(false)"),
        ("VECTOR_RULERS_ON", "pending_rulers = Some(true)"),
    ] {
        let at = code
            .find(&format!("ids::{id} {{"))
            .unwrap_or_else(|| panic!("{id} nao e' despachado no render_loop"));
        let tail = &code[at..];
        let end = tail.find("} else if").unwrap_or(tail.len());
        assert!(
            tail[..end].contains(slot),
            "o braco de {id} nao escreve `{slot}` -- os interruptores estao cruzados"
        );
    }
}

/// E cada `pending` chega ao campo CORRESPONDENTE de `vec_snap`. O braço acima decide o
/// destino; este decide o que o destino faz — trocar os dois aqui é o mesmo defeito um passo
/// adiante, e o gate anterior não o vê.
#[test]
fn each_pending_snap_toggle_lands_on_its_own_field() {
    let code = std::fs::read_to_string("src/render_loop/mod.rs").expect("render_loop");
    for (pending, field) in [
        ("pending_vec_snap_path", "self.vec_snap.path = on"),
        ("pending_vec_snap_cross", "self.vec_snap.crossings = on"),
        ("pending_vec_snap_guides", "self.vec_snap.guides = on"),
        // ⚠️ A régua é o único dos cinco que NÃO é campo de ferramenta: ela é vista, e o
        // destino é o hero. Colapsá-la com os outros faria esconder a régua desligar o ímã.
        ("pending_rulers", "hero.view.rulers_visible = on"),
    ] {
        let at = code
            .find(&format!("if let Some(on) = {pending} {{"))
            .unwrap_or_else(|| panic!("{pending} nao e' aplicado"));
        let tail = &code[at..];
        let end = tail.find('}').unwrap_or(tail.len());
        assert!(
            tail[..end].contains(field),
            "`{pending}` nao escreve `{field}`"
        );
    }
}

/// **O gesto da régua vem ANTES de toda ferramenta.**
///
/// A faixa da régua está VISÍVEL com qualquer ferramenta na mão, então um press nela que
/// caísse no picking/gizmo moveria um objeto em vez de puxar uma guia — chrome desenhado e
/// morto sob o mouse. A ordem é a afirmação inteira, e ela não é observável por nenhum teste
/// de unidade: o `dispatch_pointer` exige janela.
///
/// ⚠️ Afirma uma RELAÇÃO POSICIONAL, nunca uma distância em bytes: um proxy de janela expira
/// no dia em que alguém acrescenta um bloco no meio (a cicatriz que os dois arch-gates desta
/// linha já carregaram em 23/07).
#[test]
fn the_guide_gesture_runs_before_any_tool_claims_the_pointer() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/input_dispatch.rs"),
    )
    .expect("input_dispatch.rs");
    let guide = src
        .find("self.guide_pointer_down(")
        .expect("o press de guia é despachado");
    for (needle, what) in [
        ("self.flip_canvas_down(", "o canvas do Flip"),
        ("self.joint_draw_press(", "o arrasto de joint"),
        // ⚠️ A âncora é a CHAMADA, nunca a definição: a posição de um `fn` no arquivo não diz
        // nada sobre ordem de despacho, e a primeira versão deste gate reprovou por comparar
        // com a declaração — um oráculo que não podia estar certo.
        ("self.vec_path_pick_click(", "o picker do Vector"),
        ("self.joint_body_pick_click(", "o picker de corpo do joint"),
    ] {
        if let Some(other) = src.find(needle) {
            assert!(
                guide < other,
                "o press de guia corre DEPOIS de {what} — um press na régua cairia nele"
            );
        }
    }
}

/// O corpo de um `fn` do `input_dispatch`, do nome dele até o `fn` seguinte, **sem comentários**.
///
/// ⚠️ A remoção dos comentários é load-bearing (a lição de `the_grab_is_wired_to_the_pointer`):
/// um gate sobre ordem de CÓDIGO que lê prosa é um gate que qualquer frase dispara, nos dois
/// sentidos — e a prosa desta wave cita os nomes das próprias funções que ele procura.
fn fn_body(src: &str, name: &str, next: &[&str]) -> String {
    let i = src.find(name).unwrap_or_else(|| panic!("`{name}` existe"));
    let tail = &src[i..];
    // O primeiro delimitador que aparecer fecha o corpo. ⚠️ O `#[cfg(test)]` está na lista
    // porque o `on_mouse_input` é o ÚLTIMO `fn` do `impl`: sem ele o "corpo" engoliria o
    // módulo de testes do arquivo, e as asserções NEGATIVAS deste gate passariam a falhar
    // sobre um teste que mencione a função.
    let end = next
        .iter()
        .filter_map(|n| tail.find(n))
        .min()
        .unwrap_or(tail.len());
    tail[..end]
        .lines()
        .map(|l| match l.find("//") {
            Some(c) => &l[..c],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// **Cada FASE do arrasto de guia está ligada à porta que ENTREGA aquela fase.**
///
/// Um arrasto tem três — press, movimento, release — e o winit as entrega por **duas** portas:
/// `on_mouse_input` (botão) e `on_cursor_moved` (movimento). A política pura (`press_plan`,
/// `guide_pos_under`) é **cega a qual delas a chamou**, então nenhum dos gates de unidade pode
/// ver uma fase ligada no lugar errado.
///
/// ⚠️ **Este gate existe porque a wave shipou exatamente esse defeito.** O braço
/// `PointerKind::Move` nasceu dentro do `on_mouse_input`, que **só produz `Down` e `Up`** — o
/// braço era estruturalmente inalcançável, `guide_pointer_move` ficou sem chamador nenhum, e
/// os seis gates de política seguiram VERDES (eles afirmam o que a guia deve fazer *quando
/// alguém a move*, e ninguém a movia). No produto isso saiu como dois sintomas de um defeito
/// só: *"clicar na faixa cria a linha mas ela não segue o mouse"* e *"mover linha não é
/// possível"* — a guia nascia sob o cursor, ficava, e o Up a largava ali.
///
/// O irmão é `the_move_advances_the_hand` (W-Grab), cuja prosa já dizia a consequência:
/// *"sem isto a mão não segue o cursor — ela pega e fica onde estava."*
#[test]
fn each_phase_of_the_guide_drag_is_wired_to_the_door_that_delivers_it() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/input_dispatch.rs"),
    )
    .expect("input_dispatch.rs");
    let moved = fn_body(
        &src,
        "pub(crate) fn on_cursor_moved(",
        &["pub(crate) fn on_mouse_wheel("],
    );
    let button = fn_body(
        &src,
        "pub(crate) fn on_mouse_input(",
        &["\n    pub(crate) fn ", "\n#[cfg(test)]"],
    );

    // O MOVIMENTO mora onde o movimento chega.
    let mv = moved.find("self.guide_pointer_move(").unwrap_or_else(|| {
        panic!(
            "o `guide_pointer_move` tem de ser chamado do `on_cursor_moved` — sem isto a guia \
             nasce sob o cursor e fica onde nasceu"
        )
    });
    assert!(
        !button.contains("self.guide_pointer_move("),
        "o `on_mouse_input` só produz Down e Up: um braço de Move ali é inalcançável, e \
         acreditar nele é o que deixou o gesto meio-ligado"
    );

    // O PRESS e o RELEASE moram onde o botão chega.
    for needle in ["self.guide_pointer_down(", "self.guide_pointer_up("] {
        assert!(
            button.contains(needle),
            "`{needle}` tem de ser chamado do `on_mouse_input`"
        );
        assert!(
            !moved.contains(needle),
            "`{needle}` não pertence ao handler de movimento"
        );
    }

    // E o movimento de um arrasto VIVO precede os outros gestos: um arrasto em curso é dono do
    // ponteiro, senão sair da faixa devolve o gesto ao gizmo no meio do caminho.
    for (needle, what) in [
        ("self.painter_canvas_move(", "o traço do Painter"),
        ("self.flip_canvas_move(", "o traço do Flip"),
        ("self.field_gizmo_move(", "o gizmo de field"),
    ] {
        if let Some(other) = moved.find(needle) {
            assert!(
                mv < other,
                "o movimento da guia corre DEPOIS de {what} — o arrasto perderia o ponteiro"
            );
        }
    }
}

/// A régua é pintada, e o canvas que ela usa é o que o paint RESOLVEU.
///
/// ⚠️ Gate de FONTE porque o `paint_hero_screen` calcula o layout dentro de si: um teste de
/// unidade da régua nunca veria se alguém a alimentou com o `hero.grid.view` cru, cujo canvas
/// é um retângulo de fachada `(0,0,0,0)` — as faixas nasceriam vazias e o gesto nunca
/// dispararia, com todos os 9 gates da régua verdes.
#[test]
fn the_ruler_is_painted_with_the_canvas_the_layout_resolved() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/ph2d-editor-core/src/screens/hero/paint.rs"),
    )
    .expect("hero/paint.rs");
    let call = src
        .find("crate::ruler::paint_rulers(")
        .expect("as réguas são pintadas");
    let head = &src[..call];
    let block = head
        .rfind("if hero.rulers_live()")
        .expect("o desenho pergunta a PORTA ÚNICA, não uma condição própria");
    assert!(
        head[block..].contains("canvas: layout.draw_area,"),
        "a régua recebe a ÁREA DE DESENHO resolvida pelo layout, não o retângulo de fachada do \
         `grid.view`"
    );
    // ⛔ E, desde 2026-08-30, **não** o `layout.canvas`: ele é a viewport inteira, e ancorar a
    // régua nele devolve os dois defeitos de uma vez — 87,8 % da régua esquerda tapada pelo
    // trilho, e o gesto da guia a roubar o clique dos 6 px de cima da barra (ele é geométrico e
    // corre antes do hit-test de chrome). A lei e os dois controlos vivem em
    // `ph2d-editor-core/tests/the_rulers_never_share_a_pixel_with_docked_chrome.rs`.
    assert!(
        !head[block..].contains("canvas: layout.canvas,"),
        "a régua voltou a ser ancorada na viewport inteira — o trilho tapa-a e ela rouba o \
         clique da barra de topo"
    );
    assert!(
        head[block..].contains("hero.grid.snap_state.active_origin()"),
        "o zero da régua É a origem da grade — um número, dois consumidores"
    );
    assert!(
        src.contains("hero.last_canvas = layout.draw_area;"),
        "a área de desenho resolvida é publicada para quem trata ponteiro; sem isto o gesto da \
         régua teria de espelhar a aritmética do layout — e pintar e agarrar divergiriam"
    );
}

/// **Uma faixa que desenha e não responde é chrome morto sob o mouse** — e o inverso, uma que
/// responde sem aparecer, é pior: o artista clica no vazio e nasce uma guia.
///
/// O invariante é *visível ⇔ vivo*, e ele só se sustenta enquanto as DUAS metades perguntarem
/// à mesma função. Este gate afirma isso onde nenhum teste de unidade chega: o paint mora na
/// `editor-core` e o gesto na shell, então nada os obriga a concordar exceto a porta.
#[test]
fn the_paint_and_the_gesture_ask_the_same_door_about_the_rulers() {
    let paint = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/ph2d-editor-core/src/screens/hero/paint.rs"),
    )
    .expect("hero/paint.rs");
    let gesture = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/guide_gesture.rs"),
    )
    .expect("guide_gesture.rs");
    assert!(
        paint.contains("hero.rulers_live()"),
        "o paint tem de perguntar a porta, não uma condição própria"
    );
    assert!(
        gesture.contains("HeroScreen::rulers_live"),
        "o gesto tem de perguntar a MESMA porta"
    );
    // E nenhum dos dois pode ter uma segunda cópia da condição composta.
    for (name, src) in [("o paint", &paint), ("o gesto", &gesture)] {
        assert!(
            !src.contains("view.rulers_visible &&") && !src.contains("&& view.rulers_visible"),
            "{name} recompõe a condição em vez de a perguntar — duas cópias divergem"
        );
    }
}
