//! B20.11: as formigas traçadas em espaço de ECRÃ a partir do contorno guardado
//! ([`super::selection_contorno`]) — 1 px de ecrã a qualquer zoom (o desenho na imagem perdia linhas com
//! zoom < 1), cantos no centro do pixel, o tracejado marcha pelo passo. A shell só passa o afim.

use super::selection_contorno::{ContornosDaSelecao, Polilinha};
use ph2d_vector::{Affine, BezPath, Brush, Color, Point, Stroke, VectorScene};

/// As formigas: claro contínuo + escuro tracejado, e o ciano do Offset tracejado.
pub const COR_FORMIGA_CLARA: [f32; 4] = [1.0, 1.0, 1.0, 1.0]; // LITERAL-COLOR-OK: marching ants
pub const COR_FORMIGA_ESCURA: [f32; 4] = [0.0, 0.0, 0.0, 1.0]; // LITERAL-COLOR-OK: marching ants
pub const COR_DO_OFFSET: [f32; 4] = [0.0, 0.863, 1.0, 1.0]; // LITERAL-COLOR-OK: offset contour
/// Meio período do tracejado, em px de ECRÃ; a fase anda 1 px por passo.
pub const TRACO_DAS_FORMIGAS: f64 = 4.0;

fn caminho(lacos: &[Polilinha], imagem_para_ecra: Affine) -> BezPath {
    let mut path = BezPath::new();
    for laco in lacos {
        for (k, p) in laco.iter().enumerate() {
            let s = imagem_para_ecra * Point::new(f64::from(p[0]), f64::from(p[1]));
            let s = Point::new(s.x.floor() + 0.5, s.y.floor() + 0.5);
            if k == 0 {
                path.move_to(s);
            } else {
                path.line_to(s);
            }
        }
        path.close_path();
    }
    path
}

/// Traça `c` (px de imagem) em `vs` pelo afim imagem→ecrã, com 1 px de ecrã, no `passo` das formigas.
pub fn desenha_formigas(
    vs: &mut VectorScene,
    c: &ContornosDaSelecao,
    imagem_para_ecra: Affine,
    passo: u32,
) {
    let fase = f64::from(passo) % (2.0 * TRACO_DAS_FORMIGAS);
    let tracejado = Stroke::new(1.0).with_dashes(fase, [TRACO_DAS_FORMIGAS; 2]);
    let mut traca = |path: &BezPath, s: &Stroke, cor| {
        let b = Brush::Solid(Color::new(cor));
        vs.inner_mut().stroke(s, Affine::IDENTITY, &b, None, path);
    };
    if !c.mascara.is_empty() {
        let path = caminho(&c.mascara, imagem_para_ecra);
        traca(&path, &Stroke::new(1.0), COR_FORMIGA_CLARA);
        traca(&path, &tracejado, COR_FORMIGA_ESCURA);
    }
    if !c.niveis.is_empty() {
        traca(
            &caminho(&c.niveis, imagem_para_ecra),
            &tracejado,
            COR_DO_OFFSET,
        );
    }
}

#[cfg(test)]
#[path = "selection_formigas_ecra_tests.rs"]
mod tests;
