//! O roteador de smoke do editor de scripts (`PH2D_CODE_SMOKE`), inerte sem a variável.
//!
//! A CENA mora na `ph2d-app-components` (`code_smoke`: precisa da cena vectorial); a shell lê o
//! nível por [`armed_level`] e compõe-a. Níveis 2 e 3 abrem a aba pelo pedido do «Edit script», que
//! o [`crate::frame`] drena como qualquer outro; o 3 tem a porta das fotos ([`crate::smoke_drive`]).

use std::sync::OnceLock;

/// O maior nível — o mesmo `ph2d_app_components::code_smoke::CENAS` (as cenas contam-se lá).
pub const CENAS: u32 = 3;

pub const ROUTERS: &[ph2d_app_host::SmokeRouter] = &[ph2d_app_host::SmokeRouter {
    env: "PH2D_CODE_SMOKE",
    max_level: CENAS,
}];

/// O nível pedido, lido por `get` (a porta injectável: os testes não mexem no ambiente).
#[must_use]
pub fn armed_level_in(get: impl Fn(&str) -> Option<String>) -> Option<u32> {
    get(ROUTERS[0].env)?
        .trim()
        .parse()
        .ok()
        .filter(|n| (1..=CENAS).contains(n))
}

/// O nível do processo (lido uma vez).
#[must_use]
pub fn armed_level() -> Option<u32> {
    static LEVEL: OnceLock<Option<u32>> = OnceLock::new();
    *LEVEL.get_or_init(|| armed_level_in(|k| std::env::var(k).ok()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_nivel_le_se_pela_porta_e_fora_da_faixa_e_inerte() {
        let com = |v: &'static str| move |k: &str| (k == "PH2D_CODE_SMOKE").then(|| v.to_owned());
        assert_eq!(armed_level_in(com("1")), Some(1));
        assert_eq!(armed_level_in(com("2")), Some(2));
        assert_eq!(armed_level_in(com("3")), Some(3));
        assert_eq!(armed_level_in(com("4")), None);
        assert_eq!(armed_level_in(com("x")), None);
        assert_eq!(armed_level_in(|_| None), None);
    }
}
