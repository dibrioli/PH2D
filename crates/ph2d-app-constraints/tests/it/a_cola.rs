//! ⭐ **A cola no cartão, no palco e na cena 8** (`docs/BlenderConstraints/05_W5_diferenciais.md` §2.5 e §5): o
//! menu arma, o clique no palco cola, o cartão mostra a cola nos DOIS objectos, «Unglue» desfaz, o gizmo leva o
//! pino nos dois.

use bevy_ecs::entity::Entity;
use ph2d_app_constraints::gizmo::{Parte, Peca, arrastar, pecas};
use ph2d_app_constraints::inspector::{Escolha, apply_all, armado, escolher_alvo, info};
use ph2d_app_constraints::smoke;
use ph2d_constraints::Pose2;
use ph2d_constraints_cola::{Cola, Tipo};
use ph2d_constraints_ecs::{Colas, Constraints, colar};
use ph2d_ecs::{Locked, Name, SimWorld, StableId, Transform, world_transform};
use ph2d_editor_core::action_bus::{ComponentEdit, EditorAction};
use ph2d_editor_core::constraint_edits::{
    ConstraintFieldEdit as E, InspectorConstraintsInfo, TIPO_COLA_RIGIDA,
};
use ph2d_editor_core::zones::Rect;
use ph2d_host::WindowSize;
use ph2d_panel_inspector::{
    InspectorPanel, InspectorState, ids, set_current_inspector_constraints,
};
use ph2d_preview_drive::PreviewDrive;
use ph2d_render::Camera2d;
use ph2d_ui_testkit::MockPanelHost;

const VIEWPORT: Rect = Rect {
    x: 0.0,
    y: 0.0,
    w: 320.0,
    h: 2400.0,
};

fn em(x: f32, y: f32, rot: f32) -> Transform {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t.translation.y = y;
    t.rotation = rot;
    t
}

fn pose(sim: &SimWorld, e: Entity) -> Pose2 {
    let t = world_transform(sim.world(), e).expect("pose mundial");
    Pose2 {
        x: f64::from(t.translation.x),
        y: f64::from(t.translation.y),
        rot: f64::from(t.rotation),
        sx: f64::from(t.scale.x),
        sy: f64::from(t.scale.y),
    }
}

/// Clica no centro de `id` com o cartão `i` no painel; devolve o que chegou ao barramento, como a shell o
/// entrega ao `apply_all`.
fn clica(
    host: &mut MockPanelHost,
    state: &mut InspectorState,
    i: InspectorConstraintsInfo,
    id: ph2d_a11y::NodeId,
) -> Vec<(u64, E)> {
    set_current_inspector_constraints(Some(i));
    let rects = host.paint::<InspectorPanel>(state, VIEWPORT);
    let r = rects
        .iter()
        .find(|(n, _)| *n == id)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("o cartão não pintou {id:?}"));
    for ev in host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5) {
        let _ = host.apply_panel_event::<InspectorPanel>(state, ev);
    }
    set_current_inspector_constraints(None);
    host.drained_actions()
        .into_iter()
        .filter_map(|a| match a {
            EditorAction::InspectorComponentEdit {
                entity_bits,
                edit: ComponentEdit::Constraint(e),
            } => Some((entity_bits, e)),
            _ => None,
        })
        .collect()
}

/// ⭐⭐⭐ Pelo caminho REAL: «Add» → «Glue · Rigid» no cartão do A → o clique no B no palco → o cartão do A e
/// o do B mostram a cola, cada um com o OUTRO por parceiro → «Unglue» no cartão do B → some dos dois.
///
/// **Mutações que devem sangrar:** `linhas` só com as colas que a entidade GUARDA · `parceiro` = o próprio ·
/// `Adicionar(18)` a criar item na pilha · `Descolar` endereçado à entidade escolhida em vez do dono.
#[test]
fn the_glue_card_shows_on_both_objects() {
    let mut sim = SimWorld::new();
    let a = sim
        .world_mut()
        .spawn((
            StableId(1),
            Name("Braço".into()),
            em(0.0, 0.0, 0.0),
            Constraints::default(),
        ))
        .id();
    let b = sim
        .world_mut()
        .spawn((StableId(2), Name("Mão".into()), em(3.0, 1.0, 0.4)))
        .id();
    let (ab, bb) = (a.to_bits(), b.to_bits());
    assert!(info(&sim, bb, 1).is_none(), "o B não tem pilha nem cola");

    let mut host = MockPanelHost::with_panel::<InspectorPanel>();
    let mut state = InspectorState::default();
    let i = info(&sim, ab, 1).unwrap();
    assert!(clica(&mut host, &mut state, i.clone(), ids::INSP_CONSTRAINT_ADD).is_empty());
    let pedido = clica(
        &mut host,
        &mut state,
        i,
        ids::INSP_CONSTRAINT_TIPO[usize::from(TIPO_COLA_RIGIDA)],
    );
    assert_eq!(pedido, vec![(ab, E::Adicionar(TIPO_COLA_RIGIDA))]);
    assert!(!apply_all(&mut sim, &pedido), "armar não muda o documento");
    assert!(armado(&sim));
    let i = info(&sim, ab, 1).unwrap();
    assert_eq!(i.cola_armada, Some(false));
    assert!(i.rows.is_empty(), "a cola não é item da pilha");

    assert_eq!(
        escolher_alvo(&mut sim, &[bb, ab], [3.0, 1.0]),
        Escolha::Gravado
    );
    assert!(!armado(&sim));
    for (quem, parceiro) in [(ab, "Mão"), (bb, "Braço")] {
        let i = info(&sim, quem, 1).expect("o cartão aparece nos DOIS objectos");
        assert_eq!(i.colas.len(), 1);
        assert_eq!(i.colas[0].parceiro, parceiro);
        assert_eq!(i.colas[0].dono_bits, ab);
        assert!(!i.colas[0].pino);
        assert_eq!(i.cola_armada, None);
    }

    let i = info(&sim, bb, 1).unwrap();
    let id = i.colas[0].id;
    let pedido = clica(&mut host, &mut state, i, ids::INSP_COLA_DESCOLAR[0]);
    assert_eq!(pedido, vec![(bb, E::Descolar(ab, id))]);
    assert!(apply_all(&mut sim, &pedido));
    assert!(info(&sim, ab, 1).unwrap().colas.is_empty());
    assert!(
        info(&sim, bb, 1).is_none(),
        "descolado, o B volta a não ter cartão"
    );
}

