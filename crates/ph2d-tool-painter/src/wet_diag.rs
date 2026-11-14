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
// O COMPOSITOR DA ÓPTICA fora do fio da tela (J13b P5) — a cópia para o espelho, a óptica, as imagens
// ---------------------------------------------------------------------------

static COPIA_US: AtomicU64 = AtomicU64::new(0);
static COPIA_MAX_US: AtomicU64 = AtomicU64::new(0);
static COPIA_N: AtomicU64 = AtomicU64::new(0);
static OTICA_US: AtomicU64 = AtomicU64::new(0);
static OTICA_MAX_US: AtomicU64 = AtomicU64::new(0);
static OTICA_N: AtomicU64 = AtomicU64::new(0);

/// O worker copiou as linhas sujas para o espelho em `ms` (no fim de um passo fechado).
pub fn note_copia(ms: f32) {
    add(&COPIA_US, &COPIA_MAX_US, &COPIA_N, ms);
}

/// O compositor fez UMA imagem em `ms` (a metade de cima sobre o espelho): `n` é o número de imagens.
pub fn note_otica(ms: f32) {
    add(&OTICA_US, &OTICA_MAX_US, &OTICA_N, ms);
}

/// `(cópia, óptica)` da janela, `(soma ms, pico ms, n)`, ZERANDO; `imagens/s` = `óptica.n ÷ janela`.
#[must_use]
pub fn take_otica() -> (Half, Half) {
    let take = |sum: &AtomicU64, mx: &AtomicU64, n: &AtomicU64| {
        (
            sum.swap(0, Ordering::Relaxed) as f64 / 1000.0,
            mx.swap(0, Ordering::Relaxed) as f64 / 1000.0,
            n.swap(0, Ordering::Relaxed),
        )
    };
    (
        take(&COPIA_US, &COPIA_MAX_US, &COPIA_N),
        take(&OTICA_US, &OTICA_MAX_US, &OTICA_N),
    )
}

/// `2` = ainda por ler do ambiente.
static OTICA_FORA: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(2);

/// **A óptica sai do fio da tela?** Ligada por omissão (J13 §6: o quadro parado 72–82 → 16,6 ms); `PH2D_WET_OTICA_FORA=0`
/// volta ao caminho de casa. Lida do ambiente UMA vez;
/// [`set_otica_fora`] troca-a em execução para as variantes intercaladas no mesmo processo (MEDIR_VELOCIDADE). Vale
/// para os workers que sobem DEPOIS (o compositor nasce com o worker da sessão).
pub fn otica_fora() -> bool {
    match OTICA_FORA.load(Ordering::Relaxed) {
        2 => {
            let on = std::env::var("PH2D_WET_OTICA_FORA").map_or(true, |v| v.trim() != "0");
            OTICA_FORA.store(u8::from(on), Ordering::Relaxed);
            on
        }
        v => v == 1,
    }
}

/// A variante em execução de [`otica_fora`].
pub fn set_otica_fora(on: bool) {
    OTICA_FORA.store(u8::from(on), Ordering::Relaxed);
}

/// `2` = ainda por ler do ambiente.
static OTICA_RGBA_SUAVE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(2);

/// **O compositor devolve os RGBA com Smooth Edges e razão > 1?** (B11.3) Ligado por omissão;
/// `PH2D_OTICA_COMPOSITOR_SMOOTH=0` volta aos filtros antigos (razão 1, sem suave/sombra) para o A×B no mesmo processo.
pub fn otica_rgba_suave() -> bool {
    match OTICA_RGBA_SUAVE.load(Ordering::Relaxed) {
        2 => {
            let on =
                std::env::var("PH2D_OTICA_COMPOSITOR_SMOOTH").map_or(true, |v| v.trim() != "0");
            OTICA_RGBA_SUAVE.store(u8::from(on), Ordering::Relaxed);
            on
        }
        v => v == 1,
    }
}

/// A variante em execução de [`otica_rgba_suave`] (lida por imagem no fio do compositor).
pub fn set_otica_rgba_suave(on: bool) {
    OTICA_RGBA_SUAVE.store(u8::from(on), Ordering::Relaxed);
}

// ---------------------------------------------------------------------------
// A metade do WORKER — os três baldes que particionam a janela dele
// ---------------------------------------------------------------------------

static BUSY_US: AtomicU64 = AtomicU64::new(0);
static SLEEP_US: AtomicU64 = AtomicU64::new(0);
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
    AWAY.open(now_us());
}

/// O motor VOLTOU — fecha o intervalo, creditando só o que ainda não foi.
///
/// ⚠️ O `swap` é o que torna isto correto contra o leitor concorrente: quem
/// tirar o timestamp é quem credita aquele trecho, e o outro lado não o vê
/// mais (nem duplica, nem perde).
pub fn note_away_closed() {
    AWAY.close(now_us());
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
    let take = |c: &AtomicU64| c.swap(0, Ordering::Relaxed) as f64 / 1000.0;
    (
        take(&BUSY_US),
        AWAY.drain(now_us()) as f64 / 1000.0,
        take(&SLEEP_US),
    )
}

