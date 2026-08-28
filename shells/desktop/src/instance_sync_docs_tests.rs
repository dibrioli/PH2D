//! Os gates da propagação de documentos (ADR-0164 / F4.6b).
//!
//! ⚠️ **O oráculo é o CONTEÚDO do path da instância**, e nunca *«o passe escreveu»*: a peça tem o
//! id dela para sempre, então um gate sobre o `VecPathRef` mediria a coisa que por construção não
//! muda.

use crate::instance_docs::OwnedDocs;
use crate::instance_sync::{MasterEcho, sync_instances};
use ph2d_ecs::{ChildOf, Entity, MasterRoot, Name, SimWorld, Transform, VecPathRef};
use ph2d_physics_ecs::PhysicsBridge;
use ph2d_vec_scene::{VecPathId, VecScene, rectangle};

use crate::vec_entities::VecEntityMap;

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// A cena de trabalho: uma receita com UMA peça vetorial, e uma instância dela.
struct Fixture {
    sim: SimWorld,
    scene: VecScene,
    map: VecEntityMap,
    registry: ph2d_ecs::scene::ComponentRegistry,
    bridge: PhysicsBridge,
    echo: MasterEcho,
    master_path: VecPathId,
    inst_piece: Entity,
    master_piece: Entity,
    inst_root: Entity,
}

impl Fixture {
    /// A cópia com arte **PRÓPRIA** (*Instantiate*) — o caso de sempre.
    fn new() -> Self {
        Self::with_link(crate::instantiate::ArtLink::Own)
    }

    /// ⭐ **A cópia LIGADA** (*Instantiate Linked*, o `Alt+D`) — Enio, 2026-08-27.
    fn linked() -> Self {
        Self::with_link(crate::instantiate::ArtLink::Shared)
    }

    fn with_link(link: crate::instantiate::ArtLink) -> Self {
        let mut sim = SimWorld::new();
        let mut scene = VecScene::new();
        let mut map = VecEntityMap::new();
        let registry = reg();
        let master_path = scene.push_path(rectangle([-1.0, -1.0], [1.0, 1.0]));
        let root = sim
            .world_mut()
            .spawn((Transform::IDENTITY, Name::new("Badge"), MasterRoot))
            .id();
        let master_piece = sim
            .world_mut()
            .spawn((
                Transform::IDENTITY,
                Name::new("Plate"),
                VecPathRef(master_path),
                ChildOf(root),
            ))
            .id();
        map.insert(master_path, master_piece.to_bits());
        ph2d_ecs::assign_master_pieces(sim.world_mut());
        let inst_root = crate::instantiate::instantiate_master(
            &mut sim,
            &registry,
            root,
            None,
            &mut OwnedDocs {
                vec_scene: &mut scene,
                vec_entities: &mut map,
            },
            link,
        )
        .expect("instanciou");
        let inst_piece = piece(&sim, inst_root, "Plate");
        let mut f = Self {
            sim,
            scene,
            map,
            registry,
            bridge: PhysicsBridge::new(),
            echo: MasterEcho::default(),
            master_path,
            inst_piece,
            master_piece,
            inst_root,
        };
        f.pass(); // o 1.º passe semeia o eco
        f
    }

    fn pass(&mut self) -> usize {
        sync_instances(
            &mut self.sim,
            &self.registry,
            &self.bridge,
            &mut self.echo,
            &mut OwnedDocs {
                vec_scene: &mut self.scene,
                vec_entities: &mut self.map,
            },
        )
    }

    fn inst_path(&self) -> VecPathId {
        self.sim
            .world()
            .get::<VecPathRef>(self.inst_piece)
            .expect("a peca da instancia tem geometria")
            .0
    }

    /// Move o 1.º vértice de um path — a edição mais curta que muda a forma.
    fn nudge(&mut self, id: VecPathId, dx: f64) {
        let p = self.scene.path_mut(id).expect("path");
        p.verts[0].anchor[0] += dx;
    }

    fn verts(&self, id: VecPathId) -> Vec<[f64; 2]> {
        self.scene
            .path(id)
            .expect("path")
            .verts
            .iter()
            .map(|v| v.anchor)
            .collect()
    }
}

