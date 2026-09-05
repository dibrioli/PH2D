//! Theme resolution from environment variable `PH2D_THEME`.
//!
//! Extracted from [`main`] (Track B1). Unknown values fall back to the **default of the
//! look** and emit an eprintln so misspellings surface in CI / dev logs.
//!
//! ⚠️ **O default segue a APARÊNCIA, e é isto que o smoke de 2026-09-04 apanhou:** o
//! `HeroScreen::new` já abria no `dark` (o *Default* do Godot) sob o redesenho, e esta função
//! — chamada logo a seguir por `init.rs` — devolvia `Forge` e escrevia por cima. *Duas portas
//! para o tema de arranque, e a que corria por último era a que ninguém tinha mudado.* Agora há
//! uma lei ([`Theme::default_for`]) e as duas portas lêem-na.

use ph2d_tokens::{Theme, UiLook};

/// O tema pedido por `PH2D_THEME`, ou o default da aparência.
///
/// Os nomes válidos são os [`Theme::id`] das DUAS famílias — um `PH2D_THEME=dark` sob o
/// clássico é honrado (é uma escolha explícita), tal como um `=forge` sob o redesenho.
pub fn resolve_theme(name: Option<&str>, look: UiLook) -> Theme {
    let fallback = Theme::default_for(look);
    match name {
        None => fallback,
        Some(id) => Theme::from_id(id).unwrap_or_else(|| {
            let valid: Vec<&str> = Theme::ALL.iter().map(|t| t.id()).collect();
            eprintln!(
                "[ph2d] PH2D_THEME={id:?} not recognized; falling back to {}. Valid: {}.",
                fallback.id(),
                valid.join(", ")
            );
            fallback
        }),
    }
}

pub fn parse_theme_env() -> Theme {
    resolve_theme(
        std::env::var("PH2D_THEME").ok().as_deref(),
        ph2d_editor::screens::hero::ui_look_from_env(),
    )
}
