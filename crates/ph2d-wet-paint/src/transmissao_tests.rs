//! Gates da **Transmission** e do **Relevo do papel** (W3b, `oraculo/README.md` §Transmission): o neutro ao bit, a
//! tinta transmissiva mais saturada que a do Alpha à mesma força, o dente que a tinta mostra e a sombra no papel nu
//! de quem tem a Transparency/Transmission só no PINCEL. O grão na tela final: `ph2d-tool-painter`
//! `a_transmissao_e_o_relevo_do_papel_mostram_o_grao`.

use crate::grid::Grid;
use crate::oleo_real::{
    CampoAltura, OleoReal, grao_do_papel, sombra_no_papel, sombra_transparente,
    transparencia_de_quem_tapa,
};
use crate::render::{PigmentVisual, RenderLayer, render_pigment_row_oleo_com_sombra};

const W: usize = 48;

/// Uma pilha azul com estrias no miolo, papel em volta.
fn tinta() -> Grid {
    let mut g = Grid::new(W, W);
    for y in 12..=36 {
        for x in 12..=36 {
            let i = x + y * g.s;
            g.sett[i] = 3000.0 + 1500.0 * (x as f32 * 0.9).sin();
            g.sett_rgb[i] = [40.0, 70.0, 200.0];
        }
    }
    g
}

fn oleo(f: impl Fn(&mut OleoReal)) -> OleoReal {
    let mut o = OleoReal::default();
    f(&mut o);
    o
}

fn render(g: &Grid, o: &OleoReal) -> (Vec<u8>, Vec<u8>) {
    let camadas = [RenderLayer {
        grid: g,
        opacity: 1.0,
        visible: true,
    }];
    let c = CampoAltura::de(&[(g, true)], o, 1, 1, 1, W, W);
    let (mut px, mut sombra) = (vec![0u8; W * W * 4], vec![0u8; W * W]);
    let mut feita = vec![false; W * W];
    for cy in 1..=W {
        render_pigment_row_oleo_com_sombra(
            None,
            &camadas,
            PigmentVisual::default(),
            o,
            &c,
            cy,
            1,
            W,
            &mut px[(cy - 1) * W * 4..cy * W * 4],
            &mut sombra[(cy - 1) * W..cy * W],
            &mut feita[(cy - 1) * W..cy * W],
        );
    }
    (px, sombra)
}

/// A cor do miolo sobre o papel BRANCO (o píxel por cima do branco pelo alfa) e a SATURAÇÃO `(max − min)/max`.
fn croma_no_miolo(px: &[u8]) -> f64 {
    let (mut soma, mut n) = (0.0, 0.0);
    for y in 16..=32 {
        for x in 16..=32 {
            let p = &px[((y - 1) * W + x - 1) * 4..][..4];
            let a = f64::from(p[3]) / 255.0;
            let c = [0, 1, 2].map(|k| f64::from(p[k]) * a + 255.0 * (1.0 - a));
            let mx = c.iter().copied().fold(0.0, f64::max);
            soma += (mx - c.iter().copied().fold(255.0, f64::min)) / mx.max(1.0);
            n += 1.0;
        }
    }
    soma / n
}

/// ⭐ Neutro AO BIT: a Transmission e o Relevo do papel a 0 — no campo da célula alocado (o ramo do pincel) e no
/// cartão — dão os bytes da tinta de sempre; o dente que a tinta mostra é 0 em toda a célula.
#[test]
fn a_transmissao_e_o_relevo_do_papel_a_zero_sao_a_imagem_de_antes() {
    let g = tinta();
    let mut com_campo = g.clone();
    crate::grid::material::garante(&mut com_campo);
    let antes = render(&g, &OleoReal::default());
    assert_eq!(antes, render(&com_campo, &OleoReal::default()));
    assert_eq!(antes, render(&g, &oleo(|o| o.material.transmissao = 0.0)));
    let c = CampoAltura::de(&[(&com_campo, true)], &OleoReal::default(), 1, 1, 1, W, W);
    for y in 1..=W {
        for x in 1..=W {
            assert_eq!(
                grao_do_papel(&OleoReal::default(), &c, x, y).to_bits(),
                0f64.to_bits()
            );
        }
    }
    assert_eq!(
        crate::material::transmite(0.3, 0.2, 0.9, 0.0).to_bits(),
        0.3f64.to_bits()
    );
}