fn piece(sim: &SimWorld, root: Entity, name: &str) -> Entity {
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if e != root && sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) {
            return e;
        }
        if let Some(kids) = sim.world().get::<ph2d_ecs::Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    panic!("nao ha' peca chamada {name:?}");
}

/// ⭐⭐⭐ **Editar a FORMA da receita muda todas as instâncias** — o smoke-gate 2 da fase, agora
/// para arte vetorial.
///
/// ⚠️ E os **dois** lados: a forma chega **e** o id da instância não se mexe. Escrever o id do
/// mestre poria as duas a apontar para o mesmo documento — o defeito que a F4.6a existe para não
/// cometer.
///
/// (Mutação: apagar a chamada a `sync_one` ⇒ RED na forma.)
#[test]
fn editing_the_master_shape_reaches_the_instance() {
    let mut f = Fixture::new();
    let before_id = f.inst_path();
    f.nudge(f.master_path, 3.0);
    assert!(f.pass() > 0, "o passe nao escreveu nada");
    assert_eq!(
        f.verts(f.inst_path()),
        f.verts(f.master_path),
        "a instancia nao recebeu a forma da receita"
    );
    assert_eq!(
        f.inst_path(),
        before_id,
        "a instancia trocou de path — as duas passam a escrever no mesmo documento"
    );
}

/// ⚠️ **E o passe continua a ser um PONTO FIXO** com arte vetorial — sem isto, cada quadro viraria
/// um passo de undo para sempre.
///
/// (Mutação: comparar `p.id` junto com o conteúdo ⇒ RED, porque os ids diferem de propósito.)
#[test]
fn a_second_pass_over_vector_art_writes_nothing() {
    let mut f = Fixture::new();
    f.nudge(f.master_path, 3.0);
    f.pass();
    assert_eq!(f.pass(), 0, "o passe deixou de ser ponto fixo");
}

/// ⭐⭐ **Editar a forma de UMA cópia vira EXCEPÇÃO** — e a receita deixa de a alcançar.
///
/// ⚠️ O 3.º lado é a lei do empate, já declarada: quando o **mestre** mexe, ele ganha.
///
/// (Mutação: nunca inserir a chave ⇒ RED, a cópia perde a forma dela no passe seguinte.)
#[test]
fn a_shape_edited_on_the_copy_becomes_an_override() {
    let mut f = Fixture::new();
    let id = f.inst_path();
    f.nudge(id, -5.0);
    let mine = f.verts(id);
    f.pass();
    assert_eq!(f.verts(id), mine, "a receita achatou a forma da copia");
    assert_eq!(
        f.sim
            .world()
            .get::<ph2d_ecs::ObjectInstance>(f.inst_root)
            .map_or(0, |o| o.overrides.len()),
        1,
        "a edicao na copia nao virou excepcao"
    );
    // E a receita continua a poder mandar.
    f.nudge(f.master_path, 9.0);
    f.pass();
    assert_eq!(
        f.verts(id),
        mine,
        "o mestre atropelou uma excepcao — a lei diz que ele so' ganha o EMPATE"
    );
}

/// ⭐⭐⭐ **A MESMA edição numa cópia LIGADA SOBE à receita** (Enio, 2026-08-27) — o `Alt+D`.
///
/// > *«Podemos criar um modo similar ao Blender onde em qualquer que se mudar todas mudam?»*
///
/// ⚠️ **É o gémeo exacto de [`a_shape_edited_on_the_copy_becomes_an_override`], e de propósito:**
/// a mesma cena, o mesmo gesto, o mesmo passe — muda só a lei que a cópia segue. Um gate do modo
/// ligado numa cena diferente não provaria que é a MARCA que decide.
///
/// ⚠️ E a metade que mantém o ponto fixo: **não** pode virar excepção (senão a cópia editada ficava
/// surda à receita), e o passe seguinte não pode reescrever nada.
///
/// (Mutação: apagar o ramo do `LinkedArt` no `sync_one` ⇒ RED nas duas primeiras asserções.)
#[test]
fn a_shape_edited_on_a_linked_copy_rises_to_the_master() {
    let mut f = Fixture::linked();
    let id = f.inst_path();
    f.nudge(id, -5.0);
    let mine = f.verts(id);
    f.pass();
    assert_eq!(
        f.verts(f.master_path),
        mine,
        "a edicao da copia LIGADA nao subiu — ela ficou uma excepcao dela, que e' o outro modo"
    );
    assert_eq!(
        f.sim
            .world()
            .get::<ph2d_ecs::ObjectInstance>(f.inst_root)
            .map_or(0, |o| o.overrides.len()),
        0,
        "a copia LIGADA capturou uma excepcao — ela ficaria surda a' receita para sempre"
    );
    // ⚠️ O ponto fixo: com o eco em dia, o passe seguinte não escreve nada.
    assert_eq!(
        f.pass(),
        0,
        "o passe seguinte voltou a escrever — a subida repete-se"
    );
    assert_eq!(f.verts(id), mine, "a forma da copia mudou sozinha");
}

