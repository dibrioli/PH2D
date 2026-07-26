//! **O gesto de desenhar um joint alcança o canvas e o barramento** — arch-gates
//! sobre a costura window-gated que um unit test não dirige (W-J4).
//!
//! O que dá para gatear headless está gateado: `joint_draw` prova onde as
//! âncoras nascem e o que a corrente monta, e o seam do painel prova que os dois
//! botões chegam ao barramento. O que sobra é a GLUE — o press modal, o Move que
//! estica, o Up que cria, e a banda no overlay — porque cada um precisa de um
//! `App` com janela.

use std::fs;

/// O bloco do gesto no handler de ponteiro: do arm até o fim do braço de Up.
fn gesture_block() -> String {
    let src = fs::read_to_string("src/input_dispatch.rs").expect("input_dispatch.rs");
    let start = src
        .find("if self.joint_draw_armed")
        .expect("o arm do gesto de desenhar sumiu do handler de ponteiro");
    let rest = &src[start..];
    let end = rest
        .find("// **A alça do TEXTO EM CAMINHO**")
        .expect("o fim do bloco (o vizinho seguinte) sumiu");
    rest[..end].to_string()
}

/// **As três fases do gesto estão ligadas.**
///
/// Um press que começa uma banda que nada estica é um gesto que agarra e não
/// segue; um Move sem Up é um que nunca cria. As três são estados de UMA coisa,
/// e é por isso que a ausência de qualquer uma é o mesmo bug.
#[test]
fn the_press_the_move_and_the_release_are_all_wired() {
    let block = gesture_block();
    for needle in [
        "self.joint_draw_press(",
        "self.joint_draw_move(",
        "self.joint_draw_release(",
    ] {
        assert!(
            block.contains(needle),
            "o gesto de desenhar não chama `{needle}` — sem ele o press agarra e \
             nada acontece"
        );
    }
}

/// **O gesto é MODAL: ele consome o press, e corre antes do picking genérico.**
///
/// Armado, o próximo press no canvas é dele. Se caísse depois do caminho
/// genérico, o press selecionaria um sprite (ou pegaria uma alça) e o gesto nunca
/// começaria — a mesma razão do eyedropper do §12.
///
/// ⚠️ **A comparação é DENTRO do dispatch, não sobre o arquivo** — a 1ª versão
/// deste gate comparou posições de byte no arquivo inteiro e ficou VERMELHA sobre
/// produto correto: a 1ª ocorrência de `pick_sprites_at_world` mora no HELPER do
/// eyedropper, definido ACIMA do handler ([[feedback_a_gate_anchored_on_a_byte_distance_is_a_proxy_that_expires]]).
#[test]
fn the_gesture_is_modal_and_precedes_the_generic_picking() {
    let src = fs::read_to_string("src/input_dispatch.rs").expect("input_dispatch.rs");
    // O corpo do dispatch de ponteiro, onde a ORDEM entre os candidatos importa.
    let dispatch = &src[src
        .find("pub(crate) fn on_mouse_input")
        .expect("o handler de botão do mouse sumiu — este gate tem de ser re-apontado")..];
    let gesture = dispatch
        .find("if self.joint_draw_armed")
        .expect("o arm do gesto não está no dispatch de ponteiro");
    let pick = dispatch
        .find("pick_sprites_at_world")
        .expect("o picking genérico sumiu do dispatch");
    assert!(
        gesture < pick,
        "o gesto de desenhar corre DEPOIS do picking — o press seria consumido \
         pela seleção e a banda nunca começaria"
    );
    // E ele CONSOME: um press que cai adiante é um press que o gizmo pega.
    let block = gesture_block();
    assert!(
        block.contains("return;"),
        "o bloco do gesto não consome o evento; o press seguiria para o \
         picking/gizmo e a sprite se moveria em vez de a banda nascer"
    );
}

