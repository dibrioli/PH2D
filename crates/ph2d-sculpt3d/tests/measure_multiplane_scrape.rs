//! **O QUE A LÂMINA EM V DEIXA** — a sonda que decide o ângulo de fábrica do
//! Multiplane Scrape e as barras dos gates da wave.
//!
//! ⚠️ **Ela dirige `SculptStroke::dab`, a porta do artista** — a §7.11 já pagou
//! duas vezes por medir peça isolada.
//!
//! ⚠️ **E o oráculo é a GEOMETRIA:** o ângulo entre as duas facetas é lido de
//! volta dos vértices que o traço moveu, por ajuste de plano em cada metade. Uma
//! sonda que imprimisse o knob estaria a citar o código sob teste.
//!
//! Rodar: `cargo test -p ph2d-sculpt3d --release --test measure_multiplane_scrape -- --ignored --nocapture --test-threads=1`

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};

/// ⚠️ **Mais fina que a esfera dos gates, e por uma razão MEDIDA:** o perfil é
/// lido em bandas de `0,12·R` ao longo do eixo que atravessa o traço, e na malha
/// de `96×144` isso é **UMA fileira de longitude** — a primeira leitura devolveu
/// `2 pontos por banda` e a sonda inteira saiu `NaN`. Uma sonda que amostra
/// menos que a estrutura que mede não mede nada.
fn sphere() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(160, 240, 1.0)
}

/// ⚠️ **O polo desta esfera é `+Y`**, então `[0, 0, 1]` é um ponto do EQUADOR —
/// o que serve igual (a normal ali é `+z`) e é o que a fixture do polegar já
/// usava. Ler o nome *polo* nos comentários e assumir `+z` foi o que fez a
/// primeira janela de amostragem cair na coluna errada.
const R: f32 = 0.35;
/// Quanto o traço anda POR DAB, em raios.
const STEP: f32 = 0.06;
const DABS: usize = 20;

