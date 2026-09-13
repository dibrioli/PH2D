//! Os gates da superfície — o oráculo corrido (`fixtures/`) e as propriedades que ele não cobre.

use super::*;

const FIXTURE: &str = include_str!("../fixtures/materialx_openpbr_direct.txt");

/// **A barra relativa contra o oráculo.** Medido em 2026-09-13, este port em `f32`: luz directa pior
/// `9,8e-6` (945 amostras), luz do céu pior `1,6e-6` (315) — a mesma ordem da réplica `f64` da
/// composição (`1,05e-5`), isto é, a precisão `f32` da placa; e o material ERRADO fica a `595`.
/// ⚠️ **A NDF com `1 − (H·N)²` leu `1,12e-4` no espelho** (ver `bsdf::ggx_ndf_isotropic`): a barra a
/// `10×` o chão é o que apanhou essa perda de precisão, e é por isso que ela não sobe.
const ORACLE_BAR: f32 = 1.0e-4;

/// O céu constante da fixture, como um [`Environment`].
struct Uniform(Rgb);

impl Environment for Uniform {
    fn radiance(&self, _dir: Rgb, _alpha: f32) -> Rgb {
        self.0
    }
    fn irradiance(&self, _n: Rgb) -> Rgb {
        self.0
    }
}

struct Oracle {
    defaults: Vec<(String, Vec<f32>)>,
    materials: Vec<OpenPbr>,
    lights: Vec<(Rgb, Rgb, f32)>,
    direct: Vec<(usize, usize, Rgb, Rgb, Rgb)>,
    sky: Rgb,
    indirect: Vec<(usize, Rgb, Rgb, Rgb)>,
}

fn nums(tokens: &[&str]) -> Vec<f32> {
    tokens
        .iter()
        .map(|t| t.parse().expect("a fixture tem números"))
        .collect()
}

fn v3(x: &[f32]) -> Rgb {
    [x[0], x[1], x[2]]
}

/// Escreve um parâmetro pelo NOME da nodedef — `false` se a crate não tem esse campo.
fn set(m: &mut OpenPbr, name: &str, v: &[f32]) -> bool {
    let colour = || v3(v);
    match name {
        "base_weight" => m.base_weight = v[0],
        "base_color" => m.base_color = colour(),
        "base_diffuse_roughness" => m.base_diffuse_roughness = v[0],
        "base_metalness" => m.base_metalness = v[0],
        "specular_weight" => m.specular_weight = v[0],
        "specular_color" => m.specular_color = colour(),
        "specular_roughness" => m.specular_roughness = v[0],
        "specular_ior" => m.specular_ior = v[0],
        "coat_weight" => m.coat_weight = v[0],
        "coat_color" => m.coat_color = colour(),
        "coat_roughness" => m.coat_roughness = v[0],
        "coat_ior" => m.coat_ior = v[0],
        "coat_darkening" => m.coat_darkening = v[0],
        "emission_luminance" => m.emission_luminance = v[0],
        "emission_color" => m.emission_color = colour(),
        _ => return false,
    }
    true
}

fn parse_assignments(tokens: &[&str]) -> Vec<(String, Vec<f32>)> {
    tokens
        .iter()
        .filter_map(|kv| {
            let (k, val) = kv.split_once('=')?;
            let parsed: Result<Vec<f32>, _> = val.split(',').map(str::parse).collect();
            parsed.ok().map(|v| (k.to_string(), v))
        })
        .collect()
}

fn oracle() -> Oracle {
    let mut o = Oracle {
        defaults: Vec::new(),
        materials: Vec::new(),
        lights: Vec::new(),
        direct: Vec::new(),
        sky: [0.0; 3],
        indirect: Vec::new(),
    };
    for line in FIXTURE.lines() {
        let t: Vec<&str> = line.split_whitespace().collect();
        match t.first().copied() {
            Some("D") => o.defaults = parse_assignments(&t[1..]),
            Some("M") => {
                let mut m = OpenPbr::default();
                for (k, v) in parse_assignments(&t[2..]) {
                    assert!(
                        set(&mut m, &k, &v),
                        "a fixture usa `{k}`, que a crate não tem"
                    );
                }
                o.materials.push(m);
            }
            Some("L") => {
                let f = nums(&t[2..]);
                o.lights.push((v3(&f[0..3]), v3(&f[3..6]), f[6]));
            }
            Some("S") => {
                let f = nums(&t[3..]);
                o.direct.push((
                    t[1].parse().expect("índice"),
                    t[2].parse().expect("índice"),
                    v3(&f[0..3]),
                    v3(&f[3..6]),
                    v3(&f[6..9]),
                ));
            }
            Some("E") => o.sky = v3(&nums(&t[1..])),
            Some("I") => {
                let f = nums(&t[2..]);
                o.indirect.push((
                    t[1].parse().expect("índice"),
                    v3(&f[0..3]),
                    v3(&f[3..6]),
                    v3(&f[6..9]),
                ));
            }
            _ => {}
        }
    }
    o
}

/// O erro relativo, com piso de `1e-3` no denominador (um preto não pode inflar a razão).
fn relative(ours: Rgb, oracle: Rgb) -> f32 {
    let e = (0..3)
        .map(|k| (ours[k] - oracle[k]).abs())
        .fold(0.0_f32, f32::max);
    e / oracle.iter().copied().fold(1.0e-3_f32, f32::max)
}

