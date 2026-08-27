//! ⭐ **Onde uma CÓPIA aterra** — irmão de `instance_verbs_tests` por assunto (e pelo tecto de
//! 600 LOC). Lá os gates dos verbos; aqui a pergunta que os dois reports do Enio fizeram: *«a cópia
//! nova está num sítio onde eu a vejo?»*.
//!
//! ⚠️ **A régua é `world_transform`, nunca o `Transform`.** Em espaço LOCAL uma cópia e o que ela
//! copiou concordam mesmo com o defeito presente — foi assim que a fixtura antiga (mestre na raiz
//! da cena, régua local) ficou verde por cima do §1.3 da auditoria de 2026-08-27.

use crate::instance_smoke::spawn_master;
use crate::instance_sync::MasterEcho;
use ph2d_ecs::{Children, Entity, InstanceOf, Name, SimWorld, Transform};

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// O dreno de um verbo, com o par de documentos vazio (estes gates não têm arte vetorial).
fn drain(
    verb: super::Verb,
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    echo: &mut MasterEcho,
    toasts: &mut ph2d_editor::ToastQueue,
    entity: Entity,
    step: [f32; 2],
) -> bool {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    super::drain(
        verb,
        sim,
        r,
        echo,
        entity.to_bits(),
        toasts,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
        step,
    )
}

fn piece(sim: &SimWorld, root: Entity, name: &str) -> Entity {
    let mut stack = vec![root];
    while let Some(e) = stack.pop() {
        if e != root && sim.world().get::<Name>(e).is_some_and(|n| n.0 == name) {
            return e;
        }
        if let Some(kids) = sim.world().get::<Children>(e) {
            stack.extend(kids.iter().copied());
        }
    }
    panic!("nao ha' peca chamada {name:?}");
}

/// ⭐⭐⭐ **UMA CÓPIA NUNCA ATERRA EM CIMA DO QUE VEIO** (report do Enio, 2026-08-26 → 27).
///
/// Duas formas idênticas sobrepostas fazem *«mudei o mestre»* e *«mudei a cópia por cima dele»*
/// serem o mesmo gesto na tela — e foi isso que fez a propagação **parecer morta estando viva**.
///
/// ⚠️ Os **três** lados: a 1.ª cópia sai um passo, a 2.ª sai dois (cascata), e o *Criar componente*
/// **não** desloca — ali a cópia tem de ficar exactamente onde a seleção estava.
///
/// (Mutação: `cascade` não escrever a translação ⇒ RED; cascatear no `Verb::Make` ⇒ RED.)
#[test]
fn a_placed_instance_never_lands_on_top_of_what_it_came_from() {
    let mut sim = SimWorld::new();
    let r = reg();
    let mut echo = MasterEcho::default();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let at = ph2d_core::Vec2::new(2.0, -1.0);
    let src = sim
        .world_mut()
        .spawn((Transform::from_translation(at), Name::new("Badge")))
        .id();
    let step = [0.5_f32, -0.25];
    // ⚠️ **Pelo DRENO, e não pela função** — os dois verbos partilham o `place_step`, e uma
    // mutação que cascateasse o *Criar componente* passava enquanto o gate chamava
    // `make_master` directamente. *Um gate que salta o dreno não mede o verbo, mede a função.*
    {
        let (mut sc, mut mp) = crate::instance_docs::empty_docs();
        assert!(
            super::drain(
                super::Verb::Make,
                &mut sim,
                &r,
                &mut echo,
                src.to_bits(),
                &mut toasts,
                &mut crate::instance_docs::OwnedDocs {
                    vec_scene: &mut sc,
                    vec_entities: &mut mp,
                },
                step,
            ),
            "o *Criar componente* nao fez nada"
        );
    }
    let master = src;
    let mut place = |sim: &mut SimWorld, echo: &mut MasterEcho| {
        let (mut sc, mut mp) = crate::instance_docs::empty_docs();
        super::drain(
            super::Verb::Place,
            sim,
            &r,
            echo,
            master.to_bits(),
            &mut toasts,
            &mut crate::instance_docs::OwnedDocs {
                vec_scene: &mut sc,
                vec_entities: &mut mp,
            },
            step,
        )
    };
    assert!(place(&mut sim, &mut echo), "o *Instantiate* nao fez nada");
    assert!(
        place(&mut sim, &mut echo),
        "o 2o *Instantiate* nao fez nada"
    );

    // As poses das instâncias, sem a que o *Criar componente* deixou no lugar.
    let master_id = sim.world().get::<ph2d_ecs::StableId>(master).expect("id").0;
    let mut poses: Vec<(f32, f32)> = {
        // ⚠️ Só a RAIZ de uma instância tem o elo a apontar para o `master_id`; as peças apontam
        // para as peças do mestre. Não é preciso um segundo componente para as distinguir.
        let mut q = sim.world_mut().query::<(&InstanceOf, &Transform)>();
        q.iter(sim.world())
            .filter(|(link, _)| link.master == master_id)
            .map(|(_, t)| (t.translation.x, t.translation.y))
            .collect()
    };
    poses.sort_by(|a, b| a.0.total_cmp(&b.0));
    assert_eq!(poses.len(), 3, "esperavam-se TRES copias: {poses:?}");
    for (i, (x, y)) in poses.iter().enumerate() {
        let want = (at.x + step[0] * i as f32, at.y + step[1] * i as f32);
        assert!(
            (x - want.0).abs() < 1e-4 && (y - want.1).abs() < 1e-4,
            "a copia {i} aterrou em {:?} e devia aterrar em {want:?} — a cascata (a copia 0 e' a \
             do *Criar componente*, que NAO desloca)",
            (x, y)
        );
    }
}

