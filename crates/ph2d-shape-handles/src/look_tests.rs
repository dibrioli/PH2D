use super::*;
use ph2d_physics_ecs::ShapeLook;

/// A edição pela porta [`aplica`] e a fila aplicada ao mundo (o passo que a shell dá no quadro).
fn aplica_ja(sim: &mut SimWorld, e: Entity, edit: PhysicsFieldEdit) {
    let mut reg = ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    let queue = EditorCommandQueue::default();
    assert!(aplica(sim, e, edit, &queue, &reg), "é uma edição do Look");
    ph2d_ecs::scene::apply_editor_commands(sim.world_mut(), &queue, &reg).expect("commit");
}

/// ⭐ A alça mostra e puxa pelo CENTRO real; com o pivô na borda esquerda, o `offset` cru que volta põe o centro
/// exatamente onde a alça o deixou, e o pivô continua na borda da forma nova.
#[test]
fn a_alca_numa_forma_com_pivo_deixa_o_centro_onde_puxou() {
    let mut sim = SimWorld::new();
    let cur = Collider {
        shape: ColliderShape::Cuboid {
            half_x: 0.5,
            half_y: 0.5,
        },
        ..Collider::default()
    };
    let e = sim
        .world_mut()
        .spawn((
            cur,
            ShapeLook {
                pivot: [-1.0, 0.0],
                ..ShapeLook::default()
            },
        ))
        .id();
    let novo = forma_arrastada(sim.world(), e, &cur, [1.5, 0.5], [1.5, 0.0]);
    let r = ph2d_physics_ecs::forma_resolvida(sim.world(), e, &novo);
    assert_eq!(r.offset, [1.5, 0.0], "o centro que a alça deixou");
    assert_eq!(
        r.offset[0] - 1.5,
        0.0,
        "a borda esquerda, o pivô, na origem"
    );
}

/// Um corpo parado com a forma FILHA `1 × 1` em `(3, 2)`, girada 90° e esticada 2× em X no corpo.
fn corpo_com_forma() -> (SimWorld, Entity) {
    let mut sim = SimWorld::new();
    let p = sim
        .world_mut()
        .spawn(ph2d_ecs::Transform::from_translation(ph2d_core::Vec2::new(
            3.0, 2.0,
        )))
        .id();
    ph2d_physics_ecs::torna_corpo(
        sim.world_mut(),
        p,
        ph2d_physics_ecs::BodyKind::Static,
        [0.5, 0.5],
        false,
    );
    let f = ph2d_physics_ecs::main_shape(sim.world(), p).expect("a forma");
    let mut t = *sim.world().get::<ph2d_ecs::Transform>(f).expect("t");
    t.rotation = std::f32::consts::FRAC_PI_2;
    t.scale.x = 2.0;
    sim.world_mut().entity_mut(f).insert(t);
    sim.world_mut().get_mut::<Collider>(f).expect("c").offset = [0.25, 0.0];
    (sim, f)
}

/// Os quatro cantos da forma em MUNDO, pela porta única (o que o solver e o overlay leem).
fn cantos(sim: &SimWorld, f: Entity) -> Vec<[f32; 2]> {
    let w = sim.world();
    let col = ph2d_physics_ecs::forma_resolvida(w, f, w.get::<Collider>(f).expect("c"));
    let t = ph2d_ecs::world_transform(w, f).expect("mundo");
    let h = ph2d_physics_ecs::shape_look::meia_extensao(col.shape);
    let (s, c) = libm::sincosf(t.rotation);
    [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]]
        .iter()
        .map(|k: &[f32; 2]| {
            let q = [
                (col.offset[0] + k[0] * h[0]) * t.scale.x,
                (col.offset[1] + k[1] * h[1]) * t.scale.y,
            ];
            [
                t.translation.x + c * q[0] - s * q[1],
                t.translation.y + s * q[0] + c * q[1],
            ]
        })
        .collect()
}

/// ⭐ O ponto de baixo à esquerda (e depois o Custom): a forma não sai do lugar no mundo, a origem do objeto vai
/// para o canto (a fração `[-1, -1]`) e o Offset de antes dobra-se na posição.
#[test]
fn o_pivo_novo_deixa_a_forma_no_lugar_e_leva_a_origem_ao_ponto() {
    let (mut sim, f) = corpo_com_forma();
    let antes = cantos(&sim, f);
    aplica_ja(&mut sim, f, PhysicsFieldEdit::LookPivotPoint([0, 2]));
    let depois = cantos(&sim, f);
    for (a, b) in antes.iter().zip(&depois) {
        assert!(
            (a[0] - b[0]).abs() < 1e-5 && (a[1] - b[1]).abs() < 1e-5,
            "{antes:?} → {depois:?}"
        );
    }
    let w = sim.world();
    assert_eq!(w.get::<ShapeLook>(f).expect("o look").pivot, [-1.0, -1.0]);
    assert_eq!(
        w.get::<Collider>(f).expect("c").offset,
        [0.0, 0.0],
        "o Offset dobrou-se"
    );
    let o = ph2d_ecs::world_transform(w, f).expect("mundo").translation;
    assert!(
        depois
            .iter()
            .any(|q| (q[0] - o.x).abs() < 1e-5 && (q[1] - o.y).abs() < 1e-5),
        "a origem está num canto: {o:?} {depois:?}"
    );
    aplica_ja(&mut sim, f, PhysicsFieldEdit::LookPivotX(0.25));
    let custom = cantos(&sim, f);
    for (a, b) in antes.iter().zip(&custom) {
        assert!(
            (a[0] - b[0]).abs() < 1e-5 && (a[1] - b[1]).abs() < 1e-5,
            "Custom: {custom:?}"
        );
    }
    assert_eq!(
        sim.world().get::<ShapeLook>(f).expect("o look").pivot,
        [0.5, -1.0]
    );
}

