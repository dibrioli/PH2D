use super::FracKey;

/// Gerador determinista (LCG) — sem depender de `rand` num crate puro.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self, n: usize) -> usize {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        usize::try_from(self.0 >> 33).unwrap() % n
    }
}

fn key(s: &str) -> FracKey {
    // Reconstrói por serde, a única porta pública para uma chave arbitrária.
    postcard::from_bytes(&postcard::to_allocvec(&s.to_string()).unwrap()).unwrap()
}

fn btw(a: Option<&str>, b: Option<&str>) -> String {
    let (a, b) = (a.map(key), b.map(key));
    FracKey::between(a.as_ref(), b.as_ref())
        .as_str()
        .to_string()
}

/// ⭐ Os vectores publicados com o algoritmo (`fractional-indexing`, rocicorp) — o oráculo do passo.
#[test]
fn the_published_vectors_hold() {
    let cases: &[(Option<&str>, Option<&str>, &str)] = &[
        (None, None, "a0"),
        (None, Some("a0"), "Zz"),
        (Some("a0"), None, "a1"),
        (Some("a0"), Some("a1"), "a0V"),
        (Some("a1"), Some("a2"), "a1V"),
        (Some("a0V"), Some("a1"), "a0l"),
        (Some("Zz"), Some("a0"), "ZzV"),
        (Some("Zz"), Some("a1"), "a0"),
        (None, Some("Y00"), "Xzzz"),
        (Some("bzz"), None, "c000"),
        (Some("a0"), Some("a0V"), "a0G"),
        (Some("a0"), Some("a0G"), "a08"),
        (Some("b125"), Some("b129"), "b127"),
        (Some("a0"), Some("a1V"), "a1"),
        (Some("Zz"), Some("a01"), "a0"),
        (None, Some("a0V"), "a0"),
        (None, Some("b999"), "b99"),
    ];
    for (a, b, want) in cases {
        assert_eq!(btw(*a, *b), *want, "between({a:?}, {b:?})");
    }
}

/// ⛔ O defeito que a parte inteira cura: 100 mil acrescentos no fim (e no princípio) dão chaves
/// CURTAS — com a fracção sozinha davam milhares de caracteres.
#[test]
fn a_hundred_thousand_appends_keep_keys_short() {
    let mut top = FracKey::between(None, None);
    let mut bottom = top.clone();
    for _ in 0..100_000 {
        top = FracKey::between(Some(&top), None);
        bottom = FracKey::between(None, Some(&bottom));
    }
    assert!(top.as_str().len() <= 5, "{top:?}");
    assert!(bottom.as_str().len() <= 5, "{bottom:?}");
    assert!(bottom < top);
}

#[test]
fn appending_and_prepending_keep_order() {
    let mut keys = vec![FracKey::between(None, None)];
    for _ in 0..200 {
        let last = keys.last().cloned();
        keys.push(FracKey::between(last.as_ref(), None));
        let first = keys[0].clone();
        keys.insert(0, FracKey::between(None, Some(&first)));
    }
    assert!(keys.windows(2).all(|w| w[0] < w[1]));
}

#[test]
fn random_inserts_stay_strictly_sorted_and_never_end_in_zero() {
    let mut rng = Lcg(0x5EED);
    let mut keys: Vec<FracKey> = Vec::new();
    for _ in 0..5_000 {
        let at = rng.next(keys.len() + 1);
        let lo = at.checked_sub(1).map(|i| keys[i].clone());
        let hi = keys.get(at).cloned();
        let k = FracKey::between(lo.as_ref(), hi.as_ref());
        assert!(lo.as_ref().is_none_or(|l| *l < k), "{lo:?} < {k:?}");
        assert!(hi.as_ref().is_none_or(|h| k < *h), "{k:?} < {hi:?}");
        let int_len = super::integer_len(k.as_str().as_bytes()[0]);
        let frac = &k.as_str()[int_len..];
        assert!(!frac.ends_with('0'), "{k:?}: a fracção termina em zero");
        keys.insert(at, k);
    }
}

#[test]
fn inserting_between_adjacent_digits_descends_a_level() {
    let a = FracKey::between(None, None); // "a0"
    let b = FracKey::between(Some(&a), None);
    let mut lo = a.clone();
    for _ in 0..50 {
        let k = FracKey::between(Some(&lo), Some(&b));
        assert!(lo < k && k < b);
        lo = k;
    }
}

#[test]
#[should_panic(expected = "não é menor")]
fn out_of_order_neighbours_are_a_caller_bug() {
    let a = FracKey::between(None, None);
    let _ = FracKey::between(Some(&a), Some(&a));
}
