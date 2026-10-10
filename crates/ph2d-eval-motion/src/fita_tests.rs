use super::*;
use ph2d_nodegraph::attr::{Column, Stream};

fn ext(v: f32) -> External {
    let value = Stream::new(1).with("v", Column::Scalar(vec![v]));
    External {
        rev: ph2d_nodegraph::external::fingerprint(&value),
        value,
    }
}

fn pub_(pares: &[(&str, f32)]) -> All {
    pares
        .iter()
        .map(|(n, v)| ((*n).to_string(), ext(*v)))
        .collect()
}

fn v(all: &All, nome: &str) -> Option<f32> {
    match all.get(nome)?.value.get("v")? {
        Column::Scalar(x) => x.first().copied(),
        _ => None,
    }
}

#[test]
fn each_tick_reads_back_what_was_published_and_unchanged_ticks_cost_nothing() {
    let mut f = Fita::new();
    for t in 0..10u64 {
        let g = if t < 5 { 1.0 } else { 2.0 };
        assert_eq!(
            f.grava(t, &pub_(&[("g", g), ("parado", 7.0)])),
            Gravacao::Nova
        );
    }
    assert_eq!(
        f.marcas.len(),
        2,
        "só as mudanças (o tique 0 e o 5) se gravam"
    );
    for t in 0..10u64 {
        let s = f.estado_em(t).unwrap();
        assert_eq!(v(&s, "g"), Some(if t < 5 { 1.0 } else { 2.0 }), "tique {t}");
        assert_eq!(v(&s, "parado"), Some(7.0));
    }
    assert!(
        f.estado_em(10).is_none(),
        "para lá do gravado não se inventa"
    );
}

#[test]
fn a_name_that_stops_being_published_is_absent_at_its_tick() {
    let mut f = Fita::new();
    f.grava(0, &pub_(&[("a", 1.0), ("b", 2.0)]));
    f.grava(1, &pub_(&[("a", 1.0)]));
    assert!(!f.estado_em(1).unwrap().contains_key("b"));
    assert_eq!(v(&f.estado_em(0).unwrap(), "b"), Some(2.0));
}

#[test]
fn walking_tick_by_tick_equals_rebuilding_each_tick() {
    let mut f = Fita::new();
    for t in 0..30u64 {
        f.grava(t, &pub_(&[("g", (t / 4) as f32)]));
    }
    let mut estado = f.estado_em(3).unwrap();
    for t in 4..30u64 {
        f.avanca(t, &mut estado);
        assert!(
            mesmas_revisoes(&estado, &f.estado_em(t).unwrap()),
            "tique {t}"
        );
    }
}

#[test]
fn replaying_over_recorded_ground_is_equal_and_a_different_input_is_a_branch() {
    let mut f = Fita::new();
    for t in 0..20u64 {
        f.grava(t, &pub_(&[("g", t as f32)]));
    }
    assert_eq!(f.grava(7, &pub_(&[("g", 7.0)])), Gravacao::Igual);
    assert_eq!(f.grava(8, &pub_(&[("g", 99.0)])), Gravacao::Ramo);
    assert_eq!(v(&f.estado_em(8).unwrap(), "g"), Some(99.0));
    assert_eq!(
        v(&f.estado_em(7).unwrap(), "g"),
        Some(7.0),
        "o passado do ramo fica"
    );
    assert!(f.estado_em(9).is_none(), "o futuro velho foi esquecido");
    assert_eq!(f.grava(9, &pub_(&[("g", 100.0)])), Gravacao::Nova);
}

#[test]
fn recording_before_the_first_tick_starts_over() {
    let mut f = Fita::new();
    for t in 10..20u64 {
        f.grava(t, &pub_(&[("g", 1.0)]));
    }
    assert!(f.estado_em(5).is_none());
    assert_eq!(f.grava(5, &pub_(&[("g", 3.0)])), Gravacao::Ramo);
    assert!(f.estado_em(12).is_none());
    assert_eq!(v(&f.estado_em(5).unwrap(), "g"), Some(3.0));
}

#[test]
fn over_budget_the_oldest_change_folds_into_the_base_and_the_bytes_stay_bounded() {
    let mut f = Fita::new();
    let um = peso(&vec![("g".to_string(), Some(ext(0.0)))]);
    f.set_orcamento(um * 10);
    for t in 0..100u64 {
        f.grava(t, &pub_(&[("g", t as f32)]));
        assert!(f.bytes() <= um * 10, "tique {t}: {} bytes", f.bytes());
    }
    assert_eq!(v(&f.estado_em(99).unwrap(), "g"), Some(99.0));
    assert_eq!(
        v(&f.estado_em(95).unwrap(), "g"),
        Some(95.0),
        "o recente fica exacto"
    );
    // O limite NOMEADO: antes da base, repete-se a base.
    assert_eq!(
        v(&f.estado_em(3).unwrap(), "g"),
        v(&f.estado_em(f.base_ate.unwrap()).unwrap(), "g")
    );
}
