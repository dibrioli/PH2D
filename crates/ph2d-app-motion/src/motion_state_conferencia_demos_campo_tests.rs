//! Gates da cena `=83` — **o campo que era um número** (Grupo Y, doc 90 §5).
//!
//! ⚠️ **O oráculo é a VARIAÇÃO INTERNA de cada figura, não a excursão dela.** Um gate que
//! medisse "a direita move-se mais" passaria numa cena em que as duas metades fossem uniformes
//! com amplitudes diferentes — e é exactamente uma figura uniforme que o defeito produzia.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

/// As posições de uma célula, já sem o deslocamento que a coloca na grelha.
///
/// ⚠️ A linha da ONDA precisa do relógio a andar (o laço `pre`), então o harness corre
/// `ticks` quadros e lê o último — a lição que a cena `=80` pagou.
fn points(
    doc: &MotionDoc,
    reg: &NodeRegistry,
    sink: NodeId,
    cell: usize,
    ticks: usize,
) -> Vec<[f32; 2]> {
    let mut c = Cook::new();
    let mut last = Vec::new();
    for k in 0..ticks.max(1) {
        let t = k as f64 / 60.0;
        let out = c.cook(&doc.graph, reg, sink, t).expect("coze");
        if let Some(CookValue::Instances(s)) = out.first()
            && let Some(Column::Vec2(v)) = Stream::get(s, "P")
        {
            last = v.clone();
        }
        c.advance_tick(&doc.graph, reg, t).expect("o quadro fecha");
    }
    let row = cell / 2;
    let half = cell % 2;
    let cx = if half == 0 { -COL_X } else { COL_X };
    let cy = (ROWS_TABLE.len() as f32 - 1.0) * 0.5 * ROW_GAP - row as f32 * ROW_GAP;
    last.iter().map(|p| [p[0] - cx, p[1] - cy]).collect()
}

/// **A cena constrói as seis células.**
#[test]
fn the_field_scene_builds_every_cell() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_campo_demo_document(&mut doc, &reg).expect("a cena constroi");
    assert_eq!(sinks.len(), ROWS_TABLE.len() * 2, "duas celulas por linha");
    let (n, count) = authored();
    assert_eq!(n, ROWS_TABLE.len(), "o anuncio conta a mesma tabela");
    assert!(count >= 8.0, "uma figura precisa de pecas que cheguem");
    for (k, sink) in sinks.iter().enumerate() {
        assert!(
            !points(&doc, &reg, *sink, k, 40).is_empty(),
            "celula {k} desenha alguma coisa"
        );
    }
}

/// **A METADE DIREITA VARIA AO LONGO DE SI MESMA; A ESQUERDA NÃO.**
///
/// É a cena inteira numa asserção, e o oráculo é o **desvio da figura em relação à sua própria
/// tendência**: com um valor uniforme cada linha é uma reta (ou um favo regular); com o campo a
/// valer, a figura curva-se de um lado só.
///
/// ⚠️ **As duas metades são obrigatórias.** Só a direita passaria numa cena que nunca mostrasse
/// o defeito; só a esquerda, numa em que o campo não chegasse a lado nenhum.
#[test]
fn only_the_field_half_varies_along_itself() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_campo_demo_document(&mut doc, &reg).expect("constroi");
    // O desvio máximo em Y face à RETA que liga as pontas — zero numa figura sem curvatura.
    let bow = |p: &[[f32; 2]]| -> f32 {
        let n = p.len();
        if n < 3 {
            return 0.0;
        }
        let (a, b) = (p[0], p[n - 1]);
        (1..n - 1)
            .map(|i| {
                let t = (p[i][0] - a[0]) / (b[0] - a[0]).abs().max(1e-6);
                (p[i][1] - (a[1] + (b[1] - a[1]) * t)).abs()
            })
            .fold(0.0f32, f32::max)
    };
    for (k, row) in ROWS_TABLE.iter().enumerate() {
        if row.case != Case::Wrap {
            continue;
        }
        let flat = bow(&points(&doc, &reg, sinks[k * 2], k * 2, 1));
        let field = bow(&points(&doc, &reg, sinks[k * 2 + 1], k * 2 + 1, 1));
        assert!(
            flat < 1e-3,
            "linha {}: a metade do NUMERO tinha de sair reta (curvou {flat})",
            k + 1
        );
        assert!(
            field > 0.1,
            "linha {}: a metade do CAMPO tem de curvar (curvou {field})",
            k + 1
        );
    }
}

