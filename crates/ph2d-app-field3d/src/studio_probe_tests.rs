//! As SONDAS do [`super`] — as medições de que os números do módulo saíram.
//!
//! ⚠️ **Elas vivem à parte dos GATES por responsabilidade, e o corte foi FORÇADO pelo tecto de LOC**
//! (`715` contra `700`): a cura de um tecto é o CORTE, nunca uma entrada nova no `FILE_OVERAGE_OK`
//! (`CLAUDE.md` §5.0). ⭐ E o corte caiu no sítio certo por si: um gate **afirma** uma lei e uma sonda
//! **imprime uma tabela e não afirma nada** — são dois leitores diferentes (o portão de fecho e
//! quem está a escolher uma constante).

use super::*;

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
    // ⛔⛔ **O OLHAR DO PRODUTO É O `OPENING_LOOK`, NUNCA O `Look::default()`.** O segundo é a
    // omissão do TIPO (`Standard`, a identidade para luz em `0..=1`); o modelador abre em `Neutral`
    // desde a §15, por decisão do dono. *Uma sonda que chama o default do tipo mede outro programa
    // que o pill* — e foi assim que a primeira redacção desta wave apresentou ao dono, como decisão
    // por tomar, uma decisão que ele já tinha tomado e que já estava shipada.
    let produto = crate::shading::OPENING_LOOK;
    let pinta = |m: ph2d_material::OpenPbr, sky: &(dyn ph2d_material::Environment + Sync)| {
        pinta_com(m, sky, produto)
    };
    // O outro lado do A/B: o que quem trocar a vista para `Standard` na fileira do Shading vê.
    let padrao_do_tipo = Look::default();
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
        let (a, b) = (pinta_com(m, sky, produto), pinta_com(m, &nu, produto));
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
        let n_espelho = pinta_com(mat(0.05, 1.0), sky, padrao_do_tipo);
        let n_giz = pinta_com(mat(1.0, 0.0), sky, padrao_do_tipo);
        let n30 = pinta_com(mat(0.30, 0.0), sky, padrao_do_tipo);
        let n05 = pinta_com(mat(0.05, 0.0), sky, padrao_do_tipo);
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
