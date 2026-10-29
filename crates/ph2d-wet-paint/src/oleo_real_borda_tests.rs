//! Gates do Óleo Real (filho de [`super`]): o relevo nunca sai do pigmento, a borda inclinada sem escada e o custo do
//! contorno (partido de `oleo_real_tests.rs`, 29/10).

use super::*;

/// §BUG-ressalto ⭐ **O relevo nunca sai do pigmento** (smoke do dono 27/10, `oraculo/README.md` §BUG-ressalto): a
/// forma desfocada fazia uma SAIA de altura ~3–4 células no papel à volta da tinta grossa; o papel na saia,
/// inclinado para fora, via o céu inteiro (faixa branca em relevo) e a sombra só começava no pé dela. A célula
/// sem pigmento tem altura 0 nas três escalas, e a sombra no papel é a MAIOR ao pé da parede, nos quatro lados.
#[test]
fn o_relevo_nao_sai_do_pigmento() {
    let g = grade(|x, y| {
        let fora = (20..=44).contains(&x) && (20..=44).contains(&y);
        let janela = (27..=37).contains(&x) && (27..=37).contains(&y);
        if fora && !janela { 12_000.0 } else { 0.0 }
    });
    let o = OleoReal::default();
    let c = campo(&g, &o);
    for y in 1..=W {
        for x in 1..=W {
            if g.sett[x + y * g.s] > 0.0 {
                continue;
            }
            let (xi, yi) = (x as isize, y as isize);
            let h = [c.at(xi, yi), c.at_fina(xi, yi), c.at_sombra(xi, yi)];
            assert!(h == [0.0; 3], "papel ({x}, {y}) com relevo {h:?}");
        }
    }
    // A sombra no papel cai a partir do contorno: a célula ao pé da tinta escurece pelo menos tanto como as de
    // fora (antes: 0 ao pé e a sombra só a 4 células — a faixa branca).
    let s = |x, y| sombra_no_papel(&o, &c, x, y);
    for (rot, pe, longe) in [
        ("fora, esquerda", (19, 32), (15, 32)),
        ("fora, direita", (45, 32), (49, 32)),
        ("fora, cima", (32, 19), (32, 15)),
        ("fora, baixo", (32, 45), (32, 49)),
        ("janela, esquerda", (27, 32), (30, 32)),
        ("janela, direita", (37, 32), (34, 32)),
    ] {
        let (a, b) = (s(pe.0, pe.1), s(longe.0, longe.1));
        // Folga de 0,01: no fundo da janela funda as duas já estão na sombra cheia (0,998 · 0,998).
        assert!(a + 0.01 >= b, "{rot}: ao pé {a} × mais longe {b}");
    }
    // Sob a orla PARCIAL (uma coluna de tinta fina ao pé da parede, cobertura < 1) o papel continua plano: a
    // sombra dele é a do papel ao pé, não a de uma encosta virada para o céu (o fio claro da folha).
    let mut orla = g.clone();
    for y in 20..=44 {
        orla.sett[19 + y * orla.s] = 40.0;
    }
    assert!(crate::opacity::alpha_of_mass(40.0) < 0.9);
    let co = campo(&orla, &o);
    let (_, sob) = ilumina_e_sombra(
        &o,
        &co,
        19,
        32,
        [30.0, 70.0, 210.0],
        crate::opacity::alpha_of_mass(40.0),
    );
    let longe = sombra_no_papel(&o, &co, 15, 32);
    assert!(
        sob + 0.01 >= longe,
        "o papel sob a orla parcial acende-se: {sob} × mais longe {longe}"
    );
    // Com a Subsurface a luz do céu de cada célula fica guardada com a normal DELA: o papel sob a orla não a lê.
    let sss = OleoReal {
        material: MaterialCartao {
            subsuperficie: 1.0,
            raio_subsuperficie: 4.0,
            ..MaterialCartao::NEUTRO
        },
        ..o
    };
    let cs = campo(&orla, &sss);
    let (_, sob) = ilumina_e_sombra(
        &sss,
        &cs,
        19,
        32,
        [30.0, 70.0, 210.0],
        crate::opacity::alpha_of_mass(40.0),
    );
    assert!(
        sob + 0.01 >= sombra_no_papel(&sss, &cs, 15, 32),
        "com a Subsurface o papel sob a orla acende-se: {sob}"
    );
}

