//! **A SONDA DA §4.3 do plano da UI viva** — o cursor pode ser PRESO nesta máquina?
//!
//! O scrub numérico (E1) shipa com a possibilidade **(A)**: o cursor VIAJA, e o curso acaba ao fim
//! de ~600 px. A possibilidade **(B)** — `set_cursor_grab(Locked)` + esconder ⇒ arrasto **infinito**
//! — está no plano marcada `⚠️ desconhecido`, com a ordem de **sondar primeiro, prometer depois**.
//! Este arquivo é essa sonda, e ela é a única coisa que pode fechar aquela linha.
//!
//! # Porque a pergunta tem DUAS metades, e a segunda é a que decide
//!
//! Prender o cursor **não basta**: preso, ele deixa de se mover, então
//! [`winit::event::WindowEvent::CursorMoved`] **para de chegar** — e é justamente dele que todo o
//! gesto do app se alimenta hoje. O deslocamento passa a viver no canal CRU
//! ([`winit::event::DeviceEvent::MouseMotion`]), que esta shell **nunca** consumiu (nenhum
//! `device_event` em `shells/desktop/src/`). Logo:
//!
//! | metade | o que ela responde | quem pode responder |
//! |---|---|---|
//! | **1. o LOCK pega?** | `set_cursor_grab` devolve `Ok`? | a máquina, sozinha |
//! | **2. o CRU chega?** | há `MouseMotion` com o cursor preso? | a máquina **e uma mão** |
//!
//! ⚠️ **A metade 2 precisa de CONTROLE, senão um zero é inatribuível.** *Nenhum evento cru* tem
//! duas causas de vereditos opostos — *o canal não existe nesta plataforma* (⇒ **(B) é
//! inconstruível**) e *ninguém mexeu o rato* (⇒ a sonda não mediu nada). Por isso ela corre em
//! **duas fases**: a primeira SEM prender (o controle — se nem `CursorMoved` chega ali, a corrida
//! não vale) e a segunda presa. É a lei do [[feedback_a_negative_search_needs_a_positive_control]].
//!
//! # O que a FONTE do winit já declara, e por isso não é preciso medir
//!
//! Lido em `winit-0.30.13/src/window.rs`, o mapa de plataformas é **complementar** — nenhum dos
//! dois modos existe em toda parte:
//!
//! | modo | X11 | Wayland | macOS | Windows |
//! |---|---|---|---|---|
//! | `Confined` | ✅ | ✅ | ⛔ *"Not implemented"* | ✅ |
//! | `Locked` | ⛔ *"Not implemented"* | ✅ | ✅ | ✅ |
//!
//! ⇒ **(B) nunca pode ser uma promessa incondicional.** O que esta sonda acrescenta é o
//! comportamento REAL desta máquina (a fonte declara intenção; a chamada devolve o facto) e a
//! resposta da metade 2, que a documentação do winit não dá.
//!
//! # Rodar
//!
//! ```text
//! cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Vector && \
//!   cargo test -p ph2d-host-desktop --release probe_can_the_cursor_be_locked -- --ignored --nocapture
//! ```
//!
//! Uma janela pequena abre. **Mexa o rato dentro dela durante os dois períodos** que ela anuncia;
//! ao fim a sonda imprime a tabela e fecha sozinha. Ela **não afirma nada** — é `probe_`, imprime e
//! cala-se, como as irmãs deste repo.

use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{CursorGrabMode, Window, WindowId};

/// Quanto tempo cada fase dura. Curto o bastante para a sonda não prender a sessão, longo o
/// bastante para uma mão chegar à janela e mexer.
const PHASE: Duration = Duration::from_secs(4);

/// O que uma tentativa de prender devolveu.
fn grab_verdict(w: &Window, mode: CursorGrabMode) -> String {
    match w.set_cursor_grab(mode) {
        Ok(()) => "Ok".to_string(),
        Err(e) => format!("ERRO: {e}"),
    }
}

#[derive(Default)]
struct Counts {
    cursor_moved: u32,
    raw_motion: u32,
}

enum Phase {
    /// O CONTROLE: sem prender. Se nem aqui houver `CursorMoved`, ninguém mexeu o rato.
    Free,
    /// Preso: é aqui que a pergunta que decide (B) é feita.
    Locked,
    Done,
}

