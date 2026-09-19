//! ⭐⭐⭐⭐ **SONDA — o que um carimbo na FRENTE de uma parede fina faz nas
//! COSTAS dela, e a FOTO disso.**
//!
//! # A pergunta
//!
//! A [`sonda_do_falloff_pela_superficie`](super::sonda_do_falloff_pela_superficie)
//! mediu que o dab de hoje pesa cada vértice pela distância **pelo ar**, e que a
//! máscara de alcance que shipa (`dab_alcance`, tecto `2,00 × R`) **não apanha**
//! o caso da parede fina. Ela respondeu em **fracção de PESO** (`9,20 %` no tubo
//! que ela mede). ⛔ Isso não diz o que o artista VÊ.
//!
//! Esta sonda responde na grandeza dele — **quanto a parede de TRÁS se move, em
//! fracção do que a da FRENTE se moveu** — e desenha o corte.
//!
//! # ⛔⛔ A PRIMEIRA medição corrigiu a pergunta, e a correcção é a sonda toda
//!
//! A 1.ª redacção usou o tubo daquela sonda (menor `0,10`, pincel `0,40`) e leu
//! as costas a andar **`68,8 %`** do que a frente andou. Parecia o achado — e
//! **não é um defeito**: ali a distância *pela superfície* entre as duas paredes
//! é `π × 0,10 = 0,314`, que é **`0,78 ×` o raio do pincel** ⇒ *um pincel
//! geodésico honesto TAMBÉM lhes tocaria*, só que com menos peso. Eu tinha
//! escrito ao lado do número que «um carimbo honesto não lhes tocaria», e a
//! medição desmentiu-me.
//!
//! ⇒ **o defeito vive numa BANDA, e ela é estreita:**
//!
//! ```text
//!     ar(frente→costas)  <  R          o carimbo de hoje alcança
//!     superficie         >  R          um pincel honesto NÃO alcançaria
//!     superficie         <= 2 x R      a máscara que shipa TAMBÉM não corta
//! ```
//!
//! Fora dessa banda não há nada a curar: ou a superfície alcança de verdade, ou
//! a máscara de hoje já corta. ⚠️ *Uma régua que mede fora da banda mede um
//! produto que já está certo.*
//!
//! # ⭐ E a peça que a habita é uma BARBATANA, não um tubo
//!
//! Num tubo de raio `m` a superfície mede `π m` e o ar `2 m` ⇒ a razão é
//! **`π/2 = 1,571` FIXA**, logo a banda é um intervalo apertado de `m` e o peso
//! que escapa é sempre modesto. Numa **chapa fina** (uma orelha, uma barbatana,
//! uma folha) a razão é **livre**: o ar é a espessura e a superfície é *ir até à
//! beira e voltar* ⇒ perto da beira a superfície mede `2 d + t` e o ar mede `t`,
//! e a razão cresce sem limite à medida que a peça afina.
//!
//! ⇒ a fixtura da foto é uma barbatana de `0,06` de espessura com o carimbo a
//! `0,30` da beira, que é **o gesto normal de quem esculpe uma orelha**.
//!
//! ```text
//! bash scripts/ph2d-run.sh cargo test -p ph2d-sculpt3d --test it \
//!   sonda_da_parede_fina -- --ignored --nocapture --test-threads=1
//! ```

use ph2d_mesh::{Face, Mesh, shapes};
use ph2d_sculpt3d::{Brush, Dab, SculptStroke, Symmetry, Verb};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::fmt::Write as _;

/// O raio do pincel, em todas as células desta sonda.
const RAIO: f32 = 0.40;
/// A força — o topo da pista, porque é onde o artista trabalha uma barbatana.
const FORCA: f32 = 1.0;
/// O tecto da máscara que shipa, em raios — [`ph2d_sculpt3d`] `ALCANCE_TECTO`.
const TECTO_DE_HOJE: f32 = 2.0;

