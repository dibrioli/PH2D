//! Gates da cena `=80` — **o metrónomo** (doc 89, folha 12).
//!
//! ⚠️ **Esta cena é TEMPORAL e sequencial, e isso muda o instrumento.** Os nós de
//! pulso têm memória de borda por um laço `pre`, e o `pre` só avança quando o
//! quadro FECHA ([`Cook::advance_tick`]) — um laço que só `cook`a lê o mesmo tique
//! N vezes e vê uma cena PARADA. A sonda de movimento desta linha pagou essa lição
//! uma vez; aqui ela está escrita no harness.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

/// Corre a cena por `ticks` quadros a 60 Hz e devolve, por fileira, a ALTURA de
/// cada peça no fim — o número de degraus que ela subiu, em unidades de mundo.
///
/// ⚠️ **`advance_tick` a cada quadro**: sem ele os laços `pre` nunca avançam e toda
/// fileira fica no chão.
fn heights_after(ticks: usize) -> (Vec<Vec<f32>>, Vec<f32>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_pulse_demo_document(&mut doc, &reg).expect("a cena constroi");
    let mut cook = Cook::new();
    let mut last: Vec<Vec<f32>> = vec![Vec::new(); sinks.len()];
    for k in 0..ticks {
        let t = k as f64 / 60.0;
        for (r, sink) in sinks.iter().enumerate() {
            let out = cook.cook(&doc.graph, &reg, *sink, t).expect("coze");
            let CookValue::Instances(s) = &out[0] else {
                panic!("stream")
            };
            if let Some(Column::Vec2(v)) = Stream::get(s, "P") {
                last[r] = v.iter().map(|p| p[1]).collect();
            }
        }
        cook.advance_tick(&doc.graph, &reg, t)
            .expect("o quadro fecha");
    }
    // A linha de base de cada fileira — a mesma expressão que o construtor usa.
    let bases: Vec<f32> = (0..sinks.len())
        .map(|r| (ROWS as f32 - 1.0) * 0.5 * ROW_GAP - r as f32 * ROW_GAP)
        .collect();
    (last, bases)
}

/// Quantos degraus a peça `i` da fileira `r` subiu (arredondado).
fn steps(h: &[Vec<f32>], bases: &[f32], r: usize, i: usize) -> i32 {
    ((h[r][i] - bases[r]) / STEP_H).round() as i32
}

/// **A cena constrói as oito fileiras.**
#[test]
fn the_pulse_scene_builds_every_row() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_pulse_demo_document(&mut doc, &reg).expect("a cena constroi");
    assert_eq!(sinks.len(), ROWS_TABLE.len(), "uma sink por fileira");
    let (n, period, bpm, window) = authored();
    assert_eq!(n, ROWS_TABLE.len(), "o anuncio conta a mesma tabela");
    assert!(
        (bpm - 60.0 / period).abs() < 1e-6,
        "o BPM anunciado tem de ser o MESMO numero que o periodo"
    );
    assert!(window >= 2.0, "a janela tem de deixar passar mais de uma");
}

/// **TODA FILEIRA SOBE** — uma escada parada é um metrónomo que não bate, e ela
/// concordaria com qualquer outra escada parada.
#[test]
fn every_row_climbs() {
    let (h, bases) = heights_after(200);
    for (r, row) in ROWS_TABLE.iter().enumerate() {
        let top = (0..COLS as usize)
            .map(|i| steps(&h, &bases, r, i))
            .max()
            .unwrap();
        assert!(top >= 1, "fileira {r} ({}) nao subiu", row.label);
    }
}

/// **A RÉGUA: SEGUNDOS E BPM SOBEM EM LOCK-STEP** — é o mesmo número, então as duas
/// escadas têm de estar à mesma altura em TODO instante, não só no fim.
///
/// ⚠️ Comparar só o fim deixaria passar uma conversão que erra o ritmo e acerta o
/// total (por exemplo uma que atrase meia batida e depois recupere).
#[test]
fn the_two_rulers_climb_in_lock_step() {
    for ticks in [30usize, 60, 97, 150, 200] {
        let (h, bases) = heights_after(ticks);
        let a = steps(&h, &bases, 0, 0);
        let b = steps(&h, &bases, 1, 0);
        assert_eq!(
            a, b,
            "aos {ticks} tiques as duas reguas divergiram: {a} vs {b}"
        );
    }
}

