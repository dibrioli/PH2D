//! As duas funções puras da ponte da vida.

use super::*;

/// O sorteio fica em `[0, 1)` e não é constante — a esquiva depende dele.
#[test]
fn o_sorteio_fica_no_intervalo_e_anda() {
    let mut rng = 0_u64;
    let xs: Vec<f64> = (0..10_000).map(|_| sorteio(&mut rng)).collect();
    assert!(xs.iter().all(|&x| (0.0..1.0).contains(&x)));
    let media = xs.iter().sum::<f64>() / xs.len() as f64;
    assert!((media - 0.5).abs() < 0.02, "média {media}");
}

/// Os factos saem da DIFERENÇA: um golpe que parte o escudo e passa à vida dá os dois, e a morte
/// só é contada na transição.
#[test]
fn os_factos_saem_da_diferenca() {
    let cfg = ph2d_health::Config {
        escudo_max: 50.0,
        ..ph2d_health::Config::default()
    };
    let mut v = Vida::nasce(100.0, &cfg);
    v.activa_escudo(&cfg, 10.0, true);
    let antes = v;
    v.golpe(&cfg, Regras::CASA, 30.0, true, true, &mut || 0.5);
    assert_eq!(
        factos(&antes, &v),
        vec![
            HealthEventKind::Shielded { amount: 10.0 },
            HealthEventKind::Damaged { amount: 20.0 }
        ]
    );
    let antes = v;
    v.golpe(&cfg, Regras::CASA, 500.0, true, true, &mut || 0.5);
    assert_eq!(
        factos(&antes, &v),
        vec![
            HealthEventKind::Damaged { amount: 80.0 },
            HealthEventKind::Died
        ]
    );
    // Um golpe num morto não produz nada (a casa: um morto é final).
    let antes = v;
    v.golpe(&cfg, Regras::CASA, 10.0, true, true, &mut || 0.5);
    assert!(factos(&antes, &v).is_empty());
}
