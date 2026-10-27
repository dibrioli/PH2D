//! Gates do material no sombreado (W2 E3-B, filho de [`super`]): cada lóbulo do Principled pela lei dele,
//! a regra de combinação do cartão e a porta do grid. O neutro bit a bit vive em
//! `oleo_real_tests::o_material_neutro_e_bit_identico_a_de_antes`.

use super::super::*;
use super::*;
use crate::material::efectivo;

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

/// ⭐ A regra do cartão: os de neutro 0 tomam o MAIOR (o cartão põe verniz em toda a tinta; a célula metálica
/// continua metálica); os outros `cartão × célula / neutro` — célula NEUTRA ⇒ o cartão, cartão no padrão ⇒ a célula.
#[test]
fn o_cartao_combina_pelo_maior_e_pela_razao_do_neutro() {
    let cel = mat(|m| {
        m.metal = 1.0;
        m.rugosidade = 0.25;
        m.verniz = 0.2;
        m.subsuperficie = 0.3;
    });
    let k = MaterialCartao {
        verniz: 0.7,
        rugosidade: 0.56,
        nivel_ior: 0.75,
        ..MaterialCartao::NEUTRO
    };
    let e = efectivo(&cel, &k);
    assert_eq!(e.metal, 1.0, "o cartão a 0 não tira o metal da célula");
    assert_eq!(e.verniz, 0.7, "o maior dos dois");
    assert_eq!(e.subsuperficie, f64::from(0.3f32));
    assert_eq!(e.rugosidade, 0.28, "0,56 × 0,25 / 0,5");
    assert_eq!(e.nivel_ior, 0.75, "0,75 × 0,5 / 0,5");
    // Célula NEUTRA ⇒ o cartão.
    let so_cartao = efectivo(&MaterialCelula::NEUTRO, &k);
    assert_eq!((so_cartao.rugosidade, so_cartao.nivel_ior), (0.56, 0.75));
    let n = efectivo(&MaterialCelula::NEUTRO, &MaterialCartao::NEUTRO);
    assert_eq!(
        (n.metal, n.rugosidade, n.nivel_ior, n.subsuperficie),
        (0.0, RUGOSIDADE_PADRAO, 0.5, 0.0)
    );
}

/// ⭐⭐ **A Roughness do cartão no PADRÃO é a imagem de antes** (W3 M0): a lei de antes era `0,28 × m` (a
/// `Afinacao::rugosidade` × o `roughnessMul` m do pincel × o cartão `1`); hoje a célula guarda `0,5·m` e o cartão
/// `0,28`. A conta nova dá o MESMO f64 bit a bit para os `m` dos presets (e o neutro), e o termo do brilho também.
#[test]
fn o_roughness_do_cartao_no_padrao_e_a_imagem_de_antes() {
    for m in [0.0f64, 0.4, 0.5, 0.8, 1.0, 1.4, 2.0, 0.37, 1.93] {
        let antes = RUGOSIDADE_PADRAO * (f64::from(m as f32) * 1.0);
        let cel = mat(|c| c.rugosidade = (0.5 * m) as f32);
        let agora = efectivo(&cel, &MaterialCartao::NEUTRO).rugosidade;
        assert_eq!(agora.to_bits(), antes.to_bits(), "m = {m}");
    }
    // E no sombreado: a Roughness da célula a `0,25` = o cartão a `0,14` com a célula neutra, bit a bit.
    let g = estrias();
    let o = OleoReal::default();
    let c = campo(&g, &o);
    let cartao = com(MaterialCartao {
        rugosidade: 0.14,
        ..MaterialCartao::NEUTRO
    });
    let m = mat(|m| m.rugosidade = 0.25);
    for (x, y) in [(20, 20), (33, 31), (40, 12)] {
        let a = termos_com(&o, &c, x, y, &m, [0.5; 3]);
        let b = termos(&cartao, &c, x, y);
        assert_eq!(a.espec, b.espec, "({x},{y})");
    }
}