/// **A FASE POR LINHA FAZ O DEGRAU PERCORRER A FILEIRA.**
///
/// ⚠️ **O oráculo é a fileira de CIMA tanto quanto a de baixo.** Sem o controle,
/// uma fileira em que TODAS as peças estivessem em alturas diferentes por qualquer
/// outro motivo passaria; o que a cena promete é que a de cima é PLANA e a de baixo
/// não é.
#[test]
fn the_per_row_phase_makes_the_step_travel_along_the_row() {
    let (h, bases) = heights_after(200);
    let spread = |r: usize| {
        let v: Vec<i32> = (0..COLS as usize)
            .map(|i| steps(&h, &bases, r, i))
            .collect();
        v.iter().max().unwrap() - v.iter().min().unwrap()
    };
    assert_eq!(spread(2), 0, "a fileira SEM fase tem de subir em bloco");
    assert!(
        spread(3) >= 1,
        "a fileira COM fase tem de ficar escalonada, espalhou {}",
        spread(3)
    );
}

/// **A JANELA PARA A ESCADA, E A SEM-JANELA CONTINUA.**
///
/// ⚠️ O `Clamp` do contador é o que torna isto legível: com `Wrap` a fileira que
/// parou voltaria ao chão de tempos a tempos e ficaria indistinguível de uma que
/// deu a volta.
#[test]
fn the_window_stops_the_staircase_and_the_free_one_keeps_going() {
    let (h, bases) = heights_after(300);
    let free = steps(&h, &bases, 4, 0);
    let limited = steps(&h, &bases, 5, 0);
    let (_, _, _, window) = authored();
    assert_eq!(
        limited, window as i32,
        "a janela devia parar em {window} degraus, parou em {limited}"
    );
    assert!(
        free > limited,
        "a fileira sem janela tem de estar mais alta: {free} contra {limited}"
    );
}

/// **O LIMIAR POR-ELEMENTO DESENHA UM PADRÃO QUE O LIMIAR ÚNICO NÃO SABE
/// DESENHAR.**
///
/// ⚠️ **É o par que mostra o bug silencioso.** Um fio ligado ao *param* `rise`
/// colapsa na linha `0` (o `driven_value` é `xs.first()`), então ele desenharia
/// **exactamente a fileira de cima** — meia fila — e um smoke rápido leria isso
/// como *"o limiar funciona"*. O oráculo aqui é a FORMA: a de cima é um único
/// bloco contíguo, a de baixo alterna.
#[test]
fn the_per_element_threshold_draws_a_pattern_the_single_one_cannot() {
    let (h, bases) = heights_after(120);
    let risen = |r: usize| -> Vec<bool> {
        (0..COLS as usize)
            .map(|i| steps(&h, &bases, r, i) >= 1)
            .collect()
    };
    let single = risen(6);
    let per_el = risen(7);
    // A de cima: um único bloco contíguo (a metade de cima da rampa).
    let flips = |v: &[bool]| v.windows(2).filter(|w| w[0] != w[1]).count();
    assert_eq!(flips(&single), 1, "o limiar unico corta a fila UMA vez");
    assert!(
        flips(&per_el) > 4,
        "o limiar por-elemento tem de alternar, alternou {} vezes",
        flips(&per_el)
    );
    // E as duas de facto diferem — o controle do par.
    assert_ne!(single, per_el, "os dois limiares desenharam o mesmo");
}

/// **Nenhuma fileira invade a vizinha** — a mesma lei das cenas irmãs.
#[test]
fn no_row_climbs_into_its_neighbour() {
    let (h, bases) = heights_after(400);
    for (r, row) in h.iter().enumerate() {
        let top = row
            .iter()
            .map(|y| y - bases[r])
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            top < ROW_GAP,
            "fileira {r} subiu {top}, mais que o vao de {ROW_GAP}"
        );
    }
}