/// ⭐ **APLICAR ao mestre escreve o CONTEÚDO, nunca o id.**
///
/// ⚠️ Pelo caminho geral (`insert_from_bytes`) o mestre receberia o `VecPathRef` da instância, e as
/// duas passariam a apontar para o mesmo path.
///
/// (Mutação: deixar o `apply_one` fora do `apply_to_master` ⇒ RED.)
#[test]
fn applying_a_shape_to_the_master_writes_the_content_not_the_id() {
    let mut f = Fixture::new();
    let id = f.inst_path();
    f.nudge(id, -5.0);
    f.pass(); // vira excepção
    let mine = f.verts(id);

    let n = {
        let mut docs = OwnedDocs {
            vec_scene: &mut f.scene,
            vec_entities: &mut f.map,
        };
        crate::instance_verbs::apply_to_master(
            &mut f.sim,
            &f.registry,
            &mut f.echo,
            f.inst_piece,
            &mut docs,
        )
        .expect("e' uma instancia")
    };
    assert_eq!(n, 1, "o Apply nao levou a forma");
    assert_eq!(
        f.verts(f.master_path),
        mine,
        "a receita nao recebeu a forma da copia"
    );
    assert_eq!(
        f.sim
            .world()
            .get::<VecPathRef>(f.master_piece)
            .expect("a receita tem geometria")
            .0,
        f.master_path,
        "a receita passou a apontar para o path da INSTANCIA"
    );
}

