//! Os gates da cena `=66` — a família dos campos.
//!
//! ⚠️ **Cada par separa pela grandeza que a banda anuncia**, e o oráculo de cada um é
//! escolhido para falsificar a lei ERRADA, não só para ver que dois números diferem.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// `(P, size)` de cada banda, na ordem em que a cena as monta.
fn bands() -> Vec<(Vec<[f32; 2]>, Vec<f32>)> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_field_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 6, "três pares");
    doc.graph.validate(&reg).expect("bem-tipado");
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(|s| {
            let v = cook.cook(&doc.graph, &reg, *s, 0.0).expect("a banda coze");
            let st = v[0].as_stream();
            let p = match st.get("P") {
                Some(Column::Vec2(v)) => v.clone(),
                _ => Vec::new(),
            };
            let size = match st.get("size") {
                Some(Column::Vec2(v)) => v.iter().map(|q| q[0]).collect(),
                _ => Vec::new(),
            };
            (p, size)
        })
        .collect()
}

/// O tamanho da peça mais próxima de `(x, y)` **relativo ao centro da banda**.
fn near(band: &(Vec<[f32; 2]>, Vec<f32>), dx: f32, dy: f32) -> f32 {
    let (p, size) = band;
    // O centro da banda é a média das posições (a grelha é centrada e o `move` desloca-a
    // inteira), então a sonda é relativa a ele e não depende do layout.
    let n = p.len() as f32;
    let c = p
        .iter()
        .fold([0.0f32, 0.0], |a, q| [a[0] + q[0], a[1] + q[1]]);
    let (cx, cy) = (c[0] / n, c[1] / n);
    let d2 = |q: &[f32; 2]| (q[0] - cx - dx).powi(2) + (q[1] - cy - dy).powi(2);
    let k = p
        .iter()
        .enumerate()
        .min_by(|a, b| d2(a.1).total_cmp(&d2(b.1)))
        .expect("há peças")
        .0;
    size[k]
}

/// **O ANEL ESVAZIA O MEIO E MANTÉM A COROA** — as duas metades.
///
/// ⚠️ A segunda é a que separa o anel de *"o campo encolheu"*: na coroa (a meio caminho entre
/// o buraco e a borda) o par tem de CONCORDAR, senão o `inner` estaria a mexer no raio
/// externo em vez de abrir um buraco.
#[test]
fn the_ring_empties_the_middle_and_keeps_the_crown() {
    let b = bands();
    let (disc, ring) = (&b[0], &b[1]);
    assert!(
        near(disc, 0.0, 0.0) > near(ring, 0.0, 0.0) + 0.05,
        "no centro o disco cresce e o anel não: {} vs {}",
        near(disc, 0.0, 0.0),
        near(ring, 0.0, 0.0)
    );
    let crown = (INNER + RADIUS) * 0.5;
    assert!(
        (near(disc, crown, 0.0) - near(ring, crown, 0.0)).abs() < 1e-4,
        "na coroa os dois têm de concordar — o buraco não pode mexer na borda externa"
    );
}

/// **O SINAL INVERTE QUEM CRESCE MAIS** — e não é o `invert`.
///
/// ⚠️ Com `+1` a caixa é o relevo e o fundo fica em repouso; com `−1` a caixa fica em repouso
/// e o FUNDO sobe acima dela. O oráculo compara os dois sítios nas duas bandas: a ORDEM entre
/// eles tem de trocar. Um `invert` daria a mesma troca **dentro** da faixa e o fundo pararia
/// no tamanho que a caixa tinha; aqui ele **ultrapassa-o**, e é isso que a última linha mede.
#[test]
fn the_sign_swaps_which_side_rises_and_the_outside_overshoots() {
    let b = bands();
    let (pos, neg) = (&b[2], &b[3]);
    let (in_p, out_p) = (near(pos, 0.0, 0.0), near(pos, 3.6, 0.0));
    let (in_n, out_n) = (near(neg, 0.0, 0.0), near(neg, 3.6, 0.0));
    assert!(
        in_p > out_p + 0.05,
        "com +1 a caixa cresce: {in_p} vs {out_p}"
    );
    assert!(
        out_n > in_n + 0.05,
        "com −1 o fundo cresce: {out_n} vs {in_n}"
    );
    assert!(
        out_n > in_p + 0.05,
        "e ele passa do que a caixa media com +1 ({in_p}) — é overshoot, não inversão: {out_n}"
    );
}

/// **SEM TRUNCAR, O CRUZAMENTO CRESCE MAIS** — e só o cruzamento.
///
/// ⚠️ As duas metades: no meio (onde as duas caixas se somam) o par tem de separar; num sítio
/// coberto por **uma só** caixa ele tem de concordar, senão o toggle estaria a mudar a soma
/// inteira em vez de só a parte que saturava.
#[test]
fn only_the_overlap_separates_when_the_clamp_comes_off() {
    let b = bands();
    let (clamped, loose) = (&b[4], &b[5]);
    let mid_c = near(clamped, 0.0, 0.0);
    let mid_l = near(loose, 0.0, 0.0);
    assert!(
        mid_l > mid_c + 0.1,
        "o cruzamento tem de crescer mais sem o clamp: {mid_c} vs {mid_l}"
    );
    // Longe do cruzamento, onde só uma caixa alcança, a soma nunca passou de 1.
    let side_c = near(clamped, -3.6, 0.0);
    let side_l = near(loose, -3.6, 0.0);
    assert!(
        (side_c - side_l).abs() < 1e-4,
        "fora do cruzamento o toggle não pode mudar nada: {side_c} vs {side_l}"
    );
}

/// **NENHUMA PEÇA ESCONDE OUTRA, nem no pico** — a lei da cena `=63`, aqui contra o maior
/// tamanho que QUALQUER das seis bandas produz (o cruzamento sem truncar).
#[test]
fn no_piece_is_wide_enough_to_hide_its_neighbour() {
    for (i, (p, size)) in bands().iter().enumerate() {
        assert!(!size.is_empty(), "a banda {i} tem de trazer `size`");
        let mut nearest = f32::INFINITY;
        for (a, q) in p.iter().enumerate() {
            for r in p.iter().skip(a + 1) {
                nearest = nearest.min((q[0] - r[0]).abs().max((q[1] - r[1]).abs()));
            }
        }
        let widest = size.iter().fold(0.0f32, |m, s| m.max(*s));
        assert!(
            widest <= nearest,
            "banda {i}: peça {widest:.3} contra o passo {nearest:.3}"
        );
    }
}
