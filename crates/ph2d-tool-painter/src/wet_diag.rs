//! **O split do tick da água, para o log do produto** (`PH2D_FLUID_PROFILE`).
//!
//! Um pico de 71 ms no `tool-tick` não diz **de que** ele é feito: um passo de sim caro ou um
//! composite sobre um casco grande são a mesma linha de log e curas completamente diferentes. Isto
//! acumula as duas metades sobre a mesma janela que o shell imprime, e o shell as lê.
//!
//! ⚠️ **Contadores atômicos e não um canal:** o tick roda na thread de UI e o leitor é o mesmo
//! frame; o custo é uma soma relaxada por passo, e o caminho sem o profiler não paga nada além de
//! duas somas atômicas (medido irrelevante contra um passo de 12-40 ms).
//!
//! # A metade do WORKER (2026-07-29) — e o buraco que ela fecha
//!
//! Com a sim fora da thread do frame, **`sim media` passou a imprimir `0.00ms x0`**: nada mais
//! chamava [`note_step`], porque quem dá o passo é o worker. A linha lia-se como *"a simulação não
//! custa nada"* quando significava *"ninguém mede a simulação"* — e era exatamente o número que
//! decidia se a água lenta é **trabalho** ou **agendamento**.
//!
//! Os três baldes abaixo são a resposta, e eles PARTICIONAM a janela do worker:
//!
//! - **busy** — dentro de `step_stage` (o trabalho da física);
//! - **away** — o motor está com o frame (o worker bloqueado no `recv`);
//! - **sleep** — o ritmo de 40 Hz (`IDLE_SLEEP`), o worker adiantado de propósito.
//!
//! Sobra = o que os três não explicam. E é a leitura DELES que separa três mundos com curas
//! opostas: *busy ≈ 100%* é work-limited (só a GPU move) · *away* grande é o **handshake** custando
//! a taxa · *sleep* grande diz que a água já alcançou o nominal e o gargalo está noutro lugar.

use std::sync::atomic::{AtomicU64, Ordering};

static STEP_US: AtomicU64 = AtomicU64::new(0);
static STEP_MAX_US: AtomicU64 = AtomicU64::new(0);
static STEP_N: AtomicU64 = AtomicU64::new(0);
static COMP_US: AtomicU64 = AtomicU64::new(0);
static COMP_MAX_US: AtomicU64 = AtomicU64::new(0);
static COMP_N: AtomicU64 = AtomicU64::new(0);
static WAIT_US: AtomicU64 = AtomicU64::new(0);
static WAIT_MAX_US: AtomicU64 = AtomicU64::new(0);
static WAIT_N: AtomicU64 = AtomicU64::new(0);

fn add(sum: &AtomicU64, mx: &AtomicU64, n: &AtomicU64, ms: f32) {
    let us = (f64::from(ms) * 1000.0) as u64;
    sum.fetch_add(us, Ordering::Relaxed);
    mx.fetch_max(us, Ordering::Relaxed);
    n.fetch_add(1, Ordering::Relaxed);
}

/// Um `step_simulation` custou isto.
///
/// ⚠️ **É o tempo de COMPUTE do passo, somado sobre os estágios dele** — nunca o
/// wall-clock de ponta a ponta, que inclui o motor viajando para o frame e
/// voltar. Os dois números existem e respondem perguntas diferentes: este diz
/// *quanto a física custa*; o span diz *quanto tempo a água levou*, e a diferença
/// entre eles é o [`note_away`].
pub fn note_step(ms: f32) {
    add(&STEP_US, &STEP_MAX_US, &STEP_N, ms);
}

/// Um `wetpaint_composite` custou isto.
pub fn note_composite(ms: f32) {
    add(&COMP_US, &COMP_MAX_US, &COMP_N, ms);
}

/// O tick ESPEROU isto pelo motor (`try_bring_home`) — a 3ª metade, e a única
/// que a sim off-thread criou. Ela existe porque a aritmética do log do produto
/// (tick 308 ms, composite 97,7) deixou **210 ms sem dono**, e atribuir sem medir
/// é a doença do doc 28 §5.13.
pub fn note_wait(ms: f32) {
    add(&WAIT_US, &WAIT_MAX_US, &WAIT_N, ms);
}

// ---------------------------------------------------------------------------
// A metade do WORKER — os três baldes que particionam a janela dele
// ---------------------------------------------------------------------------

