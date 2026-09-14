//! Os gates da tradução da luz — o sinal, o `π` e o céu. Ver [`super`].

use super::*;
use ph2d_material::Environment;

fn luminance(c: [f32; 3]) -> f32 {
    0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2]
}

/// ⭐ **Uma lâmpada de intensidade `1` chega como `π`** — o contrato do rig («o plano de frente
/// devolve `1`») escrito na convenção do MaterialX.
#[test]
fn a_lamp_of_intensity_one_arrives_as_pi() {
    let mut rig = ph2d_light::LightRig::default();
    rig.lights[0].color = [1.0; 3];
    rig.lights[0].intensity = 1.0;
    let l = lamps(&rig);
    assert_eq!(l.len(), 1, "o rig de omissão tem UMA lâmpada acesa");
    for c in l[0].radiance {
        assert!(
            (c - core::f32::consts::PI).abs() < 1.0e-6,
            "{:?}",
            l[0].radiance
        );
    }
}

/// ⭐⭐ **A lâmpada de omissão ilumina de CIMA e da ESQUERDA** — o sinal de `y`, provado contra o que
/// o rig promete por escrito (*«superior-esquerda a 30°»*, o `Light::KEY` afinado pelo Enio).
///
/// ⚠️ Sem a negação de `y` a mesma lâmpada acenderia a peça por BAIXO — a regressão que a casa já
/// pagou uma vez entre a tinta e a escultura.
#[test]
fn the_default_lamp_lights_from_the_upper_left() {
    let l = lamps(&ph2d_light::LightRig::default());
    let to_light = l[0].to_light;
    assert!(
        to_light[1] > 0.0,
        "a lâmpada está por BAIXO em espaço de vista: {to_light:?}"
    );
    assert!(
        to_light[0] < 0.0,
        "a lâmpada está à DIREITA em espaço de vista: {to_light:?}"
    );
    assert!(
        to_light[2] > 0.0,
        "a lâmpada está ATRÁS da peça: {to_light:?}"
    );

    // E o render diz o mesmo: o topo acende mais do que a base.
    let s = ph2d_material::OpenPbr::default().prepare();
    let v = [0.0, 0.0, 1.0];
    let top = s.direct([0.0, 0.8, 0.6], v, to_light, l[0].radiance);
    let bottom = s.direct([0.0, -0.8, 0.6], v, to_light, l[0].radiance);
    assert!(
        luminance(top) > luminance(bottom),
        "topo {top:?} · base {bottom:?}"
    );
}

/// ⭐ **O céu é mais claro em cima** — o céu de estúdio é o `env_ambient` da casa, com o sinal certo.
#[test]
fn the_studio_sky_is_brighter_above() {
    let sky = StudioSky;
    assert!(
        luminance(sky.irradiance([0.0, 1.0, 0.0])) > luminance(sky.irradiance([0.0, -1.0, 0.0]))
    );
    assert!(
        luminance(sky.radiance([0.0, 1.0, 0.0], 0.0))
            > luminance(sky.radiance([0.0, -1.0, 0.0], 0.0))
    );
}

/// ⭐⭐ **A irradiância é a média-cosseno da radiância** — o `3/2` que desfaz a convolução do
/// `ENV_SLOPE` provado por INTEGRAÇÃO, e não pela aritmética que o escreveu.
///
/// ⚠️ É a propriedade que liga as duas perguntas do [`Environment`]: se elas discordassem, a difusa e
/// o especular do mesmo céu acenderiam a peça com duas luzes diferentes.
#[test]
fn the_irradiance_is_the_cosine_mean_of_the_radiance() {
    let sky = StudioSky;
    let normais: [[f32; 3]; 4] = [
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.6, 0.48, 0.64],
    ];
    for n in normais {
        // A base ortonormal da normal, e a quadratura do hemisfério em (cos θ, φ).
        let helper = if n[0].abs() < 0.9 {
            [1.0, 0.0, 0.0]
        } else {
            [0.0, 1.0, 0.0]
        };
        let cross = |a: [f32; 3], b: [f32; 3]| {
            [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ]
        };
        let norm = |a: [f32; 3]| {
            let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
            [a[0] / l, a[1] / l, a[2] / l]
        };
        let t = norm(cross(helper, n));
        let b = cross(n, t);
        let (rings, sectors) = (200, 200);
        let mut sum = [0.0_f64; 3];
        let mut weight = 0.0_f64;
        for i in 0..rings {
            let mu = (i as f32 + 0.5) / rings as f32; // cos θ
            let sin = (1.0 - mu * mu).sqrt();
            for j in 0..sectors {
                let phi = (j as f32 + 0.5) / sectors as f32 * core::f32::consts::TAU;
                let dir =
                    [0, 1, 2].map(|k| n[k] * mu + t[k] * sin * phi.cos() + b[k] * sin * phi.sin());
                let l = sky.radiance(dir, 0.0);
                for k in 0..3 {
                    sum[k] += f64::from(l[k] * mu);
                }
                weight += f64::from(mu);
            }
        }
        let mean = sum.map(|s| (s / weight) as f32);
        let e = sky.irradiance(n);
        for k in 0..3 {
            assert!(
                (mean[k] - e[k]).abs() < 1.0e-3,
                "normal {n:?}: a média-cosseno da radiância dá {mean:?} e a irradiância {e:?}"
            );
        }
    }
}

