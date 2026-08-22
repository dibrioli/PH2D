//! Gates da cena `=78` — **os knobs que faltavam ao domínio de valor** (doc 89,
//! folha 15).
//!
//! ⚠️ A cena é um gráfico de dezoito perfis em PARES, e um gráfico assim tem
//! quatro modos de falhar em silêncio: uma fileira **chata** (a cadeia não produziu
//! nada), duas fileiras do mesmo par **iguais** (o knob não chegou ao kernel), duas
//! fileiras vizinhas que **se cruzam** (a amplitude passou o vão), e duas colunas
//! que **se sobrepõem** (a largura de uma fileira não está escrita em lado nenhum
//! do grafo — ela sai de `(cols − 1) · gap_x`, três nós acima do transform).
//!
//! O segundo é o que importa: é a falha que um knob nunca lido produz, e a única
//! que um smoke de *"apareceu alguma coisa?"* deixa passar.
//!
//! ⚠️ **A ordem da tabela É a ordem na tela**, e [`PAIRS`] depende disso: os dois
//! membros de cada par são VIZINHOS. Reordenar `ROWS_TABLE` sem reordenar `PAIRS`
//! deixaria os gates a comparar coisas que não são par.

use super::*;
use ph2d_nodegraph::attr::{Column, Stream};
use ph2d_nodegraph::cook::Cook;
use ph2d_nodegraph::value::CookValue;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

/// O perfil de uma fileira no instante `t`: o `y` de cada peça MENOS o piso dela —
/// o valor que o `motion.drive` somou, isolado do lugar onde a fileira foi posta.
fn profile_at(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId, t: f64) -> Vec<f32> {
    let mut c = Cook::new();
    let out = c.cook(&doc.graph, reg, sink, t).expect("a cena coze");
    let CookValue::Instances(s) = &out[0] else {
        panic!("a saida e um stream")
    };
    let ys: Vec<f32> = match Stream::get(s, "P") {
        Some(Column::Vec2(v)) => v.iter().map(|p| p[1]).collect(),
        _ => Vec::new(),
    };
    let base = ys.iter().copied().fold(f32::INFINITY, f32::min);
    ys.into_iter().map(|y| y - base).collect()
}

fn profile(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> Vec<f32> {
    profile_at(doc, reg, sink, 0.0)
}

fn excursion(p: &[f32]) -> f32 {
    p.iter().copied().fold(f32::NEG_INFINITY, f32::max)
}

fn worst(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(x, y)| (x - y).abs())
        .fold(0.0f32, f32::max)
}

/// A caixa em X de uma fileira, em coordenadas de mundo (sem subtrair piso nenhum).
fn x_span(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> (f32, f32) {
    let mut c = Cook::new();
    let out = c.cook(&doc.graph, reg, sink, 0.0).expect("coze");
    let CookValue::Instances(s) = &out[0] else {
        panic!("stream")
    };
    let Some(Column::Vec2(v)) = Stream::get(s, "P") else {
        panic!("P")
    };
    v.iter()
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), p| {
            (lo.min(p[0]), hi.max(p[0]))
        })
}

/// Os pares `(controle, com o knob, barra, o que é)`. **Um por knob apendado**, e a
/// barra de cada um é a excursão que a fileira dele percorre a dividir por ~4 — não
/// um número redondo: um par cujo efeito é 20% do perfil não pode ser cobrado a
/// 50%, e um cujo efeito é o perfil inteiro não deve passar com 5%.
const PAIRS: &[(usize, usize, f32, &str)] = &[
    (0, 1, 0.15, "step: a mascara contra o espelho dela"),
    (
        2,
        3,
        0.05,
        "quantize: a grade na origem contra a grade em fase",
    ),
    (4, 5, 0.05, "pattern: o padrao contra o padrao deslizado"),
    (6, 7, 0.05, "switch: a escada contra a dissolucao"),
    (8, 9, 0.10, "curve: a tenda inteira contra a tenda a meio"),
    (10, 11, 0.10, "mix: a soma livre contra a soma travada"),
    (
        12,
        13,
        0.15,
        "math: a rampa contra a rampa somada a` inversa",
    ),
    (
        14,
        15,
        0.03,
        "math: a quina seca contra a quina arredondada",
    ),
];

