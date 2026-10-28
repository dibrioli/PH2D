//! Os gates da [`super`] — a família LISTA de ponta a ponta (F30 §A1).

use super::*;
use crate::component_registry_for_tests::registo;
use ph2d_component_desc::lista::codigo;
use ph2d_ecs::{Timer, Timers, Transform};

fn timer(nome: &str, s: u64) -> Timer {
    Timer {
        name: nome.into(),
        duration_us: s * 1_000_000,
        ..Timer::default()
    }
}

/// Um objecto com três timers, já com ids.
fn cena() -> (SimWorld, Entity, [ItemId; 3]) {
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Timers(vec![timer("a", 1), timer("b", 2), timer("c", 3)]),
        ))
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let t = sim.world().get::<Timers>(e).expect("timers");
    let ids = [t.0[0].item, t.0[1].item, t.0[2].item];
    (sim, e, ids)
}

fn duracao(id: ItemId) -> FieldKey {
    cl::chave(codigo::TIMERS, 2, 0, id).expect("cabe")
}

/// A curva escreve `x` em `k` pelo funil inteiro (o `CanaisEscritos` e o `aplicar`).
fn curva(sim: &mut SimWorld, reg: &ComponentRegistry, e: Entity, k: FieldKey, x: f32) -> bool {
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(e.to_bits(), k, x);
    aplicar(
        sim,
        reg,
        &mut PreviewDrive::default(),
        &mut TextosDosCanais::default(),
    )
}

fn duracoes(sim: &SimWorld, e: Entity) -> Vec<(String, u64)> {
    let t = sim.world().get::<Timers>(e).expect("timers");
    t.0.iter()
        .map(|t| (t.name.clone(), t.duration_us))
        .collect()
}

/// ⭐⭐⭐ **A curva conduz a duração de um timer** — e o eco publica o que ficou.
#[test]
fn a_curva_conduz_a_duracao_de_um_timer() {
    let reg = registo();
    let (mut sim, e, ids) = cena();
    assert!(curva(&mut sim, &reg, e, duracao(ids[1]), 5_000_000.0));
    assert_eq!(duracoes(&sim, e)[1], ("b".into(), 5_000_000));
    assert_eq!(duracoes(&sim, e)[0].1, 1_000_000, "o vizinho não mexe");
    publicar(
        &mut sim,
        &reg,
        &[e.to_bits()],
        &mut TextosDosCanais::default(),
    );
    let lidos = sim.world().resource::<CanaisLidos>();
    assert_eq!(lidos.valor(e.to_bits(), duracao(ids[1])), Some(5_000_000.0));
}

/// ⭐⭐⭐ **Apagar um item do meio não muda a chave do vizinho**: a chave de `c` continua a
/// conduzir `c`, e a de `b` fica órfã (não escreve ninguém).
#[test]
fn apagar_um_item_do_meio_nao_muda_a_chave_do_vizinho() {
    let reg = registo();
    let (mut sim, e, ids) = cena();
    sim.world_mut()
        .get_mut::<Timers>(e)
        .expect("timers")
        .0
        .remove(1);
    assert!(curva(&mut sim, &reg, e, duracao(ids[2]), 9_000_000.0));
    assert_eq!(
        duracoes(&sim, e),
        vec![("a".into(), 1_000_000), ("c".into(), 9_000_000)]
    );
    assert!(
        !curva(&mut sim, &reg, e, duracao(ids[1]), 7_000_000.0),
        "órfã"
    );
    assert_eq!(duracoes(&sim, e)[1].1, 9_000_000);
}

/// ⭐⭐ **Reordenar os itens não muda as chaves**: cada chave segue o seu item.
#[test]
fn reordenar_os_itens_nao_muda_as_chaves() {
    let reg = registo();
    let (mut sim, e, ids) = cena();
    sim.world_mut()
        .get_mut::<Timers>(e)
        .expect("timers")
        .0
        .reverse();
    assert!(curva(&mut sim, &reg, e, duracao(ids[0]), 4_000_000.0));
    assert_eq!(
        duracoes(&sim, e)[2],
        ("a".into(), 4_000_000),
        "`a` está no fim agora"
    );
    let mut t = TextosDosCanais::default();
    let pub_ = campos(&reg, sim.world(), e, &mut t);
    assert!(pub_.contains(&(duracao(ids[2]), 3_000_000.0)), "{pub_:?}");
}

