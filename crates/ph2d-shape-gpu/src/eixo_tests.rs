use super::*;
use ph2d_vector::{Circle, Shape};

fn itens(bp: &BezPath, st: &Stroke) -> (Vec<EixoItem>, f32) {
    let mut v = Vec::new();
    let ext = eixo(bp, st, 1.0e-3, &mut v);
    (v, ext)
}

fn conta(v: &[EixoItem], tipo: u32) -> usize {
    v.iter().filter(|i| i.tipo == tipo).count()
}

/// Um caminho ABERTO com uma quina: dois troços, UMA junta com o estilo, e as DUAS pontas — e a
/// extensão para fora é o limite da esquadria.
#[test]
fn um_aberto_com_quina_leva_troços_junta_e_pontas() {
    let mut bp = BezPath::new();
    bp.move_to((0.0, 0.0));
    bp.line_to((1.0, 0.0));
    bp.line_to((1.0, 1.0));
    let st = Stroke::new(0.1)
        .with_join(Join::Miter)
        .with_caps(Cap::Square);
    let (v, ext) = itens(&bp, &st);
    assert_eq!(conta(&v, ITEM_TROCO), 2);
    assert_eq!(conta(&v, ITEM_JUNTA), 1);
    assert_eq!(conta(&v, ITEM_PONTA), 2);
    let j = v.iter().find(|i| i.tipo == ITEM_JUNTA).expect("a junta");
    assert_eq!(j.junta, JUNTA_ESQUADRIA, "a quina leva o estilo");
    assert_eq!(j.b, [1.0, 0.0]);
    assert!(
        v.iter()
            .filter(|i| i.tipo == ITEM_PONTA)
            .all(|i| i.ponta == PONTA_QUADRADA)
    );
    #[expect(clippy::cast_possible_truncation, reason = "o registo é f32")]
    let limite = st.miter_limit as f32;
    assert_eq!(ext, limite);
    assert!((v[0].meia_largura - 0.05).abs() < 1e-7);
}

/// Um quadrado FECHADO: quatro troços e quatro juntas (a do fecho incluída), nenhuma ponta.
#[test]
fn um_fechado_nao_tem_pontas_e_junta_o_fecho() {
    let mut bp = BezPath::new();
    bp.move_to((0.0, 0.0));
    bp.line_to((1.0, 0.0));
    bp.line_to((1.0, 1.0));
    bp.line_to((0.0, 1.0));
    bp.close_path();
    let (v, _) = itens(&bp, &Stroke::new(0.1).with_join(Join::Bevel));
    assert_eq!(conta(&v, ITEM_TROCO), 4);
    assert_eq!(conta(&v, ITEM_JUNTA), 4);
    assert_eq!(conta(&v, ITEM_PONTA), 0);
    assert!(
        v.iter()
            .filter(|i| i.tipo == ITEM_JUNTA)
            .all(|i| i.junta == JUNTA_CHANFRO),
        "as quatro quinas levam o estilo"
    );
    assert!(
        v.iter().any(|i| i.tipo == ITEM_JUNTA && i.b == [0.0, 0.0]),
        "o fecho tem junta no ponto 0"
    );
}

/// ⭐ **Dentro de uma curva a junta é REDONDA, seja qual for o estilo** — um círculo (quatro
/// cúbicas lisas) não tem quina nenhuma, logo nenhuma junta leva a esquadria. ⚠️ Sem isto cada
/// corda de uma curva ganharia uma esquadria, e a esquadria de um ângulo pequeno cresce até ao
/// limite dela.
#[test]
fn dentro_de_uma_curva_a_junta_e_redonda() {
    let bp = Circle::new((0.0, 0.0), 0.5).to_path(0.1);
    let (v, ext) = itens(&bp, &Stroke::new(0.1).with_join(Join::Miter));
    let juntas: Vec<_> = v.iter().filter(|i| i.tipo == ITEM_JUNTA).collect();
    assert!(juntas.len() > 8, "o circulo aplana em muitas cordas");
    assert!(juntas.iter().all(|i| i.junta == JUNTA_REDONDA));
    assert_eq!(
        conta(&v, ITEM_TROCO),
        juntas.len(),
        "fechado: uma junta por troço"
    );
    assert_eq!(ext, 1.0, "sem quina, nada vai além da caneta");
}