/// **A TRELIÇA DERRETE DE UM LADO SÓ** — e o oráculo é a METADE ESQUERDA como referência.
///
/// ⚠️ **A primeira versão mediu o desvio-padrão dos espaçamentos por metade e reprovou sobre
/// produto correcto** (esquerda 0,057 contra direita 0,031): com ~5 pontos por metade isso é
/// ruído, não estatística. *Uma régua fraca sobre um efeito real lê como o efeito ao contrário.*
///
/// A régua exacta estava ali ao lado: **a metade do NÚMERO é a treliça perfeita** (jitter `0`
/// em toda a parte). Então o deslocamento de cada ponto da metade do CAMPO contra o seu gémeo
/// é o jitter que ele recebeu — e ele tem de CRESCER com o índice.
#[test]
fn the_lattice_melts_on_one_side_only() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_campo_demo_document(&mut doc, &reg).expect("constroi");
    let k = ROWS_TABLE
        .iter()
        .position(|r| r.case == Case::Jitter)
        .expect("a linha da trelica existe");
    let perfect = points(&doc, &reg, sinks[k * 2], k * 2, 1);
    let melted = points(&doc, &reg, sinks[k * 2 + 1], k * 2 + 1, 1);
    assert_eq!(
        perfect.len(),
        melted.len(),
        "as duas metades tem os mesmos pontos"
    );
    let n = perfect.len();
    let disp: Vec<f32> = (0..n)
        .map(|i| (melted[i][0] - perfect[i][0]).hypot(melted[i][1] - perfect[i][1]))
        .collect();
    // O primeiro quarto quase não se mexe; o último move-se de facto.
    let q = n / 4;
    let head: f32 = disp[..q].iter().sum::<f32>() / q as f32;
    let tail: f32 = disp[n - q..].iter().sum::<f32>() / q as f32;
    // ⚠️ **A barra é a RAZÃO, não um piso absoluto sobre o início.** A rampa é LINEAR: o
    // primeiro quarto dela vale ~1/8 do máximo, então ele derrete um pouco de facto — uma
    // primeira versão exigiu `head < 0,02`, mediu `0,039` e reprovou sobre produto correcto.
    // *O que esta linha afirma é que o jitter CRESCE ao longo da figura, não que ele começa
    // exactamente em zero.*
    assert!(
        tail > head * 3.0,
        "o jitter tem de CRESCER ao longo da trelica ({head} -> {tail})"
    );
    assert!(tail > 0.15, "e o fim tem de derreter de facto ({tail})");
}

/// **A ONDA MOSTRA O VALE** — e é o defeito que esta linha encena.
///
/// ⚠️ Com a altura no `size` a onda **não move uma peça**: crista e vale desenham a mesma
/// bolha. Com ela no `Y`, metade das peças fica ABAIXO da linha de repouso — que é a metade
/// que era invisível.
#[test]
fn the_wave_shows_its_trough_only_when_the_height_drives_y() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_campo_demo_document(&mut doc, &reg).expect("constroi");
    let k = ROWS_TABLE
        .iter()
        .position(|r| r.case == Case::Wave)
        .expect("a linha da onda existe");
    let spread = |p: &[[f32; 2]]| -> f32 {
        p.iter().map(|q| q[1]).fold(f32::NEG_INFINITY, f32::max)
            - p.iter().map(|q| q[1]).fold(f32::INFINITY, f32::min)
    };
    // 40 tiques: tempo de o pino do centro radiar e o campo ter crista E vale.
    let size_half = points(&doc, &reg, sinks[k * 2], k * 2, 40);
    let y_half = points(&doc, &reg, sinks[k * 2 + 1], k * 2 + 1, 40);
    let (a, b) = (spread(&size_half), spread(&y_half));
    assert!(
        b > a + 0.05,
        "no canal Y a onda tem de ABRIR em Y mais que no canal Size ({a} vs {b})"
    );
}

/// **NENHUMA CÉLULA INVADE A VIZINHA** — a lei de layout das cenas irmãs.
#[test]
fn no_cell_climbs_into_its_neighbour() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_campo_demo_document(&mut doc, &reg).expect("constroi");
    for (k, sink) in sinks.iter().enumerate() {
        let p = points(&doc, &reg, *sink, k, 40);
        let (mut mx, mut my) = (0.0f32, 0.0f32);
        for q in &p {
            mx = mx.max(q[0].abs());
            my = my.max(q[1].abs());
        }
        assert!(
            my < ROW_GAP * 0.5,
            "celula {k} sobe {my}, meia linha e' {}",
            ROW_GAP * 0.5
        );
        assert!(
            mx < COL_X,
            "celula {k} alarga {mx}, a coluna vive a {COL_X}"
        );
    }
}
