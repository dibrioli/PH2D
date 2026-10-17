//! **A SIMULAÇÃO FORA DA THREAD DO FRAME** (filho de [`super`]).
//!
//! # Por que, com os números
//!
//! O custo por célula é o **piso declarado da física** (ADR-0134: *"o teto escalar serial desta
//! física com aritmética f64-espelho-do-JS"*), medido em **16 ns por visita de célula-passe**, e o
//! agendamento está esgotado — seis rodadas de smoke fecharam realimentação, orçamento fixo,
//! atribuição, catraca, régua e o passo atômico. O que sobrava era **quem paga o tempo**: enquanto a
//! sim rodava na thread do frame, a taxa dela era `orçamento ÷ custo do passo` ≈ **15 Hz** numa poça
//! de 4K, e *a taxa visual da água É a taxa de passos* — foi isso que o Enio viu como *"lenta e
//! truncada"* com o FPS intacto.
//!
//! Num núcleo próprio a água ganha os 1000 ms/s inteiros ⇒ `1000 ÷ 33 ≈ 30 Hz`, **o dobro**, e o
//! frame para de pagar qualquer coisa além do composite (1,9 ms medidos).
//!
//! # O desenho: o engine VIAJA, não é compartilhado
//!
//! ⚠️ **Sem mutex, e a escolha é de SEGURANÇA, não de gosto.** Um `Arc<Mutex<Engine>>` com 76 sítios
//! de acesso tem um modo de falha silencioso — travar duas vezes no mesmo escopo é *deadlock*, não
//! erro de compilação. Aqui o engine viaja por canal e [`super::WetSession::eng`] devolve
//! `&mut Engine`: **o borrow checker garante um acesso por vez**, e o pior caso de um acesso é
//! *esperar um ESTÁGIO* (3-10 ms), não um passo (33-38) — que é exatamente o que o passo retomável
//! comprou.
//!
//! # O contrato de quem chama
//!
//! - `WetSession::eng` — o engine, aqui e agora. Bloqueia se o worker o tem (≤ um estágio).
//! - `WetSession::hand_off_sim` — devolve o engine ao worker. O tick faz isto no FIM, uma vez.
//! - Um passo pode voltar **EM VOO** (o worker devolve em fronteira de estágio). Isso é seguro para
//!   o composite (um estado intermediário do pigmento é um estado válido, e é o que o artista veria
//!   um frame depois de qualquer jeito) e é por isso que as ações de CANVAS drenam
//!   (`Engine::drain_step`, já costurado nas portas do engine).
//!
//! # O que NÃO muda
//!
//! A aritmética do motor, a ordem dos estágios e o fingerprint pinado — só **quem chama**
//! `step_stage` muda. A suíte de aceitação dirige o engine direto e não vê esta camada.

use super::fita::{self, Divergencia, MarcaDaAgua, Pergunta, RespostaDoWorker, RitmoDaAgua};
use super::offthread_ritmo::{self, Ritmado};
use ph2d_wet_paint::painter::Engine;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

/// A sim é 40 Hz (SPEC §5) — o worker se ritma nisso, e ficar para trás é câmera lenta (a política
/// do `max_substeps` da física, que este módulo herda).
pub(super) const STEP_S: f64 = 1.0 / 40.0;

/// Quanto atraso o worker aceita antes de DESISTIR do tempo perdido.
///
/// ⚠️ **Dois passos, e o número aperta porque o relógio é CONTÍNUO** (ver
/// `worker_loop`): com o tempo das locações contando como dívida, uma pincelada
/// de três segundos acumularia os 0,25 s antigos = **dez passos de catch-up numa
/// rajada**, que é precisamente o que a política do `max_substeps` da física
/// recusa. A 0,05 s o pior catch-up é de dois passos — o bastante para absorver
/// um round trip, curto o bastante para nunca virar rajada.
const MAX_BEHIND_S: f64 = 2.0 * STEP_S;

/// Quanto o worker dorme quando não há nada a simular. Curto o bastante para a água acordar sem
/// atraso perceptível, longo o bastante para não ocupar um núcleo à toa.
const IDLE_SLEEP: Duration = Duration::from_millis(4);

