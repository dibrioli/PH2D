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
    fn new() -> Self {
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
