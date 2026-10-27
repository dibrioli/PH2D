//! Gates do material no sombreado (W2 E3-B, filho de [`super`]): cada lóbulo do Principled pela lei dele,
//! a regra de combinação do cartão e a porta do grid. O neutro bit a bit vive em
//! `oleo_real_tests::o_material_neutro_e_bit_identico_a_de_antes`.

use super::super::*;
use super::*;

const W: usize = 64;

fn grade(m: impl Fn(usize, usize) -> f32) -> Grid {
    let mut g = Grid::new(W, W);
    for y in 1..=W {
        for x in 1..=W {
            g.sett[x + y * g.s] = m(x, y);
        }
    }
    g
}

fn campo(g: &Grid, o: &OleoReal) -> CampoAltura {
    CampoAltura::de(&[(g, true)], o, 1, 1, 1, W, W)
}

/// Estrias verticais (o relevo sobe e desce ao longo de x) sobre uma tinta alta.
fn estrias() -> Grid {
    grade(|x, _| 4000.0 + 1500.0 * (x as f32 * 0.9).sin())
}

fn com(k: MaterialCartao) -> OleoReal {
    OleoReal {
        material: k,
        ..OleoReal::default()
    }
}

fn mat(f: impl FnOnce(&mut MaterialCelula)) -> MaterialCelula {
    let mut m = MaterialCelula::NEUTRO;
    f(&mut m);
    m
}

const VERMELHO: [f64; 3] = [215.0, 25.0, 30.0];

/// A média do brilho somado numa janela de células.
fn brilho_medio(o: &OleoReal, c: &CampoAltura, m: &MaterialCelula, a: [f64; 3]) -> [f64; 3] {
    let mut s = [0.0; 3];
    let mut n = 0.0;
    for y in 20..44 {
        for x in 20..44 {
            let t = termos_com(o, c, x, y, m, a);
            for k in 0..3 {
                s[k] += t.espec[k];
            }
            n += 1.0;
        }
    }
    s.map(|v| v / n)
}

/// ⭐ A regra do cartão: os aditivos tomam o MAIOR (o cartão põe verniz em toda a tinta; a célula metálica
/// continua metálica), os de neutro ≠ 0 multiplicam-se.
#[test]
fn o_cartao_combina_pelo_maior_e_pelo_produto() {
    let cel = mat(|m| {
        m.metal = 1.0;
        m.rugosidade = 0.5;
        m.verniz = 0.2;
    });
    let k = MaterialCartao {
        verniz: 0.7,
        rugosidade: 2.0,
        especular: 1.5,
        ..MaterialCartao::NEUTRO
    };
    let e = efectivo(&cel, &k);
    assert_eq!(e.metal, 1.0, "o cartão a 0 não tira o metal da célula");
    assert_eq!(e.verniz, f64::from(0.7f32).max(0.7), "o maior dos dois");
    assert_eq!(e.rugosidade, 1.0, "0,5 × 2");
    assert_eq!(e.especular, 0.75, "0,5 × 1,5");
    let n = efectivo(&MaterialCelula::NEUTRO, &MaterialCartao::NEUTRO);
    assert_eq!((n.metal, n.rugosidade, n.especular), (0.0, 1.0, 0.5));
}

/// ⭐ Roughness MULTIPLICA a do Gloss: o material a `0,5` é a afinação com metade da rugosidade, bit a bit.
#[test]
fn a_rugosidade_do_material_multiplica_a_do_gloss() {
    let g = estrias();
    let o = OleoReal::default();
    let c = campo(&g, &o);
    let metade = OleoReal {
        af: Afinacao {
            rugosidade: o.af.rugosidade * 0.5,
            ..o.af
        },
        ..o
    };
    let m = mat(|m| m.rugosidade = 0.5);
    for (x, y) in [(20, 20), (33, 31), (40, 12)] {
        let a = termos_com(&o, &c, x, y, &m, [0.5; 3]);
        let b = termos(&metade, &c, x, y);
        assert_eq!(a.espec, b.espec, "({x},{y})");
    }
}

