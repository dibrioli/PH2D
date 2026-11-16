//! ⭐ **«Make the parts»** (plano §W4, dono 12/11: «objetos prontos sim, mágicos não»): o botão do gabarito
//! por fazer cria as peças de fábrica reais — nomeadas, com a forma de colisão filha e a aparência, no meio
//! da vista, montadas, TODAS selecionadas e anunciadas —, e o retrato de antes (o Ctrl+Z) não tem nada delas.
//! Na ponte, o campo «Planks» diz quantas tábuas, e o poste da direita anda com a última.

use ph2d_app_joint_editor::Hit;
use ph2d_app_joint_editor::doc::{Field, Menu};
use ph2d_ecs::scene::{ComponentRegistry, WorldSnapshot, snapshot_to_world, world_to_snapshot};
use ph2d_ecs::{Name, SimWorld, TransformPropagationState, WorklistBuf};
use ph2d_editor_core::GridView;
use ph2d_editor_core::zones::Rect;
use ph2d_physics_ecs::{JointAssembly, JointKind, PhysicsJoint, RigidBody};
use ph2d_render::Sprite;

use super::Mesa;

const VIEW: [f32; 2] = [30.0, 6.0];

fn registry() -> ComponentRegistry {
    let mut reg = ComponentRegistry::new();
    ph2d_ecs::scene::register_ecs_components(&mut reg);
    ph2d_physics_ecs::register_physics_components(&mut reg);
    reg
}

fn snap(sim: &mut SimWorld) -> WorldSnapshot {
    let mut s = WorldSnapshot::new();
    let mut prop = TransformPropagationState::new(sim.world_mut());
    let mut wl = WorklistBuf::default();
    world_to_snapshot(sim.world_mut(), &mut prop, &mut wl, &registry(), &mut s).expect("retrato");
    s
}

/// (montagens, juntas, corpos) de um retrato.
fn census(s: &WorldSnapshot) -> (usize, usize, usize) {
    let mut w = SimWorld::new();
    snapshot_to_world(w.world_mut(), s, &registry()).expect("abre");
    let w = w.world_mut();
    let a = w.query::<&JointAssembly>().iter(w).count();
    let j = w.query::<&PhysicsJoint>().iter(w).count();
    let b = w.query::<&RigidBody>().iter(w).count();
    (a, j, b)
}

impl Mesa {
    /// A aba no gabarito `name` da família `kind`, por fazer, com a câmara da cena em [`VIEW`].
    fn draft(kind: JointKind, name: &str) -> Self {
        let mut m = Self::aberta();
        m.hero.set_grid_view(Some(GridView {
            camera_center: VIEW,
            camera_height_world: 10.0,
            window_w: 1400.0,
            window_h: 900.0,
            canvas: Rect::new(0.0, 0.0, 0.0, 0.0),
        }));
        m.click(&Hit::OpenMenu(Menu::Kind));
        m.click(&Hit::Kind(kind));
        m.click(&Hit::OpenMenu(Menu::Template));
        let t = ph2d_joint_templates::by_name(kind, name).expect("gabarito");
        m.click(&Hit::Template(t.id.0));
        m
    }

    /// O objeto `name`: (bits, centro no mundo).
    fn object(&mut self, name: &str) -> (u64, [f32; 2]) {
        let w = self.sim.world_mut();
        let e = w
            .query::<(ph2d_ecs::Entity, &Name)>()
            .iter(w)
            .find(|(_, n)| n.as_str() == name)
            .map(|(e, _)| e)
            .unwrap_or_else(|| panic!("«{name}» na cena"));
        let t = ph2d_ecs::world_transform(w, e).expect("pose").translation;
        (e.to_bits(), [t.x, t.y])
    }
}