/// §BUG-bordas: a tela (luminância e cobertura) de uma borda RECTA inclinada `1/8` (tinta grossa por cima de
/// `y = 20,3 + (x − 1)/8`), pela óptica do composite: a cor da tinta pela cobertura da célula (× a da orla) sobre o
/// papel com a sombra dele. Devolve `(escada, luz)`: o RMS do resíduo da cobertura somada por coluna contra a recta
/// (px) e, em cada distância `d` à recta (−3..3 px, passo ½), o RMS ao longo da borda da luminância menos a média
/// dela — o maior `d` (níveis). A borda recta é invariante ao longo dela: tudo o que varia é a grade.
pub(in crate::oleo_real) fn serrilhado_pub(o: &OleoReal) -> (f64, f64) {
    serrilhado(o)
}

pub(super) fn serrilhado(o: &OleoReal) -> (f64, f64) {
    let recta = |x: usize| 20.3 + (x as f64 - 1.0) / 8.0;
    let mut g = grade(|x, y| {
        if (y as f64) <= recta(x) {
            12_000.0
        } else {
            0.0
        }
    });
    g.sett_rgb.fill([30.0, 70.0, 210.0]);
    let c = campo(&g, o);
    let camadas = [crate::render::RenderLayer {
        grid: &g,
        opacity: 1.0,
        visible: true,
    }];
    let visual = crate::render::PigmentVisual {
        paper: false,
        relevo: true,
    };
    // A tela pela porta do composite: a linha de pigmento com a sombra das células pintadas, e a do papel nu.
    let mut px = vec![[0.0f64; 2]; (W + 1) * (W + 1)];
    for y in 1..=W {
        let (mut out, mut sombra, mut feita) = (vec![0u8; W * 4], vec![0u8; W], vec![false; W]);
        crate::render::render_pigment_row_oleo_com_sombra(
            None,
            &camadas,
            visual,
            o,
            &c,
            y,
            1,
            W,
            &mut out,
            &mut sombra,
            &mut feita,
        );
        for x in 1..=W {
            if !feita[x - 1] {
                sombra[x - 1] = sombra_em_u8(sombra_no_papel(o, &c, x, y));
            }
            let p = &out[(x - 1) * 4..x * 4];
            let pa = f64::from(p[3]) / 255.0;
            let l = 0.2126 * f64::from(p[0]) + 0.7152 * f64::from(p[1]) + 0.0722 * f64::from(p[2]);
            px[x + y * (W + 1)] = [l * pa + f64::from(255 - sombra[x - 1]) * (1.0 - pa), pa];
        }
    }
    let tela = |x: usize, y: usize| (px[x + y * (W + 1)][0], px[x + y * (W + 1)][1]);
    let colunas = 9..=56;
    let mut res = Vec::new();
    let mut perfis = Vec::new();
    for x in colunas.clone() {
        let col: Vec<(f64, f64)> = (1..=W).map(|y| tela(x, y)).collect();
        let soma: f64 = col.iter().map(|p| p.1).sum();
        res.push(soma - (recta(x) + 0.5));
        perfis.push((recta(x), col));
    }
    let mu = res.iter().sum::<f64>() / res.len() as f64;
    let escada = (res.iter().map(|r| (r - mu).powi(2)).sum::<f64>() / res.len() as f64).sqrt();
    let mut luz = 0.0f64;
    for k in -6..=6 {
        let d = f64::from(k) * 0.5;
        let v: Vec<f64> = perfis
            .iter()
            .map(|(r, col)| {
                // A célula `y` (1..) tem o centro em `y`; interpolação linear em `y`.
                let yy = r + d;
                let i = libm::floor(yy) as usize;
                let t = yy - i as f64;
                col[i - 1].0 * (1.0 - t) + col[i].0 * t
            })
            .collect();
        let m = v.iter().sum::<f64>() / v.len() as f64;
        let r = (v.iter().map(|a| (a - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt();
        luz = luz.max(r);
    }
    (escada, luz)
}

/// ⭐ §BUG-bordas (smoke do dono 27/10, `oraculo/README.md` §BUG-bordas): **a borda inclinada não tem escada.** A
/// silhueta de uma recta a 1/8 era uma escada de células e a parede, a sombra e o brilho caíam nela (escada 0,286 px
/// = o dente de serra do binário, luz 29,9 níveis no Clássico); com o contorno sub-célula a cobertura somada segue a
/// recta (0,066) e a luz ao longo da borda fica no piso do antisserrilhado IDEAL (a área exacta amostrada assim dá
/// 8,3 níveis): 8,38 · 6,32 Clássico · Estúdio. Mutações vermelhas: sem o contorno · a cobertura do composite sem ele
/// (`render.rs`) · o contorno arredondado à célula inteira.
#[test]
fn a_borda_inclinada_nao_tem_escada() {
    for ambiente in [Ambiente::Classico, Ambiente::Estudio] {
        let o = OleoReal {
            ambiente,
            ..OleoReal::default()
        };
        let (escada, luz) = serrilhado(&o);
        println!("[serrilhado] {ambiente:?}: escada {escada:.3} px · luz {luz:.2} níveis");
        assert!(
            escada <= 0.12,
            "{ambiente:?}: a silhueta em escada, {escada:.3} px"
        );
        assert!(
            luz <= 10.0,
            "{ambiente:?}: a luz em degraus ao longo da borda, {luz:.2} níveis"
        );
    }
}

#[test]
#[ignore = "medicao — o custo do contorno sub-célula dentro do campo (perfil smoke)"]
fn medida_custo_da_borda_suave() {
    pub(super) const N: usize = 768;
    let mut g = Grid::new(N, N);
    for y in 1..=N {
        for x in 1..=N {
            let (dx, dy) = (x as f64 - 384.0, y as f64 - 384.0);
            if dx.hypot(dy) < 250.0 + 12.0 * (dy.atan2(dx) * 7.0).sin() {
                g.sett[x + y * g.s] = 12_000.0;
            }
        }
    }
    let o = OleoReal::default();
    let celulas = (N * N) as f64;
    let (mut campo_min, mut borda_min) = (f64::MAX, f64::MAX);
    let cob: Vec<f64> = (0..N * N)
        .map(|k| cobertura_da_celula(&[(&g, true)], (k % N + 1) + (k / N + 1) * g.s))
        .collect();
    let orla = cob.iter().filter(|&&c| c >= 0.5).count();
    for _ in 0..5 {
        let t = std::time::Instant::now();
        let c = CampoAltura::de_com(&EmSerie, &[(&g, true)], &o, 1, (1, 1, N, N));
        campo_min = campo_min.min(t.elapsed().as_secs_f64());
        std::hint::black_box(c.at(5, 5));
        let t = std::time::Instant::now();
        let b = borda_suave(&EmSerie, &cob, N, N);
        borda_min = borda_min.min(t.elapsed().as_secs_f64());
        std::hint::black_box(b.len());
    }
    println!(
        "[custo] campo {:.1} ns/célula · contorno {:.2} ns/célula ({:.1} %) · {} células de tinta",
        campo_min * 1e9 / celulas,
        borda_min * 1e9 / celulas,
        100.0 * borda_min / campo_min,
        orla
    );
}
