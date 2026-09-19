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

/// ⭐ **As SONDAS de medição** — irmão por responsabilidade e por tecto de LOC.
#[path = "render_light_sondas.rs"]
mod sondas;
