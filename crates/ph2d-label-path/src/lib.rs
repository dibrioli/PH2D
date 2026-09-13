//! **`ph2d-label-path` — a álgebra de um caminho de rótulos hierárquicos** (`Personagens/Heróis`).
//!
//! # ⭐ Porque é uma folha
//!
//! Duas árvores desta casa guardam a **hierarquia num CAMINHO** e a **identidade num id** — o modelo do
//! *Asset Browser* do Blender: os catálogos da biblioteca (`ph2d_asset_index::CatalogTree`) e as tags
//! (`ph2d-tags`, `docs/Components/08_plano_tags.md`). As operações sobre o caminho são as mesmas nas
//! duas, e a mais traiçoeira — *um prefixo de TEXTO não é um prefixo de CAMINHO* (`Hero` /
//! `Heroine`) — já tinha sido paga uma vez dentro do `CatalogTree`. ⇒ **uma lei, duas árvores**; uma
//! segunda cópia seria o sítio onde o `Heroine` volta.
//!
//! # ⚠️ A comparação é um PARÂMETRO
//!
//! Os catálogos comparam o texto EXACTO, como sempre compararam; as tags comparam DOBRADO (decisão do
//! dono D2, `ph2d_label_fold::fold`). ⇒ cada porta que compara segmentos recebe a **chave**, e esta
//! folha não escolhe política de idioma nenhuma.

/// O separador de níveis. ⚠️ É o FORMATO, não apresentação: mudá-lo reinterpreta todo caminho gravado.
pub const SEP: char = '/';

/// Um caminho sem espaços nas pontas de cada nível e sem níveis vazios.
///
/// ⚠️ Sem isto `"A//B"` e `"A/ B"` criariam níveis que a UI desenha como linhas em branco, e `"A/"`
/// um filho sem nome que ninguém escolhe nem apaga.
#[must_use]
pub fn normalise(path: &str) -> String {
    path.split(SEP)
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(&SEP.to_string())
}

/// O último nível — o rótulo que a linha mostra.
#[must_use]
pub fn label(path: &str) -> &str {
    path.rsplit(SEP).next().unwrap_or(path)
}

/// A profundidade (`0` = raiz).
#[must_use]
pub fn depth(path: &str) -> usize {
    path.matches(SEP).count()
}

/// O caminho do pai, ou `None` numa raiz.
#[must_use]
pub fn parent(path: &str) -> Option<&str> {
    path.rsplit_once(SEP).map(|(p, _)| p)
}

/// ⭐⭐ **`path` é `ancestor` ou está debaixo dele?** — nível a nível, pela `key`.
///
/// ⛔ Nunca `starts_with` sobre o texto: `Heroine` começa por `Hero` e não é filho dele.
#[must_use]
pub fn is_self_or_descendant(path: &str, ancestor: &str, key: impl Fn(&str) -> String) -> bool {
    let mut niveis = path.split(SEP);
    ancestor
        .split(SEP)
        .all(|a| niveis.next().is_some_and(|p| key(p) == key(a)))
}

/// ⭐⭐ **O caminho de `path` depois de `old` passar a chamar-se `new`** — ou `None` quando `path`
/// não é `old` nem descendente dele. É o que leva os filhos junto num renomear ou num mover.
///
/// ⚠️ Os níveis ABAIXO de `old` guardam a grafia de quem os escreveu: a chave decide *se* casa, nunca
/// *como* se escreve.
#[must_use]
pub fn rebase(path: &str, old: &str, new: &str, key: impl Fn(&str) -> String) -> Option<String> {
    if !is_self_or_descendant(path, old, &key) {
        return None;
    }
    Some(replace_prefix(path, old.split(SEP).count(), new))
}

/// ⭐ **`path` com os primeiros `levels` níveis trocados por `new`** — a metade de [`rebase`] que não
/// pergunta, para quem JÁ sabe que `path` descende (pela chave que guarda).
#[must_use]
pub fn replace_prefix(path: &str, levels: usize, new: &str) -> String {
    let resto: Vec<&str> = path.split(SEP).skip(levels).collect();
    if resto.is_empty() {
        new.to_string()
    } else {
        format!("{new}{SEP}{}", resto.join(&SEP.to_string()))
    }
}

/// ⭐⭐ **A mesma pergunta do [`is_self_or_descendant`], sobre chaves JÁ calculadas** por
/// [`tree_order_key`] — para quem as guarda (a árvore de tags) e não quer pagar a `key` a cada
/// comparação.
///
/// ⚠️ **Não é uma segunda lei, é a MESMA**: os níveis do antepassado são o começo dos níveis do
/// caminho. Há gate a pôr as duas formas lado a lado (`the_keyed_and_the_textual_descendant_checks_agree`).
#[must_use]
pub fn key_is_self_or_descendant(key: &[String], ancestor: &[String]) -> bool {
    key.starts_with(ancestor)
}

/// ⭐⭐⭐ **A chave de ORDEM de uma árvore** — a sequência de níveis, cada um pela `key`.
///
/// ⛔ Ordenar pela string crua parte a árvore: `'-'` (0x2D) é menor que `'/'` (0x2F), então `"A-x"`
/// cairia entre `"A"` e `"A/B"`. Nível a nível, um pai é prefixo do filho e vem sempre antes.
#[must_use]
pub fn tree_order_key(path: &str, key: impl Fn(&str) -> String) -> Vec<String> {
    path.split(SEP).map(key).collect()
}

#[cfg(test)]
#[path = "path_tests.rs"]
mod tests;