/// O Car: três peças reais com os nomes do gabarito, cada uma com forma filha e aparência, no meio da
/// vista, a montagem «Car 1» com as duas rodas, as três selecionadas, o aviso, e o retrato de antes limpo.
/// **Mutações que sangram:** sem `assemble` · as peças na origem (sem `at`) · só a 1.ª selecionada ·
/// sem o aviso.
#[test]
fn make_the_parts_cria_o_carro_real_no_meio_da_vista() {
    let mut m = Mesa::draft(JointKind::Wheel, "Car");
    let before = snap(&mut m.sim);
    m.click(&Hit::Make);
    let names = ["Car Body", "Car Rear Wheel", "Car Front Wheel"];
    let mut bits = Vec::new();
    let mut sum = [0.0, 0.0];
    for n in names {
        let (b, at) = m.object(n);
        bits.push(b);
        sum = [sum[0] + at[0], sum[1] + at[1]];
        let w = m.sim.world_mut();
        let e = ph2d_ecs::Entity::from_bits(b);
        assert!(
            w.get::<RigidBody>(e).is_some() && w.get::<Sprite>(e).is_some(),
            "{n}: corpo e aparência"
        );
        assert!(
            ph2d_physics_ecs::forma_para_editar(w, e).is_some(),
            "{n}: a forma filha (ADR-0199)"
        );
    }
    let (cx, cy) = (sum[0] / 3.0, sum[1] / 3.0);
    assert!(
        (cx - VIEW[0]).abs() < 0.5 && (cy - VIEW[1]).abs() < 0.5,
        "no meio da vista: ({cx}, {cy})"
    );
    let after = snap(&mut m.sim);
    let (a0, j0, b0) = census(&before);
    assert_eq!(
        census(&after),
        (a0 + 1, j0 + 2, b0 + 3),
        "uma montagem, duas juntas, três corpos"
    );
    assert_eq!(
        census(&before),
        (a0, j0, b0),
        "o retrato de antes (o Ctrl+Z) não tem nada do gesto"
    );
    let mut sel: Vec<u64> = m.hero.gizmo.selection.into_iter().collect();
    sel.extend(m.hero.gizmo.extra_selection.iter().copied());
    sel.sort_unstable();
    bits.sort_unstable();
    assert_eq!(sel, bits, "as três selecionadas");
    let toast: Vec<&str> = m.toasts.iter().map(|t| t.message.as_str()).collect();
    assert_eq!(
        toast,
        ["Made 3 objects: Car Body, Car Rear Wheel, Car Front Wheel."]
    );
    let open = m.tab().doc.assembly().map(|a| a.name.clone());
    assert_eq!(
        open.as_deref(),
        Some("Car 1"),
        "a aba abre a montagem feita"
    );
    // Outra vez: nomes novos, nunca dois «Car Body».
    m.click(&Hit::New);
    m.click(&Hit::Make);
    m.object("Car Body (1)");
}