/// **A CONTABILIDADE DE UM INTERVALO QUE ATRAVESSA JANELAS** — um tipo, e não
/// um par de `static`s, porque é o único jeito de ela ter gate.
///
/// ⚠️ **A 1ª versão era um par de estáticos e o gate dela FLAKAVA**, por um
/// motivo que uma trava entre os dois gates não resolve: o slot do intervalo
/// aberto é **UM**, e os workers de ~900 outros testes da suíte o escrevem
/// enquanto o gate mede. *Um gate sobre um singleton que o produto dirige em
/// paralelo não é um gate — é uma corrida com asserção.*
///
/// Aqui a aritmética é do TIPO; o produto usa uma instância `static` e o gate
/// usa a dele. Nenhuma cópia: é o mesmo código.
#[derive(Default)]
pub struct AwayLedger {
    /// O instante em que o motor SAIU (0 = está em casa).
    open: AtomicU64,
    /// O que já foi creditado à janela corrente.
    total: AtomicU64,
}

impl AwayLedger {
    pub const fn new() -> Self {
        Self {
            open: AtomicU64::new(0),
            total: AtomicU64::new(0),
        }
    }

    /// O motor FOI para o frame — abre o intervalo.
    ///
    /// ⚠️ **Um intervalo aberto pertence à janela em que ele está ABERTO, e não
    /// àquela em que fecha** — foi assim que o log do Enio de 2026-07-31 trouxe
    /// `busy 39% away 161% sleep 29%`, somando **229%** numa linha que diz
    /// PARTIÇÃO. O motor tinha ficado com o frame por mais tempo que a janela
    /// inteira (uma rajada de carimbos), e a versão anterior creditava os quatro
    /// segundos inteiros à janela onde o `recv` voltou.
    ///
    /// É a MESMA classe do `sleep 909%` que a wave anterior curou, por outra
    /// via: lá a janela era assumida, aqui o intervalo atravessa a janela.
    pub fn open(&self, now: u64) {
        self.open.store(now.max(1), Ordering::Release);
    }

    /// O motor VOLTOU — fecha o intervalo, creditando só o que ainda não foi.
    ///
    /// ⚠️ O `swap` é o que torna isto correto contra o leitor concorrente: quem
    /// tirar o timestamp é quem credita aquele trecho, e o outro lado não o vê
    /// mais (nem duplica, nem perde).
    pub fn close(&self, now: u64) {
        let t = self.open.swap(0, Ordering::AcqRel);
        if t != 0 {
            self.total
                .fetch_add(now.saturating_sub(t), Ordering::Relaxed);
        }
    }

    /// Drena a janela, **creditando primeiro a parte ABERTA** e re-baseando.
    ///
    /// ⚠️ O CAS é o que impede a contagem dupla: se o worker fechou o intervalo
    /// entre a leitura e a troca, ele já creditou aquele trecho e o CAS falha —
    /// nada é somado aqui, e nada se perde. **Ele é uma defesa DOCUMENTADA e
    /// não gateada**: a mutação que o troca por um `store` não sangra, porque a
    /// contagem dupla só existe sob corrida real (precedente do ADR-0145).
    pub fn drain(&self, now: u64) -> u64 {
        let open = self.open.load(Ordering::Acquire);
        if open != 0 {
            let n = now.max(1);
            if self
                .open
                .compare_exchange(open, n, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                self.total
                    .fetch_add(n.saturating_sub(open), Ordering::Relaxed);
            }
        }
        self.total.swap(0, Ordering::Relaxed)
    }
}

static AWAY: AwayLedger = AwayLedger::new();

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

// ---------------------------------------------------------------------------
// A metade do GESTO — o que o carimbo custa enquanto a caneta está em baixo
// ---------------------------------------------------------------------------
//
// ⚠️ **Com a caneta em baixo a sim está PARADA** (`sim_should_run` é `false` sob o gesto), logo os
// três baldes acima leem zero durante o traço — e é exactamente onde o artista sente o Wet Paint
// pesar (medido 2026-09-24, `mede_o_wet_paint`: raio 250 ⇒ `~31 ms` por quadro, todos nos eventos
// do ponteiro). Sem esta metade o log do produto não sabia dizer se um quadro lento de pintar é o
// DEPÓSITO dos dabs ou o COMPOSITE que o carimbo faz a seguir.

/// `false` = o lote e a composição do Wet correm no fio de quem chama, como antes de R3 (a variante da sonda
/// `examples/mede_o_dab.rs`; o produto deixa-o `true`).
pub static NO_POOL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