/// Um item ainda sem id (acabado de acrescentar) não tem canal até o passe lho dar.
#[test]
fn um_item_sem_id_nao_publica() {
    let reg = registo();
    let mut sim = SimWorld::new();
    let e = sim.world_mut().spawn(Timers(vec![timer("a", 1)])).id();
    let mut t = TextosDosCanais::default();
    assert!(campos(&reg, sim.world(), e, &mut t).is_empty());
}

/// ⭐ **Toda lista do catálogo publica os campos dos itens** — um objecto com um item de cada.
#[test]
fn toda_lista_do_catalogo_publica_os_campos_dos_itens() {
    use ph2d_ecs::state_machine::{MachineState, Transition};
    let reg = registo();
    let mut sim = SimWorld::new();
    let mut anc = ph2d_ecs::NamedAnchorList::default();
    anc.insert(ph2d_ecs::NamedAnchor::socket("boca"))
        .expect("cabe");
    let e = sim
        .world_mut()
        .spawn((
            Transform::IDENTITY,
            Timers(vec![timer("a", 1)]),
            ph2d_ecs::Tweens(vec![ph2d_tween::Tween::default()]),
            ph2d_ecs::StateMachine {
                states: vec![MachineState::default()],
                transitions: vec![Transition::default()],
                initial: 0,
            },
            ph2d_ecs::CounterWatch(vec![ph2d_ecs::CounterWatchRow::default()]),
            ph2d_ecs::SignalActions(vec![ph2d_ecs::SignalAction::default()]),
            ph2d_ecs::SignalOnAction(vec![ph2d_ecs::ActionTriggerRow::default()]),
            ph2d_ecs::ShakeEmitter(vec![ph2d_ecs::ShakeSource::default()]),
            anc,
        ))
        .id();
    sim.world_mut()
        .entity_mut(e)
        .insert(ph2d_physics_ecs::Health {
            resistances: vec![ph2d_physics_ecs::Resistance::default()],
            ..ph2d_physics_ecs::Health::default()
        });
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let mut t = TextosDosCanais::default();
    let pub_ = campos(&reg, sim.world(), e, &mut t);
    for l in lista::todas() {
        for f in l.campos {
            // A caixa e o miolo da âncora só publicam quando a âncora os tem.
            if l.codigo == codigo::ANCORAS && f.field_id >= 4 {
                continue;
            }
            assert!(
                pub_.iter().any(|(k, _)| cl::resolver(*k)
                    .is_some_and(|(m, g, _)| m.codigo == l.codigo && g.field_id == f.field_id)),
                "{} #{} não publicou",
                l.componente,
                f.field_id
            );
        }
    }
}

/// O NOME da linha diz a POSIÇÃO de agora (só nome: a chave continua a ser o id).
#[test]
fn a_posicao_do_item_acompanha_a_lista() {
    let reg = registo();
    let (mut sim, e, ids) = cena();
    sim.world_mut()
        .get_mut::<Timers>(e)
        .expect("timers")
        .0
        .remove(0);
    publicar(
        &mut sim,
        &reg,
        &[e.to_bits()],
        &mut TextosDosCanais::default(),
    );
    let pos = sim.world().resource::<cl::PosicoesDosItens>();
    assert_eq!(pos.0.get(&(e.to_bits(), ids[2].0)), Some(&2));
    assert_eq!(pos.0.get(&(e.to_bits(), ids[0].0)), None, "o apagado sai");
}