// ---------------------------------------------------------------------------
// A BARBATANA — a fixtura da foto
// ---------------------------------------------------------------------------

/// A espessura da barbatana. `0,06` numa peça de `2,0` de lado.
const ESPESSURA: f32 = 0.06;
/// Meio lado da chapa.
const MEIO: f32 = 1.0;
/// Quantos vértices por lado, em cada face.
const N: usize = 81;
/// A que distância da BEIRA o artista carimba.
const DA_BEIRA: f32 = 0.30;

/// Índice do vértice da face da FRENTE (`z = +t/2`).
fn f(i: usize, j: usize) -> u32 {
    (i * N + j) as u32
}
/// Índice do vértice da face de TRÁS (`z = −t/2`).
fn t(i: usize, j: usize) -> u32 {
    (N * N + i * N + j) as u32
}

/// **Uma chapa fina fechada** — duas grelhas `N × N` a `±t/2` costuradas por uma
/// cinta de um quad de altura na beira.
///
/// ⚠️ **A cinta reutiliza os vértices das duas faces** (não há anel extra): é
/// isso que faz o caminho pela superfície medir exactamente *ir à beira, subir a
/// espessura, e voltar*, sem um degrau inventado pela malha.
fn barbatana() -> Mesh {
    let passo = 2.0 * MEIO / (N - 1) as f32;
    let meia_esp = ESPESSURA * 0.5;
    let mut pos = Vec::with_capacity(2 * N * N);
    for meia in [meia_esp, -meia_esp] {
        for i in 0..N {
            for j in 0..N {
                pos.push([-MEIO + passo * i as f32, -MEIO + passo * j as f32, meia]);
            }
        }
    }
    let mut faces = Vec::with_capacity(2 * (N - 1) * (N - 1) + 4 * (N - 1));
    for i in 0..N - 1 {
        for j in 0..N - 1 {
            // A frente olha para `+z`.
            faces.push(Face::quad(
                f(i, j),
                f(i + 1, j),
                f(i + 1, j + 1),
                f(i, j + 1),
            ));
            // As costas olham para `−z` — a volta ao contrário.
            faces.push(Face::quad(
                t(i, j),
                t(i, j + 1),
                t(i + 1, j + 1),
                t(i + 1, j),
            ));
        }
    }
    // A cinta da beira, os quatro lados.
    for k in 0..N - 1 {
        faces.push(Face::quad(f(k + 1, 0), f(k, 0), t(k, 0), t(k + 1, 0)));
        faces.push(Face::quad(
            f(k, N - 1),
            f(k + 1, N - 1),
            t(k + 1, N - 1),
            t(k, N - 1),
        ));
        faces.push(Face::quad(f(0, k), f(0, k + 1), t(0, k + 1), t(0, k)));
        faces.push(Face::quad(
            f(N - 1, k + 1),
            f(N - 1, k),
            t(N - 1, k),
            t(N - 1, k + 1),
        ));
    }
    Mesh::from_parts(pos, faces).expect("a barbatana é construída aqui e é válida")
}

// ---------------------------------------------------------------------------
// As duas réguas
// ---------------------------------------------------------------------------

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    d[0].mul_add(d[0], d[1].mul_add(d[1], d[2] * d[2])).sqrt()
}

