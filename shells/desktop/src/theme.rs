//! Theme resolution from environment variable `PH2D_THEME`.
//!
//! Extracted from [`main`] (Track B1). Falls back to `Theme::Forge`
//! for unknown values and emits an eprintln so misspellings surface
//! in CI / dev logs.

use ph2d_tokens::Theme;

pub fn resolve_theme(name: Option<&str>) -> Theme {
    match name {
        None => Theme::Forge,
        Some("forge") => Theme::Forge,
        Some("workshop") => Theme::Workshop,
        Some("sunstone") => Theme::Sunstone,
        Some("blueprint") => Theme::Blueprint,
        Some(other) => {
            eprintln!(
                "[ph2d] PH2D_THEME={other:?} not recognized; falling back to forge. Valid: forge, workshop, sunstone, blueprint."
            );
            Theme::Forge
        }
    }
}

pub fn parse_theme_env() -> Theme {
    resolve_theme(std::env::var("PH2D_THEME").ok().as_deref())
}
