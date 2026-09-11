//! **ARCH-GATE: a máquina de estados de UI ANDA no frame, e o undo espera por ela** (plano W7).
//!
//! ⚠️ **Por que um arch-gate e não um teste de comportamento:** as duas linhas moram dentro do
//! `run_render_frame`, que exige janela + GPU; headless ele retorna no primeiro `let Some(gfx)` e
//! nenhuma asserção sobre o efeito é alcançável. É o mesmo muro que
//! `the_z_projection_reads_the_tree_after_the_sync.rs` e `the_ui_states_survive_the_undo.rs`
//! documentam.
//!
//! ⚠️ **E cada metade falha sozinha, em silêncio:** sem o `dispatch` o botão Show acende, o
//! artista clica e **nada se move** (a máquina existe e nunca anda); sem a supressão a transição
//! anda e cada quadro dela vira um passo de undo — nove Ctrl+Z para desfazer um clique. Nenhum
//! teste de unidade vê qualquer uma das duas.

use std::fs;

fn src(rel: &str) -> String {
    fs::read_to_string(format!("{}/{rel}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|e| panic!("{rel} legivel: {e}"))
}

/// **CONTROLE POSITIVO.** As âncoras existem — sem isto um rename deixaria os gates abaixo verdes
/// por vácuo, a afirmar coisas sobre um arquivo que já não fala delas.
#[test]
fn the_anchors_these_gates_read_still_exist() {
    let loop_src = src("src/render_loop/mod.rs");
    assert!(
        loop_src.contains("fn run_render_frame"),
        "o `run_render_frame` saiu — estes gates descrevem outro arquivo"
    );
    assert!(
        src("src/undo_app.rs").contains("fn post_frame_undo"),
        "o `post_frame_undo` saiu — o gate da supressao descreve outro arquivo"
    );
}

/// **A ponte ANDA a cada frame.** Sem a chamada, `Show` marca um alvo e a cena nunca chega lá.
#[test]
fn the_frame_advances_the_ui_state_machines() {
    let s = src("src/render_loop/mod.rs");
    assert!(
        s.contains("ui_state_bridge::dispatch("),
        "ninguem chama o `ui_state_bridge::dispatch` — o Show acende, o artista clica e nada se \
         move"
    );
    // ⚠️ E o resultado tem de POUSAR no flag; um `dispatch` cujo retorno é descartado deixa a
    // supressão de undo cega, que é a outra metade desta wave.
    //
    // ⚠️ **A atribuição ganhou TERMOS em 2026-08-07** (o modo de preview, W7r: um hover parado
    // também tem de suprimir), então o gate deixou de poder afirmar a expressão INTEIRA — ele
    // afirma a propriedade *o retorno do `dispatch` alimenta o `ui_state_live`*, e quem pina os
    // termos da preview é o `the_preview_owns_the_pointer_and_the_undo`. A comparação é sobre o
    // fonte sem espaço em branco porque quem decide onde a expressão quebra é o `rustfmt`.
    let flat: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let assign = flat
        .find("self.ui_state_live=")
        .expect("ninguem escreve no `ui_state_live` — a supressao de undo nasce morta");
    let stmt = &flat[assign..flat[assign..].find(");").map_or(flat.len(), |e| assign + e)];
    assert!(
        stmt.contains("ui_state_bridge::dispatch("),
        "o `dispatch` corre e o resultado dele nao pousa no `ui_state_live` — a supressao de undo \
         fica cega e uma transicao vira um passo por quadro"
    );
}

/// **O relógio da máquina é o do FRAME.**
///
/// ⚠️ Um relógio próprio (um `Instant` local, um contador) divergiria do resto do app, e o modo
/// de falha é a UI a andar noutra velocidade que a cena — a lição W4.T7 do Motion, onde o
/// `MotionTransport` morreu por isto.
#[test]
fn the_machine_runs_on_the_frames_clock() {
    let s = src("src/render_loop/mod.rs");
    assert!(
        s.contains("let ui_state_dt = report.ticks as f64 * self.fixed_step.fixed_dt();"),
        "o `dt` da maquina deixou de sair dos ticks do frame — um segundo relogio diverge do \
         resto do app"
    );
}

/// **O undo ESPERA a transição terminar.**
#[test]
fn the_undo_waits_for_a_live_transition() {
    // ⚠️⚠️ **O PAR, não um ficheiro.** A `App` que opera a fila mudou-se para o irmão
    // `undo_app.rs` na integração de 2026-09-04 (tecto de LOC estourado pela SOMA de duas
    // linhas), e todo gate que lia só `undo.rs` ficou a afirmar sobre o ficheiro errado — em
    // silêncio no dia seguinte, se a lei ainda lá estivesse. ⇒ *um gate que PARSEIA o fonte lê
    // a família inteira, nunca um nome de ficheiro.*
    let s = src("src/undo_app.rs");
    let at = s
        .find("fn post_frame_undo")
        .expect("o `post_frame_undo` existe");
    let body = &s[at..];
    let guard = body
        .find("let Some(current) = self.capture_project()")
        .expect("o corpo do `post_frame_undo` ainda captura o estado");
    assert!(
        body[..guard].contains("self.ui_state_live"),
        "o `post_frame_undo` nao espera pela transicao viva — um Show de 150 ms vira nove passos \
         de undo, e o artista aperta Ctrl+Z nove vezes para desfazer um clique"
    );
}