/// A ponte com 12 tábuas: «Plank 1» … «Plank 12», a corrente inteira (11 entre tábuas + 2 nos postes), o
/// poste da direita a 4 passos mais longe que na de fábrica (8 tábuas), e uma segunda ponte bem nomeada.
/// **Mutações que sangram:** o campo sem chegar ao `Make` · `shift` sem andar com a última tábua · o
/// sufixo peça a peça · a montagem «Bridge 1 (1)» · o aviso com as catorze.
#[test]
fn a_ponte_faz_as_tabuas_que_o_campo_pede() {
    let mut m = Mesa::draft(JointKind::Pin, "Bridge");
    m.click(&Hit::Field(Field::Count));
    m.escreve("12");
    m.tecla(
        ph2d_editor_core::screens::hero::board_keys::BoardKey::Enter,
        None,
    );
    m.click(&Hit::Make);
    for k in 1..=12 {
        m.object(&format!("Plank {k}"));
    }
    let (_, l) = m.object("Bridge Left Post");
    let (_, r) = m.object("Bridge Right Post");
    let t = ph2d_joint_templates::by_name(JointKind::Pin, "Bridge").expect("ponte");
    let factory = t.roles[2].at[0] - t.roles[0].at[0];
    let step = t.roles[1].many.expect("fila").step[0];
    assert!(
        ((r[0] - l[0]) - (factory + 4.0 * step)).abs() < 1e-3,
        "o poste anda: {}",
        r[0] - l[0]
    );
    let sid = m.tab().doc.assembly().map(|a| a.sid).expect("montagem");
    let joints = ph2d_joint_assembly::joint_assembly::joints_of(m.sim.world_mut(), sid);
    assert_eq!(joints.len(), 13, "11 entre tábuas + os dois postes");
    let toast: Vec<&str> = m.toasts.iter().map(|t| t.message.as_str()).collect();
    assert_eq!(
        toast,
        ["Made 14 objects: Bridge Left Post, Plank 1 \u{2026} Plank 12, Bridge Right Post."],
        "a fila diz-se pela primeira e a última"
    );
    // A segunda ponte: «Bridge 2», e TODAS as peças com o mesmo sufixo (nunca umas sim e outras não).
    // Com 14: as tábuas 13 e 14 estão livres, e mesmo assim levam o sufixo da máquina.
    m.click(&Hit::New);
    m.click(&Hit::Field(Field::Count));
    m.escreve("14");
    m.tecla(
        ph2d_editor_core::screens::hero::board_keys::BoardKey::Enter,
        None,
    );
    m.click(&Hit::Make);
    assert_eq!(
        m.tab().doc.assembly().map(|a| a.name.clone()).as_deref(),
        Some("Bridge 2")
    );
    for k in 1..=14 {
        m.object(&format!("Plank {k} (1)"));
    }
    m.object("Bridge Left Post (1)");
}

/// ⭐⭐ **Os números da junta no painel de criação** (dono 15/11): a ponte por fazer mostra o cartão Breaks
/// de fábrica (desligado); com «Applies to: Plank», ligar Breakable e escrever 500 N vale para as 11 juntas
/// entre as 12 tábuas, e o «Make the parts» fá-las assim — os dois postes ficam de fábrica.
/// **Mutações que sangram:** o `Make` sem as edições · a edição em todas as regras (sem o alcance) ·
/// as edições escritas antes do `assemble`.
#[test]
fn o_painel_de_criacao_muda_as_juntas_das_tabuas_antes_de_fazer() {
    let mut m = Mesa::draft(JointKind::Pin, "Bridge");
    let t = ph2d_joint_templates::by_name(JointKind::Pin, "Bridge").expect("ponte");
    let fab = t.joints[1].joint(0.0);
    assert!(!fab.break_enabled, "a ponte de fábrica não parte");
    assert_eq!(
        m.tab().doc.value(Field::BreakForce),
        Some(fab.break_force),
        "o número de fábrica no cartão"
    );
    let plank = ph2d_app_joint_editor::doc::scope_options(t)
        .iter()
        .position(|o| o.label == "Plank")
        .expect("«Plank» no Applies to");
    m.click(&Hit::Scope(plank));
    m.click(&Hit::Field(Field::Count));
    m.escreve("12");
    m.tecla(
        ph2d_editor_core::screens::hero::board_keys::BoardKey::Enter,
        None,
    );
    m.click(&Hit::Flag(ph2d_app_joint_editor::doc::Flag::Breakable));
    m.click(&Hit::Field(Field::BreakForce));
    m.escreve("500");
    m.tecla(
        ph2d_editor_core::screens::hero::board_keys::BoardKey::Enter,
        None,
    );
    assert_eq!(
        m.tab().doc.value(Field::BreakForce),
        Some(500.0),
        "o cartão mostra o escrito"
    );
    m.click(&Hit::Make);
    let sid = m.tab().doc.assembly().map(|a| a.sid).expect("montagem");
    let joints = ph2d_joint_assembly::joint_assembly::joints_of(m.sim.world_mut(), sid);
    let (planks, posts): (Vec<_>, Vec<_>) = joints.iter().partition(|(_, j)| j.role == 1);
    assert_eq!(planks.len(), 11, "11 juntas entre 12 tábuas");
    for (_, j) in &planks {
        assert!(
            j.break_enabled && (j.break_force - 500.0).abs() < 1e-3,
            "{j:?}"
        );
    }
    assert_eq!(posts.len(), 2);
    for (_, j) in &posts {
        assert!(!j.break_enabled, "os postes ficam de fábrica");
    }
}

