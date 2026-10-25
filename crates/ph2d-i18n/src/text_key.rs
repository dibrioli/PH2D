//! A [`TextKey`] — filho do `lib.rs` pelo tecto de LOC.

/// ⭐⭐ **Uma CHAVE guardada numa tabela `const`** — o rótulo de um segmentado, a linha de um card.
///
/// O [`crate::tr`] não é `const fn`, logo uma tabela `const [&str; N]` de rótulos não o pode chamar, e a
/// tabela passa a guardar CHAVES. ⛔ Guardadas como `&str`, uma chave e um texto são o MESMO tipo: o
/// consumidor que se esquece de traduzir compila, passa em todo teste que não leia o pixel, e pinta
/// `panel.inspector.joint.pin` no ecrã. Com `TextKey` esse esquecimento é **erro de compilação** — o
/// `seg_row(…, &[&str])` não aceita `[TextKey; N]` até alguém escrever `.map(TextKey::tr)`.
///
/// ⚠️ A tabela continua `const` de propósito: há contagens derivadas dela em tempo de compilação
/// (`PLAYER_ROW_COUNT` do Inspector), que uma função no lugar da tabela partiria.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextKey(&'static str);

impl TextKey {
    /// A chave, tal como está na tabela (`"panel.inspector.joint.pin"`).
    #[must_use]
    pub const fn new(key: &'static str) -> Self {
        Self(key)
    }

    /// O texto que a chave diz — o mesmo [`crate::tr`], com a chave desconhecida a pintar-se crua.
    #[must_use]
    pub fn tr(self) -> &'static str {
        crate::tr(self.0)
    }

    /// ⭐ **A chave CRUA, para quem faz censo dela** — nunca para pintar.
    ///
    /// ⚠️ Ela existe porque um gate precisa de perguntar *«esta chave EXISTE na tabela?»*, e a
    /// resposta é `tr(k) != k` (a chave desconhecida volta crua, com `leak_key`). ⛔ O tipo continua
    /// a ser a cerca: quem quer TEXTO chama [`Self::tr`], e o nome deste método diz em voz alta que
    /// o que sai daqui é um identificador.
    #[must_use]
    pub const fn key(self) -> &'static str {
        self.0
    }
}
