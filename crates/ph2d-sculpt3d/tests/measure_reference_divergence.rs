//! **O ATLAS DA DIVERGÊNCIA** — quanto o nosso motor difere do porte da
//! referência, verbo a verbo, num único dab.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test measure_reference_divergence \
//!   -- --ignored --nocapture
//! ```
//!
//! # Por que uma sonda e não um gate
//!
//! Ela não afirma nada: ela **mede**. Os números que ela imprime é que dizem
//! quais verbos estão longe e quanto, e é isso que decide a ordem do trabalho —
//! a alternativa é eu escolher a ordem pela minha leitura do código, que é
//! exatamente como se conserta o verbo errado primeiro.
//!
//! # ⚠️ Ela alimenta os DOIS lados com a MESMA malha, de propósito
//!
//! A referência recebe as posições, as normais e a máscara da **nossa** malha,
//! e a **mesma** pegada. Não é conveniência: é o que isola *a lei do kernel* —
//! a pergunta — de *como as normais são computadas* e *quem está sob o pincel*,
//! que são outras duas perguntas com gates próprios. Uma sonda que deixasse as
//! três variarem juntas reportaria um número que não aponta para nada.
//!
//! # ⚠️ E ela converte a POLARIDADE da máscara
//!
//! O nosso `DEFAULT_MASK = 0` é *totalmente esculpível*; o `mAr[ind + 2]` da
//! referência é o oposto. Quem esquece isto mede o pincel de um lado contra a
//! máscara do outro e conclui que a lei diverge.

use ph2d_mesh::Mesh;
use ph2d_sculpt3d::{Brush, Dab, Falloff, SculptStroke, Symmetry, Verb, ref_kernels as rk};

/// A malha das duas medições — a nossa, com as nossas normais.
fn sphere() -> Mesh {
    ph2d_mesh::shapes::uv_sphere(64, 128, 1.0)
}

/// Onde o pincel pousa: um ponto SOBRE a superfície, no equador, longe dos
/// polos (onde a topologia da esfera UV degenera e o número falaria sobre a
/// malha em vez de sobre o verbo).
fn dab_at(radius: f32) -> Dab {
    let c = [0.7f32.cos(), 0.0, 0.7f32.sin()];
    let len = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt();
    let eye = [-c[0] / len, -c[1] / len, -c[2] / len];
    Dab::at(c, radius, eye)
}

/// A pegada: os vértices dentro da esfera do dab.
fn footprint(mesh: &Mesh, d: &Dab) -> Vec<u32> {
    let r2 = d.radius * d.radius;
    mesh.positions()
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            let v = [p[0] - d.center[0], p[1] - d.center[1], p[2] - d.center[2]];
            v[0] * v[0] + v[1] * v[1] + v[2] * v[2] < r2
        })
        .map(|(i, _)| i as u32)
        .collect()
}

/// A máscara **na polaridade da referência** (`1` = livre) — a nossa é o
/// contrário, e este é o único lugar em que a conversão acontece.
fn free_of(mesh: &Mesh) -> Vec<f32> {
    match mesh.masks() {
        Some(m) => m
            .iter()
            .map(|&x| ph2d_sculpt3d::mask_ops::free_weight(x))
            .collect(),
        None => vec![1.0; mesh.vert_count()],
    }
}

/// As posições achatadas em `xyz`, que é a forma que o porte fala.
fn flat(mesh: &Mesh) -> Vec<f32> {
    mesh.positions().iter().flat_map(|p| *p).collect()
}

/// O maior deslocamento por componente de um campo contra a linha de base.
fn reach_of(a: &[f32], base: &[f32]) -> f64 {
    a.iter()
        .zip(base)
        .map(|(x, b)| (f64::from(*x) - f64::from(*b)).abs())
        .fold(0.0, f64::max)
}

struct Row {
    verb: &'static str,
    ours: f64,
    theirs: f64,
    /// O maior |nosso − deles| por componente.
    gap: f64,
}