/// **Quanto o TICK espera pelo motor** — e o número é MEDIDO, com um joelho
/// nítido e um mecanismo que o explica.
///
/// Varredura a 4096², 120 frames a 60 Hz (`measure_what_the_off_thread_sim_buys`):
///
/// ```text
///   espera      0 ms    1 ms    2 ms    4 ms    8 ms
///   sim         23,4    26,9    28,9    33,4    33,9   Hz   (nominal 40)
///   pior tick    7,4     8,2     7,7     9,0    10,7   ms
/// ```
///
/// ⚠️ **O joelho em 4 ms não é arbitrário: é o [`IDLE_SLEEP`].** Quando o worker
/// não tem o que simular ele dorme, e só olha o `want` ao acordar — então a
/// granularidade da resposta dele **é** o sono. Esperar menos que isso erra um
/// worker adormecido e o frame volta de mãos vazias por nada; esperar mais só
/// compra o resto de um estágio, e aí o frame passa a pagar trabalho de sim
/// outra vez (é o que a versão bloqueante fazia: **pior tick de 60,6 ms** na
/// poça de três traços).
///
/// A 4 ms a taxa é **99% da bloqueante** com o tick preso num dígito.
const TICK_WAIT: Duration = IDLE_SLEEP;

/// **Onde o engine está.** Os canais moram no [`SimWorker`] (persistentes); o slot só diz o lado.
pub(in crate::tool::paint) enum EngineSlot {
    /// Na thread do frame — o estado enquanto o artista toca o canvas.
    Here(Box<Engine>),
    /// Com o worker.
    Away,
    /// A tocar uma macro: o motor está «com o worker» do roteiro — fica nesta thread e anda, quando
    /// volta, até a marca gravada ([`super::fita`]). Lê-se como [`Self::Away`] em todo o resto.
    Roteiro(Box<Engine>),
}

/// O worker vivo de uma sessão: os dois canais e o pedido de volta.
///
/// A thread morre quando a sessão termina — nenhum `join` explícito, e é o certo: o engine que ela
/// por acaso tivesse morre com ela, e uma sessão encerrada não tem mais nada a dizer sobre a água.
///
/// ⚠️ **E quem a mata é o [`Drop`], não o canal fechado.** Este doc afirmava
/// *"a thread morre quando `to_worker` é dropado"*, e a afirmação é **falsa
/// exatamente no caso que importa**: com o motor COM ela, o
/// `while let Ok(engine) = rx.recv()` só observa o canal fechado no TOPO do
/// laço externo, e o laço INTERNO só sai quando `want` é setado — que é o que
/// ninguém faz depois de a sessão morrer. O preço era uma thread girando o
/// ritmo de 40 Hz sobre um motor órfão, **por sessão encerrada**, cada uma
/// segurando um `Box<Engine>` inteiro.
pub(in crate::tool::paint) struct SimWorker {
    to_worker: Sender<Box<Engine>>,
    back: Receiver<Box<Engine>>,
    want: Arc<AtomicBool>,
    /// Quantos PASSOS o worker completou. É o que diz ao tick se há algo novo a compositar — sem
    /// isto, o tick traria o engine de volta a cada frame e o worker perderia ~30% do núcleo
    /// esperando.
    steps: Arc<AtomicU64>,
    /// `Some` = o worker anda num ritmo escrito ([`RitmoDaAgua`]), não ao relógio.
    ritmo: Option<Ritmado>,
}

/// Copia o motor que está nesta thread; um motor com o worker não se alcança por `&self` e PANICA
/// nomeado, como o [`std::ops::Deref`] abaixo (quem copia confere antes: `PainterTool::wet_copiavel`).
impl Clone for EngineSlot {
    fn clone(&self) -> Self {
        match self {
            Self::Here(e) => Self::Here(e.clone()),
            Self::Roteiro(e) => Self::Roteiro(e.clone()),
            Self::Away => {
                panic!("EngineSlot::clone com o motor no worker — `wet_copiavel` recusa antes")
            }
        }
    }
}

