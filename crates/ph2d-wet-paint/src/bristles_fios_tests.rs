use super::*;

fn com(f: impl FnOnce(&mut Fios)) -> Fios {
    let mut x = Fios::NEUTRO;
    f(&mut x);
    x
}

/// O neutro não chama a lei; qualquer um dos três a liga.
#[test]
fn o_neutro_nao_chama_a_lei() {
    assert!(Fios::NEUTRO.no_dab(37.0, [1.0, 0.0], 1.0).is_none());
    // Os comprimentos e a largura sozinhos não ligam nada.
    let so_escalas = com(|f| {
        f.ruido_px = 40.0;
        f.largura_px = 9.0;
        f.onda_px = 77.0;
    });
    assert!(so_escalas.no_dab(37.0, [1.0, 0.0], 1.0).is_none());
    for f in [
        com(|f| f.pressao = 0.5),
        com(|f| f.onda = 0.3),
        com(|f| f.oscilacao_px = 2.0),
        com(|f| f.span_px = 2.0),
        com(|f| f.cor = [0.0, 0.0, 0.1]),
    ] {
        assert!(f.no_dab(37.0, [1.0, 0.0], 1.0).is_some(), "{f:?}");
    }
}

/// O Perlin: `0` nos inteiros, dentro de `[-1, 1]`, determinístico, e não constante entre eles.
#[test]
fn o_perlin_e_zero_nos_inteiros_e_limitado() {
    let mut max = 0.0f64;
    for k in -6..6 {
        for i in -40..400 {
            let t = f64::from(i) * 0.37;
            let n = perlin(t, k, SEMENTE_PRESSAO);
            assert_eq!(n.to_bits(), perlin(t, k, SEMENTE_PRESSAO).to_bits());
            assert!(n.abs() <= 1.0, "{n} em {t}");
            max = max.max(n.abs());
        }
        assert_eq!(perlin(5.0, k, SEMENTE_PRESSAO), 0.0);
    }
    assert!(max > 0.4, "o ruído quase não varia: {max}");
}

/// ⭐ **A pressão é do FIO e muda ao longo do caminho:** dois pontos do mesmo fio no mesmo arco têm o mesmo
/// ganho; o mesmo fio noutro arco, outro; fios vizinhos no mesmo arco, outros — e a média fica perto de 1.
#[test]
fn a_pressao_e_do_fio_e_muda_ao_longo_do_caminho() {
    let f = com(|f| {
        f.pressao = 1.0;
        f.largura_px = 4.0;
        f.ruido_px = 100.0;
    });
    let g = |arco: f64, l: f64| {
        f.no_dab(arco, [1.0, 0.0], 1.0)
            .unwrap()
            .no_ponto([3.0, l])
            .ganho
    };
    assert_eq!(g(230.0, 4.2).to_bits(), g(230.0, 7.9).to_bits());
    assert_ne!(g(230.0, 4.2), g(290.0, 4.2));
    let vizinhos = (0..12)
        .filter(|&k| g(230.0, 4.0 * f64::from(k) + 1.0) != g(230.0, 4.0 * f64::from(k) + 5.0));
    assert!(vizinhos.count() >= 10);
    let n = 4000;
    let media = (0..n)
        .map(|i| g(f64::from(i) * 7.3, f64::from(i % 37) * 1.3 - 20.0))
        .sum::<f64>()
        / f64::from(n);
    assert!((media - 1.0).abs() < 0.12, "{media}");
    assert!(
        (0..n).all(|i| g(f64::from(i) * 7.3, 2.0) >= 0.0),
        "o ganho nunca é negativo"
    );
}