/// ⭐⭐ **O eco acha o item pelo id, não pela posição** — com ids ≠ posições (um apagado, a ordem
/// invertida), a curva escreve um valor que a gravação arredonda e o publicado é o que ela mandou.
#[test]
fn o_eco_acha_o_item_pelo_id_depois_de_apagar_e_reordenar() {
    let reg = registo();
    let (mut sim, e, ids) = cena();
    {
        let mut t = sim.world_mut().get_mut::<Timers>(e).expect("timers");
        t.0.remove(0);
        t.0.reverse();
    }
    for (id, x) in [(ids[2], 6_000_000.5_f32), (ids[1], 8_000_000.5)] {
        assert!(curva(&mut sim, &reg, e, duracao(id), x));
        publicar(
            &mut sim,
            &reg,
            &[e.to_bits()],
            &mut TextosDosCanais::default(),
        );
        let lidos = sim.world().resource::<CanaisLidos>();
        assert_eq!(lidos.valor(e.to_bits(), duracao(id)), Some(x), "{id:?}");
    }
}

/// Um quadro da curva com o ledger da sessão (o `PreviewDrive` vive entre quadros) e o «outra mão»
/// do ledger, como o quadro dos scripts o corre.
fn quadro(
    sim: &mut SimWorld,
    reg: &ComponentRegistry,
    drive: &mut PreviewDrive,
    k: FieldKey,
    x: f32,
) {
    let b = sim_bits(sim);
    sim.world_mut()
        .get_resource_or_insert_with(CanaisEscritos::default)
        .escrever(b, k, x);
    aplicar(sim, reg, drive, &mut TextosDosCanais::default());
    drive.fields.observe(sim);
}

/// A entidade com `Timers` da cena (uma só).
fn sim_bits(sim: &mut SimWorld) -> u64 {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Timers)>();
    q.iter(w).next().expect("timers").0.to_bits()
}

/// O REBOBINAR dos scripts (`script_bridge::rewind`, que corre em TODO quadro parado no início):
/// a metade que toca no ledger dos campos.
fn rebobinar(sim: &mut SimWorld, drive: &mut PreviewDrive) {
    drive.fields.release(sim);
}

/// ⭐⭐⭐ **Apagar o 1.º timer não congela a curva do 2.º** (smoke do dono 25/10, cena 35): a mão
/// apaga o Short, a curva continua a conduzir o Long, e o rebobinar (parado no início) não repõe o
/// valor de quando a mão mexeu — senão o Auto Key gravava esse valor por cima da 1.ª chave.
#[test]
fn apagar_o_primeiro_timer_nao_congela_a_curva_do_segundo() {
    let reg = registo();
    let (mut sim, e, ids) = cena();
    let mut drive = PreviewDrive::default();
    quadro(&mut sim, &reg, &mut drive, duracao(ids[1]), 2_000_000.0);
    sim.world_mut()
        .get_mut::<Timers>(e)
        .expect("timers")
        .0
        .remove(0);
    drive.fields.observe(&sim);
    quadro(&mut sim, &reg, &mut drive, duracao(ids[1]), 5_000_000.0);
    rebobinar(&mut sim, &mut drive);
    assert_eq!(
        duracoes(&sim, e),
        vec![("b".into(), 5_000_000), ("c".into(), 3_000_000)],
        "o rebobinar dos scripts devolveu o que a CURVA conduz"
    );
}

/// ⭐⭐ **Reordenar à mão não congela a curva** — o mesmo ledger, a outra metade da família.
#[test]
fn reordenar_os_timers_nao_congela_a_curva() {
    let reg = registo();
    let (mut sim, e, ids) = cena();
    let mut drive = PreviewDrive::default();
    quadro(&mut sim, &reg, &mut drive, duracao(ids[0]), 4_000_000.0);
    sim.world_mut()
        .get_mut::<Timers>(e)
        .expect("timers")
        .0
        .reverse();
    drive.fields.observe(&sim);
    quadro(&mut sim, &reg, &mut drive, duracao(ids[0]), 6_000_000.0);
    rebobinar(&mut sim, &mut drive);
    assert_eq!(duracoes(&sim, e)[2], ("a".into(), 6_000_000));
}