impl EngineSlot {
    /// Nasce na thread do frame — o worker só entra em cena no primeiro `hand_off_sim`.
    pub(in crate::tool::paint) fn new_here(engine: Engine) -> Self {
        Self::Here(Box::new(engine))
    }
}

/// **O motor por FIELD, não por método** — e é isto que mantém os 87 sítios de
/// `sess.engine.…` intactos, junto com o *field splitting* de que o composite
/// depende (ele precisa das camadas do motor **e** do `pigment`/`base` ao mesmo
/// tempo, e um método `&mut self` emprestaria a sessão INTEIRA).
///
/// ⚠️ **`deref_mut` TRAZ o motor para casa** (bloqueia ≤ um estágio) — ele tem
/// `&mut self`, então pode. O `deref` imutável **não pode** e por isso ele
/// PANICA: a regra é que toda porta que alcança a sessão chama
/// [`super::WetSession::bring_home`] primeiro (as doze estão enumeradas pelo
/// COMPILADOR, não por uma lista — quem esquecer recebe um pânico nomeado no
/// sítio exato, na primeira vez que a suíte passa por ele).
impl std::ops::Deref for EngineSlot {
    type Target = Engine;
    fn deref(&self) -> &Engine {
        match self {
            Self::Here(e) => e,
            Self::Away | Self::Roteiro(_) => panic!(
                "leitura imutavel do engine com ele no worker: \
                 chame `sess.bring_home()` na porta que abriu este caminho"
            ),
        }
    }
}

impl std::ops::DerefMut for EngineSlot {
    fn deref_mut(&mut self) -> &mut Engine {
        // Só o dono da sessão sabe pedir de volta (os canais moram no `SimWorker`), então o braço fora
        // de casa é inalcançável: o `bring_home` da porta já rodou. Ele existe para o caso de alguém
        // adicionar uma porta nova e esquecer — e aí o pânico nomeia o conserto.
        match self {
            Self::Here(e) => e,
            Self::Away | Self::Roteiro(_) => panic!(
                "escrita no engine com ele no worker: \
                 chame `sess.bring_home()` na porta que abriu este caminho"
            ),
        }
    }
}

/// **O que de fato encerra a thread.**
///
/// Setar `want` é a MESMA porta que o tick usa para pedir o motor de volta — e
/// é por isso que ela é a certa aqui: o worker acorda em no máximo um
/// [`IDLE_SLEEP`], tenta devolver, e a devolução falha (o `back` morre com este
/// struct) ou pousa num canal que já está sendo destruído. Nos DOIS caminhos
/// ele sai do laço interno e cai num `rx.recv()` cujo `to_worker` já morreu,
/// então a thread retorna e o motor órfão é liberado.
///
/// ⚠️ **Um sinal PRÓPRIO de shutdown seria uma segunda porta** para a pergunta
/// *"largue o motor"*, e as duas divergiriam no dia em que uma ganhasse um
/// passo a mais.
impl Drop for SimWorker {
    fn drop(&mut self) {
        self.want.store(true, Ordering::Release);
    }
}

impl SimWorker {
    /// Sobe a thread — ao relógio, ou no ritmo escrito de um gate.
    fn spawn(ritmo: Option<&RitmoDaAgua>) -> Self {
        let (to_worker, rx_engine) = channel::<Box<Engine>>();
        let (to_ui, back) = channel::<Box<Engine>>();
        let want = Arc::new(AtomicBool::new(false));
        let steps = Arc::new(AtomicU64::new(0));
        let (w, st) = (Arc::clone(&want), Arc::clone(&steps));
        let (lado, ui) = ritmo.map(offthread_ritmo::par).unzip();
        std::thread::Builder::new()
            .name("ph2d-wet-sim".into())
            .spawn(move || match lado {
                Some(l) => offthread_ritmo::worker_ritmado(
                    &rx_engine,
                    &to_ui,
                    &w,
                    &st,
                    &l,
                    offthread_ritmo::corre_a_quota,
                ),
                None => worker_loop(&rx_engine, &to_ui, &w, &st),
            })
            .expect("a thread da sim de agua");
        Self {
            to_worker,
            back,
            want,
            steps,
            ritmo: ui,
        }
    }
}