#[derive(PartialEq)]
struct Perto(f32, u32);
impl Eq for Perto {}
impl Ord for Perto {
    fn cmp(&self, o: &Self) -> Ordering {
        o.0.partial_cmp(&self.0).unwrap_or(Ordering::Equal)
    }
}
impl PartialOrd for Perto {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

/// A distância **pela superfície** a partir de `semente`, sem tecto.
fn pela_superficie(mesh: &Mesh, semente: u32) -> Vec<f32> {
    let pos = mesh.positions();
    let mut d = vec![f32::INFINITY; pos.len()];
    d[semente as usize] = 0.0;
    let viz = &mesh.adjacency().vert_verts;
    let mut fila = BinaryHeap::new();
    fila.push(Perto(0.0, semente));
    while let Some(Perto(du, u)) = fila.pop() {
        if du > d[u as usize] {
            continue;
        }
        for &v in viz.neighbours(u as usize) {
            let nd = du + dist(pos[u as usize], pos[v as usize]);
            if nd < d[v as usize] {
                d[v as usize] = nd;
                fila.push(Perto(nd, v));
            }
        }
    }
    d
}

/// ⭐⭐⭐ **A geodésica EXACTA da barbatana, por DESDOBRAMENTO** — e ela existe
/// aqui porque o passeio por arestas não serve para desenhar a cura.
///
/// ⛔⛔ **O passeio por arestas sobrestima até `√2`** (numa grelha quadrada ir a
/// `45°` custa `2n` arestas onde a superfície mede `n√2`). Usá-lo como régua da
/// CURA cortaria vértices da **FRENTE** que estão dentro do raio, só por estarem
/// na diagonal — a figura mostraria a cura a comer o relevo do artista, e isso
/// seria um artefacto do instrumento, não do desenho. *É exactamente o mesmo
/// viés que obriga o tecto da máscara que shipa a ser `2,00 × R`.*
///
/// Numa chapa a geodésica escreve-se à mão:
/// * **para a FRENTE**, é a recta no plano da face — a chapa é convexa, logo o
///   segmento nunca sai dela;
/// * **para as COSTAS**, desdobra-se a face de trás por cima de cada uma das
///   quatro beiras (a de `x = +M` manda `x ↦ 2M + t − x`) e mede-se a recta no
///   plano desdobrado.
///
/// ⚠️ **O mínimo das quatro é um LIMITE INFERIOR** — um caminho que tivesse de
/// contornar um CANTO seria mais longo —, e a direcção do erro é a que interessa:
/// ele só pode fazer a cura cortar **menos**. *Uma aproximação que favorece o
/// lado que já estava lá não infla o resultado que a figura afirma.*
fn geodesica_da_barbatana(centro: [f32; 3]) -> Vec<f32> {
    let (x0, y0) = (centro[0], centro[1]);
    let passo = 2.0 * MEIO / (N - 1) as f32;
    let mut d = Vec::with_capacity(2 * N * N);
    for costas in [false, true] {
        for i in 0..N {
            for j in 0..N {
                let x = -MEIO + passo * i as f32;
                let y = -MEIO + passo * j as f32;
                d.push(if costas {
                    let dobra = |a: f32, borda: f32| 2.0 * borda + ESPESSURA.copysign(borda) - a;
                    [
                        (dobra(x, MEIO) - x0).hypot(y - y0),
                        (dobra(x, -MEIO) - x0).hypot(y - y0),
                        (x - x0).hypot(dobra(y, MEIO) - y0),
                        (x - x0).hypot(dobra(y, -MEIO) - y0),
                    ]
                    .into_iter()
                    .fold(f32::INFINITY, f32::min)
                } else {
                    (x - x0).hypot(y - y0)
                });
            }
        }
    }
    d
}

/// O vértice da malha mais perto de `p`.
fn semente(mesh: &Mesh, p: [f32; 3]) -> u32 {
    let mut melhor = (f32::INFINITY, 0u32);
    for (i, q) in mesh.positions().iter().enumerate() {
        let d = dist(*q, p);
        if d < melhor.0 {
            melhor = (d, i as u32);
        }
    }
    melhor.1
}

/// Um carimbo pelo caminho do PRODUTO — a malha depois dele.
fn carimba(repouso: &Mesh, centro: [f32; 3], olho: [f32; 3]) -> Mesh {
    let mut m = repouso.clone();
    let b = Brush {
        verb: Verb::Draw,
        radius: RAIO,
        strength: FORCA,
        ..Brush::default()
    };
    let mut st = SculptStroke::default();
    st.begin(&m);
    st.dab(
        &mut m,
        &b,
        &Dab::at(centro, RAIO, olho),
        Symmetry::default(),
    );
    m
}

// ---------------------------------------------------------------------------
// (1) A BANDA — onde o defeito vive, e onde não há nada a curar
// ---------------------------------------------------------------------------

#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn a_banda_onde_o_defeito_vive() {
    println!("\n== (1) A BANDA — um tubo, a varrer a espessura da parede ==");
    println!("   pincel R = {RAIO:.2}\n");
    println!(
        "{:>8} {:>8} {:>10} {:>9} | {:>9} {:>9} | {:>7} {:>7}",
        "menor", "parede", "superficie", "sup/R", "frente", "costas", "costas%", "veredito"
    );
    println!("{}", "-".repeat(86));

