//! Os gates do `pulse.beat` — irmão do [`super`] pelo tecto de LOC (HR-18).
//!
//! ⚠️ O corte é por RESPONSABILIDADE e não por tamanho: ali declara-se o metrónomo (o manifesto,
//! a lei da batida e o kernel do dispositivo), aqui prova-se que ele bate.

use super::*;
use ph2d_nodegraph::cook::OpResolver;

/// Run the metronome over `ticks` frames at 60 Hz, feeding the output back
/// as `state` (what the `pre` self-loop does). Returns the fired playheads.
fn beats(ticks: u64, period: f32, offset: f32) -> Vec<f64> {
    let mut state = Stream::new(1);
    let mut fired = Vec::new();
    for i in 0..ticks {
        let t = i as f64 / 60.0;
        state = step(1, &[cycle_index(t, period, offset, 0.0, 0)], 0.0, &state);
        if let Some(Column::Scalar(v)) = state.get(PULSE_COL)
            && v[0] > 0.5
        {
            fired.push(t);
        }
    }
    fired
}

/// FALSIFICATION of the edge detection: over one second at 60 Hz with a
/// 0.5 s period, the metronome fires exactly twice — the start beat and the
/// one boundary inside the window (t = 0.5). Firing "while inside a cycle"
/// — the bug the carried cycle index exists to prevent — would fire on all
/// 60 ticks.
#[test]
fn it_beats_on_the_cycle_boundary_not_once_per_tick() {
    let fired = beats(60, 0.5, 0.0);
    assert_eq!(fired, vec![0.0, 0.5], "start beat + one boundary, not 60");
}

/// FALSIFICATION of "period is wired": halving the period doubles the beat
/// count over the same window. A metronome that ignored its period (fixed
/// rate, or the un-clamped division) could not show this ratio.
#[test]
fn halving_the_period_doubles_the_beats() {
    let slow = beats(240, 1.0, 0.0).len(); // 4 s → beats at 0,1,2,3
    let fast = beats(240, 0.5, 0.0).len(); // 4 s → beats at 0,.5,…,3.5
    assert_eq!((slow, fast), (4, 8));
}

/// The offset shifts the beat GRID: with `offset 0.3` the boundaries land
/// at 0.3, 1.3, … The start beat still fires at t = 0 (a metronome bangs
/// when it starts, wherever it is in the cycle — Max `metro`).
#[test]
fn the_offset_shifts_the_beat_grid() {
    let fired = beats(120, 1.0, 0.3);
    assert_eq!(fired.len(), 3);
    assert_eq!(fired[0], 0.0, "the start beat");
    // Boundaries land on the first tick at/after 0.3 and 1.3.
    assert!((fired[1] - 0.3).abs() < 1.0 / 60.0 + 1e-9, "{}", fired[1]);
    assert!((fired[2] - 1.3).abs() < 1.0 / 60.0 + 1e-9, "{}", fired[2]);
}

/// A NEGATIVE offset must not fake the start beat away: `beat_primed` is a
/// separate column, so a first-tick cycle index that happens to be non-zero
/// (offset −2.5 → k = 2 at t = 0) still reads as "no state yet" → fire.
/// (A sentinel value inside `beat_cycle` would collide exactly here.)
#[test]
fn a_negative_offset_still_fires_the_start_beat() {
    let fired = beats(31, 1.0, -2.5);
    assert_eq!(fired[0], 0.0, "primed flag, not a magic cycle value");
    assert_eq!(fired.len(), 2, "then the 3.0-boundary at t = 0.5");
}

/// A degenerate `period ≤ 0` is clamped to `MIN_PERIOD`, never divided by:
/// the cycle index stays finite and the node keeps ticking (at most one
/// pulse per tick — a pulse column cannot express more).
#[test]
fn a_degenerate_period_is_clamped_not_divided_by_zero() {
    let fired = beats(10, 0.0, 0.0);
    assert!(fired.len() == 10, "every tick crosses ≥1 clamped boundary");
    for t in [0.0, 1.0, -3.0] {
        assert!(cycle_index(t, 0.0, 0.0, 0.0, 0).is_finite());
        assert!(cycle_index(t, -1.0, 0.0, 0.0, 0).is_finite());
    }
}

/// The beat is UNIFORM: every instance fires on the same tick (a global
/// beat — the whole point of replacing the per-channel clock hack).
#[test]
fn every_instance_beats_together() {
    let state = step(3, &[0.0; 3], 0.0, &Stream::new(3));
    match state.get(PULSE_COL).unwrap() {
        Column::Scalar(v) => assert_eq!(v, &vec![1.0, 1.0, 1.0]),
        _ => panic!(),
    }
}