/// ⛔ **O tracejado marca a geometria e não dá eixo.**
#[test]
fn o_tracejado_so_se_desenha_conforme() {
    let mut bp = BezPath::new();
    bp.move_to((0.0, 0.0));
    bp.line_to((1.0, 0.0));
    let st = Stroke::new(0.1).with_dashes(0.0, [0.1, 0.1]);
    let g = crate::ShapeGeometry::prepare(&crate::ShapeInput {
        fill: None,
        strokes: vec![crate::StrokeInput {
            path: &bp,
            style: &st,
            color: [0.0, 0.0, 0.0, 1.0],
        }],
        stroke_fills: Vec::new(),
    })
    .expect("prepara");
    assert_ne!(g.record.flags & crate::FLAG_SO_CONFORME, 0);
    assert!(g.eixo.is_empty());
    assert!(g.record.eixo.iter().all(|e| e[1] == 0));
    // CONTROLO: o mesmo traço sem tracejado dá eixo, em todo nível.
    let liso = Stroke::new(0.1);
    let g = crate::ShapeGeometry::prepare(&crate::ShapeInput {
        fill: None,
        strokes: vec![crate::StrokeInput {
            path: &bp,
            style: &liso,
            color: [0.0, 0.0, 0.0, 1.0],
        }],
        stroke_fills: Vec::new(),
    })
    .expect("prepara");
    assert_eq!(g.record.flags & crate::FLAG_SO_CONFORME, 0);
    assert!(g.record.eixo.iter().all(|e| e[1] > 0));
}

/// ⭐ **Os BLOCOS guardam as peças todas, pela ordem, e a caixa de cada um cobre o alcance delas.**
///
/// ⚠️ O shader salta um bloco pela caixa do cabeçalho: uma caixa curta de mais apaga peças que
/// tocam no pixel (buracos no traço), um alcance curto apaga a ponta de uma esquadria. As duas
/// metades são medidas contra as próprias peças.
#[test]
fn os_blocos_guardam_as_pecas_e_cobrem_o_alcance_delas() {
    let mut st = Stroke::new(0.1);
    st.join = Join::Miter;
    st.miter_limit = 4.0;
    st.start_cap = Cap::Square;
    st.end_cap = Cap::Square;
    let mut bp = BezPath::new();
    bp.move_to((0.0, 0.0));
    for k in 1..40 {
        bp.line_to((f64::from(k) * 0.1, if k % 2 == 0 { 0.0 } else { 0.3 }));
    }
    let (pecas, _) = itens(&bp, &st);
    let blocos = em_blocos(&pecas);
    let so_pecas: Vec<EixoItem> = blocos
        .iter()
        .filter(|i| i.tipo != ITEM_BLOCO)
        .copied()
        .collect();
    assert_eq!(
        so_pecas, pecas,
        "as peças atravessam os blocos todas e pela ordem"
    );
    let mut i = 0;
    let mut vistos = 0;
    while i < blocos.len() {
        let cab = blocos[i];
        assert_eq!(cab.tipo, ITEM_BLOCO, "o item {i} tinha de ser um cabeçalho");
        let n = cab._pad as usize;
        assert!((1..=PECAS_POR_BLOCO).contains(&n), "um bloco com {n} peças");
        for it in &blocos[i + 1..i + 1 + n] {
            let mut pts = vec![it.a, it.b];
            if it.tipo == ITEM_JUNTA {
                pts.push(it.c);
            }
            for q in pts {
                assert!(
                    (0..2).all(|k| cab.a[k] <= q[k] && q[k] <= cab.b[k]),
                    "a caixa do bloco nao cobre {q:?}"
                );
            }
            let fator = match it.tipo {
                ITEM_JUNTA if it.junta == JUNTA_ESQUADRIA => it.limite_esquadria,
                ITEM_PONTA if it.ponta == PONTA_QUADRADA => 1.5,
                _ => 1.0,
            };
            assert!(
                cab.meia_largura >= it.meia_largura * fator,
                "o alcance do bloco ({}) nao cobre a peça ({})",
                cab.meia_largura,
                it.meia_largura * fator
            );
        }
        vistos += 1;
        i += 1 + n;
    }
    assert!(
        vistos >= 10,
        "controlo: a fixtura tem de dar varios blocos ({vistos})"
    );
    assert!(
        blocos
            .iter()
            .any(|c| c.tipo == ITEM_BLOCO && (c.meia_largura - 0.05 * 4.0).abs() < 1e-6),
        "controlo: ha' um bloco cujo alcance e' o da esquadria (0,05 x 4)"
    );
}