fn direct_of(m: &OpenPbr, light: (Rgb, Rgb, f32), n: Rgb, v: Rgb) -> Rgb {
    let s = m.prepare();
    let (dir, colour, intensity) = light;
    let to_light = dir.map(|d| -d);
    let radiance = colour.map(|c| c * intensity);
    bsdf::add3(s.direct(n, v, to_light, radiance), s.emission(n, v))
}

/// ⭐ **O `Default` é a nodedef do padrão instalado**, campo a campo — lida da fixture, nunca copiada.
#[test]
fn the_default_is_the_nodedef_of_the_installed_standard() {
    let o = oracle();
    let mut from_nodedef = OpenPbr::default();
    let mut ours = 0;
    for (k, v) in &o.defaults {
        if set(&mut from_nodedef, k, v) {
            ours += 1;
        }
    }
    assert_eq!(
        ours, 15,
        "a nodedef tem de dar valor aos 15 campos da crate"
    );
    assert_eq!(OpenPbr::default(), from_nodedef);
}

/// ⭐⭐ **A luz directa é a do MaterialX**, em todo pixel de toda esfera da fixture.
///
/// ⚠️ **As duas metades**: a lei certa passa por baixo da barra, e sombrear toda amostra com o
/// material ERRADO (o padrão) fica muito acima dela — senão uma fixture de materiais parecidos
/// aprovaria qualquer lei.
#[test]
fn the_direct_light_is_the_oracles_on_every_sphere() {
    let o = oracle();
    assert_eq!(o.materials.len(), 7);
    assert_eq!(o.lights.len(), 3);
    assert_eq!(o.direct.len(), 945, "a fixture perdeu ou ganhou amostras");
    let (mut worst, mut wrong) = ((0.0_f32, 0, 0), 0.0_f32);
    for &(mi, li, n, v, expected) in &o.direct {
        let e = relative(direct_of(&o.materials[mi], o.lights[li], n, v), expected);
        if e > worst.0 {
            worst = (e, mi, li);
        }
        wrong = wrong.max(relative(
            direct_of(&OpenPbr::default(), o.lights[li], n, v),
            expected,
        ));
    }
    // A medição fica impressa (`--nocapture`): é ela que recalibra a barra, não a memória de alguém.
    eprintln!(
        "luz directa: pior {:e} (material {}, luz {}) · material errado {wrong:e}",
        worst.0, worst.1, worst.2
    );
    assert!(
        worst.0 <= ORACLE_BAR,
        "a luz directa afasta-se do oráculo {:e} (barra {ORACLE_BAR:e}) no material {} com a luz {}",
        worst.0,
        worst.1,
        worst.2
    );
    assert!(
        wrong > 100.0 * ORACLE_BAR,
        "o material errado ficou a {wrong:e} — a fixture não distingue materiais"
    );
}

/// ⭐⭐ **A luz do céu é a do MaterialX** (`PREFILTER` sobre um céu constante).
#[test]
fn the_sky_light_is_the_oracles_on_every_sphere() {
    let o = oracle();
    assert_eq!(o.indirect.len(), 315, "a fixture perdeu ou ganhou amostras");
    let sky = Uniform(o.sky);
    let mut worst = (0.0_f32, 0);
    for &(mi, n, v, expected) in &o.indirect {
        let s = o.materials[mi].prepare();
        let ours = bsdf::add3(s.indirect(n, v, &sky), s.emission(n, v));
        let e = relative(ours, expected);
        if e > worst.0 {
            worst = (e, mi);
        }
    }
    eprintln!("luz do céu: pior {:e} (material {})", worst.0, worst.1);
    assert!(
        worst.0 <= ORACLE_BAR,
        "a luz do céu afasta-se do oráculo {:e} (barra {ORACLE_BAR:e}) no material {}",
        worst.0,
        worst.1
    );
}

/// ⭐ **O *furnace test*** (`docs/Render3d/03` W2): sob um céu branco uniforme, uma superfície branca
/// devolve branco — a qualquer rugosidade e a qualquer ângulo. É o teste que apanha energia perdida.
///
/// ⚠️ **A barra é `1e-5`, e a medição é `1,19e-7`** (2026-09-13: dois ULP de `f32` em `1,0`, na
/// dieléctrica a rugosidade `0,1`). A lei do MaterialX fecha **por construção** — a difusa compensada
/// devolve `1` com cor `1`, e a camada especular devolve `FG·comp + (1 − FG·comp)·1` —, então o que
/// resta é arredondamento; qualquer termo de compensação partido afasta-se ordens de grandeza.
#[test]
fn a_white_surface_under_a_white_sky_returns_white() {
    let sky = Uniform([1.0; 3]);
    let mut worst = (0.0_f32, 0.0_f32, 0.0_f32, "");
    for (label, metal) in [("dielectric", 0.0), ("metal", 1.0)] {
        for step in 0..=10 {
            let roughness = step as f32 / 10.0;
            let m = OpenPbr {
                base_color: [1.0; 3],
                base_metalness: metal,
                specular_roughness: roughness,
                base_diffuse_roughness: roughness,
                ..OpenPbr::default()
            };
            let s = m.prepare();
            for k in 1..=20 {
                let ndv = k as f32 / 20.0;
                let v = [(1.0 - ndv * ndv).sqrt(), 0.0, ndv];
                let out = s.indirect([0.0, 0.0, 1.0], v, &sky);
                let dev = (out[1] - 1.0).abs();
                if dev > worst.0 {
                    worst = (dev, roughness, ndv, label);
                }
            }
        }
    }
    assert!(
        worst.0 <= 1.0e-5,
        "o furnace test perdeu {:.4} ({} a rugosidade {} e n·v {})",
        worst.0,
        worst.3,
        worst.1,
        worst.2
    );
}
