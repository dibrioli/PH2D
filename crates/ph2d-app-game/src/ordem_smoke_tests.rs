//! A cena da ordem: a env, e o `=2` que inverte a Hierarchy por baixo das raízes que já havia.

use super::*;

#[test]
fn a_cena_da_ordem_le_se_pela_env() {
    let com = |v: &'static str| move |k: &str| (k == "PH2D_ORDEM_SMOKE").then(|| v.to_owned());
    assert_eq!(nivel_in(com("1")), Some(1));
    assert_eq!(nivel_in(com("2")), Some(2));
    assert_eq!(nivel_in(com("3")), None);
    assert_eq!(nivel_in(|_| None), None);
}

/// ⭐ **O `=2` é a mesma tabela INVERTIDA**, e as peças ficam DEPOIS das raízes que já havia (que não
/// mudam de ordem entre si). (Mutação: `lugar` sem o `nivel == 2` ⇒ RED.)
#[test]
fn o_nivel_2_inverte_a_hierarchy_por_baixo_das_raizes_de_antes() {
    for nivel in [1, 2] {
        let mut sim = SimWorld::new();
        let w = sim.world_mut();
        let antiga = w.spawn((Transform::IDENTITY, RootOrder(5))).id();
        let pecas: Vec<(Peca, u64)> = ORDEM
            .iter()
            .map(|&p| {
                (
                    p,
                    w.spawn((Transform::IDENTITY, RootOrder(0))).id().to_bits(),
                )
            })
            .collect();
        ordena(w, nivel, &pecas);
        let ordem = |b: u64| w.get::<RootOrder>(Entity::from_bits(b)).map(|r| r.0);
        assert_eq!(ordem(antiga.to_bits()), Some(5), "a raiz de antes não mexe");
        let mut lida: Vec<(u32, Peca)> = pecas
            .iter()
            .map(|&(p, b)| (ordem(b).expect("ordenada"), p))
            .collect();
        lida.sort_by_key(|(o, _)| *o);
        assert!(lida.iter().all(|(o, _)| *o > 5), "as peças vêm depois");
        let mut esperada = ORDEM.to_vec();
        if nivel == 2 {
            esperada.reverse();
        }
        assert_eq!(
            lida.into_iter().map(|(_, p)| p).collect::<Vec<_>>(),
            esperada
        );
    }
}