    for menor in [0.05f32, 0.08, 0.10, 0.13, 0.15, 0.17, 0.19, 0.25] {
        let repouso = shapes::torus(96, 48, 1.0, menor);
        let centro = [1.0 + menor, 0.0, 0.0];
        let depois = carimba(&repouso, centro, [-1.0, 0.0, 0.0]);
        // ⭐ Aqui o passeio por arestas é EXACTO na direcção que decide: o
        // círculo MENOR do toro é uma linha da malha, logo frente→costas anda
        // por arestas sem diagonal nenhuma.
        let geo = pela_superficie(&repouso, semente(&repouso, centro));
        let (a, z) = (repouso.positions(), depois.positions());
        let costas = semente(&repouso, [1.0 - menor, 0.0, 0.0]) as usize;
        let frente = semente(&repouso, centro) as usize;
        let d_f = dist(a[frente], z[frente]);
        let d_c = dist(a[costas], z[costas]);
        let sup = geo[costas];
        let veredito = if 2.0 * menor >= RAIO {
            "o ar nao alcanca"
        } else if sup <= RAIO {
            "a superficie alcanca"
        } else if sup > TECTO_DE_HOJE * RAIO {
            "a mascara de hoje corta"
        } else {
            "*** DEFEITO ***"
        };
        println!(
            "{menor:>8.2} {:>8.2} {sup:>10.3} {:>9.2} | {d_f:>9.4} {d_c:>9.4} | {:>6.1}% {veredito:>7}",
            2.0 * menor,
            sup / RAIO,
            100.0 * d_c / d_f.max(1e-9)
        );
    }
    println!(
        "\n  LEITURA: num TUBO a razao superficie/ar e' `pi/2 = 1,571` FIXA, entao a\n  \
         banda e' um intervalo apertado de espessura e o que escapa e' modesto. Numa\n  \
         CHAPA a razao e' livre -- e' por isso que a foto usa uma barbatana."
    );
}

// ---------------------------------------------------------------------------
// (2) A BARBATANA — os números na grandeza do artista
// ---------------------------------------------------------------------------

/// O centro do carimbo: na face da frente, a [`DA_BEIRA`] da beira `x = +MEIO`.
fn centro_do_carimbo() -> [f32; 3] {
    [MEIO - DA_BEIRA, 0.0, ESPESSURA * 0.5]
}

