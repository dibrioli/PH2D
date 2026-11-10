use std::cell::Cell;

// ⚠️ **POR THREAD, não atômicos globais.** Estes baldes são lidos por um gate que afirma
// `banda > 0`, e poluição só SOMA — um teste vizinho carimbando em paralelo tornaria essa
// afirmação verdadeira mesmo com a fixture tomando a rota serial: **falso VERDE**, a metade
// exata que a flake do `ph2d-painter-brush` (2026-08-01) provou ser a vulnerável. Contador
// por thread torna a poluição estruturalmente impossível e não deixa lista de quem precisa
// se isolar.
//
// ⚠️ **O invariante que isto assume:** quem CARIMBA e quem LÊ são a mesma thread. No app são
// as duas o laço do quadro (o flush coalescido roda antes do `cpu_start`, e o `[frame]` o lê
// no fim). `note` é chamado ANTES do `thread::scope` do lote, então as bandas não contam.
// Se um dia o carimbo sair da thread do quadro, este balde **emudece** — e um balde mudo
// lê-se como resultado; quem mover o carimbo move este contador junto.
thread_local! {
    static BANDED: Cell<u32> = const { Cell::new(0) };
    static SERIAL: Cell<u32> = const { Cell::new(0) };
    static DABS: Cell<u32> = const { Cell::new(0) };
    static VISITS: Cell<u64> = const { Cell::new(0) };
    static CPU_US: Cell<u64> = const { Cell::new(0) };
    static DELIVERIES: Cell<u32> = const { Cell::new(0) };
    static DEVICE: Cell<u32> = const { Cell::new(0) };
    static DEV_DABS: Cell<u32> = const { Cell::new(0) };
    static DEV_VISITS: Cell<u64> = const { Cell::new(0) };
    static DEV_US: Cell<u64> = const { Cell::new(0) };
    static RESTORE_US: Cell<u64> = const { Cell::new(0) };
    static RELIEF_US: Cell<u64> = const { Cell::new(0) };
    static SAVE_US: Cell<u64> = const { Cell::new(0) };
    static STAMP_US: Cell<u64> = const { Cell::new(0) };
}

/// **O lote foi para o DISPOSITIVO?** — o instrumento que a wave da GPU exige pela mesma razão
/// que o `banded`/`serial` existe: sem ele, *"não melhorou"* admite duas leituras opostas (a
/// rota não é tomada · é tomada e o tempo está noutro lugar), e as curas são opostas.
/// ⚠️ **O trabalho do device é contado AQUI, e por isso a linha descreve o TRAÇO.** Quando ele
/// aceita o lote, o `stamp_plain_dabs_banded` nem é chamado — então até 2026-08-04 os `dabs` e as
/// `visitas` do log eram os da **metade que ficou na CPU**, com o log dizendo `775 dabs` para um
/// traço que carimbou mais. Um contador que descreve um subconjunto sem dizer qual é um número
/// que se lê como o todo.
pub(in crate::tool::paint) fn note_device(dabs: usize, work: usize, us: u64) {
    DEVICE.with(|c| c.set(c.get().saturating_add(1)));
    DEV_DABS.with(|c| {
        c.set(
            c.get()
                .saturating_add(u32::try_from(dabs).unwrap_or(u32::MAX)),
        )
    });
    DEV_VISITS.with(|c| c.set(c.get().saturating_add(work as u64)));
    DEV_US.with(|c| c.set(c.get().saturating_add(us)));
}

pub(super) fn note(banded: bool, dabs: usize, work: usize, us: u64) {
    let bucket = if banded { &BANDED } else { &SERIAL };
    bucket.with(|c| c.set(c.get().saturating_add(1)));
    DABS.with(|c| {
        c.set(
            c.get()
                .saturating_add(u32::try_from(dabs).unwrap_or(u32::MAX)),
        )
    });
    VISITS.with(|c| c.set(c.get().saturating_add(work as u64)));
    CPU_US.with(|c| c.set(c.get().saturating_add(us)));
}

/// As QUATRO fases de um quadro de re-stamp, em µs.
///
/// ⚠️ **Elas dividem o dreno com os baldes do lote de propósito.** Descrevem o MESMO evento (uma
/// entrega de ponteiro que re-carimba a figura), e dois drenos publicariam janelas diferentes
/// como se fossem a mesma — a lei do `wash_diag`, que esta sessão já pagou duas vezes.
pub(in crate::tool::paint) fn note_restamp(
    restore_us: u64,
    relief_us: u64,
    save_us: u64,
    stamp_us: u64,
) {
    DELIVERIES.with(|c| c.set(c.get().saturating_add(1)));
    RESTORE_US.with(|c| c.set(c.get().saturating_add(restore_us)));
    RELIEF_US.with(|c| c.set(c.get().saturating_add(relief_us)));
    SAVE_US.with(|c| c.set(c.get().saturating_add(save_us)));
    STAMP_US.with(|c| c.set(c.get().saturating_add(stamp_us)));
}

/// O que o depósito fez desde a última chamada — e ZERA.
///
/// Como ele ZERA, há **um leitor só** por thread (a lei do `wash_diag`): dois leitores
/// publicariam pedaços do mesmo quadro como se fossem quadros.
#[derive(Debug, Clone, Copy, Default)]
pub struct DepositDiag {
    /// Lotes que tomaram a rota em BANDA.
    pub banded: u32,
    /// Lotes carimbados pelo DISPOSITIVO (subconjunto dos que entram no ramo de falloff puro).
    pub device: u32,
    /// Lotes que ficaram seriais (pequenos demais para dividir).
    pub serial: u32,
    /// Dabs somados sobre os lotes que ficaram na **CPU** (banda + serial).
    pub dabs: u32,
    /// **Visitas de texel** da CPU — a soma das pegadas. É o TRABALHO, e não assume raio nenhum.
    pub visits: u64,
    /// µs gastos carimbando na CPU. ⚠️ Numerador e denominador do `ns/visita` da CPU vêm do
    /// **mesmo** evento — a razão anterior dividia os µs do RE-STAMP pelas visitas do DEPÓSITO,
    /// duas populações diferentes, e por isso imprimia `0.0` em toda sessão de mão livre.
    pub cpu_us: u64,
    /// O mesmo trio, para os lotes que o DISPOSITIVO aceitou.
    pub dev_dabs: u32,
    pub dev_visits: u64,
    pub dev_us: u64,
    /// Quadros de re-stamp (entregas que passaram pelo `stamp_drag_preview`).
    pub deliveries: u32,
    /// µs somados em cada fase de um quadro de re-stamp.
    pub restore_us: u64,
    pub relief_us: u64,
    pub save_us: u64,
    pub stamp_us: u64,
}

#[must_use]
pub fn take() -> DepositDiag {
    DepositDiag {
        banded: BANDED.with(Cell::take),
        serial: SERIAL.with(Cell::take),
        device: DEVICE.with(Cell::take),
        dabs: DABS.with(Cell::take),
        visits: VISITS.with(Cell::take),
        cpu_us: CPU_US.with(Cell::take),
        dev_dabs: DEV_DABS.with(Cell::take),
        dev_visits: DEV_VISITS.with(Cell::take),
        dev_us: DEV_US.with(Cell::take),
        deliveries: DELIVERIES.with(Cell::take),
        restore_us: RESTORE_US.with(Cell::take),
        relief_us: RELIEF_US.with(Cell::take),
        save_us: SAVE_US.with(Cell::take),
        stamp_us: STAMP_US.with(Cell::take),
    }
}
