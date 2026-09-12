//! Gates da cena `=90` — **qual fatia, que eixo, que leque** (doc 89, folha 04).
//!
//! ⚠️ **Cada gate mede a DIFERENÇA entre os dois lados do par**, nunca um lado sozinho: o que
//! a cena promete ao Enio é *"a direita não é a esquerda, e é diferente ASSIM"*. Um gate sobre
//! um lado só ficaria verde no dia em que os dois voltassem a coincidir.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    reg
}

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_slice_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// As posições de uma banda, **já sem o deslocamento do quadrante** — as figuras dos dois
/// lados de um par são para ser comparadas FORMA a FORMA, e cada uma vive no seu canto.
fn shape(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> Vec<[f32; 2]> {
    let mut cook = Cook::new();
    let out = cook.cook(&doc.graph, reg, sink, 0.0).expect("cozinha");
    let Some(Column::Vec2(p)) = out[0].as_stream().get("P") else {
        panic!("a banda tem de emitir P");
    };
    let n = p.len() as f32;
    let c = p
        .iter()
        .fold([0.0_f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
    let c = [c[0] / n, c[1] / n];
    p.iter().map(|q| [q[0] - c[0], q[1] - c[1]]).collect()
}

/// A coluna `size` de uma banda.
fn sizes(doc: &MotionDoc, reg: &NodeRegistry, sink: NodeId) -> Vec<[f32; 2]> {
    let mut cook = Cook::new();
    let out = cook.cook(&doc.graph, reg, sink, 0.0).expect("cozinha");
    match out[0].as_stream().get("size") {
        Some(Column::Vec2(v)) => v.clone(),
        _ => Vec::new(),
    }
}

/// O maior desvio entre duas figuras já centradas.
fn apart(a: &[[f32; 2]], b: &[[f32; 2]]) -> f32 {
    assert_eq!(a.len(), b.len(), "os dois lados do par tem de ter n igual");
    a.iter()
        .zip(b)
        .map(|(p, q)| (p[0] - q[0]).abs().max((p[1] - q[1]).abs()))
        .fold(0.0_f32, f32::max)
}

/// **Quanto uma FAIXA se afasta de uma recta** — o desvio perpendicular máximo ao seu eixo
/// principal, em frações do comprimento nesse eixo. Uma faixa recta mede a própria espessura
/// relativa (pequena); uma faixa curvada mede a curvatura.
///
/// ⚠️ **A primeira versão desta régua media o desvio à CORDA do primeiro ao último ponto, e
/// reprovava três cenas correctas.** Aquela régua supõe uma CADEIA 1-D — e é legítima onde
/// nasceu, na crate do `motion.spline_wrap`, cuja fixtura é uma fileira de pontos. As bandas
/// desta cena são **grelhas 2-D**: o primeiro e o último ponto são cantos opostos, a "corda" é
/// uma diagonal, e o que ela mede é a espessura da faixa (`0,94` numa banda perfeitamente
/// recta). *Uma régua importada de outra fixtura mede a fixtura de onde veio.*
fn straightness(p: &[[f32; 2]]) -> f32 {
    let n = p.len() as f32;
    let c = p
        .iter()
        .fold([0.0_f32; 2], |a, q| [a[0] + q[0], a[1] + q[1]]);
    let c = [c[0] / n, c[1] / n];
    let (mut sxx, mut sxy, mut syy) = (0.0_f32, 0.0_f32, 0.0_f32);
    for q in p {
        let (x, y) = (q[0] - c[0], q[1] - c[1]);
        sxx += x * x;
        sxy += x * y;
        syy += y * y;
    }
    // O eixo principal da nuvem (o de maior variância).
    let th = 0.5 * (2.0 * sxy).atan2(sxx - syy);
    let (ct, st) = (th.cos(), th.sin());
    let (mut lo, mut hi, mut perp) = (f32::MAX, f32::MIN, 0.0_f32);
    for q in p {
        let (x, y) = (q[0] - c[0], q[1] - c[1]);
        let a = x * ct + y * st;
        lo = lo.min(a);
        hi = hi.max(a);
        perp = perp.max((-x * st + y * ct).abs());
    }
    let len = hi - lo;
    if len < 1e-6 { 0.0 } else { perp / len }
}

/// A maior distância entre dois pontos da figura.
fn extent(p: &[[f32; 2]]) -> f32 {
    let mut d = 0.0_f32;
    for a in p {
        for b in p {
            d = d.max((a[0] - b[0]).hypot(a[1] - b[1]));
        }
    }
    d
}

/// **A CENA MONTA AS DEZ BANDAS**, e todas cospem.
#[test]
fn the_slice_scene_builds_all_ten_bands() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 10, "cinco pares");
    assert_eq!(band_labels().count(), 10, "um rotulo por banda");
    for (k, &s) in sinks.iter().enumerate() {
        assert!(!shape(&doc, &reg, s).is_empty(), "banda {k} vazia");
    }
}

/// **OS DEZ LADOS SÃO DEZ FIGURAS** — o gate que apanha um par que colapsou (um param que
/// deixou de ser lido, um default que voltou), e que é a promessa da cena inteira.
#[test]
fn every_pair_shows_two_different_pictures() {
    let (doc, reg, sinks) = scene();
    for pair in 0..5 {
        let (a, b) = (
            shape(&doc, &reg, sinks[pair * 2]),
            shape(&doc, &reg, sinks[pair * 2 + 1]),
        );
        let d = apart(&a, &b);
        // O par 4 (afunilamento) mexe no `size`, não em `P` — ele tem o seu gate abaixo.
        if pair == 3 {
            assert!(d < 1e-5, "o afunilamento NAO pode mover as pecas ({d:.5})");
            continue;
        }
        assert!(d > 0.1, "o par {} coincidiu (desvio {d:.4})", pair + 1);
    }
}

/// ⭐ **O par 1: `Limited` ESTICA a figura, `Unlimited` enrola-a.**
///
/// A `150°` sobre a extensão inteira o `Unlimited` dá `300°` de ponta a ponta — a fileira
/// enrola-se quase num círculo, e fica COMPACTA. O `Limited` gasta a mesma volta na metade de
/// trás (a fatia re-escala a curvatura) e manda a metade da frente embora **num troço recto
/// pela tangente**, então a figura fica muito mais ESPALHADA.
///
/// ⚠️ **A régua é a extensão e não a rectidão de um pedaço**, porque separar *o pedaço de
/// fora* exigiria o índice da grelha, e a ordem dos pontos de uma banda é assunto do
/// `motion.grid`, não desta cena. A extensão pergunta o mesmo por fora: *a cauda foi-se
/// embora ou ficou enrolada?*
#[test]
fn the_limited_bend_throws_the_far_half_out_while_the_whole_bend_curls_up() {
    let (doc, reg, sinks) = scene();
    let curled = extent(&shape(&doc, &reg, sinks[0]));
    let thrown = extent(&shape(&doc, &reg, sinks[1]));
    assert!(
        thrown > curled * 1.3,
        "a dobra Limited tinha de espalhar mais que a inteira ({thrown:.2} contra {curled:.2})"
    );
    // CONTROLE: as duas desenham alguma coisa (uma figura vazia teria extensão 0).
    assert!(curled > 1.0 && thrown > 1.0, "as duas bandas desenham");
}

/// ⭐ **O par 3: a COLUNA sai reta no eixo de sempre e segue a curva com `Axis 90°`.** É o
/// defeito e a cura lado a lado, na cena.
#[test]
fn the_column_only_follows_the_curve_with_the_new_axis() {
    let (doc, reg, sinks) = scene();
    let plain = straightness(&shape(&doc, &reg, sinks[4]));
    let turned = straightness(&shape(&doc, &reg, sinks[5]));
    assert!(
        plain < 0.2,
        "sem o eixo, a coluna tinha de sair numa FAIXA recta ({plain:.4})"
    );
    assert!(
        turned > plain * 3.0,
        "com Axis 90 ela tinha de seguir o S ({turned:.4} contra {plain:.4})"
    );
}

/// ⭐ **O par 4: o afunilamento muda o TAMANHO e não o lugar.**
#[test]
fn the_taper_changes_the_size_and_nothing_else() {
    let (doc, reg, sinks) = scene();
    let flat = sizes(&doc, &reg, sinks[6]);
    let tapered = sizes(&doc, &reg, sinks[7]);
    assert!(
        !flat.is_empty() && !tapered.is_empty(),
        "os dois tem `size`"
    );
    // A esquerda é uniforme; a direita vai do grosso ao fino.
    let uniform = flat
        .iter()
        .map(|v| v[0])
        .fold((f32::MAX, f32::MIN), |(lo, hi), s| (lo.min(s), hi.max(s)));
    assert!(
        (uniform.1 - uniform.0).abs() < 1e-4,
        "a esquerda tinha de ser uniforme ({:.4}..{:.4})",
        uniform.0,
        uniform.1
    );
    let range = tapered
        .iter()
        .map(|v| v[0])
        .fold((f32::MAX, f32::MIN), |(lo, hi), s| (lo.min(s), hi.max(s)));
    assert!(
        range.1 > range.0 * 4.0,
        "a direita tinha de afunilar de verdade ({:.4}..{:.4})",
        range.0,
        range.1
    );
}

/// ⭐ **O par 5: o leque é um ANEL, a fila é uma RECTA.**
#[test]
fn the_fan_is_a_ring_and_the_row_is_a_line() {
    let (doc, reg, sinks) = scene();
    let row = shape(&doc, &reg, sinks[8]);
    let fan = shape(&doc, &reg, sinks[9]);
    assert_eq!(row.len(), fan.len(), "a mesma contagem dos dois lados");
    assert!(
        straightness(&row) < 0.2,
        "a fila Linear tinha de ser uma FAIXA recta ({:.4})",
        straightness(&row)
    );
    assert!(
        straightness(&fan) > straightness(&row) * 2.0,
        "e o leque NAO ({:.4} contra {:.4})",
        straightness(&fan),
        straightness(&row)
    );
    // O leque: as peças espalham-se nos DOIS eixos, e a fila só num.
    let axes = |p: &[[f32; 2]]| {
        let f = |a: usize| {
            p.iter().fold((f32::MAX, f32::MIN), |(lo, hi), q| {
                (lo.min(q[a]), hi.max(q[a]))
            })
        };
        let (x, y) = (f(0), f(1));
        ((x.1 - x.0), (y.1 - y.0))
    };
    let (fx, fy) = axes(&fan);
    let (_, ry) = axes(&row);
    assert!(
        fy > fx * 0.5,
        "o leque tinha de se abrir nos dois eixos ({fx:.2} x {fy:.2})"
    );
    assert!(
        fy > ry * 2.0,
        "e MUITO mais alto que a fila ({fy:.2} contra {ry:.2})"
    );
}

/// As fichas do canvas: uma por banda, acima dela, e nenhuma com o `--` do rótulo longo.
#[test]
fn every_band_carries_its_caption() {
    let caps = captions();
    assert_eq!(caps.len(), 10, "uma ficha por banda");
    for c in &caps {
        assert!(
            !c.text.contains("--"),
            "a ficha do canvas e' curta: {:?}",
            c.text
        );
        assert!(!c.text.is_empty(), "ficha vazia");
    }
}
