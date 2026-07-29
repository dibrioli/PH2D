//! **O ORÇAMENTO DE TEMPO da simulação** (filho de [`super`] — teto de LOC da workspace): quanto
//! deste frame a água pode gastar, e como esse número se **descobre** em vez de ser escolhido.
//!
//! O `WET_MAX_STEPS` do pai é um cap de **CONTAGEM**, que é a forma de teto que este repo já
//! descobriu ser um MULTIPLICADOR duas vezes (ADR-0117 no editor de áudio, o plano 26 no undo do
//! Painter): capar a contagem só limita o custo se o custo POR unidade for limitado, e o de um passo
//! de água não é — ele é linear na área molhada, que a mão do artista escolhe.
//!
//! # ⚠️ Por que o orçamento NÃO é um número fixo, e o smoke que provou isso
//!
//! A primeira versão orçava **4 ms/frame fixos**, e o smoke seguinte (Enio, 2026-07-29) reportou
//! *"o FPS não caiu abaixo de 60 mas a animação estava tão lenta e travada como se o FPS fosse 6"* —
//! com o log dizendo exatamente por quê:
//!
//! ```text
//! [frame] total=16.03ms (~62 fps) | cpu-encode(raw)=4.32ms
//!         | present/acquire-stall=11.71ms | tool-tick=0.00ms | stamps=0.00ms
//! ```
//!
//! **A CPU passa 11,7 dos 16,0 ms PARADA esperando o vsync.** Orçar 4 ms num frame com 12 ms de folga
//! ociosa não protege coisa alguma: deixa o hardware parado e põe a água em `4 × 60 ÷ 40 ≈ 6` passos
//! por segundo — o *"FPS 6"* que o Enio viu **era a água, não o app**.
//!
//! A lição é a do §0: *o teto é o do HARDWARE, nunca o do caminho lento* — e um orçamento fixo é um
//! palpite sobre um recurso que **se mede a cada frame**.
//!
//! # O controlador
//!
//! AIMD sobre o `dt` que o próprio `on_tick` recebe. ⚠️ O `Tool` é contrato **CONGELADO** (§6), então
//! não há parâmetro novo a pedir ao shell: o período do frame É o sinal disponível.
//!
//! - **o período** = o PISO do `dt` observado (com decaimento lento para cima) — com vsync ele é o
//!   intervalo do monitor, seja 60 Hz ou 144. É a régua, e ela é MEDIDA.
//! - **cresce** devagar ([`WET_BUDGET_GROW_MS`]) enquanto o frame cabe em `período + folga`;
//! - **encolhe** pela metade quando estoura **E a culpa é da água** — ver abaixo;
//! - **teto** em [`WET_BUDGET_MAX_FRACTION`] do período, que é o que impede a água de comer o quadro
//!   inteiro quando a sim não consegue alcançar o relógio.
//!
//! # ⚠️ A ATRIBUIÇÃO, e o segundo smoke que a exigiu
//!
//! A primeira versão do controlador recuava sempre que o frame estourava — e o smoke seguinte (Enio,
//! 2026-07-29) reportou **o mesmo sintoma de novo**: 60 fps com a simulação parada. O log mostrou
//! `tool-tick=0.00ms` em TODA amostra e a causa ao lado:
//!
//! ```text
//! [frame] total=19.15ms | stamps=13.96ms  | tool-tick=0.00ms
//! [frame] total=32.90ms | stamps=116.03ms | tool-tick=0.00ms
//! ```
//!
//! **O `stamps` é o carimbo de dabs dentro do `on_canvas_pointer`** — outro inquilino do frame, que a
//! água não causa e não controla. O controlador lia o `dt` INTEIRO, concluía *"não há espaço"* e
//! estrangulava a sim até ~2 Hz. **Ele punia a água por uma conta que era de outro.**
//!
//! A regra agora separa as duas: `non_sim = dt − o que a sim gastou`. O recuo só dispara quando **o
//! frame teria cabido sem nós** — se `non_sim` já estourou sozinho, encolher a água não salva o frame
//! e só congela a tinta. Aí o orçamento **segura**, não desce.
//!
//! No log acima o controlador sobe até `0,6 × 16,6 ≈ 10 ms/frame` — **600 ms/s de simulação**, os
//! 40 Hz cheios — gastando a folga do vsync, e o frame continua a 60 fps porque quem encolhe é o
//! *stall*, não o trabalho. Medido na fixture do produto: a sim vai de **13,0 / 11,0 Hz** (orçamento
//! fixo) para **38,0 / 36,5 Hz**. Se a poça crescer a ponto de o frame estourar, ele desce sozinho.
//!
//! ⚠️ **O trade continua sendo o do `max_substeps` da física:** sob carga a água simula MENOS tempo em
//! vez de derrubar o frame. O que mudou é que agora *"sob carga"* é uma MEDIÇÃO do frame, não um
//! palpite.
//!
//! ⚠️ **As três metades se cobrem mutuamente, e por isso cada uma tem gate PRÓPRIO** — a primeira
//! rodada de mutação deixou duas passarem justamente por isso: o crescimento é gateado sob `dt`
//! pinado, o recuo sob um app CPU-bound (com vsync o piso ABSORVE a água e o recuo nunca dispara) e o
//! teto numa poça que a sim não alcança (numa poça leve o `acc` já limita a sim a 40 Hz e o teto é
//! inerte — foi assim que a mutação dele sobreviveu à primeira rodada).