/// **A cena constrói as dezoito fileiras.** Se um `wire` falhasse, o roteador cairia
/// no `unwrap_or_default()` — uma tela VAZIA, que num smoke lê como *"a feature não
/// foi construída"* em vez de *"a cena está partida"*.
#[test]
fn the_knobs_scene_builds_every_row() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_knobs_demo_document(&mut doc, &reg).expect("a cena constroi");
    assert_eq!(sinks.len(), ROWS_TABLE.len(), "uma sink por fileira");
    let (n, _, _) = authored();
    assert_eq!(n, ROWS_TABLE.len(), "o anuncio conta a mesma tabela");
    // As duas colunas: a mais alta não pode passar a escada de Y.
    let left = ROWS_TABLE.iter().filter(|r| r.col == 0).count();
    let right = ROWS_TABLE.len() - left;
    assert!(
        left <= LADDER && right <= LADDER,
        "uma coluna passou a escada: {left} / {right} contra {LADDER}"
    );
}

/// **Nenhuma fileira é CHATA.** Um perfil sem excursão é uma cadeia que não produziu
/// nada — e ele concordaria com qualquer outro perfil chato, o que faria o gate de
/// distinção abaixo passar por vácuo.
///
/// ⚠️ **As duas fileiras da LFO são medidas com o relógio JÁ FORA da rampa**, senão
/// a de baixo seria legitimamente chata em `t = 0` — que é precisamente a
/// propriedade que ela demonstra. Uma fixture que a cobrasse em `t = 0` estaria a
/// acusar a cena de fazer o que lhe foi pedido.
#[test]
fn every_row_draws_something() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_knobs_demo_document(&mut doc, &reg).expect("a cena constroi");
    for (k, sink) in sinks.iter().enumerate() {
        let temporal = matches!(ROWS_TABLE[k].knob, Knob::Lfo { .. });
        let t = if temporal {
            f64::from(LFO_FADE) + 1.0
        } else {
            0.0
        };
        let p = profile_at(&doc, &reg, *sink, t);
        assert_eq!(p.len(), COLS as usize, "fileira {k}: contagem de pecas");
        let e = excursion(&p);
        assert!(
            e > 0.15,
            "fileira {k} ({}) e' chata: excursao {e:e}",
            ROWS_TABLE[k].label
        );
    }
}

/// **CADA KNOB DESENHA COISA DIFERENTE DO SEU CONTROLE** — o oráculo da cena,
/// medido no stream que o render de facto consome.
///
/// ⚠️ Sem este gate a cena passaria com um knob que nunca fosse lido: as dezoito
/// fileiras apareceriam, todas com excursão, e o olho num smoke rápido veria
/// "dezoito gráficos". A pergunta é *nove PARES diferentes*.
#[test]
fn every_knob_draws_a_different_profile_from_its_control() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_knobs_demo_document(&mut doc, &reg).expect("a cena constroi");
    let profiles: Vec<Vec<f32>> = sinks.iter().map(|s| profile(&doc, &reg, *s)).collect();
    for &(a, b, bar, what) in PAIRS {
        let d = worst(&profiles[a], &profiles[b]);
        assert!(
            d > bar,
            "{what} (fileiras {a} e {b}) desenham o MESMO perfil: max |d| = {d:e} \
             contra a barra {bar:e}"
        );
    }
}

/// **A RAMPA DA LFO É O PAR TEMPORAL, e ele mede-se no TEMPO** — cedo a onda da
/// direita é muito menor que a da esquerda; tarde, as duas são a mesma onda.
///
/// ⚠️ **É a metade que os outros gates não veem.** Um `fade_in` que multiplicasse o
/// valor INTEIRO (em vez da amplitude) também começaria pequeno e também acabaria
/// igual — o que o separa é o par ser *a mesma onda* no fim, e nada mais.
#[test]
fn the_lfo_pair_grows_in_time_and_converges() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_knobs_demo_document(&mut doc, &reg).expect("a cena constroi");
    let (plain, fading) = (sinks[16], sinks[17]);
    // Cedo: a rampa está a um terço, então a onda da direita percorre ~1/3 da outra.
    let t_early = f64::from(LFO_FADE) / 3.0;
    let e_plain = excursion(&profile_at(&doc, &reg, plain, t_early));
    let e_fade = excursion(&profile_at(&doc, &reg, fading, t_early));
    assert!(
        e_fade < 0.6 * e_plain,
        "cedo, a onda com rampa tem de ser MENOR: {e_fade:e} contra {e_plain:e}"
    );
    // Tarde: a rampa está cheia, e as duas fileiras são a MESMA onda, ponto a ponto.
    let t_late = f64::from(LFO_FADE) + 0.7;
    let d = worst(
        &profile_at(&doc, &reg, plain, t_late),
        &profile_at(&doc, &reg, fading, t_late),
    );
    assert!(d < 1e-5, "tarde, as duas ondas tem de coincidir: {d:e}");
}