/// O laço do worker: recebe o engine, roda estágios no ritmo de 40 Hz, devolve quando pedido.
fn worker_loop(
    rx: &Receiver<Box<Engine>>,
    tx: &Sender<Box<Engine>>,
    want: &AtomicBool,
    steps: &AtomicU64,
) {
    // O relógio é do WORKER: ele deve `STEP_S` de tempo simulado por `STEP_S` de tempo de parede, e
    // ficar para trás é câmera lenta (nunca uma rajada de catch-up — a lição do `max_substeps`).
    let mut acc = 0.0f64;
    // ⚠️ **O relógio NÃO reinicia a cada locação, e esta linha vale 2,4× de taxa.**
    //
    // Ele nascia dentro do `while let`, com o racional de que *"o tempo em que o
    // engine esteve na UI não é dívida do worker"* — e o preço disso é aritmético:
    // `acc` é a DÍVIDA que decide se há passo, então descartar tempo real faz o
    // worker sub-passar na **exata fração descartada**. Medido, um traço a 4096²:
    // **31,9 → 38,4 Hz**, 96% do nominal.
    //
    // ⚠️ **E o ganho é 1,2×, não os 2,4× que a aritmética sugeria** — a estimativa
    // era minha, o número é do produto. O `16,5 Hz` do log do Enio **não** é este
    // defeito: naquela poça o limite é TRABALHO (~60 ms/passo), não relógio, e
    // nenhuma correção de agendamento o move. A fixture de três traços mede
    // **12,5 Hz antes e depois desta linha**, que é a assinatura do regime
    // work-limited — a câmera lenta honesta do `max_substeps`.
    let mut last = Instant::now();
    // A instrumentação (`PH2D_FLUID_PROFILE`): os três baldes que particionam a
    // janela do worker + o COMPUTE acumulado do passo em voo. Ver
    // [`crate::wet_diag`] — sem eles o log do produto imprimia `sim 0.00ms x0`,
    // e a pergunta *"a água lenta é trabalho ou agendamento?"* não tinha número.
    let mut step_us: u64 = 0;
    while let Ok(mut engine) = rx.recv() {
        crate::wet_diag::note_away_closed();
        loop {
            if want.swap(false, Ordering::AcqRel) {
                // Devolve em fronteira de ESTÁGIO — o pior caso de espera da UI.
                // ⚠️ O relógio do `away` arma ANTES do `send`, que MOVE o engine.
                // ⚠️ E o intervalo é PUBLICADO em vez de guardado aqui: uma
                // retenção mais longa que a janela de diagnóstico tem de ser
                // creditada às janelas que ela ATRAVESSA, não àquela em que
                // fecha (`wet_diag::note_away_open`).
                crate::wet_diag::note_away_open();
                if tx.send(engine).is_err() {
                    return;
                }
                break;
            }
            let now = Instant::now();
            acc = (acc + (now - last).as_secs_f64()).min(MAX_BEHIND_S);
            last = now;
            let in_flight = engine.sim.step_pending();
            if !in_flight && (acc < STEP_S || !engine.sim_should_run()) {
                // Nada a fazer: dorme um pouco em vez de girar num núcleo.
                let t = Instant::now();
                std::thread::sleep(IDLE_SLEEP);
                crate::wet_diag::note_sleep(t.elapsed().as_micros() as u64);
                continue;
            }
            let t = Instant::now();
            let closed = engine.step_stage().is_some();
            let spent = t.elapsed().as_micros() as u64;
            step_us += spent;
            crate::wet_diag::note_busy(spent);
            if closed {
                acc -= STEP_S;
                steps.fetch_add(1, Ordering::Release);
                crate::wet_diag::note_step(step_us as f32 / 1000.0);
                // ⚠️ **O custo sem o TAMANHO não é atribuível** — a mesma linha
                // de log serve *"a poça é grande"* e *"a máquina está
                // disputada"*, que pedem curas opostas (`note_cells`). É
                // `O(linhas)` e sai de uma faixa que o rebuild já materializou.
                crate::wet_diag::note_cells(engine.active_grid().live_span_cells());
                step_us = 0;
            }
        }
    }
}