#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn quanto_as_costas_andam_quando_a_frente_e_esculpida() {
    let repouso = barbatana();
    let centro = centro_do_carimbo();
    let depois = carimba(&repouso, centro, [0.0, 0.0, -1.0]);
    let geo = geodesica_da_barbatana(centro);
    let (a, z) = (repouso.positions(), depois.positions());

    let frente = semente(&repouso, centro) as usize;
    let costas = semente(&repouso, [centro[0], centro[1], -ESPESSURA * 0.5]) as usize;
    let d_f = dist(a[frente], z[frente]);
    let d_c = dist(a[costas], z[costas]);

    println!("\n== (2) A BARBATANA — um carimbo na frente, a {DA_BEIRA:.2} da beira ==");
    println!(
        "  peca              {:.1} x {:.1}, {ESPESSURA:.2} de espessura",
        2.0 * MEIO,
        2.0 * MEIO
    );
    println!("  pincel            R = {RAIO:.2}, forca {FORCA:.2}");
    println!();
    println!(
        "  frente -> costas  pelo AR          {ESPESSURA:.3}   ({:.2}x o raio)",
        ESPESSURA / RAIO
    );
    println!(
        "                    pela SUPERFICIE  {:.3}   ({:.2}x o raio)",
        geo[costas],
        geo[costas] / RAIO
    );
    println!(
        "                    (a mascara de hoje so' corta acima de {:.3})",
        TECTO_DE_HOJE * RAIO
    );
    println!();
    println!("  a FRENTE andou    {d_f:.4}");
    println!(
        "  as COSTAS andaram {d_c:.4}   = {:.1}% do que a frente andou",
        100.0 * d_c / d_f.max(1e-9)
    );
    println!(
        "  (a espessura da peca e' {ESPESSURA:.2} -- a frente anda {:.0}% dela)",
        100.0 * d_f / ESPESSURA
    );

    let (mut n_longe, mut peso_longe, mut n_perto, mut peso_perto) =
        (0usize, 0.0f64, 0usize, 0.0f64);
    for i in 0..a.len() {
        let d = dist(a[i], z[i]);
        if d <= 0.0 {
            continue;
        }
        if geo[i] > RAIO {
            n_longe += 1;
            peso_longe += f64::from(d);
        } else {
            n_perto += 1;
            peso_perto += f64::from(d);
        }
    }
    let tot = peso_longe + peso_perto;
    println!();
    println!("  vertices que a superficie ALCANCA:      {n_perto:>5}");
    println!(
        "  vertices que a superficie NAO alcanca:  {n_longe:>5}   = {:.1}% de TODO o movimento do carimbo",
        100.0 * peso_longe / tot.max(1e-12)
    );

    // ⚠️⚠️ **O CONTROLO da figura: a cura não pode comer o relevo do artista.**
    // Se ela tirasse vértices da FRENTE, o painel verde estaria a mostrar uma
    // régua com viés e não o desenho — e foi por isso que a geodésica desta
    // fixtura deixou de ser o passeio por arestas.
    let comidos = (0..N * N)
        .filter(|&k| geo[k] > RAIO && dist(a[k], z[k]) > 0.0)
        .count();
    println!();
    println!("  CONTROLO — vertices da FRENTE que a cura tiraria: {comidos}  (tem de ser 0)");
}

// ---------------------------------------------------------------------------
// (3) A FOTO
// ---------------------------------------------------------------------------

#[test]
#[ignore = "sonda: escreve as fotos, nao afirma nada"]
fn desenha_o_corte_da_barbatana() {
    let repouso = barbatana();
    let centro = centro_do_carimbo();
    let depois = carimba(&repouso, centro, [0.0, 0.0, -1.0]);
    let geo = geodesica_da_barbatana(centro);
    let (a, z) = (repouso.positions(), depois.positions());

    // ⚠️ **O painel da CURA é o MESMO carimbo com a máscara certa** — quem a
    // superfície não alcança volta ao repouso, e mais nada muda. É a definição
    // de uma máscara, e é por isso que ele é honesto: não há lei nova, não há
    // peso novo, não há número afinado.
    let curado: Vec<[f32; 3]> = (0..a.len())
        .map(|i| if geo[i] > RAIO { a[i] } else { z[i] })
        .collect();

    // O corte é a linha `j = N/2` (`y = 0`): a frente de `x = −1` a `+1`, e as
    // costas de volta. As duas pontas fecham pela cinta da beira.
    let jc = N / 2;
    let mut corte: Vec<usize> = (0..N).map(|i| f(i, jc) as usize).collect();
    corte.extend((0..N).rev().map(|i| t(i, jc) as usize));

    let dir = std::env::var("PH2D_SONDA_DIR").unwrap_or_else(|_| "target/sonda".into());
    std::fs::create_dir_all(&dir).expect("a pasta da sonda");

    let c1 = format!("{dir}/parede_fina_corte.svg");
    std::fs::write(&c1, desenha(&corte, a, z, &curado, centro)).expect("escrever o corte");
    println!("\n  foto (o corte):   {c1}");

    let c2 = format!("{dir}/parede_fina_costas.svg");
    std::fs::write(&c2, desenha_as_costas(a, z, &curado, centro)).expect("escrever as costas");
    println!("  foto (as costas): {c2}");
}