/// O carro por fazer mostra a suspensão de fábrica e o Drive; escolher «None» faz as rodas sem motor, e
/// «Applies to: Rear wheel» muda só a de trás.
/// **Mutações que sangram:** o Drive do rascunho sem chegar ao `Make` · a edição em todas as regras.
#[test]
fn o_carro_por_fazer_mostra_a_suspensao_de_fabrica_e_o_drive() {
    let mut m = Mesa::draft(JointKind::Wheel, "Car");
    let t = ph2d_joint_templates::by_name(JointKind::Wheel, "Car").expect("carro");
    let fab = t.joints[0].joint(0.0);
    assert_eq!(m.tab().doc.value(Field::Stiffness), Some(fab.stiffness));
    m.click(&Hit::Drive(ph2d_app_joint_editor::doc::Drive::None));
    // «Applies to: Rear wheel»: o número só nessa roda.
    let rear = ph2d_app_joint_editor::doc::scope_options(t)
        .iter()
        .position(|o| o.label == "Rear wheel")
        .expect("Rear wheel");
    m.click(&Hit::Scope(rear));
    m.click(&Hit::Field(Field::Stiffness));
    m.escreve("77");
    m.tecla(
        ph2d_editor_core::screens::hero::board_keys::BoardKey::Enter,
        None,
    );
    m.click(&Hit::Make);
    let sid = m.tab().doc.assembly().map(|a| a.sid).expect("montagem");
    let joints = ph2d_joint_assembly::joint_assembly::joints_of(m.sim.world_mut(), sid);
    assert_eq!(joints.len(), 2);
    assert!(joints.iter().all(|(_, j)| !j.motor_enabled), "Drive None");
    let k: Vec<f32> = joints.iter().map(|(_, j)| j.stiffness).collect();
    assert_eq!(k, [77.0, t.joints[1].joint(0.0).stiffness], "só a de trás");
}

impl Mesa {
    /// Escreve `n` no campo «Planks»/«Links» da aba.
    fn planks(&mut self, n: &str) {
        self.click(&Hit::Field(Field::Count));
        self.escreve(n);
        self.tecla(
            ph2d_editor_core::screens::hero::board_keys::BoardKey::Enter,
            None,
        );
    }

    fn bridge_joints(&mut self) -> Vec<PhysicsJoint> {
        let sid = self.tab().doc.assembly().map(|a| a.sid).expect("montagem");
        ph2d_joint_assembly::joint_assembly::joints_of(self.sim.world_mut(), sid)
            .into_iter()
            .map(|(_, j)| j)
            .collect()
    }

    fn exists(&mut self, name: &str) -> bool {
        let w = self.sim.world_mut();
        w.query::<&Name>().iter(w).any(|n| n.as_str() == name)
    }
}