struct Probe {
    window: Option<Window>,
    phase: Phase,
    started: Instant,
    free: Counts,
    locked: Counts,
    confined_verdict: String,
    locked_verdict: String,
    hide_verdict: String,
}

impl Probe {
    fn new() -> Self {
        Self {
            window: None,
            phase: Phase::Free,
            started: Instant::now(),
            free: Counts::default(),
            locked: Counts::default(),
            confined_verdict: "(nao tentado)".into(),
            locked_verdict: "(nao tentado)".into(),
            hide_verdict: "(nao tentado)".into(),
        }
    }

    fn counts(&mut self) -> &mut Counts {
        match self.phase {
            Phase::Free => &mut self.free,
            Phase::Locked => &mut self.locked,
            Phase::Done => &mut self.free,
        }
    }

    /// A leitura, e ela é uma ESCADA — cada degrau só faz sentido se o de cima passou.
    fn report(&self) {
        eprintln!("\n=== SONDA DO CURSOR PRESO (plano UI viva §4.3) ===");
        eprintln!(
            "sessao   : XDG_SESSION_TYPE={:?}  WAYLAND_DISPLAY={:?}",
            std::env::var("XDG_SESSION_TYPE").unwrap_or_default(),
            std::env::var("WAYLAND_DISPLAY").unwrap_or_default(),
        );
        eprintln!("--- metade 1: a chamada pega? ---");
        eprintln!("  set_cursor_grab(Confined) : {}", self.confined_verdict);
        eprintln!("  set_cursor_grab(Locked)   : {}", self.locked_verdict);
        eprintln!("  set_cursor_visible(false) : {}", self.hide_verdict);
        eprintln!("--- metade 2: o deslocamento CRU chega com o cursor preso? ---");
        eprintln!(
            "  LIVRE  (controle): CursorMoved {:>5}   MouseMotion {:>5}",
            self.free.cursor_moved, self.free.raw_motion
        );
        eprintln!(
            "  PRESO            : CursorMoved {:>5}   MouseMotion {:>5}",
            self.locked.cursor_moved, self.locked.raw_motion
        );
        eprintln!("--- VEREDITO ---");
        if self.free.cursor_moved == 0 {
            eprintln!(
                "  PARE: nem o CONTROLE viu o rato mexer. A corrida nao mediu nada -- \
                 rode outra vez e mexa o rato DENTRO da janela."
            );
            return;
        }
        if !self.locked_verdict.starts_with("Ok") {
            eprintln!(
                "  (B) e' INCONSTRUIVEL nesta plataforma: o lock foi RECUSADO. \
                 O scrub fica na possibilidade (A), o cursor viaja."
            );
            return;
        }
        if self.locked.raw_motion > 0 {
            eprintln!(
                "  (B) e' CONSTRUIVEL: o lock pegou e o canal CRU entrega {} deslocamentos. \
                 O preco e' a fiacao de `device_event` na shell + o par prender/soltar no gesto.",
                self.locked.raw_motion
            );
        } else if self.locked.cursor_moved > 0 {
            eprintln!(
                "  O lock devolveu Ok e NAO pegou: o cursor continuou a mover-se \
                 ({} CursorMoved). Um Ok que nao prende e' pior que um erro.",
                self.locked.cursor_moved
            );
        } else {
            eprintln!(
                "  O lock PEGOU (o cursor parou) e o canal CRU esta MUDO. \
                 (B) e' inconstruivel: nao ha de onde tirar o deslocamento."
            );
        }
    }
}

