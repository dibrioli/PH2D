//! **AS STRINGS DAS SECÇÕES FACTORY e LIFECYCLE** (TOP-20 #11 e #12, W3) — o irmão de tabela do
//! [`super`].
//!
//! ⚠️ **Um módulo por ASSUNTO, como o [`super::tags`]**, e pela mesma razão de isolamento
//! (`CLAUDE.md` §0.2): enquanto todas as chaves moram num `match` só, duas linhas paralelas que
//! acrescentem uma chave cada colidem no mesmo punhado de linhas.
//!
//! ⛔ **Os AVISOS não estão aqui**, e a ausência é a decisão — eles são frases que dependem do
//! ESTADO (*«no recipe»*, *«the clock is stopped»*) e vivem no pintor da secção, ao lado da
//! condição que os produz. Uma tabela só de chaves espalharia a condição e o texto por dois
//! ficheiros, que é o defeito que a `TagError::message` evita do outro lado.

/// A tradução de uma chave `panel.factory.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ⚠️ **A receita viaja pelo NOME** — é isso que o marcador de posição tem de dizer, senão o
        // artista tenta escrever um número.
        "panel.factory.recipe" => "recipe name\u{2026}",
        "panel.factory.on_signal" => "on signal\u{2026}",
        "panel.factory.tag" => "spawn point tag\u{2026}",
        "panel.factory.on_spawned" => "on spawned\u{2026}",
        "panel.factory.on_exhausted" => "on exhausted\u{2026}",
        "panel.factory.on_death" => "on death\u{2026}",
        // ph2d-migrar-texto:begin
        "panel.factory.here" => "Here",
        "panel.factory.area" => "Area",
        "panel.factory.at_tag" => "At Tag",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