#[test]
#[ignore = "sonda: imprime a tabela, não afirma nada"]
fn what_separates_our_kernels_from_the_reference() {
    const R: f32 = 0.45;
    let mut rows: Vec<Row> = Vec::new();

    // Cada linha é `(verbo, a intensidade DE FÁBRICA da tool correspondente no
    // original, o kernel da referência)`. A intensidade não é escolhida por
    // mim: é o `_intensity` do construtor de cada tool.
    let cases: &[(&'static str, Verb, f64, bool)] = &[
        ("Draw/brush", Verb::Draw, 0.5, false),
        ("Clay", Verb::Clay, 0.5, false),
        ("Flatten", Verb::Flatten, 0.75, true),
        // ⚠️ **O `Flatten` da referência É o nosso `Fill` ou o nosso `Scrape`**
        // (`comp = ±1` + `continue`, §3.3): as duas linhas abaixo medem os dois
        // lados dele contra o mesmo kernel, e são o que torna a nossa
        // bilateralidade uma escolha MEDIDA em vez de uma afirmação.
        ("Fill", Verb::Fill, 0.75, false),
        ("Scrape", Verb::Scrape, 0.75, true),
        ("Inflate", Verb::Inflate, 0.3, false),
        ("Crease", Verb::Crease, 0.75, true),
        ("Pinch", Verb::Pinch, 0.75, false),
        // O `Magnify` é o `Pinch` com o sinal trocado — o mesmo kernel.
        ("Magnify", Verb::Magnify, 0.75, true),
        // ⚠️ **O `Smooth` da referência NÃO tem falloff** (§3.3), então a nossa
        // curva é isolada com a `Constant` — que é exatamente o valor em que a
        // nossa família reproduz a dela. Medi-lo com a `Plateau` somaria a
        // divergência declarada da CURVA à da lei.
        ("Smooth", Verb::Smooth, 0.75, false),
    ];

    for &(name, verb, intensity, negative) in cases {
        let mut mesh = sphere();
        let base = flat(&mesh);
        let d = dab_at(R);
        let fp = footprint(&mesh, &d);
        let free = free_of(&mesh);
        let normals: Vec<f32> = mesh.normals().iter().flat_map(|n| *n).collect();

        // --- O NOSSO, pela porta do produto.
        let brush = Brush {
            verb,
            strength: intensity as f32,
            // ⚠️ **A curva da REFERÊNCIA, e não o default do pincel.** Esta
            // tabela existe para isolar *a lei e as constantes*; medi-la com a
            // `Smooth` misturaria a diferença de FORMA (que a `Plateau` fechou
            // em 2026-08-11, `1,000×` em toda a linha) com a de MAGNITUDE, e um
            // número que soma duas causas não aponta para nenhuma.
            falloff: if verb == Verb::Smooth {
                Falloff::Constant
            } else {
                Falloff::Plateau
            },
            // ⚠️ **O Crease parqueia o default no OUTRO lado do flag.** A tool
            // da referência nasce com `_negative = true` (`Crease.js:11`) e cava
            // por ali; o nosso kernel cava com o sinal e `invert = false`. O
            // PRODUTO se comporta igual — os dois cavam sem Ctrl e criam crista
            // com ele —, e comparar `invert = negative` põe um a cavar e o outro
            // a levantar: o `|diferença|` sai `0,0358`, quase **2×** o próprio
            // deslocamento (`0,019`), que é a assinatura de um sinal e não de
            // uma lei.
            invert: if verb == Verb::Crease {
                !negative
            } else {
                negative
            },
            // ⚠️ **O `pinch` é NOSSO e a referência não o tem** — lá o termo
            // lateral do Crease entra inteiro (`dx * fallOff`). O default de
            // fábrica é `0,5`, então medi-lo aqui somaria a divergência de um
            // KNOB à da lei, que é o que esta tabela existe para separar. O
            // custo dele está na linha `pinch` da varredura abaixo.
            pinch: 1.0,
            ..Brush::default()
        };
        let mut stroke = SculptStroke::default();
        stroke.begin(&mesh);
        stroke.dab(&mut mesh, &brush, &d, Symmetry::default());
        let ours = flat(&mesh);

        // --- O DELES, sobre a MESMA malha e a MESMA pegada.
        let mut theirs = base.clone();
        let center = [
            f64::from(d.center[0]),
            f64::from(d.center[1]),
            f64::from(d.center[2]),
        ];
        let eye = [
            f64::from(d.eye[0]),
            f64::from(d.eye[1]),
            f64::from(d.eye[2]),
        ];
        let r2 = f64::from(d.radius) * f64::from(d.radius);
        let mut front = Vec::new();
        rk::front_vertices(&normals, &fp, eye, &mut front);
        let a_normal = rk::area_normal(&normals, &free, &front).expect("normal de área");
        match verb {
            Verb::Draw => rk::brush(
                &mut theirs,
                &free,
                &fp,
                None,
                a_normal,
                center,
                r2,
                intensity,
                negative,
            ),
            Verb::Clay => {
                let mut ctr = rk::area_center(&base, &free, &front).expect("centro");
                let off = rk::clay_plane_offset(r2.sqrt());
                for k in 0..3 {
                    ctr[k] += a_normal[k] * off;
                }
                rk::flatten(
                    &mut theirs,
                    &free,
                    &fp,
                    None,
                    a_normal,
                    ctr,
                    center,
                    r2,
                    intensity,
                    negative,
                );
            }
            Verb::Flatten => {
                let ctr = rk::area_center(&base, &free, &front).expect("centro");
                rk::flatten(
                    &mut theirs,
                    &free,
                    &fp,
                    None,
                    a_normal,
                    ctr,
                    center,
                    r2,
                    intensity,
                    negative,
                );
            }
            Verb::Inflate => rk::inflate(
                &mut theirs,
                &normals,
                &free,
                &fp,
                Some(&base),
                center,
                r2,
                intensity,
                negative,
            ),
            Verb::Crease => rk::crease(
                &mut theirs,
                &free,
                &fp,
                Some(&base),
                a_normal,
                center,
                r2,
                intensity,
                negative,
            ),
            Verb::Pinch | Verb::Magnify => {
                rk::pinch(&mut theirs, &free, &fp, center, r2, intensity, negative);
            }
            // Os dois lados do `Flatten` da referência, contra o mesmo kernel.
            Verb::Fill | Verb::Scrape => {
                let ctr = rk::area_center(&base, &free, &front).expect("centro");
                rk::flatten(
                    &mut theirs,
                    &free,
                    &fp,
                    None,
                    a_normal,
                    ctr,
                    center,
                    r2,
                    intensity,
                    negative,
                );
            }
            Verb::Smooth => {
                let adj = mesh.adjacency();
                let (starts, lens, values) = adj.vert_verts.parts();
                let on_edge: Vec<u8> = (0..mesh.vert_count())
                    .map(|v| u8::from(adj.is_border(v)))
                    .collect();
                let mut smoothed = Vec::new();
                rk::laplacian(&base, &fp, starts, lens, values, &on_edge, &mut smoothed);
                rk::smooth(&mut theirs, &free, &fp, &smoothed, intensity);
            }
            _ => unreachable!("a tabela só tem verbos de carimbo"),
        }

        rows.push(Row {
            verb: name,
            ours: reach_of(&ours, &base),
            theirs: reach_of(&theirs, &base),
            gap: reach_of(&ours, &theirs),
        });
    }

    println!(
        "\n== UM DAB, a MESMA malha e a MESMA pegada ({} vértices) ==",
        {
            let m = sphere();
            footprint(&m, &dab_at(R)).len()
        }
    );
    println!(
        "{:<12} {:>12} {:>12} {:>10} {:>12}",
        "verbo", "nosso", "referência", "razão", "|diferença|"
    );
    for r in &rows {
        let ratio = if r.theirs > 0.0 {
            format!("{:.2}x", r.ours / r.theirs)
        } else {
            "-".into()
        };
        // ⚠️ **A diferença sai em NOTAÇÃO CIENTÍFICA de propósito.** Com seis
        // casas ela imprime `0,000000` em três verbos, e `0,000000` responde
        // *"abaixo do que eu mostro"* — não *"zero"*. A pergunta aberta que
        // sobra na §3.2.5 é a cadeia de peso em `f64` (o nosso `w` é `f32`
        // desde o falloff, a referência arredonda uma vez), e ela só é
        // respondida por um número que chega ao epsilon do `f32`.
        println!(
            "{:<12} {:>12.6} {:>12.6} {:>10} {:>12.3e}",
            r.verb, r.ours, r.theirs, ratio, r.gap
        );
    }

    // A curva, isolada — o fator que multiplica TODOS os verbos.
    //
    // ⚠️ **Duas colunas, e é a segunda que diz onde a paridade está:** a
    // `Plateau` É a quártica da referência (entrou na família em 2026-08-11), e
    // a razão dela tem de ser `1,000` em toda a linha. A `Smooth` fica ao lado
    // porque é o DEFAULT do pincel, e a distância dela é o que o artista vê ao
    // trocar de curva.
    println!("\n== A CURVA, ponto a ponto ==");
    println!(
        "{:>6} {:>12} {:>12} {:>8} {:>12} {:>8}",
        "t", "referência", "Smooth", "razão", "Plateau", "razão"
    );
    for i in 0..=8 {
        let t = f64::from(i) / 8.0;
        let r = rk::falloff(t);
        let ratio = |x: f64| {
            if x > 0.0 {
                format!("{:.3}x", r / x)
            } else {
                "-".into()
            }
        };
        let s = f64::from(Falloff::Smooth.weight(t as f32));
        let p = f64::from(Falloff::Plateau.weight(t as f32));
        println!(
            "{t:>6.3} {r:>12.6} {s:>12.6} {:>8} {p:>12.6} {:>8}",
            ratio(s),
            ratio(p)
        );
    }

    // O ACÚMULO — o checkbox que o Enio reportou como quebrado. Um traço reto
    // de N dabs sobre o mesmo lugar, com e sem ele.
    println!("\n== O ACUMULA, medido: N dabs no MESMO lugar ==");
    println!(
        "{:>5} {:>16} {:>16} {:>16}",
        "dabs", "nosso OFF", "nosso ON", "referência"
    );
    for n in [1usize, 2, 4, 8, 16] {
        let d = dab_at(R);
        let mk = |accumulate: bool| {
            let mut mesh = sphere();
            let base = flat(&mesh);
            let brush = Brush {
                verb: Verb::Draw,
                strength: 0.5,
                accumulate,
                ..Brush::default()
            };
            let mut s = SculptStroke::default();
            s.begin(&mesh);
            for _ in 0..n {
                s.dab(&mut mesh, &brush, &d, Symmetry::default());
            }
            reach_of(&flat(&mesh), &base)
        };
        // A referência: N aplicações do kernel sobre o estado VIVO, que é o que
        // ela faz — a normal de área é recomputada a cada dab, como no
        // `Brush.stroke`.
        let mut mesh = sphere();
        let base = flat(&mesh);
        let free = free_of(&mesh);
        let mut theirs = base.clone();
        let center = [
            f64::from(d.center[0]),
            f64::from(d.center[1]),
            f64::from(d.center[2]),
        ];
        let eye = [
            f64::from(d.eye[0]),
            f64::from(d.eye[1]),
            f64::from(d.eye[2]),
        ];
        let r2 = f64::from(d.radius) * f64::from(d.radius);
        let fp = footprint(&mesh, &d);
        for _ in 0..n {
            let normals: Vec<f32> = mesh.normals().iter().flat_map(|n| *n).collect();
            let mut front = Vec::new();
            rk::front_vertices(&normals, &fp, eye, &mut front);
            let an = rk::area_normal(&normals, &free, &front).expect("normal");
            rk::brush(&mut theirs, &free, &fp, None, an, center, r2, 0.5, false);
            for (i, p) in mesh.positions_mut().iter_mut().enumerate() {
                *p = [theirs[i * 3], theirs[i * 3 + 1], theirs[i * 3 + 2]];
            }
            mesh.rebuild();
        }
        println!(
            "{n:>5} {:>16.6} {:>16.6} {:>16.6}",
            mk(false),
            mk(true),
            reach_of(&theirs, &base)
        );
    }
    println!();
}

/// **DE QUEM É A DIVERGÊNCIA QUE SOBRA: da LEI ou do PLANO?**
///
/// O Clay e o Flatten são os dois verbos que ainda não batem depois de a família
/// do carimbo trocar de lei (doc 19 §3.2.5), e os dois são os que consomem o
/// **PONTO** do plano — o Draw usa só a normal e mede `1,01×`. Esta sonda separa
/// as duas perguntas: quanto o nosso `fit_plane` difere do
/// `area_center`/`area_normal` da referência, e quanto dessa diferença chega ao
/// deslocamento.
///
/// ⚠️ **Ela NÃO afirma nada.** Se o plano explicar o resto, o trabalho é um; se
/// não explicar, é outro — e escolher sem medir é como se conserta o verbo
/// errado primeiro.
#[test]
#[ignore = "sonda"]
fn whose_divergence_is_left_the_law_or_the_plane() {
    let radius = 0.5f32;
    let d = dab_at(radius);
    let mesh = sphere();
    let verts = footprint(&mesh, &d);
    let base: Vec<f32> = mesh.positions().iter().flatten().copied().collect();
    let normals: Vec<f32> = mesh.normals().iter().flatten().copied().collect();
    // A máscara da referência: `1` é livre, e a nossa é o oposto.
    let free = vec![1.0f32; mesh.vert_count()];
    let eye = [
        f64::from(d.eye[0]),
        f64::from(d.eye[1]),
        f64::from(d.eye[2]),
    ];
    let mut front = Vec::new();
    rk::front_vertices(&normals, &verts, eye, &mut front);

    let their_n = rk::area_normal(&normals, &free, &front).expect("normal de área");
    let their_c = rk::area_center(&base, &free, &front).expect("centro de área");

    // O NOSSO plano, pela porta do produto: um traço que começa e pergunta.
    let mut ours = sphere();
    let brush = Brush {
        verb: Verb::Flatten,
        radius,
        strength: 1.0,
        falloff: Falloff::Plateau,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&ours);
    let (our_c, our_n) = s.probe_plane(&mut ours, &brush, &d);

    let dot = f64::from(our_n[0]) * their_n[0]
        + f64::from(our_n[1]) * their_n[1]
        + f64::from(our_n[2]) * their_n[2];
    let dc = [
        f64::from(our_c[0]) - their_c[0],
        f64::from(our_c[1]) - their_c[1],
        f64::from(our_c[2]) - their_c[2],
    ];
    let dist = (dc[0] * dc[0] + dc[1] * dc[1] + dc[2] * dc[2]).sqrt();
    // A componente que IMPORTA: o deslocamento do centro AO LONGO da normal é o
    // que entra em `signed_distance`, e é ele que move o resultado. O resto
    // desliza dentro do próprio plano e não muda nada.
    let along = dc[0] * their_n[0] + dc[1] * their_n[1] + dc[2] * their_n[2];

    println!("== O PLANO: nosso `fit_plane` contra o `area_*` da referência ==");
    println!(
        "  pegada {} vértices, frontais {}",
        verts.len(),
        front.len()
    );
    println!("  normal: cos {dot:.6}  (1 = mesma direção)");
    println!(
        "  centro: distância {dist:.6} · AO LONGO da normal {along:.6} ({:.1}% do raio)",
        100.0 * along.abs() / f64::from(radius)
    );
    println!("  ^ o `signed_distance` do Flatten/Clay só enxerga a componente ao longo");
}
