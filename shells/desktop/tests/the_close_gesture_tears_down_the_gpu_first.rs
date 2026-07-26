//! **Arch-gate: o gesto de fechar derruba a GPU ANTES de sair do laço.**
//!
//! ## O defeito (medido, 2026-07-25)
//!
//! O `EventLoop` é **consumido** por `run_app`, então ele — e com ele a conexão Wayland — morre
//! quando `run_app` retorna, e só **depois** o `App` desenrola seus campos. A `SurfaceContext` do
//! `AppGfx` caía nesse rabo, e destruir uma superfície EGL sobre um `wl_display` que já se foi
//! marshala num proxy morto:
//!
//! ```text
//! wl_proxy_marshal_array_flags  (libwayland-client)
//!   <- libnvidia-egl-wayland2 <- libEGL_nvidia <- ph2d-host-desktop  (epílogo do main)
//! ```
//!
//! **217 coredumps desde 2026-07-22, com a MESMA stack, nas SEIS worktrees** — é da shell, não de
//! linha nenhuma. Benigno (dispara depois do `exited cleanly`) e mesmo assim caro: devolve **139** e
//! some com todo `$status` que um smoke poderia checar.
//!
//! A cura é a que a shell já aplicava ao `cpal::Stream` três linhas acima — derrube o recurso de
//! plataforma **aqui**, enquanto tudo está vivo, em vez de deixá-lo cair na cascata de campos.
//!
//! ## Por que um gate de TEXTO
//!
//! O defeito vive no DESLIGAMENTO de um app com janela e superfície EGL de verdade: nenhum teste
//! headless o alcança, e o `ship.sh` roda sem display. O oráculo REAL é o `$?` do processo, e existe
//! (`PH2D_EXIT_AFTER_FRAMES=<n>` fecha pela MESMA porta do X da janela):
//!
//! | build | `$?` |
//! |---|---|
//! | com o teardown | **0** |
//! | sem ele (mutação medida) | **139** |
//!
//! Só que ele precisa de uma máquina com tela. Este gate é o que sobra para o CI: a **ordem** dentro
//! do `on_close_request`, afirmada sobre o fonte.
//!
//! ⚠️ Ele afirma uma **relação posicional**, nunca uma distância em bytes — a lição dos dois
//! arch-gates que a `line/Vector` levou vermelhos ao `main` em 2026-07-23.

const SRC: &str = include_str!("../src/input_dispatch.rs");

/// O corpo de `on_close_request`, do `fn` até o fim do `match` que ele é.
///
/// Ler daqui, e não do arquivo inteiro, é o que impede um `self.gfx = None` de QUALQUER outro lugar
/// (há muitos: rebind de sprite, teardown de tool) de satisfazer as asserções abaixo.
fn close_request_body() -> &'static str {
    let start = SRC
        .find("pub(crate) fn on_close_request(")
        .expect("`on_close_request` sumiu do dispatch — se foi movido, mova este gate junto");
    let end = SRC[start..]
        .find("\n    pub(crate) fn ")
        .map(|o| start + o)
        .expect("o corpo de `on_close_request` não termina numa função irmã");
    &SRC[start..end]
}

/// Controle positivo: o arquivo foi mesmo lido, e é o que este gate pensa que é.
///
/// Sem isto, um `include_str!` apontando para o lugar errado deixaria as asserções de ORDEM abaixo
/// passar por vacuidade — um gate que não pode falhar pelo motivo que alega.
#[test]
fn the_gate_reads_the_dispatch_it_claims_to_read() {
    assert!(
        SRC.len() > 20_000,
        "o dispatch encolheu para {} bytes — este gate provavelmente lê o arquivo errado",
        SRC.len()
    );
    let body = close_request_body();
    assert!(
        body.contains("event_loop.exit()"),
        "o corpo de `on_close_request` não sai do laço; o gate não tem o que ordenar"
    );
}

/// **A superfície EGL morre ANTES do `exit()`, e a janela morre por último.**
///
/// Três afirmações, porque são três fatos independentes:
///
/// 1. o `AppGfx` (que possui a `SurfaceContext`) é derrubado aqui — não na cascata do `main`;
/// 2. ele é derrubado **antes** do `event_loop.exit()`, que é o instante em que a conexão Wayland
///    passa a ter os dias contados;
/// 3. a **janela** vai depois dele — ela possui o `wl_surface` que a superfície referencia, e
///    invertê-los é o mesmo bug com outra ordem de campos.
///
/// **Mutações que devem sangrar:** apagar `self.gfx = None` (mata 1 e 2) · movê-lo para depois do
/// `exit()` (mata 2) · trocar a ordem `gfx`/`window` (mata 3). A primeira foi rodada de verdade e
/// devolveu **139** no oráculo de processo.
#[test]
fn the_close_gesture_drops_the_surface_before_it_leaves_the_loop() {
    let body = close_request_body();
    let gfx = body
        .find("self.gfx = None")
        .expect("`on_close_request` não derruba mais o `AppGfx` — a superfície EGL volta a cair na \
                 cascata de campos do `main`, depois do `wl_display`, e o processo volta a sair 139");
    let window = body
        .find("self.window = None")
        .expect("`on_close_request` não derruba mais a janela");
    let exit = body
        .find("event_loop.exit()")
        .expect("`on_close_request` não sai do laço");
    assert!(
        gfx < exit,
        "o `AppGfx` tem de ser derrubado ANTES do `event_loop.exit()` — depois dele a conexão \
         Wayland está de saída e destruir a superfície marshala num proxy morto"
    );
    assert!(
        gfx < window,
        "a janela possui o `wl_surface` que a superfície referencia: ela vai por ÚLTIMO"
    );
    assert!(
        window < exit,
        "a janela também tem de morrer antes de sair do laço"
    );
}

/// **O auto-fechamento passa pela MESMA porta que o X da janela.**
///
/// `PH2D_EXIT_AFTER_FRAMES` existe para dar um oráculo de processo ao teardown. Se ele saísse por um
/// caminho próprio (`event_loop.exit()` direto, `std::process::exit`), provaria a ordem de destruição
/// de um caminho que o artista nunca toma — verde sobre nada.
#[test]
fn the_self_close_hook_uses_the_window_close_door() {
    let start = SRC
        .find("pub(crate) fn exit_after_frames_tick(")
        .expect("o gancho de auto-fechamento sumiu — se foi movido, mova este gate junto");
    let end = SRC[start..]
        .find("\n    pub(crate) fn ")
        .map(|o| start + o)
        .expect("o corpo de `exit_after_frames_tick` não termina numa função irmã");
    let body = &SRC[start..end];
    assert!(
        body.contains("self.on_close_request(event_loop)"),
        "o auto-fechamento tem de chamar `on_close_request` — é a porta do X da janela, e é a ordem \
         de destruição DELA que o oráculo de processo precisa medir"
    );
    assert!(
        !body.contains("process::exit"),
        "um `process::exit` pula TODO drop: sairia 0 sem provar nada sobre a ordem de morte"
    );
}
