//! ⭐⭐⭐ **A CASCATA** — nenhuma cópia aterra em cima de outra, e o degrau é só das MINHAS
//! cópias.
//!
//! # Porque é um ficheiro irmão
//!
//! O tecto de 600 LOC do shell (HR-18) — e o corte é por RESPONSABILIDADE: o
//! [`super::tests`] mede *o que cada verbo FAZ*, e isto mede *onde a cópia POUSA*, que é a
//! pergunta que dois reports do dono levantaram e nenhum verbo sozinho responde.
//!
//! # ⛔⛔ Estes dois gates nasceram de um CENSO, e por pouco não nasciam (F4.6c, 2026-09-07)
//!
//! A cascata do dreno geral ([`super::cascade`]) declara, no próprio doc, *«a lei é a que o verbo
//! VETORIAL já tinha»* — e a **única régua dela vivia do outro lado**, em
//! `vec_component_edit_tests`, a exercitar o motor `VecInstance`. Apagar aquele motor sem esta
//! porta deixaria a lei **herdada por comentário e provada por ninguém**.
//!
//! ⚠️ *Um censo de fatia tem de contar as RÉGUAS e não só as features* — esta linha já se queimou
//! duas vezes a contar verbos a menos, e a terceira teria sido contar gates a menos.

use super::VerbRefusal;
// ⚠️ **A FIXTURA é partilhada de propósito** — o que se dividiu foi o gate, não a cena. Uma
// segunda montagem do mesmo rig divergiria da irmã no primeiro campo que alguém acrescentasse.
use super::tests::plain_rig;
use crate::instance_sync::MasterEcho;
use ph2d_ecs::{Entity, SimWorld, Transform};

fn reg() -> ph2d_ecs::scene::ComponentRegistry {
    crate::init::build_component_registry()
}

/// ⚠️ **Sem documentos vetoriais** — ver `crate::instance_docs`.
fn make(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    entity: Entity,
) -> Result<(Entity, Entity), VerbRefusal> {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    super::make_master(
        sim,
        r,
        entity,
        &mut crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        },
    )
}

/// O dreno do *Instantiate* com o degrau ESCOLHIDO — o irmão do [`place`], para as réguas da
/// cascata, que precisam de um passo que elas próprias dizem.
fn place_with_step(
    sim: &mut SimWorld,
    r: &ph2d_ecs::scene::ComponentRegistry,
    echo: &mut MasterEcho,
    toasts: &mut ph2d_editor::ToastQueue,
    entity: Entity,
    step: [f32; 2],
) -> Option<Entity> {
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut out = None;
    super::drain(
        super::Verb::Place,
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
        &mut out,
    );
    out.map(Entity::from_bits)
}

fn pose(sim: &SimWorld, e: Entity) -> [f32; 2] {
    let t = sim.world().get::<Transform>(e).expect("pose").translation;
    [t.x, t.y]
}