/// ⭐ Specular `0` = IOR 1: nenhum reflexo, nem na borda (o Principled a `0` dá preto — `oraculo` §E3).
#[test]
fn o_specular_zero_nao_reflecte_nada() {
    let g = estrias();
    for ambiente in Ambiente::TODOS {
        let o = OleoReal {
            ambiente,
            ..OleoReal::default()
        };
        let c = campo(&g, &o);
        let m = mat(|m| m.especular = 0.0);
        assert_eq!(brilho_medio(&o, &c, &m, [0.5; 3]), [0.0; 3], "{ambiente:?}");
        let dobro = mat(|m| m.especular = 1.0);
        let b0 = brilho_medio(&o, &c, &MaterialCelula::NEUTRO, [0.5; 3]);
        let b2 = brilho_medio(&o, &c, &dobro, [0.5; 3]);
        assert!(b2[0] > 1.5 * b0[0], "{ambiente:?}: {b2:?} × {b0:?}");
    }
}

/// ⭐ METAL: sem difuso, o reflexo com a COR da tinta (F0 = a cor) — o vermelho metálico reflecte vermelho e
/// a tinta lisa metálica não é a cor dela (é o que o ambiente lhe dá).
#[test]
fn o_metal_tira_o_difuso_e_reflecte_a_cor() {
    let g = estrias();
    let o = OleoReal::default();
    let c = campo(&g, &o);
    let a = VERMELHO.map(crate::colorops::transfer::srgb255_to_linear);
    let m = mat(|m| m.metal = 1.0);
    let b = brilho_medio(&o, &c, &m, a);
    assert!(b[0] > 5.0 * b[1], "o reflexo é vermelho: {b:?}");
    let t = termos_com(&o, &c, 30, 30, &m, a);
    assert_eq!(t.metal, 1.0);
    let b_d = brilho_medio(&o, &c, &MaterialCelula::NEUTRO, a);
    assert!(b[0] > 4.0 * b_d[0], "F0 0,68 × 0,04: {b:?} × {b_d:?}");
    let out = ilumina_com(&o, &c, 30, 30, VERMELHO, 1.0, &m);
    let neutro = ilumina(&o, &c, 30, 30, VERMELHO, 1.0);
    assert!(
        out[1] < neutro[1] + 0.5,
        "sem difuso o verde não sobe: {out:?} × {neutro:?}"
    );
    // Reflectivity escala o F0 do metal.
    let meio = com(MaterialCartao {
        refletividade: 0.5,
        ..MaterialCartao::NEUTRO
    });
    let bm = brilho_medio(&meio, &c, &m, a);
    assert!(bm[0] < 0.75 * b[0], "{bm:?} × {b:?}");
}

/// ⭐ Tint: o reflexo dieléctrico toma a cor da tinta (o F90 continua branco).
#[test]
fn o_tint_tinge_o_reflexo_com_a_cor_da_tinta() {
    let g = estrias();
    let o = OleoReal::default();
    let c = campo(&g, &o);
    let a = VERMELHO.map(crate::colorops::transfer::srgb255_to_linear);
    let b0 = brilho_medio(&o, &c, &MaterialCelula::NEUTRO, a);
    let b1 = brilho_medio(&o, &c, &mat(|m| m.tinta_reflexo = 1.0), a);
    assert!(
        (b0[0] - b0[1]).abs() < 0.2 * b0[0],
        "sem tint o reflexo é branco: {b0:?}"
    );
    assert!(b1[0] > 3.0 * b1[1], "com tint é vermelho: {b1:?}");
}