impl super::WetSession {
    /// **Traz o motor para casa** — a porta que TODO caminho que alcança a
    /// sessão chama antes de tocar o engine. No-op quando ele já está aqui
    /// (o caso comum: um `matches!`), e bloqueia ≤ **um estágio** quando não —
    /// que é exatamente o que o passo interrompível comprou (3-10 ms em vez dos
    /// 33-38 de um passo atômico).
    pub(in crate::tool::paint) fn bring_home(&mut self) {
        match self.engine {
            EngineSlot::Here(_) => return,
            EngineSlot::Roteiro(_) => {
                self.volta_do_roteiro(Pergunta::Volta);
                return;
            }
            EngineSlot::Away => {}
        }
        let w = self
            .worker
            .as_ref()
            .expect("o slot so fica Away depois de um hand_off, que cria o worker");
        w.want.store(true, Ordering::Release);
        // A thread só sai quando o canal de IDA fecha, e esse canal vive no `SimWorker` desta
        // sessão — logo, enquanto o worker existe o `recv` tem resposta. O `expect` documenta o
        // invariante em vez de o esconder.
        let engine = w.back.recv().expect("o worker devolve o engine");
        self.engine = EngineSlot::Here(engine);
        self.grava_que_voltou();
    }

    /// **Pede o motor e NÃO espera** — a porta do TICK.
    ///
    /// ⚠️ O tick é a única porta que pode se dar ao luxo de voltar de mãos
    /// vazias, e é a única que precisa: a água corre a ~33 Hz e o display a 60,
    /// então mostrar um passo **um frame depois** é invisível — enquanto
    /// BLOQUEAR custa o estágio inteiro dentro do frame. Medido, poça de três
    /// traços: o `bring_home` bloqueante deixava o pior tick em **60,6 ms**
    /// (uma travada visível) para adiantar um composite que ninguém veria mais
    /// cedo.
    ///
    /// As outras portas (o dab, as ações de canvas, o commit, o fim de sessão)
    /// **têm** de bloquear: elas vão ESCREVER no motor, e não há "no frame
    /// seguinte" para um evento do artista.
    pub(in crate::tool::paint) fn try_bring_home(&mut self) -> bool {
        match self.engine {
            EngineSlot::Here(_) => return true,
            EngineSlot::Roteiro(_) => return self.volta_do_roteiro(Pergunta::TentaVoltar),
            EngineSlot::Away => {}
        }
        if let Some(voltou) = self.pede_ritmado() {
            return self.recebe(voltou);
        }
        let Some(w) = self.worker.as_ref() else {
            return false;
        };
        w.want.store(true, Ordering::Release);
        // Se ele ainda não chegou, o motor está no canal — o `try_recv` do
        // próximo frame o encontra. Nada se perde: `want` já está armado.
        let voltou = w.back.recv_timeout(TICK_WAIT).ok();
        self.recebe(voltou)
    }

    /// **Pede o motor sem esperar NADA** — a porta do evento de ponteiro com o traço aberto (W1e, F6):
    /// esperar o estágio em voo custava o quadro a pintar `1,3 → 11,9 ms` (pior `19`) na poça pesada a
    /// 4096²; o lote que não o encontra espera na fila da sessão e pousa no tick.
    pub(in crate::tool::paint) fn try_bring_home_now(&mut self) -> bool {
        match self.engine {
            EngineSlot::Here(_) => return true,
            EngineSlot::Roteiro(_) => return self.volta_do_roteiro(Pergunta::TentaVoltar),
            EngineSlot::Away => {}
        }
        if let Some(voltou) = self.pede_ritmado() {
            return self.recebe(voltou);
        }
        let Some(w) = self.worker.as_ref() else {
            return false;
        };
        w.want.store(true, Ordering::Release);
        let voltou = w.back.try_recv().ok();
        self.recebe(voltou)
    }

