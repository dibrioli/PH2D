//! **Arch-gate: a linha que o gizmo desenha é a COZIDA; as alças ficam na quina AFIADA.**
//!
//! ## O pedido (Enio, 2026-08-07)
//!
//! *"Line tem em seu gizmo a possibilidade de criar Chamfer e Fillet. Contudo, agora que não temos mais
//! o preview da tinta, vamos precisar que o preview do chamfer e do fillet aconteça na própria linha."*
//!
//! ## Por que é um gate de TEXTO
//!
//! O comportamento tem gates de unidade na `ph2d-tool-painter`
//! (`stroke_outline_tests::the_line_gizmo_draws_the_cooked_corner_not_the_sharp_one` e o irmão do
//! não-SALTO), mas **quem escolhe qual das duas listas alimenta o traço é a shell**, e ela só existe com
//! janela. Um gate de unidade fica VERDE com o `LineOverlay` perfeito e a shell desenhando a outra lista
//! — foi exatamente esse o defeito.
//!
//! ## As duas metades, e por que são duas
//!
//! - o **traço** vem de `overlay.outline` — a geometria cozida (`line_corner::cooked_path`), o que a
//!   figura É;
//! - as **alças** vêm de `overlay.points` — a fonte autorada, onde o artista pega. Num fillet grande a
//!   quina afiada fica FORA do desenho, e é o [ADR-0121] que diz que isso é o certo (`inkscape:original-d`
//!   + `d`): mover a alça para cima do arco tiraria do artista o ponto que ele edita.
//!
//! [ADR-0121]: ../../../docs/architecture/decisions/0121-vector-live-corners-authored-source-cooked-geometry.md

const LINE: &str = include_str!("../src/render_loop/painter_bridge_line_overlay.rs");

/// O traço dos segmentos é alimentado pelo contorno COZIDO.
///
/// **Mutação que deve sangrar:** `let pts = &overlay.points;` de volta — o Fillet/Chamfer some do
/// desenho e o artista arrasta uma alça que não muda nada na tela.
#[test]
fn the_segments_are_stroked_from_the_cooked_outline() {
    assert!(
        LINE.contains("let pts = &overlay.outline;"),
        "o gizmo da Line nao desenha mais o contorno COZIDO — com o gesto rascunhado, o Fillet/Chamfer \
         fica invisivel enquanto a alca que o cria e arrastada"
    );
}

/// As alças de quina continuam na fonte AUTORADA.
///
/// **Mutação que deve sangrar:** iterar `overlay.outline` no laço das alças — os pontos de agarrar
/// migrariam para o arco e a quina deixaria de ser editável.
#[test]
fn the_corner_handles_stay_on_the_authored_points() {
    assert!(
        LINE.contains("for (i, &p) in overlay.points.iter().enumerate()"),
        "as alcas de quina sairam da fonte autorada — a quina afiada e o que se arrasta"
    );
}

/// Controle positivo: o arquivo lido é mesmo o overlay da Line, e ele de fato TRAÇA a lista.
#[test]
fn the_scanned_file_is_the_line_overlay_and_it_strokes() {
    assert!(LINE.contains("fn draw_line_overlay("));
    assert!(
        LINE.contains("stroke_box(scene, &sp, &pal)")
            && LINE.contains("stroke_open(scene, &sp, &pal)"),
        "o overlay nao traca mais a polilinha — as duas asserções acima ficariam verdes por vacuo"
    );
}