/// ⭐⭐⭐⭐ **A FACE QUE O ARTISTA NÃO ESTÁ A TOCAR**, vista de trás, com a cor a
/// dizer quanto cada ponto dela andou.
///
/// ⚠️ **Os dois painéis são normalizados pelo MESMO número** — o pico da face da
/// FRENTE —, logo a comparação é de quantidade e não de paleta. *Duas rampas
/// diferentes fariam a figura decidir o que a medição devia decidir.*
fn desenha_as_costas(
    a: &[[f32; 3]],
    z: &[[f32; 3]],
    curado: &[[f32; 3]],
    centro: [f32; 3],
) -> String {
    const L: f32 = 470.0;
    let px = |x: f32, y: f32| -> (f32, f32) {
        // Visto DE TRÁS: o `x` cresce para a esquerda, senão a figura é o
        // espelho do que o artista veria ao rodar a peça.
        (L * 0.5 - x / MEIO * L * 0.46, L * 0.5 - y / MEIO * L * 0.46)
    };
    // A referência: o quanto a FRENTE andou no pico.
    let pico = (0..N * N)
        .map(|k| dist(a[k], z[k]))
        .fold(0.0f32, f32::max)
        .max(1e-9);

    let celulas = |pts: &[[f32; 3]]| -> String {
        let mut s = String::new();
        for i in 0..N - 1 {
            for j in 0..N - 1 {
                let k = |ii: usize, jj: usize| {
                    let v = t(ii, jj) as usize;
                    dist(a[v], pts[v])
                };
                let d = 0.25 * (k(i, j) + k(i + 1, j) + k(i, j + 1) + k(i + 1, j + 1));
                let q = (d / pico).clamp(0.0, 1.0);
                if q < 0.004 {
                    continue;
                }
                // Rampa do fundo do painel ao âmbar.
                let cor = |c0: f32, c1: f32| (c0 + (c1 - c0) * q).round() as u32;
                let (r, g, b) = (cor(22.0, 255.0), cor(24.0, 150.0), cor(29.0, 60.0));
                let p = |ii: usize, jj: usize| {
                    let v = t(ii, jj) as usize;
                    px(a[v][0], a[v][1])
                };
                let (x0, y0) = p(i, j);
                let (x1, y1) = p(i + 1, j + 1);
                let _ = write!(
                    s,
                    r##"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="#{r:02x}{g:02x}{b:02x}"/>"##,
                    x0.min(x1),
                    y0.min(y1),
                    (x1 - x0).abs() + 0.6,
                    (y1 - y0).abs() + 0.6
                );
            }
        }
        s
    };

    let (cx, cy) = px(centro[0], centro[1]);
    let r_px = RAIO / MEIO * L * 0.46;
    // O pico das COSTAS, em fracção do da frente — o número que a figura afirma.
    let pico_costas = (0..N * N)
        .map(|k| dist(a[N * N + k], z[N * N + k]))
        .fold(0.0f32, f32::max);

    let painel = |dx: f32, titulo: &str, sub: &str, corpo: &str, cor: &str| {
        format!(
            r##"<g transform="translate({dx},170)">
  <text x="{tx}" y="-44" fill="#f0f2f5" font-family="DejaVu Sans, sans-serif" font-size="29" font-weight="700" text-anchor="middle">{titulo}</text>
  <text x="{tx}" y="-16" fill="{cor}" font-family="DejaVu Sans, sans-serif" font-size="20" text-anchor="middle">{sub}</text>
  <rect x="0" y="0" width="{L}" height="{L}" rx="5" fill="#16181d" stroke="#2c3038" stroke-width="1"/>
  {corpo}
  <rect x="{pb:.1}" y="{pb:.1}" width="{pl:.1}" height="{pl:.1}" fill="none" stroke="#3d434e" stroke-width="1.6"/>
  <circle cx="{cx:.1}" cy="{cy:.1}" r="{r_px:.1}" fill="none" stroke="#7d8798" stroke-width="1.6" stroke-dasharray="6 6"/>
</g>"##,
            tx = L * 0.5,
            pb = L * 0.04,
            pl = L * 0.92,
        )
    };

    let w = L * 2.0 + 150.0;
    let h = L + 296.0;
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">
<rect width="{w}" height="{h}" fill="#0d0f13"/>
<text x="{tx}" y="46" fill="#f0f2f5" font-family="DejaVu Sans, sans-serif" font-size="25" font-weight="600" text-anchor="middle">a MESMA barbatana vista POR TRAS — o lado em que o artista NAO tocou</text>
<text x="{tx}" y="76" fill="#8b93a1" font-family="DejaVu Sans, sans-serif" font-size="19" text-anchor="middle">a cor diz quanto cada ponto se mexeu · o tracejado marca onde ficou o pincel, do outro lado</text>
{p1}
{p2}
<text x="{tx}" y="{ry}" fill="#c9cfda" font-family="DejaVu Sans, sans-serif" font-size="21" text-anchor="middle">no pior ponto, as costas andam {pct:.0}% do que a frente andou</text>
<text x="{tx}" y="{ry2}" fill="#5d6472" font-family="DejaVu Sans, sans-serif" font-size="17" text-anchor="middle">PH2D — sonda da parede fina</text>
</svg>"##,
        tx = w * 0.5,
        pct = 100.0 * pico_costas / pico,
        p1 = painel(
            60.0,
            "HOJE",
            "as costas foram esculpidas junto",
            &celulas(z),
            "#e05b5b"
        ),
        p2 = painel(
            60.0 + L + 30.0,
            "COM A CURA",
            "as costas ficam onde estavam",
            &celulas(curado),
            "#4a9d5f"
        ),
        ry = 170.0 + L + 44.0,
        ry2 = h - 24.0,
    )
}

