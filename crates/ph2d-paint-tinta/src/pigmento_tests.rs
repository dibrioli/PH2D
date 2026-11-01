use super::*;
use crate::oklab::oklab_para_srgb;
use crate::transfere::transfere;

const AZUL: [f32; 3] = [40.0, 90.0, 220.0];
const AMARELO: [f32; 3] = [245.0, 205.0, 25.0];

fn tinta(c: [f32; 3]) -> Tinta {
    Tinta::de_srgb(c.map(|x| x / 255.0), 0.3)
}

fn fatia(c: [f32; 3], v: f32, p: f32) -> (Soma, Espectro) {
    let t = tinta(c);
    (Soma::de(t, v), Espectro::de(t, v, p))
}

fn junta(a: (Soma, Espectro), b: (Soma, Espectro)) -> (Soma, Espectro) {
    (a.0.junta(b.0), a.1.junta(b.1))
}

fn de(a: Lab, b: Lab) -> f32 {
    let d: [f32; 3] = core::array::from_fn(|k| a[k] - b[k]);
    ((d[0] * d[0] + d[1] * d[1]) + d[2] * d[2]).sqrt()
}

fn amostras() -> Vec<[f32; 3]> {
    let mut v = vec![
        AZUL,
        AMARELO,
        [0.0; 3],
        [255.0; 3],
        [255.0, 0.0, 0.0],
        [0.0, 0.0, 255.0],
    ];
    for r in (0..=255).step_by(51) {
        for g in (0..=255).step_by(51) {
            for b in (0..=255).step_by(51) {
                v.push([r as f32, g as f32, b as f32]);
            }
        }
    }
    v
}

/// (a) `p = 0` ⇒ a cor é a de `Soma::cor` AO BIT, também depois de juntar e transferir.
#[test]
fn pigmento_p0_e_o_oklab_de_hoje_ao_bit() {
    let cs = amostras();
    for w in cs.windows(3) {
        let (mut a, mut b) = (
            junta(fatia(w[0], 1.3, 0.0), fatia(w[1], 0.7, 0.0)),
            fatia(w[2], 2.1, 0.0),
        );
        let _ = transfere_pigmento((&mut a.0, &mut a.1), (&mut b.0, &mut b.1), 0.37);
        for s in [&a, &b] {
            let (x, y) = (cor(&s.0, &s.1).unwrap(), s.0.cor().unwrap());
            assert_eq!(x.lab.map(f32::to_bits), y.lab.map(f32::to_bits));
            assert_eq!(x.transparencia.to_bits(), y.transparencia.to_bits());
        }
    }
    let (s, _) = fatia(AZUL, 1.0, 0.0);
    assert_eq!(
        cor(&s, &Espectro::default()),
        s.cor(),
        "lado espectral vazio"
    );
}

/// (b) identidade: `mix(c,c)` a `p = 1`. Sem o resíduo (o WGM cru) é a ida-volta da base; com ele é exacta sem atalho.
#[test]
fn pigmento_identidade_mix_c_c() {
    let (mut cru, mut com) = (0.0f32, 0.0f32);
    for c in amostras() {
        let m = junta(fatia(c, 1.0, 1.0), fatia(c, 0.6, 1.0));
        let o = tinta(c).lab;
        cru = cru.max(de(oklab_wgm_cru(&m.0, &m.1).unwrap(), o));
        com = com.max(de(cor(&m.0, &m.1).unwrap().lab, o));
    }
    eprintln!(
        "identidade ΔE×100: cru {:.4} · com resíduo {:.6}",
        cru * 100.0,
        com * 100.0
    );
    assert!(cru * 100.0 <= 0.39 + 0.2, "WGM cru {cru}");
    // Medido 31/10: cru 0,3910 · com resíduo 2,4e-6 (o arredondamento `f32` de `Σv·x / Σv`).
    assert!(com <= 1e-5, "com resíduo {com}");
}

/// (c) associativa: 3 tintas juntas em ordens diferentes.
#[test]
fn pigmento_associativo_em_qualquer_ordem() {
    let (a, b, c) = (
        fatia(AZUL, 1.0, 1.0),
        fatia(AMARELO, 0.5, 0.3),
        fatia([200.0, 30.0, 60.0], 0.8, 0.7),
    );
    let x = cor(&junta(junta(a, b), c).0, &junta(junta(a, b), c).1)
        .unwrap()
        .lab;
    let mut pior = 0.0f32;
    for s in [
        junta(a, junta(b, c)),
        junta(junta(c, a), b),
        junta(junta(b, c), a),
        junta(c, junta(b, a)),
    ] {
        pior = pior.max(de(cor(&s.0, &s.1).unwrap().lab, x));
    }
    eprintln!("associatividade ΔE {pior:e}");
    assert!(pior <= 1e-5, "{pior}");
}