/// The playhead standing still (paused transport re-cooking) does NOT
/// retrigger: same cycle index, primed state → silence until t moves on.
#[test]
fn a_paused_playhead_does_not_retrigger() {
    let mut state = Stream::new(1);
    let mut total = 0.0;
    for _ in 0..5 {
        state = step(1, &[cycle_index(1.25, 1.0, 0.0, 0.0, 0)], 0.0, &state);
        if let Some(Column::Scalar(v)) = state.get(PULSE_COL) {
            total += v[0];
        }
    }
    assert_eq!(total, 1.0, "the start beat only; a held k never refires");
}

#[test]
fn registers_and_resolves() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    assert!(reg.resolve(MANIFEST.id).is_some());
}

/// Corre o metrónomo com N linhas e devolve, por linha, os instantes em que ela
/// disparou — a versão por-fileira do [`beats`].
fn beats_rows(ticks: u64, n: usize, period: f32, stagger: f32, count: f32) -> Vec<Vec<f64>> {
    let mut state = Stream::new(n);
    let mut fired = vec![Vec::new(); n];
    for i in 0..ticks {
        let t = i as f64 / 60.0;
        let ks: Vec<f32> = (0..n)
            .map(|r| cycle_index(t, period, 0.0, stagger, r))
            .collect();
        state = step(n, &ks, count, &state);
        if let Some(Column::Scalar(v)) = state.get(PULSE_COL) {
            for (r, p) in v.iter().enumerate() {
                if *p > 0.5 {
                    fired[r].push(t);
                }
            }
        }
    }
    fired
}

/// **OS DEFAULTS SÃO O METRÓNOMO QUE SEMPRE SHIPOU** — `phase_stagger = 0` e
/// `count = 0` dão exactamente a mesma lista de batidas, em toda linha.
#[test]
fn the_appended_params_default_to_the_metronome_that_shipped() {
    let one = beats(180, 0.5, 0.0);
    for row in beats_rows(180, 4, 0.5, 0.0, 0.0) {
        assert_eq!(row, one, "uma linha divergiu do metronomo uniforme");
    }
}

/// **A FASE POR LINHA ESCALONA AS BATIDAS — cada linha no seu tempo.**
///
/// ⚠️ **O oráculo é a DIFERENÇA entre linhas vizinhas, não a lista de uma
/// linha.** Um `stagger` que fosse lido uma vez e aplicado a todas (o bug de
/// broadcast que a coluna de estado N-wide convidava) deslocaria as quatro
/// juntas — a lista de cada uma mudaria, e só a diferença entre elas o apanha.
#[test]
fn the_per_row_phase_staggers_the_beats() {
    let stagger = 0.1_f32;
    let rows = beats_rows(240, 4, 1.0, stagger, 0.0);
    assert_eq!(rows.len(), 4, "quatro linhas pedidas, quatro devolvidas");
    for (r, row) in rows.iter().enumerate() {
        assert!(!row.is_empty(), "a linha {r} nao bateu");
    }
    // ⚠️ **O oráculo é a GRELHA de cada linha, não a n-ésima batida dela.** A
    // primeira versão deste gate comparou `rows[r][1]` com `rows[r−1][1]` e
    // reprovou sobre produto correcto: a *batida de partida* (o tique não
    // primado, comum a todas as linhas) desloca a indexação, então a «segunda
    // batida» da linha 1 e a da linha 0 não são a mesma batida da grelha.
    //
    // O que é exacto e não depende de quantas batidas houve: as batidas da
    // linha `r` caem em `r·stagger + k·period`, logo `(batida − r·stagger)` é
    // múltiplo do período — a menos de um quadro de 60 Hz, que é a resolução
    // com que a sonda observa.
    let period = 1.0_f64;
    for (r, row) in rows.iter().enumerate() {
        for &t in row.iter().skip(1) {
            let phase = (t - r as f64 * f64::from(stagger)).rem_euclid(period);
            let off = phase.min(period - phase);
            assert!(
                off < 1.0 / 60.0 + 1e-9,
                "linha {r}: a batida em {t} nao esta' na grelha dela ({off} fora)"
            );
        }
    }
    // ⚠️ E o CONTROLE: as quatro linhas TÊM de diferir. Um `stagger` lido uma
    // vez e aplicado a todas (o bug de broadcast que a coluna de estado N-wide
    // convidava) deslocaria as quatro juntas e passaria no teste acima.
    assert!(
        rows.iter().any(|r| *r != rows[0]),
        "as quatro linhas bateram igual — o stagger nao e' por-linha"
    );
}