/// ⭐ Verniz: um lobo liso por cima (mais brilho) e a camada de baixo recebe o que o Fresnel dele deixa.
#[test]
fn o_verniz_poe_brilho_e_tapa_um_pouco_a_base() {
    let g = estrias();
    let o = OleoReal::default();
    let c = campo(&g, &o);
    let v = mat(|m| m.verniz = 1.0);
    let b0 = brilho_medio(&o, &c, &MaterialCelula::NEUTRO, [0.5; 3]);
    let b1 = brilho_medio(&o, &c, &v, [0.5; 3]);
    assert!(b1[0] > 1.3 * b0[0], "{b1:?} × {b0:?}");
    let t = termos_com(&o, &c, 30, 30, &v, [0.5; 3]);
    assert!(t.sob_verniz < 1.0 && t.sob_verniz > 0.9, "{}", t.sob_verniz);
    // A rugosidade do verniz espalha o lobo da LÂMPADA: menos pico (só a lâmpada: força do ambiente 0).
    let o = OleoReal {
        ambiente: Ambiente::Classico,
        forca_ambiente: 0.0,
        ..o
    };
    let rugoso = mat(|m| {
        m.verniz = 1.0;
        m.verniz_rugosidade = 0.5;
    });
    // O contraste (desvio ÷ média) do brilho do verniz cai com a rugosidade dele: o lobo largo apaga o relevo.
    let contraste = |m: &MaterialCelula| {
        let v: Vec<f64> = (10..54)
            .flat_map(|y| (10..54).map(move |x| (x, y)))
            .map(|(x, y)| termos_com(&o, &c, x, y, m, [0.5; 3]).espec[0])
            .collect();
        let media = v.iter().sum::<f64>() / v.len() as f64;
        let var = v.iter().map(|x| (x - media).powi(2)).sum::<f64>() / v.len() as f64;
        var.sqrt() / media
    };
    let (liso, aspero) = (contraste(&v), contraste(&rugoso));
    assert!(aspero < liso, "{liso} × {aspero}");
}

/// ⭐ Sheen: cresce onde a normal foge da vista (o Principled: sheen/e 0,031 de frente → 0,108 a 1−nz 0,6).
#[test]
fn o_veludo_cresce_onde_a_normal_foge_da_vista() {
    assert!((veludo(1.0, 1.0, 1.0) - VELUDO_K0).abs() < 1e-12);
    let encosta = veludo(1.0, 0.4, 1.0);
    assert!((encosta / 0.108 - 1.0).abs() < 0.15, "{encosta}");
    assert_eq!(veludo(0.0, 0.3, 1.0), 0.0);
}

/// ⭐ Anisotropia: a estria corre AO LONGO do sulco (perpendicular ao gradiente) e o brilho alonga-se nela:
/// com estrias verticais a direcção é `(0, ±1)`, e o lobo anisotrópico muda o brilho.
#[test]
fn a_anisotropia_corre_ao_longo_da_estria() {
    let g = estrias();
    let o = OleoReal::default();
    let c = campo(&g, &o);
    let e = estria(&c, 30, 30).expect("há relevo");
    assert!(
        e[0].abs() < 1e-6 && (e[1].abs() - 1.0).abs() < 1e-6,
        "{e:?}"
    );
    let plano = grade(|_, _| 3000.0);
    assert_eq!(estria(&campo(&plano, &o), 30, 30), None);
    let b0 = brilho_medio(&o, &c, &MaterialCelula::NEUTRO, [0.5; 3]);
    let b1 = brilho_medio(&o, &c, &mat(|m| m.anisotropia = 0.8), [0.5; 3]);
    assert!((b1[0] - b0[0]).abs() > 1e-4 * b0[0], "{b1:?} × {b0:?}");
}

