//! B20.11 (foto do dono, 12/11): as formigas são traço de ECRÃ — as quatro bordas a qualquer zoom, com
//! 1 px de ecrã, e marcham pelo passo. A régua lê o que a cena EMITE (`probe_strokes`).

use super::desenha_formigas;
use crate::tool::PainterTool;
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_vector::{Affine, Point, VectorScene};

const N: u32 = 400;
/// A seleção retangular da foto: x 60..310, y 50..260.
const CAIXA: [u32; 4] = [60, 50, 250, 210];

/// Imagem → ecrã a `zoom` px de ecrã por px de imagem, com um deslocamento não inteiro.
fn afim(zoom: f64) -> Affine {
    Affine::scale(zoom).then_translate((17.3, 9.6).into())
}

fn painter() -> PainterTool {
    let mut p = PainterTool::default();
    p.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    let [x, y, w, h] = CAIXA;
    p.set_rect_selection(x, y, w, h);
    assert!(p.selection_active());
    p
}

fn emite(p: &PainterTool, zoom: f64, passo: u32) -> (Vec<f32>, Vec<[f32; 2]>) {
    let mut vs = VectorScene::new();
    if let Some(c) = p.selection_contornos() {
        desenha_formigas(&mut vs, &c, afim(zoom), passo);
    }
    vs.probe_strokes()
}

#[test]
fn as_formigas_cobrem_as_quatro_bordas_a_qualquer_zoom_com_1_px_de_ecra() {
    let p = painter();
    for zoom in [0.37, 0.5, 1.0, 3.3] {
        let (larguras, pontos) = emite(&p, zoom, 3);
        assert!(!larguras.is_empty(), "zoom {zoom}: a cena não emite traço");
        assert!(
            larguras.iter().all(|&l| l == 1.0),
            "zoom {zoom}: largura em px de ecrã {larguras:?}"
        );
        let [x, y, w, h] = CAIXA.map(f64::from);
        let a = afim(zoom) * Point::new(x, y);
        let b = afim(zoom) * Point::new(x + w, y + h);
        let (x0, x1, y0, y1) = (a.x.min(b.x), a.x.max(b.x), a.y.min(b.y), a.y.max(b.y));
        // Cada lado: (coordenada fixa, eixo fixo é x?, início, fim do lado ao longo do outro eixo).
        for (nome, fixo, em_x, lo, hi) in [
            ("cima", y0, false, x0, x1),
            ("baixo", y1, false, x0, x1),
            ("esquerda", x0, true, y0, y1),
            ("direita", x1, true, y0, y1),
        ] {
            let ao_longo: Vec<f64> = pontos
                .iter()
                .map(|q| (f64::from(q[0]), f64::from(q[1])))
                .filter_map(|(px, py)| {
                    let (f, l) = if em_x { (px, py) } else { (py, px) };
                    ((f - fixo).abs() <= 1.0 && l >= lo - 1.0 && l <= hi + 1.0).then_some(l)
                })
                .collect();
            let comp = hi - lo;
            let (mn, mx) = ao_longo
                .iter()
                .fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), &l| {
                    (a.min(l), b.max(l))
                });
            assert!(
                ao_longo.len() as f64 >= comp / 8.0 && mx - mn >= 0.95 * comp,
                "zoom {zoom}: a borda de {nome} ({comp:.0} px) não tem traço: {} pontos",
                ao_longo.len()
            );
        }
    }
}

#[test]
fn as_formigas_marcham_pelo_passo_e_param_dentro_dele() {
    let p = painter();
    assert_eq!(
        emite(&p, 0.5, 5),
        emite(&p, 0.5, 5),
        "o mesmo passo, o mesmo traço"
    );
    assert_ne!(
        emite(&p, 0.5, 5).1,
        emite(&p, 0.5, 6).1,
        "o passo seguinte marcha"
    );
    let mut sem = PainterTool::default();
    sem.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    assert!(
        emite(&sem, 0.5, 5).0.is_empty(),
        "sem seleção, sem formigas"
    );
}
