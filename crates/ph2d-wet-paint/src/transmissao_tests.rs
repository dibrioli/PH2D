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

/// A grade `g` com o material de cada célula mexido por `f` (o do pincel: o cartão não tem os três pesos).
fn cel(g: &Grid, f: impl Fn(&mut crate::material::MaterialCelula)) -> Grid {
    let mut g = g.clone();
    for m in crate::grid::material::garante(&mut g) {
        f(m);
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

/// ⭐ Neutro AO BIT: a Transmission e o Relevo do papel a 0 no campo da célula alocado (o ramo do pincel) dão os bytes
/// da tinta de sempre; o dente que a tinta mostra é 0 em toda a célula.
#[test]
fn a_transmissao_e_o_relevo_do_papel_a_zero_sao_a_imagem_de_antes() {
    let g = tinta();
    let mut com_campo = g.clone();
    crate::grid::material::garante(&mut com_campo);
    let antes = render(&g, &OleoReal::default());
    assert_eq!(antes, render(&com_campo, &OleoReal::default()));
    let c = CampoAltura::de(&[(&com_campo, true)], &OleoReal::default(), 1, 1, 1, W, W);
    for y in 1..=W {
        for x in 1..=W {
            assert_eq!(
                grao_do_papel(&OleoReal::default(), &c, x, y).to_bits(),
                0f64.to_bits()
            );
        }
    }
    for claro in [0.0, 0.5, 1.0] {
        assert_eq!(
            crate::material::transmite(0.3, 0.2, 0.9, 0.0, claro).to_bits(),
            0.3f64.to_bits()
        );
        assert_eq!(
            crate::oleo_real::transparencia_a_luz(0.4, 0.0, [0.2, 0.3, 0.8], claro).to_bits(),
            0.4f64.to_bits()
        );
        // Transmission 0 = a imagem de antes ao bit nos dois modos (o modo sozinho não morde).
        assert_eq!(
            antes,
            render(&cel(&g, |m| m.vidro_claro = claro as f32), &OleoReal::default()),
            "o modo {claro} sem Transmission mudou a tinta"
        );
    }
}

/// ⭐ **Clear glass = a lei de `ad16f60b75` AO BIT** (dono 28/10: «sem tirar a atual»): a conta de antes (`base·(1 − w) +
/// w·a^(2·VIDRO_K)·d` e a luz pelo vidro `√(1 − (1 − t²)(1 − w·L(a^(2·VIDRO_K))))`) escrita aqui à mão, contra a de
/// agora com o modo 1, numa grelha de entradas. A tela inteira: `ph2d-tool-painter`
/// `o_modo_blender_casa_com_o_cycles_e_o_clear_glass_e_o_de_antes` (o hash da tela de `ad16f60b75`).
#[test]
fn o_clear_glass_e_a_lei_de_antes_ao_bit() {
    let vidro = |a: f64| if a <= 0.0 { 0.0 } else { libm::pow(a.min(1.0), 2.0 * 0.2) };
    let antes = |base: f64, a: f64, d: f64, w: f64| {
        if w <= 0.0 { base } else { base * (1.0 - w) + w * vidro(a) * d }
    };
    let luz = |t: f64, w: f64, a: [f64; 3]| {
        if w <= 0.0 {
            return t;
        }
        let v = a.map(vidro);
        let passa = w * (0.2126 * v[0] + 0.7152 * v[1] + 0.0722 * v[2]);
        libm::sqrt(1.0 - (1.0 - t * t) * (1.0 - passa))
    };
    for i in 0..=20 {
        let a = f64::from(i) / 20.0;
        for w in [0.0, 0.3, 0.5, 1.0] {
            assert_eq!(
                crate::material::transmite(0.37, a, 0.81, w, 1.0).to_bits(),
                antes(0.37, a, 0.81, w).to_bits()
            );
            assert_eq!(crate::material::filtro(a, 1.0).to_bits(), vidro(a).to_bits());
            for t in [0.0, 0.4] {
                let c = [a, 0.5 * a, 1.0 - a];
                assert_eq!(
                    crate::oleo_real::transparencia_a_luz(t, w, c, 1.0).to_bits(),
                    luz(t, w, c).to_bits()
                );
            }
        }
    }
}

/// ⭐ **Blender = o Principled do Cycles, mais escuro** (dono 28/10): o filtro `a^(2·K_BLENDER)` (`a`), a tinta a 1 mais
/// escura que o Clear glass e com mais croma; a Transmission não passa luz para o papel (`transparencia_a_luz` = o
/// Alpha ao bit). Os números × Cycles: `ph2d-tool-painter` `o_modo_blender_casa_com_o_cycles_e_o_clear_glass_e_o_de_antes`.
#[test]
fn o_modo_blender_e_o_principled_escuro() {
    for i in 1..=20 {
        let a = f64::from(i) / 20.0;
        assert_eq!(
            crate::material::filtro(a, 0.0).to_bits(),
            libm::pow(a, 2.0 * crate::material::K_BLENDER).to_bits()
        );
        assert!(crate::material::filtro(a, 0.0) <= crate::material::filtro(a, 1.0));
        assert_eq!(
            crate::oleo_real::transparencia_a_luz(0.3, 1.0, [a; 3], 0.0).to_bits(),
            0.3f64.to_bits(),
            "Blender: a luz não atravessa"
        );
    }
    let g = tinta();
    let (pb, _) = render(&cel(&g, |m| m.transmissao = 1.0), &OleoReal::default());
    let (pv, _) = render(
        &cel(&g, |m| {
            m.transmissao = 1.0;
            m.vidro_claro = 1.0;
        }),
        &OleoReal::default(),
    );
    let (lb, lv) = (luz_no_miolo(&pb), luz_no_miolo(&pv));
    let (sb, sv) = (croma_no_miolo(&pb), croma_no_miolo(&pv));
    println!("[transmission] Transmission 1, papel branco: L Blender {lb:.3} × Clear glass {lv:.3} · saturação {sb:.3} × {sv:.3}");
    assert!(lb < 0.7 * lv, "Blender mais escuro: {lb} × {lv}");
    assert!(sb > sv, "Blender mais saturado: {sb} × {sv}");
}

/// ⭐ O modo mistura-se pela massa que TRANSMITE: a tinta sem Transmission não tem modo a dar; dois no mesmo modo dão
/// esse modo ao bit; dois a transmitir, a média pesada.
#[test]
fn o_modo_mistura_pela_massa_transmissiva() {
    use crate::material::MaterialCelula as M;
    let vidro = M { transmissao: 0.8, vidro_claro: 1.0, ..M::NEUTRO };
    let blender = M { transmissao: 0.8, ..M::NEUTRO };
    let opaca_vidro = M { vidro_claro: 1.0, ..M::NEUTRO };
    for t in [0.1, 0.37, 0.5, 0.9] {
        assert_eq!(M::mistura(vidro, M::NEUTRO, t).vidro_claro.to_bits(), 1f32.to_bits());
        assert_eq!(M::mistura(M::NEUTRO, vidro, t).vidro_claro.to_bits(), 1f32.to_bits());
        assert_eq!(M::mistura(vidro, opaca_vidro, t).vidro_claro.to_bits(), 1f32.to_bits());
        let m = M::mistura(vidro, blender, t).vidro_claro;
        assert!((m - (1.0 - t)).abs() < 1e-6, "a média pesada: {m} × {}", 1.0 - t);
    }
    assert_eq!(M::mistura(M::NEUTRO, opaca_vidro, 0.25).vidro_claro, 0.25, "sem Transmission: linear");
}

/// A luminância LINEAR média do miolo sobre o papel branco (o píxel por cima do branco pelo alfa), ÷ a do papel.
fn luz_no_miolo(px: &[u8]) -> f64 {
    let (mut soma, mut n) = (0.0, 0.0);
    for y in 16..=32 {
        for x in 16..=32 {
            let p = &px[((y - 1) * W + x - 1) * 4..][..4];
            let a = f64::from(p[3]) / 255.0;
            let c = [0, 1, 2].map(|k| {
                crate::colorops::transfer::srgb255_to_linear(f64::from(p[k]) * a + 255.0 * (1.0 - a))
            });
            soma += 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
            n += 1.0;
        }
    }
    soma / n
}

/// ⭐ **A Transmission a 1 é um VIDRO COLORIDO CLARO** (dono 27/10, W3b ronda 3: «o papel bem claro, com o grão, só
/// tingido pela cor»): o miolo sobre o papel branco sai claro (L ÷ papel ≥ 0,3; a opaca 0,1), com o MATIZ da tinta
/// (a ordem dos canais) e mais saturado que o Alpha à MESMA luminância (o vidro multiplica, o Alpha mistura branco);
/// sobe com o peso. A força 0,5 fica entre a opaca e o vidro.
#[test]
fn a_transmissao_e_um_vidro_colorido_claro() {
    let g = tinta();
    let (po, _) = render(&g, &OleoReal::default());
    // O modo Clear glass (o de antes, dono 28/10).
    let vidro = |w: f32| {
        cel(&g, move |m| {
            m.transmissao = w;
            m.vidro_claro = 1.0;
        })
    };
    let (p1, _) = render(&vidro(1.0), &OleoReal::default());
    let (pm, _) = render(&vidro(0.5), &OleoReal::default());
    let (lo, l1, lm) = (luz_no_miolo(&po), luz_no_miolo(&p1), luz_no_miolo(&pm));
    let s1 = croma_no_miolo(&p1);
    // O Alpha da mesma luminância que o vidro (procura em passos de 0,05).
    let (mut melhor, mut sa) = (f64::INFINITY, 0.0);
    for k in 1..20 {
        let (pa, _) = render(&cel(&g, |m| m.transparencia = k as f32 * 0.05), &OleoReal::default());
        let d = (luz_no_miolo(&pa) - l1).abs();
        if d < melhor {
            (melhor, sa) = (d, croma_no_miolo(&pa));
        }
    }
    println!(
        "[transmission] vidro: L ÷ papel opaca {lo:.3} · 0,5 {lm:.3} · 1 {l1:.3}; saturação vidro {s1:.3} × Alpha da mesma L {sa:.3} (ΔL {melhor:.3})"
    );
    assert!(l1 >= 0.3, "o vidro é claro: {l1}");
    assert!(lo < lm && lm < l1, "sobe com o peso: {lo} · {lm} · {l1}");
    assert!(s1 > sa, "o vidro tinge, o Alpha lava: {s1} × {sa}");
    let canal = |px: &[u8], k: usize| px[((24 - 1) * W + 23) * 4 + k];
    assert!(
        canal(&p1, 2) > canal(&p1, 1) && canal(&p1, 1) > canal(&p1, 0),
        "o matiz da tinta (azul > verde > vermelho)"
    );
}

/// O dente que se vê ATRAVÉS da tinta é o da Transmission da CÉLULA (o cartão não a tem); o Relevo do papel não passa
/// por aqui (vai na altura fina, `o_relevo_do_papel_assenta_a_tinta_no_dente`).
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
    let mut k = OleoReal::default();
    for c in 0..crate::oleo_real::N_CAMPOS {
        k.material.poe(c, 0.9);
    }
    assert!((d(&k) - 0.5).abs() < 1e-3, "o cartão não muda o dente que a tinta mostra");
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
/// papel — a altura e o brilho são os de antes ao bit; o papel nu continua plano. Só a do PINCEL (célula).
#[test]
fn o_relevo_do_papel_assenta_a_tinta_no_dente() {
    let lisa = lisa_sobre_papel();
    let com = |r: f32, f: &dyn Fn(&mut OleoReal)| {
        let g = cel(&lisa, |m| m.relevo_papel = r);
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
    let antes = com(1.0, &|o| o.dente_papel = 0.0);
    let zero = com(0.0, &|_| {});
    let um = com(1.0, &|_| {});
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
}

/// ⭐ A Transmission tinge a cor REAL do papel: sobre um papel CREME a tinta lisa transmissiva sai, canal a canal, o
/// papel × a do papel branco no canal da tinta (o difuso linear `w·vidro(a)·D·papel`), e não a do branco.
#[test]
fn a_transmissao_tinge_a_cor_do_papel() {
    let g = cel(&lisa_sobre_papel(), |m| m.transmissao = 1.0);
    let creme = [0.80, 0.70, 0.52];
    let cor = |p: [f64; 3]| {
        let (px, _) = render(
            &g,
            &oleo(|o| o.papel = p),
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

/// ⭐ O aberto da Transparency (27/10): a tinta transparente ou transmissiva do pincel projecta no papel NU uma sombra
/// que atravessa — menor que a da opaca, `0` com a Transparency 1. Sem material, a opaca (`0`).
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
    let sw = soma(&com(&|m| {
        m.transmissao = 1.0;
        m.vidro_claro = 1.0;
    }));
    let sb = soma(&com(&|m| m.transmissao = 1.0));
    println!(
        "[transmission] sombra no papel nu: opaca {s0:.3} · T 0,5 {st:.3} · T 1 {s1:.3} · Transmission 1 Clear glass {sw:.3} · Blender {sb:.3}"
    );
    assert_eq!(sb.to_bits(), s0.to_bits(), "Blender: a sombra da opaca (o Cycles)");
    assert!(s0 > 0.5, "a opaca projecta sombra: {s0}");
    assert!(st < 0.9 * s0, "T 0,5 do pincel atenua: {st} × {s0}");
    assert!(s1 < 1e-9, "T 1 do pincel não projecta: {s1}");
    assert!(
        sw < s0 && sw > 0.0,
        "a Transmission atenua pela cor: {sw} × {s0}"
    );
    let c = CampoAltura::de(&[(&opaca, true)], &o, 1, 1, 1, W, W);
    assert_eq!(transparencia_de_quem_tapa(&o, &c, 40, 40), 0.0);
}

/// O campo guardado (`oleo_cache`) não serve a altura fina de outro dente (o Relevo do papel é da célula: entra pela
/// revisão das grades).
#[test]
fn a_chave_do_campo_le_o_dente() {
    let g = lisa_sobre_papel();
    let k = |o: &OleoReal| crate::oleo_real::ChaveCampo::de(o, &[(&g, true)], 1);
    let base = oleo(|o| o.dente_papel = 1.0);
    assert_ne!(k(&base), k(&oleo(|o| o.dente_papel = 0.5)));
}

/// ⭐ **D3/C2 (dono 27/10): a Transmission, a Transparency e o Relevo do papel SAÍRAM do cartão Real Oil** — o cartão
/// tem os 8 do Principled e nenhum destes nomes; um cartão com todos os campos mexidos não muda o peso que uma célula
/// NEUTRA mostra (eles são só da tinta pousada).
#[test]
fn o_cartao_nao_tem_os_pesos_da_tinta() {
    use crate::oleo_real::{MaterialCartao, N_CAMPOS};
    assert_eq!(N_CAMPOS, 8);
    for nome in ["transparencia", "transmissao", "relevo_papel"] {
        assert!(!MaterialCartao::CAMPOS.contains(&nome), "{nome} no cartão");
    }
    let mut k = MaterialCartao::NEUTRO;
    for c in 0..N_CAMPOS {
        k.poe(c, 1.0);
    }
    let e = crate::material::efectivo(&crate::material::MaterialCelula::NEUTRO, &k);
    assert_eq!((e.transparencia, e.transmissao, e.relevo_papel), (0.0, 0.0, 0.0));
}

/// ⭐ **C5: a sombra SOB a tinta e no papel NU usam a mesma conta da Transmission** (`transparencia_a_luz`): uma tinta
/// só com Transmission 1 (Transparency 0) atenua a sombra que projecta na MESMA razão dos dois lados da orla — sob a
/// orla parcial (o plano que o `render` escreve) e no papel nu ao lado (`transparencia_de_quem_tapa`). A sombra SEGUE
/// O MODO (dono 28/10): Clear glass atenua (0,579 · 0,600 em `ad16f60b75`); Blender = a da opaca nos dois lados, ao
/// byte (o Cycles: Transmission 1 ÷ opaca 1,00).
#[test]
fn a_sombra_da_transmissao_casa_sob_a_tinta_e_no_papel_nu() {
    let g = tinta();
    let (po, so) = render(&g, &OleoReal::default());
    let razao = |claro: f32| {
        let (_, sw) = render(
            &cel(&g, |m| {
                m.transmissao = 1.0;
                m.vidro_claro = claro;
            }),
            &OleoReal::default(),
        );
        let (mut sob, mut nu) = ([0.0f64; 2], [0.0f64; 2]);
        for k in 0..W * W {
            if so[k] < 16 {
                continue;
            }
            let lado = if po[k * 4 + 3] > 0 { &mut sob } else { &mut nu };
            lado[0] += f64::from(sw[k]);
            lado[1] += f64::from(so[k]);
        }
        assert!(sob[1] > 0.0 && nu[1] > 0.0, "controlo: sombra dos dois lados");
        (sob[0] / sob[1], nu[0] / nu[1], sw)
    };
    let (rs, rn, _) = razao(1.0);
    let (bs, bn, sb) = razao(0.0);
    println!(
        "[transmission] sombra Transmission 1 ÷ opaca: Clear glass sob a tinta {rs:.3} · papel nu {rn:.3} · Blender {bs:.3} · {bn:.3}"
    );
    assert!(rn < 0.97, "Clear glass: o papel nu atenua: {rn}");
    assert!((rs - rn).abs() < 0.1, "Clear glass: a costura: sob {rs} × nu {rn}");
    assert_eq!(sb, so, "Blender: a sombra da opaca, sob a tinta e no papel nu");
}