/// ⭐ A Transmission é VIVA: à mesma força a tinta sobre o papel branco é mais saturada que a do Alpha (que a mistura
/// com o branco) e que a opaca; a força 1 do Alpha some, a da Transmission continua cor. A da CÉLULA (o pincel) também.
#[test]
fn a_transmissao_e_mais_saturada_que_o_alfa() {
    let g = tinta();
    let opaca = croma_no_miolo(&render(&g, &OleoReal::default()).0);
    for f in [0.5, 1.0] {
        let tr = croma_no_miolo(&render(&g, &oleo(|o| o.material.transmissao = f)).0);
        let al = croma_no_miolo(&render(&g, &oleo(|o| o.material.transparencia = f)).0);
        println!(
            "[transmission] força {f}: saturação transmissão {tr:.3} · alfa {al:.3} · opaca {opaca:.3}"
        );
        assert!(tr > al + 0.1, "força {f}: transmissão {tr} × alfa {al}");
        assert!(tr >= opaca, "força {f}: transmissão {tr} × opaca {opaca}");
    }
    let mut cel = g.clone();
    for m in crate::grid::material::garante(&mut cel) {
        m.transmissao = 1.0;
    }
    // A do PINCEL (o material que se vê mistura-se com o papel pela cobertura, como o Metallic): a mesma direcção.
    let pincel = croma_no_miolo(&render(&cel, &OleoReal::default()).0);
    println!("[transmission] pincel 1: saturação {pincel:.3}");
    assert!(
        pincel > opaca,
        "a Transmission do pincel satura: {pincel} × {opaca}"
    );
}

/// O dente que se vê ATRAVÉS da tinta é o da Transmission (célula ∘ cartão por `max`); o Relevo do papel não passa por
/// aqui (vai na altura fina, `o_relevo_do_papel_assenta_a_tinta_no_dente`).
#[test]
fn o_grao_do_papel_atraves_da_tinta_e_o_da_transmissao() {
    let mut g = tinta();
    for m in crate::grid::material::garante(&mut g) {
        m.transmissao = 0.5;
        m.relevo_papel = 1.0;
    }
    let c = CampoAltura::de(&[(&g, true)], &OleoReal::default(), 1, 1, 1, W, W);
    let d = |o: &OleoReal| grao_do_papel(o, &c, 24, 24);
    assert!((d(&OleoReal::default()) - 0.5).abs() < 1e-3);
    assert!((d(&oleo(|o| o.material.transmissao = 0.8)) - 0.8).abs() < 1e-6);
}

/// Uma tinta LISA (massa constante: a única variação é a do papel) sobre um papel em ondas.
fn lisa_sobre_papel() -> Grid {
    let mut g = Grid::new(W, W);
    for y in 0..g.h + 2 {
        for x in 0..g.s {
            let i = x + y * g.s;
            if i < g.paper.len() {
                g.paper[i] = 0.5 + 0.5 * (x as f32 * 1.3).sin() * (y as f32 * 0.7).cos();
            }
        }
    }
    for y in 8..=40 {
        for x in 8..=40 {
            let i = x + y * g.s;
            g.sett[i] = 6000.0;
            g.sett_rgb[i] = [40.0, 70.0, 200.0];
        }
    }
    g
}