/// ⭐⭐⭐ **E as IRMÃS recebem** — report do Enio, 2026-08-27: *«nem todas as instâncias aceitam a
/// edição dos pontos da shape»*.
///
/// ⛔⛔ **O defeito era a subida atualizar o ECO.** Medido por sonda, com três cópias:
///
/// ```text
/// editei a LIGADA -> mestre=-2.0  copias=[-1.0, -1.0, -2.0]  overrides=[0, 0, 0]
///    +1 quadro:      mestre=-2.0  copias=[-1.0, -1.0, -2.0]   <- as irmas NUNCA recebem
///    e a seguir:                             overrides=[1, 1, 0]  <- e ficam SURDAS
/// ```
///
/// O eco é a memória de *quem se mexeu*. Ensinado o valor novo no mesmo passe, o quadro seguinte
/// conclui *«o mestre não se mexeu»* — e cada irmã, que ainda tem a forma velha, lê-se como *«só a
/// instância mexeu»* e captura uma excepção que ninguém pediu. ⇒ **as duas metades do report** (as
/// irmãs não mudam · depois recusam mandar) eram o mesmo bug.
///
/// ⚠️ **Este gate precisa de DUAS instâncias, e é por isso que ele não cabia no irmão**
/// [`a_shape_edited_on_a_linked_copy_rises_to_the_master`]: com uma só, a subida é indistinguível
/// de uma subida que ninguém recebe. *Uma fixtura de um não pode medir «e as outras».*
///
/// (Mutação: repor o `next_master.insert(..)` na subida ⇒ RED na irmã.)
#[test]
fn a_linked_copy_rises_and_the_sisters_receive_it() {
    let mut sim = SimWorld::new();
    let mut scene = VecScene::new();
    let mut map = VecEntityMap::new();
    let registry = reg();
    let master_path = scene.push_path(rectangle([-1.0, -1.0], [1.0, 1.0]));
    let root = sim
        .world_mut()
        .spawn((Transform::IDENTITY, Name::new("Badge"), MasterRoot))
        .id();
    let mpiece = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Name::new("Plate"),
            VecPathRef(master_path),
            ChildOf(root),
        ))
        .id();
    map.insert(master_path, mpiece.to_bits());
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    // ⚠️⚠️ **A ORDEM é a metade que decide, e a 1.ª versão deste gate não a tinha** — ele criava
    // DUAS ligadas e a mutação SOBREVIVEU. Uma irmã processada DEPOIS da que sobe lê o mestre já
    // novo no mesmo passe e recebe de qualquer maneira; quem o eco engana é a irmã processada
    // ANTES, que só pode receber no passe seguinte. `live_instances` ordena por `StableId`, logo a
    // irmã tem de NASCER primeiro. *Um gate cuja fixtura esconde a ordem mede a metade fácil.*
    let mut roots = Vec::new();
    for link in [
        crate::instantiate::ArtLink::Own,
        crate::instantiate::ArtLink::Shared,
    ] {
        roots.push(
            crate::instantiate::instantiate_master(
                &mut sim,
                &registry,
                root,
                None,
                &mut OwnedDocs {
                    vec_scene: &mut scene,
                    vec_entities: &mut map,
                },
                link,
            )
            .expect("instanciou"),
        );
    }
    let id_of = |sim: &SimWorld, e: Entity| sim.world().get::<ph2d_ecs::StableId>(e).expect("id").0;
    assert!(
        id_of(&sim, roots[0]) < id_of(&sim, roots[1]),
        "a irma nao nasceu antes da ligada — o passe processa-a DEPOIS e o gate mede a metade facil"
    );
    let bridge = PhysicsBridge::new();
    let mut echo = MasterEcho::default();
    let run = |sim: &mut SimWorld,
               scene: &mut VecScene,
               map: &mut VecEntityMap,
               echo: &mut MasterEcho| {
        sync_instances(
            sim,
            &registry,
            &bridge,
            echo,
            &mut OwnedDocs {
                vec_scene: scene,
                vec_entities: map,
            },
        )
    };
    run(&mut sim, &mut scene, &mut map, &mut echo); // semeia o eco
    let ids: Vec<VecPathId> = roots
        .iter()
        .map(|&r| {
            sim.world()
                .get::<VecPathRef>(piece(&sim, r, "Plate"))
                .expect("geometria")
                .0
        })
        .collect();
    let x = |sc: &VecScene, id: VecPathId| sc.path(id).expect("path").verts[0].anchor[0];

    // O gesto: mover um ponto da cópia LIGADA (a segunda).
    scene.path_mut(ids[1]).expect("path").verts[0].anchor[0] = -7.0;
    // Dois quadros: o 1.º sobe, o 2.º leva às irmãs. (O passe é um ponto fixo, não um oráculo
    // instantâneo — e um quadro de atraso é invisível, uma irmã surda para sempre não é.)
    run(&mut sim, &mut scene, &mut map, &mut echo);
    run(&mut sim, &mut scene, &mut map, &mut echo);

    assert!(
        (x(&scene, master_path) + 7.0).abs() < 1e-9,
        "a edicao da copia LIGADA nao subiu: mestre={}",
        x(&scene, master_path)
    );
    assert!(
        (x(&scene, ids[0]) + 7.0).abs() < 1e-9,
        "a IRMA nao recebeu: ela ficou em {} enquanto o mestre esta' em {}",
        x(&scene, ids[0]),
        x(&scene, master_path)
    );
    for (i, &r) in roots.iter().enumerate() {
        assert_eq!(
            sim.world()
                .get::<ph2d_ecs::ObjectInstance>(r)
                .map_or(0, |o| o.overrides.len()),
            0,
            "a copia {} capturou uma excepcao FALSA — ela fica surda a' receita para sempre",
            i + 1
        );
    }
    // ⚠️ E o ponto fixo chega: um 3.º passe não escreve nada.
    assert_eq!(
        run(&mut sim, &mut scene, &mut map, &mut echo),
        0,
        "o passe nao assentou — a subida repete-se todo o quadro"
    );
}
