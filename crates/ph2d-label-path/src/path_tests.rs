//! Os gates da álgebra de caminhos — escritos contra um *stub* (`todo!()`) e vistos vermelhos.

use super::*;

fn exacta(s: &str) -> String {
    s.to_string()
}

fn minusculas(s: &str) -> String {
    s.to_lowercase()
}

/// ⛔⛔ **Um prefixo de TEXTO não é um prefixo de CAMINHO** — o defeito que o `CatalogTree` já pagou.
///
/// **Mutação que deve sangrar:** comparar com `starts_with` sem a fronteira do separador.
#[test]
fn a_text_prefix_is_not_a_path_prefix() {
    assert!(is_self_or_descendant("Hero", "Hero", exacta));
    assert!(is_self_or_descendant("Hero/Sword", "Hero", exacta));
    assert!(!is_self_or_descendant("Heroine", "Hero", exacta));
    assert!(!is_self_or_descendant("Heroine/Sword", "Hero", exacta));
    assert!(
        !is_self_or_descendant("Hero", "Hero/Sword", exacta),
        "o pai nao e' filho do filho"
    );
}

/// ⭐ **A comparação é da CHAVE, não do texto** — é isto que deixa as tags dobrar e os catálogos não.
#[test]
fn the_descendant_check_compares_through_the_key() {
    assert!(!is_self_or_descendant(
        "personagens/Heróis",
        "Personagens",
        exacta
    ));
    assert!(is_self_or_descendant(
        "personagens/Heróis",
        "Personagens",
        minusculas
    ));
}

/// ⭐⭐ **Renomear leva os filhos, e nunca o vizinho.**
///
/// **Mutação que deve sangrar:** `rebase` a aceitar `Heroine` como descendente de `Hero`.
#[test]
fn rebasing_carries_the_descendants_and_never_the_neighbour() {
    assert_eq!(
        rebase("Hero", "Hero", "Villain", exacta),
        Some("Villain".into())
    );
    assert_eq!(
        rebase("Hero/Sword/Blade", "Hero", "Villain", exacta),
        Some("Villain/Sword/Blade".into())
    );
    assert_eq!(rebase("Heroine", "Hero", "Villain", exacta), None);
    // Mover para dentro de outro é o mesmo gesto com um prefixo mais fundo.
    assert_eq!(
        rebase("Hero/Sword", "Hero", "Cast/Hero", exacta),
        Some("Cast/Hero/Sword".into())
    );
    // ⚠️ Pela chave: a grafia do resto do caminho FICA a de quem a escreveu.
    assert_eq!(
        rebase("hero/Sword", "Hero", "Villain", minusculas),
        Some("Villain/Sword".into())
    );
}

#[test]
fn normalising_drops_empty_levels_and_trims_each_one() {
    assert_eq!(normalise(" A // B /"), "A/B");
    assert_eq!(normalise("A/ B"), "A/B");
    assert_eq!(normalise("///"), "");
}

#[test]
fn label_depth_and_parent_read_the_levels() {
    assert_eq!(label("A/B/C"), "C");
    assert_eq!(label("A"), "A");
    assert_eq!(depth("A"), 0);
    assert_eq!(depth("A/B/C"), 2);
    assert_eq!(parent("A/B/C"), Some("A/B"));
    assert_eq!(parent("A"), None);
}

/// ⭐⭐⭐ **Um pai vem SEMPRE imediatamente antes dos filhos** — o gate que o `CatalogTree` já tinha,
/// agora sobre a porta partilhada.
///
/// **Mutação que deve sangrar:** ordenar pela string crua (`"A-x"` cai entre `"A"` e `"A/B"`).
#[test]
fn the_tree_order_puts_a_parent_immediately_before_its_children() {
    let mut caminhos = vec!["A-x", "A/B", "A", "a/C"];
    caminhos.sort_by_cached_key(|p| tree_order_key(p, minusculas));
    assert_eq!(caminhos, vec!["A", "A/B", "a/C", "A-x"]);
}

/// ⚠️ **As duas formas da pergunta são UMA lei** — a de texto (com a `key`) e a de chaves guardadas.
///
/// **Mutação que deve sangrar:** a forma de chaves a comparar só o último nível, ou a de texto a
/// perder a fronteira de nível.
#[test]
fn the_keyed_and_the_textual_descendant_checks_agree() {
    let casos = [
        ("Hero", "Hero"),
        ("Hero/Sword", "Hero"),
        ("Heroine", "Hero"),
        ("Heroine/Sword", "Hero"),
        ("Hero", "Hero/Sword"),
        ("personagens/Heróis", "Personagens"),
        ("A/B/C", "A/B"),
        ("A/BC", "A/B"),
        ("X/A/B", "A/B"),
    ];
    let chaves: [fn(&str) -> String; 2] = [exacta, minusculas];
    let mut verdadeiros = 0;
    for (p, a) in casos {
        for key in chaves {
            let textual = is_self_or_descendant(p, a, key);
            assert_eq!(
                key_is_self_or_descendant(&tree_order_key(p, key), &tree_order_key(a, key)),
                textual,
                "{p:?} dentro de {a:?}"
            );
            verdadeiros += usize::from(textual);
        }
    }
    // ⚠️ Controlo: os casos cobrem as DUAS respostas (senão duas funções constantes concordariam).
    assert!(verdadeiros > 0 && verdadeiros < casos.len() * chaves.len());
}

#[test]
fn replacing_a_prefix_keeps_the_levels_below() {
    assert_eq!(
        replace_prefix("Hero/Sword/Blade", 1, "Cast/Hero"),
        "Cast/Hero/Sword/Blade"
    );
    assert_eq!(replace_prefix("Hero", 1, "Villain"), "Villain");
    assert_eq!(replace_prefix("A/B/C", 2, "Z"), "Z/C");
}
