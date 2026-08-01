//! **A cena da SIMETRIA de desenho** — `PH2D_BUILD_SMOKE=46` (plano 25 §9, a W6.3).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC (HR-18), como os `*_smoke` vizinhos.
//!
//! ⚠️ **Ela dá o MATERIAL e não arma modo nenhum** — a cicatriz que o `impasto_smoke` do Painter
//! prega: um smoke que arma o estado por baixo do pano pula justamente a costura que existe para
//! provar. A simetria nasce DESARMADA porque esse é o default do produto (um modo que se arma
//! sozinho muda a cena antes de o artista olhar), e é o artista que a liga na seção.
//!
//! O que ela monta:
//! - um **meio-perfil aberto** com as duas pontas no eixo `x = 0` — a forma para a qual a
//!   simetria existe, e a única em que a FUSÃO tem o que fazer (as metades soldam num vaso);
//! - uma **pétala** solta, longe do eixo, para a rosácea ter o que repetir;
//! - um **controle**: uma barra que ninguém vai seleccionar. Ela não pode mudar em passo nenhum
//!   do roteiro — se mudar, alguma coisa está a alcançar o que não devia.

use ph2d_vec_scene::{Rgba8, StrokeSpec, VecPath, VecVertex, VertexKind};

/// Largura do traço das referências, em unidades de mundo.
const STROKE_W: f64 = 0.04;