/// ⭐⭐ O pino aparece no gizmo dos DOIS objectos; arrastá-lo leva-o nos dois locais e nenhum objecto se mexe.
#[test]
fn dragging_the_pin_moves_it_on_both_objects() {
    let mut sim = SimWorld::new();
    let a = sim.world_mut().spawn((StableId(1), em(0.0, 0.0, 0.3))).id();
    let b = sim
        .world_mut()
        .spawn((StableId(2), em(2.0, 1.0, -0.7)))
        .id();
    let (pa, pb) = (pose(&sim, a), pose(&sim, b));
    let cola = Cola::pino(1, 1, 2, pa, pb, [1.0, 0.5]).unwrap();
    sim.world_mut().entity_mut(a).insert(Colas {
        items: vec![cola],
        next_id: 2,
    });
    let (cam, win) = (Camera2d::new([0.0, 0.0], 10.0), WindowSize::new(1000, 1000));
    let pega_de = |sim: &SimWorld, sel: Entity| {
        pecas(sim.world(), Some(sel.to_bits()), &cam, win)
            .into_iter()
            .find_map(|p| match p {
                Peca::Pino((g, _)) => Some(g),
                _ => None,
            })
    };
    let pega = pega_de(&sim, a).expect("o gizmo do A não mostra o pino");
    assert_eq!(
        pega_de(&sim, b),
        Some(pega),
        "o gizmo do B mostra o MESMO pino"
    );
    assert_eq!(
        (pega.parte, pega.dono, pega.item),
        (Parte::Pino, a.to_bits(), 1)
    );

    let alvo = [-1.5, 2.25];
    assert!(arrastar(&mut sim, pega, alvo));
    let c = sim.world().get::<Colas>(a).unwrap().items[0];
    let Tipo::Pino { em_a, em_b } = c.tipo else {
        panic!("o pino deixou de ser pino")
    };
    for (p, em) in [(pose(&sim, a), em_a), (pose(&sim, b), em_b)] {
        let m = p.ponto(em);
        assert!(
            (m[0] - alvo[0]).abs() < 1e-9 && (m[1] - alvo[1]).abs() < 1e-9,
            "o pino ficou em {m:?}, o dedo largou em {alvo:?}"
        );
    }
    assert_eq!(
        (pose(&sim, a), pose(&sim, b)),
        (pa, pb),
        "nenhum objecto se mexe"
    );
}

fn chamada(sim: &mut SimWorld, nome: &str) -> Entity {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w)
        .find(|(_, n)| n.0 == nome)
        .map(|(e, _)| e)
        .unwrap_or_else(|| panic!("a cena não tem «{nome}»"))
}

/// ⭐⭐ Cena 8: depois de um `colar`, os quatro pinos fecham (os dois lados no mesmo ponto do mundo) e o chão
/// tem cadeado.
#[test]
fn the_articulated_arm_scene_closes_its_four_pins_on_a_locked_ground() {
    let mut sim = SimWorld::new();
    assert!(smoke::encenar(sim.world_mut(), 8));
    let mut drive = PreviewDrive::default();
    colar(&mut sim, &mut drive);
    drive.settle();
    let chao = chamada(&mut sim, "Chão");
    assert!(sim.world().get::<Locked>(chao).is_some());
    let colas = sim
        .world()
        .get::<Colas>(chao)
        .expect("as colas moram no chão")
        .clone();
    let corpo = |sim: &mut SimWorld, s: u64| {
        let w = sim.world_mut();
        let mut q = w.query::<(Entity, &StableId)>();
        q.iter(w).find(|(_, i)| i.0 == s).map(|(e, _)| e).unwrap()
    };
    let mut pinos = 0;
    for c in colas.items {
        let Tipo::Pino { em_a, em_b } = c.tipo else {
            continue;
        };
        let (ea, eb) = (corpo(&mut sim, c.a), corpo(&mut sim, c.b));
        let (ma, mb) = (pose(&sim, ea).ponto(em_a), pose(&sim, eb).ponto(em_b));
        let erro = (ma[0] - mb[0]).hypot(ma[1] - mb[1]);
        assert!(erro <= 1e-4, "o pino {} abre {erro:e}", c.id);
        pinos += 1;
    }
    assert_eq!(pinos, 4);
}
