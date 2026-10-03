use super::*;
use ph2d_vector::{BezPath, Circle, Shape, Stroke};

/// Um círculo como o `ph2d-vec-scene` o constrói: QUATRO cúbicas. ⚠️ Um `Circle::to_path(1e-9)`
/// dá dezenas de arcos, e o aplanamento emite pelo menos um segmento por curva — a fixtura mediria
/// o chão da ENTRADA e não o nível.
fn circulo() -> BezPath {
    Circle::new((0.0, 0.0), 0.5).to_path(0.1)
}

fn so_preenchido(bp: &BezPath) -> ShapeGeometry {
    ShapeGeometry::prepare(&ShapeInput {
        fill: Some((bp, FillRule::NonZero)),
        strokes: Vec::new(),
        stroke_fills: Vec::new(),
    })
    .expect("um circulo desenha")
}

/// A área de um polígono fechado pela fórmula do laço — a régua independente do aplanamento.
fn area_do_nivel(g: &ShapeGeometry, nivel: usize, stroke: bool) -> f64 {
    let [fs, fc, ss, sc] = g.record.ranges[nivel];
    let (ini, n) = if stroke { (ss, sc) } else { (fs, fc) };
    // Soma de ∫ x dy sobre os segmentos: a área (com sinal) de contornos fechados.
    g.segments[ini as usize..(ini + n) as usize]
        .iter()
        .map(|s| {
            let (x0, y0, x1, y1) = (
                f64::from(s[0]),
                f64::from(s[1]),
                f64::from(s[2]),
                f64::from(s[3]),
            );
            0.5 * (x0 + x1) * (y1 - y0)
        })
        .sum::<f64>()
        .abs()
}

/// ⭐ **Cada nível erra no máximo a sua tolerância, e os níveis CONVERGEM para a curva.**
///
/// A régua é a ÁREA do círculo (`π r²`), que não depende do aplanamento: um polígono inscrito de
/// flecha `h` perde no máximo `perímetro × h` de área. ⚠️ As duas metades: o erro de cada nível
/// cabe na barra dele **e** o nível mais fino é estritamente melhor que o mais grosso — sem a
/// segunda, uma geometria com os oito níveis iguais (um aplanamento só, copiado) passaria.
#[test]
fn cada_nivel_cabe_na_sua_tolerancia_e_os_niveis_convergem() {
    let bp = circulo();
    let g = so_preenchido(&bp);
    // ⚠️ A régua é a área da ENTRADA (quatro cúbicas), não a do círculo ideal: a cúbica já erra
    // `~3e-4` do raio, e um nível fino convergiria para ELA e nunca para `π r²`.
    let exacta = bp.area().abs();
    let perimetro = std::f64::consts::PI;
    let mut erros = Vec::new();
    for nivel in 0..LEVELS {
        let tol = f64::from(g.record.tol[nivel]);
        let erro = (area_do_nivel(&g, nivel, false) - exacta).abs();
        assert!(
            erro <= perimetro * tol,
            "nivel {nivel}: erro de area {erro:e} passa da barra {:e}",
            perimetro * tol
        );
        erros.push(erro);
    }
    assert!(
        erros[LEVELS - 1] < erros[0] * 1e-3,
        "os niveis nao convergem: {erros:?}"
    );
}

/// ⭐ **Os segmentos crescem de nível para nível, e o nível 0 é BARATO.** O passe escolhe o nível
/// mais grosso que serve; se todos fossem igualmente finos, uma cópia de `20 px` pagaria os
/// segmentos de uma de `20 000`.
#[test]
fn o_nivel_grosso_e_barato_e_o_fino_e_rico() {
    let g = so_preenchido(&circulo());
    let n0 = g.record.ranges[0][1];
    let n7 = g.record.ranges[LEVELS - 1][1];
    assert!(n0 <= 16, "o nivel 0 de um circulo tem {n0} segmentos");
    assert!(
        n7 >= 8 * n0,
        "o nivel fino ({n7}) nao e' mais rico que o grosso ({n0})"
    );
}

