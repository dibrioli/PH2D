//! O Copy Point prende o vértice pelo ID (W4.2): inserir um ponto antes dele não o troca de ponto, e
//! apagá-lo deixa a regra sem alvo em vez de a mudar para o vizinho.

use ph2d_constraints::{Kind, PontoDeForma};
use ph2d_constraints_ecs::{Constraints, Estado, Estados};
use ph2d_ecs::{SimWorld, StableId, Transform, VecPathRef, world_transform};
use ph2d_preview_drive::PreviewDrive;
use ph2d_vec_scene::{Contour, VecPath, VecScene, VecVertex, VertId};

fn em(sim: &SimWorld, e: bevy_ecs::entity::Entity) -> [f32; 2] {
    let t = world_transform(sim.world(), e).unwrap();
    [t.translation.x, t.translation.y]
}

#[test]
fn copy_point_follows_its_vertex_after_an_insert_before_it() {
    let mut sim = SimWorld::new();
    let mut cena = VecScene::new();
    let caminho = cena.push_path(VecPath {
        verts: [[0.0, 0.0], [4.0, 0.0], [4.0, 3.0]]
            .map(VecVertex::corner)
            .to_vec(),
        subpaths: vec![Contour::new_closed(
            [[1.0, 1.0], [2.0, 1.0], [2.0, 2.0]]
                .map(VecVertex::corner)
                .to_vec(),
        )],
        ..VecPath::default()
    });
    let p = cena.path(caminho).unwrap();
    let canto = p.verts[1].id;
    let do_buraco = p.subpaths[0].verts[2].id;
    sim.world_mut()
        .spawn((StableId(1), Transform::IDENTITY, VecPathRef(caminho)));
    let mut c = Constraints::default();
    let item = c.push(
        Kind::PontoDeForma(PontoDeForma { vert: canto }),
        Some(StableId(1)),
    );
    let mut c2 = Constraints::default();
    c2.push(
        Kind::PontoDeForma(PontoDeForma { vert: do_buraco }),
        Some(StableId(1)),
    );
    let dono = sim
        .world_mut()
        .spawn((StableId(2), Transform::IDENTITY, c))
        .id();
    let dono2 = sim
        .world_mut()
        .spawn((StableId(3), Transform::IDENTITY, c2))
        .id();
    let mut drive = PreviewDrive::default();
    ph2d_app_constraints::quadro(&mut sim, &cena, &mut drive, None);
    assert_eq!(em(&sim, dono), [4.0, 0.0]);
    assert_eq!(
        em(&sim, dono2),
        [2.0, 2.0],
        "o ponto de um buraco também prende"
    );

    // Um ponto novo ANTES do alvo (o que a caneta faz) — o gancho do undo dá-lhe id.
    cena.path_mut(caminho)
        .unwrap()
        .verts
        .insert(0, VecVertex::corner([-5.0, 0.0]));
    cena.normaliza_ids();
    ph2d_app_constraints::quadro(&mut sim, &cena, &mut drive, None);
    assert_eq!(em(&sim, dono), [4.0, 0.0], "o mesmo ponto, pelo id");
    assert_eq!(
        cena.path(caminho).unwrap().verts[1].anchor,
        [0.0, 0.0],
        "⛔ CONTROLO: pelo índice seria outro ponto"
    );

    // Apagar o alvo: a regra fica sem alvo, e NÃO salta para o vizinho.
    let p = cena.path_mut(caminho).unwrap();
    let i = p.verts.iter().position(|v| v.id == canto).unwrap();
    p.verts.remove(i);
    ph2d_app_constraints::quadro(&mut sim, &cena, &mut drive, None);
    assert_eq!(
        sim.world().resource::<Estados>().of(dono, item),
        Some(Estado::SemAlvo)
    );
    assert!(cena.path(caminho).unwrap().vert_by_id(VertId(2)).is_none());
}