/// ⭐ **A largura ondula e nenhum fio some:** com a onda no tecto, as fronteiras andam ao longo do caminho, a
/// atribuição é crescente através do traço e cada fio fica com ≥ ¼ da largura.
#[test]
fn a_onda_mexe_as_fronteiras_e_nenhum_fio_some() {
    let f = com(|f| {
        f.pressao = 1.0;
        f.onda = ONDA_MAX;
        f.largura_px = 4.0;
    });
    let mut andou = false;
    for a in 0..200 {
        let arco = f64::from(a) * 13.0;
        let fd = f.no_dab(arco, [0.0, 1.0], 1.0).unwrap();
        let fios: Vec<i32> = (0..800)
            .map(|i| fd.fio(f64::from(i) * 0.01 - 2.0))
            .collect();
        assert!(
            fios.windows(2).all(|w| w[1] == w[0] || w[1] == w[0] + 1),
            "{arco}"
        );
        for k in fios[0] + 1..*fios.last().unwrap() {
            let largura = fios.iter().filter(|&&x| x == k).count();
            assert!(
                largura >= 24,
                "o fio {k} tem {largura} amostras de 100 em {arco}"
            );
        }
        andou |= fd.fio(0.2) != 0 || fd.fio(0.8) != 0;
    }
    assert!(andou, "a onda não mexeu nenhuma fronteira");
    // CONTROLO: sem onda as fronteiras são os inteiros.
    let fixa = com(|f| f.pressao = 1.0)
        .no_dab(500.0, [1.0, 0.0], 1.0)
        .unwrap();
    assert_eq!([fixa.fio(-0.01), fixa.fio(0.0), fixa.fio(2.99)], [-1, 0, 2]);
}

/// **A oscilação desloca o pincel na NORMAL do traço**, no máximo `oscilacao_px`, e varia com o arco.
#[test]
fn a_oscilacao_desloca_na_normal() {
    let f = com(|f| f.oscilacao_px = 6.0);
    let rumo = [0.6, 0.8];
    let mut max = 0.0f64;
    for a in 0..500 {
        let Ponto { q, ganho, .. } = f
            .no_dab(f64::from(a) * 3.1, rumo, 1.0)
            .unwrap()
            .no_ponto([2.0, -1.0]);
        assert_eq!(ganho, 1.0);
        let s = [q[0] - 2.0, q[1] + 1.0];
        assert!(
            (s[0] * rumo[0] + s[1] * rumo[1]).abs() < 1e-12,
            "fora da normal: {s:?}"
        );
        let m = s[0].hypot(s[1]);
        assert!(m <= 6.0 + 1e-12);
        max = max.max(m);
    }
    assert!(max > 2.5, "{max}");
}

/// ⭐ **O Random Span desvia cada FIO por si**: dois fios no mesmo arco desviam-se diferente, o mesmo fio
/// muda ao longo do caminho, sempre na normal e dentro de `span_px` — e sem pressão o ganho fica `1`.
#[test]
fn o_span_desvia_cada_fio_por_si() {
    let f = com(|f| {
        f.span_px = 3.0;
        f.largura_px = 2.0;
        f.span_comprimento_px = 8.0;
    });
    let desvio = |arco: f64, l: f64| {
        let p = f.no_dab(arco, [1.0, 0.0], 1.0).unwrap().no_ponto([0.0, l]);
        assert_eq!(p.ganho, 1.0);
        assert_eq!(p.q[0], 0.0, "fora da normal");
        p.q[1] - l
    };
    let mut max = 0.0f64;
    for a in 0..300 {
        let arco = f64::from(a) * 1.7;
        for k in -6..6 {
            let d = desvio(arco, f64::from(k) * 2.0 + 0.5);
            assert!(d.abs() <= 3.0 + 1e-12);
            max = max.max(d.abs());
        }
    }
    assert!(max > 1.2, "{max}");
    let distintos = (-6..6)
        .filter(|&k| {
            desvio(41.3, f64::from(k) * 2.0 + 0.5) != desvio(41.3, f64::from(k) * 2.0 + 2.5)
        })
        .count();
    assert!(distintos >= 10, "{distintos}");
    assert_ne!(desvio(41.3, 0.5), desvio(47.9, 0.5));
}

/// **A cor é do FIO e anda pelo caminho**: o mesmo fio no mesmo arco dá a mesma cor (qualquer rumo); noutro
/// arco, outra; fios diferentes, cores diferentes; sem ruído de cor, a cor intacta (ao bit).
#[test]
fn a_cor_e_do_fio() {
    let azul = [23.0f32, 101.0, 179.0];
    let f = com(|f| f.cor = [0.08, 0.1, 0.1]);
    let a = f.no_dab(10.0, [1.0, 0.0], 1.0).unwrap();
    let b = f.no_dab(900.0, [1.0, 0.0], 1.0).unwrap();
    assert_eq!(
        a.cor_do_fio(azul, 4),
        f.no_dab(10.0, [0.0, 1.0], 1.0).unwrap().cor_do_fio(azul, 4)
    );
    assert_ne!(
        a.cor_do_fio(azul, 4),
        b.cor_do_fio(azul, 4),
        "a cor anda ao longo do caminho"
    );
    let cores: Vec<[u32; 3]> = (0..10)
        .map(|k| a.cor_do_fio(azul, k).map(f32::to_bits))
        .collect();
    assert!(cores.windows(2).all(|w| w[0] != w[1]));
    let sem = com(|f| f.pressao = 1.0)
        .no_dab(10.0, [1.0, 0.0], 1.0)
        .unwrap();
    assert_eq!(
        sem.cor_do_fio(azul, 4).map(f32::to_bits),
        azul.map(f32::to_bits)
    );
}

