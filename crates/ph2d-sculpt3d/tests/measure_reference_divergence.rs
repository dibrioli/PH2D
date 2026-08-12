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

/// **O GRAB — as quatro tools que PUXAM, medidas contra o porte.**
///
/// A família do carimbo fechou em 2026-08-11 (§3.2.5-§3.2.7) e esta é a outra
/// metade do catálogo: `Move` · `SnakeHook` · `Twist` · `LocalScale`, os quatro
/// [`ph2d_sculpt3d::Grip`] que não carimbam. Os kernels da referência para os
/// quatro **já existem e já são bit-idênticos contra o JS executando**
/// (`tests/sculptgl_parity.rs`) — o que esta sonda mede é se o **PRODUTO** os
/// reproduz, que é exatamente a pergunta que o Enio fez sobre o carimbo
/// (*"nada está idêntico! as mudanças já foram linkadas?"*) e que ali a resposta
/// era **não**.
///
/// ⚠️ **Todos com `strength = 1.0`, e não é conveniência.** Nas quatro tools do
/// original a intensidade **não** é um fator do falloff:
///
/// - `Move.js:10` tem `_intensity = 1.0` e o dobra DENTRO do `dir`;
/// - `Drag.js` **não tem intensidade nenhuma** — o `dir` é o delta do rato;
/// - `Twist.js` multiplica pelo **ÂNGULO**, que é o gesto;
/// - `LocalScale.js:69` multiplica pelo `delta * 0.01`, que também é o gesto.
///
/// Os nossos quatro dobram o gesto no `Dab` e a intensidade no `w`, então
/// `strength = 1.0` é o que põe as duas leis na mesma unidade. Medir com `0.5`
/// somaria um FATOR conhecido à divergência que a tabela existe para achar.
#[test]
#[ignore = "sonda: imprime a tabela, não afirma nada"]
fn what_separates_our_grab_family_from_the_reference() {
    const R: f32 = 0.45;
    // O gesto: uma puxada TANGENTE à esfera no ponto do dab. O `dab_at` pousa em
    // `(cos 0.7, 0, sin 0.7)`, então `+Y` é exatamente tangente ali — um gesto
    // radial misturaria *puxar* com *inflar* e o número diria as duas coisas.
    const PULL: [f32; 3] = [0.0, 0.2, 0.0];
    // O giro e a escala, na unidade que cada verbo declara.
    const RADIANS: f32 = 0.35;
    const FRACTION: f32 = 0.25;

    let mut rows: Vec<Row> = Vec::new();

    for name in ["Move/grab", "SnakeHook", "Twist", "LocalScale"] {
        let mut mesh = sphere();
        let base = flat(&mesh);
        let d0 = dab_at(R);
        let fp = footprint(&mesh, &d0);
        let free = free_of(&mesh);

        // O gesto, construído pela porta que NOMEIA a leitura — um `Dab { pull,
        // .. }` à mão carregaria um total onde se espera um incremento sem o
        // compilador piscar (o doc do `Dab::pull` é explícito sobre isso).
        let (verb, d) = match name {
            "Move/grab" => (
                Verb::Move,
                Dab::pulling(d0.center, d0.radius, d0.eye, PULL),
            ),
            "SnakeHook" => (
                Verb::SnakeHook,
                Dab::hooking(d0.center, d0.radius, d0.eye, PULL),
            ),
            "Twist" => (
                Verb::Twist,
                Dab::turning(d0.center, d0.radius, d0.eye, RADIANS),
            ),
            _ => (
                Verb::LocalScale,
                Dab::scaling(d0.center, d0.radius, d0.eye, FRACTION),
            ),
        };

        // --- O NOSSO, pela porta do produto.
        let brush = Brush {
            verb,
            strength: 1.0,
            falloff: Falloff::Plateau,
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
        let r2 = f64::from(d.radius) * f64::from(d.radius);
        let pull = [
            f64::from(PULL[0]),
            f64::from(PULL[1]),
            f64::from(PULL[2]),
        ];
        match verb {
            // ⚠️ **O proxy é indexado pela POSIÇÃO NA LISTA**, não pelo id do
            // vértice — o doc do `rk::move` avisa, e trocar os dois lê a
            // vizinhança errada em silêncio.
            Verb::Move => {
                let proxy: Vec<f32> = fp
                    .iter()
                    .flat_map(|&v| {
                        let i = v as usize * 3;
                        [base[i], base[i + 1], base[i + 2]]
                    })
                    .collect();
                rk::r#move(&mut theirs, &free, &fp, &proxy, pull, center, r2);
            }
            Verb::SnakeHook => rk::drag(&mut theirs, &free, &fp, pull, center, r2),
            // ⚠️ **O eixo é MENOS o olho** (`Twist.js:41`), e o nosso
            // `compute_target` o nega no mesmo lugar — passar `d.eye` aqui
            // mediria os dois a girar para lados opostos.
            Verb::Twist => {
                let axis = [
                    -f64::from(d.eye[0]),
                    -f64::from(d.eye[1]),
                    -f64::from(d.eye[2]),
                ];
                ph2d_sculpt3d::ref_kernels::twist(
                    &mut theirs,
                    &free,
                    &fp,
                    center,
                    r2,
                    f64::from(RADIANS),
                    axis,
                );
            }
            // ⚠️ **A `intensity` do `scale` é o DELTA EM PIXELS**, e o `0.01`
            // de dentro do kernel é o que a converte em fração: a nossa fração
                // entra multiplicada por 100 para as duas falarem a mesma coisa.
            _ => rk::scale(
                &mut theirs,
                &free,
                &fp,
                center,
                r2,
                f64::from(FRACTION) * 100.0,
            ),
        }

        rows.push(Row {
            verb: name,
            ours: reach_of(&ours, &base),
            theirs: reach_of(&theirs, &base),
            gap: reach_of(&ours, &theirs),
        });
    }

    println!("\n== O GRAB: um gesto, a MESMA malha e a MESMA pegada ==");
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
        println!(
            "{:<12} {:>12.6} {:>12.6} {:>10} {:>12.3e}",
            r.verb, r.ours, r.theirs, ratio, r.gap
        );
    }
    println!();
}