static STAMP_DABS: AtomicU64 = AtomicU64::new(0);
static STAMP_DEP_US: AtomicU64 = AtomicU64::new(0);
static STAMP_COMP_US: AtomicU64 = AtomicU64::new(0);
static STAMP_N: AtomicU64 = AtomicU64::new(0);

/// Uma entrega de dabs ao motor: quantos, o depósito deles e o composite que se lhe seguiu.
pub fn note_stamp(dabs: usize, deposit_ms: f32, composite_ms: f32) {
    STAMP_DABS.fetch_add(dabs as u64, Ordering::Relaxed);
    STAMP_DEP_US.fetch_add((f64::from(deposit_ms) * 1000.0) as u64, Ordering::Relaxed);
    STAMP_COMP_US.fetch_add((f64::from(composite_ms) * 1000.0) as u64, Ordering::Relaxed);
    STAMP_N.fetch_add(1, Ordering::Relaxed);
}

/// Liga a contagem da silhueta do dab (B14c; só a sonda `mede_o_dab` a liga — desligada custa uma leitura relaxada).
pub static CONTA_SIL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static SIL: [AtomicU64; 6] = [const { AtomicU64::new(0) }; 6];
/// A variante A da sonda (`mede_o_dab`, `SEM_CORTE=1`): toda célula da caixa da silhueta avaliada, sem o corte além da
/// pegada — o A/B em instruções no MESMO binário. Lida 1× por dab.
pub static SIL_SEM_CORTE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Uma leitura da silhueta, no nível `k`: `0` a do motor · `1` a da ponta (memo falhada) · `2` a base · `3` os
/// sub-pontos do AA da base · `4` as do motor FORA da caixa da memo · `5` as da base que dão `0`.
#[inline]
pub fn conta_sil(k: usize, n: u64) {
    if CONTA_SIL.load(Ordering::Relaxed) {
        SIL[k].fetch_add(n, Ordering::Relaxed);
    }
}

/// As quatro contagens de [`conta_sil`] desde a última leitura, ZERANDO.
#[must_use]
pub fn take_sil() -> [u64; 6] {
    [0, 1, 2, 3, 4, 5].map(|k| SIL[k].swap(0, Ordering::Relaxed))
}

/// `(dabs, depósito ms, composite ms, entregas)` desde a última leitura, ZERANDO.
#[must_use]
pub fn take_stamp() -> (u64, f64, f64, u64) {
    let ms = |c: &AtomicU64| c.swap(0, Ordering::Relaxed) as f64 / 1000.0;
    (
        STAMP_DABS.swap(0, Ordering::Relaxed),
        ms(&STAMP_DEP_US),
        ms(&STAMP_COMP_US),
        STAMP_N.swap(0, Ordering::Relaxed),
    )
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

/// `0` sem factos ainda · `1` repouso · `2` o pincel na tela (o último `factos_da_otica`).
static NA_TELA: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
static FACTOS_N: AtomicU64 = AtomicU64::new(0);

/// Os factos da óptica de um tique: o pincel na tela?
pub fn note_factos(na_tela: bool) {
    NA_TELA.store(1 + u8::from(na_tela), Ordering::Relaxed);
    FACTOS_N.fetch_add(1, Ordering::Relaxed);
}

/// As fases da óptica (gesto · repouso) por ordem, para o log da cena: uma linha por fase que FECHA.
#[derive(Debug, Default)]
pub struct FasesDaOtica {
    fase: Option<(bool, u32, std::time::Instant)>,
    gestos: u32,
}

impl FasesDaOtica {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            fase: None,
            gestos: 0,
        }
    }

    /// `na_tela` = a fase de agora; se mudou, a linha da que fechou com `(cópia, óptica)` e os factos dela.
    pub fn vira(
        &mut self,
        na_tela: bool,
        agora: std::time::Instant,
        otica: (Half, Half),
        factos: u64,
    ) -> Option<String> {
        let fechou = self.fase.replace((na_tela, 0, agora));
        if na_tela {
            self.gestos += 1;
        }
        let (era, _, desde) = fechou?;
        let k = self.gestos - u32::from(na_tela);
        let ((c_ms, c_max, c_n), (o_ms, o_max, o_n)) = otica;
        let s = (agora - desde).as_secs_f64();
        let media = |soma: f64, n: u64| soma / n.max(1) as f64;
        Some(format!(
            "[otica] {} {k}: {o_n} imagens em {s:.2} s ({:.1}/s) · {factos} factos · óptica {:.1}/{:.1} ms · cópia {:.2}/{:.2} ms ({c_n})",
            if era { "gesto" } else { "repouso" },
            o_n as f64 / s.max(1e-3),
            media(o_ms, o_n),
            o_max,
            media(c_ms, c_n),
            c_max,
        ))
    }
}

