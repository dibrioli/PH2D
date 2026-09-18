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
        "subsurface_weight" => m.subsurface_weight = v[0],
        "subsurface_color" => m.subsurface_color = colour(),
        "subsurface_radius" => m.subsurface_radius = v[0],
        "subsurface_radius_scale" => m.subsurface_radius_scale = colour(),
        "subsurface_scatter_anisotropy" => m.subsurface_scatter_anisotropy = v[0],
        // ⚠️ O booleano chega como `1`/`0` do [`parse_assignments`] — ver lá porquê.
        "geometry_thin_walled" => m.geometry_thin_walled = v[0] != 0.0,
        _ => return false,
    }
    true
}

/// ⚠️ **O `true`/`false` da nodedef entra como `1`/`0`** — o `geometry_thin_walled` é o único input
/// booleano que esta crate lê, e sem esta linha ele era **descartado em silêncio** pelo `parse`:
/// o `Default` passaria a ser medido contra `20` campos em vez de `21`, e a metade que prova que a
/// omissão é a do padrão instalado deixaria de o cobrir.
fn parse_assignments(tokens: &[&str]) -> Vec<(String, Vec<f32>)> {
    tokens
        .iter()
        .filter_map(|kv| {
            let (k, val) = kv.split_once('=')?;
            let v = match val {
                "true" => vec![1.0],
                "false" => vec![0.0],
                _ => val
                    .split(',')
                    .map(str::parse)
                    .collect::<Result<Vec<f32>, _>>()
                    .ok()?,
            };
            Some((k.to_string(), v))
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

/// ⭐⭐⭐ **A curvatura da esfera do oráculo**, `1/R`, em unidades do mundo.
///
/// ⚠️ **Ela é MEDIDA da própria fixture e não assumida:** a geometria é a `sphere.obj` do MaterialX,
/// e resolvendo `v = unit(−p.x, −p.y, olho − p.z)` com `p = R·n` sobre as amostras dela sai
/// `R = 0,9975` (a posição interpolada de uma esfera facetada fica **dentro** dos vértices, que
/// estão em `1`). A `1/R` é `1,0025`, e a diferença para `1` é `0,25 %`.
///
/// # ⚠️⚠️ E é aqui que a única divergência estrutural desta wave se MEDE
///
/// O oráculo estima a curvatura por **derivada de ecrã** (`fwidth`), e esta porta recebe-a do
/// CAMPO — ver [`crate::subsurface::thick`] para porque ela não podia vir de derivadas de ecrã sem
/// partir a paridade CPU↔placa que esta linha tem em `100,000 %`. ⇒ **a barra do caminho maciço
/// mede as duas coisas ao mesmo tempo**: a lei portada, e quanto o `fwidth` dele se afasta da
/// verdade analítica sobre uma esfera facetada.
const SPHERE_CURVATURE: f32 = 1.0;

/// ⛔⛔⛔ **Os materiais cuja lei lê a CURVATURA — medidos por DISTRIBUIÇÃO e não pelo pior.**
///
/// O caminho maciço da subsuperfície é o único do OpenPBR que pergunta uma grandeza **geométrica**,
/// e o renderizador de referência responde-a por **derivada de ecrã** (`fwidth`). Medido sobre a
/// fixture, a curvatura implícita de cada amostra dele espalha-se de `0,02` a `54` numa esfera cuja
/// curvatura verdadeira é `1` — *é uma diferença finita por quad de `2×2` píxeis sobre normais
/// interpoladas, e ela salta em toda fronteira de triângulo e na silhueta*.
///
/// ⚠️ **E não é a tesselação:** a mesma medição sobre uma esfera `256×128` **nossa** devolve a mesma
/// dispersão. *A grandeza é do ECRÃ, não da malha.*
///
/// ⇒ estes dois têm gate próprio ([`o_caminho_macico_bate_o_oraculo_na_mediana`]), que afirma **mais**
/// do que o pior: a mediana, os quartis, e que o mínimo em `κ = 1` é **AGUDO**. ⛔ Sair desta lista é
/// o gate irmão a reprovar, nunca uma decisão.
const MACICOS: [usize; 2] = [9, 10];

fn direct_of(m: &OpenPbr, light: (Rgb, Rgb, f32), n: Rgb, v: Rgb) -> Rgb {
    let s = m.prepare().at_curvature(SPHERE_CURVATURE);
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
        ours, 21,
        "a nodedef tem de dar valor aos 21 campos da crate"
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
    assert_eq!(o.materials.len(), 11);
    assert_eq!(o.lights.len(), 4);
    assert_eq!(o.direct.len(), 1980, "a fixture perdeu ou ganhou amostras");
    let (mut worst, mut wrong) = ((0.0_f32, 0, 0), 0.0_f32);
    let mut por_material = vec![0.0_f32; o.materials.len()];
    let mut medidos = 0usize;
    for &(mi, li, n, v, expected) in &o.direct {
        let e = relative(direct_of(&o.materials[mi], o.lights[li], n, v), expected);
        por_material[mi] = por_material[mi].max(e);
        // ⚠️ Os dois do [`MACICOS`] entram na tabela impressa e ficam FORA da barra — ver lá porquê.
        if !MACICOS.contains(&mi) {
            medidos += 1;
            if e > worst.0 {
                worst = (e, mi, li);
            }
        }
        wrong = wrong.max(relative(
            direct_of(&OpenPbr::default(), o.lights[li], n, v),
            expected,
        ));
    }
    // A medição fica impressa (`--nocapture`): é ela que recalibra a barra, não a memória de alguém.
    // ⚠️ **A coluna POR MATERIAL não é enfeite** — um único pior esconde QUAL lei se afastou, e esta
    // fixture tem quatro leis diferentes lá dentro (base · verniz · parede fina · maciça).
    eprintln!(
        "luz directa: pior {:e} (material {}, luz {}) · material errado {wrong:e}",
        worst.0, worst.1, worst.2
    );
    for (mi, e) in por_material.iter().enumerate() {
        eprintln!("  material {mi}: {e:e}");
    }
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
    // ⚠️ **Piso de população:** sem isto, alargar o [`MACICOS`] esvaziaria a barra em silêncio.
    assert_eq!(
        medidos,
        o.direct.len() - MACICOS.len() * (o.direct.len() / o.materials.len()),
        "a conta das amostras medidas não fecha com o que o MACICOS declara"
    );
    assert!(
        medidos >= 1600,
        "só {medidos} amostras sob a barra — a fixture encolheu ou o MACICOS cresceu"
    );
}

/// ⭐⭐⭐ **O CAMINHO MACIÇO da subsuperfície bate o oráculo na MEDIANA — e o mínimo é AGUDO.**
///
/// # ⚠️ Porque esta lei não pode ser medida pelo PIOR, como as outras
///
/// Ver [`MACICOS`]: a curvatura que o renderizador de referência usa é uma derivada de **ECRÃ**, que
/// salta em toda fronteira de triângulo. ⇒ a maioria dos píxeis concorda muito bem e uma minoria
/// discorda muito, e *um máximo sobre essa população mede o estimador dele, não a nossa lei*.
///
/// # ⭐⭐⭐ E é a SEGUNDA metade que afirma a lei, não a primeira
///
/// Números pequenos não provam nada sozinhos — uma lei errada com um `subsurface_weight` pequeno
/// também os daria. O que prova é o **mínimo ser AGUDO na curvatura VERDADEIRA da esfera**: medido,
/// a mediana sobe `100×` ao pedir `κ = 0,5` ou `κ = 2`. *Uma lei que não fosse a dele não teria vale
/// nenhum em `1/R`.*
///
/// | | `κ = 0,5` | **`κ = 1` (a verdade)** | `κ = 2` |
/// |---|---:|---:|---:|
/// | material `9` p50 | `0,0769` | **`0,0005`** | `0,0582` |
/// | material `10` p50 | `0,0449` | **`0,0004`** | `0,0392` |
///
/// As barras saem desse vale, com folga de `2×` sobre o medido.
#[test]
fn o_caminho_macico_bate_o_oraculo_na_mediana() {
    let o = oracle();
    let erros = |mi: usize, kappa: f32| -> Vec<f32> {
        let s = o.materials[mi].prepare().at_curvature(kappa);
        let mut e: Vec<f32> = o
            .direct
            .iter()
            .filter(|s| s.0 == mi)
            .map(|&(_, li, n, v, esperado)| {
                let (dir, colour, intensity) = o.lights[li];
                let ours = bsdf::add3(
                    s.direct(n, v, dir.map(|d| -d), colour.map(|c| c * intensity)),
                    s.emission(n, v),
                );
                relative(ours, esperado)
            })
            .collect();
        e.sort_by(f32::total_cmp);
        e
    };
    for mi in MACICOS {
        let certo = erros(mi, SPHERE_CURVATURE);
        let n = certo.len();
        assert!(n >= 100, "material {mi}: só {n} amostras");
        let (p50, p75) = (certo[n / 2], certo[3 * n / 4]);
        eprintln!(
            "maciço {mi}: p50 {p50:.4} · p75 {p75:.4} · max {:.4}",
            certo[n - 1]
        );
        assert!(
            p50 <= 1.0e-3,
            "material {mi}: mediana {p50:e} acima de 1e-3"
        );
        assert!(p75 <= 2.0e-2, "material {mi}: p75 {p75:e} acima de 2e-2");
        // A metade que afirma a LEI: o vale existe e é fundo.
        for kappa in [0.5_f32, 2.0] {
            let torto = erros(mi, kappa)[n / 2];
            assert!(
                torto > 20.0 * p50,
                "material {mi}: com κ={kappa} a mediana é {torto:e} contra {p50:e} na verdade — \
                 o mínimo não é agudo, logo isto não mede a lei"
            );
        }
    }
}

/// ⭐⭐ **A luz do céu é a do MaterialX** (`PREFILTER` sobre um céu constante).
#[test]
fn the_sky_light_is_the_oracles_on_every_sphere() {
    let o = oracle();
    assert_eq!(o.indirect.len(), 495, "a fixture perdeu ou ganhou amostras");
    let sky = Uniform(o.sky);
    let mut worst = (0.0_f32, 0);
    for &(mi, n, v, expected) in &o.indirect {
        let s = o.materials[mi].prepare().at_curvature(SPHERE_CURVATURE);
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

/// ⏱️ **SONDA — que curvatura o oráculo está a USAR?**
///
/// O `mx_subsurface_scattering_approx` estima-a por `fwidth`, e esta porta recebe-a de fora. Se o
/// desvio do caminho maciço for **só** a curvatura, existe um valor que o fecha; se não existir
/// nenhum, o que está errado é a LEI. *Varrer o parâmetro livre fecha a porta antes de qualquer
/// cura* — é a mesma jogada que a `line/sculpt3d` pagou na altura da folga simétrica.
#[test]
#[ignore = "sonda: imprime uma tabela, não afirma"]
fn sonda_que_curvatura_o_oraculo_usa() {
    let o = oracle();
    for mi in [9usize, 10] {
        let mut linha = Vec::new();
        for k in 0..25 {
            let kappa = 0.05 * f32::powf(1.3, k as f32);
            let s = o.materials[mi].prepare().at_curvature(kappa);
            let mut worst = 0.0_f32;
            for &(m, li, n, v, expected) in &o.direct {
                if m != mi {
                    continue;
                }
                let (dir, colour, intensity) = o.lights[li];
                let ours = bsdf::add3(
                    s.direct(n, v, dir.map(|d| -d), colour.map(|c| c * intensity)),
                    s.emission(n, v),
                );
                worst = worst.max(relative(ours, expected));
            }
            linha.push((kappa, worst));
        }
        let best =
            linha.iter().copied().fold(
                (0.0_f32, f32::INFINITY),
                |a, b| if b.1 < a.1 { b } else { a },
            );
        eprintln!(
            "material {mi}: melhor curvatura {:.4} com pior {:e}",
            best.0, best.1
        );
        for (kappa, e) in &linha {
            eprintln!("   κ {kappa:8.4} → {e:e}");
        }
    }
}

/// ⏱️ **SONDA — a DISTRIBUIÇÃO do erro do caminho maciço, pela rota do produto.**
///
/// O pior de uma população não diz se a lei está certa: um único pixel onde o `fwidth` do oráculo
/// salta domina o máximo. ⇒ percentis.
#[test]
#[ignore = "sonda: imprime uma tabela, não afirma"]
fn sonda_a_distribuicao_do_caminho_macico() {
    let o = oracle();
    for mi in [9usize, 10] {
        for kappa in [0.5_f32, 1.0, 2.0] {
            let mut e: Vec<f32> = o
                .direct
                .iter()
                .filter(|s| s.0 == mi)
                .map(|&(_, li, n, v, esperado)| {
                    let s = o.materials[mi].prepare().at_curvature(kappa);
                    let (dir, colour, intensity) = o.lights[li];
                    let ours = bsdf::add3(
                        s.direct(n, v, dir.map(|d| -d), colour.map(|c| c * intensity)),
                        s.emission(n, v),
                    );
                    relative(ours, esperado)
                })
                .collect();
            e.sort_by(f32::total_cmp);
            let n = e.len();
            eprintln!(
                "material {mi} κ={kappa}: n={n} · p50 {:.4} · p75 {:.4} · p90 {:.4} · MAX {:.4}",
                e[n / 2],
                e[3 * n / 4],
                e[9 * n / 10],
                e[n - 1]
            );
        }
    }
}