/// ⭐ **Os fios não voltam ao neutro JUNTOS** (o defeito medido no Express Oil: 149 px a 57 % da potência): em
/// nenhum arco múltiplo do comprimento do ruído os ganhos de 12 fios ficam todos a 1.
#[test]
fn os_fios_nao_voltam_ao_neutro_juntos() {
    let f = com(|f| {
        f.pressao = 1.0;
        f.ruido_px = 150.0;
    });
    for m in 0..20 {
        let fd = f.no_dab(150.0 * f64::from(m), [1.0, 0.0], 1.0).unwrap();
        let fora = (0..12)
            .filter(|&k| (fd.no_ponto([0.0, 3.0 * f64::from(k) + 1.0]).ganho - 1.0).abs() > 0.05)
            .count();
        assert!(
            fora >= 6,
            "no arco {}: só {fora} de 12 fios fora do neutro",
            150 * m
        );
    }
}

/// ⭐ **A mão leve levanta os fios fracos**: com `levanta`, o contacto 1 (a pressão cheia) dá os ganhos de
/// sem ele, AO BIT; o contacto baixo tira o mesmo a todos — mais fios a zero (o leve mais texturado).
#[test]
fn a_mao_leve_levanta_os_fios_fracos() {
    let sem = com(|f| f.pressao = 1.0);
    let com_l = com(|f| {
        f.pressao = 1.0;
        f.levanta = 1.0;
    });
    let ganhos = |f: &Fios, contacto: f64| -> Vec<f64> {
        (0..200)
            .map(|i| {
                f.no_dab(f64::from(i) * 9.1, [1.0, 0.0], contacto)
                    .unwrap()
                    .no_ponto([0.0, f64::from(i % 13) * 3.0])
                    .ganho
            })
            .collect()
    };
    let bits = |v: Vec<f64>| v.into_iter().map(f64::to_bits).collect::<Vec<_>>();
    assert_eq!(bits(ganhos(&com_l, 1.0)), bits(ganhos(&sem, 1.0)));
    let zeros = |v: &[f64]| v.iter().filter(|&&g| g == 0.0).count();
    let (cheio, leve) = (ganhos(&com_l, 1.0), ganhos(&com_l, 0.4));
    assert!(
        zeros(&leve) > zeros(&cheio) + 4,
        "{} × {}",
        zeros(&leve),
        zeros(&cheio)
    );
    let media = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    assert!(
        media(&leve) < media(&cheio) - 0.5,
        "{} × {}",
        media(&leve),
        media(&cheio)
    );
    assert!(
        com(|f| f.levanta = 0.5)
            .no_dab(0.0, [1.0, 0.0], 0.5)
            .is_some()
    );
}

