//! Gates da frente **L0** do plano 26 — o relógio `evento → frame`.
//!
//! ⚠️ O estado do agregador é `thread_local` e cada teste do Rust roda na própria thread, então o
//! `ON` cacheado e o `AGG` são privados a cada gate — nenhum contamina o outro.

use super::{
    FrameInfo, InputPhase, end_frame, force_on, latency_samples, pq, record_dispatch, record_input,
    stamp_pointer,
};

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

/// **O custo do `on_canvas_pointer` é acumulado POR FRAME, e zerado a cada frame.**
///
/// Ele mede o trabalho que acontece **fora** do `run_render_frame` — o `PaintFrameTimer` cobre só
/// aquele escopo, e carimbar dabs roda no handler de input do winit. Foi por isso que a 1ª leitura com
/// período real mostrou `frame p50 16,7 ms` contra **99,9 ms de período**: 83 ms por frame que nenhum
/// dos 17 sub-slots do relatório enxergava.
///
/// ⚠️ **Mutação que deve sangrar:** não zerar o acumulador no `end_frame` (`a.input_ms` em vez de
/// `std::mem::take`) — ele viraria um total corrido e o p50 do relatório subiria sozinho a cada frame.
#[test]
fn the_input_cost_is_per_frame_not_a_running_total() {
    arm();
    record_input(4.0, InputPhase::Move);
    record_input(6.0, InputPhase::Move);
    record_dispatch(FrameInfo::default());
    end_frame(1.0);
    record_dispatch(FrameInfo::default());
    end_frame(1.0); // um frame SEM evento nenhum
    let hist = super::input_hist();
    assert_eq!(hist.len(), 2);
    assert!(
        (hist[0] - 10.0).abs() < 1e-3,
        "o 1o frame somou {:.1} ms em vez dos 10 que recebeu",
        hist[0]
    );
    assert!(
        hist[1].abs() < 1e-3,
        "o 2o frame nao recebeu evento nenhum e mesmo assim reportou {:.1} ms — o acumulador virou \
         um total corrido, e o p50 do relatorio sobe sozinho a cada frame",
        hist[1]
    );
}

/// **As três fases não se misturam** — o divisor que o log de 2026-08-01 cobrou.
///
/// `INPUT p50=0,0 max=1016,5` admitia *"um pen-up custa um segundo"* e *"um move custa um segundo"*,
/// que pedem curas OPOSTAS. Um balde só não pode responder, e três baldes que somam no mesmo lugar
/// são um balde só com três nomes — daí o gate ser sobre a SEPARAÇÃO, não sobre o total.
///
/// ⚠️ **Mutação que deve sangrar:** indexar o balde por uma constante (todo evento em `Move`).
#[test]
fn the_input_cost_is_split_by_phase_so_a_slow_pen_up_names_itself() {
    arm();
    record_input(1.0, InputPhase::Down);
    record_input(2.0, InputPhase::Move);
    record_input(64.0, InputPhase::Up);
    record_dispatch(FrameInfo::default());
    end_frame(1.0);
    let (d, m, u) = super::input_hist_by_phase();
    assert_eq!(
        (d.len(), m.len(), u.len()),
        (1, 1, 1),
        "cada fase mantem o proprio historico por frame"
    );
    assert!(
        (d[0] - 1.0).abs() < 1e-3 && (m[0] - 2.0).abs() < 1e-3 && (u[0] - 64.0).abs() < 1e-3,
        "as fases se misturaram: down={:.1} move={:.1} up={:.1} — um pen-up caro fica indistinguivel \
         de um move caro, e as duas curas sao opostas",
        d[0],
        m[0],
        u[0]
    );
    // …e o leitor agregado segue somando as tres, que e' o que ele sempre quis dizer.
    let all = super::input_hist();
    assert!(
        (all[0] - 67.0).abs() < 1e-3,
        "a soma das fases e' {:.1}",
        all[0]
    );
}