/// ⭐ Translucidez: a película FINA deixa passar e volta do papel; a grossa não (bit a bit a de sem).
#[test]
fn a_pelicula_fina_deixa_passar_e_a_grossa_nao() {
    let o = OleoReal {
        brilho: 0.0,
        ..OleoReal::default()
    };
    let m = mat(|m| m.translucidez = 1.0);
    let grossa = grade(|_, _| 60_000.0);
    let cg = campo(&grossa, &o);
    assert!(passagem(&cg, 32, 32) < 1e-6);
    assert_eq!(
        ilumina_com(&o, &cg, 32, 32, VERMELHO, 1.0, &m).map(f64::round),
        ilumina(&o, &cg, 32, 32, VERMELHO, 1.0).map(f64::round)
    );
    let fina = grade(|_, _| 150.0);
    let cf = campo(&fina, &o);
    let p = passagem(&cf, 32, 32);
    assert!(p > 0.5, "{p}");
    let t = termos_com(&o, &cf, 32, 32, &m, [0.5; 3]);
    assert!((t.translucida - p).abs() < 1e-12);
    let out = ilumina_com(&o, &cf, 32, 32, VERMELHO, 1.0, &m);
    let sem = ilumina(&o, &cf, 32, 32, VERMELHO, 1.0);
    assert_ne!(out, sem);
}

/// ⭐ Strength e Scale do cartão: o reflexo do ambiente escala com a Strength; Scale ≠ 1 muda-o; o
/// Reflection Map põe reflexo no Clássico.
#[test]
fn strength_scale_e_reflection_map_mexem_no_reflexo_do_ambiente() {
    let g = estrias();
    let base = OleoReal {
        forca_ambiente: 1.0,
        ..OleoReal::default()
    };
    let c = campo(&g, &base);
    let b1 = brilho_medio(&base, &c, &MaterialCelula::NEUTRO, [0.5; 3]);
    let forte = OleoReal {
        material: MaterialCartao {
            forca_reflexo: 2.0,
            ..MaterialCartao::NEUTRO
        },
        ..base
    };
    let b2 = brilho_medio(&forte, &c, &MaterialCelula::NEUTRO, [0.5; 3]);
    assert!(
        (b2[0] / b1[0] - 2.0).abs() < 1e-9,
        "a lâmpada é 0 com força 1: {b2:?} × {b1:?}"
    );
    let escala = OleoReal {
        material: MaterialCartao {
            escala_mapa: 3.0,
            ..MaterialCartao::NEUTRO
        },
        ..base
    };
    assert_ne!(
        brilho_medio(&escala, &c, &MaterialCelula::NEUTRO, [0.5; 3]),
        b1
    );
    let classico = OleoReal {
        ambiente: Ambiente::Classico,
        sombra: 0.0,
        ..base
    };
    assert_eq!(
        brilho_medio(&classico, &c, &MaterialCelula::NEUTRO, [0.5; 3]),
        [0.0; 3]
    );
    let com_mapa = OleoReal {
        material: MaterialCartao {
            mapa_reflexo: Some(Ambiente::Estudio),
            ..MaterialCartao::NEUTRO
        },
        ..classico
    };
    assert!(brilho_medio(&com_mapa, &c, &MaterialCelula::NEUTRO, [0.5; 3])[0] > 0.0);
}

/// ⭐ A porta do grid: o material que a camada pousou chega ao sombreado pelo [`CampoAltura`]; sem material
/// em camada nenhuma o campo não o guarda (NEUTRO, o caminho de antes).
#[test]
fn o_material_da_camada_chega_ao_sombreado() {
    let o = OleoReal::default();
    let mut g = grade(|_, _| 4000.0);
    let c0 = campo(&g, &o);
    assert_eq!(c0.material_em(30, 30), MaterialCelula::NEUTRO);
    let metal = mat(|m| m.metal = 1.0);
    g.material = vec![MaterialCelula::NEUTRO; g.sett.len()];
    let i = 30 + 30 * g.s;
    g.material[i] = metal;
    let c = campo(&g, &o);
    assert!(
        c.material_em(30, 30).metal > 0.99,
        "{:?}",
        c.material_em(30, 30)
    );
    assert_eq!(c.material_em(10, 10), MaterialCelula::NEUTRO);
    assert_ne!(
        ilumina(&o, &c, 30, 30, VERMELHO, 1.0),
        ilumina(&o, &c0, 30, 30, VERMELHO, 1.0)
    );
}
