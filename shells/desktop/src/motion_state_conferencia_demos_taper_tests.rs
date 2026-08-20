//! Os gates da cena `=64` — a fila e a mistura.
//!
//! ⚠️ **Cada par tem de SEPARAR, e pela grandeza que a banda anuncia** — não "as duas listas
//! diferem", que passaria por qualquer motivo. E as peças não se tapam: a lei que a cena `=63`
//! pagou (o `SIZE_IDENTITY` de 1,0 contra um passo menor) vale para toda cena desta série.

use super::*;
use ph2d_nodegraph::attr::Column;
use ph2d_nodegraph::cook::Cook;

fn registry() -> NodeRegistry {
    let mut reg = NodeRegistry::new();
    ph2d_node_registry_init::register_all_nodes(&mut reg).expect("todo nó registra");
    reg
}

/// Uma banda pelas colunas que o olho junta.
type Band = (Vec<[f32; 2]>, Vec<[f32; 2]>, Vec<f32>);

/// Coze a cena e devolve `(P, size, rot)` de cada banda.
fn bands() -> Vec<Band> {
    let reg = registry();
    let mut doc = MotionDoc::default();
    let sinks = build_taper_demo_document(&mut doc, &reg).expect("a cena monta");
    assert_eq!(sinks.len(), 4, "dois pares");
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
                Some(Column::Vec2(v)) => v.clone(),
                _ => Vec::new(),
            };
            let rot = match st.get("rot") {
                Some(Column::Scalar(v)) => v.clone(),
                _ => Vec::new(),
            };
            (p, size, rot)
        })
        .collect()
}

/// **O PAR DA FILA SEPARA — e pelas DUAS grandezas que a banda 2 anuncia.**
///
/// ⚠️ O oráculo do tamanho é a RAZÃO entre a última cópia e a primeira, não um valor absoluto:
/// o `motion.scale` que encolhe a peça multiplica as duas por igual, então a razão é a única
/// coisa que fala do taper e não do enquadramento.
#[test]
fn the_queue_pair_separates_on_both_taper_axes() {
    let b = bands();
    let (flat, tapered) = (&b[0], &b[1]);
    assert_eq!(flat.0.len(), COPIES as usize, "seis cópias");
    assert_eq!(tapered.0.len(), COPIES as usize);

    // A banda 1: todas do mesmo tamanho, e nenhuma girada.
    let w0: Vec<f32> = flat.1.iter().map(|s| s[0]).collect();
    assert!(
        w0.windows(2).all(|p| (p[0] - p[1]).abs() < 1e-6),
        "a fila sem taper tem de sair uniforme: {w0:?}"
    );
    assert!(
        flat.2.is_empty() || flat.2.iter().all(|r| *r == 0.0),
        "…e sem rotação: {:?}",
        flat.2
    );

    // A banda 2: a última mede um quarto da primeira, e girou o quarto de volta.
    let w1: Vec<f32> = tapered.1.iter().map(|s| s[0]).collect();
    let ratio = w1.last().expect("última") / w1.first().expect("primeira");
    assert!(
        (ratio - TAPER_SCALE).abs() < 1e-5,
        "a última cópia tem de medir {TAPER_SCALE} da primeira, e mede {ratio:.4}"
    );
    let r = &tapered.2;
    assert_eq!(r.len(), COPIES as usize, "a rotação existe");
    assert!(
        (r.last().expect("última") - TAPER_ROT).abs() < 1e-4 && r[0] == 0.0,
        "a volta corre de 0 a {TAPER_ROT}: {r:?}"
    );
    // ⚠️ E o CONTROLE da lei: o meio é o LERP, não a potência composta.
    let mid = w1[w1.len() / 2] / w1[0];
    assert!(
        mid > 0.5,
        "a cópia do meio mede {mid:.3} da primeira — abaixo de 0,5 seria a lei composta"
    );
}

/// **O PAR DA MISTURA SEPARA — e pela INCLINAÇÃO, que é o que o peso muda.**
///
/// ⚠️ Com pesos iguais a fileira e a coluna contribuem o mesmo, então a linha é a diagonal
/// exacta (`|dy/dx| = 1`). Com a coluna a pesar o triplo, `|dy/dx| = 3`. Medir a inclinação e
/// não "as posições diferem" é o que torna o gate uma afirmação sobre o PESO.
#[test]
fn the_mix_pair_separates_on_the_slope_the_weight_sets() {
    let b = bands();
    let slope = |band: &Band| {
        let p = &band.0;
        let (a, z) = (p.first().expect("primeiro"), p.last().expect("último"));
        let (dx, dy) = (z[0] - a[0], z[1] - a[1]);
        assert!(dx.abs() > 1e-6, "a linha não pode ser vertical exacta");
        (dy / dx).abs()
    };
    let even = slope(&b[2]);
    let heavy = slope(&b[3]);
    assert!(
        (even - 1.0).abs() < 1e-4,
        "pesos iguais dão a diagonal exacta, e deram {even:.4}"
    );
    assert!(
        (heavy - HEAVY).abs() < 1e-4,
        "com a coluna a pesar {HEAVY} a inclinação tem de ser {HEAVY}, e é {heavy:.4}"
    );
    assert_eq!(b[2].0.len(), POINTS as usize, "sete pontos");
    assert_eq!(
        b[3].0.len(),
        POINTS as usize,
        "e o peso não muda a contagem"
    );
}

/// **NENHUMA PEÇA ESCONDE OUTRA** — a lei da cena `=63`, medida no cozido de cada banda.
///
/// ⚠️ A régua é o maior lado desenhado contra a menor distância entre duas peças da banda. Uma
/// coluna `size` ausente **reprova**: sem ela a peça é `SIZE_IDENTITY` = 1,0, e é exactamente
/// o estado em que a cena irmã reprovou num smoke.
#[test]
fn no_piece_is_wide_enough_to_hide_its_neighbour() {
    for (i, (p, size, _)) in bands().iter().enumerate() {
        assert!(!size.is_empty(), "a banda {i} tem de trazer `size`");
        let mut nearest = f32::INFINITY;
        for (a, q) in p.iter().enumerate() {
            for r in p.iter().skip(a + 1) {
                let (dx, dy) = ((q[0] - r[0]).abs(), (q[1] - r[1]).abs());
                nearest = nearest.min(dx.max(dy));
            }
        }
        let widest = size.iter().fold(0.0f32, |m, s| m.max(s[0]).max(s[1]));
        assert!(
            widest <= nearest,
            "banda {i}: peça {widest:.3} contra a vizinha mais próxima a {nearest:.3}"
        );
    }
}