    /// O pedido sem espera a um worker RITMADO ([`offthread_ritmo::Ritmado::pede`]). `None` = o worker anda ao
    /// relógio.
    fn pede_ritmado(&mut self) -> Option<Option<Box<Engine>>> {
        let w = self.worker.as_mut()?;
        Some(w.ritmo.as_mut()?.pede(&w.want, &w.back))
    }

    /// O resultado de um pedido sem espera: o motor (se veio) vai para casa, e a fita grava a resposta.
    fn recebe(&mut self, voltou: Option<Box<Engine>>) -> bool {
        let Some(engine) = voltou else {
            if let Some(f) = &self.fita {
                fita::trava(f).grava(RespostaDoWorker::NaoVoltou);
            }
            return false;
        };
        self.engine = EngineSlot::Here(engine);
        self.grava_que_voltou();
        true
    }

    fn grava_que_voltou(&self) {
        if let Some(f) = &self.fita {
            fita::trava(f).grava(RespostaDoWorker::Voltou(MarcaDaAgua::de(&self.engine)));
        }
    }

    /// O motor volta do worker do ROTEIRO: a fita diz se voltou e em que estado; o motor local anda até
    /// lá. Uma divergência fica registada na fita e o motor volta como está — quem toca nunca pendura.
    fn volta_do_roteiro(&mut self, p: Pergunta) -> bool {
        let Some(f) = self.fita.clone() else {
            unreachable!("o motor só fica no roteiro com uma fita a tocar")
        };
        let mut ft = fita::trava(&f);
        let alvo = match ft.responde(p) {
            Some(RespostaDoWorker::NaoVoltou) => return false,
            Some(RespostaDoWorker::Voltou(m)) => Some(m),
            _ => None,
        };
        let EngineSlot::Roteiro(mut engine) = std::mem::replace(&mut self.engine, EngineSlot::Away)
        else {
            unreachable!("checado por quem chama")
        };
        if let Some(m) = alvo
            && !fita::anda_ate(&mut engine, m)
        {
            ft.diverge(Divergencia::MarcaInalcancavel(m));
        }
        self.engine = EngineSlot::Here(engine);
        true
    }

    /// **A macro deixa de tocar pela fita** (uma etapa editada: a gravação já não diz o que o worker
    /// responderia dali em diante) — o motor do roteiro fica nesta thread, e nada mais pergunta à fita.
    pub(in crate::tool::paint) fn solta_o_roteiro(&mut self) {
        self.engine = match std::mem::replace(&mut self.engine, EngineSlot::Away) {
            EngineSlot::Roteiro(e) => EngineSlot::Here(e),
            outro => outro,
        };
        self.fita = None;
        self.roteiro_teve_worker = false;
    }

    /// **Quantos passos a sim completou** — `0` quando nunca houve worker.
    pub(in crate::tool::paint) fn sim_steps(&self) -> u64 {
        if self.roteiro_teve_worker {
            let f = self.fita.as_ref().expect("roteiro sem fita");
            return match fita::trava(f).responde(Pergunta::Passos) {
                Some(RespostaDoWorker::Passos(n)) => n,
                _ => self.seen_steps,
            };
        }
        let Some(w) = self.worker.as_ref() else {
            return 0;
        };
        if let Some(r) = &w.ritmo {
            r.espera_a_quota();
        }
        let n = w.steps.load(Ordering::Acquire);
        if let Some(f) = &self.fita {
            fita::trava(f).grava(RespostaDoWorker::Passos(n));
        }
        n
    }