/// ⭐⭐⭐ **O *Instantiate* de uma receita ANINHADA aterra onde as irmãs estão** — auditoria §1.3.
///
/// O ramo passava `parent = None` LITERAL enquanto os outros dois chamadores da cópia profunda
/// derivam o pai da fonte. ⇒ a cópia caía na raiz da cena **com a pose LOCAL do mestre**: perdia-se
/// o transform de mundo inteiro do pai (sítio, tamanho **e** ângulo — medido: mundo (9,3) escala 2×
/// saía (0.5,0) escala 1×) e — pior — as cópias nº2..n deixavam de seguir o grupo que a nº1 seguia,
/// o que se lê como *«mover o grupo move umas instâncias e não outras»*.
///
/// ⚠️ Ver o cabeçalho quanto à régua: `a_placed_instance_never_lands_on_top_of_what_it_came_from`
/// ficou verde por cima disto por DUAS cegueiras independentes — a fixtura monta o mestre na raiz
/// da cena (onde a diferença é zero) **e** o oráculo lê `Transform`, que é local.
///
/// (Mutação: voltar a `None` no `Verb::Place` ⇒ RED nas duas asserções.)
#[test]
fn a_placed_instance_lands_where_a_nested_recipe_lives() {
    let mut sim = SimWorld::new();
    let r = reg();
    let mut echo = MasterEcho::default();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let group = sim
        .world_mut()
        .spawn((
            Transform::from_translation(ph2d_core::Vec2::new(8.0, 3.0)),
            Name::new("Group"),
        ))
        .id();
    let master = spawn_master(&mut sim);
    sim.world_mut()
        .entity_mut(master)
        .insert(ph2d_ecs::ChildOf(group));
    ph2d_ecs::assign_master_pieces(sim.world_mut());
    let at_master = ph2d_ecs::world_transform(sim.world(), master).expect("mestre");

    assert!(
        drain(
            super::Verb::Place,
            &mut sim,
            &r,
            &mut echo,
            &mut toasts,
            master,
            [0.0, 0.0],
        ),
        "o *Instantiate* nao fez nada"
    );
    let master_id = sim.world().get::<ph2d_ecs::StableId>(master).expect("id").0;
    let copy = {
        let mut q = sim.world_mut().query::<(Entity, &InstanceOf)>();
        q.iter(sim.world())
            .find(|(_, link)| link.master == master_id)
            .map(|(e, _)| e)
            .expect("a copia")
    };
    assert_eq!(
        sim.world().get::<ph2d_ecs::ChildOf>(copy).map(|c| c.0),
        Some(group),
        "a copia saiu do grupo em que a receita vive — mover o grupo passa a mover umas e nao outras"
    );
    let at_copy = ph2d_ecs::world_transform(sim.world(), copy).expect("copia");
    assert!(
        (at_copy.translation - at_master.translation).length() < 1e-4
            && (at_copy.scale.x - at_master.scale.x).abs() < 1e-4,
        "a copia aterrou em {:?} escala {:?} e a receita esta' em {:?} escala {:?} — o transform \
         de mundo do pai perdeu-se",
        at_copy.translation,
        at_copy.scale,
        at_master.translation,
        at_master.scale
    );
}

/// ⛔ **Uma receita na RAIZ da cena continua a aterrar na raiz** — o controlo negativo da cura
/// acima: derivar o pai não pode inventar um.
#[test]
fn a_placed_instance_of_a_root_recipe_stays_at_the_root() {
    let mut sim = SimWorld::new();
    let r = reg();
    let mut echo = MasterEcho::default();
    let mut toasts = ph2d_editor::ToastQueue::default();
    let master = spawn_master(&mut sim);
    assert!(drain(
        super::Verb::Place,
        &mut sim,
        &r,
        &mut echo,
        &mut toasts,
        master,
        [0.0, 0.0],
    ));
    let master_id = sim.world().get::<ph2d_ecs::StableId>(master).expect("id").0;
    let copy = {
        let mut q = sim.world_mut().query::<(Entity, &InstanceOf)>();
        q.iter(sim.world())
            .find(|(_, link)| link.master == master_id)
            .map(|(e, _)| e)
            .expect("a copia")
    };
    assert!(
        sim.world().get::<ph2d_ecs::ChildOf>(copy).is_none(),
        "a copia de uma receita solta ganhou um pai"
    );
    // A peça continua a existir — a cópia é profunda, não uma linha vazia.
    let _ = piece(&sim, copy, "Arm");
}