/// ⭐ **Um sub-caminho ABERTO fecha-se no preenchimento** — a regra do Vello. Sem isso um meio
/// círculo aberto teria área de um contorno que não fecha (a fórmula do laço daria outra coisa, e
/// o rasterizador pintaria uma faixa até ao infinito).
#[test]
fn um_caminho_aberto_fecha_se_no_preenchimento() {
    let mut bp = BezPath::new();
    bp.move_to((0.0, 0.0));
    bp.line_to((1.0, 0.0));
    bp.line_to((1.0, 1.0));
    // Sem `close_path` — um triângulo aberto.
    let g = so_preenchido(&bp);
    let a = area_do_nivel(&g, 0, false);
    assert!((a - 0.5).abs() < 1e-6, "o triangulo aberto deu area {a}");
}

/// ⛔⛔ **Uma aresta HORIZONTAL no espaço local fica na geometria** — porque a cópia RODA.
///
/// A régua `∫ x dy` é cega a uma aresta horizontal (ela contribui `dy = 0`), e é por isso que o
/// defeito passou: ele só existe no referencial do ECRÃ. ⇒ a área mede-se DEPOIS de rodar os
/// segmentos `30°`, que é o que a placa faz a uma cópia rodada. Sem as arestas de cima e de baixo,
/// o contorno rodado fica aberto e a área sai errada (medido: o gate de paridade do produto viu
/// riscos horizontais na caixa de todo rectângulo arredondado rodado).
#[test]
fn uma_aresta_horizontal_sobrevive_a_rotacao() {
    let mut bp = BezPath::new();
    bp.move_to((-0.5, -0.25));
    bp.line_to((0.5, -0.25));
    bp.line_to((0.5, 0.25));
    bp.line_to((-0.5, 0.25));
    bp.close_path();
    let g = so_preenchido(&bp);
    let [ini, n, _, _] = g.record.ranges[0];
    let (s, c) = 30f64.to_radians().sin_cos();
    let rodada: f64 = g.segments[ini as usize..(ini + n) as usize]
        .iter()
        .map(|seg| {
            let r = |x: f32, y: f32| {
                let (x, y) = (f64::from(x), f64::from(y));
                (c * x - s * y, s * x + c * y)
            };
            let ((x0, y0), (x1, y1)) = (r(seg[0], seg[1]), r(seg[2], seg[3]));
            0.5 * (x0 + x1) * (y1 - y0)
        })
        .sum::<f64>()
        .abs();
    assert!(
        (rodada - 0.5).abs() < 1e-6,
        "o rectangulo rodado deu area {rodada}, e nao 0,5"
    );
}

/// ⭐ **O traço é um preenchimento com a área de uma faixa:** um círculo de raio `r` com traço de
/// largura `w` cobre `2πr·w` (a coroa entre `r − w/2` e `r + w/2`). A régua é outra vez a área,
/// independente de como o contorno foi expandido.
#[test]
fn o_traco_expandido_cobre_a_coroa() {
    let bp = circulo();
    let estilo = Stroke::new(0.1);
    let g = ShapeGeometry::prepare(&ShapeInput {
        fill: None,
        strokes: vec![StrokeInput {
            path: &bp,
            style: &estilo,
            color: [1.0, 0.0, 0.0, 1.0],
        }],
        stroke_fills: Vec::new(),
    })
    .expect("um traco desenha");
    let exacta = 2.0 * std::f64::consts::PI * 0.5 * 0.1;
    let medida = area_do_nivel(&g, LEVELS - 1, true);
    // ⚠️ A área com sinal de uma coroa: os dois contornos têm sentidos opostos, e o laço dá a
    // DIFERENÇA — que é a coroa. Barra relativa, do tamanho do erro do nível fino.
    assert!(
        (medida - exacta).abs() < exacta * 1e-3,
        "a coroa mede {medida} contra {exacta}"
    );
    assert_eq!(
        g.record.ranges[0][1], 0,
        "sem preenchimento, zero segmentos de fill"
    );
    assert_eq!(g.record.stroke_color, [1.0, 0.0, 0.0, 1.0]);
}

