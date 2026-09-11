//! Gates da cena `=96` — o que um campo de valor não sabia dizer (folha 15).

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn scene() -> (MotionDoc, NodeRegistry, Vec<NodeId>) {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("os nos registram");
    let mut doc = MotionDoc::default();
    let sinks = build_value_demo_document(&mut doc, &reg).expect("a cena monta");
    doc.graph.validate(&reg).expect("bem-tipada");
    (doc, reg, sinks)
}

/// A pose e as duas colunas que esta cena conduz, por banda.
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
            // O `size` é `Vec2`; esta cena lê a LARGURA, que é o que ela conduz.
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
fn the_value_scene_builds_all_eight_bands() {
    let (doc, reg, sinks) = scene();
    assert_eq!(sinks.len(), 8, "quatro pares");
    assert_eq!(band_labels().count(), 8, "um rotulo por banda");
    for (k, (p, w)) in bake(&doc, &reg, &sinks).into_iter().enumerate() {
        assert_eq!(p.len(), PIECES as usize, "banda {k}: a fileira inteira");
        for q in &p {
            assert!(q[0].is_finite() && q[1].is_finite(), "banda {k} explodiu");
        }
        assert!(
            w.iter().all(|v| v.is_finite() && *v > 0.0),
            "banda {k}: alguma peca ficou sem tamanho"
        );
    }
}

/// ⭐⭐ **NENHUMA BANDA SAI DO SEU LUGAR** — a lei que a cena `=73` pagou com um smoke
/// ilegível (*"tudo misturado e bagunçado"*).
///
/// ⚠️ **Todo comportamento desta biblioteca é mascarado pelo `falloff`**, então um
/// deslocamento de colocação posto DEPOIS do campo vira `dx · falloff_i`: cada peça anda
/// uma distância diferente e a banda estica-se por cima das vizinhas. O gate mede a CAIXA
/// contra a prevista — `(n − 1) · passo`, **derivada** e nunca escrita ao lado.
#[test]
fn no_band_leaves_its_slot() {
    let (doc, reg, sinks) = scene();
    // A largura autorada da fileira, mais a folga que a peça mais gorda pode acrescentar.
    let want = (PIECES - 1.0) * GAP;
    for (k, (p, _)) in bake(&doc, &reg, &sinks).into_iter().enumerate() {
        let lo = p.iter().map(|q| q[0]).fold(f32::MAX, f32::min);
        let hi = p.iter().map(|q| q[0]).fold(f32::MIN, f32::max);
        let got = hi - lo;
        assert!(
            (got - want).abs() < 1e-3,
            "banda {k}: a fileira mede {got:.4} e a caixa autorada e' {want:.4} -- \
             a colocacao foi mascarada pelo campo?"
        );
        // E ela está do lado certo: o centro cai na coluna que lhe pertence.
        let mid = (lo + hi) * 0.5;
        let side = if k % 2 == 0 { -COL_X } else { COL_X };
        assert!(
            (mid - side).abs() < 1e-3,
            "banda {k}: o centro esta' em {mid:.3} e devia estar em {side:.3}"
        );
    }
}

/// ⭐⭐⭐ **CADA PAR MOSTRA UMA DIFERENÇA** — o que a cena existe para deixar ver.
///
/// ⚠️ **A régua é por LINHA e não global**: um gate que só somasse tudo passaria com três
/// pares idênticos e um muito diferente. E ela mede a grandeza que a linha CONDUZ — as três
/// primeiras conduzem o tamanho, a quarta a rotação.
#[test]
fn every_pair_shows_a_difference() {
    let (doc, reg, sinks) = scene();
    let baked = bake(&doc, &reg, &sinks);
    for row in 0..3 {
        let (l, r) = (&baked[row * 2].1, &baked[row * 2 + 1].1);
        let apart = l
            .iter()
            .zip(r.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f32, f32::max);
        assert!(
            apart > 0.02,
            "linha {row}: os dois lados desenham o mesmo tamanho (maior diferenca {apart:.4})"
        );
    }
}

/// ⭐⭐ **A ÚLTIMA LINHA: a magnitude quase não varre, e o EIXO varre.**
///
/// ⚠️ **Esta é a linha cuja diferença não está no tamanho mas na ROTAÇÃO**, e é a única em
/// que a régua tem de olhar outra coluna. A fixtura é deliberada: a largura cresce e a
/// altura encolhe, então `|size|` é quase constante ao longo da banda — *uma leitura polar
/// satisfaz «alcançável» e não devolve um eixo*.
#[test]
fn the_lane_sweeps_where_the_magnitude_does_not() {
    let (doc, reg, sinks) = scene();
    let mut cook = Cook::new();
    let span = |s: NodeId, cook: &mut Cook| {
        let out = cook.cook(&doc.graph, &reg, s, 0.0).expect("coze");
        match out[0].as_stream().get("rot") {
            Some(Column::Scalar(v)) => {
                v.iter().copied().fold(f32::MIN, f32::max)
                    - v.iter().copied().fold(f32::MAX, f32::min)
            }
            _ => 0.0,
        }
    };
    let polar = span(sinks[6], &mut cook);
    let lane = span(sinks[7], &mut cook);
    assert!(
        lane > polar * 3.0,
        "o EIXO tinha de varrer muito mais que a magnitude: {lane:.2} contra {polar:.2}"
    );
    // CONTROLE: a magnitude não é ZERO — ela mexe-se um pouco, e é isso que a torna uma
    // resposta plausível-e-errada em vez de um canal morto.
    assert!(
        polar > 0.0,
        "CONTROLE: a magnitude varia alguma coisa, senao o par nao mede o que diz"
    );
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
