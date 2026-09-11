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

/// **TODA FILEIRA CONTINUA A SUBIR — e não «subiu alguma coisa».**
///
/// ⚠️ **Este gate nasceu de um smoke reprovado, e a versão anterior dele passou
/// sobre a cena morta** (Enio, 2026-08-22: *"as duas últimas fileiras de baixo não
/// se movem"*). O par da comparação era alimentado por um sinal **estático**, então
/// cada peça armava **uma vez** e ficava: as duas fileiras subiam um degrau no
/// primeiro quadro e nunca mais se mexiam. O gate media `top >= 1` no fim — que é
/// **verdade de uma fileira que saltou uma vez e morreu**.
///
/// *Um total não distingue uma cena que anda de uma que andou.* O oráculo é a
/// DIFERENÇA entre dois instantes, e a fileira da janela é a única que pode parar —
/// e mesmo essa tem de ter andado ANTES de parar.
#[test]
fn every_row_keeps_moving_and_only_the_window_stops() {
    let early = heights_after(90); // 1,5 s
    let mid = heights_after(200); // ~3,3 s
    let late = heights_after(400); // ~6,7 s
    let top = |(h, bases): &(Vec<Vec<f32>>, Vec<f32>), r: usize| -> i32 {
        (0..COLS as usize)
            .map(|i| steps(h, bases, r, i))
            .max()
            .unwrap()
    };
    for (r, row) in ROWS_TABLE.iter().enumerate() {
        // Toda fileira ANDA no primeiro trecho — inclusive a que vai parar.
        assert!(
            top(&mid, r) > top(&early, r),
            "fileira {r} ({}) nao andou entre 1,5 s e 3,3 s",
            row.label
        );
        let is_window = matches!(row.kind, Kind::Beat { count, .. } if count > 0.5);
        if is_window {
            assert_eq!(
                top(&late, r),
                top(&mid, r),
                "fileira {r}: a janela devia ter PARADO"
            );
        } else {
            assert!(
                top(&late, r) > top(&mid, r),
                "fileira {r} ({}) parou de andar entre 3,3 s e 6,7 s",
                row.label
            );
        }
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
    // A de cima: TODA peça cruza o limiar único (ele está abaixo do pico da onda).
    assert!(
        single.iter().all(|b| *b),
        "o limiar unico devia subir a fila inteira"
    );
    // A de baixo: bolinha sim, bolinha não — as de limiar alto NUNCA cruzam.
    let flips = |v: &[bool]| v.windows(2).filter(|w| w[0] != w[1]).count();
    assert!(
        flips(&per_el) > COLS as usize / 2,
        "o limiar por-elemento tem de alternar peca a peca, alternou {} vezes",
        flips(&per_el)
    );
    assert!(
        per_el.iter().any(|b| !*b),
        "alguma peca tem de NAO subir, senao o par nao diz nada"
    );
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
