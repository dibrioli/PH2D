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

/// Um caminho ABERTO com uma quina: dois troços que se encontram NELA (com os bits de quina dos dois
/// lados) e as DUAS pontas — sem peça de junta — e a extensão para fora é o limite da esquadria.
#[test]
fn um_aberto_com_quina_leva_troços_e_pontas() {
    let mut bp = BezPath::new();
    bp.move_to((0.0, 0.0));
    bp.line_to((1.0, 0.0));
    bp.line_to((1.0, 1.0));
    let st = Stroke::new(0.1)
        .with_join(Join::Miter)
        .with_caps(Cap::Square);
    let (v, ext) = itens(&bp, &st);
    assert_eq!(conta(&v, ITEM_TROCO), 2);
    assert_eq!(conta(&v, ITEM_PONTA), 2);
    assert_eq!(v.len(), 4, "nenhuma peça de junta");
    let t: Vec<_> = v.iter().filter(|i| i.tipo == ITEM_TROCO).collect();
    assert_eq!(t[0].ponta, FAIXA_FIM | QUINA_FIM, "o 1.º chega à quina");
    assert_eq!(t[1].ponta, FAIXA_INICIO | QUINA_INICIO, "o 2.º sai dela");
    assert_eq!(t[0].b, [1.0, 0.0]);
    assert_eq!(t[0].c, [1.0, 1.0], "quem chega lê o vizinho da frente");
    assert_eq!(t[1].d, [0.0, 0.0], "quem sai lê o de trás");
    assert!(
        t.iter().all(|i| i.junta == JUNTA_ESQUADRIA),
        "a quina leva o estilo"
    );
    assert!(t.iter().copied().all(alcanca_a_esquadria));
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

/// Um quadrado FECHADO: quatro troços, as quatro quinas marcadas nos dois lados (a do fecho
/// incluída), nenhuma ponta.
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
    assert_eq!(conta(&v, ITEM_PONTA), 0);
    assert_eq!(v.len(), 4, "nenhuma peça de junta");
    let tudo = FAIXA_INICIO | FAIXA_FIM | QUINA_INICIO | QUINA_FIM;
    assert!(
        v.iter()
            .all(|i| i.ponta == tudo && i.junta == JUNTA_CHANFRO),
        "as quatro quinas, nos dois lados, com o estilo"
    );
    assert!(
        v.iter().any(|i| i.b == [0.0, 0.0] && i.c == [1.0, 0.0]),
        "o fecho: quem chega ao ponto 0 lê o 1.º troço como vizinho"
    );
    assert!(
        !v.iter().any(alcanca_a_esquadria),
        "o chanfro não vai além da caneta"
    );
}

/// ⭐ **Dentro de uma curva NÃO há junta: há FAIXA** (doc 121 §9.4). Um círculo (quatro cúbicas
/// lisas) não tem quina nenhuma, logo nenhuma peça de junta — cada troço leva os DOIS vizinhos e os
/// dois bits lisos, e o shader fá-lo acabar na bissectriz que o troço seguinte também usa.
/// ⚠️ Sem isto cada corda de uma curva ganharia uma peça própria (era assim até 01/10: uma junta
/// redonda por corda, o grosso do traço esticado no proxy de telemóvel).
#[test]
fn dentro_de_uma_curva_nao_ha_junta_ha_faixa() {
    let bp = Circle::new((0.0, 0.0), 0.5).to_path(0.1);
    let (v, ext) = itens(&bp, &Stroke::new(0.1).with_join(Join::Miter));
    let troços: Vec<_> = v.iter().filter(|i| i.tipo == ITEM_TROCO).collect();
    assert!(troços.len() > 8, "o circulo aplana em muitas cordas");
    assert_eq!(v.len(), troços.len(), "só troços: sem quina e sem ponta");
    let n = troços.len();
    for (k, t) in troços.iter().enumerate() {
        assert_eq!(
            t.ponta,
            FAIXA_INICIO | FAIXA_FIM,
            "o troço {k} é liso dos dois lados"
        );
        assert_eq!(
            t.c,
            troços[(k + 1) % n].b,
            "o vizinho da frente do troço {k}"
        );
        assert_eq!(
            t.d,
            troços[(k + n - 1) % n].a,
            "o vizinho de trás do troço {k}"
        );
    }
    assert_eq!(ext, 1.0, "sem quina, nada vai além da caneta");
}

/// A faixa pára nas PONTAS e SABE das QUINAS: num aberto o 1.º troço não tem bit de início nem o
/// último de fim (lá moram as pontas), e os dois lados de uma quina levam o bit de quina — o shader
/// só a cobre com a bissectriz se a junta autorada for a esquadria.
#[test]
fn a_faixa_para_nas_pontas_e_sabe_das_quinas() {
    let mut bp = BezPath::new();
    bp.move_to((0.0, 0.0));
    bp.quad_to((0.5, 0.4), (1.0, 0.0));
    bp.line_to((1.0, -1.0));
    let (v, _) = itens(&bp, &Stroke::new(0.05).with_join(Join::Bevel));
    let troços: Vec<_> = v.iter().filter(|i| i.tipo == ITEM_TROCO).collect();
    assert!(troços.len() >= 3, "a quadrática aplana em várias cordas");
    let ultimo = troços.len() - 1;
    assert_eq!(
        troços[0].ponta & FAIXA_INICIO,
        0,
        "o 1.º troço começa numa ponta"
    );
    assert_eq!(
        troços[ultimo].ponta & FAIXA_FIM,
        0,
        "o último acaba numa ponta"
    );
    let quina = troços
        .iter()
        .position(|t| t.b == [1.0, 0.0])
        .expect("o troço que chega à quina");
    assert_eq!(
        troços[quina].ponta & QUINA_FIM,
        QUINA_FIM,
        "chega à quina e sabe-o"
    );
    assert_eq!(
        troços[quina + 1].ponta & QUINA_INICIO,
        QUINA_INICIO,
        "sai da quina e sabe-o"
    );
    // CONTROLO: dentro da curva o vértice é liso — a régua não lê «quina» em todo lado.
    assert_eq!(
        troços[0].ponta, FAIXA_FIM,
        "liso entre as cordas da curva, ponta atrás"
    );
}

/// ⚠️ Dois pontos distintos em `f64` que caem no MESMO `f32` não dão um troço de comprimento zero
/// (a faixa partir-se-ia lá): a deduplicação é sobre o que a placa lê.
#[test]
fn um_troço_nunca_tem_comprimento_zero_na_placa() {
    let mut bp = BezPath::new();
    bp.move_to((0.0, 0.0));
    bp.line_to((1.0, 0.0));
    bp.line_to((1.0 + 1.0e-12, 0.0));
    bp.line_to((2.0, 0.5));
    let (v, _) = itens(&bp, &Stroke::new(0.1));
    assert!(
        v.iter()
            .filter(|i| i.tipo == ITEM_TROCO)
            .all(|i| i.a != i.b),
        "um troço de comprimento zero chegou à placa"
    );
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
    // `80` vértices: sem peças de junta (§9.4) são `79` troços e as duas pontas — dez blocos.
    for k in 1..80 {
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
            for q in [it.a, it.b] {
                assert!(
                    (0..2).all(|k| cab.a[k] <= q[k] && q[k] <= cab.b[k]),
                    "a caixa do bloco nao cobre {q:?}"
                );
            }
            let fator = match it.tipo {
                ITEM_TROCO if alcanca_a_esquadria(it) => it.limite_esquadria,
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