/// ⭐ **Entre que dois fios** (W3b): o ponto cai entre os CENTROS de dois fios vizinhos, e a posição anda
/// contínua de 0 a 1 entre eles — também com a onda a mexer as fronteiras (a onda MÁXIMA, rumo vertical).
#[test]
fn entre_fios_anda_continuo_de_centro_a_centro() {
    for onda in [0.0, ONDA_MAX] {
        let f = com(|f| {
            f.pressao = 1.0;
            f.onda = onda;
            f.largura_px = 2.0;
        });
        let fd = f.no_dab(321.0, [1.0, 0.0], 1.0).unwrap();
        let mut antes = fd.entre_fios([0.0, -6.0]);
        for i in 1..=1200 {
            let d = [0.0, -6.0 + f64::from(i) * 0.01];
            let agora = fd.entre_fios(d);
            let ([a, b], t) = agora;
            assert_eq!(b, a + 1, "{onda} {d:?}");
            assert!((0.0..=1.0).contains(&t));
            if agora.0 == antes.0 {
                assert!(
                    t >= antes.1 && t - antes.1 < 0.05,
                    "{onda} {d:?}: {antes:?} → {agora:?}"
                );
            } else {
                assert_eq!(a, antes.0[1], "{onda} {d:?}: salta um par");
                assert!(
                    antes.1 > 0.95 && t < 0.05,
                    "{onda} {d:?}: {antes:?} → {agora:?}"
                );
            }
            antes = agora;
        }
    }
    // CONTROLO: sem onda, o centro do fio 0 é meia largura acima do eixo (a 1 px na normal [0, 1]).
    let fd = com(|f| f.pressao = 1.0)
        .no_dab(0.0, [1.0, 0.0], 1.0)
        .unwrap();
    let largura = Fios::NEUTRO.largura_px;
    assert_eq!(fd.entre_fios([0.0, 0.5 * largura]), ([0, 1], 0.0));
    assert_eq!(fd.entre_fios([0.0, 0.0]), ([-1, 0], 0.5));
}

/// ⭐ **O VEIO** (J4 óleo, `Knob::BristleStreak`): sem ele nada muda (o neutro não liga a lei, o veio de todo fio é 0);
/// com ele cada fio passa a MAIOR parte do caminho na sua cor e, a troços raros com princípio e fim (dezenas de px), pesa
/// (a luminosidade desce até `escuro`) ou falta (sobe até `claro`). CONTROLO: o ruído de cor da luminosidade tinge o
/// fio no caminho inteiro (nenhum troço a zero).
#[test]
fn o_veio_e_raro_tem_principio_e_fim_e_os_dois_sinais() {
    assert!(Fios::NEUTRO.no_dab(37.0, [1.0, 0.0], 1.0).is_none());
    assert!(!Fios::NEUTRO.tem_cor());
    let veio = com(|f| {
        f.veio = [0.6, 0.3];
        f.veio_px = 800.0;
        f.veio_corte = 0.3;
    });
    assert!(veio.tem_cor() && veio.no_dab(37.0, [1.0, 0.0], 1.0).is_some());
    let (mut zero, mut total, mut escuros, mut claros) = (0usize, 0usize, 0usize, 0usize);
    let mut corridas: Vec<usize> = Vec::new();
    let (mut min, mut max) = (0.0f64, 0.0f64);
    for fio in -40..40 {
        let mut corrida = 0;
        for i in 0..3000 {
            let v = veio.no_dab(f64::from(i), [1.0, 0.0], 1.0).unwrap().veio_do_fio(fio);
            assert_eq!(v.to_bits(), veio.no_dab(f64::from(i), [1.0, 0.0], 1.0).unwrap().veio_do_fio(fio).to_bits());
            total += 1;
            (min, max) = (min.min(v), max.max(v));
            if v == 0.0 {
                zero += 1;
                if corrida > 0 {
                    corridas.push(corrida);
                }
                corrida = 0;
            } else {
                corrida += 1;
                if v < 0.0 { escuros += 1 } else { claros += 1 }
            }
        }
    }
    let livre = zero as f64 / total as f64;
    let media = corridas.iter().sum::<usize>() as f64 / corridas.len().max(1) as f64;
    println!("[veio] livre {livre:.3} · escuros {escuros} · claros {claros} · troços {} de {media:.0} px · [{min:.2}, {max:.2}]", corridas.len());
    assert!((0.6..0.97).contains(&livre), "o veio não é raro: {livre:.3} do caminho livre");
    assert!(escuros > 0 && claros > 0, "um sinal falta: {escuros} escuros, {claros} claros");
    assert!(media >= 20.0, "troços de {media:.1} px: o veio pisca em vez de correr");
    assert!(min <= -0.59 && max >= 0.29, "o troço não chega à cor dele: [{min}, {max}]");
    // CONTROLO: o ruído de cor da luminosidade não tem troços livres.
    let cor = com(|f| f.cor = [0.0, 0.0, 0.1]).no_dab(100.0, [1.0, 0.0], 1.0).unwrap();
    let ruidos = (-40..40).filter(|&k| cor.cor_do_fio([23.0, 101.0, 179.0], k) != [23.0, 101.0, 179.0]).count();
    assert!(ruidos > 70, "o controlo não mede: {ruidos} de 80 fios tingidos");
}