impl ApplicationHandler for Probe {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("PH2D -- sonda do cursor preso")
            .with_inner_size(winit::dpi::LogicalSize::new(520, 200));
        let Ok(window) = event_loop.create_window(attrs) else {
            eprintln!(
                "PARE: nao foi possivel criar janela (sem display?). A sonda nao mediu nada."
            );
            event_loop.exit();
            return;
        };
        // A metade 1 e' respondida AQUI, antes de qualquer mao entrar em jogo: sao tres chamadas
        // que a maquina responde sozinha. O `Confined` e' medido mesmo sem ser usado depois --
        // ele e' o recuo da tabela do winit, e saber que ele existe muda a decisao de produto.
        self.confined_verdict = grab_verdict(&window, CursorGrabMode::Confined);
        let _ = window.set_cursor_grab(CursorGrabMode::None);
        eprintln!(
            "\n>>> FASE 1 de 2 (CONTROLE, {} s): o cursor esta LIVRE. MEXA O RATO na janela.",
            PHASE.as_secs()
        );
        self.started = Instant::now();
        self.window = Some(window);
    }

    fn window_event(&mut self, _e: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if matches!(event, WindowEvent::CursorMoved { .. }) {
            self.counts().cursor_moved += 1;
        }
    }

    fn device_event(&mut self, _e: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        if matches!(event, DeviceEvent::MouseMotion { .. }) {
            self.counts().raw_motion += 1;
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::Poll);
        if self.started.elapsed() < PHASE {
            return;
        }
        let Some(window) = self.window.as_ref() else {
            return;
        };
        match self.phase {
            Phase::Free => {
                self.locked_verdict = grab_verdict(window, CursorGrabMode::Locked);
                self.hide_verdict = {
                    window.set_cursor_visible(false);
                    "Ok (a API nao devolve erro)".to_string()
                };
                eprintln!(
                    ">>> FASE 2 de 2 ({} s): o cursor foi PRESO ({}). MEXA O RATO outra vez.",
                    PHASE.as_secs(),
                    self.locked_verdict
                );
                self.phase = Phase::Locked;
                self.started = Instant::now();
            }
            Phase::Locked => {
                // Soltar SEMPRE, e antes de imprimir: uma sonda que deixa o cursor preso numa
                // sessao de trabalho e' pior que uma sonda que nao mede.
                let _ = window.set_cursor_grab(CursorGrabMode::None);
                window.set_cursor_visible(true);
                self.phase = Phase::Done;
                self.report();
                event_loop.exit();
            }
            Phase::Done => {}
        }
    }
}

/// Constroi o event loop de dentro do harness de teste.
///
/// ⚠️ **DIFERENÇA DE FIXTURE, nomeada em vez de escondida:** o winit **panica** ao criar um
/// `EventLoop` fora da thread principal em Linux (`platform_impl/linux/mod.rs:725`), e um harness de
/// teste corre sempre numa thread de trabalho — mesmo com `--test-threads=1`. O `with_any_thread`
/// levanta essa recusa. O produto cria o loop na **main**, então esta sonda mede numa configuração
/// que o produto não usa; para a pergunta dela isso não muda a resposta (o `pointer-constraints` do
/// Wayland é por-SUPERFÍCIE, não por-thread), e é por isso que a diferença é aceitável **aqui** e
/// seria inaceitável numa sonda de relógio.
///
/// ⚠️ **E é por isso que ela é Linux-only:** em macOS a mesma recusa existe e **não** tem escape —
/// lá a resposta vem da FONTE do winit (`Confined` ⛔), que a tabela do topo já traz.
#[cfg(target_os = "linux")]
fn build_event_loop() -> Option<EventLoop<()>> {
    use winit::platform::wayland::EventLoopBuilderExtWayland;
    use winit::platform::x11::EventLoopBuilderExtX11;
    let mut builder = EventLoop::builder();
    EventLoopBuilderExtWayland::with_any_thread(&mut builder, true);
    EventLoopBuilderExtX11::with_any_thread(&mut builder, true);
    builder.build().ok()
}

#[cfg(not(target_os = "linux"))]
fn build_event_loop() -> Option<EventLoop<()>> {
    None
}

/// ⚠️ `#[ignore]` e sem asserção: ela IMPRIME e cala-se. O oraculo e' a tabela + a mao do operador,
/// como o `push_look_probe` do Painter precisa de um olho.
#[test]
#[ignore = "abre janela e precisa de uma mao mexendo o rato; rode com --nocapture"]
fn probe_can_the_cursor_be_locked() {
    let Some(event_loop) = build_event_loop() else {
        eprintln!(
            "PARE: sem event loop (fora do Linux, ou sem display). A sonda nao mediu nada -- \
             e um skip NAO e' verde."
        );
        return;
    };
    let mut probe = Probe::new();
    if let Err(e) = event_loop.run_app(&mut probe) {
        eprintln!("PARE: run_app falhou: {e}");
    }
}