/// **Duas fileiras vizinhas NÃO se cruzam.** A amplitude por-fileira existe
/// exactamente por isto (a coluna `scale`), e sem o gate a próxima fileira com
/// alcance maior sobreporia a de baixo — dois gráficos que se cruzam deixam de ser
/// dois gráficos, e o smoke passaria a julgar um borrão.
#[test]
fn no_row_climbs_into_its_neighbour() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_knobs_demo_document(&mut doc, &reg).expect("a cena constroi");
    for (k, sink) in sinks.iter().enumerate() {
        let temporal = matches!(ROWS_TABLE[k].knob, Knob::Lfo { .. });
        let t = if temporal {
            f64::from(LFO_FADE) + 1.0
        } else {
            0.0
        };
        let e = excursion(&profile_at(&doc, &reg, *sink, t));
        assert!(
            e < ROW_GAP,
            "fileira {k} ({}) sobe {e:e}, mais que o vao de {ROW_GAP:e}",
            ROWS_TABLE[k].label
        );
    }
}

/// **AS DUAS COLUNAS NÃO SE TOCAM** — o gate que a sonda `measure_scene_layout`
/// existe para não ter de ser corrido à mão.
///
/// ⚠️ A largura de uma fileira **não está escrita em lado nenhum do grafo**: ela sai
/// de `(cols − 1) · gap_x`, três nós acima do `motion.transform` que a coloca. Dois
/// centros a `2·COL_X` de distância sobrepõem-se alegremente se cada fileira medir
/// mais do que isso, e o resultado é a queixa que originou a sonda (*"tudo
/// misturado e bagunçado"*).
#[test]
fn the_two_columns_do_not_overlap() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_knobs_demo_document(&mut doc, &reg).expect("a cena constroi");
    let mut right_lo = f32::INFINITY;
    let mut left_hi = f32::NEG_INFINITY;
    for (k, sink) in sinks.iter().enumerate() {
        let (lo, hi) = x_span(&doc, &reg, *sink);
        if ROWS_TABLE[k].col == 0 {
            left_hi = left_hi.max(hi);
        } else {
            right_lo = right_lo.min(lo);
        }
    }
    assert!(
        left_hi < right_lo,
        "as colunas sobrepoem-se: a esquerda chega a {left_hi:.2}, a direita comeca em {right_lo:.2}"
    );
    // E a cena inteira cabe na área que uma demo ocupa (medido na `=41`: ±5,17).
    for sink in &sinks {
        let (lo, hi) = x_span(&doc, &reg, *sink);
        assert!(
            lo > -5.4 && hi < 5.4,
            "a cena sai da tela: [{lo:.2} .. {hi:.2}]"
        );
    }
}

/// **A CENA INTEIRA É REIVINDICADA PELO DEVICE.** Os oito nós que este grupo tocou
/// têm kernel, e o valor de os ter é o sequenciador não cair para a CPU — uma
/// fileira que cozesse na CPU desenharia a mesma imagem, e a cena não diria nada
/// sobre o WGSL que os gates de paridade medem.
///
/// ⚠️ `plan` é headless: ele responde *quem reivindica o quê* sem adapter nenhum.
#[test]
fn every_row_is_claimed_by_the_device() {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_knobs_demo_document(&mut doc, &reg).expect("a cena constroi");
    for (k, sink) in sinks.iter().enumerate() {
        let plan = ph2d_gpu_cook::plan(&doc.graph, &reg, &reg, *sink);
        assert!(
            plan.is_fully_gpu(),
            "fileira {k} ({}) cai para a CPU",
            ROWS_TABLE[k].label
        );
    }
}

/// **CADA KNOB APENDADO TEM UMA FILEIRA** — o gate que impede a cena de envelhecer
/// em silêncio quando o próximo knob chegar.
///
/// ⚠️ A contagem é DERIVADA da tabela por variante, não escrita à mão: um knob novo
/// sem par nesta cena reprova aqui, e não seis semanas depois num smoke.
#[test]
fn every_appended_knob_has_a_pair_in_the_scene() {
    let mut seen = [0usize; 9];
    for row in ROWS_TABLE {
        let i = match row.knob {
            Knob::Step { .. } => 0,
            Knob::Quantize { .. } => 1,
            Knob::Pattern { .. } => 2,
            Knob::Switch { .. } => 3,
            Knob::Curve { .. } => 4,
            Knob::MixClamp { .. } => 5,
            Knob::MultiplyAdd { .. } => 6,
            Knob::SmoothMin { .. } => 7,
            Knob::Lfo { .. } => 8,
        };
        seen[i] += 1;
    }
    for (i, n) in seen.iter().enumerate() {
        assert_eq!(*n, 2, "o knob {i} nao tem um PAR (tem {n} fileira(s))");
    }
    assert_eq!(
        PAIRS.len() + 1,
        seen.len(),
        "um par por knob (a LFO tem gate proprio)"
    );
}