/// ⭐⭐ **«Planks» numa ponte JÁ feita refaz as cópias** (plano §W4, 2.º ponto): de UMA tábua saem cinco
/// (cópias da última, com aparência e forma filha, «Plank 2» … «Plank 5»), as juntas novas levam os 500 N
/// afinados na que já havia, o poste da direita anda com a última; 12 e depois 3 tiram do fim.
/// **Mutações que sangram:** a cópia sem tirar a junta da original (juntas a dobrar) · as juntas novas
/// de fábrica (sem `tuned`) · o poste parado · encolher sem apagar as tábuas que saem.
#[test]
fn planks_numa_ponte_feita_refaz_as_copias() {
    let mut m = Mesa::draft(JointKind::Pin, "Bridge");
    m.planks("1");
    let plank = ph2d_app_joint_editor::doc::scope_options(
        ph2d_joint_templates::by_name(JointKind::Pin, "Bridge").expect("ponte"),
    )
    .iter()
    .position(|o| o.label == "Plank")
    .expect("Plank");
    m.click(&Hit::Make);
    assert_eq!(m.bridge_joints().len(), 2, "uma tábua: só os dois postes");
    let (_, r0) = m.object("Bridge Right Post");
    let (_, p1) = m.object("Plank 1");
    m.planks("2");
    m.click(&Hit::Scope(plank));
    m.click(&Hit::Flag(ph2d_app_joint_editor::doc::Flag::Breakable));
    m.click(&Hit::Field(Field::BreakForce));
    m.escreve("500");
    m.tecla(
        ph2d_editor_core::screens::hero::board_keys::BoardKey::Enter,
        None,
    );
    m.planks("5");
    for k in 1..=5 {
        let (b, _) = m.object(&format!("Plank {k}"));
        let w = m.sim.world_mut();
        let e = ph2d_ecs::Entity::from_bits(b);
        assert!(w.get::<Sprite>(e).is_some(), "Plank {k}: a aparência");
        assert!(
            ph2d_physics_ecs::forma_para_editar(w, e).is_some(),
            "Plank {k}: a forma filha"
        );
    }
    // Na Hierarchy, as cópias a seguir à tábua de onde saíram, antes do poste da direita.
    let asm =
        ph2d_ecs::Entity::from_bits(m.tab().doc.assembly().map(|a| a.bits).expect("montagem"));
    let names: Vec<String> = ph2d_ecs::ordered_children(m.sim.world(), asm)
        .into_iter()
        .filter_map(|c| m.sim.world().get::<Name>(c).map(|n| n.as_str().to_owned()))
        .collect();
    let pos = |n: &str| names.iter().position(|x| x == n).unwrap_or(usize::MAX);
    assert!(pos("Plank 5") < pos("Bridge Right Post"), "{names:?}");
    let j = m.bridge_joints();
    assert_eq!(j.len(), 6, "4 entre 5 tábuas + 2 postes, nenhuma a dobrar");
    let planks: Vec<&PhysicsJoint> = j.iter().filter(|j| j.role == 1).collect();
    assert_eq!(planks.len(), 4);
    assert!(
        planks
            .iter()
            .all(|j| j.break_enabled && (j.break_force - 500.0).abs() < 1e-3),
        "{planks:?}"
    );
    let step = 0.52;
    let (_, r) = m.object("Bridge Right Post");
    assert!(
        (r[0] - r0[0] - 4.0 * step).abs() < 1e-3,
        "o poste anda 4 passos: {}",
        r[0] - r0[0]
    );
    let (_, p5) = m.object("Plank 5");
    assert!((p5[0] - p1[0] - 4.0 * step).abs() < 1e-3);
    m.planks("12");
    assert_eq!(m.bridge_joints().len(), 13);
    m.planks("3");
    assert_eq!(m.bridge_joints().len(), 4, "2 entre 3 tábuas + 2 postes");
    assert!(m.exists("Plank 3") && !m.exists("Plank 4") && !m.exists("Plank 12"));
    let (_, r) = m.object("Bridge Right Post");
    assert!(
        (r[0] - r0[0] - 2.0 * step).abs() < 1e-3,
        "o poste volta: {}",
        r[0] - r0[0]
    );
    let toast = m
        .toasts
        .iter()
        .last()
        .map(|t| t.message.clone())
        .unwrap_or_default();
    assert!(toast.contains("Made 7 copies"), "{toast}");
}

/// ⭐ **O tecto medido da fila** (smoke do dono 16/11: um número grande no «Planks» travava o app — a corrente de
/// n tábuas custava ~n³ por passo). O campo aceita até `MAX_ROW_PARTS` (uma ponte cabe num quadro a 60 Hz,
/// tabela no doc 02) e o «Make the parts» faz exatamente essas.
/// **Mutações que sangram:** o campo sem o tecto · `count_of` sem o tecto.
#[test]
fn planks_para_no_tecto_medido_da_fila() {
    let max = ph2d_joint_factory::joint_factory::MAX_ROW_PARTS;
    let mut m = Mesa::draft(JointKind::Pin, "Bridge");
    m.click(&Hit::Field(Field::Count));
    m.escreve("9999");
    m.tecla(
        ph2d_editor_core::screens::hero::board_keys::BoardKey::Enter,
        None,
    );
    assert_eq!(m.tab().doc.value(Field::Count), Some(f32::from(max)));
    m.click(&Hit::Make);
    m.object(&format!("Plank {max}"));
    let w = m.sim.world_mut();
    let over = format!("Plank {}", max + 1);
    assert!(
        w.query::<&Name>().iter(w).all(|n| n.as_str() != over),
        "nem uma tábua acima do tecto"
    );
}

