//! (W4) **A curva do Detail é pré-visualização também no TRAÇO**: os anéis são derivados do pedido,
//! e a fotografia do undo tem de ver o par AUTORADO (pedido + anéis + caixa), senão cada quadro a
//! tocar com input vira um passo. *Mutação: `substituir` sem repor o contorno ⇒ vermelho.*

use super::*;
use crate::physics_tests::registry;
use ph2d_ecs::{ChildOf, Entity, SpritePixels, Transform};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderOutline, ColliderShape, RigidBody};
use ph2d_render::Sprite;

const PPM: f32 = 8.0;

/// Um pente 32×32 (dentes de 4 px que o Detail de 2 px guarda e o de 6 px fecha) num corpo Static;
/// a forma FILHA em «Outline», já traçada e anotada.
fn pente() -> (ComponentRegistry, SimWorld, Entity, ph2d_asset::AssetDb) {
    use ph2d_editor_core::PhysicsFieldEdit as P;
    let reg = registry();
    let mut sim = SimWorld::new();
    let db = ph2d_asset::AssetDb::new();
    let px: Vec<u8> = (0..32u32)
        .flat_map(|y| (0..32u32).map(move |x| (x, y)))
        .flat_map(|(x, y)| {
            let on = (4..28).contains(&x) && (4..28).contains(&y) && (y >= 8 || (x / 3) % 2 == 0);
            [255, 255, 255, if on { 255 } else { 0 }]
        })
        .collect();
    let img = db.insert_image_rgba8(32, 32, px);
    let body = sim
        .world_mut()
        .spawn((
            RigidBody {
                kind: BodyKind::Static,
            },
            Transform::default(),
            Sprite::atlas(0, [4.0, 4.0], [1.0; 4]),
            SpritePixels(img),
        ))
        .id();
    let c = Collider {
        shape: ColliderShape::Outline {
            half_x: 2.0,
            half_y: 2.0,
        },
        ..Collider::default()
    };
    sim.world_mut()
        .spawn((c, Transform::default(), ChildOf(body)));
    let q = EditorCommandQueue::new();
    let none = std::collections::BTreeMap::new();
    let pick = [(body.to_bits(), P::Shape(3))];
    ph2d_physics_outline::retrace_edited(sim.world(), &pick, &db, &none, PPM, &q, &reg);
    apply_editor_commands(sim.world_mut(), &q, &reg).expect("aplica");
    segue(&mut sim, &db, &reg);
    (reg, sim, body, db)
}

/// A passagem do quadro (`inspector_commits`): retraça o que a curva pediu, pela fila.
fn segue(sim: &mut SimWorld, db: &ph2d_asset::AssetDb, reg: &ComponentRegistry) {
    let q = EditorCommandQueue::new();
    let none = std::collections::BTreeMap::new();
    ph2d_physics_outline::retrace_followed(sim.world_mut(), db, &none, PPM, &q, reg);
    apply_editor_commands(sim.world_mut(), &q, reg).expect("aplica");
}

fn par(sim: &SimWorld, forma: Entity) -> (Collider, ColliderOutline) {
    let w = sim.world();
    (
        *w.get::<Collider>(forma).expect("c"),
        w.get::<ColliderOutline>(forma).cloned().expect("o"),
    )
}

/// ⭐⭐ Chaves no Detail (2 px em t0, 6 px em t1), a tocar através delas pelo caminho do quadro
/// (`aplicar` → `retrace_followed` → fila → `settle` → fotografia): a fotografia é a de ANTES em
/// todo quadro (zero passos), e o vivo segue a chave. Parada, a curva vira documento (um passo).
#[test]
fn a_curva_do_detail_nao_e_passo_de_undo_a_tocar() {
    let (reg, mut sim, body, db) = pente();
    let dono = ph2d_physics_ecs::dono_do_campo(sim.world(), body.to_bits(), f::CONTORNO_DETALHE);
    let forma = Entity::from_bits(dono);
    let base = par(&sim, forma);
    assert!(!base.1.rings.is_empty(), "o pente traçou");
    let mut conducao = ConducaoFisica::default();
    conducao.a_tocar(true);
    for detalhe in [3.0, 4.5, 6.0, 6.0] {
        sim.world_mut()
            .get_resource_or_insert_with(CanaisEscritos::default)
            .escrever(dono, k(f::CONTORNO_DETALHE), detalhe);
        aplicar(&mut sim, &reg, &mut conducao);
        segue(&mut sim, &db, &reg);
        conducao.settle();
        let vivo = conducao.substituir(&mut sim, &reg);
        assert_eq!(
            par(&sim, forma),
            base,
            "a tocar, Detail {detalhe}: a fotografia é a de antes"
        );
        conducao.repor(&mut sim, &reg, vivo);
        let o = par(&sim, forma).1;
        assert_eq!(o.detail_q4, (detalhe * 16.0) as u16, "o vivo segue a chave");
        assert_ne!(o.rings, base.1.rings, "e retraçou");
    }
    let vivo_final = par(&sim, forma);
    conducao.a_tocar(false);
    conducao.settle();
    conducao.settle();
    assert!(conducao.is_empty());
    let vivo = conducao.substituir(&mut sim, &reg);
    assert_eq!(par(&sim, forma), vivo_final, "parada, a curva é documento");
    conducao.repor(&mut sim, &reg, vivo);
}
