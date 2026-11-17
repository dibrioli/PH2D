use super::*;

fn l(v: f32, when: i32, pulsos: usize) -> Leitura {
    Leitura {
        v,
        when,
        level: 0.5,
        pulsos,
    }
}

/// Corre uma série de valores em tiques seguidos e devolve em que tiques enviou.
fn serie(when: i32, valores: &[f32]) -> Vec<usize> {
    let mut e = Envios::default();
    valores
        .iter()
        .enumerate()
        .filter(|(i, v)| e.passo(NodeId(7), *i as u64, &l(**v, when, 0)) > 0)
        .map(|(i, _)| i)
        .collect()
}

#[test]
fn falls_below_fires_once_on_the_crossing_and_not_every_frame() {
    assert_eq!(
        serie(WHEN_FALLS, &[0.9, 0.7, 0.4, 0.3, 0.2, 0.6, 0.1]),
        vec![2, 6]
    );
}

#[test]
fn rises_above_fires_once_on_the_crossing() {
    assert_eq!(serie(WHEN_RISES, &[0.1, 0.5, 0.9, 0.2, 0.8]), vec![1, 4]);
}

#[test]
fn the_first_tick_never_fires_a_false_edge() {
    assert!(serie(WHEN_FALLS, &[0.1, 0.1]).is_empty());
    assert!(serie(WHEN_RISES, &[0.9, 0.9]).is_empty());
}

#[test]
fn a_jump_in_ticks_only_rebaselines() {
    let mut e = Envios::default();
    assert_eq!(e.passo(NodeId(1), 10, &l(0.9, WHEN_FALLS, 0)), 0);
    // Parou e recomeçou noutro tique: sem borda.
    assert_eq!(e.passo(NodeId(1), 40, &l(0.1, WHEN_FALLS, 0)), 0);
    assert_eq!(e.passo(NodeId(1), 41, &l(0.9, WHEN_FALLS, 0)), 0);
    assert_eq!(e.passo(NodeId(1), 42, &l(0.1, WHEN_FALLS, 0)), 1);
    assert_eq!(e.vezes(NodeId(1)), 1);
}

#[test]
fn pulse_sends_on_every_firing_and_counts() {
    let mut e = Envios::default();
    assert_eq!(e.passo(NodeId(3), 0, &l(0.0, WHEN_PULSE, 0)), 0);
    assert_eq!(e.passo(NodeId(3), 1, &l(0.0, WHEN_PULSE, 2)), 2);
    assert_eq!(e.passo(NodeId(3), 5, &l(0.0, WHEN_PULSE, 1)), 1);
    assert_eq!(e.vezes(NodeId(3)), 2);
    e.retem(&[]);
    assert_eq!(e.vezes(NodeId(3)), 0);
}

#[test]
fn the_output_passes_the_value_and_carries_what_the_bridge_reads() {
    let value = Stream::new(2).with(VALUE_COL, Column::Scalar(vec![0.3, 0.8]));
    let pulse = Stream::new(3).with(PULSE_COL, Column::Scalar(vec![1.0, 0.0, 1.0]));
    let out = saida(&value, &pulse, 2.0, 0.25);
    assert_eq!(out.get(VALUE_COL), value.get(VALUE_COL));
    let lida = ler(&out).expect("é um Send signal");
    assert_eq!(
        lida,
        Leitura {
            v: 0.3,
            when: WHEN_FALLS,
            level: 0.25,
            pulsos: 2
        }
    );
    // Sem fio no `value`: uma linha a zero, e o pulso ainda viaja.
    let so_pulso = ler(&saida(&Stream::new(0), &pulse, 0.0, 0.5)).expect("uma linha");
    assert_eq!((so_pulso.v, so_pulso.pulsos), (0.0, 2));
    // Um stream de `pulse.signal` não é lido como Send signal.
    assert!(ler(&pulse).is_none());
}

#[test]
fn it_registers_with_a_name_and_its_params() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).expect("regista");
    assert_eq!(MANIFEST.param_default(LEVEL), Some(0.5));
    assert_eq!(MANIFEST.param_default(WHEN), Some(0.0));
}

#[test]
fn a_contagem_zera_a_cada_play_e_nao_ao_parar() {
    let mut e = Envios::default();
    let n = NodeId(7);
    e.corrida(true);
    e.passo(n, 1, &l(0.0, WHEN_PULSE, 1));
    e.passo(n, 2, &l(0.0, WHEN_PULSE, 1));
    assert_eq!(e.vezes(n), 2);
    e.corrida(true);
    assert_eq!(e.vezes(n), 2, "a correr, o quadro seguinte não zera");
    e.corrida(false);
    assert_eq!(e.vezes(n), 2, "parar guarda o número da corrida");
    e.corrida(true);
    assert_eq!(e.vezes(n), 0, "o Play seguinte começa do zero");
}

#[test]
fn the_level_itself_counts_as_crossed_only_on_the_arrival_side() {
    assert_eq!(serie(WHEN_FALLS, &[0.9, 0.5, 0.1, 0.5, 0.1]), vec![1]);
    assert_eq!(serie(WHEN_RISES, &[0.1, 0.5, 0.9, 0.5, 0.9]), vec![1]);
}

#[test]
fn the_play_edge_baseline_does_not_survive_a_stop() {
    let mut e = Envios::default();
    let n = NodeId(2);
    e.corrida(true);
    assert_eq!(e.passo(n, 5, &l(0.9, WHEN_FALLS, 0)), 0);
    e.corrida(false);
    e.corrida(true);
    assert_eq!(
        e.passo(n, 6, &l(0.1, WHEN_FALLS, 0)),
        0,
        "o Play zera a base"
    );
}
