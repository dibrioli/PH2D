use super::*;

fn ponto(fio: i32, ganho: f64) -> Ponto {
    Ponto {
        q: [0.0, 0.0],
        ganho,
        fio,
    }
}

#[test]
fn a_fraccao_dos_que_escavam_e_a_do_knob() {
    for escava in [0.0, 0.25, 0.65, 1.0] {
        let r = Relevo {
            altura_px: 1.0,
            escava,
        };
        let n = (-500..500).filter(|&f| r.escava(f)).count() as f64 / 1000.0;
        assert!((n - escava).abs() < 0.04, "{escava}: {n}");
    }
}

#[test]
fn o_alvo_e_o_ganho_com_o_sinal_do_fio() {
    let r = Relevo {
        altura_px: 2.0,
        escava: 0.5,
    };
    let (cava, constroi) = (
        (0..).find(|&f| r.escava(f)).unwrap(),
        (0..).find(|&f| !r.escava(f)).unwrap(),
    );
    assert_eq!(r.alvo(&ponto(cava, 1.5), 1.0), -3.0);
    assert_eq!(r.alvo(&ponto(constroi, 1.5), 1.0), 3.0);
    assert_eq!(r.alvo(&ponto(constroi, 0.0), 1.0), 0.0);
    assert_eq!(
        r.alvo(&ponto(constroi, 1.5), 0.5),
        0.75,
        "a mão leve: metade × metade"
    );
}

#[test]
fn pentear_chega_ao_alvo_e_fora_da_silhueta_nao_mexe() {
    let mut h = 0.0f32;
    for _ in 0..30 {
        h = penteia(h, 2.0, 1.0);
    }
    assert!((h - 2.0).abs() < 1e-4, "{h}");
    assert_eq!(penteia(0.7, -2.0, 0.0), 0.7);
    assert!(Relevo::NEUTRO.neutro());
}
