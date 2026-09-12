//! **QUAL MOTOR desenha o traço do Flip** — o percurso (omissão) ou o rasterizador (a escape).
//!
//! Morava no passe da `ph2d-app-flip` e desceu para a crate do renderizador na auditoria de
//! arquitectura de 2026-09-12 (A1): a assadura do Motion faz a mesma pergunta e, para a fazer,
//! dependia da família Flip inteira.

/// **O PERCURSO É O DEFAULT** (doc 12 §22) — `PH2D_FLIP_NEW_ENGINE=0` é a ESCAPE para o
/// rasterizador que shipava.
///
/// A inversão é a decisão do padrão-ouro, e o que a sustenta é a hierarquia das leis, não uma
/// preferência: a lei do percurso (`τ = ∫ f(dn) ds`, `α = 1 − exp(−τ)`) é o **limite contínuo** que
/// os dab buffers de GIMP/Krita/Procreate — e o do nosso próprio Painter — aproximam por soma
/// finita, e o rasterizador (união global + eleição por depth) não está na família. Medido contra o
/// depósito do Painter, o pico na ponta: raster **+129/+131/+175** contra percurso
/// **−12/−17/−46** (durezas 0,2/0,4/0,7).
pub fn new_engine_armed() -> bool {
    static ARMED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ARMED.get_or_init(|| walk_from_env(std::env::var("PH2D_FLIP_NEW_ENGINE").ok().as_deref()))
}

/// A política do interruptor, **PURA** — quem decide *qual motor* a partir do que o ambiente diz.
///
/// ⚠️ **Ela existe separada porque o default não era testável:** o [`new_engine_armed`] é um
/// `OnceLock` sobre uma variável de processo, então nenhum teste consegue exercitá-lo duas vezes no
/// mesmo binário, e um default que ninguém pode afirmar é um default que a próxima edição inverte
/// em silêncio.
///
/// ⚠️ **Só o desligamento EXPLÍCITO volta ao raster** — ausente, vazio ou irreconhecível dá o
/// percurso. Isso é deliberado: um erro de digitação na escape (`=flase`) falha **para o default**,
/// nunca para um terceiro comportamento.
fn walk_from_env(v: Option<&str>) -> bool {
    !matches!(v, Some("0" | "false" | "off"))
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