#[test]
fn a_opacidade_e_a_cor_nao_mexem_na_forma() {
    let (mut sim, f) = corpo_com_forma();
    let col = *sim.world().get::<Collider>(f).expect("c");
    let t = *sim.world().get::<ph2d_ecs::Transform>(f).expect("t");
    aplica_ja(&mut sim, f, PhysicsFieldEdit::LookOpacity(0.3));
    aplica_ja(
        &mut sim,
        f,
        PhysicsFieldEdit::LookTint([1.0, 0.0, 0.0, 1.0]),
    );
    let w = sim.world();
    let l = w.get::<ShapeLook>(f).expect("o look");
    assert_eq!((l.opacity, l.tint), (0.3, [1.0, 0.0, 0.0, 1.0]));
    assert_eq!(*w.get::<Collider>(f).expect("c"), col);
    assert_eq!(*w.get::<ph2d_ecs::Transform>(f).expect("t"), t);
}

/// ⭐ **A ferramenta Pivot do canvas sobre a forma**: o mesmo cálculo do gesto (`move_pivot_transform`, o que a shell
/// faz a cada `CursorMoved`) e o `arrasta_pivo` — a forma fica nos quatro cantos, a origem vai para o ponto largado,
/// e o pivô guardado é o que o campo X/Y do cartão lê.
#[test]
fn arrastar_o_pivo_no_canvas_deixa_a_forma_e_leva_a_origem_ao_cursor() {
    let (mut sim, f) = corpo_com_forma();
    let antes = cantos(&sim, f);
    let w = sim.world();
    let t = *w.get::<ph2d_ecs::Transform>(f).expect("t");
    let pw = ph2d_ecs::parent_world_transform(w, f);
    let snap = |t: ph2d_ecs::Transform| ph2d_editor_core::TransformSnapshot {
        translation: [t.translation.x, t.translation.y],
        rotation: t.rotation,
        scale: [t.scale.x, t.scale.y],
    };
    let (anchor, _) = anchor_half(w, f).expect("é uma forma");
    let mundo = ph2d_ecs::world_transform(w, f).expect("mundo");
    let (s, c) = libm::sincosf(mundo.rotation);
    let (ax, ay) = (anchor[0] * mundo.scale.x, anchor[1] * mundo.scale.y);
    let centro = [
        mundo.translation.x + c * ax - s * ay,
        mundo.translation.y + s * ax + c * ay,
    ];
    let alvo = antes[2];
    let (origem, nova) = ph2d_editor_core::move_pivot_transform(snap(t), centro, alvo, snap(pw));
    assert!(arrasta_pivo(sim.world_mut(), f, origem, nova));
    let depois = cantos(&sim, f);
    for (a, b) in antes.iter().zip(&depois) {
        assert!(
            (a[0] - b[0]).abs() < 1e-4 && (a[1] - b[1]).abs() < 1e-4,
            "{antes:?} → {depois:?}"
        );
    }
    let o = ph2d_ecs::world_transform(sim.world(), f)
        .expect("mundo")
        .translation;
    assert!(
        (o.x - alvo[0]).abs() < 1e-4 && (o.y - alvo[1]).abs() < 1e-4,
        "a origem no cursor: {o:?} {alvo:?}"
    );
    let p = sim.world().get::<ShapeLook>(f).expect("o look").pivot;
    assert!(
        (p[0] - 1.0).abs() < 1e-4 && (p[1] - 1.0).abs() < 1e-4,
        "o canto de cima à direita: {p:?}"
    );
    let col = *sim.world().get::<Collider>(f).expect("c");
    let fa = fatos(sim.world(), f, &col, true);
    assert_eq!(fa.ponto, Some([2, 0]), "o cartão acende o ponto do canto");
}

/// O clique da ferramenta agarra DENTRO da forma (girada e esticada), não fora; e só numa forma de colisão.
#[test]
fn o_clique_agarra_dentro_da_forma() {
    let (sim, f) = corpo_com_forma();
    let k = cantos(&sim, f);
    let meio = [(k[0][0] + k[2][0]) * 0.5, (k[0][1] + k[2][1]) * 0.5];
    assert!(contem(sim.world(), f, meio), "o centro");
    // Rente à borda do lado longe da origem (o Offset põe a forma de um lado): só cai dentro com a rotação certa.
    let borda = [(k[1][0] + k[2][0]) * 0.5, (k[1][1] + k[2][1]) * 0.5];
    let rente = [
        borda[0] + (meio[0] - borda[0]) * 0.1,
        borda[1] + (meio[1] - borda[1]) * 0.1,
    ];
    assert!(contem(sim.world(), f, rente), "rente à borda, por dentro");
    let fora = [k[2][0] + (k[2][0] - meio[0]), k[2][1] + (k[2][1] - meio[1])];
    assert!(!contem(sim.world(), f, fora), "além do canto");
    let corpo = sim
        .world()
        .get::<ph2d_ecs::ChildOf>(f)
        .expect("filha")
        .parent();
    assert!(!contem(sim.world(), corpo, meio), "o corpo não é a forma");
}

/// O piso da Opacity mora em dois sítios (o cartão não vê a física): têm de ser o mesmo número.
#[test]
fn o_piso_da_opacidade_e_um_so() {
    assert_eq!(
        ShapeLookFacts::OPACIDADE_MINIMA,
        ph2d_physics_ecs::shape_look::OPACIDADE_MINIMA
    );
}
