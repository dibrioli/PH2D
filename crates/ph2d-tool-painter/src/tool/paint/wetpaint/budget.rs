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
//! - **a régua** = [`WET_FRAME_REFERENCE_MS`], o quadro de 60 fps que o app tem como alvo — uma
//!   referência DECLARADA, e a razão de não ser medida está na seção seguinte.
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
//! # ⚠️ Por que a régua é DECLARADA e não medida — os dois estimadores que falharam
//!
//! Tentei duas vezes derivar o período do frame a partir do `dt`, e cada um falhou por um motivo
//! **oposto** ao do outro:
//!
//! - **EWMA do `dt`** → um frame lento por culpa de outro inquilino LEVANTA a régua, e o teto junto
//!   (`0,6 × 100 ms` = licença para a água comer 60 ms de quadro). Pior: com a água rodando, a maioria
//!   dos frames contém um passo, então a média inclui o **nosso próprio custo** e o laço é
//!   auto-realizável.
//! - **`min` do `dt`** → uma **catraca de mão única**. E a premissa dela era falsa: `dt` abaixo do
//!   vsync **não é espúrio neste app**, é comum (dois frames em sequência depois de um evento dão
//!   `dt ≈ 1 ms`). Um único deles pregava a régua no piso ⇒ teto ≈ 2 ms ⇒ **a água a 4 Hz num app com
//!   14 ms de folga ociosa por frame**, que é exatamente o que o smoke do Enio mediu
//!   (`agua: sim media 28,70ms x8` em 120 frames).
//!
//! O sinal disponível (`dt` sozinho, porque o `Tool` é contrato congelado) **não separa** *"o display
//! mudou de taxa"* de *"este frame foi rápido/lento por outro motivo"*. Então a régua deixou de ser
//! uma inferência: ela é o quadro de **60 fps que o app tem como alvo**.
//!
//! ⚠️ **A limitação, nomeada em vez de escondida:** num display de 144 Hz a água pode tomar 16,6 ms de
//! um quadro de 6,9 ⇒ **enquanto há água viva o app roda a ~60 fps**, não a 144. É o mesmo trade que o
//! Enio declarou três vezes (a água antes dos últimos quadros por segundo), agora também no eixo do
//! monitor. Levantá-lo exige o shell CONTAR o período do display para o tool — parâmetro novo no
//! `Tool`, ou seja **§6 + ADR**.
//!
//! ⚠️ **As três metades se cobrem mutuamente, e por isso cada uma tem gate PRÓPRIO** — a primeira
//! rodada de mutação deixou duas passarem justamente por isso: o crescimento é gateado sob `dt`
//! pinado, o recuo sob um app CPU-bound (com vsync o piso ABSORVE a água e o recuo nunca dispara) e o
//! teto numa poça que a sim não alcança (numa poça leve o `acc` já limita a sim a 40 Hz e o teto é
//! inerte — foi assim que a mutação dele sobreviveu à primeira rodada).

