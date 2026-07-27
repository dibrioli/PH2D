//! **A MÃO está ligada ao ponteiro** — arch-gate sobre a costura que nenhum unit
//! test alcança (W-Grab).
//!
//! A decisão (`body_grab::take_hold`) tem gates headless ao lado dela. O que só
//! existe no `input_dispatch`/`render_loop` — e precisa de `App` + `HeroScreen` +
//! janela — são quatro fatos, cada um com um modo de falha próprio e silencioso:
//!
//! 1. **O press PERGUNTA à porta**, com o relógio e o toggle do transporte. Sem
//!    isto a mão não existe e o Play volta a ser só-leitura.
//! 2. **Pegou ⇒ nenhum arrasto de gizmo abre.** Os dois juntos seriam um gesto
//!    inerte cavalgando um vivo (o `Transform` que o arrasto escreve é
//!    sobrescrito pelo readback do mesmo frame) — *inócuo porque outra coisa o
//!    desfaz* é o raciocínio que apodrece.
//! 3. **O release vem ANTES de todo `return` do `on_mouse_input`.** Aquele
//!    handler tem muitos early-returns; um deles engolindo o release deixa o
//!    corpo colado no cursor para sempre.
//! 4. **O overlay LÊ as marcas da ponte**, nunca as re-deriva: uma segunda
//!    estimativa do ponto de pega desenharia uma mola apontando para onde o
//!    corpo não está, e ninguém lê número numa screenshot.
//!
//! Nada aqui afirma distância em bytes nem vizinhança de linhas — a lição de
//! `the_dispatch_is_handed_the_live_geometry` (2026-07-23) é que um proxy
//! posicional expira na wave seguinte. Afirma-se *quem é chamado*, *com que
//! argumentos*, e (no caso 3) uma **relação de ordem** que nenhum formato move.

use std::fs;

fn dispatch_src() -> String {
    fs::read_to_string("src/input_dispatch.rs").expect("input_dispatch.rs")
}

