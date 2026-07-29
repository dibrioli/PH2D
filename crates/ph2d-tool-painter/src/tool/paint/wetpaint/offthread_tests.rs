//! **Os gates da SIM FORA DA THREAD** (filho de [`super`] por `#[path]`).
//!
//! A wave move *quem paga o tempo da simulação*, então os gates dela afirmam
//! coisas que nenhum gate anterior podia afirmar: que o FRAME não simula, que a
//! ÁGUA anda sozinha, e que o motor nunca fica largado entre as duas threads.
//!
//! ⚠️ **Dois destes gates existem por causa de defeitos que eu escrevi nesta
//! mesma sessão**, e os dois tinham a MESMA assinatura enganosa — a taxa da água
//! caía 60× enquanto o tick parecia ótimo (0,001 ms p50), porque de fato não
//! havia nada a compositar. Um relógio de frame saudável sobre água parada é
//! exatamente o que o Enio reportou como *"o FPS não cai mas a animação é lenta
//! e truncada"*, então estas duas mutações são a doença original de volta.

use super::*;
use ph2d_painter_brush::Falloff;
use std::time::{Duration, Instant};

/// Uma poça grande a 4096² — a escala em que o passo é caro e a wave importa.
/// ⚠️ **A tela é PARTE do fixture:** a 1024² um passo cabe num frame e todo gate
/// desta família fica VERDE sobre o defeito.
///
/// Os helpers são LOCAIS de propósito: os irmãos (`tests::cp`,
/// `measure_wetpaint_cost::wetted`) são privados aos módulos deles, e abrir
/// visibilidade para um gate é o tipo de erosão que a próxima linha herda.
fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// A poça PEQUENA — para os gates de MECANISMO.
///
/// ⚠️ **A tela grande é fixture do gate de TEMPO, não dos de mecanismo**, e
/// insistir nela ali é como se compra flake: *"a água anda sozinha"* é uma
/// afirmação sobre o worker existir e o relógio ser dele, e um passo a 4096²
/// custa ~25 ms nominais que sob a suíte carregada viram muito mais — o gate
/// falhou uma vez em release por isso. A 512² um passo é ~50× mais barato e a
/// margem fica enorme, sem perder uma vírgula do que ele afirma.
fn small_puddle() -> PainterTool {
    sized_puddle(512, 40.0, 20)
}

fn puddle() -> PainterTool {
    sized_puddle(4096, 100.0, 60)
}

fn sized_puddle(size: u32, radius: f32, moves: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (size * size * 4) as usize], size, size);
    let b = BrushSpec {
        radius_px: radius,
        hardness: 0.5,
        falloff: Falloff::Smooth,
        strength: 1.0,
        color: [0.8, 0.2, 0.1],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t.set_paint_media(PaintMedia::WetPaint);
    let y = f32::from(u16::try_from(size / 2).expect("tela cabe em u16"));
    let step = radius * 0.4;
    t.on_canvas_pointer(cp([radius, y], PointerPhase::Down));
    for k in 1..=moves {
        t.on_canvas_pointer(cp([radius + step * k as f32, y], PointerPhase::Move));
        let _ = t.take_preview_arc();
    }
    t.on_canvas_pointer(cp([radius + step * moves as f32, y], PointerPhase::Up));
    t
}

/// Uma poça PESADA — três traços sobrepostos.
///
/// ⚠️ **É a única fixture em que o gate da espera pode falhar:** com UM traço a
/// versão bloqueante media **10,5 ms** de pior tick, abaixo de qualquer barra
/// honesta, e o gate ficaria verde sobre o desenho que ele existe para recusar.
/// Com três, ela media **60,6 ms**.
fn heavy_puddle() -> PainterTool {
    const DIAG: f32 = std::f32::consts::FRAC_1_SQRT_2;
    let mut t = puddle();
    for lane in 0..2 {
        let x0 = 300.0f32 + 260.0 * lane as f32;
        let y0 = 300.0f32;
        t.on_canvas_pointer(cp([x0, y0], PointerPhase::Down));
        for k in 1..=90 {
            let d = 40.0 * k as f32;
            t.on_canvas_pointer(cp([x0 + d * DIAG, y0 + d * DIAG], PointerPhase::Move));
            let _ = t.take_preview_arc();
        }
        let d = 40.0 * 90.0;
        t.on_canvas_pointer(cp([x0 + d * DIAG, y0 + d * DIAG], PointerPhase::Up));
    }
    t
}