/// **A banda é desenhada, e o overlay a recebe mesmo com o contorno DESLIGADO.**
///
/// Ela é feedback de um gesto em andamento, não anotação de algo que existe: com
/// a tecla `B` desligada o artista continua vendo o que está fazendo. O gate lê
/// os dois lados — o `draw` recebe a banda, e o desenho dela não está atrás do
/// gate de `show`.
///
/// Mutação-testada: pôr `if show` em volta do bloco da banda vai RED.
#[test]
fn the_band_is_drawn_even_with_the_outline_off() {
    let call = fs::read_to_string("src/render_loop/mod.rs").expect("render_loop/mod.rs");
    assert!(
        call.contains("joint_draw::band(self.joint_draw)"),
        "o `physics_overlay::draw` não recebe mais a banda do gesto"
    );
    let overlay =
        fs::read_to_string("src/render_loop/physics_overlay.rs").expect("physics_overlay.rs");
    let band_at = overlay
        .find("draw_band(join_band")
        .expect("o overlay não desenha mais a banda");
    let show_at = overlay
        .find("joint_marks(show,")
        .expect("o gate de `show` das marcas de joint sumiu");
    assert!(
        band_at < show_at,
        "a banda é desenhada depois das marcas gateadas em `show` — ela tem de \
         vir ANTES e sem esse gate, porque é gesto e não anotação"
    );
    // …e o bloco da banda não olha `show`. ⚠️ A 1ª versão media a JANELA entre a
    // banda e o `joint_marks(show, …)` e ficou vermelha sobre produto correto: o
    // FANTASMA, que fica no meio, abre com `if show` legitimamente. A pergunta é
    // sobre a condição da PRÓPRIA banda.
    let cond_at = overlay[..band_at]
        .rfind("if let Some(band)")
        .expect("o bloco da banda perdeu sua condição");
    let cond = &overlay[cond_at..band_at];
    assert!(
        !cond.contains("show"),
        "a condição que desenha a banda passou a olhar `show` — com a tecla B \
         desligada o gesto ficaria invisível:\n{cond}"
    );
}

/// **O release entrega os DOIS PONTOS do gesto à porta de criação.**
///
/// ⚠️ **Este gate nasceu de uma mutação SOBREVIVENTE**, e o buraco era exatamente
/// esta linha: os gates de comportamento chamam `create_joint_at` **direto**, então
/// provavam que a porta ancora nos pontos e **nada** provava que o release os
/// PASSA. Trocar `Some((d.from, world))` por `None` — a forma da rota por seleção
/// — deixava os 8 verdes com o gesto jogando fora os dois pontos que ele existe
/// para capturar.
///
/// Mutação-testada: `None` no lugar do par vai RED aqui.
#[test]
fn the_release_hands_the_two_points_to_the_creation_door() {
    let src = fs::read_to_string("src/joint_draw.rs").expect("joint_draw.rs");
    let call_at = src
        .find("create_joint_at(")
        .expect("o release não chama mais a porta de criação com pontos");
    // ⚠️ Até o `else` do `let Some(joint) = …`, não até o primeiro `)`: a lista de
    // argumentos contém `to_bits()`, e parar no 1º fecha-parêntese cortaria
    // justamente o par que este gate procura.
    let args = &src[call_at
        ..src[call_at..]
            .find(" else {")
            .map_or(src.len(), |e| call_at + e)];
    assert!(
        args.contains("Some((d.from, world))"),
        "o release não entrega o par (press, release) — sem ele a política de \
         semeadura decide, e a âncora de B nasce no CENTRO do corpo: {args}"
    );
}

/// **Um gesto completo desarma; uma RECUSA não.**
///
/// Soltar fora de um corpo não cria nada (um `pin-to-world` é outra coisa), e o
/// gesto tem de continuar armado para a próxima tentativa — o precedente é o
/// eyedropper do §12, que fica armado quando o clique não resolve. Desarmar numa
/// recusa faria o artista re-clicar o botão a cada mira errada.
#[test]
fn a_completed_gesture_disarms_and_a_refusal_does_not() {
    let src = fs::read_to_string("src/joint_draw.rs").expect("joint_draw.rs");
    let release = &src[src
        .find("pub(crate) fn joint_draw_release")
        .expect("o release sumiu")..];
    let disarm = release
        .find("self.joint_draw_armed = false")
        .expect("um gesto completo não desarma — o próximo press criaria outro joint");
    let refusal = release
        .find("return; // segue armado")
        .expect("a recusa não está marcada como 'segue armado'");
    assert!(
        refusal < disarm,
        "a recusa sai DEPOIS do desarme, então uma mira errada desarmaria o gesto"
    );
}
