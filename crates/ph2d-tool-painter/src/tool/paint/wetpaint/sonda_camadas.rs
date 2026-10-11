//! **As sondas (ignoradas) das camadas de amarelo sobre azul seco** (prompt 15): a tabela H do 05 §4b
//! «Mistura», as duas etapas do traço no Up e a velatura re-medida — fora de [`super::mistura_camadas_tests`]
//! pelo tecto de LOC por ficheiro (a cena, as réguas e os portões ficam lá).
use super::mistura_camadas_tests::{
    Cond, H, SIMPLES, TUDO, Trajectoria, W, barra, cena, delta, duas_etapas, lum, para_o_amarelo,
    r, tom,
};

/// `PH2D_SONDA_ESTADO=seco|molhado`.
#[test]
#[ignore = "sonda: corre-se à mão"]
fn sonda_camadas() {
    use ph2d_wet_paint::tuning::Knob;
    type Aj = Box<dyn Fn(&mut ph2d_wet_paint::painter::Engine)>;
    let filtro = |var: &str, nome: &str| {
        std::env::var(var)
            .ok()
            .is_none_or(|s| s.split(',').any(|x| x == nome))
    };
    let variantes: Vec<(&str, Aj)> = vec![
        ("produto", Box::new(|_| {})),
        (
            "antes",
            Box::new(|e| {
                e.sim.pickup_assentado = 0.5;
                e.sim.glaze = ph2d_wet_paint::colorops::VELATURA_MEDIDA;
            }),
        ),
        (
            "pincel apanha o assentado",
            Box::new(|e| e.sim.pickup_assentado = 0.5),
        ),
        ("velatura 0", Box::new(|e| e.sim.glaze = 0.0)),
        (
            "velatura medida",
            Box::new(|e| e.sim.glaze = ph2d_wet_paint::colorops::VELATURA_MEDIDA),
        ),
        (
            "Pickup 0",
            Box::new(|e| {
                e.tuning.set(Knob::Pickup, 0.0);
            }),
        ),
        (
            "Staining 1",
            Box::new(|e| {
                e.tuning.set(Knob::ExtStaining, 1.0);
            }),
        ),
    ];
    let km = |c: Cond| Cond { km: true, ..c };
    let esfrega = Cond {
        tamanho: 60.0,
        por_traco: 5,
        ..SIMPLES
    };
    let conds: Vec<(&str, Cond)> = vec![
        ("simples", SIMPLES),
        (
            "tilt 4",
            Cond {
                opacidade: None,
                tilt: Some(4),
                ..SIMPLES
            },
        ),
        (
            "borda 1",
            Cond {
                borda: 1.0,
                ..SIMPLES
            },
        ),
        (
            "viva",
            Cond {
                viva: true,
                ..SIMPLES
            },
        ),
        ("esfrega", esfrega),
        ("tudo", TUDO),
        ("K–M simples", km(SIMPLES)),
        ("K–M esfrega", km(esfrega)),
        ("K–M tudo", km(TUDO)),
    ];
    let estados: Vec<bool> = match std::env::var("PH2D_SONDA_ESTADO").as_deref() {
        Ok("seco") => vec![true],
        Ok("molhado") => vec![false],
        _ => vec![false, true],
    };
    for (cnome, cond) in &conds {
        if !filtro("PH2D_SONDA_COND", cnome) {
            continue;
        }
        for &seco_entre in &estados {
            for (nome, aj) in &variantes {
                if !filtro("PH2D_SONDA_SO", nome) {
                    continue;
                }
                let tr = cena(seco_entre, 10, *cond, aj.as_ref());
                let az = tr.passadas[0].ecra;
                let est = if seco_entre { "seco" } else { "molhado" };
                if let Ok(dir) = std::env::var("PH2D_SONDA_OUT") {
                    for (k, px) in tr.telas.iter().enumerate() {
                        let mut ppm = format!("P6 {W} {H} 255\n").into_bytes();
                        for q in px.as_chunks::<4>().0 {
                            ppm.extend_from_slice(&q[..3]);
                        }
                        let nome = format!("{cnome}_{est}_{nome}_{k:02}.ppm").replace(' ', "_");
                        std::fs::write(format!("{dir}/{nome}"), ppm).expect("grava a tela");
                    }
                }
                println!(
                    "== [{cnome} · {est}] {nome}: azul {:?} lum {:.4} · traço amarelo no branco {:?} · barra {}",
                    r(az),
                    lum(az),
                    r(tr.amarelo),
                    if barra("", &tr, false).is_ok() {
                        "ok"
                    } else {
                        "✗"
                    }
                );
                for (k, m) in tr
                    .passadas
                    .iter()
                    .enumerate()
                    .chain([(98, &tr.fim), (99, &tr.seco)])
                {
                    println!(
                        "  p{k:02} ecrã {:?} lum {:.4} escuro {:.4} →amarelo {:+.2} | sett {:?} m {:.0} | susp {:?} m {:.0}",
                        r(m.ecra),
                        lum(m.ecra),
                        m.escuro,
                        para_o_amarelo(az, tr.amarelo, m.ecra),
                        r(m.motor.sett_rgb),
                        m.motor.sett,
                        r(m.motor.susp_rgb),
                        m.motor.susp
                    );
                }
            }
        }
    }
}

