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
    frame_at(t, Duration::from_micros(16_666))
}

fn frame_at(t: &mut PainterTool, period: Duration) -> f32 {
    let a = Instant::now();
    t.paint_tick(1.0 / 60.0);
    let spent = a.elapsed();
    if let Some(rest) = period.checked_sub(spent) {
        std::thread::sleep(rest);
    }
    spent.as_secs_f32() * 1e3
}

// ⚠️ **O RELÓGIO CONTÍNUO DO WORKER fica documentado e NÃO gateado — e a razão
// é que eu escrevi o gate, o rodei, e a medição derrubou o oráculo dele.**
//
// O defeito é real e a correção é de princípio: `last` nascia dentro do
// `while let Ok(engine) = rx.recv()`, então o tempo de cada round trip era
// **descartado** do `acc` — e `acc` é a dívida que decide se há passo, logo o
// worker sub-passava na fração descartada. Tempo real passou; a sim o deve.
// Medido, um traço a 4096²: **31,9 → 38,4 Hz** (96% do nominal de 40).
//
// O gate que eu tentei afirmava que *"a taxa é indiferente ao ritmo do frame"*,
// pela teoria de que um round trip mais longo amplificaria o erro. **Falso, e
// medido nas duas rotas:**
//
// ```text
//              frames 60 Hz   frames 30 Hz   razao
//   limpo         37,8 Hz        31,9 Hz      1,18
//   mutante       32,9 Hz        30,8 Hz      1,07   <- razao MELHOR
// ```
//
// A 30 Hz há **metade** dos round trips, então o erro por-locação cresce e a
// contagem cai — os dois efeitos se cancelam, e a razão anda para o lado errado.
// Uma barra de taxa ABSOLUTA discriminaria (36 Hz), mas fica a 5% do valor
// limpo, que é como se compra flake numa máquina carregada.
//
// O que cobre a regressão catastrófica é o gate de taxa utilizável acima (8 Hz,
// discriminação de 60×). O 1,2× desta linha fica sem gate, de propósito, com o
// número escrito no `worker_loop`.

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
/// ⚠️ **ESTE GATE ERA UM RELÓGIO E VIROU ESTRUTURA — e a razão intermediária
/// TAMBÉM flakou** (2026-08-02, doc 28 §5.68). O histórico inteiro fica porque
/// ele é sobre o oráculo, não sobre o código:
///
/// 1. **Wall-clock `worst < 30.0`.** A doc dele já admitia a flake com o número
///    (**30,64 ms sob a suíte em paralelo**, 2026-07-29), mandava *"re-rode
///    sozinho"* e concluía *"a barra fica onde está"*. Ficou — e em `load 38`
///    passou a falhar **ISOLADA**: FAILED / ok / FAILED em três corridas.
/// 2. **Razão `worst / passo`**, pela teoria de que sob carga os dois termos
///    inflam juntos. **Medida, e REPROVADA:** 1,82 · 1,41 · ok · 0,77 — o pior
///    tick chegou a **47,63 ms contra 26,12 de passo**. A premissa estava errada
///    e o número diz por quê: **`worst` é um MAX sobre 90 amostras**, então ele
///    não mede o desenho e sim *a pior preempção do SO em 1,5 s* — ruído aditivo
///    só no numerador, que razão nenhuma cancela.
///
/// ⇒ **A propriedade é estrutural, então o oráculo é o fonte.** O que separa os
/// dois desenhos não é um milissegundo: é [`WetSession::try_bring_home`] pedir o
/// motor com espera LIMITADA. Com `recv()` no lugar do `recv_timeout(TICK_WAIT)`
/// o tick contém o estágio **por construção** — e isso um scanner vê em qualquer
/// máquina, carregada ou não.
///
/// ⚠️ **Controle positivo nas duas pontas** (o padrão do irmão
/// [`the_frame_does_not_run_a_sim_stage`]): a porta BLOQUEANTE `bring_home` tem
/// de continuar com o `recv()` nu — ela é a das ESCRITAS, e um artista não tem
/// "no frame seguinte". Sem essa metade o gate passaria por o scanner estar
/// olhando o lugar errado.
#[test]
fn the_tick_asks_for_the_engine_with_a_bounded_wait() {
    let src = include_str!("offthread.rs");
    let i = src
        .find("fn try_bring_home")
        .expect("a porta do tick mora neste arquivo");
    let j = src[i..].find("\n    }").expect("a porta do tick termina");
    let tick_door = &src[i..i + j];
    assert!(
        tick_door.contains("recv_timeout(TICK_WAIT)"),
        "a porta do TICK deixou de pedir o motor com espera limitada — com um `recv` nu \
         ela contem o estagio de sim POR CONSTRUCAO (medido: pior tick 60,6 ms)"
    );

    // Controle positivo: a porta das ESCRITAS ainda bloqueia. Se as duas
    // usassem `recv_timeout`, a asserção acima passaria sobre um desenho em que
    // um clique do artista pode voltar de mãos vazias.
    let k = src
        .find("fn bring_home")
        .expect("a porta bloqueante mora neste arquivo");
    let l = src[k..].find("\n    }").expect("a porta bloqueante termina");
    assert!(
        src[k..k + l].contains(".recv()"),
        "controle: nem a porta BLOQUEANTE usa `recv` — o scanner esta olhando o lugar \
         errado e a asserção acima nao pode falhar"
    );
}