/// ⭐ O leitor tolerante do cartão de antes da W3: os multiplicadores viram valores absolutos pelo padrão, a
/// Translucency vira Subsurface, e os campos que saíram não mexem em nada.
#[test]
fn o_cartao_antigo_converte_se_pelo_padrao() {
    let mut k = MaterialCartao::NEUTRO;
    for (nome, v) in [
        ("rugosidade", 1.0),
        ("especular", 1.0),
        ("verniz_rugosidade", 1.0),
        ("refletividade", 2.0),
        ("anisotropia", 0.5),
        ("mapa_reflexo", 2.0),
    ] {
        k.poe_campo_antigo(nome, v);
    }
    assert_eq!(
        k,
        MaterialCartao::NEUTRO,
        "os neutros de antes = o padrão de hoje"
    );
    k.poe_campo_antigo("rugosidade", 2.0);
    k.poe_campo_antigo("translucidez", 0.4);
    assert_eq!((k.rugosidade, k.subsuperficie), (0.56, 0.4));
}

/// ⭐ IOR Level `0` = IOR 1: nenhum reflexo, nem na borda (o Principled a `0` dá preto — `oraculo` §E3).
#[test]
fn o_ior_level_zero_nao_reflecte_nada() {
    let g = estrias();
    for ambiente in Ambiente::TODOS {
        let o = OleoReal {
            ambiente,
            ..OleoReal::default()
        };
        let c = campo(&g, &o);
        let m = mat(|m| m.nivel_ior = 0.0);
        assert_eq!(brilho_medio(&o, &c, &m, [0.5; 3]), [0.0; 3], "{ambiente:?}");
        let dobro = mat(|m| m.nivel_ior = 1.0);
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

/// Um muro alto de tinta vermelha sobre tinta baixa: a sombra dele cai à direita (luz do topo-esquerda).
fn muro() -> Grid {
    grade(|x, _| {
        if (18..=24).contains(&x) {
            9000.0
        } else {
            3000.0
        }
    })
}

fn sat(c: [f64; 3]) -> f64 {
    let (mx, mn) = (c[0].max(c[1]).max(c[2]), c[0].min(c[1]).min(c[2]));
    (mx - mn) / mx.max(1e-9)
}

/// W3 M1 ⭐ **Subsurface 0 = a imagem de antes, bit a bit** — com o raio a 16 no cartão e a 12 nas células de uma
/// camada (peso 0 em toda a parte), cada célula sai igual à do material NEUTRO.
#[test]
fn a_subsuperficie_zero_e_a_imagem_de_antes() {
    let base = OleoReal::default();
    let g0 = muro();
    let mut g1 = muro();
    for (i, m) in crate::grid::material::garante(&mut g1)
        .iter_mut()
        .enumerate()
    {
        if i % g0.s < 40 {
            m.raio_subsuperficie = 12.0;
        }
    }
    let mut k = MaterialCartao::NEUTRO;
    k.raio_subsuperficie = 16.0;
    let o = com(k);
    let (c0, c1) = (campo(&g0, &base), campo(&g1, &o));
    for y in 1..=W {
        for x in 1..=W {
            assert_eq!(
                ilumina(&o, &c1, x, y, VERMELHO, 1.0),
                ilumina(&base, &c0, x, y, VERMELHO, 1.0),
                "({x}, {y})"
            );
        }
    }
}

/// W3 M1 ⭐ **A lei da Subsurface** (`oraculo/README.md` §M-SSS, o Cycles Random Walk): a luz que entra ao lado
/// ENCHE a sombra com a cor da tinta — mais clara e mais SATURADA (o vermelho espalha-se 5× mais longe que o
/// verde) —; a tinta lisa iluminada longe da sombra fica como estava; e a energia tem TECTO: o que a célula recebe
/// nunca passa da irradiância mais forte da vizinhança.
#[test]
fn a_subsuperficie_enche_a_sombra_com_a_cor_da_tinta_e_a_energia_tem_tecto() {
    let g = muro();
    let sem = OleoReal {
        brilho: 0.0,
        ..OleoReal::default()
    };
    let mut k = MaterialCartao::NEUTRO;
    k.subsuperficie = 1.0;
    let com_sss = OleoReal { material: k, ..sem };
    let (c0, c1) = (campo(&g, &sem), campo(&g, &com_sss));
    let mut sombra = (0, [0.0; 3], [0.0; 3]);
    for y in 20..=44 {
        for x in 25..=34 {
            if termos(&sem, &c0, x, y).luz < 0.5 {
                let (a, b) = (
                    ilumina(&sem, &c0, x, y, VERMELHO, 1.0),
                    ilumina(&com_sss, &c1, x, y, VERMELHO, 1.0),
                );
                sombra.0 += 1;
                for i in 0..3 {
                    sombra.1[i] += a[i];
                    sombra.2[i] += b[i];
                }
            }
        }
    }
    assert!(sombra.0 > 20, "a cena não tem sombra: {}", sombra.0);
    let (a, b) = (
        sombra.1.map(|v| v / f64::from(sombra.0)),
        sombra.2.map(|v| v / f64::from(sombra.0)),
    );
    assert!(b[0] > a[0] + 3.0, "a sombra não se encheu: {a:?} → {b:?}");
    assert!(
        sat(b) > sat(a),
        "a sombra não ficou mais saturada: {a:?} → {b:?}"
    );
    for (x, y) in [(52, 30), (58, 40), (55, 10)] {
        let (a, b) = (
            ilumina(&sem, &c0, x, y, VERMELHO, 1.0),
            ilumina(&com_sss, &c1, x, y, VERMELHO, 1.0),
        );
        for i in 0..3 {
            assert!(
                (a[i] - b[i]).abs() < 0.5,
                "a tinta lisa iluminada mudou em ({x}, {y}): {a:?} → {b:?}"
            );
        }
    }
    let e = |x, y| termos(&com_sss, &c1, x, y).e(&com_sss);
    let teto = (1..=W)
        .flat_map(|y| (1..=W).map(move |x| (x, y)))
        .map(|(x, y)| e(x, y))
        .fold(0.0, f64::max);
    for y in 1..=W {
        for x in 1..=W {
            for s in c1.subsuperficie_em(x, y) {
                assert!(
                    s + e(x, y) <= teto + 1e-6,
                    "({x}, {y}): {s} + {} > {teto}",
                    e(x, y)
                );
            }
        }
    }
}

/// K4 ⭐ **A luz e o céu que a Subsurface já contou são os do sombreado, bit a bit**: com eles guardados ou
/// contados de novo (`esquece_luz_do_ceu`), os termos, a cor e a sombra no papel de cada célula são os MESMOS —
/// numa janela que não é a grade inteira, com a fenda forte e o Estúdio; e uma óptica com OUTRA luz no mesmo campo
/// não os usa (a chave).
#[test]
fn a_luz_do_ceu_guardada_e_a_de_sempre() {
    let mut k = MaterialCartao::NEUTRO;
    k.subsuperficie = 1.0;
    let o = OleoReal {
        fenda: 2.0,
        ..com(k)
    };
    let g = estrias();
    let novo = || CampoAltura::de(&[(&g, true)], &o, 1, 10, 12, 50, 48);
    let (guardado, mut contado) = (novo(), novo());
    contado.esquece_luz_do_ceu();
    let outra = OleoReal {
        azimute: 100f64.to_radians(),
        altitude: 30f64.to_radians(),
        ..o
    };
    for oo in [o, outra] {
        for y in 1..=W {
            for x in 1..=W {
                let m = guardado.material_em(x, y);
                let a = termos_com(&oo, &guardado, x, y, &m, [0.6, 0.1, 0.1]);
                let b = termos_com(&oo, &contado, x, y, &m, [0.6, 0.1, 0.1]);
                assert_eq!(a, b, "({x}, {y})");
                assert_eq!(
                    sombra_no_papel(&oo, &guardado, x, y).to_bits(),
                    sombra_no_papel(&oo, &contado, x, y).to_bits(),
                    "papel ({x}, {y})"
                );
                assert_eq!(
                    ilumina(&oo, &guardado, x, y, VERMELHO, 1.0),
                    ilumina(&oo, &contado, x, y, VERMELHO, 1.0)
                );
            }
        }
    }
}