/// ⭐⭐⭐ **NENHUMA CÓPIA ATERRA EM CIMA DE OUTRA** — a lei da cascata, medida no motor GERAL.
///
/// # ⛔⛔ Porque este gate nasce agora, e o que ele quase perdeu (F4.6c, 2026-09-07)
///
/// A cascata do dreno geral ([`super::cascade`]) declara, no próprio doc, *«a lei é a que o verbo
/// VETORIAL já tinha»* — e a **única régua dela vivia do outro lado**, em
/// `vec_component_edit_tests`, a exercitar o motor `VecInstance`. Apagar aquele motor sem esta
/// porta deixaria a lei **herdada por comentário e provada por ninguém**.
///
/// ⚠️ *Um censo de fatia tem de contar as RÉGUAS e não só as features* — esta linha já se queimou
/// duas vezes a contar verbos a menos, e a terceira teria sido contar gates a menos.
///
/// # O que ele afirma, e porque são DUAS metades
///
/// A **aritmética** (cada cópia a um múltiplo do degrau) e a **pilha** (duas cópias nunca
/// coincidem). A segunda diz o que o artista vê, e sobrevive a uma mudança de fórmula; a primeira
/// morde quando o degrau deixa de multiplicar. *Uma sozinha fica verde sobre metade do defeito.*
///
/// ⚠️ **O *Make* deixa a 1.ª cópia NO LUGAR da selecção**, e é por isso que a contagem dela é zero
/// — ver [`super::cascade`]. O gate parte daí, que é o estado que o gesto real produz.
#[test]
fn no_two_copies_of_a_recipe_ever_land_on_each_other() {
    let mut sim = SimWorld::new();
    let r = reg();
    let mut echo = MasterEcho::default();
    let mut toasts = ph2d_editor::ToastQueue::new();
    let rig = plain_rig(&mut sim);
    let (master, first) = make(&mut sim, &r, rig).expect("o gesto");

    let step = [3.0_f32, -3.0];
    let mut copias = vec![first];
    for _ in 0..3 {
        copias.push(
            place_with_step(&mut sim, &r, &mut echo, &mut toasts, master, step)
                .expect("o Instantiate recusou uma copia"),
        );
    }

    let base = pose(&sim, copias[0]);
    for (i, &c) in copias.iter().enumerate() {
        let p = pose(&sim, c);
        let quer = [base[0] + step[0] * i as f32, base[1] + step[1] * i as f32];
        assert!(
            (p[0] - quer[0]).abs() < 1e-4 && (p[1] - quer[1]).abs() < 1e-4,
            "a copia {i} nasceu em {p:?}, e o degrau manda {quer:?}"
        );
    }
    // A metade da PILHA, dita sem a aritmética acima: duas cópias nunca coincidem.
    for a in 0..copias.len() {
        for b in (a + 1)..copias.len() {
            let (pa, pb) = (pose(&sim, copias[a]), pose(&sim, copias[b]));
            assert!(
                (pa[0] - pb[0]).abs() > 1e-4 || (pa[1] - pb[1]).abs() > 1e-4,
                "as copias {a} e {b} pousaram no mesmo sitio ({pa:?})"
            );
        }
    }
}

/// ⭐⭐ **A CASCATA CONTA SÓ AS CÓPIAS DESTA RECEITA.**
///
/// ⚠️ Sem isto, colocar uma cópia de um botão empurraria a próxima cópia de um ícone — a folga
/// passaria a depender do que mais existe no documento, e o artista não teria como a prever.
///
/// ⚠️ **A fixtura precisa das DUAS receitas povoadas**: com uma só, *«conta as minhas»* e *«conta
/// todas»* devolvem o mesmo número e o gate ficaria verde sobre a contagem errada. É a mesma lei
/// de fixtura que o `a_shape_born_into_a_populated_scene_lands_at_the_end` paga do outro lado.
#[test]
fn the_step_of_one_recipe_does_not_count_the_copies_of_another() {
    let mut sim = SimWorld::new();
    let r = reg();
    let mut echo = MasterEcho::default();
    let mut toasts = ph2d_editor::ToastQueue::new();

    let rig_a = plain_rig(&mut sim);
    let (a_master, a_first) = make(&mut sim, &r, rig_a).expect("receita A");
    let rig_b = plain_rig(&mut sim);
    let (b_master, _) = make(&mut sim, &r, rig_b).expect("receita B");
    // A receita B leva DUAS cópias além da que o *Make* deixou.
    for _ in 0..2 {
        place_with_step(&mut sim, &r, &mut echo, &mut toasts, b_master, [1.0, -1.0])
            .expect("o Instantiate de B");
    }

    let step = [3.0_f32, -3.0];
    let base = pose(&sim, a_first);
    let a_second = place_with_step(&mut sim, &r, &mut echo, &mut toasts, a_master, step)
        .expect("o Instantiate de A");
    let p = pose(&sim, a_second);
    assert!(
        (p[0] - (base[0] + step[0])).abs() < 1e-4,
        "a 2a copia de A nasceu no degrau {:.2}, e nao no 1o — as copias de B entraram na conta",
        (p[0] - base[0]) / step[0]
    );
}
