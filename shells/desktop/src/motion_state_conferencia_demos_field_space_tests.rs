//! Os gates da cena `=60` — o espaço do campo.

use super::*;
use ph2d_eval_motion::MotionCookPump;

/// A altura de GRADE de cada peça do bloco — a régua de que o deslocamento se subtrai.
///
/// ⚠️ **Monta-se pela MESMA função `block` da cena**, não por uma cópia dos params: uma
/// segunda escrita de `SIDE`/`GAP` aqui seria um número a envelhecer sozinho.
fn block_grid_y() -> Vec<f32> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    let mut g = Graph::new();
    let node = block(&mut g, 0.0, 0.0);
    let mut cook = ph2d_nodegraph::cook::Cook::new();
    let out = cook.cook(&g, &reg, node, 0.0).expect("coza");
    let ph2d_nodegraph::value::CookValue::Instances(st) = &out[0] else {
        panic!("stream")
    };
    match st.get("P") {
        Some(ph2d_nodegraph::attr::Column::Vec2(v)) => v.iter().map(|p| p[1]).collect(),
        _ => Vec::new(),
    }
}

/// O deslocamento vertical de cada peça de uma banda: a altura de mundo menos a da GRADE.
///
/// ⚠️ **A subtração é o que faz este oráculo medir o CAMPO e não a arrumação** — e a régua
/// tem de ser a grade REAL, não a média. A primeira versão subtraiu a média e reprovou sobre
/// produto correto: a grade tem 15 fileiras e varre 4,48 de mundo em Y, então o «desvio da
/// média» era dominado pelo `gap_y` e a razão dx/dy do CONTROLE media **0,21** em vez de ~1.
/// *Uma régua errada mede a régua.*
fn band_delta(band: usize) -> Vec<f32> {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    let mut doc = MotionDoc::default();
    let sinks = build_field_space_demo_document(&mut doc, &reg).expect("a cena monta");

    let mut pump = MotionCookPump::new();
    pump.advance_or_scrub_scoped(
        &doc.graph,
        &reg,
        std::slice::from_ref(&sinks[band]),
        0,
        |k| k as f64 / 60.0,
        [0.0, 0.0, 1.0, 1.0],
        [1.0, 1.0],
        &Default::default(),
    );
    let base = block_grid_y();
    pump.instances
        .iter()
        .zip(&base)
        .map(|(i, g)| i.world_pos[1] - g)
        .collect()
}

/// O pior `|Δ|` entre dois padrões de deslocamento.
fn worst(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b)
        .fold(0.0f32, |m, (x, y)| m.max((x - y).abs()))
}

/// A maior excursão de um padrão — a régua contra a qual as diferenças se leem.
fn span(v: &[f32]) -> f32 {
    let (lo, hi) = v
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), x| (l.min(*x), h.max(*x)));
    hi - lo
}

/// **AS QUATRO BANDAS EXISTEM, e a mensagem tem quatro rótulos.**
#[test]
fn the_scene_builds_the_four_bands_its_message_names() {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    let mut doc = MotionDoc::default();
    let sinks = build_field_space_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 4, "quatro bandas");
    assert_eq!(band_labels().count(), 4, "quatro rotulos");
}

/// **AS QUATRO SÃO DIFERENTES ENTRE SI, e nenhuma é «mais agitada».**
///
/// ⚠️ As duas metades e a segunda é a que custa. *"As bandas diferem"* ficaria verde numa
/// cena em que o espaço mudasse a AMPLITUDE — e aí o artista leria «o de baixo mexe mais»
/// em vez de «o campo virou», que é a coisa errada de ensinar. Então o gate pede que os
/// quatro padrões sejam distintos **e** que a excursão deles fique na mesma ordem.
#[test]
fn every_band_is_a_different_field_and_none_is_merely_louder() {
    let bands: Vec<Vec<f32>> = (0..4).map(band_delta).collect();
    for (i, a) in bands.iter().enumerate() {
        for (j, b) in bands.iter().enumerate().skip(i + 1) {
            let d = worst(a, b);
            assert!(
                d > 0.05,
                "as bandas {} e {} tem de amostrar o campo em sitios diferentes, e diferem {d}",
                i + 1,
                j + 1
            );
        }
    }
    let spans: Vec<f32> = bands.iter().map(|b| span(b)).collect();
    let (lo, hi) = spans
        .iter()
        .fold((f32::MAX, f32::MIN), |(l, h), x| (l.min(*x), h.max(*x)));
    assert!(
        hi < lo * 2.0,
        "nenhuma banda pode ser «mais agitada»: as excursoes vao de {lo} a {hi}"
    );
}

