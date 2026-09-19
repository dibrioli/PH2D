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

/// ⭐⭐⭐ **O MODELADOR ABRE NUMA VISTA QUE A CAIXA NÃO ESTOURA.**
///
/// # ⛔⛔ Porque este gate nasceu DEPOIS da wave, e do report do dono
///
/// A primeira redacção da §24 apresentou ao dono, como decisão por tomar, **uma decisão que ele já
/// tinha tomado nessa manhã e que já estava shipada** ([`crate::shading::OPENING_LOOK`], §15). O
/// mecanismo do erro é o do `CLAUDE.md` §5.0: as sondas desta wave pintavam com
/// `ph2d_view_transform::Look::default()` — a omissão do **TIPO**, que é `Standard` — e eu li a
/// coluna delas como sendo o produto. *Uma sonda que chama o default do tipo mede outro programa
/// que o pill.*
///
/// ⚠️ **E havia gate a mais e a menos ao mesmo tempo:** o `shading_tests` já proibia o `view.rs` de
/// voltar ao `Look::default()`, logo a DECISÃO estava presa — mas **nada ligava essa decisão ao que
/// ela protege**. Uma catraca sobre o valor não diz porque é que o valor importa, e foi por isso
/// que eu a pude ler como aberta.
///
/// ⇒ este gate afirma a CONSEQUÊNCIA, sobre o caminho do produto: com o céu de estúdio e o material
/// de omissão, o olhar de abertura **não satura um único pixel**.
///
/// ⚠️ **Com o controlo do outro lado**, senão ele passaria sobre um céu sem caixa nenhuma: na
/// `Standard` a `0` stops a mesma peça satura às centenas. *É esse par que torna o número do
/// `OPENING_LOOK` uma lei em vez de uma preferência.*
#[test]
fn the_modeller_opens_in_a_view_the_softbox_does_not_blow_out() {
    use ph2d_field_render::{Lighting, Orbit, shade_render, trace};

    const BG: [u8; 4] = [0, 0, 0, 0];
    let (w, h) = (320_u32, 180_u32);
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
    let chapado =
        |sky: &(dyn ph2d_material::Environment + Sync), look: ph2d_view_transform::Look| -> usize {
            let light = Lighting {
                lamps: &lamps,
                points: &[],
                sky,
                shadows: None,
            };
            shade_render(
                &g,
                &cam,
                &surface,
                &light,
                &ph2d_field_render::Presentation::of(look),
                BG,
            )
            .as_chunks::<4>()
            .0
            .iter()
            .enumerate()
            .filter(|(i, p)| g.hit[*i] && p[0] == 255 && p[1] == 255 && p[2] == 255)
            .count()
        };
    let estudio = Studio::of_the_product();
    let rampa = Studio::bare_ramp();
    let pecas = g.hit.iter().filter(|h| **h).count();
    assert!(pecas > 5_000, "a fixtura encolheu: {pecas} pixels de peça");
    assert_eq!(
        chapado(&estudio, crate::shading::OPENING_LOOK),
        0,
        "o olhar de abertura passou a estourar sobre a caixa de luz — ou a vista mudou, ou a caixa \
         ficou mais concentrada do que a §24.6 mediu"
    );

    // ⭐⭐ **O CONTROLO, e a PRIMEIRA REDACÇÃO DELE ESTAVA ERRADA — uma mutação provou-o.**
    //
    // Ela pedia só *«na `Standard` isto satura»*, e a rampa NUA já satura `5 180` px sozinha ⇒
    // apagar a caixa do produto deixava o gate **verde**. *Um controlo que o sujeito da mutação não
    // toca não é um controlo: é uma segunda asserção sobre outra coisa.*
    //
    // ⇒ o controlo é o A/B da PRÓPRIA CAIXA: numa vista que corta, ela tem de acrescentar corte.
    let com = chapado(&estudio, ph2d_view_transform::Look::default());
    let sem = chapado(&rampa, ph2d_view_transform::Look::default());
    assert!(
        com > sem + sem / 5,
        "a caixa só acrescentou {} px de corte sobre {sem} numa vista que CORTA: ela deixou de \
         concentrar energia, e a metade de cima deste gate deixou de afirmar seja o que for",
        com.saturating_sub(sem)
    );
}