fn sim_frame(t: &mut PainterTool) -> u64 {
    t.wet_bring_home();
    t.paint
        .wetpaint
        .session
        .as_ref()
        .expect("sessao")
        .engine
        .sim
        .frame
}

/// Um frame do produto: o tick, e depois a folga até o vsync (`sleep`, nunca
/// spin — um spin queimaria o núcleo que o worker acabou de ganhar).
fn frame(t: &mut PainterTool) -> f32 {
    const PERIOD: Duration = Duration::from_micros(16_666);
    let a = Instant::now();
    t.paint_tick(1.0 / 60.0);
    let spent = a.elapsed();
    if let Some(rest) = PERIOD.checked_sub(spent) {
        std::thread::sleep(rest);
    }
    spent.as_secs_f32() * 1e3
}

// ---------------------------------------------------------------------------
// 1 — O FRAME NÃO SIMULA
// ---------------------------------------------------------------------------

/// **O tick não roda estágio de sim** — a afirmação inteira da wave, e ela é
/// sobre a ESTRUTURA, então o oráculo é o fonte com controle positivo nas duas
/// pontas: o tick **não** menciona `step_stage`, a porta de teste sincrona
/// **menciona** (senão o gate passaria por o scanner estar quebrado).
#[test]
fn the_frame_does_not_run_a_sim_stage() {
    let src = include_str!("../wetpaint.rs");
    let i = src
        .find("fn wetpaint_tick")
        .expect("o tick mora neste arquivo");
    let j = src[i..]
        .find("\n    /// The canvas-identity guard")
        .expect("o tick termina antes do guard");
    let body = &src[i..i + j];
    assert!(
        !body.contains("step_stage"),
        "o TICK voltou a rodar estagio de sim — o frame paga o passo outra vez, \
         que e a doenca que esta wave existe para curar"
    );
    // Controle positivo: quem DEVE rodar estágio ainda o faz.
    let door = include_str!("offthread.rs");
    assert!(
        door.contains("step_stage"),
        "controle: nem a porta sincrona de teste chama `step_stage` — o scanner \
         esta olhando o lugar errado e o gate acima nao pode falhar"
    );
}

/// **E o motor termina o tick FORA da thread do frame** — a metade de
/// comportamento: se o tick o deixasse em casa, o frame seguinte o encontraria
/// aqui e a sim jamais correria em paralelo.
#[test]
fn the_tick_leaves_the_engine_with_the_worker() {
    let mut t = puddle();
    t.paint_tick(1.0 / 60.0);
    let sess = t.paint.wetpaint.session.as_ref().expect("sessao");
    assert!(
        !matches!(sess.engine, EngineSlot::Here(_)),
        "o tick terminou com o motor em CASA — nada vai simular entre frames"
    );
}

// ---------------------------------------------------------------------------
// 2 — A ÁGUA ANDA SOZINHA
// ---------------------------------------------------------------------------

/// **A sim avança SEM tick nenhum** — o worker é o dono do relógio.
///
/// Zero frames entre o hand-off e a leitura: o que move a água aqui é
/// exclusivamente a outra thread. Mutação que sangra: o `worker_loop` sem o
/// `step_stage` (0 passos), ou o `hand_off_sim` gateado em algo que nunca é
/// verdade (o motor nunca sai e nada anda).
#[test]
fn the_water_advances_with_no_tick_at_all() {
    let mut t = small_puddle();
    t.paint_tick(1.0 / 60.0); // o único tick: é ele que entrega o motor
    let before = {
        let sess = t.paint.wetpaint.session.as_ref().expect("sessao");
        assert!(
            !matches!(sess.engine, EngineSlot::Here(_)),
            "premissa: o motor foi entregue"
        );
        sess.seen_steps
    };
    std::thread::sleep(Duration::from_millis(300));
    let after = sim_frame(&mut t);
    assert!(
        after > before + 2,
        "a agua nao andou fora do frame: {before} -> {after} passos em 300 ms sem \
         um unico tick. A sim voltou a depender do frame."
    );
}

