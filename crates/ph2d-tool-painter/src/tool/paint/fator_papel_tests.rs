//! B34.1 — o papel à escala do documento: a 4K (fator 2) o dente reduzido a 1080p é o dente de 1080p, e no
//! fator 1 o papel que os leitores amostram é o `brush.paper` de sempre, ao bit.

use crate::tool::PainterTool;
use crate::tool::fator_aparencia::papel_a_escala;

fn tela(lado: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.bind_document(1, vec![255; (lado * lado * 4) as usize], lado, lado);
    t.paint.brush.paper = super::super::substrate_relief::default_paper();
    t.paint.substrate_depth = 1.0;
    t
}

/// `(diferença média, correlação)` entre o dente da tela de 1080 e o da de 2160 reduzido 2×2, num canto 256².
fn compara(a: &PainterTool, b: &PainterTool) -> (f64, f64) {
    let (sa, sb) = (a.substrate().expect("papel"), b.substrate().expect("papel"));
    let n = 256i64;
    let (mut va, mut vb) = (Vec::new(), Vec::new());
    for y in 0..n {
        for x in 0..n {
            va.push(f64::from(sa.tooth_loads(x, y)));
            let q = [(0, 0), (1, 0), (0, 1), (1, 1)]
                .map(|(dx, dy)| f64::from(sb.tooth_loads(2 * x + dx, 2 * y + dy)));
            vb.push(q.iter().sum::<f64>() / 4.0);
        }
    }
    let m = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let (ma, mb) = (m(&va), m(&vb));
    let (mut cov, mut da, mut db, mut dif) = (0.0, 0.0, 0.0, 0.0);
    for (p, q) in va.iter().zip(&vb) {
        cov += (p - ma) * (q - mb);
        da += (p - ma).powi(2);
        db += (q - mb).powi(2);
        dif += (p - q).abs();
    }
    (dif / va.len() as f64, cov / (da * db).sqrt())
}

#[test]
fn mesmo_traco_1080p_e_4k_com_fator() {
    let a = tela(1080);
    let mut b = tela(2160);
    assert_eq!((a.fator_aparencia(), b.fator_aparencia()), (1.0, 2.0));
    // Fator 1: o papel que os leitores amostram é o do pincel, ao bit.
    assert!(a.papel_na_escala(a.paint.brush.paper) == a.paint.brush.paper);
    assert!(a.pincel_com_papel_na_escala().paper == a.paint.brush.paper);
    let (dif, corr) = compara(&a, &b);
    // Controlo: a 4K SEM o fator, o dente é metade do tamanho e a redução não o reencontra.
    b.fator_aparencia = 1.0;
    let (dif0, corr0) = compara(&a, &b);
    eprintln!(
        "[B34] 4K com fator: dif {dif:.4} corr {corr:.4} · sem fator: dif {dif0:.4} corr {corr0:.4}"
    );
    assert!(
        corr > 0.9,
        "o dente a 4K com fator 2 reduzido a 1080p tem de ser o de 1080p: corr {corr}"
    );
    assert!(
        corr0 < 0.5,
        "o controlo sem fator tem de reprovar: corr {corr0}"
    );
    assert!(dif < dif0 * 0.5, "dif {dif} × controlo {dif0}");
    assert_eq!(
        papel_a_escala(a.paint.brush.paper, 2.0).size,
        a.paint.brush.paper.size.map(|s| s / 2.0)
    );
}
