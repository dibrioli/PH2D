//! ⭐⭐⭐ **A pose que a coleta lê é a do EDITOR** (smoke do dono, 2026-10-09: *«mover a lâmpada não
//! atualiza a posição da luz»*). No editor o `GlobalTransform` do mundo NÃO é mantido — a
//! propagação escreve-o só no `PresentWorld` (`sim_extract`) —, logo ele fica no valor com que o
//! objeto nasceu, e um objeto criado pelo menu Add nem o tem. A porta canónica da pose no mundo é
//! `ph2d_ecs::world_transform` (a local composta com a cadeia de pais), a do clique e do gizmo.
//!
//! As fixturas nascem como o produto as cria: SÓ `Transform` (o menu Add), ou com um
//! `GlobalTransform` VELHO (uma cena antiga, depois de arrastada).

use bevy_ecs::world::World;
use ph2d_ecs::{ChildOf, GlobalTransform, Transform};
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_ecs::{BloqueiaLuz, Emissor, Luz};
use ph2d_render::Sprite;

fn em(x: f32, y: f32) -> Transform {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t.translation.y = y;
    t
}

fn centro(e: &Emissor) -> [f32; 2] {
    match e {
        Emissor::Disco { centro, .. } | Emissor::Ponto { centro, .. } => *centro,
        Emissor::Poligono { vertices, .. } => {
            let n = vertices.len() as f32;
            let s = vertices
                .iter()
                .fold([0.0, 0.0], |a, v| [a[0] + v[0], a[1] + v[1]]);
            [s[0] / n, s[1] / n]
        }
    }
}

/// A lâmpada do menu Add (só `Transform`) EMITE, onde está; arrastá-la move a luz.
#[test]
fn a_lampada_do_menu_add_emite_e_segue_o_arrasto() {
    let mut w = World::new();
    let e = w.spawn((Luz::default(), em(1.0, 2.0))).id();
    let c = cena_de_luz(&w);
    assert_eq!(
        c.emissores.len(),
        1,
        "a lâmpada nascida pelo menu Add não emite"
    );
    assert_eq!(centro(&c.emissores[0]), [1.0, 2.0]);
    *w.get_mut::<Transform>(e).unwrap() = em(-3.0, 0.5);
    assert_eq!(
        centro(&cena_de_luz(&w).emissores[0]),
        [-3.0, 0.5],
        "o arrasto não moveu a luz"
    );
}

/// Um `GlobalTransform` velho (o da cena antes do arrasto) NÃO manda: manda a pose do editor.
#[test]
fn um_global_velho_nao_manda_na_luz_nem_na_sombra() {
    let mut w = World::new();
    let velho = GlobalTransform::from_transform(em(9.0, 9.0));
    w.spawn((Luz::default(), em(1.0, 0.0), velho));
    w.spawn((
        Sprite::atlas(0, [1.0, 1.0], [1.0; 4]),
        BloqueiaLuz::default(),
        em(-2.0, 0.0),
        velho,
    ));
    let c = cena_de_luz(&w);
    assert_eq!(centro(&c.emissores[0]), [1.0, 0.0]);
    let o = &c.oclusores[0].vertices;
    let cx = o.iter().map(|v| v[0]).sum::<f32>() / o.len() as f32;
    assert!(
        (cx - -2.0).abs() < 1e-5,
        "a sombra ficou onde o objeto ESTAVA ({cx})"
    );
}

/// Uma lâmpada FILHA de um objeto segue o pai (a cadeia inteira, não só a pose local).
#[test]
fn a_lampada_filha_segue_o_pai() {
    let mut w = World::new();
    let pai = w.spawn(em(5.0, 1.0)).id();
    w.spawn((Luz::default(), em(1.0, 0.0), ChildOf(pai)));
    assert_eq!(centro(&cena_de_luz(&w).emissores[0]), [6.0, 1.0]);
}
