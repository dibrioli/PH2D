//! A lei da AGENDA, pura — `τ`, o nascimento, e o texto (TOP-20 #18, W0).

use super::*;

fn sched(t: &str) -> Schedule {
    Schedule::parse(t).unwrap_or_else(|e| panic!("`{t}` devia ser uma agenda: {e}"))
}

/// **O texto vai e volta**, e a forma canónica é a que o componente escreve no nó.
#[test]
fn o_texto_vai_e_volta() {
    for t in [
        "",
        "off",
        "0-0.5",
        "0-0.5 2-",
        "pulse 0.25/1",
        "0-1 3-4.5 6- pulse 0.25/2",
        "0-1 3-4.5 6-",
    ] {
        let s = sched(t);
        assert_eq!(Schedule::parse(&s.format()).unwrap(), s, "`{t}`");
    }
    assert_eq!(
        sched("0-1, 2-3").format(),
        "0-1 2-3",
        "a vírgula é separador"
    );
    assert!(sched("").is_always_on());
    assert!(sched("   ").is_always_on(), "só espaços = a identidade");
    assert!(sched("off").is_never() && sched("OFF").is_never());
    assert!(
        !sched("off").is_always_on(),
        "«desligado» não se confunde com a identidade"
    );
}

/// ⛔ **Malformado é RECUSADO, nunca lido como «sempre ligado»** — cada forma pelo seu motivo.
#[test]
fn o_texto_malformado_e_recusado_pelo_seu_motivo() {
    use ScheduleError as E;
    type Motivo = fn(&E) -> bool;
    let casos: &[(&str, Motivo)] = &[
        ("abc", |e| matches!(e, E::Token(_))),
        ("1", |e| matches!(e, E::Token(_))),
        ("-1-2", |e| matches!(e, E::Token(_))),
        ("2-1", |e| matches!(e, E::Span(_))),
        ("1-1", |e| matches!(e, E::Span(_))),
        ("0-2 1-3", |e| matches!(e, E::Order)),
        ("2-3 0-1", |e| matches!(e, E::Order)),
        ("0- 2-3", |e| matches!(e, E::Open)),
        ("pulse 2/1", |e| matches!(e, E::Pulse)),
        ("pulse 0/1", |e| matches!(e, E::Pulse)),
        ("pulse 1", |e| matches!(e, E::Pulse)),
        ("0-1 pulse", |e| matches!(e, E::Pulse)),
        ("pulse 0.1/1 pulse 0.2/1", |e| matches!(e, E::Pulse)),
        ("pulse x/1", |e| matches!(e, E::Pulse)),
        ("off 0-1", |e| matches!(e, E::Off)),
    ];
    for (t, motivo) in casos {
        let e = Schedule::parse(t).expect_err(t);
        assert!(motivo(&e), "`{t}` recusado pelo motivo errado: {e:?}");
        assert!(!e.to_string().is_empty());
    }
}

/// **`τ` é o tempo LIGADO acumulado** — monótona, plana nas pausas, a repetir com o período.
#[test]
fn tau_e_o_tempo_ligado_acumulado() {
    let s = sched("1-2 3-");
    let casos = [
        (-1.0, 0.0),
        (0.5, 0.0),
        (1.5, 0.5),
        (2.0, 1.0),
        (2.9, 1.0),
        (4.0, 2.0),
    ];
    for (t, e) in casos {
        assert!(
            (s.on_time(t) - e).abs() < 1e-12,
            "τ({t}) = {}",
            s.on_time(t)
        );
    }
    let p = sched("pulse 0.25/1");
    assert!((p.on_time(2.5) - 0.75).abs() < 1e-12, "{}", p.on_time(2.5));
    assert!((p.on_time(2.1) - 0.6).abs() < 1e-12, "{}", p.on_time(2.1));
    // ⭐ O pulso conta desde o início de CADA segmento — religar recomeça o ciclo.
    let q = sched("0-1.2 3- pulse 0.5/1");
    assert!(
        (q.on_time(1.2) - 0.7).abs() < 1e-12,
        "cortado a meio do 2.º pulso: {}",
        q.on_time(1.2)
    );
    assert!(
        (q.on_time(3.4) - 1.1).abs() < 1e-12,
        "o 3.º segmento pulsa desde 3: {}",
        q.on_time(3.4)
    );
    let mut prev = 0.0;
    for i in 0..400 {
        let t = f64::from(i) * 0.013;
        let e = s.on_time(t);
        assert!(e >= prev, "τ recuou em t = {t}");
        prev = e;
    }
}

