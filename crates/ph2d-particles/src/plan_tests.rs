//! O plano da emissão, puro.

use super::*;

fn cfg() -> ParticleEmitter {
    ParticleEmitter {
        amount: 8,
        life: 1.0,
        ..ParticleEmitter::default()
    }
}

/// **L1** — uma rajada emite `amount` em `(1 − e)·vida`, e a explosividade `1` é o `Burst` exacto.
#[test]
fn a_rajada_e_o_ciclo_do_oraculo() {
    for (e, d) in [(0.0_f32, 1.0_f64), (0.5, 0.5), (0.75, 0.25)] {
        let c = ParticleEmitter {
            one_shot: true,
            explosiveness: e,
            ..cfg()
        };
        let em = Emission::born(&c);
        let p = emit_params(&c, &em);
        assert_eq!(p.mode, EMIT_SCHEDULED as f32);
        assert!(
            (f64::from(p.rate) - 8.0 / d).abs() < 1e-4,
            "e = {e}: rate {}",
            p.rate
        );
        assert_eq!(p.schedule.spans(), &[(0.0, d)], "e = {e}");
        assert_eq!(emission_end(&c, &em), Some(d));
    }
    let burst = ParticleEmitter {
        one_shot: true,
        explosiveness: 1.0,
        ..cfg()
    };
    let p = emit_params(&burst, &Emission::born(&burst));
    assert_eq!(
        (p.mode, p.burst_count),
        (1.0, 8.0),
        "explosividade 1 = Burst"
    );
    assert_eq!(emission_end(&burst, &Emission::born(&burst)), Some(0.0));
}

/// O contínuo sem explosividade, sempre ligado, É a identidade da agenda.
#[test]
fn o_continuo_sempre_ligado_e_a_identidade() {
    let c = cfg();
    let p = emit_params(&c, &Emission::born(&c));
    assert!(p.schedule.is_always_on());
    assert!((p.rate - 8.0).abs() < 1e-6, "amount / vida");
    assert_eq!(emission_end(&c, &Emission::born(&c)), None, "nunca acaba");
}

/// **L5 · L6** — desligar fecha o segmento agora; religar abre outro; nada no passado muda.
#[test]
fn ligar_e_desligar_so_mexem_no_futuro() {
    let c = cfg();
    let mut em = Emission::born(&c);
    em.stop(0.5);
    assert_eq!(em.segments(), &[(0.0, Some(0.5))]);
    assert_eq!(emission_end(&c, &em), Some(0.5));
    em.stop(0.9);
    assert_eq!(
        em.segments(),
        &[(0.0, Some(0.5))],
        "desligar o desligado não faz nada"
    );
    em.start(0.75);
    em.start(0.8);
    assert_eq!(
        em.segments(),
        &[(0.0, Some(0.5)), (0.75, None)],
        "ligar o ligado não faz nada"
    );
    assert_eq!(emission_end(&c, &em), None);
    let antes = emit_params(&c, &em).schedule.on_time(0.6);
    em.stop(2.0);
    assert!(
        (emit_params(&c, &em).schedule.on_time(0.6) - antes).abs() < 1e-12,
        "τ do passado não muda — as ids de quem nasceu ficam"
    );
    em.start(3.0);
    em.stop(3.0);
    assert_eq!(em.segments().len(), 2, "um segmento vazio desaparece");
}

/// **C0** — quem nasce desligado não tem agenda, e não tem fim.
#[test]
fn quem_nasce_desligado_nao_emite() {
    let c = ParticleEmitter {
        emitting: false,
        ..cfg()
    };
    let em = Emission::born(&c);
    assert!(!em.ever());
    assert!(emit_params(&c, &em).schedule.is_never());
    assert_eq!(
        emission_end(&c, &em),
        None,
        "sem emissão não há fim a gritar"
    );
    let rajada = ParticleEmitter {
        one_shot: true,
        explosiveness: 1.0,
        ..c
    };
    let p = emit_params(&rajada, &Emission::born(&rajada));
    assert_eq!(p.burst_count, 0.0, "uma rajada desligada não rebenta");
}

/// O contínuo com explosividade pulsa uma vida de cada vez, e `1` não fica sem pulso.
#[test]
fn o_continuo_explosivo_pulsa_por_vida() {
    let c = ParticleEmitter {
        explosiveness: 0.5,
        ..cfg()
    };
    let p = emit_params(&c, &Emission::born(&c));
    assert_eq!(p.schedule.pulse(), Some((0.5, 1.0)));
    assert!((p.rate - 16.0).abs() < 1e-4, "amount por pulso");
    let total = ParticleEmitter {
        explosiveness: 1.0,
        ..cfg()
    };
    let q = emit_params(&total, &Emission::born(&total));
    let (on, per) = q.schedule.pulse().expect("pulsa");
    assert!(on > 0.0 && on <= f64::from(MIN_PULSE) + 1e-9 && (per - 1.0).abs() < 1e-12);
}