/// ⏱️ **SONDA (`--ignored`): o que o modo RENDER custa, e que luz ele DEVOLVE** — com o rig, o céu,
/// o material e o olhar **do produto**.
///
/// ⛔⛔ **Ela mudou de crate em 2026-09-13, e a mudança É a correcção.** Ela nasceu na
/// `ph2d-field-render`, que **não alcança** este ficheiro — e por isso escrevia a luz à mão: uma
/// lâmpada com a direcção em literal e um céu **CONSTANTE**, onde o produto tem um [`StudioSky`]
/// que é um GRADIENTE. ⇒ a tabela de luz do `docs/Render3d/05` §6 media um programa que ninguém
/// corre, e errava no sentido optimista (`7,1 %` da peça cortada a `0` stops, contra os `10,9 %`
/// reais). *Uma segunda cópia escrita à mão do valor que uma porta produz.*
///
/// ⚠️ **Não é um gate — não há barra aqui.** É a medição que responde *«isto ainda é interactivo?»*
/// e *«a peça sai preta ou estourada?»* antes de o dono olhar para ela. Corra-a com a máquina calma
/// (`CLAUDE.md` §5.0: nenhum relógio desta workstation vale acima de `load ~5`).
#[test]
#[ignore = "sonda de medição"]
fn measure_what_the_render_mode_costs_and_paints() {
    use ph2d_field_render::{Lighting, Matcap, Orbit, shade_render, trace};
    use ph2d_view_transform::{Look, ViewTransform};
    use std::time::Instant;

    const BG: [u8; 4] = [12, 34, 56, 200];
    let (w, h) = (640, 360);
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
    let t = Instant::now();
    let g = trace(&doc, &reg, &cam, w, h);
    let trace_ms = t.elapsed().as_secs_f64() * 1e3;

    // ⭐ **A luz é a do PRODUTO, pelas mesmas portas que o `smoke_draw` chama.**
    let lamps = lamps(&ph2d_light::LightRig::default());
    let surface = ph2d_material::OpenPbr::default().prepare();
    let light = Lighting {
        lamps: &lamps,
        sky: &StudioSky,
    };
    let t = Instant::now();
    let render_px = shade_render(&g, &cam, &surface, &light, Look::default(), BG);
    let render_ms = t.elapsed().as_secs_f64() * 1e3;

    let texels = vec![0.5_f32; 2 * 2 * 3];
    let m = Matcap {
        side: 2,
        rgb_linear: &texels,
    };
    let t = Instant::now();
    let _ = ph2d_field_render::shade(&g, &m, BG);
    let matcap_ms = t.elapsed().as_secs_f64() * 1e3;

    // ⚠️ **Saturado é os TRÊS canais em `255`**, e não só o verde: um canal no tecto ainda tem cor,
    // e o que o olho lê como «branco chapado» é a peça a perder a forma nos três.
    let stats = |px: &[u8]| -> (f64, u32, u32, usize) {
        let (mut soma, mut verde, mut branco, mut n) = (0.0_f64, 0_u32, 0_u32, 0_usize);
        for (i, p) in px.as_chunks::<4>().0.iter().enumerate() {
            if !g.hit[i] {
                continue;
            }
            n += 1;
            soma += f64::from(p[1]);
            verde += u32::from(p[1] == 255);
            branco += u32::from(p[0] == 255 && p[1] == 255 && p[2] == 255);
        }
        (soma / n as f64, verde, branco, n)
    };
    let (_, _, branco0, pixels) = stats(&render_px);
    println!("esfera {w}x{h} · {pixels} pixels de peça · traçado {trace_ms:.1} ms");
    println!(
        "sombrear: matcap (2x2 texels) {matcap_ms:.2} ms · render {render_ms:.2} ms · \
         branco chapado {branco0}"
    );
    for stops in [-2.0, -1.0, 0.0, 1.0, 2.0] {
        let de = |view| {
            let look = Look {
                exposure_stops: stops,
                view,
            };
            stats(&shade_render(&g, &cam, &surface, &light, look, BG))
        };
        let (s_media, s_verde, s_branco, _) = de(ViewTransform::Standard);
        let (n_media, n_verde, n_branco, _) = de(ViewTransform::Neutral);
        println!(
            "  {stops:+.0} stop · Standard media {s_media:6.1} verde=255 {s_verde:6} branco {s_branco:6} \
             · Neutral media {n_media:6.1} verde=255 {n_verde:6} branco {n_branco:6}"
        );
    }
}
