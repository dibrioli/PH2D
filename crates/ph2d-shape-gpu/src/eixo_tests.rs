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