/// **A JANELA PARA O METRÓNOMO DEPOIS DE N BATIDAS — e `0` é sem janela.**
#[test]
fn the_window_stops_the_metronome_after_n_beats() {
    // 240 tiques = 4 s; com período 0,5 s são 8 batidas sem janela.
    let free = beats_rows(240, 1, 0.5, 0.0, 0.0);
    assert_eq!(free[0].len(), 8, "sem janela: {:?}", free[0]);
    let limited = beats_rows(240, 1, 0.5, 0.0, 3.0);
    assert_eq!(limited[0].len(), 3, "com janela de 3: {:?}", limited[0]);
    // E as três são as TRÊS PRIMEIRAS — a janela corta o fim, não o meio.
    assert_eq!(limited[0], free[0][..3].to_vec());
}

/// **A RÉGUA DE BPM É O MESMO NÚMERO QUE O PERÍODO** — `120 BPM` é meio segundo
/// por batida, e o gate pina o número que os liga.
///
/// ⚠️ E o piso é o `MIN_PERIOD` que o nó já tinha: um BPM zero dá `inf`, que
/// **não é NaN** — a grelha congela num compasso infinito, que é a leitura
/// honesta de *"zero batidas por minuto"*.
#[test]
fn bpm_is_the_same_ruler_the_period_uses() {
    assert!((seconds_per_beat(1.0, 999.0, 120.0) - 0.5).abs() < 1e-6);
    assert!((seconds_per_beat(1.0, 999.0, 60.0) - 1.0).abs() < 1e-6);
    // Desligada, a régua devolve o `period` verbatim — o `bpm` nem é olhado.
    assert_eq!(seconds_per_beat(0.0, 0.37, 999.0), 0.37);
    // O degenerado: `inf`, não NaN, e o índice do ciclo continua finito.
    assert!(seconds_per_beat(1.0, 1.0, 0.0).is_infinite());
    assert!(cycle_index(9.0, seconds_per_beat(1.0, 1.0, 0.0), 0.0, 0.0, 0).is_finite());
}

/// **O TETO DIGITÁVEL DO PERÍODO ESTÁ MUITO ACIMA DO SLIDER** — o defeito da
/// folha 12 linha 36, fechado.
///
/// ⚠️ **O que o gate mede não é o número, é a RELAÇÃO:** sem um `ParamHardMax`
/// o bridge cai no `hint.max`, e o teto do curso da mão vira o teto do produto.
/// Um período mais lento que 8 s era **indigitável**, e nenhuma referência tem
/// teto de período nenhum.
#[test]
fn the_typed_ceiling_of_the_period_is_far_above_the_slider() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).unwrap();
    let hints = reg.param_ui(MANIFEST.id).expect("hints");
    let slider = hints.iter().find(|h| h.param == "period").expect("period");
    let hard = reg
        .param_hard_max(MANIFEST.id, "period")
        .expect("o no' declara um teto digitavel para o period");
    assert!(
        hard > slider.max * 1000.0,
        "o teto digitavel ({hard}) tem de estar MUITO acima do curso da mao ({})",
        slider.max
    );
    // E o curso da mão NÃO mudou: quem arrasta continua onde estava.
    assert!(
        (slider.max - 8.0).abs() < 1e-6,
        "o slider mudou: {}",
        slider.max
    );
}

/// Fracções de quadro no fio do empate da linha `i`: a `f32` mais próxima de `frac(i·sc)` exacto e
/// os seus dois vizinhos, dentro de `[0, 1)`.
fn fracs_no_empate(i: usize, sc: f32) -> impl Iterator<Item = f32> {
    let p = i as f64 * f64::from(sc);
    let f = (p - p.floor()) as f32;
    [f.next_down(), f, f.next_up()]
        .into_iter()
        .filter(|x| (0.0..=ABAIXO_DE_UM).contains(x))
}

/// O índice da linha como a placa o faz quando FUNDE `frac − i·sc` num FMA (um arredondamento só).
fn indice_fundido(frac: f32, i: usize, sc: f32) -> f32 {
    (-(i as f32)).mul_add(sc, frac).floor()
}

/// O índice da linha sem fundir — a aritmética da CPU (`cycle_index`) e da WGSL do kernel.
fn indice_separado(frac: f32, i: usize, sc: f32) -> f32 {
    (frac - i as f32 * sc).floor()
}