/// O corpo do `fn on_mouse_input`, do `{` de abertura até o fim do arquivo (o
/// suficiente: só se compara ordem DENTRO dele, e o 1º `return` que interessa é o
/// primeiro de todos), **sem comentários**.
///
/// ⚠️ **A remoção dos comentários é load-bearing, e a 1ª versão deste gate morreu
/// por não fazê-la:** ela achou a palavra *"early-returns"* dentro do comentário
/// que eu escrevi ACIMA do release, concluiu que havia um `return` antes dele e
/// falhou sobre produto correto. Uma asserção sobre ordem de **código** que lê
/// prosa é um gate que qualquer frase pode disparar — nos dois sentidos.
fn on_mouse_input_body(src: &str) -> String {
    let i = src
        .find("pub(crate) fn on_mouse_input(")
        .expect("on_mouse_input existe");
    src[i..]
        .lines()
        .map(|l| match l.find("//") {
            Some(c) => &l[..c],
            None => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// **O press pergunta à porta, com as duas condições do chamador.**
#[test]
fn the_press_asks_the_grab_door_with_the_clock_and_the_transport() {
    let src = dispatch_src();
    let i = src
        .find("crate::body_grab::take_hold(")
        .expect("o Down tem de chamar a porta da mão");
    let call = &src[i..i + src[i..].find(");").expect("chamada sem fechamento")];
    assert!(
        call.contains("self.playhead.is_playing()"),
        "condição 1: o relógio decide. Chamada:\n{call}"
    );
    assert!(
        call.contains("self.timeline.flags.simulate_physics"),
        "condição 2: a física armada decide (senão a ponte faz `hold` e a mola não puxa nada). \
         Chamada:\n{call}"
    );
    assert_eq!(
        src.matches("crate::body_grab::take_hold(").count(),
        1,
        "UMA porta, UM sítio: uma 2ª chamada é a 2ª cópia da regra"
    );
}

/// **Pegou ⇒ o arrasto de gizmo NÃO abre.**
///
/// O guard mora na condição do bloco que constrói o `GizmoDragState` no pick de
/// canvas, e é ele que impede os dois gestos de coexistirem.
#[test]
fn taking_hold_suppresses_the_gizmo_drag() {
    let src = dispatch_src();
    // A janela entre a chamada da porta e o `opened_drag = true` do pick de canvas.
    let i = src
        .find("crate::body_grab::take_hold(")
        .expect("a porta é chamada");
    let after = &src[i..];
    let end = after
        .find("opened_drag = true;")
        .expect("o pick de canvas abre o drag depois da porta");
    let window = &after[..end];
    assert!(
        window.contains("if !grabbed"),
        "o bloco que abre o drag tem de estar gateado em `!grabbed`. Janela:\n{window}"
    );
}

/// **O release precede todo `return`.** Relação de ORDEM, imune a formatação.
#[test]
fn the_release_runs_before_any_early_return() {
    let body = on_mouse_input_body(&dispatch_src());
    let release = body
        .find("self.release_body_grab();")
        .expect("o release tem de estar no on_mouse_input");
    let first_return = body.find("return").expect("o handler tem early-returns");
    assert!(
        release < first_return,
        "o release tem de vir antes do 1º `return` (offsets {release} < {first_return}) — \
         uma mão que sobrevive ao release fica colada no cursor para sempre"
    );
    // E é gateado no Released, não em todo evento: pegar e soltar no mesmo Down
    // seria um gesto de duração zero.
    let head = &body[..release];
    assert!(
        head.contains("ElementState::Released"),
        "o release tem de ser gateado no Released. Cabeça:\n{head}"
    );
}

/// **A mão segue o cursor no laço de Move**, ao lado dos outros `advance_*`.
#[test]
fn the_move_advances_the_hand() {
    let src = dispatch_src();
    let i = src
        .find("pub(crate) fn on_cursor_moved(")
        .expect("on_cursor_moved existe");
    let body = &src[i..];
    let end = body
        .find("pub(crate) fn on_mouse_wheel(")
        .expect("o próximo fn delimita o corpo");
    assert!(
        body[..end].contains("self.advance_body_grab();"),
        "sem isto a mão não segue o cursor — ela pega e fica onde estava"
    );
}

/// **O overlay lê as marcas da PONTE.** A ponte é a única dona do fato (o wrapper
/// guarda a tralha); re-derivar o ponto de pega no shell seria uma 2ª resposta a
/// *onde a mola está presa*.
#[test]
fn the_overlay_reads_the_marks_from_the_bridge() {
    let src = fs::read_to_string("src/render_loop/mod.rs").expect("render_loop/mod.rs");
    let i = src
        .find("physics_overlay::draw(")
        .expect("o overlay é despachado");
    let call = &src[i..i + src[i..]
        .find("\n            );")
        .expect("chamada sem fechamento")];
    assert!(
        call.contains("physics.grab_marks()"),
        "o overlay tem de receber as marcas da ponte. Chamada:\n{call}"
    );
}

// ── As ferramentas de PONTO (W-Hand) ────────────────────────────────────────

/// **O intercept das ferramentas de ponto PRECEDE o picking de canvas.**
///
/// Elas precisam só de um PONTO, então pendurá-las no pick — que só dispara com
/// algo sob o cursor — as tornaria inertes no vazio, que é metade de onde um
/// estouro é útil. A relação afirmada é de ORDEM dentro do `dispatch_pointer`, não
/// distância: o `poke_press` tem de aparecer antes do sítio onde a mão é
/// perguntada (`take_hold`), que mora dentro do braço do pick.
#[test]
fn the_point_tools_are_intercepted_before_the_canvas_pick() {
    let src = dispatch_src();
    let poke = src
        .find("self.poke_press(")
        .expect("o intercept das ferramentas de ponto existe");
    let hand = src
        .find("crate::body_grab::take_hold(")
        .expect("o sítio da mão existe");
    assert!(
        poke < hand,
        "o intercept das ferramentas de ponto ({poke}) vem DEPOIS do sítio do \
         pick onde a mão é perguntada ({hand}): um estouro no vazio nunca \
         dispararia"
    );
}

/// **O intercept é gateado em `needs_a_body`, a porta única** — e não numa lista
/// de ferramentas escrita aqui.
///
/// Uma enumeração (`tool == Explode || tool == Attract`) é o que apodrece quando a
/// quarta ferramenta chega: ela nasceria fora do intercept, com o gesto caindo no
/// pick de canvas em silêncio.
#[test]
fn the_point_intercept_asks_the_one_door() {
    let src = dispatch_src();
    let i = src.find("self.poke_press(").expect("o intercept existe");
    // A janela é o BLOCO do `if`, achado para trás a partir da chamada — não uma
    // contagem de bytes.
    let start = src[..i]
        .rfind("if mapped_button")
        .expect("o guard do intercept");
    let guard = &src[start..i];
    assert!(
        guard.contains("!self.interaction.tool.needs_a_body()"),
        "o guard do intercept não pergunta a `needs_a_body` — ele está \
         enumerando ferramentas:\n{guard}"
    );
}

/// **O press de ponto pergunta à porta com o relógio E o toggle**, os mesmos dois
/// fatos que a mão recebe. Sem um deles a ferramenta dispara numa cena parada e o
/// clique não faz nada.
#[test]
fn the_poke_press_asks_the_door_with_the_clock_and_the_transport() {
    let src = dispatch_src();
    let i = src
        .find("crate::body_grab::poke_at(")
        .expect("o press de ponto chama a porta");
    let call = &src[i..i + 400];
    for needle in [
        "&mut gfx.physics",
        "&self.interaction",
        "playing",
        "simulating",
    ] {
        assert!(
            call.contains(needle),
            "a chamada de `poke_at` não passa `{needle}`:\n{call}"
        );
    }
    // E as duas condições vêm de onde só o shell as tem.
    let head = &src[..i];
    assert!(
        head.contains("let playing = self.playhead.is_playing();")
            && head.contains("let simulating = self.timeline.flags.simulate_physics;"),
        "o press de ponto não lê o relógio e o toggle do transporte"
    );
}

/// **O overlay recebe as três marcas da ferramenta**, e a MIRA honra as mesmas
/// duas condições do gesto.
///
/// Uma mira desenhada com o relógio parado promete um clique que a porta recusa —
/// e uma promessa que a ferramenta não cumpre é pior que nenhuma marca.
#[test]
fn the_overlay_is_handed_the_tool_marks() {
    let src = fs::read_to_string("src/render_loop/mod.rs").expect("render_loop/mod.rs");
    let i = src
        .find("physics_overlay::draw(")
        .expect("a chamada do overlay existe");
    let call = &src[i..i + 3000];
    assert!(
        call.contains("physics.attract_marks()"),
        "o campo de atração não chega ao overlay"
    );
    assert!(
        call.contains("self.blast_flash.map("),
        "o flash do estouro não chega ao overlay"
    );
    assert!(
        call.contains("self.interaction.aim_radius()"),
        "a mira não chega ao overlay"
    );
    // A mira é gateada nas MESMAS duas condições da porta.
    let aim = call
        .find("self.interaction.aim_radius()")
        .expect("checked above");
    let window = &call[aim.saturating_sub(300)..aim];
    assert!(
        window.contains("self.playhead.is_playing()")
            && window.contains("self.timeline.flags.simulate_physics"),
        "a mira é desenhada sem honrar o relógio e o toggle:\n{window}"
    );
}