/// **Sonda: o traço muda de aparência no Up?** (dono 10/10: «Pigment mixing + Glaze layering pintando
/// sobre tinta seca ainda tem uma aparência em 2 etapas»). As 4 combinações das caixas × Live Water.
#[test]
#[ignore = "sonda: corre-se à mão"]
fn sonda_duas_etapas() {
    let st1 = |e: &mut ph2d_wet_paint::painter::Engine| {
        e.tuning.set(ph2d_wet_paint::tuning::Knob::ExtStaining, 1.0);
    };
    let porto =
        |e: &mut ph2d_wet_paint::painter::Engine| e.sim.lei = ph2d_wet_paint::assenta::Lei::Porto;
    let sem_secar = |e: &mut ph2d_wet_paint::painter::Engine| {
        e.tuning.set(ph2d_wet_paint::tuning::Knob::Evaporation, 0.0);
        e.tuning
            .set(ph2d_wet_paint::tuning::Knob::BaseEvaporation, 0.0);
    };
    let sem_orla = |e: &mut ph2d_wet_paint::painter::Engine| {
        e.tuning
            .set(ph2d_wet_paint::tuning::Knob::EdgeDarkening, 0.0);
    };
    let so: Option<String> = std::env::var("PH2D_SONDA_SO").ok();
    for viva in [false, true] {
        for (km, glaze, nome, aj) in [
            (
                false,
                false,
                "",
                &(|_: &mut ph2d_wet_paint::painter::Engine| {}) as &dyn Fn(&mut _),
            ),
            (true, false, "", &|_| {}),
            (false, true, "", &|_| {}),
            (true, true, "", &|_| {}),
            (false, false, "Staining 1", &st1),
            (false, true, "Staining 1", &st1),
            (true, true, "lei de antes", &porto),
            (false, false, "sem secar", &sem_secar),
            (false, false, "sem orla", &sem_orla),
        ] {
            if so.as_deref().is_some_and(|s| s != nome) {
                continue;
            }
            let v = duas_etapas(km, glaze, viva, aj);
            let pt = v[1..].iter().map(|c| tom(v[0], *c)).fold(0.0, f64::max);
            println!("  tom: pior {pt:.3}");
            println!(
                "[K–M {km:5} · Glaze {glaze:5} · Live {viva:5} {nome}] em baixo {:?} | +1 {:?} Δ{:.0} | +30 {:?} Δ{:.0} | +120 {:?} Δ{:.0} | +600 {:?} Δ{:.0} | seco {:?} Δ{:.0}",
                r(v[0]),
                r(v[1]),
                delta(v[0], v[1]),
                r(v[2]),
                delta(v[0], v[2]),
                r(v[3]),
                delta(v[0], v[3]),
                r(v[4]),
                delta(v[0], v[4]),
                r(v[5]),
                delta(v[0], v[5])
            );
        }
    }
}