/// **A MÁSCARA — o último membro do catálogo com divergência NOMEADA.**
///
/// O `Grip::Paint` declara, no próprio doc, que carrega *"a lei do ENVELOPE,
/// verbatim"* e que portar o `clamp(m + f)` da referência é wave própria. Esta
/// sonda mede as DUAS metades dessa frase antes de qualquer linha ser escrita:
///
/// 1. **A CURVA.** A referência dá ao canal uma curva PRÓPRIA —
///    `(1 − d)^{2(1 − hardness)}`, `hardness = 0.25` de fábrica ⇒ expoente
///    `1.5` — que não é a quártica da geometria. Nenhum membro do nosso
///    [`Falloff`] é essa curva.
/// 2. **A LEI.** A nossa é `toward(base, goal, envelope)`: assintótica, e
///    idempotente sob re-carimbo. A dela é `clamp(m + f, 0, 1)`: ADITIVA, e é
///    ela que faz **esfregar construir máscara** — exatamente o gesto que o
///    smoke da máscara do Painter 2D já ensinou a este repo a testar.
///
/// ⚠️ **A polaridade é convertida numa ponta só.** O nosso `DEFAULT_MASK = 0` é
/// *totalmente esculpível*; o `mAr[ind+2]` da referência é o oposto. A tabela
/// compara `free_weight(nosso)` contra o `free` dela — comparar os dois crus
/// mediria uma máscara contra o complemento da outra.
#[test]
#[ignore = "sonda: imprime a tabela, não afirma nada"]
fn what_separates_our_mask_from_the_reference() {
    const R: f32 = 0.45;
    // Os defaults DE FÁBRICA da tool `Masking` do original (`Masking.js:13-16`).
    const INTENSITY: f64 = 0.5;
    const HARDNESS: f64 = 0.25;

    println!("\n== A MÁSCARA: N esfregadas no MESMO lugar (o `free`, 1 = livre) ==");
    println!(
        "{:>5} {:>16} {:>16} {:>14}",
        "dabs", "nosso", "referência", "|diferença|"
    );

    for n in [1usize, 2, 4, 8, 16] {
        let d = dab_at(R);

        // --- O NOSSO, pela porta do produto.
        let mut mesh = sphere();
        let fp = footprint(&mesh, &d);
        let brush = Brush {
            verb: Verb::Mask,
            strength: INTENSITY as f32,
            falloff: Falloff::Plateau,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        for _ in 0..n {
            s.dab(&mut mesh, &brush, &d, Symmetry::default());
        }
        let ours: Vec<f32> = free_of(&mesh);

        // --- O DELES: N aplicações do kernel sobre o canal VIVO, que é o que a
        // referência faz (a máscara não tem proxy — o doc do `ref_mask` é
        // explícito: ela lê `vAr`, e o acúmulo é o produto da ferramenta).
        let clean = sphere();
        let pos = flat(&clean);
        let mut theirs = vec![1.0f32; clean.vert_count()];
        for _ in 0..n {
            ph2d_sculpt3d::ref_kernels::mask(
                &mut theirs, &pos, &fp, {
                    [
                        f64::from(d.center[0]),
                        f64::from(d.center[1]),
                        f64::from(d.center[2]),
                    ]
                },
                f64::from(d.radius) * f64::from(d.radius),
                INTENSITY,
                HARDNESS,
                true,
            );
        }

        // O canal mais PROTEGIDO da pegada — o miolo, que é onde as duas leis
        // divergem primeiro (a assintótica nunca chega a 0, a aditiva satura).
        let low = |v: &[f32]| fp.iter().map(|&i| v[i as usize]).fold(1.0f32, f32::min);
        let gap = fp
            .iter()
            .map(|&i| f64::from(ours[i as usize]) - f64::from(theirs[i as usize]))
            .fold(0.0f64, |a, b| a.max(b.abs()));
        println!(
            "{n:>5} {:>16.6} {:>16.6} {:>14.3e}",
            low(&ours),
            low(&theirs),
            gap
        );
    }

    // A CURVA do canal, isolada — a segunda curva do SculptGL, que a nossa
    // família de falloff não contém.
    println!("\n== A CURVA DO CANAL (hardness 0.25 => expoente 1.5) ==");
    println!(
        "{:>6} {:>14} {:>12} {:>12} {:>12}",
        "t", "referência", "Plateau", "Smooth", "Sphere"
    );
    for i in 0..=8 {
        let t = f64::from(i) / 8.0;
        let r = (1.0 - t).powf(2.0 * (1.0 - HARDNESS));
        println!(
            "{t:>6.3} {r:>14.6} {:>12.6} {:>12.6} {:>12.6}",
            f64::from(Falloff::Plateau.weight(t as f32)),
            f64::from(Falloff::Smooth.weight(t as f32)),
            f64::from(Falloff::Sphere.weight(t as f32)),
        );
    }
    println!();
}

/// **O INTERRUPTOR FAZ ALGUMA COISA? Verbo a verbo, pela porta do produto.**
///
/// ⚠️ **A pergunta não é sobre a LEI — é sobre o EFEITO.** A metade 2 pôs o
/// Accumulate no mecanismo da referência (`from_live`), e um gate afirma que
/// ele muda o comportamento **no `Draw`**. Isso não diz nada sobre os outros
/// quinze, e *"o accumulate não funciona"* é um relato sobre uma ferramenta que
/// o artista tinha na mão — que pode não ser aquela.
///
/// A sonda esfrega o MESMO lugar quatro vezes com cada verbo, armado e
/// desarmado, e imprime a razão. **`1,00×` é um interruptor INERTE naquele
/// verbo**, e a lista dos inertes é o resultado.
#[test]
#[ignore = "sonda: imprime a tabela, não afirma nada"]
fn does_the_accumulate_switch_do_anything_verb_by_verb() {
    use ph2d_sculpt3d::Grip;
    const R: f32 = 0.45;
    const PASSES: usize = 4;
    // O mesmo raio do gate que já mede este mecanismo.
    #[allow(clippy::items_after_statements)]

    println!("\n== O ACCUMULATE, verbo a verbo ({PASSES} esfregadas no MESMO lugar) ==");
    println!(
        "{:<12} {:>8} {:>14} {:>14} {:>10}",
        "verbo", "grip", "desarmado", "armado", "razão"
    );

    for verb in Verb::ALL {
        let reach = |accumulate: bool| -> f64 {
            let mut mesh = sphere();
            let base = flat(&mesh);
            let brush = Brush {
                verb,
                strength: 0.5,
                accumulate,
                falloff: Falloff::Plateau,
                ..Brush::default()
            };
            let mut st = SculptStroke::default();
            st.begin(&mesh);
            // ⚠️ **VARREDURA, e não N dabs no mesmo ponto — a primeira versão
            // desta sonda mediu `1,01×` em toda a linha e eu quase reportei que
            // o interruptor era inerte.** O mecanismo é o vértice AFASTAR-SE do
            // proxy congelado, e quatro dabs levantam ~0,07 sobre um raio de
            // 0,45: a distância normalizada anda 0,16 e a quártica mal cai. A
            // fixture tem de conter o fenômeno, e quem o contém é o CAMINHO —
            // é o que o gate `the_disarmed_brush_exhausts_itself` já fazia.
            let step = ph2d_sculpt3d::min_spacing(R);
            let n = (1.0 / step).floor() as usize;
            for _ in 0..PASSES {
                for k in 0..=n {
                    let x = step.mul_add(k as f32, -0.5);
                    let c = live_dab_centre(&mesh, R, x);
                    st.dab(&mut mesh, &brush, &c, Symmetry::default());
                }
            }
            // A máscara não move geometria: para ela a grandeza é o canal.
            if verb.paints_mask() {
                return mesh
                    .masks()
                    .map_or(0.0, |m| f64::from(m.iter().copied().fold(0.0f32, f32::max)));
            }
            reach_of(&flat(&mesh), &base)
        };
        let (off, on) = (reach(false), reach(true));
        let ratio = if off > 1e-12 {
            format!("{:.2}x", on / off)
        } else {
            "-".into()
        };
        let grip = match verb.grip() {
            Grip::Stamp => "Stamp",
            Grip::Hold => "Hold",
            Grip::Hook => "Hook",
            Grip::Turn(_) => "Turn",
            Grip::Paint => "Paint",
        };
        println!(
            "{:<12} {grip:>8} {off:>14.6} {on:>14.6} {ratio:>10}",
            verb.label()
        );
    }
    println!();
}

/// O dab do passo seguinte, com o centro na superfície VIVA — o `hit.point` que
/// o produto entrega.
fn live_dab_centre(mesh: &Mesh, radius: f32, x: f32) -> Dab {
    // O ponto da superfície VIVA mais perto de `(x, 0)` no hemisfério `+Z` — o
    // `hit.point` do raycast, sem precisar de um raycaster na sonda. O caminho
    // anda pelo polo, onde o olho `-Z` vê tudo de frente e o culling não entra
    // na conta.
    let mut best = [x, 0.0, 1.0];
    let mut bd = f32::MAX;
    for p in mesh.positions() {
        if p[2] < 0.0 {
            continue;
        }
        let d = (p[0] - x).abs() + p[1].abs();
        if d < bd {
            bd = d;
            best = *p;
        }
    }
    Dab::at(best, radius, [0.0, 0.0, -1.0])
}