/// **A BANDA 3 É ANISOTRÓPICA e o CONTROLE não é** — a metade que prova o `scale_y`.
///
/// O oráculo é a razão entre o quanto o campo varia ao longo de X e ao longo de Y, medida
/// por vizinhos: um campo isotrópico varia igual nos dois eixos.
///
/// ⚠️ **A DIREÇÃO é contra-intuitiva e a medição a corrigiu.** Um `scale_y` MAIOR faz o
/// mesmo passo de mundo cobrir mais espaço de ruído, então o campo varia **mais depressa**
/// em Y e as manchas ficam **baixas e largas** — listras deitadas. Logo `dx/dy` tem de
/// **CAIR**, não subir. A primeira versão deste gate pediu o contrário e reprovou sobre
/// código correcto (0,341 contra 0,976 do controle).
///
/// ⚠️ Sem o controle, *"a banda 3 varia diferente nos dois eixos"* ficaria verde sobre um
/// campo de ruído qualquer — um Perlin de uma oitava **não** é perfeitamente isotrópico numa
/// amostra finita, e é por isso que a barra é uma RAZÃO ENTRE BANDAS, não um valor absoluto.
#[test]
fn the_stretched_band_is_anisotropic_where_the_control_is_not() {
    let side = 15usize;
    let ratio = |band: usize| {
        let d = band_delta(band);
        let mut dx = 0.0f32;
        let mut dy = 0.0f32;
        for r in 0..side {
            for c in 0..side {
                let i = r * side + c;
                if c + 1 < side {
                    dx += (d[i + 1] - d[i]).abs();
                }
                if r + 1 < side {
                    dy += (d[i + side] - d[i]).abs();
                }
            }
        }
        dx / dy.max(1e-6)
    };
    let plain = ratio(0);
    let stretched = ratio(2);
    assert!(
        (plain - 1.0).abs() < 0.6,
        "o CONTROLE tem de variar parecido nos dois eixos, e a razao e' {plain}"
    );
    assert!(
        stretched < plain * 0.6,
        "a banda esticada tem de variar MUITO mais em Y do que em X: {stretched} contra {plain}"
    );
    eprintln!("[=60] razao dx/dy: controle {plain:.3}, esticada {stretched:.3}");
}

/// **A ORDEM importa: «esticar e rodar» ≠ «rodar e esticar»** — e a banda 4 é a primeira.
///
/// ⚠️ Este é o gate que defende a lei escrita no `FieldSpace::at`. Ele constrói a ordem
/// CONTRÁRIA à mão (rodar o ponto e só então esticar o Y) e exige que ela dê outro campo —
/// se as duas coincidissem, a ordem seria uma escolha sem consequência e o comentário que a
/// justifica seria uma nota a envelhecer.
#[test]
fn stretch_then_rotate_is_not_rotate_then_stretch() {
    let (turn, scale, scale_y) = knobs();
    let ph = turn / 360.0;
    // A mesma senoide parabólica dos dois lados — aqui só para construir o CONTRA-exemplo.
    let sin_c = |p: f32| {
        let f = p - p.floor();
        let q = if f < 0.5 {
            let u = f * 2.0;
            4.0 * u * (1.0 - u)
        } else {
            let u = (f - 0.5) * 2.0;
            -4.0 * u * (1.0 - u)
        };
        0.225 * (q * q.abs() - q) + q
    };
    let (c, s) = (sin_c(ph + 0.25), sin_c(ph));
    // Um ponto qualquer FORA dos eixos — nos eixos as duas ordens coincidem por simetria.
    let (px, py) = (1.7f32, 0.9f32);
    let ours = {
        let (x, y) = (px * scale, py * scale_y);
        (x * c - y * s, x * s + y * c)
    };
    let other = {
        let (x, y) = (px * c - py * s, px * s + py * c);
        (x * scale, y * scale_y)
    };
    let d = (ours.0 - other.0).abs().max((ours.1 - other.1).abs());
    assert!(
        d > 0.05,
        "as duas ordens tem de dar pontos de amostragem diferentes, e diferem {d}"
    );
}

/// **A sonda que a mensagem cita** — ela imprime, não afirma.
#[test]
#[ignore = "sonda: imprime os numeros que a mensagem da cena cita"]
fn measure_what_the_scene_shows() {
    eprintln!("\n[=60] o que a cena monta");
    for (i, label) in band_labels() {
        let d = band_delta(i);
        eprintln!(
            "  banda {}: {} pecas, excursao {:.4}  ({label})",
            i + 1,
            d.len(),
            span(&d)
        );
    }
}
