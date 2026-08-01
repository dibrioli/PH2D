//! **A cena da SIMETRIA de desenho** — `PH2D_BUILD_SMOKE=46` (plano 25 §9, a W6.3).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC (HR-18), como os `*_smoke` vizinhos.
//!
//! ⚠️ **Ela dá o MATERIAL e não arma modo nenhum** — a cicatriz que o `impasto_smoke` do Painter
//! prega: um smoke que arma o estado por baixo do pano pula justamente a costura que existe para
//! provar. A simetria nasce DESARMADA porque esse é o default do produto, e é o artista que a liga.
//!
//! ⚠️ **E TUDO o que ela monta é CONTROLE.** A wave inteira é sobre *"a simetria funciona apenas
//! para formas que serão desenhadas com a tool ligada"* (Enio), então a cena não pode entregar uma
//! forma pronta para espelhar — ela entrega o que **não pode** espelhar, e o artista desenha o
//! resto. Uma cena que já trouxesse a forma certa provaria o modelo errado.
//!
//! O que ela monta:
//! - um **meio-perfil aberto** com as duas pontas em `x = 0`, em ÂMBAR — o **molde**: é a forma que
//!   o roteiro pede para o artista desenhar por cima, e a única em que a FUSÃO tem o que fazer (as
//!   metades soldam num vaso). Ele **existia antes do botão**, então nunca pode espelhar;
//! - uma **barra** azul, o segundo controle, longe de tudo.

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
    // O MOLDE: pontas em x = 0 (onde a linha vai nascer), barriga para a direita. É o vaso pela
    // metade — e, por ter existido ANTES do botão, é também o controle da regra central.
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
    // O SEGUNDO CONTROLE: uma barra que o roteiro nunca toca.
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
    // A distância das duas PONTAS do molde ao eixo `x = 0`. É ela que decide se a fusão dispara: a
    // tolerância é 1% da extensão da forma, e o artista tem de ver isso acontecer.
    let ends = PIECES[0].0;
    let (a, b) = (ends[0][0].abs(), ends[ends.len() - 1][0].abs());
    eprintln!(
        "[symmetry] cena montada: {} formas, e as DUAS sao CONTROLE — elas existiam antes do \
         botao, entao nao podem espelhar NUNCA. O molde ambar e' um meio-perfil ABERTO com as \
         pontas a {a:.3} e {b:.3} de x=0 (elas estao NO eixo, entao a fusao dispara).",
        gfx.vec_scene.paths().len()
    );
    eprintln!(
        "[symmetry] a simetria nasce DESARMADA — é o default do produto, e é você que a liga."
    );
    eprintln!("[symmetry] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. SEM selecionar nada, na seção SYMMETRY: Enable -> On.");
    eprintln!("     ⚠️ A LINHA APARECE NA HORA, no centro da TELA. Panhe a câmera antes de ligar");
    eprintln!("     e confira: ela nasce onde você está a olhar, não onde a cena está.");
    eprintln!("  2. Com a Pen, DESENHE um meio-perfil por cima do molde âmbar, com as duas pontas");
    eprintln!("     na linha. O outro lado aparece enquanto você desenha; com Fuse On as metades");
    eprintln!("     SOLDAM num vaso fechado, com Fuse Off elas apenas se tocam.");
    eprintln!("  3. ⚠️ O MOLDE ÂMBAR NÃO ESPELHOU. Ele existia antes do botão — selecione-o e");
    eprintln!("     confirme que continua sozinho. É a regra central desta wave.");
    eprintln!("  4. DESENHE outra forma, em qualquer lugar. ⚠️ A LINHA NÃO SE MEXEU, e o desenho");
    eprintln!("     novo espelha nela — quantos desenhos você quiser, a mesma linha.");
    eprintln!("  5. MOVA o primeiro desenho com o gizmo. ⚠️ A linha DELE acompanha, mantendo a");
    eprintln!("     distância relativa; a de sessão fica onde estava. São dois fatos diferentes.");
    eprintln!("  6. Enable -> Off. As cópias somem e os seus desenhos ficam INTACTOS: elas nunca");
    eprintln!("     estiveram no documento. Ligue outra vez e elas voltam.");
    eprintln!(
        "  7. Escolha Radial e arraste Segments: a rosácea ganha e perde pétalas, e os RAIOS"
    );
    eprintln!("     no canvas acompanham a contagem.");
    eprintln!("  8. APPLY — com a simetria ligada, o botão aparece no fim da seção. Ele consolida");
    eprintln!("     as cópias em formas de documento E desliga o modo. Um Ctrl+Z desfaz tudo.");
    eprintln!(
        "  9. Os DOIS controles (o molde âmbar e a barra azul) não podem ter mudado em passo"
    );
    eprintln!("     nenhum acima.");
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
    fn the_mould_is_a_shape_that_actually_fuses() {
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

    /// **A cena não entrega NENHUMA forma pronta para espelhar.**
    ///
    /// ⚠️ Este gate existe porque a versão anterior desta cena entregava exactamente isso, e com
    /// ela o smoke encenava o modelo que o Enio recusou (*"não deve fazer simetria de formas que
    /// já existem previamente"*). Toda peça que a cena monta é CONTROLE; a forma que espelha é a
    /// que o artista desenha, e é por isso que o roteiro começa por ligar o botão com a selecção
    /// vazia.
    #[test]
    fn every_piece_the_scene_builds_is_a_control() {
        let mut sim = ph2d_ecs::SimWorld::default();
        let mut map = crate::vec_entities::VecEntityMap::new();
        let mut scene = ph2d_vec_scene::VecScene::new();
        let ids: Vec<_> = PIECES
            .iter()
            .map(|(pts, closed, rgb)| scene.push_path(poly(pts, *closed, *rgb)))
            .collect();
        for (n, id) in ids.iter().enumerate() {
            let e = sim
                .world_mut()
                .spawn((
                    ph2d_ecs::Transform::IDENTITY,
                    ph2d_ecs::Name::new(format!("P{n}")),
                    ph2d_ecs::VecPathRef(*id),
                ))
                .id();
            map.insert(*id, e.to_bits());
        }
        let xf = crate::vec_transform::build(&sim, &map);

        // O modo LIGADO, o eixo semeado — e ninguém em gesto, que é o estado do passo 1.
        let mut live = crate::symmetry_live::SymmetryLive::default();
        let armed = live.adopt(
            &mut sim,
            &map,
            &scene,
            &xf,
            ph2d_vec_scene::symmetry::SymmetryStyle {
                on: true,
                ..Default::default()
            },
            [0.0, 0.0],
            &[],
        );
        assert_eq!(
            armed, 0,
            "ligar o botão com esta cena não pode armar peça nenhuma — todas são controle"
        );
    }
}
