//! `cada_vicio_e_achado_onde_esta`, `o_sprite_limpo_nao_tem_aviso` e o tempo ao vivo.

use crate::fixture::{Tela, bola_limpa, caixa_limpa, vicios_p13};
use ph2d_pixel_lint::{Aviso, Regra, Regras, lint};
use std::collections::BTreeMap;
use std::time::Instant;

fn por_regra(av: &[Aviso]) -> BTreeMap<Regra, Vec<Vec<[u32; 2]>>> {
    let mut m: BTreeMap<Regra, Vec<Vec<[u32; 2]>>> = BTreeMap::new();
    for a in av {
        m.entry(a.regra).or_default().push(a.celulas.clone());
    }
    m
}

fn corre(t: &Tela, regras: &Regras) -> Vec<Aviso> {
    let s = t.sprite();
    lint(&s, s.frames()[0].id, regras)
}

#[test]
fn cada_vicio_e_achado_onde_esta() {
    let (t, gabarito) = vicios_p13();
    // 8 cores na fixture: com 7, a mais rara (a do órfão) passa do limite.
    let regras = Regras {
        limite_cores: 7,
        ..Regras::default()
    };
    let achado = por_regra(&corre(&t, &regras));
    for r in Regra::TODAS {
        assert_eq!(
            achado.get(&r),
            gabarito.get(&r),
            "{r:?}: achado ≠ gabarito (tudo: {achado:?})"
        );
    }
    // Com o limite de fábrica (32), as 8 cores não são «a mais».
    let fabrica = por_regra(&corre(&t, &Regras::default()));
    assert!(!fabrica.contains_key(&Regra::CoresAMais), "{fabrica:?}");
}

#[test]
fn o_sprite_limpo_nao_tem_aviso() {
    for (nome, t) in [("caixa", caixa_limpa()), ("bola", bola_limpa())] {
        let av = corre(&t, &Regras::default());
        assert!(av.is_empty(), "{nome}: {av:?}");
    }
}

#[test]
fn desligar_a_regra_cala_so_ela() {
    let (t, _) = vicios_p13();
    let mut regras = Regras {
        limite_cores: 7,
        ..Regras::default()
    };
    regras.faixa = false;
    let achado = por_regra(&corre(&t, &regras));
    assert!(!achado.contains_key(&Regra::Faixa));
    assert!(achado.contains_key(&Regra::Almofada) && achado.contains_key(&Regra::Orfao));
}

/// Um cel 256×256 com estrutura (blocos de 4 px de 12 cores, linhas, ruído): o mínimo de 10.
#[test]
fn o_lint_de_256_cabe_ao_vivo() {
    let mut t = Tela::new(256, 256);
    let mut seed: u32 = 0x1234_5678;
    let mut r = || {
        seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        seed >> 16
    };
    let pal = &ph2d_pixel_core::palette::DEFAULT;
    for by in 0..64 {
        for bx in 0..64 {
            let c = pal[(r() % 12) as usize];
            for y in 0..4 {
                for x in 0..4 {
                    t.p(bx * 4 + x, by * 4 + y, c);
                }
            }
        }
    }
    for _ in 0..2000 {
        let (x, y) = (r() % 256, r() % 256);
        let c = if r() % 3 == 0 {
            [0; 4]
        } else {
            pal[(r() % 32) as usize]
        };
        t.p(x, y, c);
    }
    let _ = t.cor(0, 0);
    let s = t.sprite();
    let f = s.frames()[0].id;
    let mut melhor = f64::MAX;
    let mut n = 0;
    for _ in 0..10 {
        let t0 = Instant::now();
        n = lint(&s, f, &Regras::default()).len();
        melhor = melhor.min(t0.elapsed().as_secs_f64() * 1e3);
    }
    eprintln!("lint 256×256: mínimo de 10 = {melhor:.2} ms ({n} avisos)");
    assert!(n > 0);
}
