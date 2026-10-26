use super::*;

/// Os oito botões e o número existem no store depois do `populate` (registar é o que os torna clicáveis).
#[test]
fn every_rules_widget_is_registered() {
    let mut store = WidgetStore::with_capacity(16);
    populate(&mut store);
    for id in VECTOR_CAD_RULE
        .into_iter()
        .chain([VECTOR_CAD_DELETE, VECTOR_CAD_VALUE])
    {
        assert!(
            store.get(id).is_some(),
            "um widget da tool Regras ficou por registar"
        );
    }
    for p in Possivel::ALL {
        assert_eq!(Possivel::de_id(p.id()), Some(p));
    }
}
