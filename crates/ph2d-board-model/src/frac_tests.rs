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

#[test]
fn the_first_key_is_the_middle_of_the_base() {
    assert_eq!(FracKey::between(None, None).as_str(), "V");
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
        assert!(!k.as_str().ends_with('0'), "{k:?} termina em zero");
        keys.insert(at, k);
    }
}

#[test]
fn inserting_between_adjacent_digits_descends_a_level() {
    let a = FracKey::between(None, None); // "V"
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
