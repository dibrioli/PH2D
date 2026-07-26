//! Gates da frente **L0** do plano 26 — o relógio `evento → frame`.
//!
//! ⚠️ O estado do agregador é `thread_local` e cada teste do Rust roda na própria thread, então o
//! `ON` cacheado e o `AGG` são privados a cada gate — nenhum contamina o outro.

use super::{FrameInfo, end_frame, force_on, latency_samples, pq, record_dispatch, stamp_pointer};

/// Arma o agregador para ESTA thread (sem tocar no ambiente — ver [`super::force_on`]).
fn arm() {
    force_on();
}

/// **O relógio mede o evento MAIS ANTIGO não-servido, nunca o mais recente.**
///
/// Entre dois frames chegam vários eventos de ponteiro; o atraso que o artista percebe é o do
/// PRIMEIRO deles, porque é ele que está esperando há mais tempo. Carimbar o último mediria a
/// latência do evento mais sortudo do lote.
///
/// ⚠️ **Mutação que deve sangrar:** `stamp_pointer` escrevendo `Some(Instant::now())` incondicional
/// (em vez de só quando não há pendente) — a latência cai para ~0 e o gate falha.
#[test]
fn the_oldest_unserved_event_is_the_one_the_clock_measures() {
    arm();
    stamp_pointer();
    std::thread::sleep(std::time::Duration::from_millis(12));
    stamp_pointer(); // um evento mais NOVO no mesmo frame: não pode roubar o relógio
    record_dispatch(FrameInfo::default());
    end_frame(1.0);
    let got = latency_samples();
    assert_eq!(
        got.len(),
        1,
        "um frame consumiu um lote e produziu uma amostra"
    );
    assert!(
        got[0] >= 11.0,
        "a latencia medida foi {:.1} ms — o relogio pegou o evento NOVO em vez do mais antigo, e \
         reporta o atraso do evento mais sortudo do lote",
        got[0]
    );
}

/// **Um frame sem evento pendente não inventa amostra** — e o pendente não sobrevive ao frame que o
/// serviu (senão o próximo frame reportaria a latência de um evento já mostrado, crescendo sozinha).
#[test]
fn a_frame_with_nothing_pending_records_nothing() {
    arm();
    stamp_pointer();
    record_dispatch(FrameInfo::default());
    end_frame(1.0);
    record_dispatch(FrameInfo::default());
    end_frame(1.0); // nenhum evento novo
    assert_eq!(
        latency_samples().len(),
        1,
        "o segundo frame inventou uma amostra: o pendente sobreviveu ao frame que o serviu"
    );
}

/// **A latência se julga pela CAUDA** — o `pq` tem de ser o quantil que afirma ser.
///
/// Uma mediana de 8 ms com um p95 de 40 é o que o artista descreve como *"às vezes trava"*, e a
/// mediana sozinha diz que está tudo bem.
#[test]
fn the_quantile_is_the_quantile_it_claims() {
    let v: Vec<f32> = (0..=100).map(|i| i as f32).collect();
    assert!((pq(&v, 0.5) - 50.0).abs() < 1e-3);
    assert!((pq(&v, 0.95) - 95.0).abs() < 1e-3);
    assert!((pq(&v, 1.0) - 100.0).abs() < 1e-3);
    assert!((pq(&v, 0.0) - 0.0).abs() < 1e-3);
    assert!(
        (pq(&[], 0.5) - 0.0).abs() < 1e-3,
        "uma janela vazia nao pode indexar fora"
    );
}

/// **O contador de eventos conta TODOS, não só os que abrem um lote.**
///
/// É ele que distingue as duas causas de uma latência alta, e elas têm curas opostas: ~1 evento por
/// frame significa que o **pipeline** demora; muitos significam que eles **enfileiram**. A 1ª leitura
/// real deu `p50 34,3 ms` sobre um `frame p50` de 4,7 — aritmética que só fecha se uma das duas for
/// verdade, e o relatório não dizia qual.
///
/// ⚠️ **Mutação que deve sangrar:** contar dentro do `if a.pending.is_none()` — o contador passaria a
/// medir LOTES, que é exatamente o número que o relatório já tinha (`n`).
#[test]
fn the_event_counter_counts_every_event_not_every_batch() {
    arm();
    stamp_pointer();
    stamp_pointer();
    stamp_pointer();
    assert_eq!(
        super::events_seen(),
        3,
        "o contador registrou lotes, nao eventos — e lotes o relatorio ja contava em `n`"
    );
}
