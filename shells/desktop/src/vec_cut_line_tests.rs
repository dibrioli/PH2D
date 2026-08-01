//! Gates da LINHA DE CORTE (plano 25 §7) — as quatro leis que o smoke do Enio (2026-07-31)
//! acrescentou, mais a que a wave já tinha.
//!
//! Estes correm no shell porque a lâmina só EXISTE aqui: o marcador vive no ECS, o mapa
//! path↔entidade é do shell, e é a combinação dos três que responde *"há lâmina, e ela está
//! selecionada?"*. Nenhum teste de unidade de crate alcança essa pergunta.

use super::{apply_cut, cut_line, upkeep};
use crate::vec_entities::VecEntityMap;
use ph2d_ecs::SimWorld;
use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecPath, VecPathId, VecScene, VecVertex};

/// Um losango fechado de arestas retas, centrado em `c`.
fn diamond(c: [f64; 2]) -> VecPath {
    VecPath {
        verts: [[-2.0, 0.0], [0.0, 2.0], [2.0, 0.0], [0.0, -2.0]]
            .into_iter()
            .map(|p| VecVertex::corner([p[0] + c[0], p[1] + c[1]]))
            .collect(),
        closed: true,
        fill: Some(Paint::solid(Rgba8::new(90, 90, 90, 255))),
        ..VecPath::default()
    }
}

fn line(a: [f64; 2], b: [f64; 2]) -> VecPath {
    VecPath {
        verts: vec![VecVertex::corner(a), VecVertex::corner(b)],
        ..VecPath::default()
    }
}

/// Um mundo com `shapes` e uma lâmina já adotada. Devolve `(sim, scene, map, id da lâmina)`.
fn world_with_blade(
    shapes: Vec<VecPath>,
    blade: VecPath,
) -> (SimWorld, VecScene, VecEntityMap, VecPathId) {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::default();
    let mut map = VecEntityMap::new();
    for s in shapes {
        scene.push_path(s);
    }
    let id = scene.push_path(blade);
    // Duas voltas: o `sync` dá entidade, e só então o `upkeep` tem onde pendurar o marcador —
    // exactamente a dança de um frame do produto.
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let mut pending = Some(id);
    upkeep(&mut sim, &mut scene, &map, &mut pending);
    assert_eq!(cut_line(&sim, &map), Some(id), "a lâmina não foi adotada");
    (sim, scene, map, id)
}

/// **A lâmina nunca carrega estilo** — nem o `fill` que a caneta estampa ao FECHAR um caminho,
/// nem o traço que um recolorir de seleção lhe daria.
///
/// Enio, 2026-07-31: *"o traço do CUT não pode receber um Fill"*. A regra é re-afirmada a cada
/// frame porque há mais de uma porta que devolve estilo a um caminho, e nenhuma delas sabe o que
/// é uma lâmina.
#[test]
fn the_blade_never_wears_a_fill_no_matter_who_stamps_it() {
    let (mut sim, mut scene, map, id) = world_with_blade(vec![], line([-4.0, 0.0], [4.0, 0.0]));
    // Alguém (a caneta ao fechar, o recolorir da seleção) estampa estilo na lâmina.
    let p = scene.path_mut(id).expect("a lâmina existe");
    p.fill = Some(Paint::solid(Rgba8::new(255, 0, 0, 255)));
    p.stroke = Some(StrokeSpec::new(Rgba8::new(0, 255, 0, 255), 1.0));

    let mut pending = None;
    upkeep(&mut sim, &mut scene, &map, &mut pending);

    let p = scene.paths().iter().find(|p| p.id == id).expect("existe");
    assert!(p.fill.is_none(), "a lâmina recebeu um Fill");
    assert!(p.stroke.is_none(), "a lâmina recebeu um traço");
}

/// **O corte alcança TUDO o que a lâmina atravessa** — não só o que está selecionado.
///
/// Enio, 2026-07-31: *"o corte deve acontecer em qualquer forma sobreposta e não apenas na forma
/// selecionada"*. A seleção que este gesto exige é a da própria lâmina; usá-la também para
/// escolher alvos daria dois significados ao mesmo estado.
#[test]
fn the_cut_reaches_every_shape_the_blade_crosses() {
    let (sim, mut scene, map, id) = world_with_blade(
        vec![
            diamond([-6.0, 0.0]),
            diamond([0.0, 0.0]),
            diamond([6.0, 0.0]),
        ],
        line([-12.0, 0.0], [12.0, 0.0]),
    );
    // Só a LÂMINA está selecionada — nenhum dos três losangos.
    let cut = apply_cut(&sim, &mut scene, &map, &[id], true);
    assert_eq!(cut, 3, "a lâmina atravessa os três, e cortou {cut}");
}

/// **Sem o pill Cut escolhido, não se corta.** A lâmina fica na cena depois de usada, e um
/// clique no botão fora do modo cortaria a cena outra vez com uma lâmina esquecida lá.
#[test]
fn a_cut_needs_the_tool_in_hand() {
    let (sim, mut scene, map, id) =
        world_with_blade(vec![diamond([0.0, 0.0])], line([-6.0, 0.0], [6.0, 0.0]));
    assert_eq!(
        apply_cut(&sim, &mut scene, &map, &[id], false),
        0,
        "cortou com a ferramenta fora da mão"
    );
    assert_eq!(
        apply_cut(&sim, &mut scene, &map, &[id], true),
        1,
        "com a ferramenta na mão tem de cortar (senão este gate seria verde por vácuo)"
    );
}

/// **Sem a lâmina SELECIONADA, não se corta** — a outra metade da condição.
#[test]
fn a_cut_needs_the_blade_selected() {
    let (sim, mut scene, map, id) =
        world_with_blade(vec![diamond([0.0, 0.0])], line([-6.0, 0.0], [6.0, 0.0]));
    assert_eq!(
        apply_cut(&sim, &mut scene, &map, &[], true),
        0,
        "cortou sem a lâmina selecionada"
    );
    assert_eq!(
        apply_cut(&sim, &mut scene, &map, &[id], true),
        1,
        "com a lâmina selecionada tem de cortar"
    );
}

/// **A lâmina não corta a si mesma, e SOBREVIVE ao corte** — ela é um objeto autorado, não um
/// gesto consumido (é o que a distingue do *Divide Objects Below* do Illustrator).
#[test]
fn the_blade_survives_the_cut_and_never_cuts_itself() {
    let (sim, mut scene, map, id) =
        world_with_blade(vec![diamond([0.0, 0.0])], line([-6.0, 0.0], [6.0, 0.0]));
    let before = scene.paths().len();
    assert_eq!(apply_cut(&sim, &mut scene, &map, &[id], true), 1);
    assert!(
        scene.paths().iter().any(|p| p.id == id),
        "a lâmina foi consumida pelo corte"
    );
    // O losango virou duas peças; a lâmina continua lá.
    assert_eq!(scene.paths().len(), before + 1, "contagem de caminhos");
}