    /// **Devolve o engine ao worker** — o tick faz isto no FIM, uma vez. No-op
    /// se ele já está lá.
    ///
    /// Com o traço ABERTO entrega só se o motor anda com ele (`sim_while_painting`; o produto desligou-o
    /// na W1f — a 4096² custava o pior quadro a pintar): a viagem é uma por QUADRO (o primeiro evento de ponteiro do quadro traz o motor, o tick
    /// devolve-o), não uma por dab. Com o port (sim pausada no traço) não há nada a ganhar.
    pub(in crate::tool::paint) fn hand_off_sim(&mut self) {
        if !matches!(self.engine, EngineSlot::Here(_))
            || (self.stroke_open && !self.engine.sim.sim_while_painting)
        {
            return;
        }
        if self.fita.as_ref().is_some_and(|f| fita::trava(f).tocando()) {
            let EngineSlot::Here(engine) = std::mem::replace(&mut self.engine, EngineSlot::Away)
            else {
                unreachable!("checado acima")
            };
            self.engine = EngineSlot::Roteiro(engine);
            self.roteiro_teve_worker = true;
            return;
        }
        let ritmo = self.ritmo.clone();
        let w = self
            .worker
            .get_or_insert_with(|| SimWorker::spawn(ritmo.as_ref()));
        // ⚠️ **DESARMA o pedido antes de entregar** — um `want` que sobrevive à
        // entrega é uma pergunta que já foi respondida, e o worker a responde de
        // novo: ele recebe o motor, faz `want.swap(false)` na PRIMEIRA linha do
        // laço, vê `true`, e devolve **sem simular um estágio**. O motor
        // ping-pongueia entre as duas threads e a água para.
        //
        // Medido: **33,4 → 0,5 Hz** (a taxa caiu 60×, com o tick parecendo
        // ótimo — 0,001 ms p50 — porque de fato não havia nada a compositar).
        // O pedido é do frame que NÃO tinha o motor; quem o tem em mãos não
        // precisa pedir.
        w.want.store(false, Ordering::Release);
        let EngineSlot::Here(engine) = std::mem::replace(&mut self.engine, EngineSlot::Away) else {
            unreachable!("checado acima")
        };
        assert!(
            w.to_worker.send(engine).is_ok(),
            "o worker morreu com o canal de ida vivo"
        );
        if let Some(r) = w.ritmo.as_mut() {
            r.entregou();
        }
    }
}

// ---------------------------------------------------------------------------
// As portas de TESTE — quem precisa de determinismo dirige o motor DIRETO
// ---------------------------------------------------------------------------

/// ⚠️ **Off-thread troca *"a sim avança por TICK"* por *"a sim avança em TEMPO
/// DE PAREDE"*, e as duas não coexistem:** se a UI esperasse o trabalho devido
/// para manter a contagem de passos por tick, o frame voltaria a pagar o passo e
/// o ganho inteiro evaporaria. Tempo de parede é o correto para uma ferramenta
/// interativa (é o que o app de referência faz).
///
/// A saída é a que a suíte de aceitação do engine já usa: **quem precisa de
/// determinismo dirige o motor direto.** Um gate que afirma *"N passos movem a
/// tinta assim"* fala do MOTOR, não do agendador — e o motor segue sendo a
/// mesma função pura, com o mesmo fingerprint.
#[cfg(test)]
impl super::super::PainterTool {
    /// **Traz o motor para casa** — para um gate LER o engine depois de um tick
    /// (que o entrega ao worker). Sem isto a leitura seria de um instante que
    /// ninguém escolheu, e o `Deref` do slot panica dizendo exatamente isso.
    pub(in crate::tool::paint) fn wet_bring_home(&mut self) {
        if let Some(s) = self.paint.wetpaint.session.as_mut() {
            s.bring_home();
        }
    }
}

/// A porta dos gates de FÍSICA — e do relógio fixo (`PainterTool::set_wet_relogio_fixo`: os gates e
/// a miniatura de um preset).
impl super::super::PainterTool {
    /// **`n` passos de sim SINCRONAMENTE, e o composite** — o `on_tick` de
    /// antes desta wave, para os gates cuja afirmação é sobre a FÍSICA (o
    /// escorrido do undo, a paridade de porta, a aceitação): eles pedem um
    /// número de passos e um estado observável, não uma cadência.
    pub(in crate::tool::paint) fn wet_step_sync(&mut self, n: usize) {
        let facts = self.wet_facts();
        let Some(s) = self.paint.wetpaint.session.as_mut() else {
            return;
        };
        s.bring_home();
        s.reconcile_facts(facts);
        let mut moved = false;
        for _ in 0..n {
            if !s.engine.sim_should_run() {
                break;
            }
            // Roda ESTÁGIOS até o passo fechar — a mesma granularidade do
            // worker, então este caminho não é uma segunda implementação do
            // passo (há gate de identidade por-estágio na crate do motor).
            let mut guard = 0;
            while s.engine.step_stage().is_none() {
                guard += 1;
                assert!(guard < 64, "um passo nao completou em 64 estagios");
            }
            moved = true;
        }
        if moved {
            self.wetpaint_composite();
        }
    }
}

