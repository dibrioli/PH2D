//! ⛔⛔ **A CÓPIA que a fábrica faria de um molde, pelo nome** — só para os gates das cenas.
//!
//! ⚠️ Partilhada pelas duas cenas da vida (`=1` e `=2`): escrita duas vezes, uma delas deixaria de
//! passar pela porta de cópia do produto no dia em que a outra mudasse.

use ph2d_ecs::MasterRoot;
use ph2d_ecs::{Name, SimWorld};

/// ⛔⛔ **A CÓPIA que a fábrica faria de um molde, pelo nome** — pela porta de cópia do PRODUTO
/// ([`crate::instantiate::instantiate_master_many`]) e com o registo que o produto monta.
///
/// ⚠️ **Nasceu do smoke do dono** (*«ninguém sumiu ao levar muitos tiros»*): a 1.ª redacção destes
/// gates lia a RECEITA do mundo e montava o alvo à mão — e a cópia profunda leva só os componentes
/// REGISTADOS, que a `Health`/`Damage` não eram. *Uma fixtura que monta à mão o que o produto COPIA
/// mede outro programa.* ⇒ os números saem da CÓPIA, e uma cópia sem vida faz o gate reprovar alto.
pub(crate) fn copia(sim: &mut SimWorld, nome: &str) -> ph2d_ecs::Entity {
    let molde = {
        let w = sim.world_mut();
        let mut q =
            w.query_filtered::<(ph2d_ecs::Entity, &Name), bevy_ecs::query::With<MasterRoot>>();
        q.iter(w)
            .find(|(_, n)| n.as_str() == nome)
            .map(|(e, _)| e)
            .unwrap_or_else(|| panic!("o molde «{nome}» nao esta' na cena"))
    };
    let registo = crate::component_registry_for_tests::registo();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut docs = crate::instance_docs::OwnedDocs {
        vec_scene: &mut sc,
        vec_entities: &mut mp,
    };
    crate::instantiate::instantiate_master_many(
        sim,
        &registo,
        molde,
        None,
        &mut docs,
        crate::instantiate::ArtLink::Own,
        1,
    )
    .unwrap_or_else(|e| panic!("«{nome}» recusou a cópia: {e:?}"))[0]
}

/// ⭐⭐ **O voo da bala de uma cena da vida** — o que a foto do dono pergunta, pelas portas do PRODUTO:
/// as fábricas (`tick_factories` + `apply_births`), a ponte da física e a porta de desenho
/// (`draws_this_frame`).
///
/// ⚠️ **Os números contam-se ATÉ ao golpe**: sem o dreno das mortes (que é da shell) a bala gasta
/// fica na cena, e contá-la depois mediria o arnês e não o voo.
#[derive(Debug)]
pub(crate) struct Voo {
    /// O `x` mais à esquerda e mais à direita por onde a bala passou antes do golpe.
    pub(crate) x: [f32; 2],
    /// O `y` mais baixo e mais alto por onde ela passou antes do golpe.
    pub(crate) y: [f32; 2],
    /// Quantos tiques ela DESENHOU antes do golpe.
    pub(crate) vista: usize,
    /// Se ela feriu a cópia do alvo.
    pub(crate) acertou: bool,
}

/// Arranca a cena com `comecar` (as fábricas dos alvos), atira UMA vez pela fábrica do herói e
/// segue a bala `120` tiques. `alvo` é o nome do MOLDE do alvo que a bala deve ferir.
pub(crate) fn voo_da_bala(sim: &mut SimWorld, comecar: &str, alvo: &str) -> Voo {
    use bevy_ecs::query::With;
    use ph2d_ecs::{Entity, Spawned, Transform};
    use ph2d_entity_visibility::off_canvas::draws_this_frame;
    use ph2d_physics_ecs::{Health, HealthNow, PhysicsBridge};
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    let tags = ph2d_tags::TagTree::default();
    let registo = crate::component_registry_for_tests::registo();
    let (mut sc, mut mp) = crate::instance_docs::empty_docs();
    let mut ponte = PhysicsBridge::new();
    let mut t = 0u64;
    let mut nascer = |sim: &mut SimWorld, sinal: &str, t: u64| {
        let tick = ph2d_ecs::tick_factories(sim.world_mut(), &tags, &[sinal]);
        let mut docs = crate::instance_docs::OwnedDocs {
            vec_scene: &mut sc,
            vec_entities: &mut mp,
        };
        crate::factory_bridge::apply_births(sim, &registo, &mut docs, &tick.births, t).nasceram
    };
    assert!(nascer(sim, comecar, t) > 0, "os alvos não nasceram");
    for _ in 0..3 {
        ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
        ponte.dispatch(sim, true, t);
        t += 1;
    }
    assert_eq!(
        nascer(sim, crate::vida_smoke::SINAL, t),
        1,
        "o herói não atirou"
    );
    let (copia, vida) = {
        let w = sim.world_mut();
        let mut q = w.query_filtered::<(Entity, &Name, &Health), With<Spawned>>();
        q.iter(w)
            .find(|(_, n, _)| n.as_str().starts_with(alvo))
            .map(|(e, _, h)| (e, f64::from(h.max)))
            .unwrap_or_else(|| panic!("a cópia de «{alvo}» não nasceu"))
    };
    let mut voo = Voo {
        x: [f32::INFINITY, f32::NEG_INFINITY],
        y: [f32::INFINITY, f32::NEG_INFINITY],
        vista: 0,
        acertou: false,
    };
    for _ in 0..120 {
        ph2d_ecs::assign_master_pieces(sim.world_mut());
        ponte.dispatch(sim, true, t);
        t += 1;
        if voo.acertou {
            continue;
        }
        let w = sim.world_mut();
        let mut q = w.query_filtered::<(Entity, &Name, &Transform), With<Spawned>>();
        let balas: Vec<_> = q
            .iter(w)
            .filter(|(_, n, _)| n.as_str().starts_with("Bala"))
            .map(|(e, _, tr)| (e, tr.translation))
            .collect();
        for (e, p) in balas {
            voo.x = [voo.x[0].min(p.x), voo.x[1].max(p.x)];
            voo.y = [voo.y[0].min(p.y), voo.y[1].max(p.y)];
            if draws_this_frame(sim.world(), e, u32::MAX, [p.x, p.y]) {
                voo.vista += 1;
            }
        }
        voo.acertou = sim
            .world()
            .get::<HealthNow>(copia)
            .is_some_and(|h| h.pontos < vida);
    }
    voo
}