/// (d) azul + amarelo 50/50: a `p = 1` dá VERDE; a `p = 0` o cinza Oklab de hoje.
#[test]
fn pigmento_azul_e_amarelo_dao_verde() {
    let m1 = junta(fatia(AZUL, 1.0, 1.0), fatia(AMARELO, 1.0, 1.0));
    let m0 = junta(fatia(AZUL, 1.0, 0.0), fatia(AMARELO, 1.0, 0.0));
    let s = |m: &(Soma, Espectro)| {
        oklab_para_srgb(cor(&m.0, &m.1).unwrap().lab).map(|x| (x * 255.0).round())
    };
    let (v, g) = (s(&m1), s(&m0));
    let mh = junta(fatia(AZUL, 1.0, 0.5), fatia(AMARELO, 1.0, 0.5));
    eprintln!("azul+amarelo p=1 {v:?} · p=0,5 {:?} · p=0 {g:?}", s(&mh));
    assert!(v[1] > v[0] && v[1] > v[2], "não é verde: {v:?}");
    assert_eq!(cor(&m0.0, &m0.1), m0.0.cor());
}

/// (e) deriva: 20× misturar a fatia com a mesma tinta (e devolver metade) não muda a cor em `f32`.
#[test]
fn pigmento_sem_deriva_20x() {
    let mut pior = 0.0f32;
    for c in [AZUL, AMARELO, [0.0; 3], [128.0, 64.0, 200.0]] {
        let mut s = fatia(c, 1.0, 1.0);
        let o0 = cor(&s.0, &s.1).unwrap().lab;
        for _ in 0..20 {
            s = junta(s, fatia(c, 1.0, 1.0));
            let mut fora = (Soma::default(), Espectro::default());
            let _ = transfere_pigmento((&mut s.0, &mut s.1), (&mut fora.0, &mut fora.1), 0.5);
            pior = pior.max(de(cor(&s.0, &s.1).unwrap().lab, o0));
        }
    }
    let o = |x: f32| (x * 255.0).round();
    eprintln!("deriva 20× ΔE×100 {:.6}", pior * 100.0);
    assert!(pior <= 2e-6 && o(pior) == 0.0, "{pior}");
}

/// (f) canais 0 e o preto: nunca NaN; o piso das bandas não muda o preto puro com o resíduo.
#[test]
fn pigmento_canais_zero_nunca_nan() {
    let preto = fatia([0.0; 3], 1.0, 1.0);
    let cp = cor(&preto.0, &preto.1).unwrap().lab;
    let cru = oklab_wgm_cru(&preto.0, &preto.1).unwrap();
    eprintln!(
        "preto: com resíduo ΔE {:e} · WGM cru L {:.5} (sRGB {:?})",
        de(cp, tinta([0.0; 3]).lab),
        cru[0],
        oklab_para_srgb(cru).map(|x| x * 255.0)
    );
    for c in amostras() {
        for p in [0.0, 0.5, 1.0] {
            let m = junta(fatia(c, 1.0, p), fatia([0.0; 3], 0.4, p));
            let x = cor(&m.0, &m.1).unwrap().lab;
            assert!(x.iter().all(|v| v.is_finite()), "{c:?} {p}: {x:?}");
        }
    }
    assert!(de(cp, tinta([0.0; 3]).lab) <= 2e-6);
    assert_eq!(cor(&Soma::default(), &Espectro::default()), None);
}

/// A soma e o lado espectral movem a MESMA fracção: transferir e voltar a juntar conserva tudo.
#[test]
fn pigmento_transfere_conserva() {
    let mut a = junta(fatia(AZUL, 1.0, 1.0), fatia(AMARELO, 0.5, 0.2));
    let total = a;
    let mut b = (Soma::default(), Espectro::default());
    let (ms, me) = transfere_pigmento((&mut a.0, &mut a.1), (&mut b.0, &mut b.1), 0.3);
    assert_eq!((ms, me), (b.0, b.1));
    let s = junta(a, b);
    for i in 0..N_WGM {
        assert!(
            (s.1.ln_r[i] - total.1.ln_r[i]).abs() <= 4.0 * f32::EPSILON * total.1.ln_r[i].abs()
        );
    }
    assert!((s.1.p - total.1.p).abs() <= 4.0 * f32::EPSILON * total.1.p);
    let _ = transfere(&mut Soma::default(), &mut Soma::default(), 0.5);
    assert_eq!(core::mem::size_of::<Espectro>(), 56);
}