#[cfg(test)]
mod leak_tests {
    //! ⚠️ **Aqui, e não no `offthread_tests.rs`:** o oráculo é o `Arc` do pedido,
    //! que é campo PRIVADO do [`SimWorker`] — e abrir a visibilidade para um gate
    //! é o tipo de erosão que a próxima linha herda.
    use super::*;
    use std::time::{Duration, Instant};

    /// **UMA SESSÃO ENCERRADA NÃO DEIXA UMA THREAD SIMULANDO PARA SEMPRE.**
    ///
    /// ⚠️ **Este gate nasceu de um LOG DE SMOKE, não de uma suspeita minha**
    /// (Enio, 2026-07-31):
    ///
    /// ```text
    ///   worker: busy 69% away 31% sleep 909% | TAXA DA AGUA 392.8 Hz (779 passos em 2.0s)
    /// ```
    ///
    /// Três baldes que dizem PARTIÇÃO somando 1009%, e uma sim dez vezes acima dos
    /// 40 Hz nominais. Nenhuma das duas coisas é possível para **um** worker — mas
    /// as duas são exatamente o que **dez** produzem, cada um no seu ritmo correto.
    ///
    /// O doc do [`SimWorker`] afirmava *"a thread morre quando `to_worker` é
    /// dropado (a sessão terminou)"*, e a afirmação é **falsa quando o motor está
    /// COM ela**: o `while let Ok(engine) = rx.recv()` só observa o canal fechado
    /// no TOPO do laço externo, e o laço INTERNO só sai quando `want` é setado —
    /// que é justamente o que ninguém faz depois de a sessão morrer. A thread fica
    /// girando o ritmo de 40 Hz sobre um motor órfão, **para sempre**, e o motor é
    /// um `Box<Engine>` com os quatorze planos dentro.
    ///
    /// O oráculo é o `Arc` do próprio pedido: o worker segura um clone, então a
    /// contagem forte cair para 1 **é** a thread ter saído. Um `join` seria melhor
    /// oráculo e não existe — a thread é solta de propósito.
    #[test]
    fn a_finished_session_does_not_leave_a_worker_simulating_forever() {
        let w = SimWorker::spawn(None);
        // O motor VAI para o worker: é este o estado em que o vazamento existe (com
        // ele em casa, o laço externo observa o canal fechado e sai sozinho).
        let mut e = Engine::new(64, 64);
        e.sim.gravity_override = Some([0.0, 1.0]);
        w.to_worker
            .send(Box::new(e))
            .expect("o worker aceita o motor");
        let want = Arc::clone(&w.want);
        // Deixa o laço interno de fato entrar em regime antes de soltar a sessão.
        std::thread::sleep(Duration::from_millis(30));
        // Três donos: o `SimWorker`, a thread, e este clone.
        assert_eq!(
            Arc::strong_count(&want),
            3,
            "o worker devia estar vivo segurando o motor — a fixture nao contem o fenomeno"
        );
        drop(w);
        // O worker acorda no máximo um `IDLE_SLEEP` depois; meio segundo é folga de
        // três ordens de grandeza, e o gate falha em vez de pendurar.
        let t0 = Instant::now();
        while Arc::strong_count(&want) > 1 && t0.elapsed() < Duration::from_millis(500) {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(
            Arc::strong_count(&want),
            1,
            "a thread da sim sobreviveu a sessao: ela segue simulando um motor orfao \
             (foi isto que o log do smoke mediu como `sleep 909%` e 392 Hz)"
        );
    }
}
