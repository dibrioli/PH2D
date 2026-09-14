//! Os gates do [`super`] — a tabela contra a definição, e as duas metades que prendem a energia.

use super::*;

/// A FORMA da caixa, **re-escrita aqui de propósito**: o oráculo mede o pré-filtro, e se ele
/// chamasse a forma do produto uma mutação na forma passaria pelos dois lados ao mesmo tempo.
fn forma(s: Softbox, cos_psi: f64) -> f64 {
    let psi = cos_psi.clamp(-1.0, 1.0).acos();
    let (dentro, fora) = (
        f64::from(s.radius_deg - s.rim_deg * 0.5).to_radians(),
        f64::from(s.radius_deg + s.rim_deg * 0.5).to_radians(),
    );
    // ⚠️ Em COSSENO, como o produto — e escrito aqui de outra maneira de propósito.
    let t = ((psi.cos() - fora.cos()) / (dentro.cos() - fora.cos())).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Uma grelha de direcções quase uniforme sobre a esfera (espiral de Fibonacci).
fn sphere(n: usize) -> Vec<[f64; 3]> {
    let phi = std::f64::consts::PI * (3.0 - 5.0_f64.sqrt());
    (0..n)
        .map(|i| {
            let y = 1.0 - 2.0 * (i as f64 + 0.5) / n as f64;
            let r = (1.0 - y * y).max(0.0).sqrt();
            let a = phi * i as f64;
            [r * a.cos(), y, r * a.sin()]
        })
        .collect()
}

/// **A DEFINIÇÃO** — o pré-filtro do `mx_environment_prefilter` (`N = V = R`), com `200 000`
/// amostras contra as `512` do produto: `390×` mais densa.
fn definicao(alpha: f64, cos_psi: f64, shape: Softbox) -> f64 {
    const N: u32 = 200_000;
    let a2 = alpha * alpha;
    let sin_psi = (1.0 - cos_psi * cos_psi).max(0.0).sqrt();
    let (mut num, mut den) = (0.0_f64, 0.0_f64);
    for i in 0..N {
        let u1 = (f64::from(i) + 0.5) / f64::from(N);
        let u2 = f64::from(i.reverse_bits()) / f64::from(u32::MAX);
        let ch2 = (1.0 - u1) / (1.0 + (a2 - 1.0) * u1);
        let ch = ch2.max(0.0).sqrt();
        let sh = (1.0 - ch2).max(0.0).sqrt();
        let phi = 2.0 * std::f64::consts::PI * u2;
        let h = [sh * phi.cos(), sh * phi.sin(), ch];
        let l = [2.0 * ch * h[0], 2.0 * ch * h[1], 2.0 * ch * h[2] - 1.0];
        if l[2] <= 0.0 {
            continue;
        }
        num += l[2] * forma(shape, sin_psi * l[0] + cos_psi * l[2]);
        den += l[2];
    }
    num / den
}

/// ⭐⭐⭐ **A TABELA É O PRÉ-FILTRO QUE ELA DIZ SER** — contra a definição, com uma quadratura `390×`
/// mais densa e em pontos que caem **entre** as células (é a interpolação que se está a medir, não a
/// aritmética de uma célula).
///
/// ⚠️ **A barra é ABSOLUTA e não relativa**, e a razão está na física: a cauda da calote vale `1e-6`
/// e um erro relativo lá não move um byte. *Uma barra relativa mediria a cauda e ignoraria o que se
/// vê.*
///
/// ⚠️⚠️ **E o VALOR da barra saiu da régua do consumidor, não desta.** A primeira redacção pedia
/// `2e-3` e reprovava com `7,4e-3` — até a sonda `measure_which_softbox_the_rulers_choose` medir o
/// que a diferença vale em PIXELS: contra uma tabela `16×` mais cara, `\|Δ\| médio 0,041 byte` e pior
/// caso `3` bytes. *Perseguir a régua intermédia custaria `4×` o preço da construção para comprar
/// `0,034` de um byte.* ⇒ a barra é `1e-2`, e o CONTROLO abaixo mostra que ela ainda prende uma
/// mudança a sério.
#[test]
fn the_table_is_the_prefilter_it_claims_to_be() {
    let forma_do_produto = Softbox::PRODUCT;
    let t = BoxPrefilter::build(forma_do_produto, 0.35);
    let mut pior = (0.0_f64, 0.0_f32, 0.0_f64);
    for &rough in &[0.03_f32, 0.17, 0.31, 0.52, 0.73, 0.94] {
        let alpha = rough * rough;
        for &psi in &[
            0.0_f64, 7.3, 18.5, 23.9, 26.2, 31.1, 52.7, 88.0, 133.0, 176.0,
        ] {
            let cos_psi = psi.to_radians().cos();
            let alvo = definicao(f64::from(alpha), cos_psi, forma_do_produto);
            let meu = f64::from(t.specular(alpha, cos_psi as f32));
            let err = (meu - alvo).abs();
            if err > pior.0 {
                pior = (err, rough, psi);
            }
        }
    }
    println!(
        "pior desvio absoluto da tabela: {:.3e} (rugosidade {}, ψ {}°)",
        pior.0, pior.1, pior.2
    );
    assert!(
        pior.0 < 1.0e-2,
        "a tabela afastou-se da definição: {:.3e} em rugosidade {} ψ {}°",
        pior.0,
        pior.1,
        pior.2
    );

    // ⭐ **CONTROLO — uma barra de `1e-2` ainda prende a FORMA.** Uma caixa `10°` maior lida contra a
    // definição da caixa do produto sai muito acima dela; sem isto a barra larga não afirmaria nada.
    let outra = BoxPrefilter::build(
        Softbox {
            radius_deg: SOFTBOX_RADIUS_DEG + 10.0,
            rim_deg: SOFTBOX_RIM_DEG,
        },
        0.35,
    );
    let mut fora = 0.0_f64;
    for &rough in &[0.03_f32, 0.31, 0.73] {
        for &psi in &[18.5_f64, 26.2, 31.1] {
            let cos_psi = psi.to_radians().cos();
            let alvo = definicao(f64::from(rough * rough), cos_psi, forma_do_produto);
            fora =
                fora.max((f64::from(outra.specular(rough * rough, cos_psi as f32)) - alvo).abs());
        }
    }
    assert!(
        fora > 0.1,
        "o controlo só se afastou {fora:.3e}: a barra deixou de prender a forma"
    );
}

/// ⭐⭐ **O DIFUSO da caixa também sai da definição** — o lóbulo cosseno recortado, contra uma grelha
/// `5×` mais densa que a da construção.
#[test]
fn the_diffuse_table_is_the_cosine_convolution() {
    let t = BoxPrefilter::build(Softbox::PRODUCT, 0.35);
    let dirs = sphere(200_000);
    let mut pior = 0.0_f64;
    for &psi in &[0.0_f64, 11.0, 29.0, 61.0, 97.0, 145.0, 180.0] {
        let (c, s) = (psi.to_radians().cos(), psi.to_radians().sin());
        let n = [s, c, 0.0];
        let soma: f64 = dirs
            .iter()
            .map(|w| {
                let cn = w[0] * n[0] + w[1] * n[1];
                if cn <= 0.0 {
                    0.0
                } else {
                    cn * forma(Softbox::PRODUCT, w[1])
                }
            })
            .sum();
        let alvo = soma * 4.0 / (dirs.len() as f64);
        let meu = f64::from(t.diffuse(c as f32));
        pior = pior.max((meu - alvo).abs());
    }
    println!("pior desvio absoluto do difuso: {pior:.3e}");
    assert!(
        pior < 2.0e-3,
        "o difuso afastou-se da definição: {pior:.3e}"
    );
}

/// ⭐⭐⭐ **UMA CAIXA INFINITAMENTE LARGA É O AMBIENTE DE VOLTA.** É a metade que prova que a
/// normalização das duas tabelas é a mesma — e é ela que mantém o *furnace test* de pé sem uma linha
/// nova: uma fonte uniforme atravessa o pré-filtro sem se mexer.
#[test]
fn a_box_as_wide_as_the_sky_is_the_sky() {
    let t = BoxPrefilter::build(
        Softbox {
            radius_deg: 180.0,
            rim_deg: 0.0,
        },
        0.35,
    );
    for rough in [0.0_f32, 0.25, 0.6, 1.0] {
        for psi in [0.0_f32, 45.0, 90.0, 180.0] {
            let m = t.specular(rough * rough, psi.to_radians().cos());
            assert!(
                (m - 1.0).abs() < 3.0e-3,
                "especular {m} em rug {rough} ψ {psi}"
            );
        }
    }
    for psi in [0.0_f32, 45.0, 90.0, 180.0] {
        let d = t.diffuse(psi.to_radians().cos());
        assert!((d - 1.0).abs() < 3.0e-3, "difuso {d} em ψ {psi}");
    }
    // E o céu inteiro volta a ser a rampa.
    let largo = Studio { softbox: Some(&t) };
    let nu = Studio::bare_ramp();
    for d in sphere(500) {
        let d = d.map(|v| v as f32);
        for a in [0.0_f32, 0.09, 1.0] {
            let (x, y) = (largo.radiance(d, a), nu.radiance(d, a));
            for c in 0..3 {
                assert!(
                    (x[c] - y[c]).abs() < 4.0e-3,
                    "dir={d:?} α={a}: {x:?} vs {y:?}"
                );
            }
        }
    }
}

/// ⭐⭐⭐ **SEM CAIXA, o céu é o de ontem — ao BIT.** É a metade que garante que esta wave não é uma
/// regressão disfarçada de feature.
#[test]
fn with_no_box_the_sky_is_the_one_it_replaces() {
    let nu = Studio::bare_ramp();
    for d in sphere(3_000) {
        let d = d.map(|v| v as f32);
        for a in [0.0_f32, 0.0025, 0.09, 0.49, 1.0] {
            assert_eq!(nu.radiance(d, a), antigo(d, a), "dir={d:?} α={a}");
        }
        assert_eq!(
            nu.irradiance(d),
            ph2d_light::env_ambient([d[0], -d[1], d[2]])
        );
    }
}

/// A lei da rampa **como ela estava escrita** antes desta wave — a referência do gate acima.
fn antigo(dir: [f32; 3], alpha: f32) -> [f32; 3] {
    const RAW: f32 = 1.5;
    let up = crate::render_light::lobe_shrink(alpha) * dir[1];
    [0, 1, 2].map(|i| {
        ph2d_light::AMBIENT * (ph2d_light::ENV_BASE[i] + RAW * ph2d_light::ENV_SLOPE[i] * up)
    })
}

/// ⭐⭐⭐ **A CAIXA NÃO ACRESCENTA LUZ: ela REDISTRIBUI.** A radiância média sobre a esfera fica igual à
/// da rampa — logo o `8,4 %` de branco chapado do `docs/Render3d/05` §8 não pode piorar por
/// acumulação —, e a irradiância média também.
///
/// ⚠️ **Com o controlo:** uma caixa que se SOMASSE (sem o desconto no termo constante) sai `35 %`
/// acima, e é essa a mutação que o gate mata.
#[test]
fn the_box_redistributes_the_sky_it_does_not_add_to_it() {
    let dirs = sphere(150_000);
    let media = |st: &Studio, f: &dyn Fn(&Studio, [f32; 3]) -> [f32; 3]| -> [f64; 3] {
        let mut s = [0.0_f64; 3];
        for d in &dirs {
            let r = f(st, d.map(|v| v as f32));
            for c in 0..3 {
                s[c] += f64::from(r[c]);
            }
        }
        s.map(|v| v / dirs.len() as f64)
    };
    // ⚠️ A média mede-se na radiância CRUA — o núcleo mais estreito é o que a devolve sem filtro.
    let crua = |st: &Studio, d: [f32; 3]| st.radiance(d, 0.0);
    let irr = |st: &Studio, d: [f32; 3]| st.irradiance(d);
    let nu = Studio::bare_ramp();
    for (raio, share) in [(12.0_f32, 0.2_f32), (25.0, 0.35), (45.0, 0.5)] {
        let t = BoxPrefilter::build(
            Softbox {
                radius_deg: raio,
                rim_deg: SOFTBOX_RIM_DEG,
            },
            share,
        );
        let com = Studio { softbox: Some(&t) };
        for (nome, f) in [
            ("radiância", &crua as &dyn Fn(&Studio, [f32; 3]) -> [f32; 3]),
            ("irradiância", &irr),
        ] {
            let a = media(&com, f);
            let b = media(&nu, f);
            for c in 0..3 {
                let rel = (a[c] - b[c]).abs() / b[c];
                assert!(
                    rel < 5.0e-3,
                    "{nome} θ={raio} f={share} canal {c}: {:.5} contra {:.5} ({rel:.2e})",
                    a[c],
                    b[c]
                );
            }
        }
    }
}

/// ⭐⭐ **A ENERGIA VAI TODA PARA CIMA** — a caixa é a razão de o chão escurecer, e é isso que faz o
/// estúdio sair de graça: o contraste cresce sem o céu ganhar um watt.
#[test]
fn the_box_brightens_the_zenith_and_darkens_the_floor() {
    let st = Studio::of_the_product();
    let nu = Studio::bare_ramp();
    let espelho = 0.0;
    let (cima, baixo) = ([0.0, 1.0, 0.0], [0.0, -1.0, 0.0]);
    assert!(st.radiance(cima, espelho)[1] > nu.radiance(cima, espelho)[1] * 1.5);
    assert!(st.radiance(baixo, espelho)[1] < nu.radiance(baixo, espelho)[1]);
    assert!(st.irradiance(cima)[1] > nu.irradiance(cima)[1]);
    assert!(st.irradiance(baixo)[1] < nu.irradiance(baixo)[1]);
}

/// ⏱️ **SONDA — DE ONDE SAEM OS DOIS NÚMEROS DA CAIXA** (`docs/Render3d/05`).
///
/// O raio e a fracção de energia são as duas únicas constantes desta wave, e nenhuma se escolhe:
/// varrem-se contra as réguas do PRODUTO, em bytes sobre os pixels da peça.
///
/// | régua | o que ela prende |
/// |---|---|
/// | estrutura do ESPELHO | é para isto que a caixa existe — tem de subir |
/// | estrutura do GIZ | um baço não pode ganhar aresta: ali não há nada a reflectir |
/// | `Roughness` num dieléctrico | o controlo que o dono mais usa movia `1,82 %` dos pixels |
/// | branco chapado a `0` stops | ⛔ **não pode subir** — é o `8,4 %` aberto no §8 |
/// | média do verde | tem de ficar parada: a energia foi redistribuída, não somada |
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_which_softbox_the_rulers_choose() {
    use ph2d_field_render::{Lighting, Orbit, shade_render, trace};
    use ph2d_view_transform::Look;

    const BG: [u8; 4] = [12, 34, 56, 200];
    let (w, h) = (640_u32, 360_u32);
    let cam = Orbit::default();
    let doc = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.6 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("esfera");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let g = trace(&doc, &reg, &cam, w, h);
    let lamps = crate::render_light::lamps(&ph2d_light::LightRig::default());

    let pinta_com =
        |m: ph2d_material::OpenPbr, sky: &(dyn ph2d_material::Environment + Sync), look: Look| {
            let so = [m.prepare()];
            let surface = ph2d_field_render::Surfaces {
                all: &so,
                owners: None,
            };
            let light = Lighting { lamps: &lamps, sky };
            shade_render(&g, &cam, &surface, &light, look, BG)
        };
    let pinta = |m: ph2d_material::OpenPbr, sky: &(dyn ph2d_material::Environment + Sync)| {
        pinta_com(m, sky, Look::default())
    };
    let neutra = Look {
        exposure_stops: 0.0,
        view: ph2d_view_transform::ViewTransform::Neutral,
    };
    let mat = |rough: f32, metal: f32| ph2d_material::OpenPbr {
        base_metalness: metal,
        base_color: [0.95, 0.95, 0.95],
        specular_roughness: rough,
        ..ph2d_material::OpenPbr::default()
    };
    let (wu, hu) = (w as usize, h as usize);
    let estrutura = |px: &[u8]| -> f64 {
        let c = px.as_chunks::<4>().0;
        let (mut s, mut n) = (0.0_f64, 0_usize);
        for y in 1..hu - 1 {
            for x in 1..wu - 1 {
                let i = y * wu + x;
                let viz = [i - 1, i + 1, i - wu, i + wu];
                if !g.hit[i] || viz.iter().any(|&j| !g.hit[j]) {
                    continue;
                }
                s += (viz.iter().map(|&j| f64::from(c[j][1])).sum::<f64>()
                    - 4.0 * f64::from(c[i][1]))
                .abs();
                n += 1;
            }
        }
        s / n as f64
    };
    let moveu = |a: &[u8], b: &[u8]| -> f64 {
        let (ca, cb) = (a.as_chunks::<4>().0, b.as_chunks::<4>().0);
        let (mut n, mut peca) = (0_usize, 0_usize);
        for i in 0..ca.len() {
            if !g.hit[i] {
                continue;
            }
            peca += 1;
            if (0..3).any(|c| ca[i][c].abs_diff(cb[i][c]) > 8) {
                n += 1;
            }
        }
        100.0 * n as f64 / peca as f64
    };
    let branco_de = |m: ph2d_material::OpenPbr,
                     sky: &(dyn ph2d_material::Environment + Sync),
                     view: ph2d_view_transform::ViewTransform,
                     stops: f32|
     -> u32 {
        let so = [m.prepare()];
        let surface = ph2d_field_render::Surfaces {
            all: &so,
            owners: None,
        };
        let light = Lighting { lamps: &lamps, sky };
        let look = Look {
            exposure_stops: stops,
            view,
        };
        let px = shade_render(&g, &cam, &surface, &light, look, BG);
        px.as_chunks::<4>()
            .0
            .iter()
            .enumerate()
            .filter(|(i, p)| g.hit[*i] && p[0] == 255 && p[1] == 255 && p[2] == 255)
            .count() as u32
    };
    let branco_e_media = |px: &[u8]| -> (u32, f64) {
        let (mut b, mut s, mut n) = (0_u32, 0.0_f64, 0_usize);
        for (i, p) in px.as_chunks::<4>().0.iter().enumerate() {
            if !g.hit[i] {
                continue;
            }
            n += 1;
            s += f64::from(p[1]);
            b += u32::from(p[0] == 255 && p[1] == 255 && p[2] == 255);
        }
        (b, s / n as f64)
    };

    println!(
        "            ·— STANDARD —·———— vista NEUTRAL ————·— quanto a caixa MOVEU (byte) —·— branco —·"
    );
    println!(
        "raio · fracção · espelho ·  Rough ·   espelho ·     giz ·  Rough · espelho ·  giz · omis · mérito · Std 0 ·  Ntr"
    );
    // ⭐⭐ **A FIGURA DE MÉRITO** — quanto a caixa move um ESPELHO a dividir por quanto ela move uma
    // superfície DIFUSA. *Um espelho tem de mudar (é para isso que ela existe) e o giz não tem nada
    // para reflectir, logo tudo o que ele se mexe é a peça que o dono já aprovou a mudar de cara.*
    let nu = Studio::bare_ramp();
    let mexeu = |m: ph2d_material::OpenPbr, sky: &(dyn ph2d_material::Environment + Sync)| -> f64 {
        let (a, b) = (pinta_com(m, sky, neutra), pinta_com(m, &nu, neutra));
        let (ca, cb) = (a.as_chunks::<4>().0, b.as_chunks::<4>().0);
        let (mut s, mut n) = (0.0_f64, 0_usize);
        for i in 0..ca.len() {
            if !g.hit[i] {
                continue;
            }
            n += 1;
            for c in 0..3 {
                s += f64::from(ca[i][c].abs_diff(cb[i][c]));
            }
        }
        s / (3 * n) as f64
    };
    let linha = |nome: String, sky: &(dyn ph2d_material::Environment + Sync)| {
        let espelho = pinta(mat(0.05, 1.0), sky);
        let d30 = pinta(mat(0.30, 0.0), sky);
        let d05 = pinta(mat(0.05, 0.0), sky);
        // O branco chapado mede-se no material de OMISSÃO, que é o que o §8 mediu.
        let (branco, _media) = branco_e_media(&pinta(ph2d_material::OpenPbr::default(), sky));
        let neutro = branco_de(
            ph2d_material::OpenPbr::default(),
            sky,
            ph2d_view_transform::ViewTransform::Neutral,
            0.0,
        );
        let _menos1 = branco_de(
            ph2d_material::OpenPbr::default(),
            sky,
            ph2d_view_transform::ViewTransform::Standard,
            -1.0,
        );
        // ⚠️ **A régua da estrutura mede-se na vista que NÃO CORTA.** Sob a `Standard` um planalto
        // saturado tem `∇² = 0`, logo o corte ESCONDE exactamente o efeito que se está a medir — a
        // régua estaria a ser lida através do defeito que ela devia acusar.
        let n_espelho = pinta_com(mat(0.05, 1.0), sky, neutra);
        let n_giz = pinta_com(mat(1.0, 0.0), sky, neutra);
        let n30 = pinta_com(mat(0.30, 0.0), sky, neutra);
        let n05 = pinta_com(mat(0.05, 0.0), sky, neutra);
        let m_esp = mexeu(mat(0.05, 1.0), sky);
        let m_giz = mexeu(mat(1.0, 0.0), sky);
        let m_dft = mexeu(ph2d_material::OpenPbr::default(), sky);
        println!(
            "{nome} · {:9.3} · {:6.2} % · {:9.3} · {:8.3} · {:6.2} % · {m_esp:6.1} · {m_giz:5.1} · {m_dft:5.1} · {:5.2} · {branco:6} · {neutro:4}",
            estrutura(&espelho),
            moveu(&d30, &d05),
            estrutura(&n_espelho),
            estrutura(&n_giz),
            moveu(&n30, &n05),
            m_esp / m_giz.max(0.01),
        );
    };

    linha("  — ·     — ".into(), &Studio::bare_ramp());
    // ⭐ **O QUE A CONTAGEM DE AMOSTRAS VALE EM PIXELS** — a pergunta que decide a const, feita na
    // única grandeza que o dono vê. A tabela cara é a referência.
    {
        let caro = BoxPrefilter::build_with_samples(Softbox::PRODUCT, SOFTBOX_SHARE, 8192);
        for n in [512_u32, 1024, 2048] {
            let barato = BoxPrefilter::build_with_samples(Softbox::PRODUCT, SOFTBOX_SHARE, n);
            let (a, b) = (
                Studio {
                    softbox: Some(&barato),
                },
                Studio {
                    softbox: Some(&caro),
                },
            );
            let mut pior = 0_u32;
            let mut soma = 0.0_f64;
            let mut np = 0_usize;
            for rough in [0.05_f32, 0.3, 0.7, 1.0] {
                for metal in [0.0_f32, 1.0] {
                    let (x, y) = (pinta(mat(rough, metal), &a), pinta(mat(rough, metal), &b));
                    let (cx, cy) = (x.as_chunks::<4>().0, y.as_chunks::<4>().0);
                    for i in 0..cx.len() {
                        if !g.hit[i] {
                            continue;
                        }
                        np += 1;
                        for c in 0..3 {
                            let d = u32::from(cx[i][c].abs_diff(cy[i][c]));
                            pior = pior.max(d);
                            soma += f64::from(d);
                        }
                    }
                }
            }
            println!(
                "amostras {n:5} contra 8192: |Δ| médio {:.3} byte · pior {pior} byte",
                soma / (3 * np) as f64
            );
        }
    }
    for raio in [12.0_f32, 18.0, 25.0, 35.0, 50.0] {
        for share in [0.20_f32, 0.35, 0.50, 0.65] {
            let t = BoxPrefilter::build(
                Softbox {
                    radius_deg: raio,
                    rim_deg: SOFTBOX_RIM_DEG,
                },
                share,
            );
            linha(
                format!("{raio:4.0} · {share:6.2}"),
                &Studio { softbox: Some(&t) },
            );
        }
    }
}

/// ⏱️ **SONDA — o preço da construção da tabela**, que é pago uma vez, no primeiro quadro de Render.
///
/// ⚠️ **Mínimo de `5` corridas com o `/proc/loadavg` ao lado** (`CLAUDE.md` §5.0) — e corre-se em
/// `--release`, que é o perfil em que o dono a paga.
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_what_the_table_costs_to_build() {
    use std::time::Instant;
    println!(
        "carga: {}",
        std::fs::read_to_string("/proc/loadavg").unwrap().trim()
    );
    println!("amostras · construção (mín de 5 · mediana)");
    for n in [512_u32, 1024, 2048] {
        let mut v: Vec<f64> = (0..5)
            .map(|_| {
                let t = Instant::now();
                let t2 = BoxPrefilter::build_with_samples(Softbox::PRODUCT, SOFTBOX_SHARE, n);
                std::hint::black_box(t2.specular(0.09, 0.9));
                t.elapsed().as_secs_f64() * 1e3
            })
            .collect();
        v.sort_by(f64::total_cmp);
        println!("{n:8} · {:8.1} ms · {:8.1} ms", v[0], v[v.len() / 2]);
    }
}

/// ⏱️ **SONDA — o que a caixa custa POR QUADRO.**
///
/// A rampa é aritmética pura; a caixa acrescenta **dois `sqrt` e uma leitura bilinear** por chamada
/// de `radiance`, e o sombreamento chama-a três vezes por pixel de peça (a dieléctrica, a Schlick e
/// o verniz).
///
/// ⚠️ **Mínimo de `7` corridas com o `/proc/loadavg` ao lado, e em `--release`** (`CLAUDE.md` §5.0).
#[test]
#[ignore = "sonda de medição: imprime uma tabela, não afirma nada"]
fn measure_what_the_box_costs_per_frame() {
    use ph2d_field_render::{Lighting, Orbit, shade_render, trace};
    use ph2d_view_transform::Look;
    use std::time::Instant;

    const BG: [u8; 4] = [12, 34, 56, 200];
    let (w, h) = (640_u32, 360_u32);
    let cam = Orbit::default();
    let doc = ph2d_field::FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            ph2d_field::Primitive::Sphere { radius: 0.6 },
            ph2d_field::Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("esfera");
    let reg = ph2d_field_eval::hybrid::Registry::new();
    let g = trace(&doc, &reg, &cam, w, h);
    let lamps = crate::render_light::lamps(&ph2d_light::LightRig::default());
    let so = [ph2d_material::OpenPbr::default().prepare()];
    let surface = ph2d_field_render::Surfaces {
        all: &so,
        owners: None,
    };
    println!(
        "carga: {}",
        std::fs::read_to_string("/proc/loadavg").unwrap().trim()
    );
    // A tabela do produto construída FORA do relógio — ela é paga uma vez, e tem sonda própria.
    let _ = Studio::of_the_product().radiance([0.0, 1.0, 0.0], 0.09);
    let med = |sky: &(dyn ph2d_material::Environment + Sync)| -> (f64, f64) {
        let light = Lighting { lamps: &lamps, sky };
        let _ = shade_render(&g, &cam, &surface, &light, Look::default(), BG);
        let mut v: Vec<f64> = (0..7)
            .map(|_| {
                let t = Instant::now();
                let px = shade_render(&g, &cam, &surface, &light, Look::default(), BG);
                std::hint::black_box(px[0]);
                t.elapsed().as_secs_f64() * 1e3
            })
            .collect();
        v.sort_by(f64::total_cmp);
        (v[0], v[v.len() / 2])
    };
    let (a_min, a_med) = med(&Studio::bare_ramp());
    let (b_min, b_med) = med(&Studio::of_the_product());
    println!("céu            ·  mín   · mediana");
    println!("rampa nua      · {a_min:6.3} · {a_med:6.3} ms");
    println!("com a caixa    · {b_min:6.3} · {b_med:6.3} ms");
    println!(
        "delta          · {:+6.3} ms ({:+.1} %) sobre um quadro de 16,7 ms",
        b_min - a_min,
        100.0 * (b_min - a_min) / a_min
    );
}