/// ⭐⭐⭐ **Desfazer o apagar devolve o MESMO item e a curva** (smoke do dono 25/10): o Ctrl+Z repõe
/// a fotografia (o autorado, `PreviewDrive::substitute`), o passe dos itens não re-atribui nada, e
/// a curva volta a conduzir o Long — o rebobinar não lhe repõe o valor da fotografia.
#[test]
fn desfazer_o_apagar_de_um_timer_devolve_o_mesmo_item_e_a_curva() {
    let reg = registo();
    let (mut sim, e, ids) = cena();
    let mut drive = PreviewDrive::default();
    quadro(&mut sim, &reg, &mut drive, duracao(ids[1]), 2_500_000.0);
    // A fotografia do passo (o `capture_project`): o autorado, e o vivo de volta.
    let vivo = drive.substitute(&mut sim);
    let foto = sim.world().get::<Timers>(e).expect("timers").clone();
    PreviewDrive::restore(&mut sim, &vivo);
    sim.world_mut()
        .get_mut::<Timers>(e)
        .expect("timers")
        .0
        .remove(0);
    drive.fields.observe(&sim);
    quadro(&mut sim, &reg, &mut drive, duracao(ids[1]), 3_500_000.0);
    // O Ctrl+Z: o mundo volta à fotografia.
    sim.world_mut().entity_mut(e).insert(foto);
    drive.fields.observe(&sim);
    assert!(
        !ph2d_ecs::item_id::assign_missing_item_ids(sim.world_mut()),
        "nada a re-atribuir"
    );
    quadro(&mut sim, &reg, &mut drive, duracao(ids[1]), 4_000_000.0);
    rebobinar(&mut sim, &mut drive);
    let t = sim.world().get::<Timers>(e).expect("timers");
    assert_eq!(t.0.iter().map(|x| x.item).collect::<Vec<_>>(), ids.to_vec());
    assert_eq!(
        duracoes(&sim, e),
        vec![
            ("a".into(), 1_000_000),
            ("b".into(), 4_000_000),
            ("c".into(), 3_000_000)
        ]
    );
}

/// ⭐ F39 (F32 §6.3): a chave sobre o `from`/`to` de uma seta guarda o `ItemId` do ESTADO, não a
/// posição — apagar um estado anterior (que reescreve as setas) não faz a curva apontar o vizinho.
#[test]
fn a_chave_de_uma_seta_segue_o_estado_depois_de_apagar_um_anterior() {
    use ph2d_ecs::StateMachine;
    use ph2d_ecs::state_machine::{MachineState, Transition};
    use ph2d_editor_core::statemachine_edits::StateMachineFieldEdit as E;
    let reg = registo();
    let mut sim = SimWorld::new();
    let estado = |n: &str| MachineState {
        name: n.into(),
        ..MachineState::default()
    };
    let e = sim
        .world_mut()
        .spawn(StateMachine {
            states: vec![estado("a"), estado("b"), estado("c")],
            transitions: vec![Transition {
                from: 1,
                on: "vai".into(),
                to: 2,
                ..Transition::default()
            }],
            initial: 0,
        })
        .id();
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let m = sim.world().get::<StateMachine>(e).expect("máquina").clone();
    let (b, c, seta) = (m.states[1].item, m.states[2].item, m.transitions[0].item);
    let para = cl::chave(codigo::SETAS, 6, 0, seta).expect("cabe");
    let mut t = TextosDosCanais::default();
    #[expect(clippy::cast_precision_loss, reason = "ids pequenos")]
    let (vb, vc) = (b.0 as f32, c.0 as f32);
    assert!(
        campos(&reg, sim.world(), e, &mut t).contains(&(para, vc)),
        "publica o id de `c`"
    );
    assert!(crate::statemachine_inspector::apply_statemachine_edit(
        sim.world_mut(),
        e.to_bits(),
        &E::RemoveState(0)
    ));
    let to = |sim: &SimWorld| {
        sim.world()
            .get::<StateMachine>(e)
            .expect("máquina")
            .transitions[0]
            .to
    };
    assert_eq!(to(&sim), 1, "a edição reescreveu a seta para `c`");
    // A chave gravada antes de apagar (o id de `b`) conduz a seta para `b`, agora na posição 0.
    assert!(curva(&mut sim, &reg, e, para, vb));
    assert_eq!(to(&sim), 0, "o id de `b` escreveu a posição de `b`");
    assert!(curva(&mut sim, &reg, e, para, vc));
    assert_eq!(
        to(&sim),
        1,
        "e o de `c` a de `c`, não a do estado que estava ali antes"
    );
}