/// A semente do orçamento (ms) — de onde o controlador parte antes de medir o frame.
const WET_BUDGET_SEED_MS: f32 = 4.0;
/// Quanto do período do frame a simulação pode tomar, no máximo.
///
/// ⚠️ **1,0 — um frame INTEIRO — e o número vem de um fato do motor, não de generosidade:** o
/// acumulador `acc` já limita a sim a `período ÷ 25 ms` passos por frame (0,67 a 60 Hz), então um
/// orçamento maior **não** compra mais simulação; ele só evita que um passo ATÔMICO seja adiado. Com
/// `0,6` o teto era 10 ms e um passo custa 12-17 ⇒ **todo passo era adiado ao menos um frame**, e a
/// água rodava a metade da taxa por um teto que não protegia nada que o `acc` já não protegesse.
const WET_BUDGET_MAX_FRACTION: f32 = 1.0;
/// Piso do orçamento: mesmo estourando, a água nunca é completamente congelada.
const WET_BUDGET_MIN_MS: f32 = 1.0;
/// Aditivo por frame na subida (AIMD: sobe devagar, desce pela metade). 1 ms leva o orçamento da
/// semente ao teto em ~13 frames (0,2 s) — rápido o bastante para o artista não ver a água acordar.
const WET_BUDGET_GROW_MS: f32 = 1.0;
/// A folga sobre o período natural que ainda conta como *"o frame coube"*.
///
/// ⚠️ **8 ms, e o número é uma DECISÃO DE PRODUTO declarada pelo Enio, não uma folga técnica.** Um
/// passo de sim custa 12-17 ms e um quadro de 60 Hz tem 16,6 ⇒ **o frame que contém um passo perde o
/// vsync por construção**. Com uma folga apertada (2 ms) o controlador estabiliza cedendo metade da
/// taxa da água para segurar os últimos quadros: medido em laço fechado, orçamento **7,0 ms ⇒ a sim
/// em ~25 Hz** com o frame a 60 fps.
///
/// O Enio reportou três vezes o MESMO veredito — *"o FPS não caiu abaixo de 60 mas a animação estava
/// lenta e travada"* — ou seja, **a água tem prioridade sobre os últimos quadros por segundo**. Com
/// 8 ms de folga o alvo vira ~24,6 ms (40 fps): a água roda os **40 Hz cheios** e o frame passa a
/// ~20 ms (50 fps) enquanto ela está viva. O recuo continua guardando o caso patológico (um passo de
/// 100 ms estoura o alvo com folga e a água desacelera).
const WET_FRAME_SLACK_MS: f32 = 8.0;
/// Peso do EWMA do `dt`. ⚠️ **A decisão é sobre carga SUSTENTADA, nunca sobre um frame** — um passo
/// de sim é ATÔMICO e custa mais que um quadro, então o frame que o contém estoura *sempre*. Decidir
/// pelo `dt` instantâneo fazia o recuo disparar em TODO passo, e como ele é ×0,5 contra +1 de
/// subida, o orçamento **catracava até o piso**: foi exactamente isto que o Enio viu como *"simulação
/// muitíssimo mais devagar"* (log: `tool-tick=17.31ms` numa amostra e `0.00` em todas as outras).
/// 0,05 dá memória de ~20 frames — um passo isolado quase não a move, carga real move.
const WET_DT_EWMA: f32 = 0.05;
/// **A RÉGUA: o quadro de 60 fps que o app tem como alvo.** Declarada, não inferida — ver a seção
/// *"Por que a régua é DECLARADA"* no topo deste módulo, com os dois estimadores que falharam.
const WET_FRAME_REFERENCE_MS: f32 = 1000.0 / 60.0;
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
    /// EWMA do `dt` e do `dt` SEM a água: a decisão é sobre carga sustentada, e um passo atômico
    /// estoura o quadro que o contém por construção.
    dt_avg_ms: f32,
    non_sim_avg_ms: f32,
    /// **Quanto o TICK INTEIRO gastou no frame anterior** (ms) — os passos E o composite.
    ///
    /// ⚠️ É o tick inteiro, não só `step_simulation`: o composite é custo da ÁGUA, e atribuí-lo ao
    /// "outro inquilino" fazia a água parecer inocente em toda medição e crescer sem limite. Medido
    /// no gate CPU-bound: com só o passo na conta, `non_sim` dava 25 contra um alvo de 20,4 ⇒ o ramo
    /// do inquilino estrangeiro vencia sempre e o recuo NUNCA disparava (frame em 2,06× o overhead).
    last_tick_ms: f32,
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
            dt_avg_ms: WET_FRAME_REFERENCE_MS,
            non_sim_avg_ms: WET_FRAME_REFERENCE_MS,
            last_tick_ms: 0.0,
        }
    }

    /// Abre o frame: amostra o período, move o orçamento e credita.
    pub(in crate::tool::paint) fn open_frame(&mut self, dt_ms: f32) {
        let target = WET_FRAME_REFERENCE_MS + WET_FRAME_SLACK_MS;
        let ceiling = WET_FRAME_REFERENCE_MS * WET_BUDGET_MAX_FRACTION;
        // A decisão é sobre carga SUSTENTADA: um passo é atômico e custa mais que um quadro, então o
        // frame que o contém estoura sempre. `dt` instantâneo aqui faz o recuo disparar em todo
        // passo e o orçamento catracar até o piso.
        self.dt_avg_ms += (dt_ms - self.dt_avg_ms) * WET_DT_EWMA;
        // E o que o frame custou SEM a água — quem decide de quem é a culpa.
        self.non_sim_avg_ms += ((dt_ms - self.last_tick_ms) - self.non_sim_avg_ms) * WET_DT_EWMA;
        let (dt_ms, non_sim) = (self.dt_avg_ms, self.non_sim_avg_ms);
        if dt_ms <= target {
            // O frame coube: há folga (num app com vsync ela é o present stall, e gastá-la é DE
            // GRAÇA). Sobe devagar.
            self.per_frame_ms = (self.per_frame_ms + WET_BUDGET_GROW_MS).min(ceiling);
        } else if non_sim <= target {
            // Estourou E o frame teria cabido sem nós: a culpa é da água. Desce pela metade.
            self.per_frame_ms = (self.per_frame_ms * 0.5).max(WET_BUDGET_MIN_MS);
        } else {
            // Estourou por conta de OUTRO inquilino (o carimbo de dabs, um load, um painel).
            // Encolher a água não salva o frame e só congela a tinta ⇒ ela CRESCE de volta à sua
            // parte, como no ramo de cima.
            //
            // ⚠️ **Crescer, não "segurar"** — a primeira versão segurava, e segurar num piso é ficar
            // preso nele para sempre: o diagnóstico mostrou o orçamento parado em **1,04 ms** por 80
            // frames depois de três recuos que a transição do EWMA disparou. Quem protege o frame
            // aqui é o TETO (a referência de quadro); o recuo é só para quando a água é a culpada.
            self.per_frame_ms = (self.per_frame_ms + WET_BUDGET_GROW_MS).min(ceiling);
        }
        // O teto do crédito é UM frame de orçamento: um bucket que entesoura devolve exatamente a
        // rajada que ele existe para impedir.
        self.credit_ms = (self.credit_ms + self.per_frame_ms).min(self.per_frame_ms);
        self.last_tick_ms = 0.0;
    }

    /// O que o TICK inteiro custou neste frame — a conta que a atribuição usa.
    pub(in crate::tool::paint) fn note_tick(&mut self, ms: f32) {
        self.last_tick_ms = ms;
    }

    /// Este frame ainda pode pagar um passo?
    pub(in crate::tool::paint) const fn can_step(&self) -> bool {
        self.credit_ms > 0.0
    }

    /// Debita o que o passo de fato custou. ⚠️ O custo REAL, nunca um estimado: o erro de uma
    /// estimativa se acumularia no bucket.
    pub(in crate::tool::paint) fn spend(&mut self, ms: f32) {
        self.credit_ms = (self.credit_ms - ms).max(WET_MAX_DEBT_MS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Simula o LAÇO FECHADO do produto com um custo de passo FIXO — sem relógio nenhum.
    ///
    /// ⚠️ **Por que unidade e não medição:** a propriedade é *"o recuo não dispara no frame que
    /// contém um passo atômico"*, e sob máquina carregada um passo custa de fato mais, o frame
    /// estoura de verdade e o recuo dispara **com razão** — as faixas do produto correto e da
    /// mutação se sobrepõem, e nenhum limiar de wall-clock as separa. A versão anterior deste gate
    /// media 16,6 ms em repouso e 4,16 sob a suíte inteira, contra 5,2-6,5 da mutação: *flake por
    /// construção*. Aqui o custo do passo é um NÚMERO, então só o controlador varia.
    ///
    /// Devolve o orçamento em regime.
    fn closed_loop(step_ms: f32, frames: usize) -> f32 {
        /// O resto do frame (encode + chrome), do log do produto.
        const OVERHEAD_MS: f32 = 3.0;
        /// O piso do vsync: o frame nunca é mais rápido que isso.
        const VSYNC_MS: f32 = 1000.0 / 60.0;
        let mut b = SimBudget::new();
        let mut dt = VSYNC_MS;
        for _ in 0..frames {
            b.open_frame(dt);
            let tick = if b.can_step() {
                b.spend(step_ms);
                step_ms
            } else {
                0.0
            };
            b.note_tick(tick);
            dt = (OVERHEAD_MS + tick).max(VSYNC_MS);
        }
        b.per_frame_ms
    }

    /// **UM ÚNICO FRAME RÁPIDO NÃO PODE PREGAR A RÉGUA NO PISO** — o quinto smoke.
    ///
    /// Log do Enio (2026-07-29), com o split do tick já no lugar:
    ///
    /// ```text
    /// tool-tick: media 5.44ms pico 49.53ms em 45/120 frames
    /// agua: sim media 28.70ms pico 47.65ms x8 | composite media 1.91ms x8
    /// ```
    ///
    /// **`x8` — oito passos em 120 frames, 4 Hz.** E `28,70 × 8 + 1,91 × 8` fecha exatamente o total
    /// do tick, então os outros 37 ticks não fizeram nada: o orçamento estava em ~1,9 ms, quase no
    /// piso, num app a 60 fps com 14 ms de folga ociosa por frame.
    ///
    /// A causa é a régua. Ela era o `min` do `dt` — escolhido para um frame lento não a levantar — e
    /// isso a fez uma **catraca de mão única**: um `dt` pequeno espúrio (o primeiro frame, um
    /// redraw duplo, um resize) a pregava no piso, e o creep de 0,05%/frame levaria **~70 segundos**
    /// para voltar de 2 a 16,6 ms. O teto é a régua ⇒ orçamento ≈ 2 ms ⇒ água a 4 Hz.
    #[test]
    fn one_fast_frame_does_not_pin_the_ruler_to_the_floor() {
        const VSYNC_MS: f32 = 1000.0 / 60.0;
        let mut b = SimBudget::new();
        // Um único frame espúrio de 1 ms — e depois um segundo inteiro de vsync limpo.
        b.open_frame(1.0);
        b.note_tick(0.0);
        for _ in 0..60 {
            b.open_frame(VSYNC_MS);
            b.note_tick(0.0);
        }
        assert!(
            b.per_frame_ms >= 8.0,
            "um frame de 1 ms pregou a regua: orcamento {:.2} ms depois de 60 frames de vsync \
             limpo (piso 8) — a agua fica a ~4 Hz num app com 14 ms de folga por frame",
            b.per_frame_ms
        );
    }

    /// **UM PASSO ATÔMICO NÃO CATRACA O ORÇAMENTO ATÉ O PISO** — o regime que o Enio reportou três
    /// vezes (*"simulação muitíssimo mais devagar"*, `tool-tick=17.31ms` numa amostra e `0.00` nas
    /// outras).
    ///
    /// Um passo custa mais que um quadro de 60 Hz ⇒ **o frame que o contém estoura por construção**.
    /// Decidindo pelo `dt` instantâneo o recuo dispara em TODO passo e, sendo ×0,5 contra +1 de
    /// subida, o orçamento catraca até o piso.
    #[test]
    fn an_atomic_step_does_not_ratchet_the_budget_to_the_floor() {
        // 17 ms: o `tool-tick` que o log do Enio mostrou, e maior que o quadro.
        let settled = closed_loop(17.0, 400);
        assert!(
            settled >= 12.0,
            "o orcamento CATRACOU ate {settled:.2} ms com um passo de 17 ms (piso 12) — o recuo \
             esta disparando no frame que contem o passo, que estoura por construcao"
        );
    }

    /// **E um passo GENUINAMENTE impagável ainda faz o orçamento recuar** — a outra ponta, no mesmo
    /// laço: se o recuo nunca disparasse, o gate acima ficaria verde com o controlador desarmado.
    #[test]
    fn a_step_the_frame_cannot_afford_still_backs_the_budget_off() {
        let settled = closed_loop(120.0, 400);
        assert!(
            settled < 12.0,
            "o orcamento ficou em {settled:.2} ms com um passo de 120 ms — o recuo nao disparou, e \
             sem ele o gate irmao passa com o controlador desarmado"
        );
    }
}