/// **Sonda: a velatura re-medida com a tinta do Rebelle** (pincéis calibrados 10/10): a grelha velatura das
/// junções × do filme, nas duas barras (a de uma camada, W1e; e as camadas, fábrica molhado e seco).
/// `PH2D_SONDA_GRELHA` = `g:f;…` (água parada) e `PH2D_SONDA_VIVA=1` para a Live Water.
#[test]
#[ignore = "sonda: corre-se à mão"]
fn sonda_velatura_recalibra() {
    use super::mistura_tests::{
        SONDA_ESCONDE, SONDA_MOLHADA, SONDA_VELATURA, barra as barra_w1e, cena_toda, le,
    };
    let viva = std::env::var("PH2D_SONDA_VIVA").is_ok_and(|v| v == "1");
    let cold = std::env::var("PH2D_SONDA_COLD").is_ok_and(|v| v == "1");
    let cena_com = |seco, produto, filme, viva| cena_toda(seco, produto, filme, viva, false, cold);
    let grelha = std::env::var("PH2D_SONDA_GRELHA").expect("PH2D_SONDA_GRELHA");
    for gf in grelha.split(';').filter(|s| !s.is_empty()) {
        let mut it = gf.split(':');
        let g: f64 = it.next().expect("g").parse().expect("g");
        let f: f64 = it.next().expect("f").parse().expect("f");
        let hh: f64 = it
            .next()
            .map_or(ph2d_wet_paint::colorops::TINTA_ESCONDE, |h| {
                h.parse().expect("h")
            });
        let kk: f64 = it
            .next()
            .map_or(ph2d_wet_paint::colorops::TINTA_MOLHADA_ESCURECE, |k| {
                k.parse().expect("k")
            });
        SONDA_VELATURA.set(Some(g));
        SONDA_ESCONDE.set(Some(hh));
        SONDA_MOLHADA.set(Some(kk));
        let ler = |(z, a, b): (Vec<u8>, Vec<u8>, Vec<u8>)| (le(&a, &z), le(&b, &z));
        let (s, m) = (
            ler(cena_com(true, true, Some(f), viva)),
            ler(cena_com(false, true, Some(f), viva)),
        );
        SONDA_VELATURA.set(None);
        SONDA_ESCONDE.set(None);
        SONDA_MOLHADA.set(None);
        let aj = move |e: &mut ph2d_wet_paint::painter::Engine| {
            e.sim.glaze = g;
            e.sim.glaze_filme = f;
            e.sim.esconde = hh;
            e.sim.molhada_escurece = kk;
        };
        let cond = Cond { viva, ..SIMPLES };
        let (mol, sec) = (cena(false, 10, cond, &aj), cena(true, 10, cond, &aj));
        let de = duas_etapas(false, false, viva, &aj);
        let tom_pior = de[1..].iter().map(|c| tom(de[0], *c)).fold(0.0, f64::max);
        let resumo = |t: &Trajectoria| {
            let az = lum(t.passadas[0].ecra);
            let pior = t.passadas[1..]
                .iter()
                .map(|m| lum(m.ecra) / az)
                .fold(f64::MAX, f64::min);
            format!(
                "p1 {:.2} p10 {:.2} pior lum/azul {:.2}",
                para_o_amarelo(t.passadas[0].ecra, t.amarelo, t.passadas[1].ecra),
                para_o_amarelo(t.passadas[0].ecra, t.amarelo, t.passadas[10].ecra),
                pior
            )
        };
        println!(
            "[g {g:.2} f {f:.2} h {hh:.1} k {kk:.1} viva {viva}] W1e seco γ {:.2}·{:.2} sangria {:+.3} · molhado γ {:.2} sangria {:+.3} · barra {} | camadas molhado {} · seco {} · barra {} {}",
            s.0.gama,
            s.1.gama,
            s.1.sangria,
            m.0.gama,
            m.1.sangria,
            if barra_w1e(s.clone(), m.clone()).is_ok() {
                "ok"
            } else {
                "✗"
            },
            resumo(&mol),
            resumo(&sec),
            if barra("", &mol, false).is_ok() {
                "ok"
            } else {
                "✗"
            },
            if barra("", &sec, false).is_ok() {
                "ok"
            } else {
                "✗"
            },
        );
        println!(
            "      duas etapas: em baixo {:?} → seco {:?} · tom pior {tom_pior:.3}",
            r(de[0]),
            r(de[5])
        );
    }
}
