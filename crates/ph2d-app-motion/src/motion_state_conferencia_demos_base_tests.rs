//! Gates da cena `=97` — a base do ruído (folha 06 linha 21).

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = build_base_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// A pose e a largura de cada peça, por banda.
fn bake(doc: &MotionDoc, reg: &NodeRegistry, sinks: &[NodeId]) -> Vec<(Vec<[f32; 2]>, Vec<f32>)> {
    let mut cook = Cook::new();
    sinks
        .iter()
        .map(|&s| {
            let out = cook.cook(&doc.graph, reg, s, 0.0).expect("coze");
            let st = out[0].as_stream();
            let p = match st.get("P") {
                Some(Column::Vec2(v)) => v.clone(),
                _ => Vec::new(),
            };
            let w = match st.get("size") {
                Some(Column::Vec2(v)) => v.iter().map(|q| q[0]).collect(),
                Some(Column::Scalar(v)) => v.clone(),
                _ => Vec::new(),
            };
            (p, w)
        })
        .collect()
}

/// **A CENA MONTA AS OITO BANDAS**, e as oito cospem sem explodir.
#[test]
fn the_base_scene_builds_all_eight_bands() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 8, "quatro pares");
    assert_eq!(band_labels().count(), 8, "um rotulo por banda");
    let n = (SIDE * SIDE) as usize;
    for (k, (p, w)) in bake(&doc, &reg, &sinks).into_iter().enumerate() {
        assert_eq!(p.len(), n, "banda {k}: a grelha inteira");
        assert!(
            w.iter().all(|v| v.is_finite() && *v > 0.0),
            "banda {k}: alguma peca ficou sem tamanho"
        );
    }
}

/// ⭐⭐ **NENHUMA BANDA SAI DO SEU LUGAR** — a lei da cena `=73`, herdada.
#[test]
fn no_band_leaves_its_slot() {
    let (doc, reg, sinks) = scene();
    let want = (SIDE - 1.0) * GAP;
    for (k, (p, _)) in bake(&doc, &reg, &sinks).into_iter().enumerate() {
        let lo = p.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
        let hi = p.iter().map(|q| q[0]).fold(f32::MIN, f32::max);
        assert!(
            (hi - lo - want).abs() < 1e-3,
            "banda {k}: a grelha mede {:.4} e a caixa autorada e' {want:.4} -- \
             a colocacao foi mascarada pelo campo?",
            hi - lo
        );
        let mid = (lo + hi) * 0.5;
        let side = if k % 2 == 0 { -COL_X } else { COL_X };
        assert!(
            (mid - side).abs() < 1e-3,
            "banda {k}: o centro esta' em {mid:.3} e devia estar em {side:.3}"
        );
    }
}

/// ⭐⭐⭐ **CADA PAR DESENHA OUTRO CAMPO** — o que a cena existe para deixar ver.
///
/// ⚠️ **A régua é por LINHA**: um gate que somasse tudo passaria com três pares idênticos e
/// um muito diferente. E o `seed` é o MESMO nos oito de propósito — o que muda entre os dois
/// lados de um par é só a base (ou a métrica), senão o olho não sabe a que atribuir a
/// diferença, e o gate mediria a semente.
#[test]
fn every_pair_draws_another_field() {
    let (doc, reg, sinks) = scene();
    let baked = bake(&doc, &reg, &sinks);
    for row in 0..4 {
        let (l, r) = (&baked[row * 2].1, &baked[row * 2 + 1].1);
        let apart = l
            .iter()
            .zip(r.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f32, f32::max);
        assert!(
            apart > AMPLITUDE * 0.3,
            "linha {row}: os dois lados desenham o mesmo campo (maior diferenca {apart:.4})"
        );
    }
    // ⛔ CONTROLE: as bandas que repetem a MESMA configuração do lado esquerdo têm de ter a
    // mesma ESTATÍSTICA — senão alguma coisa além da base mudou entre elas, e a diferença de
    // cada par deixa de ser atribuível ao knob que a cena anuncia.
    //
    // ⚠️ **E não podem ser idênticas ELEMENTO A ELEMENTO, que foi o que escrevi primeiro:**
    // o `motion.noise` amostra a POSIÇÃO, e o `place` põe cada banda no seu quadrante antes
    // dele — duas bandas em linhas diferentes lêem partes diferentes do mesmo campo, por
    // construção. *Um controlo que o produto não pode satisfazer não é um controlo.*
    let span = |v: &Vec<f32>| {
        v.iter().copied().fold(f32::MIN, f32::max) - v.iter().copied().fold(f32::MAX, f32::min)
    };
    for (a, b, name) in [(0, 2, "Gradient"), (4, 6, "Cellular Euclidean")] {
        let (sa, sb) = (span(&baked[a].1), span(&baked[b].1));
        assert!(
            (sa - sb).abs() < sa.max(sb) * 0.25,
            "as duas bandas `{name}` da esquerda tinham de ter a mesma excursao              (campo estacionario): {sa:.4} contra {sb:.4}"
        );
    }
}

/// As fichas do canvas: uma por banda, curta.
#[test]
fn every_band_carries_its_caption() {
    let caps = captions();
    assert_eq!(caps.len(), 8, "uma ficha por banda");
    for c in &caps {
        assert!(!c.text.contains("--"), "a ficha e' curta: {:?}", c.text);
        assert!(!c.text.is_empty(), "ficha vazia");
    }
}