/// Por quadro (a cena Composite): `Some(linha)` quando a fase da óptica muda; lê e ZERA `take_otica` e os factos.
pub fn fase_da_otica(fases: &mut FasesDaOtica) -> Option<String> {
    let na_tela = match NA_TELA.load(Ordering::Relaxed) {
        0 => return None,
        v => v == 2,
    };
    if fases.fase.is_some_and(|(f, ..)| f == na_tela) {
        return None;
    }
    let factos = FACTOS_N.swap(0, Ordering::Relaxed);
    fases.vira(na_tela, std::time::Instant::now(), take_otica(), factos)
}

#[cfg(test)]
mod tests {
    /// B14b-cont2: cada fase que fecha dá UMA linha com as imagens dela, as imagens/s e os factos; a 1.ª fase
    /// só abre (o que havia antes dela não é de nenhum gesto).
    #[test]
    fn a_fase_da_otica_conta_as_imagens_de_cada_gesto() {
        use std::time::{Duration, Instant};
        let mut f = super::FasesDaOtica::default();
        let t0 = Instant::now();
        let nada = ((0.0, 0.0, 0), (0.0, 0.0, 0));
        assert_eq!(f.vira(true, t0, nada, 0), None, "a 1.ª fase só abre");
        let gesto = ((4.0, 1.5, 8), (400.0, 90.0, 8));
        let l = f
            .vira(false, t0 + Duration::from_secs(2), gesto, 120)
            .expect("o gesto fecha");
        assert!(
            l.contains("gesto 1:") && l.contains(" 8 imagens") && l.contains("4.0/s"),
            "{l}"
        );
        assert!(
            l.contains("120 factos") && l.contains("óptica 50.0/90.0"),
            "{l}"
        );
        let repouso = ((0.0, 0.0, 0), (30.0, 30.0, 1));
        let l = f
            .vira(true, t0 + Duration::from_secs(4), repouso, 60)
            .expect("o repouso fecha");
        assert!(
            l.contains("repouso 1:") && l.contains(" 1 imagens") && l.contains("60 factos"),
            "{l}"
        );
    }

    use super::*;

    /// **UM INTERVALO ABERTO É CREDITADO À JANELA EM QUE ELE ESTÁ ABERTO — E
    /// NADA É CONTADO DUAS VEZES.**
    ///
    /// ⚠️ Gate nascido de um log: `busy 39% away 161% sleep 29%` — soma **229%**
    /// numa linha que diz PARTIÇÃO.
    ///
    /// ⚠️ **E ele testa uma instância PRÓPRIA do [`AwayLedger`], nunca o
    /// `static` do produto.** A 1ª versão media os estáticos e **FLAKAVA**: o
    /// slot do intervalo aberto é UM, e os workers de ~900 outros testes da
    /// suíte o escrevem enquanto o gate mede. Uma trava entre os dois gates não
    /// resolvia — os escritores não são gates. *Um gate sobre um singleton que
    /// o produto dirige em paralelo não é um gate, é uma corrida com asserção.*
    ///
    /// O relógio é INJETADO pela mesma razão: `now_us()` é tempo de parede, e um
    /// gate que dorme para produzir um número aposta na velocidade da máquina.
    /// Com o relógio na mão o teste é exato e determinístico.
    ///
    /// Mutações que sangram: creditar só no `close` (a 1ª janela vira 0) ·
    /// re-basear sem creditar (a 2ª janela recebe o intervalo inteiro).
    #[test]
    fn an_open_away_is_credited_to_the_window_it_spans_and_never_twice() {
        let led = AwayLedger::new();
        led.open(1_000);
        // Janela 1 fecha com o intervalo AINDA aberto: ela leva o que correu.
        assert_eq!(
            led.drain(1_030),
            30,
            "a janela que CONTEM o intervalo aberto tem de receber o trecho que correu nela — senao um `away` mais longo que a janela volta a cair inteiro na janela onde fecha, e a particao volta a somar mais de 100%"
        );
        led.close(1_070);
        // Janela 2 leva o RESTO, e só o resto.
        assert_eq!(
            led.drain(1_090),
            40,
            "a 2a janela tem de receber exatamente o resto do intervalo: nem menos (perda) nem o total (contagem dupla)"
        );
        // E nada sobra depois de fechado.
        assert_eq!(
            led.drain(1_200),
            0,
            "um intervalo fechado nao pode continuar creditando"
        );
    }

    /// **O intervalo que nunca abriu não credita nada** — o controle, sem o
    /// qual o gate acima passaria com um ledger que soma sozinho.
    #[test]
    fn a_ledger_that_never_opened_credits_nothing() {
        let led = AwayLedger::new();
        assert_eq!(led.drain(5_000), 0);
        led.close(6_000);
        assert_eq!(led.drain(7_000), 0);
    }
}