/// A geometria, numa tabela — `(pontos, fechado, cor)`.
///
/// ⚠️ `const` e partilhada com a sonda de baixo de propósito: os números que a mensagem anuncia
/// são MEDIDOS daqui, não escritos de memória. Uma cena que afirma um número que a geometria dela
/// não tem é a forma exata de um smoke que engana quem o corre.
type Piece = (&'static [[f64; 2]], bool, [u8; 3]);

const PIECES: &[Piece] = &[
    // O MEIO-PERFIL: pontas em x = 0 (o eixo), barriga para a direita. É o vaso pela metade.
    (
        &[
            [0.0, -1.1],
            [0.55, -0.75],
            [0.30, -0.15],
            [0.62, 0.45],
            [0.34, 0.85],
            [0.0, 1.1],
        ],
        false,
        [235, 200, 120],
    ),
    // A PÉTALA, longe do eixo: o material da rosácea.
    (
        &[[1.7, -0.2], [2.3, 0.0], [1.7, 0.2]],
        true,
        [150, 210, 180],
    ),
    // O CONTROLE: uma barra que o roteiro nunca selecciona.
    (
        &[[-2.4, -1.3], [-1.2, -1.3], [-1.2, -1.0], [-2.4, -1.0]],
        true,
        [130, 150, 200],
    ),
];

fn vertex(a: [f64; 2]) -> VecVertex {
    VecVertex {
        anchor: a,
        in_handle: a,
        out_handle: a,
        kind: VertexKind::Corner,
        corner_radius: 0.0,
    }
}

fn poly(pts: &[[f64; 2]], closed: bool, rgb: [u8; 3]) -> VecPath {
    VecPath {
        verts: pts.iter().map(|p| vertex(*p)).collect(),
        closed,
        stroke: Some(StrokeSpec::new(
            Rgba8::new(rgb[0], rgb[1], rgb[2], 255),
            STROKE_W,
        )),
        ..VecPath::default()
    }
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        4 => announce(app),
        _ => {}
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for (pts, closed, rgb) in PIECES {
        gfx.vec_scene.push_path(poly(pts, *closed, *rgb));
    }
}

/// A mensagem — com os números MEDIDOS da própria cena, nunca de memória.
fn announce(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    // A distância das duas PONTAS do meio-perfil ao eixo `x = 0`. É ela que decide se a fusão
    // dispara: a tolerância é 1% da extensão da forma, e o artista tem de ver isso acontecer.
    let ends = PIECES[0].0;
    let (a, b) = (ends[0][0].abs(), ends[ends.len() - 1][0].abs());
    eprintln!(
        "[symmetry] cena montada: {} formas — meio-perfil ABERTO com as pontas a {a:.3} e \
         {b:.3} do eixo x=0 (elas estão NO eixo, então a fusão dispara), uma pétala solta em \
         x~2 para a rosácea, e uma barra de CONTROLE que o roteiro nunca toca.",
        gfx.vec_scene.paths().len()
    );
    eprintln!(
        "[symmetry] a simetria nasce DESARMADA — é o default do produto, e é você que a liga."
    );
    eprintln!("[symmetry] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. Seleccione o MEIO-PERFIL. Na seção SYMMETRY, Enable -> On.");
    eprintln!("     Uma LINHA aparece no canvas, no centro da TELA — não da cena. Panhe a câmera");
    eprintln!("     antes de ligar e confira: ela nasce onde você está a olhar.");
    eprintln!("  2. O outro lado aparece. Com Fuse On as duas metades SOLDAM num vaso fechado;");
    eprintln!("     com Fuse Off vêem-se duas metades que apenas se tocam.");
    eprintln!("  3. MOVA a forma com o gizmo. ⚠️ A LINHA ACOMPANHA, mantendo a mesma distância");
    eprintln!("     relativa ao desenho — é a promessa central desta wave.");
    eprintln!("  4. DESMARQUE (Enable -> Off). As cópias somem e a sua metade fica INTACTA:");
    eprintln!("     elas nunca estiveram no documento. Ligue outra vez e elas voltam.");
    eprintln!("  5. Seleccione a PÉTALA e escolha Radial. Arraste Segments: a rosácea ganha e");
    eprintln!("     perde pétalas, e os RAIOS no canvas acompanham a contagem.");
    eprintln!("  6. APPLY — com a simetria ligada, o botão aparece no fim da seção. Ele");
    eprintln!("     consolida as cópias em formas de documento E desliga o modo. Um Ctrl+Z");
    eprintln!("     desfaz o gesto INTEIRO, num passo só.");
    eprintln!("  7. O CONTROLE (a barra azul, embaixo à esquerda) não pode ter mudado em nenhum");
    eprintln!("     dos passos acima.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_vec_scene::symmetry::{SymmetrySpec, symmetry_paths};

    /// **A cena afirma que a fusão dispara — este gate MEDE que ela dispara.**
    ///
    /// A tolerância é uma fracção da extensão da forma, então "as pontas estão no eixo" não é
    /// uma propriedade dos números que eu escrevi: é uma propriedade deles CONTRA o tamanho da
    /// peça. Uma cena que promete um fenômeno que a geometria dela não produz é a forma exata de
    /// um smoke que engana quem o corre — e o roteiro manda o artista comparar Fuse On com Fuse
    /// Off, o que não quer dizer nada se os dois desenharem a mesma coisa.
    #[test]
    fn the_half_profile_actually_fuses() {
        let src = poly(PIECES[0].0, PIECES[0].1, PIECES[0].2);
        let spec = SymmetrySpec::default();
        let fused = symmetry_paths(&src, &spec);
        assert_eq!(
            fused.len(),
            1,
            "com Fuse a peça vira UM caminho — as duas metades soldam"
        );
        assert!(
            fused[0].closed,
            "e ele é FECHADO: é isso que o torna um vaso"
        );

        let split = symmetry_paths(
            &src,
            &SymmetrySpec {
                fuse: false,
                ..spec
            },
        );
        assert_eq!(
            split.len(),
            2,
            "sem Fuse são duas metades que apenas se tocam — o contraste que o roteiro pede"
        );
    }

    /// **A pétala está FORA do eixo** — se estivesse sobre ele, a rosácea empilharia cópias no
    /// mesmo lugar e o passo 5 do roteiro não mostraria nada.
    #[test]
    fn the_petal_is_off_axis_so_the_rosette_has_something_to_show() {
        let min_x = PIECES[1]
            .0
            .iter()
            .map(|p| p[0])
            .fold(f64::INFINITY, f64::min);
        assert!(
            min_x > 1.0,
            "a pétala tem de estar longe do centro da rosácea, e está em x={min_x}"
        );
    }
}