/// **E o número que o gate estrutural NÃO afirma** — medição, nunca barra.
///
/// O pior tick contra o custo de um passo. Um desenho bloqueante mede razão
/// **≥ 1** (o tick CONTÉM o passo; medido em 60,6 ms contra ~25); o que shipa
/// mede ~0,5 numa máquina calma. ⚠️ **Sob carga o número não fala sobre o
/// código** — a §5.49 vale aqui na forma mais aguda, porque um MAX amplifica a
/// preempção do SO: com `load 38` esta mesma sonda deu 1,82 sobre um produto
/// correto.
///
/// ⚠️ **A régua é [`PainterTool::wet_step_sync`], não um laço próprio** — ela roda
/// os MESMOS estágios do worker (há gate de identidade por-estágio na crate do
/// motor), então esta não é uma segunda resposta a *"quanto custa um passo?"*; e
/// ela é medida **ANTES** do laço porque a poça SECA, e uma unidade colhida no
/// fim (mais barata) inflaria a razão pelo motivo errado.
#[test]
#[ignore = "medicao — rode com --release --ignored --nocapture --test-threads=1 e load < 5"]
fn measure_the_worst_tick_against_a_step() {
    let mut t = heavy_puddle();
    let mut unit = f32::INFINITY;
    for _ in 0..3 {
        let a = Instant::now();
        t.wet_step_sync(1);
        unit = unit.min(a.elapsed().as_secs_f32() * 1e3);
    }
    let mut worst = 0.0f32;
    for _ in 0..90 {
        worst = worst.max(frame(&mut t));
    }
    println!(
        "\n[tick] pior {worst:.2} ms · passo {unit:.2} ms · razao {:.2}x \
         (bloqueante >= 1,0 · o que shipa ~0,5 em maquina calma)\n",
        worst / unit
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

// ---------------------------------------------------------------------------
// 5 — O INSTRUMENTO NÃO PODE EMUDECER (o defeito desta sessão)
// ---------------------------------------------------------------------------

/// **O worker REPORTA o que custa** — e este gate existe porque o instrumento
/// emudeceu e a mudez passou por resultado.
///
/// Ao mover a sim para fora da thread do frame, ninguém mais chamou
/// [`crate::wet_diag::note_step`] — quem dá o passo passou a ser o worker — e o
/// log do produto imprimiu **`agua: sim media 0.00ms x0`** por uma wave inteira.
/// Aquela linha lê-se como *"a simulação não custa nada"* e significava *"ninguém
/// mede a simulação"*, sobre exatamente o número que decidia se a água lenta é
/// **trabalho** ou **agendamento**. Um instrumento silencioso é pior que um
/// ausente: ele TRANQUILIZA.
///
/// As três mutações sangram, uma por balde: tirar o `note_step` do worker (0
/// passos reportados) · tirar o `note_busy` (o worker não computou nada) · tirar
/// o `note_away` (o motor nunca viajou).
///
/// ⚠️ **Os contadores são globais do processo, então este gate CONSOME a janela**
/// — ele é o único teste não-`#[ignore]` que chama os `take_*`, e é por isso que
/// pode. Um segundo leitor na suíte zeraria a janela deste e o verde viraria
/// sorte; se um aparecer, o instrumento tem de virar por-sessão.
#[test]
fn the_worker_reports_what_a_step_costs() {
    let _ = crate::wet_diag::take_window();
    let _ = crate::wet_diag::take_worker();
    let mut t = small_puddle();
    t.paint_tick(1.0 / 60.0); // entrega o motor ao worker
    // Tempo de parede o bastante para vários passos numa poça pequena, e um
    // tick no fim para o motor voltar (é ele que produz o `away`).
    std::thread::sleep(Duration::from_millis(300));
    t.paint_tick(1.0 / 60.0);

    // ⚠️ **O `away` só é conhecido quando a viagem FECHA:** ele é medido do `send` até o `recv`
    // do worker VOLTAR, e o tick acima acabou de devolver o motor — ler o balde antes disso o
    // encontra vazio, e foi assim que este gate nasceu vermelho.
    //
    // A espera é uma CONDIÇÃO, não uma duração. Uma pausa fixa é uma aposta sobre a velocidade
    // da máquina, e ela perdeu duas vezes no MESMO dia (integração de 2026-07-30): sob a suíte
    // inteira em paralelo, e no perfil **debug**, onde tudo corre ~16× mais devagar. O que o
    // gate afirma não mudou — só deixou de supor quanto tempo o outro lado leva.
    //
    // ⚠️ Os `take_*` ZERAM a janela, então o laço **acumula** o que cada leitura tirou: um poll
    // que descartasse a leitura vazia perderia justamente o balde que ele espera.
    let (mut step_sum, mut step_n, mut busy, mut away) = (0.0f64, 0u64, 0.0f64, 0.0f64);
    let mut cells = 0u64;
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        let ((s, _, n), ..) = crate::wet_diag::take_window();
        let (b, a, _sleep) = crate::wet_diag::take_worker();
        // ⚠️ **Drenado no MESMO laço, e não num gate irmão:** este é o único
        // teste não-`#[ignore]` que consome a janela global, e um 2º leitor
        // zeraria a dele — o verde viraria sorte.
        cells = cells.max(crate::wet_diag::take_cells());
        step_sum += s;
        step_n += n;
        busy += b;
        away += a;
        if (step_n > 0 && step_sum > 0.0 && busy > 0.0 && away > 0.0 && cells > 0)
            || std::time::Instant::now() >= deadline
        {
            break;
        }
        // ⚠️ E a espera **dirige o produto**: o `away` só existe quando o motor VIAJA, então um
        // laço que apenas dorme nunca o preenche — foi assim que a 1ª versão desta espera falhou
        // em debug depois de 10 s. Cada volta é um frame a mais, que é o que o gate já fazia.
        std::thread::sleep(Duration::from_millis(10));
        t.paint_tick(1.0 / 60.0);
    }
    assert!(
        step_n > 0 && step_sum > 0.0,
        "o worker nao reportou passo nenhum ({step_n} passos, {step_sum:.3} ms): o log do \
         produto volta a imprimir `sim media 0.00ms x0`, que le-se como `a sim nao custa nada`"
    );
    assert!(
        busy > 0.0,
        "o balde BUSY do worker esta vazio: sem ele nao se sabe se a agua lenta e \
         trabalho (so a GPU move) ou agendamento (curas opostas)"
    );
    assert!(
        away > 0.0,
        "o balde AWAY esta vazio: o motor viajou para o frame (houve composite) e o \
         preco do handshake nao foi medido de nenhum lado"
    );
    // ⚠️ **O TAMANHO da poça é o divisor sem o qual o custo não é atribuível**
    // (`wet_diag::note_cells`): com ele o log dá `ns/célula`, e um custo por
    // célula CONSTANTE entre duas janelas diz *"a poça cresceu"* enquanto um
    // que SOBE diz *"a máquina ficou disputada"* — curas opostas. Sem ele as
    // duas leituras são o mesmo número, que foi exatamente o impasse do smoke
    // de 2026-07-31.
    assert!(
        cells > 0,
        "o balde POCA esta vazio: o log volta a dizer so quanto o passo CUSTOU, e \
         'a agua esta lenta' deixa de ser atribuivel a trabalho ou a contencao"
    );
}
