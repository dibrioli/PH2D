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

/// O carro por fazer mostra a suspensão de fábrica e o Drive; escolher «None» faz as rodas sem motor.
/// **Mutação que sangra:** o Drive do rascunho sem chegar ao `Make`.
#[test]
fn o_carro_por_fazer_mostra_a_suspensao_de_fabrica_e_o_drive() {
    let mut m = Mesa::draft(JointKind::Wheel, "Car");
    let t = ph2d_joint_templates::by_name(JointKind::Wheel, "Car").expect("carro");
    let fab = t.joints[0].joint(0.0);
    assert_eq!(m.tab().doc.value(Field::Stiffness), Some(fab.stiffness));
    m.click(&Hit::Drive(ph2d_app_joint_editor::doc::Drive::None));
    m.click(&Hit::Make);
    let sid = m.tab().doc.assembly().map(|a| a.sid).expect("montagem");
    let joints = ph2d_joint_assembly::joint_assembly::joints_of(m.sim.world_mut(), sid);
    assert_eq!(joints.len(), 2);
    assert!(joints.iter().all(|(_, j)| !j.motor_enabled), "Drive None");
}