// ---------------------------------------------------------------------------
// 3 — O MOTOR NUNCA FICA LARGADO (os dois defeitos desta sessão)
// ---------------------------------------------------------------------------

// ⚠️ **DUAS MUTAÇÕES SOBREVIVENTES, documentadas em vez de escondidas — e a
// primeira acusa uma FRASE minha, não um buraco de gate.**
//
// Eu escrevi um gate chamado *"um pedido que falha não abandona o motor no
// canal"*, atribuindo o colapso de **33,4 → 0,5 Hz** ao `return` que eu havia
// posto no ramo de falha do `try_bring_home`. **A medição já dizia o contrário e
// eu não a li:** consertar o `return` deixou a taxa em 0,5 Hz; foi desarmar o
// `want` que a levou a 23,9. A atribuição fechou a investigação no lugar errado
// (a doença do doc 28 §5.13), e o gate nasceu incapaz de falhar.
//
// As duas mutações que o provam:
//   (a) `return` no ramo de falha  → VERDE
//   (b) `seen_steps` avançando ANTES do pedido → VERDE
//
// Por quê: o desenho **se autocura**. `fresh` volta a ser verdadeiro no próximo
// passo que o worker completa, e o `recv_timeout` do frame seguinte encontra o
// motor **já no canal** — então o pior efeito de qualquer uma das duas é *um
// composite adiado por um frame*, que é precisamente o que a espera de 4 ms
// existe para tolerar. Um gate que não pode falhar pelo motivo que alega é pior
// que gate nenhum, então ele não foi shipado.
//
// O que FICA gateado é o mecanismo que de fato carrega o peso: o `want`
// (abaixo). Quem quiser fechar o vão precisa de uma fixture em que o pedido
// falhe de forma SUSTENTADA — e com `TICK_WAIT == IDLE_SLEEP` ela não existe.

/// **A ÁGUA CORRE A UMA TAXA UTILIZÁVEL sob um laço de frames REAL** — a
/// afirmação de PRODUTO da wave, e o único oráculo que pegou os defeitos desta
/// sessão.
///
/// A taxa é o que o Enio de fato vê (*"a animação da simulação é lenta e
/// truncada"* com o FPS intacto), e ela é o que **os dois** defeitos moveram —
/// **33,4 → 0,5 Hz**, medido duas vezes, com o tick parecendo ótimo (0,001 ms
/// p50) porque de fato não havia nada a compositar. Qualquer regressão futura
/// que devolva o colapso, por qualquer mecanismo, cai aqui.
///
/// A discriminação é de **60×**, então a barra a 8 Hz fica 4× abaixo do
/// observado e 16× acima do defeito — é isso que a torna imune à carga da suíte.
#[test]
#[cfg_attr(debug_assertions, ignore = "barra de wall-clock: rode em --release")]
fn the_water_runs_at_a_usable_rate_under_a_real_frame_loop() {
    let mut t = puddle();
    let f0 = sim_frame(&mut t);
    let wall = Instant::now();
    for _ in 0..60 {
        frame(&mut t);
    }
    let secs = wall.elapsed().as_secs_f32();
    let hz = (sim_frame(&mut t) - f0) as f32 / secs;
    assert!(
        hz >= 8.0,
        "a sim rodou a {hz:.1} Hz sob um laco de frames real (nominal 40, observado \
         33,4) — a agua voltou a andar em camera lenta com o frame saudavel, que e \
         exatamente o sintoma reportado"
    );
}

