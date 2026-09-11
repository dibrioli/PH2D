//! ⭐ **O painel está no REGISTRO do app** — o gate que existe por causa de um smoke do Enio
//! (*"o painel não abre"*, 2026-08-19).
//!
//! ⚠️ Este é o modo de falha que o `Cargo.toml` do shell já regista como pago pelo painel de
//! física: a crate existe, compila, e **todos** os gates dela passam, enquanto o painel não está no
//! registro — e a visibilidade alterna algo que ninguém pinta. Nenhum teste dentro da crate pode
//! ver isso, porque o que falta está numa cadeia de *features* fora dela.
//!
//! ⚠️ E ele mede a cadeia **completa**, incluindo o `default` do shell: o shell põe
//! `default-features = false` na `ph2d-panel-registry-init`, então o `default` daquela crate não
//! alcança ninguém. Um painel só está no app se estiver nos DOIS sítios.

#[test]
fn the_panel_is_in_the_registry_the_app_builds() {
    let reg = ph2d_panel_registry_init::build_typed_registry();
    let ids: Vec<&str> = reg.panels().iter().map(|p| p.manifest.id).collect();
    assert!(
        ids.contains(&ph2d_panel_model3d::PANEL_ID),
        "o painel de modelagem não está no registro. Painéis presentes: {ids:?}\n\
         fix: a feature `panel-model3d` tem de estar (a) no `default` do shell E (b) a encaminhar \
         para `ph2d-panel-registry-init/panel-model3d`"
    );
}