/// ⭐ **O nascimento é o inverso de `τ` sobre o tempo LIGADO** — e só nasce com a emissão ligada.
#[test]
fn o_nascimento_e_o_inverso_de_tau_e_so_com_a_emissao_ligada() {
    for s in [
        sched("1-2 3-"),
        sched("pulse 0.25/1"),
        sched("0.5-0.75 1-1.5"),
        sched("0-1.2 3- pulse 0.5/1"),
    ] {
        for i in 0..200 {
            let e = f64::from(i) * 0.011;
            let Some(t) = s.birth(e) else { continue };
            assert!(
                (s.on_time(t) - e).abs() < 1e-9,
                "{s:?}: τ(birth({e})) ≠ {e}"
            );
            assert!(
                s.is_on(t) || s.is_on(t + 1e-9),
                "{s:?}: nasceu em {t}, com a emissão desligada"
            );
        }
    }
    assert_eq!(
        sched("1-2 3-").birth(0.0),
        Some(1.0),
        "a primeira nasce quando liga"
    );
    assert_eq!(sched("").birth(2.5), Some(2.5), "a identidade");
    assert_eq!(sched("off").birth(0.0), None, "desligado: nunca");
    assert_eq!(sched("1-2 3-").birth(-0.1), None);
}

/// ⭐⭐ **Meio-aberto: a vez que cai num FIM é do intervalo seguinte** — e é isso que faz uma
/// rajada `[0, D)` a `n / D` dar exactamente `n` partículas (L1 do oráculo).
#[test]
fn a_vez_que_cai_no_fim_e_do_intervalo_seguinte() {
    let s = sched("1-2 3-");
    assert_eq!(
        s.birth(1.0),
        Some(3.0),
        "τ = 1 no fim de [1,2) ⇒ nasce em 3"
    );
    let (n, d) = (8u32, 1.0_f64);
    let rajada = Schedule::new(vec![(0.0, d)], None).unwrap();
    let rate = f64::from(n) / d;
    let nascidas = (0..20)
        .filter(|&k| rajada.birth(f64::from(k) / rate).is_some())
        .count();
    assert_eq!(nascidas, n as usize, "a rajada dá exactamente n");
    let per = sched("pulse 0.5/1");
    assert_eq!(
        per.birth(0.5),
        Some(1.0),
        "o fim de um pulso é o início do seguinte"
    );
    let cortado = sched("0-1.2 3- pulse 0.5/1");
    assert_eq!(
        cortado.birth(0.7),
        Some(3.0),
        "o que sobra do segmento cortado vai para o seguinte"
    );
}

/// **Quando a emissão acaba de vez** — o que o componente lê para o `finished` (L2).
#[test]
fn o_fim_da_emissao() {
    assert_eq!(sched("").end(), None, "sempre ligada");
    assert_eq!(sched("0-1 2-").end(), None, "aberta");
    assert_eq!(sched("pulse 0.5/2").end(), None, "a pulsar para sempre");
    assert_eq!(
        sched("0-3 pulse 0.5/2").end(),
        Some(3.0),
        "um pulso num segmento finito acaba"
    );
    assert_eq!(sched("0-1 2-3.5").end(), Some(3.5));
    assert_eq!(
        sched("off").end(),
        Some(0.0),
        "nunca ligou ⇒ acabou no zero"
    );
}