/// Um traço que TERMINA no polo — o que se mede é o que ficou sob os últimos
/// dabs.
///
/// ⚠️ **Os centros dos dabs correm SOBRE a esfera, num arco de círculo máximo**,
/// e a primeira versão desta sonda os punha numa RETA em `z = 1`. Os dabs de
/// trás ficavam então **fora** da malha, cada um com o plano mais alto que o
/// anterior, e o corte que a sonda media era função de quão longe o dab tinha
/// flutuado — não do ângulo do V.
fn walk_with(verb: Verb, angle_deg: f32, dynamic: bool, invert: bool) -> (Mesh, Vec<[f32; 3]>) {
    let mut mesh = sphere();
    let rest = mesh.positions().to_vec();
    let b = Brush {
        verb,
        radius: R,
        strength: 1.0,
        scrape_angle_deg: angle_deg,
        scrape_dynamic: dynamic,
        invert,
        // ⚠️ **DERIVADO do verbo** — ver a nota gémea no gate: `Brush::default()`
        // carrega o `accumulate` do Draw, e com ele ligado a lei do plano
        // congelado fica inalcançável.
        accumulate: verb.default_accumulate(),
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 0..DABS {
        // O ângulo polar do dab `k`: o último cai exactamente no polo.
        let phi = (DABS - 1 - k) as f32 * STEP * R;
        let c = [phi.sin(), 0.0, phi.cos()];
        // O olho olha para o CENTRO da esfera, ou seja contra a normal local —
        // sem isto o conjunto frontal do `fit_plane` seria o do polo em todo dab.
        let eye = [-c[0], -c[1], -c[2]];
        let d = Dab::pulling(c, R, eye, [0.0; 3]);
        s.dab(&mut mesh, &b, &d, Symmetry::default());
    }
    (mesh, rest)
}

fn walk(angle_deg: f32, dynamic: bool, invert: bool) -> (Mesh, Vec<[f32; 3]>) {
    walk_with(Verb::MultiplaneScrape, angle_deg, dynamic, invert)
}

/// O perfil da secção transversal sob o último dab: a altura MÉDIA de `z` em
/// cada banda de `|u|`, com `u` = a coordenada que ATRAVESSA o traço.
///
/// ⚠️ **É o perfil, e não um ajuste de plano 3D**, porque a superfície cortada
/// **não é um plano em toda a pegada:** a projeção é ponderada pelo falloff, e
/// na borda o vértice caminha só uma fração do caminho até o plano. Um ajuste
/// sobre a pegada inteira mede a mistura, não a faceta — foi o que fez a
/// primeira versão desta sonda devolver `14,5° · 17,2° · 12,4°` para ângulos
/// autorados de `15° · 30° · 45°`.
fn profile(mesh: &Mesh, bands: &[(f32, f32)]) -> Vec<Option<f64>> {
    bands
        .iter()
        .map(|&(lo, hi)| {
            let mut acc = 0.0f64;
            let mut n = 0usize;
            for p in mesh.positions() {
                let u = p[1].abs();
                if p[2] > 0.0 && p[0].abs() < 0.20 * R && u >= lo * R && u < hi * R {
                    acc += f64::from(p[2]);
                    n += 1;
                }
            }
            (n >= 4).then(|| acc / n as f64)
        })
        .collect()
}

/// A abertura do V lida do PERFIL: `2·atan(inclinação)`, com a inclinação medida
/// entre duas bandas onde o falloff já saturou.
fn vee_from_profile(mesh: &Mesh, rest_mesh: &Mesh) -> Option<f64> {
    const NEAR: (f32, f32) = (0.10, 0.22);
    const FAR: (f32, f32) = (0.38, 0.50);
    let now = profile(mesh, &[NEAR, FAR]);
    let was = profile(rest_mesh, &[NEAR, FAR]);
    let (z0, z1) = (now[0]?, now[1]?);
    let (r0, r1) = (was[0]?, was[1]?);
    // ⚠️ **Contra o REPOUSO, e é o que tira a esfera da conta:** a malha intocada
    // já cai `u²/2` do polo para o lado, e sem subtrair isso o ângulo medido
    // conteria a curvatura da fixture.
    let du = f64::from((f32::midpoint(FAR.0, FAR.1) - f32::midpoint(NEAR.0, NEAR.1)) * R);
    let slope = ((z1 - z0) - (r1 - r0)) / du;
    Some(2.0 * (-slope).atan().to_degrees())
}

/// Quanto o meio da secção fica ACIMA dos flancos, em raios.
///
/// ⚠️ **Ele é lido do REPOUSO também, e o número que vale é a DIFERENÇA:** a
/// esfera intocada já tem o polo acima dos lados, então um valor absoluto aqui
/// mediria a fixture. Foi o que fez a primeira leitura reportar `0,0548` de
/// crista num traço que moveu **zero** vértices.
fn ridge_of(mesh: &Mesh) -> f64 {
    let bands = profile(mesh, &[(0.0, 0.15), (0.45, 0.65)]);
    match (bands[0], bands[1]) {
        (Some(mid), Some(side)) => (mid - side) / f64::from(R),
        _ => f64::NAN,
    }
}

fn removed(mesh: &Mesh, rest: &[[f32; 3]]) -> (usize, f64) {
    let mut n = 0usize;
    let mut deepest = 0.0f64;
    for (p, r) in mesh.positions().iter().zip(rest) {
        let d = [p[0] - r[0], p[1] - r[1], p[2] - r[2]];
        let m = f64::from((d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt());
        if m > 1.0e-5 {
            n += 1;
        }
        deepest = deepest.max(m);
    }
    (n, deepest)
}

#[test]
#[ignore = "sonda"]
fn what_the_vee_leaves_at_each_angle() {
    let rest_mesh = sphere();
    let base = ridge_of(&rest_mesh);
    println!("\n== o sulco, medido na seccao transversal sob o ultimo dab ==");
    println!(
        "{:>7} | {:>10} | {:>12} | {:>9} | {:>10}",
        "autorado", "medido", "crista (r)", "movidos", "mais fundo"
    );
    for a in [0.0f32, 15.0, 30.0, 45.0, 60.0, 90.0, 120.0, 160.0] {
        let (mesh, rest) = walk(a, false, false);
        let (n, deep) = removed(&mesh, &rest);
        let v = vee_from_profile(&mesh, &rest_mesh);
        let r = ridge_of(&mesh) - base;
        match v {
            Some(v) => println!("{a:>6.0}° | {v:>9.2}° | {r:>12.4} | {n:>9} | {deep:>10.4}"),
            None => println!(
                "{a:>6.0}° | {:>10} | {r:>12.4} | {n:>9} | {deep:>10.4}",
                "-"
            ),
        }
    }
    println!(
        "\n(o MEDIDO sai da inclinacao do perfil, `2·atan(-dz/du)`, contra a\n \
         esfera em repouso; a CRISTA e' quanto o meio do sulco ficou ACIMA dos\n \
         flancos em relacao ao repouso, em raios — ZERO num Scrape)"
    );
}

#[test]
#[ignore = "sonda"]
fn the_vee_against_the_plain_scrape() {
    let rest_mesh = sphere();
    let base = ridge_of(&rest_mesh);
    println!("\n== lamina em V contra o Scrape, mesmo traco ==");
    println!(
        "{:>18} | {:>10} | {:>12} | {:>9}",
        "verbo", "medido", "crista (r)", "movidos"
    );
    let show = |name: &str, mesh: &Mesh, rest: &[[f32; 3]]| {
        let (n, _) = removed(mesh, rest);
        println!(
            "{name:>18} | {:>10} | {:>12.4} | {n:>9}",
            vee_from_profile(mesh, &rest_mesh).map_or("-".to_string(), |d| format!("{d:.2}°")),
            ridge_of(mesh) - base
        );
    };
    let (m, r) = walk_with(Verb::Scrape, 0.0, false, false);
    show("Scrape", &m, &r);
    let (m, r) = walk(ph2d_sculpt3d::DEFAULT_MULTIPLANE_ANGLE_DEG, false, false);
    show("Multiplane (fab.)", &m, &r);
}

#[test]
#[ignore = "sonda"]
fn what_the_ctrl_and_the_dynamic_mode_do() {
    println!("\n== o Ctrl e o modo dinamico ==");
    println!(
        "{:>26} | {:>12} | {:>9} | {:>12}",
        "cena", "crista (r)", "movidos", "vol assinado"
    );
    let base = ridge_of(&sphere());
    let show = |name: &str, mesh: &Mesh, rest: &[[f32; 3]]| {
        let (n, _) = removed(mesh, rest);
        let mut vol = 0.0f64;
        for (p, r) in mesh.positions().iter().zip(rest) {
            let d = [p[0] - r[0], p[1] - r[1], p[2] - r[2]];
            vol += f64::from(d[0] * r[0] + d[1] * r[1] + d[2] * r[2]);
        }
        println!(
            "{name:>26} | {:>12.4} | {n:>9} | {vol:>12.4}",
            ridge_of(mesh) - base
        );
    };
    let a = ph2d_sculpt3d::DEFAULT_MULTIPLANE_ANGLE_DEG;
    let (m, r) = walk(a, false, false);
    show("fixo", &m, &r);
    let (m, r) = walk(a, false, true);
    show("fixo + Ctrl", &m, &r);
    let (m, r) = walk(a, true, false);
    show("dinamico", &m, &r);
    let (m, r) = walk(a, true, true);
    show("dinamico + Ctrl", &m, &r);
    println!(
        "\n(o Ctrl no modo FIXO inverte o V — a crista vira vale; no DINAMICO ele\n \
         zera o angulo de proposito, que e' o 'trim plane surfaces without\n \
         changing the brush' da referencia)"
    );
}

/// **O QUE A DOBRADIÇA CONGELADA COMPRA** — a `calc_brush_plane` do Blender lê o
/// `orig` do pen-down com o Accumulate desarmado, e este verbo herda a regra.
///
/// ⚠️ **A sonda existe porque a mutação NÃO SANGROU:** num traço que ANDA a
/// diferença é indetectável (o dab vê barro fresco a cada passo), então o número
/// tem de vir de um traço que INSISTE no mesmo lugar — que é onde a normal de
/// área de facto persegue o sulco que ela própria cavou.
#[test]
#[ignore = "sonda"]
fn what_the_frozen_hinge_buys() {
    println!("\n== o traco que INSISTE no mesmo lugar ==");
    println!(
        "{:>8} | {:>10} | {:>12} | {:>9}",
        "dabs", "passo/R", "mais fundo", "movidos"
    );
    for (dabs, step) in [(20usize, 0.06f32), (60, 0.005), (120, 0.002)] {
        let mut mesh = sphere();
        let rest = mesh.positions().to_vec();
        let b = Brush {
            verb: Verb::MultiplaneScrape,
            radius: R,
            strength: 1.0,
            accumulate: Verb::MultiplaneScrape.default_accumulate(),
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        for k in 0..dabs {
            let phi = (dabs - 1 - k) as f32 * step * R;
            let c = [phi.sin(), 0.0, phi.cos()];
            let d = Dab::pulling(c, R, [-c[0], -c[1], -c[2]], [0.0; 3]);
            s.dab(&mut mesh, &b, &d, Symmetry::default());
        }
        let (n, deep) = removed(&mesh, &rest);
        println!("{dabs:>8} | {step:>10.3} | {deep:>12.5} | {n:>9}");
    }
    println!(
        "\n(compare com a MESMA tabela sob a mutacao que tira a\n \
         Verb::MultiplaneScrape da lista de planos CONGELADOS)"
    );
}

/// **O QUE O MODO DINÂMICO LÊ SOZINHO** — com o knob em ZERO o ângulo vem
/// inteiramente da superfície, e é essa coluna que separa *"ele amostra"* de
/// *"ele soma um número"*.
#[test]
#[ignore = "sonda"]
fn what_the_dynamic_mode_reads_with_the_knob_at_zero() {
    println!("\n== knob ZERO: quem corta e' a superficie ==");
    println!("{:>12} | {:>12} | {:>9}", "modo", "crista (r)", "movidos");
    let rest_mesh = sphere();
    let base = ridge_of(&rest_mesh);
    for (name, dynamic) in [("fixo", false), ("dinamico", true)] {
        let (m, r) = walk(0.0, dynamic, false);
        let (n, _) = removed(&m, &r);
        println!("{name:>12} | {:>12.4} | {n:>9}", ridge_of(&m) - base);
    }
}