/// **A junta do artista debaixo da última tábua não se multiplica com as cópias**: «Planks» 2 → 5 copia a
/// última com o que mora debaixo dela, mas a junta que lá estava é da original — no mundo ficam as 6 da
/// ponte e UMA do artista. **Mutação que sangra:** a cópia sem tirar a junta copiada (N1).
#[test]
fn planks_nao_copia_a_junta_do_artista_debaixo_da_ultima_tabua() {
    let mut m = Mesa::draft(JointKind::Pin, "Bridge");
    m.planks("2");
    m.click(&Hit::Make);
    let (p1, _) = m.object("Plank 1");
    let (p2, _) = m.object("Plank 2");
    let w = m.sim.world_mut();
    let sid = |w: &ph2d_ecs::World, b: u64| {
        ph2d_ecs::stable_id_of(w, ph2d_ecs::Entity::from_bits(b)).map_or(0, |s| s.0)
    };
    let joint = PhysicsJoint {
        body_a: sid(w, p2),
        body_b: sid(w, p1),
        ..PhysicsJoint::default()
    };
    w.spawn((
        joint,
        ph2d_ecs::Transform::default(),
        Name::new("Artist Joint"),
        ph2d_ecs::ChildOf(ph2d_ecs::Entity::from_bits(p2)),
    ));
    ph2d_ecs::assign_missing_stable_ids(w);
    ph2d_ecs::assign_missing_sibling_order(w);
    m.planks("5");
    assert!(m.exists("Plank 5"));
    assert_eq!(m.bridge_joints().len(), 6, "4 entre 5 tábuas + 2 postes");
    let w = m.sim.world_mut();
    let all = w.query::<&PhysicsJoint>().iter(w).count();
    assert_eq!(all, 7, "as 6 da ponte + a do artista, nenhuma copiada");
}

/// **O passo das cópias é o da CENA**: o artista arrastou a última tábua (passo `0,82 × 0,2` em vez do
/// `0,52` do gabarito); «Planks» 2 → 4 põe a 3.ª e a 4.ª um e dois passos da cena adiante dela.
/// **Mutação que sangra:** sempre o passo do gabarito (N6).
#[test]
fn planks_copia_com_o_passo_da_cena_quando_o_artista_moveu_a_tabua() {
    let mut m = Mesa::draft(JointKind::Pin, "Bridge");
    m.planks("2");
    m.click(&Hit::Make);
    let (_, a) = m.object("Plank 1");
    let (b2, _) = m.object("Plank 2");
    {
        let w = m.sim.world_mut();
        let mut t = w
            .get_mut::<ph2d_ecs::Transform>(ph2d_ecs::Entity::from_bits(b2))
            .expect("pose");
        t.translation.x += 0.3;
        t.translation.y += 0.2;
    }
    let (_, b) = m.object("Plank 2");
    let step = [b[0] - a[0], b[1] - a[1]];
    assert!(
        (step[0] - 0.82).abs() < 1e-3 && (step[1] - 0.2).abs() < 1e-3,
        "{step:?}"
    );
    m.planks("4");
    for (k, ahead) in [("Plank 3", 1.0), ("Plank 4", 2.0)] {
        let (_, p) = m.object(k);
        assert!(
            (p[0] - b[0] - ahead * step[0]).abs() < 1e-3
                && (p[1] - b[1] - ahead * step[1]).abs() < 1e-3,
            "{k}: {p:?} (a última em {b:?}, passo {step:?})"
        );
    }
}
