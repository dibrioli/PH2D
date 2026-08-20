//! ⭐ **Todo painel do registro é alcançado pelo passeio de z-order.**
//!
//! # Por que este gate existe
//!
//! Registar um painel, ligar a feature e escrever a visibilidade **não chega**: o `paint_hero_screen`
//! percorre uma lista de `NodeId`s, e um painel cujo id não está lá é registado, visível e **nunca
//! pintado**. Nada quebra. Nada avisa. Os gates da própria crate do painel ficam todos verdes,
//! porque o que falta está noutra crate.
//!
//! `screens/hero/paint.rs` carrega **seis** notas a dizer isto — uma por vez que o defeito foi pago
//! (motion, timeline, physics, wet-tuning, tokens, authored, sculpt3d…). A sexta foi um smoke
//! reprovado do painel de modelagem 3D (Enio, 2026-08-19: *"o painel não abre"*), e é ela que
//! transformou a nota repetida neste teste.
//!
//! *Uma regra escrita seis vezes em comentário é uma regra que ninguém está a aplicar.*

use ph2d_editor_core::panel::with_registry_ref;
use ph2d_editor_core::screens::hero::PANEL_Z_ORDER_FALLBACK;

#[test]
fn every_registered_panel_is_reachable_by_the_z_order_walk() {
    // O registro que o app de facto monta. `register_all_panels` é idempotente.
    ph2d_panel_registry_init::register_all_panels();

    let missing: Vec<&'static str> = with_registry_ref(|reg| {
        reg.panels()
            .iter()
            .filter(|p| !PANEL_Z_ORDER_FALLBACK.contains(&p.manifest.panel_node_id))
            .map(|p| p.manifest.id)
            .collect()
    });

    assert!(
        missing.is_empty(),
        "painéis REGISTADOS que o passeio de z-order nunca alcança (registados, visíveis e \
         NUNCA pintados — nada quebra, nada avisa):\n  {}\n\n\
         fix: acrescente o `NodeId` do painel a `PANEL_Z_ORDER_FALLBACK` em \
         `screens/hero/paint.rs`.",
        missing.join("\n  ")
    );
}