/// ⭐⭐ **A imunidade do compasso a FMA, com régua que NÃO depende de a placa fundir** (doc 123 §1).
///
/// Uma placa pode fundir `frac − i·sc` (a WGSL permite) e arredondar uma vez só; a CPU arredonda
/// duas. O `sc` com [`SC_BITS`] bits faz `i·sc` EXACTO, e então as duas leituras dão o MESMO
/// `⌊⌋` em toda linha que o tecto admite — aqui afirmado com o FMA da CPU (`mul_add`), sobre toda
/// `i < 2^(24 − SC_BITS)` e as fracções no fio do empate de cada uma. O CONTROLO mede o `sc` CRU
/// (sem quantizar) pela mesma varredura e EXIGE divergência: sem ele, uma varredura que nunca
/// chegasse a um empate passaria igual.
#[test]
fn the_beat_index_is_the_same_fused_or_not() {
    let cobre = 1usize << (24 - SC_BITS);
    assert!(cobre >= ph2d_nodegraph::node::MAX_INSTANCIAS_POR_NO);
    // `(period, phase_stagger)`: as duas fixturas da paridade da placa e razões irracionais.
    let fixturas = [
        (0.0537_f32, 0.0031_f32),
        (0.05, 0.0),
        (1.0, 0.1),
        (0.5, 0.0173),
        (0.0031, 0.0537),
        (0.7, 0.000_913),
    ];
    let mut divergem_sem_quantizar = 0usize;
    for (period, stagger) in fixturas {
        let sc = compasso(0.0, period, 0.0, stagger).sc;
        let sc_cru = (f64::from(stagger) / f64::from(period.max(MIN_PERIOD))) as f32;
        for i in 0..cobre {
            for frac in fracs_no_empate(i, sc) {
                assert_eq!(
                    indice_fundido(frac, i, sc),
                    indice_separado(frac, i, sc),
                    "period {period}, stagger {stagger}: a linha {i} (frac {frac}) muda de \
                     tique se a placa fundir — o `sc` ({sc}) nao e' exacto"
                );
            }
            divergem_sem_quantizar += fracs_no_empate(i, sc_cru)
                .filter(|&f| indice_fundido(f, i, sc_cru) != indice_separado(f, i, sc_cru))
                .count();
        }
    }
    assert!(
        divergem_sem_quantizar > 0,
        "controlo: sem quantizar o `sc` a varredura nao achou nenhum empate que o FMA mude — \
         ela nao chega ao fio e nao mede nada"
    );
}

/// **SONDA da fixtura da placa no fio do FMA** (doc 123 §1): procura um `offset` e um tique
/// `≥ 1` em que, com o `sc` CRU, uma linha `< 64` muda de ciclo se a placa fundir e não mudava no
/// tique anterior — a fixtura que faz a paridade da placa morder a mutação «sem quantizar».
#[test]
#[ignore = "sonda: procura a fixtura da placa no fio do FMA"]
fn sonda_a_fixtura_do_fio_do_fma() {
    let (period, stagger) = (0.0537_f32, 0.0031_f32);
    let sc_cru = (f64::from(stagger) / f64::from(period)) as f32;
    let mut achadas = 0;
    for o in 0..200_000u32 {
        let offset = o as f32 * 1e-7;
        for tique in 1..12u64 {
            let c = compasso(tique as f64 / 60.0, period, offset, stagger);
            for i in 0..64usize {
                if indice_fundido(c.frac, i, sc_cru) != indice_separado(c.frac, i, sc_cru) {
                    eprintln!(
                        "offset {offset:e} ({o}) tique {tique} linha {i} frac {}",
                        c.frac
                    );
                    achadas += 1;
                }
            }
        }
        if achadas >= 8 {
            break;
        }
    }
}

/// ⭐ **A fixtura da placa ESTÁ no fio do FMA** (doc 123 §1): com o `sc` CRU a linha `11` muda de
/// ciclo no tique `3` se fundir (e no tique `2` não muda), e com o `sc` do produto nenhuma das
/// `64` linhas muda em nenhum dos `12` tiques. Sem isto a paridade da placa sobre
/// [`FIXTURA_DO_FIO_DO_FMA`] seria uma fixtura qualquer, e voltaria a não morder.
#[test]
fn the_device_fixture_sits_on_the_fma_edge() {
    let (period, stagger, offset) = FIXTURA_DO_FIO_DO_FMA;
    let sc_cru = (f64::from(stagger) / f64::from(period)) as f32;
    let fundiu = |tique: u64, i: usize, sc: f32| {
        let c = compasso(tique as f64 / 60.0, period, offset, stagger);
        indice_fundido(c.frac, i, sc) != indice_separado(c.frac, i, sc)
    };
    assert!(
        fundiu(3, 11, sc_cru),
        "a linha 11 no tique 3 nao esta' no fio"
    );
    assert!(!fundiu(2, 11, sc_cru), "e no tique 2 ela ja' divergia");
    for tique in 0..12 {
        let sc = compasso(tique as f64 / 60.0, period, offset, stagger).sc;
        for i in 0..64 {
            assert!(
                !fundiu(tique, i, sc),
                "o `sc` do produto diverge: tique {tique}, linha {i}"
            );
        }
    }
}