static BUSY_US: AtomicU64 = AtomicU64::new(0);
static AWAY_US: AtomicU64 = AtomicU64::new(0);
static SLEEP_US: AtomicU64 = AtomicU64::new(0);
/// O instante em que o motor SAIU (0 = está em casa) — ver [`note_away_open`].
static AWAY_OPEN_US: AtomicU64 = AtomicU64::new(0);
static CELLS: AtomicU64 = AtomicU64::new(0);
static CELLS_N: AtomicU64 = AtomicU64::new(0);

/// O worker gastou isto DENTRO de um `step_stage` (o trabalho da física).
pub fn note_busy(us: u64) {
    BUSY_US.fetch_add(us, Ordering::Relaxed);
}

/// O relógio comum dos intervalos abertos — micros desde a 1ª leitura.
///
/// `Instant` não cabe num atômico, e o `away` precisa ser lido pela thread do
/// FRAME enquanto o worker ainda o está medindo (abaixo).
fn now_us() -> u64 {
    static BASE: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    BASE.get_or_init(std::time::Instant::now)
        .elapsed()
        .as_micros() as u64
}

/// O motor FOI para o frame — abre o intervalo de `away`.
///
/// ⚠️ **Um intervalo aberto pertence à janela em que ele está ABERTO, e não
/// àquela em que ele fecha** — foi assim que o log do Enio de 2026-07-31
/// trouxe `busy 39% away 161% sleep 29%`, somando **229%** numa linha que diz
/// PARTIÇÃO. O motor tinha ficado com o frame por mais tempo que a janela
/// inteira (uma rajada de carimbos), e a versão anterior creditava as quatro
/// segundos inteiros à janela onde o `recv` voltou.
///
/// É a MESMA classe do `sleep 909%` que a wave anterior curou, por outra via:
/// lá a janela era assumida, aqui o intervalo atravessa a janela.
pub fn note_away_open() {
    AWAY_OPEN_US.store(now_us().max(1), Ordering::Release);
}

/// O motor VOLTOU — fecha o intervalo, creditando só o que ainda não foi.
///
/// ⚠️ O `swap` é o que torna isto correto contra o leitor concorrente: quem
/// tirar o timestamp é quem credita aquele trecho, e o outro lado não o vê
/// mais (nem duplica, nem perde).
pub fn note_away_closed() {
    let t = AWAY_OPEN_US.swap(0, Ordering::AcqRel);
    if t != 0 {
        AWAY_US.fetch_add(now_us().saturating_sub(t), Ordering::Relaxed);
    }
}

/// O worker dormiu isto porque estava ADIANTADO (o ritmo de 40 Hz da SPEC).
pub fn note_sleep(us: u64) {
    SLEEP_US.fetch_add(us, Ordering::Relaxed);
}