// ⚠️ **DUAS DEFESAS EM CAMADA, e nenhuma é observável no regime que shipa** —
// documentadas em vez de gateadas com um oráculo que não pode falhar.
//
// (a) **`hand_off_sim` desarma o `want`.** Um pedido que sobrevive à entrega é
//     uma pergunta já respondida, e o worker a responde de novo: recebe o motor,
//     faz `want.swap(false)`, vê `true`, devolve sem simular um estágio. **É
//     assim que eu produzi o 0,5 Hz** — mas naquele momento `TICK_WAIT` era
//     ZERO. Com os 4 ms medidos o `store(true)` e o `swap(false)` se pareiam
//     dentro do mesmo tick, então nenhum `want` obsoleto se acumula e a mutação
//     passa VERDE. A linha fica: ela é correta, custa um `store`, e é a única
//     defesa se alguém baixar a espera.
//
// (b) **Nenhum `return` no ramo de falha do pedido.** O desenho se autocura:
//     `fresh` volta a ser verdadeiro no próximo passo do worker e o
//     `recv_timeout` seguinte encontra o motor já no canal — o pior efeito é um
//     composite adiado por um frame, que é o que a espera existe para tolerar.
//     Mutei as duas formas (o `return`, e `seen_steps` avançando antes do
//     pedido) e as duas passam.
//
// A lição, que é sobre mim: eu atribuí o colapso de 60× ao `return` **antes de
// reler a minha própria medição**, que já dizia o contrário (consertar o
// `return` deixou 0,5 Hz; desarmar o `want` levou a 23,9). Uma atribuição que
// fecha a investigação no lugar errado é a doença do doc 28 §5.13, e ela custou
// três gates escritos e descartados.

// ---------------------------------------------------------------------------
// 4 — O TICK NÃO ESPERA UM ESTÁGIO
// ---------------------------------------------------------------------------

/// **O tick espera POUCO** — a razão de existir do `recv_timeout` contra o
/// `recv`: a versão bloqueante media **60,6 ms de pior tick** na poça pesada,
/// porque esperava o estágio inteiro DENTRO do frame.
///
/// ⚠️ Barra de wall-clock ⇒ só em release (em debug o passo é ordens de
/// grandeza mais caro e o número mediria o perfil, não o desenho).
#[test]
#[cfg_attr(debug_assertions, ignore = "barra de wall-clock: rode em --release")]
fn the_tick_never_waits_for_a_whole_stage() {
    let mut t = heavy_puddle();
    let mut worst = 0.0f32;
    for _ in 0..90 {
        worst = worst.max(frame(&mut t));
    }
    // Um estágio da poça grande custa ~10 ms e um passo ~25; o tick tem de
    // ficar na ordem da ESPERA (4 ms) mais o composite, nunca do passo.
    assert!(
        worst < 30.0,
        "o pior tick foi {worst:.2} ms — o frame voltou a esperar trabalho de sim \
         (a versao bloqueante media 60,6 ms aqui)"
    );
}

// ---------------------------------------------------------------------------
// 5 — UMA AÇÃO DE CANVAS ALCANÇA O MOTOR
// ---------------------------------------------------------------------------

/// **Uma ação de canvas funciona com o motor FORA de casa** — a porta bloqueante.
///
/// O artista não tem "no frame seguinte": um clique em Fast dry tem de agir
/// agora. Mutação que sangra: tirar o `bring_home`/`reconcile_facts` daquela
/// porta ⇒ o `Deref` do slot panica nomeando o conserto.
#[test]
fn a_canvas_action_reaches_the_engine_while_the_worker_holds_it() {
    let mut t = puddle();
    t.paint_tick(1.0 / 60.0);
    assert!(
        !matches!(
            t.paint.wetpaint.session.as_ref().expect("sessao").engine,
            EngineSlot::Here(_)
        ),
        "premissa: o motor esta com o worker"
    );
    t.wetpaint_fast_dry();
    let sess = t.paint.wetpaint.session.as_ref().expect("sessao");
    assert!(
        !sess.engine.active_grid().has_fluid,
        "o Fast dry nao secou a folha — a acao nao alcancou o motor"
    );
}
