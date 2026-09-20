//! ⭐⭐⭐ **QUE ESQUELETO O `Bind` APANHA** — irmão do [`super::skin_live_tests`] pelo tecto de LOC, e
//! o corte é por RESPONSABILIDADE: ali mede-se o que uma pele FAZ ao desenho; aqui, **quem** ela
//! apanha quando o artista carrega no botão sem apontar.

use super::tests::{gira, osso, palco};
use super::*;
use crate::test_support::{pior_desvio_do_desenho, quadro};

/// ⚠️ **UM SEGUNDO ESQUELETO NÃO É APANHADO POR ENGANO.** Com um osso apontado, o Bind leva a árvore
/// DELE; sem nenhum, leva tudo (a leitura certa de *"há um esqueleto só"*, que é o caso comum).
#[test]
fn a_second_skeleton_is_only_bound_when_it_is_the_one_pointed_at() {
    let (mut sim, _scene, _map, _id, ossos) = palco();
    let outro = osso(&mut sim, "Other", [200.0, 0.0], 10.0, None);
    // ⚠️ **Comparado como CONJUNTO**: a ORDEM dentro de uma pele não tem sentido (os pesos
    // normalizam-se), só precisa de ser determinística — e ordenar por `to_bits` é ordenar por id
    // de ALOCAÇÃO, que numa fixtura não é o que se quer afirmar. O que importa é *quem* entra.
    let conjunto = |v: Vec<Entity>| {
        let mut s: Vec<u64> = v.into_iter().map(|e| e.to_bits()).collect();
        s.sort_unstable();
        s
    };
    assert_eq!(
        conjunto(skeleton_of(&sim, Some(ossos[1]))),
        conjunto(vec![ossos[0], ossos[1]])
    );
    assert_eq!(
        conjunto(skeleton_of(&sim, Some(outro))),
        conjunto(vec![outro])
    );
    assert_eq!(
        conjunto(skeleton_of(&sim, None)),
        conjunto(vec![ossos[0], ossos[1], outro]),
        "sem semente, o esqueleto e' a cena"
    );
}

/// ⭐⭐⭐ **UM ESQUELETO LONGE NÃO ENTRA NO DESENHO** — o suporte da lei é finito.
///
/// ⛔⛔ **A redacção anterior afirmava MAIS do que a lei dá, e a subdivisão do bind expô-lo**
/// (2026-09-19). Ela dizia: *«peso 0 ⇒ a normalização devolve exactamente os mesmos números»*, e
/// media isso comparando os dois DESENHOS a `1e-12`. Verdade sobre os pesos dos ossos longe;
/// **falsa** sobre os dos ossos perto, porque o domínio do solver do padrão-ouro é construído a
/// partir dos anéis **e dos ossos** ([`ph2d_vec_skin::pesos::pesos_do_caminho`] → `malha_do_dominio`):
/// um osso a `400` unidades muda a TRIANGULAÇÃO, e quantos mais pontos a forma tiver, mais ela
/// amostra essa diferença.
///
/// **Medido, com a subdivisão como contrafactual:**
///
/// | | peso dos ossos LONGE | diferença nos ossos PERTO |
/// |---|---|---|
/// | sem subdивisão (`4` nós, o caminho de antes) | `0,000e0` | **`0,000e0`** |
/// | com subdivisão (`20` nós) | `0,000e0` | **`1,289e-2`** |
///
/// ⇒ o que se afirma agora é a **LEI** — o osso longe tem peso exactamente zero — e não o proxy.
/// ⚠️ A 2.ª metade põe uma barra no desenho **derivada da medição**, para a diferença não poder
/// crescer sem alguém reparar.
#[test]
fn binding_to_the_whole_scene_draws_the_same_as_binding_to_the_right_skeleton() {
    let pesos_e_desenho = |semente: bool| {
        let (mut sim, mut scene, map, id, ossos) = palco();
        // Um segundo esqueleto, LONGE — o que o `Bind` sem semente também apanharia.
        let outro = osso(&mut sim, "Far", [400.0, 0.0], 30.0, None);
        osso(&mut sim, "Far2", [30.0, 0.0], 30.0, Some(outro));
        let raiz = semente.then_some(ossos[0]);
        assert_eq!(bind(&mut sim, &scene, &map, &[id], raiz), 1);
        let e = Entity::from_bits(*map.get(&id).expect("a forma tem entidade"));
        let pele = sim
            .world()
            .get::<ph2d_skeleton_ecs::SkinBind>(e)
            .expect("a pele")
            .clone();
        // Que tendão é qual — pelo NOME, nunca pela posição na lista.
        let longe: Vec<usize> = pele
            .tendons
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                sim.world()
                    .iter_entities()
                    .find(|er| er.get::<ph2d_ecs::StableId>().is_some_and(|s| *s == t.bone))
                    .and_then(|er| {
                        er.get::<ph2d_ecs::Name>()
                            .map(|n| n.as_str().starts_with("Far"))
                    })
                    .unwrap_or(false)
            })
            .map(|(i, _)| i)
            .collect();
        let g = crate::skinned_mesh::le(&pele.source).expect("a fonte");
        let k = pele.tendons.len();
        let maior_longe = g
            .pesos
            .chunks(k)
            .flat_map(|c| longe.iter().map(|i| c[*i].abs()))
            .fold(0.0_f64, f64::max);
        gira(&mut sim, ossos[1], 50.0);
        (maior_longe, longe.len(), quadro(&sim, &mut scene, id))
    };
    let (sem_longe, n_sem, com_semente) = pesos_e_desenho(true);
    let (com_longe, n_com, sem_semente) = pesos_e_desenho(false);

    // ⭐ O CONTROLO: a fixtura tem mesmo dois ossos longe quando se prende a cena inteira, e
    // nenhum quando se aponta. *Sem isto a asserção abaixo é sobre uma lista vazia.*
    assert_eq!(n_sem, 0, "a semente apanhou um osso «Far»");
    assert_eq!(n_com, 2, "prender a cena inteira nao apanhou os dois «Far»");
    assert!(sem_longe == 0.0, "{sem_longe}");
    assert!(
        com_longe == 0.0,
        "um osso a 400 unidades ficou com peso {com_longe} — o suporte deixou de ser FINITO, que e' \
         a lei que este gate existe para afirmar"
    );

    // A 2.ª metade: a diferença que sobra vem do domínio do solver, e tem tecto.
    let pior = pior_desvio_do_desenho(&com_semente, &sem_semente);
    assert!(
        pior < 0.1,
        "o esqueleto LONGE mudou o desenho em {pior} — medido em `0,064` quando esta barra foi \
         escrita, e a causa e' a malha do dominio, que inclui os ossos"
    );
}
