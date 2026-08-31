//! O espelho da ferramenta activa serve **toda** ferramenta, e não só o Painter.

use super::intern_active_tool;

fn boot_registry() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        let mut reg = ph2d_tool_registry::Registry::default();
        ph2d_tool_registry_init::register_all(&mut reg);
        reg.build().expect("o registry do boot constrói");
        ph2d_editor::install_registry(reg);
    });
}

/// ⭐ **Toda ferramenta do registry se espelha** — a mutação que sangra é voltar a um `match` de um
/// literal, que é como o campo nasceu.
#[test]
fn every_registered_tool_mirrors_not_only_the_painter() {
    boot_registry();
    let reg = ph2d_editor::installed_registry().expect("instalado");
    let mut blind = Vec::new();
    for m in reg.manifests() {
        if intern_active_tool(Some(m.id)) != Some(m.id) {
            blind.push(m.id);
        }
    }
    assert!(
        blind.is_empty(),
        "ferramentas que o espelho não vê — o chrome delas lê sempre «não está activa»: {blind:?}"
    );
    // ⭐ Os três que motivaram a mudança, pelo nome: os toggles deles leem este campo para
    // escolher entre activar e CANCELAR.
    for id in ["vector", "motion", "flip"] {
        assert_eq!(
            intern_active_tool(Some(id)),
            Some(id),
            "{id}: sem espelho, o segundo clique no menu reactiva em vez de desligar"
        );
    }
}

/// **E os três `None`** — o controlo negativo, que é o que impede o espelho de devolver qualquer
/// coisa.
#[test]
fn nothing_active_and_nothing_known_mirror_to_none() {
    boot_registry();
    assert_eq!(intern_active_tool(None), None);
    assert_eq!(intern_active_tool(Some("a_tool_that_does_not_exist")), None);
    assert_eq!(intern_active_tool(Some("")), None);
}