/// ⭐ **O Relevo do papel assenta a tinta no dente** (dono 27/10: «a tinta era viva»): com o papel (`dente_papel`) e o
/// Relevo do papel 1 a altura fina da tinta lisa segue o papel e o BRILHO do óleo varia com o grão; a 0 — ou sem
/// papel — a altura e o brilho são os de antes ao bit; o papel nu continua plano; a do pincel vale como a do cartão.
#[test]
fn o_relevo_do_papel_assenta_a_tinta_no_dente() {
    let g = lisa_sobre_papel();
    let com = |f: &dyn Fn(&mut OleoReal)| {
        let o = oleo(|o| {
            o.dente_papel = 1.0;
            f(o);
        });
        let c = CampoAltura::de(&[(&g, true)], &o, 1, 1, 1, W, W);
        (o, c)
    };
    let espec = |(o, c): &(OleoReal, CampoAltura)| -> Vec<f64> {
        let mut v = Vec::new();
        for y in 14..=34 {
            for x in 14..=34 {
                v.push(crate::oleo_real::termos(o, c, x, y).espec[1]);
            }
        }
        v
    };
    let desvio = |v: &[f64]| {
        let mu = v.iter().sum::<f64>() / v.len() as f64;
        (v.iter().map(|x| (x - mu).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
    };
    let antes = com(&|o| o.dente_papel = 0.0);
    let zero = com(&|_| {});
    let um = com(&|o| o.material.relevo_papel = 1.0);
    for y in 1..=W as isize {
        for x in 1..=W as isize {
            assert_eq!(
                antes.1.at_fina(x, y).to_bits(),
                zero.1.at_fina(x, y).to_bits()
            );
        }
    }
    let (s0, s1) = (desvio(&espec(&zero)), desvio(&espec(&um)));
    println!("[transmission] brilho da tinta lisa: desvio Relevo do papel 0 {s0:.5} · 1 {s1:.5}");
    assert!(
        s1 > 5.0 * s0,
        "o Relevo do papel põe o grão no BRILHO: {s1} × {s0}"
    );
    assert_eq!(um.1.at_fina(2, 2), 0.0, "o papel nu continua plano");
    let mut cel = g.clone();
    for m in crate::grid::material::garante(&mut cel) {
        m.relevo_papel = 1.0;
    }
    let o = oleo(|o| o.dente_papel = 1.0);
    let c = CampoAltura::de(&[(&cel, true)], &o, 1, 1, 1, W, W);
    assert!(
        desvio(&espec(&(o, c))) > 5.0 * s0,
        "o Relevo do papel do PINCEL"
    );
}

/// ⭐ A Transmission tinge a cor REAL do papel: sobre um papel CREME a tinta lisa transmissiva sai, canal a canal, o
/// papel × a do papel branco no canal da tinta (o difuso linear `w·a²·D·papel`), e não a do branco.
#[test]
fn a_transmissao_tinge_a_cor_do_papel() {
    let g = lisa_sobre_papel();
    let creme = [0.80, 0.70, 0.52];
    let cor = |p: [f64; 3]| {
        let (px, _) = render(
            &g,
            &oleo(|o| {
                o.material.transmissao = 1.0;
                o.papel = p;
            }),
        );
        let q = &px[((24 - 1) * W + 23) * 4..][..3];
        [0, 1, 2].map(|k| crate::colorops::transfer::srgb255_to_linear(f64::from(q[k])))
    };
    let (b, c) = (cor([1.0; 3]), cor(creme));
    println!("[transmission] tinta lisa Transmission 1: branco {b:.4?} · creme {c:.4?}");
    assert_ne!(b, c, "o papel creme muda a tinta transmissiva");
    // O canal da tinta (o azul): o brilho da superfície por cima não se tinge (nos canais quase pretos ele domina).
    let r = c[2] / b[2];
    assert!(
        (r - creme[2]).abs() < 0.1 * creme[2],
        "azul: creme ÷ branco {r} × papel {}",
        creme[2]
    );
}

/// ⭐ O aberto da Transparency (27/10): a tinta transparente ou transmissiva SÓ do pincel (cartão a 0) projecta no
/// papel NU uma sombra que atravessa — menor que a da opaca, `0` com a Transparency 1. Sem material, a do cartão.
#[test]
fn a_sombra_da_tinta_do_pincel_atravessa_no_papel_nu() {
    let o = OleoReal::default();
    let opaca = tinta();
    let com = |f: &dyn Fn(&mut crate::material::MaterialCelula)| {
        let mut g = tinta();
        for m in crate::grid::material::garante(&mut g) {
            f(m);
        }
        g
    };
    let soma = |g: &Grid| {
        let c = CampoAltura::de(&[(g, true)], &o, 1, 1, 1, W, W);
        let mut s = 0.0;
        for y in 37..=46 {
            for x in 37..=46 {
                s += sombra_transparente(
                    sombra_no_papel(&o, &c, x, y),
                    0.0,
                    transparencia_de_quem_tapa(&o, &c, x, y),
                );
            }
        }
        s
    };
    let s0 = soma(&opaca);
    let st = soma(&com(&|m| m.transparencia = 0.5));
    let s1 = soma(&com(&|m| m.transparencia = 1.0));
    let sw = soma(&com(&|m| m.transmissao = 1.0));
    println!(
        "[transmission] sombra no papel nu: opaca {s0:.3} · T 0,5 {st:.3} · T 1 {s1:.3} · Transmission 1 {sw:.3}"
    );
    assert!(s0 > 0.5, "a opaca projecta sombra: {s0}");
    assert!(st < 0.9 * s0, "T 0,5 do pincel atenua: {st} × {s0}");
    assert!(s1 < 1e-9, "T 1 do pincel não projecta: {s1}");
    assert!(
        sw < s0 && sw > 0.0,
        "a Transmission atenua pela cor: {sw} × {s0}"
    );
    let c = CampoAltura::de(&[(&opaca, true)], &o, 1, 1, 1, W, W);
    let k = oleo(|o| o.material.transparencia = 0.3);
    assert_eq!(transparencia_de_quem_tapa(&k, &c, 40, 40), 0.3);
}

/// O campo guardado (`oleo_cache`) não serve a altura fina de outro Relevo do papel do cartão nem de outro dente.
#[test]
fn a_chave_do_campo_le_o_relevo_do_papel_e_o_dente() {
    let g = lisa_sobre_papel();
    let k = |o: &OleoReal| crate::oleo_real::ChaveCampo::de(o, &[(&g, true)], 1);
    let base = oleo(|o| o.dente_papel = 1.0);
    assert_ne!(k(&base), k(&oleo(|o| o.dente_papel = 0.5)));
    assert_ne!(
        k(&base),
        k(&oleo(|o| {
            o.dente_papel = 1.0;
            o.material.relevo_papel = 1.0;
        }))
    );
}