/// ⭐ **A caixa cobre o TRAÇO, não só o caminho** — é ela que dimensiona o quad no ecrã, e um quad
/// da caixa do caminho cortaria a metade de fora do traço.
#[test]
fn a_caixa_cobre_o_traco() {
    let bp = circulo();
    let estilo = Stroke::new(0.2);
    let g = ShapeGeometry::prepare(&ShapeInput {
        fill: Some((&bp, FillRule::NonZero)),
        strokes: vec![StrokeInput {
            path: &bp,
            style: &estilo,
            color: [0.0; 4],
        }],
        stroke_fills: Vec::new(),
    })
    .expect("desenha");
    let [x0, _, x1, _] = g.record.bbox;
    assert!(
        x0 <= -0.59 && x1 >= 0.59,
        "a caixa {:?} nao cobre o traco",
        g.record.bbox
    );
}

/// Uma forma sem nada a desenhar devolve `None`, nunca um registo vazio que o passe teria de
/// adivinhar.
#[test]
fn uma_forma_vazia_nao_prepara() {
    let vazio = BezPath::new();
    assert!(
        ShapeGeometry::prepare(&ShapeInput {
            fill: Some((&vazio, FillRule::NonZero)),
            strokes: Vec::new(),
            stroke_fills: Vec::new(),
        })
        .is_none()
    );
}

/// O registo tem o tamanho que o `shape.wgsl` lê (`Record`: 352 bytes; o `Eixo`: 72). Um campo a mais num lado
/// só desalinharia TODOS os registos a seguir ao primeiro, em silêncio.
#[test]
fn o_registo_tem_o_tamanho_do_shader() {
    assert_eq!(std::mem::size_of::<GeometryRecord>(), 352);
    assert_eq!(std::mem::size_of::<crate::EixoItem>(), 72);
    assert_eq!(std::mem::size_of::<crate::ShapeInstance>(), 64);
    assert_eq!(std::mem::size_of::<crate::ShapeView>(), 32);
}

/// ⭐ doc 121 §9.3 — **Todo trecho começa e acaba num MÚLTIPLO do bloco, em todos os níveis.** O
/// shader lê o bloco `k` como `segs[8k..8k+8]` e soma-o pelas PONTAS quando está todo à esquerda:
/// um trecho desalinhado poria no mesmo bloco o fim do preenchimento e o começo do traço, e o
/// preenchimento somaria segmentos do traço. As três espécies de trecho (preenchimento · marcas ·
/// contorno) estão na fixtura, e há um bloco por cada `SEGS_POR_BLOCO` segmentos.
#[test]
fn todo_trecho_cai_em_blocos_inteiros() {
    let bp = circulo();
    let marca = circulo();
    let st = Stroke::new(0.05);
    let g = ShapeGeometry::prepare(&ShapeInput {
        fill: Some((&bp, FillRule::NonZero)),
        strokes: vec![StrokeInput {
            path: &bp,
            style: &st,
            color: [0.0, 0.0, 0.0, 1.0],
        }],
        stroke_fills: vec![&marca],
    })
    .expect("desenha");
    let b = crate::SEGS_POR_BLOCO as u32;
    for nivel in 0..LEVELS {
        let [fs, fc, ss, sc] = g.record.ranges[nivel];
        let marcas = g.record.eixo[nivel][2];
        for (nome, v) in [
            ("inicio do preenchimento", fs),
            ("preenchimento", fc),
            ("inicio do traco", ss),
            ("traco", sc),
            ("marcas", marcas),
        ] {
            assert_eq!(
                v % b,
                0,
                "nivel {nivel}: {nome} = {v} nao e' multiplo de {b}"
            );
        }
        assert!(
            fc > 0 && marcas > 0 && sc > marcas,
            "controlo: as tres especies existem"
        );
    }
    assert_eq!(g.segments.len() % crate::SEGS_POR_BLOCO, 0);
    assert_eq!(g.blocos.len(), g.segments.len() / crate::SEGS_POR_BLOCO);
}