/// `(busy, away, sleep)` em ms, ZERANDO a janela.
///
/// ⚠️ **Credita primeiro a parte ABERTA do `away`** e re-baseia o intervalo:
/// sem isto uma retenção mais longa que a janela cai INTEIRA na janela onde
/// termina, e a linha que diz *partição* soma 229% (o log de 2026-07-31).
///
/// O CAS é o que impede a contagem dupla: se o worker fechou o intervalo entre
/// a leitura e a troca, ele já creditou aquele trecho e o CAS falha — nada é
/// somado aqui, e nada se perde.
///
/// ⚠️ **E ele é uma defesa DOCUMENTADA e não gateada, de propósito:** a mutação
/// que o troca por um `store` **não sangra**, porque a contagem dupla só existe
/// sob corrida real com o worker e um teste single-threaded não a produz. É o
/// precedente que esta linha já usou (ADR-0145: *duas defesas em camada
/// documentadas em vez de gateadas — no regime que shipa não são
/// observáveis*), e escrevê-lo aqui é o que impede a próxima pessoa de
/// "simplificar" o CAS achando que a suíte verde a autoriza.
#[must_use]
pub fn take_worker() -> (f64, f64, f64) {
    let open = AWAY_OPEN_US.load(Ordering::Acquire);
    if open != 0 {
        let n = now_us().max(1);
        if AWAY_OPEN_US
            .compare_exchange(open, n, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
        {
            AWAY_US.fetch_add(n.saturating_sub(open), Ordering::Relaxed);
        }
    }
    let take = |c: &AtomicU64| c.swap(0, Ordering::Relaxed) as f64 / 1000.0;
    (take(&BUSY_US), take(&AWAY_US), take(&SLEEP_US))
}

/// **O TAMANHO DA POÇA sobre a qual o passo foi dado** — o divisor sem o qual
/// *"a água está a 17,7 Hz"* não é atribuível.
///
/// ⚠️ **Este balde nasceu de três hipóteses REJEITADAS por medição**
/// (2026-07-31, doc 28 §5.47): o log do smoke mostrou o passo a **45,52 ms
/// pintando** contra **20,11 ms só assistindo**, com o `busy` do worker
/// praticamente igual nas duas janelas — ele trabalha o mesmo e entrega
/// metade dos passos. Duas leituras cabem nisso e pedem curas OPOSTAS —
/// *a poça ficou maior* (não há nada a consertar; o slider `Grid Size` é a
/// resposta) contra *a máquina ficou disputada* (o solver é core-limited,
/// 11,62× sob contenção) — e **o log não as distingue**, porque as duas
/// metades se movem juntas nele.
///
/// Com o tamanho ao lado do custo a pergunta some: `ns/célula` constante entre
/// as janelas é TRABALHO; `ns/célula` subindo é CONTENÇÃO.
pub fn note_cells(n: u64) {
    CELLS.fetch_add(n, Ordering::Relaxed);
    CELLS_N.fetch_add(1, Ordering::Relaxed);
}

/// A poça MÉDIA da janela em células (0 se nenhum passo foi dado), ZERANDO.
#[must_use]
pub fn take_cells() -> u64 {
    let (sum, n) = (
        CELLS.swap(0, Ordering::Relaxed),
        CELLS_N.swap(0, Ordering::Relaxed),
    );
    sum.checked_div(n).unwrap_or(0)
}

/// Uma metade do tick, na janela: `(soma ms, pico ms, n)`.
pub type Half = (f64, f64, u64);

/// `(soma ms, pico ms, n)` de cada metade, ZERANDO a janela — o leitor é o `[frame]` do shell.
#[must_use]
pub fn take_window() -> (Half, Half, Half) {
    let take = |sum: &AtomicU64, mx: &AtomicU64, n: &AtomicU64| {
        (
            sum.swap(0, Ordering::Relaxed) as f64 / 1000.0,
            mx.swap(0, Ordering::Relaxed) as f64 / 1000.0,
            n.swap(0, Ordering::Relaxed),
        )
    };
    (
        take(&STEP_US, &STEP_MAX_US, &STEP_N),
        take(&COMP_US, &COMP_MAX_US, &COMP_N),
        take(&WAIT_US, &WAIT_MAX_US, &WAIT_N),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **UM INTERVALO ABERTO É CREDITADO À JANELA EM QUE ELE ESTÁ ABERTO — E
    /// NADA É CONTADO DUAS VEZES.**
    ///
    /// ⚠️ Gate nascido de um log: `busy 39% away 161% sleep 29%` — soma **229%**
    /// numa linha que diz PARTIÇÃO. O motor ficara com o frame por mais tempo
    /// que a janela inteira (uma rajada de carimbos), e a versão anterior
    /// creditava o intervalo INTEIRO à janela onde o `recv` voltou.
    ///
    /// ⚠️ **UM teste, e não dois, porque os baldes são GLOBAIS.** A 1ª versão
    /// eram dois gates e eles se drenaram mutuamente — a lição que o
    /// `the_worker_reports_what_a_step_costs` já pregava (*ele é o único teste
    /// não-`#[ignore]` que consome a janela global; um 2º leitor zeraria a dele
    /// e o verde viraria sorte*), violada um arquivo adiante, com um
    /// doc-comment meu afirmando o contrário.
    ///
    /// Duas mutações sangram: creditar só no `note_away_closed` (a 1ª metade
    /// vira 0) · creditar sem o CAS de `take_worker` (a soma passa o intervalo).
    #[test]
    fn an_open_away_is_credited_to_the_window_it_spans_and_never_twice() {
        let _ = take_worker(); // zera o que houver
        let t0 = std::time::Instant::now();
        note_away_open();
        std::thread::sleep(std::time::Duration::from_millis(30));
        let (_, first, _) = take_worker();
        std::thread::sleep(std::time::Duration::from_millis(30));
        note_away_closed();
        let (_, second, _) = take_worker();
        let real = t0.elapsed().as_secs_f64() * 1e3;
        assert!(
            first >= 20.0,
            "a janela que CONTEM o intervalo aberto recebeu {first:.1} ms: um `away` mais \
             longo que a janela volta a cair inteiro na janela onde fecha, e a linha de \
             particao volta a somar mais de 100%"
        );
        assert!(
            second >= 20.0,
            "a 2a janela recebeu {second:.1} ms: o resto do intervalo tem de ser creditado \
             onde ele de fato correu"
        );
        assert!(
            first + second <= real * 1.25 + 5.0,
            "as duas janelas somam {:.1} ms para um intervalo de {real:.1}: alguma parte \
             foi creditada duas vezes",
            first + second
        );
    }
}