/// A semente do orçamento (ms) — de onde o controlador parte antes de medir o frame.
const WET_BUDGET_SEED_MS: f32 = 4.0;
/// Quanto do período do frame a simulação pode tomar, no máximo. `0,6 × 16,6 ≈ 10 ms` a 60 Hz e
/// `0,6 × 6,9 ≈ 4,1` a 144 Hz — a fração é o que faz o teto ser do HARDWARE do artista.
const WET_BUDGET_MAX_FRACTION: f32 = 0.6;
/// Piso do orçamento: mesmo estourando, a água nunca é completamente congelada.
const WET_BUDGET_MIN_MS: f32 = 1.0;
/// Aditivo por frame na subida (AIMD: sobe devagar, desce pela metade).
const WET_BUDGET_GROW_MS: f32 = 0.5;
/// A folga sobre o período natural que ainda conta como *"o frame coube"*.
const WET_FRAME_SLACK_MS: f32 = 2.0;
/// Decaimento do PISO do período, por frame. O período é o `min` do `dt` observado — um frame lento
/// por culpa de outro inquilino não pode movê-lo —, e este creep para cima (0,05%/frame ≈ 3%/s) é o
/// que o deixa seguir um display que de fato mudou de taxa.
const WET_PERIOD_DECAY: f32 = 1.0005;
/// Piso do período: abaixo disto seria um monitor de 500 Hz, e mais provavelmente um `dt` espúrio.
const WET_PERIOD_MIN_MS: f32 = 2.0;
/// Teto da DÍVIDA (ms). Sem ele um passo patológico congelaria a água por dezenas de frames.
///
/// ⚠️ É **fundo**, não conforto: enquanto a dívida não bate nele o custo amortizado por frame é
/// EXATAMENTE o orçamento — é o clamp que quebraria essa igualdade, e é por isso que ele é fundo.
const WET_MAX_DEBT_MS: f32 = -100.0;

/// O controlador do orçamento — o estado que a sessão carrega.
pub(in crate::tool::paint) struct SimBudget {
    /// O crédito em ms deste frame. Negativo é dívida, paga nos frames seguintes.
    credit_ms: f32,
    /// O orçamento POR FRAME que o controlador descobriu.
    ///
    /// `pub` porque é ele que o gate do MECANISMO afirma: um oráculo de ESTADO é determinístico, e
    /// duas versões daquele gate feitas com relógio (Hz absolutos, e depois uma razão entre janelas)
    /// reprovaram sob a suíte carregada.
    pub(in crate::tool::paint) per_frame_ms: f32,
    /// O período NATURAL do app (ms) — a régua, MEDIDA em vez de assumida.
    period_ms: f32,
    /// Quanto a sim gastou no tick ANTERIOR (ms).
    last_sim_ms: f32,
}

impl SimBudget {
    /// A semente, para os gates a citarem sem a duplicar.
    pub(in crate::tool::paint) const SEED_MS: f32 = WET_BUDGET_SEED_MS;

    pub(in crate::tool::paint) const fn new() -> Self {
        Self {
            // Nasce com um frame de crédito: o primeiro passo nunca espera.
            credit_ms: Self::SEED_MS,
            per_frame_ms: Self::SEED_MS,
            // 60 Hz como semente; o controlador corrige na primeira dúzia de ticks a partir do `dt`
            // REAL, seja qual for o monitor.
            period_ms: 1000.0 / 60.0,
            last_sim_ms: 0.0,
        }
    }

    /// Abre o frame: amostra o período, move o orçamento e credita.
    pub(in crate::tool::paint) fn open_frame(&mut self, dt_ms: f32) {
        // O período é o PISO do `dt`: com vsync ele é o intervalo do monitor, e um frame lento por
        // culpa de outro inquilino (o `stamps` do log) não o move. O creep para cima é o que o
        // deixa seguir um display que de fato mudou.
        self.period_ms = (self.period_ms * WET_PERIOD_DECAY)
            .min(dt_ms.max(WET_PERIOD_MIN_MS))
            .max(WET_PERIOD_MIN_MS);
        let target = self.period_ms + WET_FRAME_SLACK_MS;
        let ceiling = self.period_ms * WET_BUDGET_MAX_FRACTION;
        // O que este frame custou SEM a água. É ele que decide de quem é a culpa.
        let non_sim = dt_ms - self.last_sim_ms;
        if dt_ms <= target {
            // O frame coube: há folga (num app com vsync ela é o present stall, e gastá-la é DE
            // GRAÇA). Sobe devagar.
            self.per_frame_ms = (self.per_frame_ms + WET_BUDGET_GROW_MS).min(ceiling);
        } else if non_sim <= target {
            // Estourou E o frame teria cabido sem nós: a culpa é da água. Desce pela metade.
            self.per_frame_ms = (self.per_frame_ms * 0.5).max(WET_BUDGET_MIN_MS);
        } else {
            // Estourou por conta de OUTRO inquilino (o carimbo de dabs, um load, um painel):
            // encolher a água não salva o frame e só congela a tinta. Segura onde está — mas nunca
            // acima do teto, que segue sendo do período.
            self.per_frame_ms = self.per_frame_ms.min(ceiling);
        }
        // O teto do crédito é UM frame de orçamento: um bucket que entesoura devolve exatamente a
        // rajada que ele existe para impedir.
        self.credit_ms = (self.credit_ms + self.per_frame_ms).min(self.per_frame_ms);
        self.last_sim_ms = 0.0;
    }

    /// Este frame ainda pode pagar um passo?
    pub(in crate::tool::paint) const fn can_step(&self) -> bool {
        self.credit_ms > 0.0
    }

    /// Debita o que o passo de fato custou. ⚠️ O custo REAL, nunca um estimado: o erro de uma
    /// estimativa se acumularia no bucket.
    pub(in crate::tool::paint) fn spend(&mut self, ms: f32) {
        self.credit_ms = (self.credit_ms - ms).max(WET_MAX_DEBT_MS);
        self.last_sim_ms += ms;
    }
}