/// Dois painéis empilhados, no MESMO enquadramento e na MESMA escala — e a
/// escala é **igual nos dois eixos**, sem exagero nenhum.
fn desenha(
    corte: &[usize],
    a: &[[f32; 3]],
    z: &[[f32; 3]],
    curado: &[[f32; 3]],
    centro: [f32; 3],
) -> String {
    const LARG: f32 = 1040.0;
    const ALT: f32 = 300.0;
    // A janela: `0,80` de largura à volta do carimbo, com a altura a sair da
    // razão do painel ⇒ escala igual nos dois eixos, por construção.
    let janela_x = 0.80f32;
    let esc = LARG / janela_x;
    let janela_z = ALT / esc;
    let x0 = centro[0] - janela_x * 0.62;
    let px = |p: [f32; 3]| -> (f32, f32) {
        ((p[0] - x0) * esc, ALT * 0.5 - p[2] * esc + janela_z * 0.0)
    };

    let caminho = |pts: &dyn Fn(usize) -> [f32; 3]| -> String {
        let mut d = String::new();
        for (n, &i) in corte.iter().enumerate() {
            let (x, y) = px(pts(i));
            let _ = write!(d, "{}{x:.2},{y:.2} ", if n == 0 { "M" } else { "L" });
        }
        d.push('Z');
        d
    };
    let repouso_d = caminho(&|i| a[i]);
    let hoje_d = caminho(&|i| z[i]);
    let curado_d = caminho(&|i| curado[i]);

    let (cx, _) = px(centro);
    // ⚠️⚠️ **Nem o CÍRCULO do pincel nem a CINTA da largura dele servem aqui, e
    // as duas foram desenhadas antes de se ver porquê:** o painel tem `0,30` de
    // altura e o pincel `0,40` de raio, logo o círculo deixa um arco solto que
    // se lê como uma linha da PEÇA, e a cinta é mais larga que o painel inteiro
    // — ela pinta tudo menos uma tira à esquerda, e essa tira lê-se como sendo
    // o assunto. *Um indicador maior que o enquadramento não indica nada.*
    // ⇒ fica a SETA, que é a única coisa que a figura precisa de dizer: onde a
    // mão empurra.
    let (bx, _) = px([MEIO, 0.0, 0.0]);

    let painel = |dy: f32, titulo: &str, sub: &str, forma: &str, cor: &str| -> String {
        format!(
            r##"<g transform="translate(70,{dy})">
  <text x="0" y="-42" fill="#f0f2f5" font-family="DejaVu Sans, sans-serif" font-size="30" font-weight="700">{titulo}</text>
  <text x="0" y="-15" fill="{cor}" font-family="DejaVu Sans, sans-serif" font-size="21">{sub}</text>
  <clipPath id="c{dy}"><rect x="0" y="0" width="{LARG}" height="{ALT}"/></clipPath>
  <rect x="0" y="0" width="{LARG}" height="{ALT}" rx="5" fill="#16181d" stroke="#2c3038" stroke-width="1"/>
  <g clip-path="url(#c{dy})">
    <path d="{repouso_d}" fill="none" stroke="#4a505c" stroke-width="2" stroke-dasharray="7 6"/>
    <path d="{forma}" fill="{cor}26" stroke="{cor}" stroke-width="3.5" stroke-linejoin="round"/>
    <path d="M{cx:.1},16 L{cx:.1},74 M{l:.1},56 L{cx:.1},76 L{r:.1},56" fill="none" stroke="#e8a33d" stroke-width="3.5"/>
    <line x1="{bx:.1}" y1="0" x2="{bx:.1}" y2="{ALT}" stroke="#6b7484" stroke-width="1.2" stroke-dasharray="3 5"/>
  </g>
</g>"##,
            l = cx - 14.0,
            r = cx + 14.0,
        )
    };

    let w = LARG + 140.0;
    let y1 = 158.0;
    let y2 = y1 + ALT + 118.0;
    let h = y2 + ALT + 96.0;
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">
<rect width="{w}" height="{h}" fill="#0d0f13"/>
<text x="{tx}" y="44" fill="#f0f2f5" font-family="DejaVu Sans, sans-serif" font-size="25" font-weight="600" text-anchor="middle">uma barbatana fina vista DE LADO — a seta e' o sitio onde o pincel empurra</text>
<text x="{tx}" y="74" fill="#8b93a1" font-family="DejaVu Sans, sans-serif" font-size="19" text-anchor="middle">peca com {esp:.2} de espessura · pincel com {RAIO:.2} de raio · o tracejado e' a forma antes do traco · escala igual nos dois eixos</text>
{a1}
{a2}
<text x="{bl:.0}" y="{ry}" fill="#6b7484" font-family="DejaVu Sans, sans-serif" font-size="17" text-anchor="end">a beira da peca ↑</text>
<text x="{tx}" y="{ry2}" fill="#5d6472" font-family="DejaVu Sans, sans-serif" font-size="17" text-anchor="middle">PH2D — sonda da parede fina</text>
</svg>"##,
        tx = w * 0.5,
        esp = ESPESSURA,
        bl = 70.0 + bx - 6.0,
        a1 = painel(
            y1,
            "HOJE",
            "a face de BAIXO vai junto — a peca desloca-se em vez de ganhar relevo",
            &hoje_d,
            "#e05b5b"
        ),
        a2 = painel(
            y2,
            "COM A CURA",
            "so' a face de cima se move — o relevo fica onde o artista o pos",
            &curado_d,
            "#4a9d5f"
        ),
        ry = y2 + ALT + 30.0,
        ry2 = h - 24.0,
    )
}
