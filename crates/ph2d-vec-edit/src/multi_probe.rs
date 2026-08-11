//! **Sondas: o que a seleção de nós alcança hoje** — irmã de `selection.rs`, o sujeito.
//!
//! O plano 25 §6 nomeia *editar nós de VÁRIAS formas* como **ausência POR CONSTRUÇÃO**, com o
//! mecanismo escrito: o `selected_verts` é uma lista de índices PLANOS dentro de um `selected`
//! único, então dois índices de formas diferentes seriam indistinguíveis — e o Delete seguinte
//! apagaria nós de uma forma que o artista não estava a olhar.
//!
//! Estas sondas **medem** essa ausência em vez de a repetir de memória: quantos nós o gesto de
//! facto apanha, de quantas formas, e quantos o gesto seguinte de facto MOVE. Um número aqui é o
//! que separa *"o marquee vê um path só"* (a metade que a W3b já fechou) de *"a seleção não
//! consegue guardar dois donos"* (esta).
//!
//! Rode com:
//! ```text
//! cargo test -p ph2d-vec-edit --release -- --ignored measure_ --nocapture
//! ```

use crate::PenTool;
use ph2d_vec_scene::{VecScene, rectangle};

/// Duas formas SEPARADAS, lado a lado: A em `x ∈ [0,1]`, B em `x ∈ [2,3]`.
///
/// Separadas de propósito — uma caixa que apanhe as duas tem de atravessar o vão, que é
/// exactamente o gesto que o artista faz para pegar "estes quatro cantos e aqueles quatro".
fn two_squares() -> (VecScene, PenTool) {
    let mut scene = VecScene::new();
    scene.push_path(rectangle([0.0, 0.0], [1.0, 1.0]));
    scene.push_path(rectangle([2.0, 0.0], [3.0, 1.0]));
    (scene, PenTool::default())
}

/// De quantas FORMAS distintas a seleção de nós fala.
///
/// ⚠️ A 1ª versão desta função **contava o primário** (`selected().is_some()`), porque no mundo
/// que a sonda mediu a resposta só podia ser 0 ou 1 — os índices não carregavam dono. Com o dono
/// no par ela passou a mentir *para baixo*: os quatro números certos apareciam ao lado de um
/// `formas=1` que descrevia a representação antiga. Uma sonda escrita contra a ausência tem de ser
/// reconferida no dia em que a ausência fecha, senão é ela o gate verde sobre o fato errado.
fn paths_touched(pen: &PenTool, scene: &VecScene) -> usize {
    let _ = scene;
    let mut seen: Vec<_> = pen.selected_verts().iter().map(|(p, _)| *p).collect();
    seen.sort_unstable();
    seen.dedup();
    seen.len()
}

#[test]
#[ignore = "sonda: mede o alcance da seleção de nós"]
fn measure_what_a_box_over_two_shapes_picks() {
    let (scene, mut pen) = two_squares();

    // A caixa cobre as DUAS formas inteiras.
    pen.box_select(&scene, [-0.5, -0.5], [3.5, 1.5]);
    println!(
        "[caixa sobre AS DUAS]        nos={:2}  formas={}  primario={:?}",
        pen.selected_verts().len(),
        paths_touched(&pen, &scene),
        pen.selected()
    );

    // E agora o gesto ADITIVO, que é como um artista soma a segunda forma: pega A, depois
    // Shift+arrasta sobre B.
    let (scene, mut pen) = two_squares();
    pen.box_select_with(&scene, [-0.5, -0.5], [1.5, 1.5], false);
    let after_a = pen.selected_verts().len();
    pen.box_select_with(&scene, [1.5, -0.5], [3.5, 1.5], true);
    println!(
        "[A, depois +B (aditivo)]     nos={:2} (A sozinha tinha {})  formas={}",
        pen.selected_verts().len(),
        after_a,
        paths_touched(&pen, &scene),
    );
    println!(
        "  ^ se o total nao for {}, a soma TROCOU de alvo em vez de somar",
        after_a * 2
    );
}

#[test]
#[ignore = "sonda: mede o alcance da seleção de nós"]
fn measure_what_a_shift_click_across_shapes_picks() {
    let (scene, mut pen) = two_squares();
    let r = 0.2;

    // Um canto de A, depois um canto de B — o gesto de duas mãos mais simples que existe.
    let a = pen.toggle_vert_at(&scene, [0.0, 0.0], r);
    let n_after_a = pen.selected_verts().len();
    let b = pen.toggle_vert_at(&scene, [2.0, 0.0], r);
    println!(
        "[shift-clique A depois B]    acertou_a={a} acertou_b={b}  nos={} (depois de A eram {})",
        pen.selected_verts().len(),
        n_after_a,
    );
    println!("  ^ os dois cliques acertaram uma ancora; se o total nao for 2, o 2o SUBSTITUIU");
}

#[test]
#[ignore = "sonda: mede o alcance da seleção de nós"]
fn measure_how_many_nodes_a_nudge_actually_moves() {
    let (mut scene, mut pen) = two_squares();
    pen.box_select(&scene, [-0.5, -0.5], [3.5, 1.5]);

    let before: Vec<[f64; 2]> = scene
        .paths()
        .iter()
        .flat_map(|p| p.verts_all().map(|v| v.anchor))
        .collect();
    pen.nudge(&mut scene, 1.0, 0.0);
    let after: Vec<[f64; 2]> = scene
        .paths()
        .iter()
        .flat_map(|p| p.verts_all().map(|v| v.anchor))
        .collect();
    let moved = before
        .iter()
        .zip(&after)
        .filter(|(b, a)| (b[0] - a[0]).abs() > 1e-9 || (b[1] - a[1]).abs() > 1e-9)
        .count();
    println!(
        "[nudge apos caixa nas duas]  moveram={} de {} ancoras da cena",
        moved,
        before.len()
    );
    println!("  ^ 8 = as duas formas andaram juntas; 4 = so uma; 0 = nada");
}
