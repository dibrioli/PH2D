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

/// R1 — vizinhos trocados (dados de um par ou de um ficheiro fora de ordem) não derrubam quem cria:
/// a chave cai entre os dois, como se viessem por ordem.
#[test]
fn out_of_order_neighbours_give_a_key_between_them() {
    let a = FracKey::between(None, None);
    let b = FracKey::between(Some(&a), None);
    let k = FracKey::between(Some(&b), Some(&a));
    assert!(a < k && k < b, "{k:?}");
}

/// R1 — texto fora da gramática da chave (vindo da rede ou de um ficheiro) NÃO vira chave: antes
/// entrava e o `between` seguinte dava pânico.
#[test]
fn text_outside_the_key_grammar_never_deserializes() {
    let smallest = format!("A{}", "0".repeat(26));
    for bad in ["", "a", "!x", "a0!", "a00", "b1", "a0 ", "é0", smallest.as_str()] {
        let bytes = postcard::to_allocvec(&bad.to_string()).unwrap();
        assert!(
            postcard::from_bytes::<FracKey>(&bytes).is_err(),
            "{bad:?} entrou como chave"
        );
        assert!(FracKey::try_from(bad.to_string()).is_err(), "{bad:?}");
    }
    for good in ["a0", "Zz", "a0V", "b125", "Xzzz", "A00000000000000000000000000V"] {
        assert_eq!(key(good).as_str(), good);
    }
}

/// R1 — para QUALQUER par de chaves válidas, em qualquer ordem e com as pontas extremas, `between`
/// devolve uma chave válida estritamente entre as duas (ou logo acima no empate), sem pânico.
#[test]
fn any_two_valid_keys_give_a_valid_key_between_them() {
    let mut rng = Lcg(0xF0CA);
    let digit = |r: &mut Lcg| super::DIGITS[r.next(62)];
    let mut pick = |r: &mut Lcg| -> FracKey {
        loop {
            let head = if r.next(2) == 0 { b'a' } else { b'A' } + u8::try_from(r.next(26)).unwrap();
            let mut k = vec![head];
            let n = super::integer_len(head);
            let edge = r.next(4);
            for _ in 1..n {
                k.push(match edge {
                    0 => b'0',
                    1 => b'z',
                    _ => digit(r),
                });
            }
            for _ in 0..r.next(4) {
                k.push(digit(r));
            }
            if let Ok(key) = FracKey::try_from(String::from_utf8(k).unwrap()) {
                return key;
            }
        }
    };
    for _ in 0..20_000 {
        let (a, b) = (pick(&mut rng), pick(&mut rng));
        let (lo, hi) = (a.clone().min(b.clone()), a.clone().max(b.clone()));
        let k = FracKey::between(Some(&a), Some(&b));
        assert!(FracKey::try_from(k.as_str().to_string()).is_ok(), "{k:?}");
        assert!(lo < k && (k < hi || lo == hi), "{a:?} {b:?} -> {k:?}");
        let up = FracKey::between(Some(&a), None);
        assert!(up > a && FracKey::try_from(up.as_str().to_string()).is_ok(), "{a:?} -> {up:?}");
        let down = FracKey::between(None, Some(&a));
        assert!(down < a && FracKey::try_from(down.as_str().to_string()).is_ok(), "{a:?} -> {down:?}");
    }
}

/// ADR-0181 — duas chaves IGUAIS (duas pessoas criaram no mesmo sítio ao mesmo tempo) são um empate
/// legítimo: a chave «entre» elas fica logo acima, abaixo do inteiro seguinte (antes: pânico).
#[test]
fn a_tie_gives_the_key_just_above_it() {
    let k = FracKey::between(None, None);
    let up = FracKey::between(Some(&k), Some(&k));
    assert!(up > k && up < FracKey::between(Some(&k), None), "{up:?}");
}

/// A cauda de uma sessão deixa a chave acima da de antes e abaixo do tecto; caudas diferentes, chaves
/// diferentes; e se não cabe, a chave fica como estava.
#[test]
fn a_salted_key_stays_in_its_gap_and_differs_by_salt() {
    let lo = FracKey::between(None, None);
    let hi = FracKey::between(Some(&lo), None);
    let k = FracKey::between(Some(&lo), Some(&hi));
    let (a, b) = (
        k.clone().salted(1, Some(&hi)),
        k.clone().salted(2, Some(&hi)),
    );
    for s in [&a, &b] {
        assert!(*s > k && *s < hi && !s.as_str().ends_with('0'), "{s:?}");
    }
    assert_ne!(a, b);
    let tight = FracKey::between(Some(&k), None);
    let just = FracKey::between(Some(&k), Some(&tight));
    assert!(just.clone().salted(u64::MAX, Some(&just)) == just);
    assert!(FracKey::between(Some(&a), Some(&b.clone().max(a.clone()))) >= a.clone().min(b));
}

/// ⭐ B7 — o prefixo inteiro ordena como a chave: chaves de inserções ao acaso, com caudas de quem colabora e
/// longas com o mesmo começo; um prefixo MENOR nunca é de uma chave maior, e uma chave menor nunca tem um
/// prefixo maior.
#[test]
fn the_prefix_orders_like_the_key() {
    let mut rng = Lcg(7);
    let mut keys: Vec<FracKey> = vec![FracKey::between(None, None)];
    for i in 0..3_000 {
        keys.sort();
        let k = rng.next(keys.len() + 1);
        let lo = k.checked_sub(1).map(|j| &keys[j]);
        let hi = keys.get(k);
        let mut new = FracKey::between(lo, hi);
        if i % 5 == 0 {
            new = new.salted(rng.next(1 << 30) as u64, hi);
        }
        keys.push(new);
    }
    // Uma pilha de inserções sempre no MESMO vão: o começo partilhado passa dos 8 bytes.
    let (mut lo, hi) = (key("a0"), key("a1"));
    for _ in 0..40 {
        lo = FracKey::between(Some(&lo), Some(&hi));
        keys.push(lo.clone());
    }
    assert!(
        keys.iter().any(|k| k.as_str().len() > 8),
        "há chaves além do prefixo"
    );
    for a in &keys {
        for b in keys.iter().step_by(7) {
            if a.prefix() < b.prefix() {
                assert!(a < b, "{a:?} {b:?}");
            }
            if a < b {
                assert!(a.prefix() <= b.prefix(), "{a:?} {b:?}");
            }
        }
    }
}
