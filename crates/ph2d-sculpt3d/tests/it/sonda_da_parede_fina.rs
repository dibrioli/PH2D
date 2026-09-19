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
    carimba_com(repouso, centro, olho, true)
}

/// O mesmo, com a máscara **desarmada** — é o que o `Connected Only` desligado
/// entrega, e a matéria-prima de que o painel do ANTES é feito.
fn carimba_com(repouso: &Mesh, centro: [f32; 3], olho: [f32; 3], mascara: bool) -> Mesh {
    let mut m = repouso.clone();
    let b = Brush {
        verb: Verb::Draw,
        radius: RAIO,
        strength: FORCA,
        surface_only: mascara,
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

    // ⚠️⚠️ **O resíduo mede-se contra a LEI QUE SHIPA, e não contra um critério
    // ideal que o produto não aplica.** A 1.ª redacção desta régua perguntava
    // *«quem se moveu apesar de a superfície não alcançar dentro de UM raio?»* e
    // lia `24 %` sobre um produto que já estava a cortar tudo o que a lei manda
    // — ela media a distância entre a lei e um ideal, e leu-se como resíduo.
    let (mut n_residuo, mut peso_residuo, mut peso_total) = (0usize, 0.0f64, 0.0f64);
    for i in 0..a.len() {
        let d = dist(a[i], z[i]);
        if d <= 0.0 {
            continue;
        }
        peso_total += f64::from(d);
        let ar = dist(a[i], centro);
        if ar > 1e-6 && geo[i] > ph2d_sculpt3d::RAZAO_MAXIMA * ar {
            n_residuo += 1;
            peso_residuo += f64::from(d);
        }
    }
    println!();
    println!(
        "  RESIDUO — movidos que a lei devia ter cortado: {n_residuo}  ({:.2}% do movimento)",
        100.0 * peso_residuo / peso_total.max(1e-12)
    );

    // ⭐⭐ **A grandeza do REPORT: quanto das COSTAS foi esculpido, ao todo.**
    // ⚠️ «o pior ponto» deixou de servir assim que a cura entrou — ele mudou-se
    // para a BEIRA, onde a superfície dá a volta em `0,06` e o carimbo alcança
    // as costas com razão. *Uma régua de extremo segue o extremo para onde ele
    // for, e o extremo mudou de sítio.*
    // ⚠️ O ANTES é a **lei antiga** e não «sem máscara nenhuma»: a figura irmã
    // compara produto com produto, e duas réguas da mesma grandeza a discordar
    // seria o achado errado.
    let cru = carimba_com(&repouso, centro, [0.0, 0.0, -1.0], false);
    let sem_mascara = cru.positions();
    let sem: Vec<[f32; 3]> = (0..a.len())
        .map(|i| {
            if geo[i] > ph2d_sculpt3d::ALCANCE_TECTO * RAIO {
                a[i]
            } else {
                sem_mascara[i]
            }
        })
        .collect();
    let sem = &sem[..];
    let (mut t_antes, mut t_agora, mut mais_fundo) = (0.0f64, 0.0f64, 0.0f32);
    for k in 0..N * N {
        let v = N * N + k;
        let d_antes = dist(a[v], sem[v]);
        let d_agora = dist(a[v], z[v]);
        t_antes += f64::from(d_antes);
        t_agora += f64::from(d_agora);
        if d_agora > 1e-6 {
            mais_fundo = mais_fundo.max(MEIO - a[v][0]);
        }
    }
    println!();
    println!(
        "  as COSTAS, ao todo:  antes {t_antes:.3} · agora {t_agora:.3}   = {:.1}% do que era",
        100.0 * t_agora / t_antes.max(1e-12)
    );
    let (mut f_antes, mut f_agora) = (0.0f64, 0.0f64);
    for k in 0..N * N {
        f_antes += f64::from(dist(a[k], sem[k]));
        f_agora += f64::from(dist(a[k], z[k]));
    }
    println!(
        "  do carimbo INTEIRO, a fraccao que cai nas costas: {:.1}% -> {:.1}%",
        100.0 * t_antes / (t_antes + f_antes).max(1e-12),
        100.0 * t_agora / (t_agora + f_agora).max(1e-12)
    );
    println!(
        "  e o que sobra vive a <= {mais_fundo:.3} da beira — ⚠️ MAIS FUNDO que o `{:.3}`\n  \
         que a formula do ponto EM FRENTE da', porque um ponto das costas deslocado\n  \
         DE LADO tem o ar grande tambem, logo a razao dele e' pequena e a superficie\n  \
         alcanca-o de verdade",
        (ph2d_sculpt3d::RAZAO_MAXIMA - 1.0) * 0.5 * ESPESSURA
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
    let geo = geodesica_da_barbatana(centro);
    let a = repouso.positions();

    // ⭐⭐⭐ **O painel do DEPOIS é o PRODUTO, e o do ANTES é a lei de ANTES —
    // nenhum dos dois é um ideal desenhado à mão.**
    //
    // ⚠️ A 1.ª redacção desta figura punha «com a cura» a ser *o carimbo com uma
    // máscara ideal a `1,0 × R`*, e o produto acabou por shipar outra lei (a
    // RAZÃO). Uma figura que mostra um ideal que o código não implementa é uma
    // promessa, não uma medição.
    //
    // O ANTES reconstrói-se exactamente: o carimbo **sem** máscara, com o corte
    // que a lei antiga fazia — *quem a superfície não alcança dentro de
    // `ALCANCE_TECTO × R`*. Na barbatana o caminho frente→beira→costas corre ao
    // longo das filas da grelha, **sem diagonal nenhuma**, logo o passeio por
    // arestas de então e a geodésica exacta de agora dão o mesmo número: a
    // reconstrução é fiel.
    let cru = carimba_com(&repouso, centro, [0.0, 0.0, -1.0], false);
    let cru = cru.positions();
    let antes: Vec<[f32; 3]> = (0..a.len())
        .map(|i| {
            if geo[i] > ph2d_sculpt3d::ALCANCE_TECTO * RAIO {
                a[i]
            } else {
                cru[i]
            }
        })
        .collect();
    let agora = carimba(&repouso, centro, [0.0, 0.0, -1.0]);
    let z = agora.positions();

    // O corte é a linha `j = N/2` (`y = 0`): a frente de `x = −1` a `+1`, e as
    // costas de volta. As duas pontas fecham pela cinta da beira.
    let jc = N / 2;
    let mut corte: Vec<usize> = (0..N).map(|i| f(i, jc) as usize).collect();
    corte.extend((0..N).rev().map(|i| t(i, jc) as usize));

    let dir = std::env::var("PH2D_SONDA_DIR").unwrap_or_else(|_| "target/sonda".into());
    std::fs::create_dir_all(&dir).expect("a pasta da sonda");

    let c1 = format!("{dir}/parede_fina_corte.svg");
    std::fs::write(&c1, desenha(&corte, a, &antes, z, centro)).expect("escrever o corte");
    println!("\n  foto (o corte):   {c1}");

    let c2 = format!("{dir}/parede_fina_costas.svg");
    std::fs::write(&c2, desenha_as_costas(a, &antes, z, centro)).expect("escrever as costas");
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
    antes: &[[f32; 3]],
    agora: &[[f32; 3]],
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
        .map(|k| dist(a[k], antes[k]))
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
    // ⚠️⚠️ **«O PIOR PONTO» deixou de servir de manchete assim que a cura
    // entrou** — ele mudou-se para a BEIRA, onde a superfície dá a volta em
    // `0,06` e o carimbo alcança as costas com razão. *Uma régua de extremo
    // segue o extremo para onde ele for.* ⇒ a figura afirma duas coisas: a
    // FRACÇÃO do carimbo que cai do lado errado, e o ponto EM FRENTE do pincel,
    // que é o que faz a peça deslizar em vez de ganhar relevo.
    let soma = |pts: &[[f32; 3]], base: usize| -> f64 {
        (0..N * N)
            .map(|k| f64::from(dist(a[base + k], pts[base + k])))
            .sum()
    };
    let (cb_antes, cb_agora) = (soma(antes, N * N), soma(agora, N * N));
    let (fr_antes, fr_agora) = (soma(antes, 0), soma(agora, 0));
    let em_frente = |pts: &[[f32; 3]]| -> f32 {
        let v = N * N
            + (0..N * N)
                .min_by(|&i, &j| {
                    let d = |k: usize| (a[k][0] - centro[0]).hypot(a[k][1] - centro[1]);
                    d(i).partial_cmp(&d(j)).unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap_or(0);
        dist(a[v], pts[v])
    };
    let (ef_antes, ef_agora) = (em_frente(antes), em_frente(agora));

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
    let h = L + 330.0;
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">
<rect width="{w}" height="{h}" fill="#0d0f13"/>
<text x="{tx}" y="46" fill="#f0f2f5" font-family="DejaVu Sans, sans-serif" font-size="25" font-weight="600" text-anchor="middle">a MESMA barbatana vista POR TRAS — o lado em que o artista NAO tocou</text>
<text x="{tx}" y="76" fill="#8b93a1" font-family="DejaVu Sans, sans-serif" font-size="19" text-anchor="middle">a cor diz quanto cada ponto se mexeu · o tracejado marca onde ficou o pincel, do outro lado</text>
{p1}
{p2}
<text x="{tx}" y="{ry}" fill="#c9cfda" font-family="DejaVu Sans, sans-serif" font-size="22" text-anchor="middle">do carimbo inteiro, o que cai nas costas: <tspan fill="#e05b5b">{pct:.0}%</tspan>  →  <tspan fill="#4a9d5f">{pct2:.0}%</tspan></text>
<text x="{tx}" y="{ry3}" fill="#c9cfda" font-family="DejaVu Sans, sans-serif" font-size="22" text-anchor="middle">e EM FRENTE do pincel, onde a peca deslizava: <tspan fill="#e05b5b">{ef:.0}%</tspan>  →  <tspan fill="#4a9d5f">{ef2:.0}%</tspan></text>
<text x="{tx}" y="{ry2}" fill="#5d6472" font-family="DejaVu Sans, sans-serif" font-size="17" text-anchor="middle">PH2D — sonda da parede fina</text>
</svg>"##,
        tx = w * 0.5,
        pct = 100.0 * cb_antes / (cb_antes + fr_antes).max(1e-12),
        pct2 = 100.0 * cb_agora / (cb_agora + fr_agora).max(1e-12),
        ef = 100.0 * f64::from(ef_antes) / f64::from(pico).max(1e-12),
        ef2 = 100.0 * f64::from(ef_agora) / f64::from(pico).max(1e-12),
        ry3 = 170.0 + L + 76.0,
        p1 = painel(
            60.0,
            "ANTES",
            "as costas foram esculpidas junto",
            &celulas(antes),
            "#e05b5b"
        ),
        p2 = painel(
            60.0 + L + 30.0,
            "AGORA",
            "so' a orla junto a' beira, onde a superficie DA' a volta",
            &celulas(agora),
            "#4a9d5f"
        ),
        ry = 170.0 + L + 44.0,
        ry2 = h - 22.0,
    )
}

/// Dois painéis empilhados, no MESMO enquadramento e na MESMA escala — e a
/// escala é **igual nos dois eixos**, sem exagero nenhum.
fn desenha(
    corte: &[usize],
    a: &[[f32; 3]],
    antes: &[[f32; 3]],
    agora: &[[f32; 3]],
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
    let antes_d = caminho(&|i| antes[i]);
    let agora_d = caminho(&|i| agora[i]);

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
            "ANTES",
            "a face de BAIXO vai junto — a peca desloca-se em vez de ganhar relevo",
            &antes_d,
            "#e05b5b"
        ),
        a2 = painel(
            y2,
            "AGORA",
            "so' a face de cima se move — o relevo fica onde o artista o pos",
            &agora_d,
            "#4a9d5f"
        ),
        ry = y2 + ALT + 30.0,
        ry2 = h - 24.0,
    )
}

// ---------------------------------------------------------------------------
// (4) O TECTO — de onde sai o número, agora que a lei mudou
// ---------------------------------------------------------------------------

/// ⭐⭐⭐⭐ **A varredura que DERIVA o [`ph2d_sculpt3d::ALCANCE_TECTO`].**
///
/// Ela imprime, para cada factor, **que fracção do peso do carimbo o corte
/// leva**. A constante é o menor factor cujas linhas **APROVADAS** leem
/// `0,00 %` — cortar peso numa peça que a superfície alcança seria a cura a
/// mexer no que já está certo — e cujas linhas de **DEFEITO** continuam altas.
///
/// ⚠️⚠️ **É a MESMA pergunta da sonda de 2026-09-10 e a lei por baixo mudou**,
/// logo a tabela tem de ser refeita: aquele `2,00` existia para caber o viés
/// `√2` do passeio por arestas, e a marcha não o tem. *Uma constante derivada de
/// uma lei não sobrevive à troca dessa lei.*
#[test]
#[ignore = "sonda: de onde sai o tecto do alcance"]
fn com_que_tecto_o_corte_nao_toca_o_que_a_superficie_alcanca() {
    use ph2d_mesh::{Geodesica, QueryScratch};
    use ph2d_sculpt3d::Falloff;

    const FACTORES: [f32; 7] = [1.00, 1.10, 1.20, 1.30, 1.50, 2.00, 3.00];
    let curva = Falloff::default();
    println!("\n== (4) O TECTO — quanto do peso cada factor CORTA ==");
    print!("{:>28} {:>6} |", "peca", "raio");
    for f in FACTORES {
        print!(" {f:>6.2}x");
    }
    println!();
    println!("{}", "-".repeat(28 + 9 + FACTORES.len() * 8));

    let linha = |nome: &str, m: &Mesh, centro: [f32; 3], r: f32| {
        let mut scratch = QueryScratch::default();
        let mut bola = Vec::new();
        m.verts_in_sphere(centro, r, &mut scratch, &mut bola);
        let pos = m.positions();
        let total: f64 = bola
            .iter()
            .map(|&i| f64::from(curva.weight((dist(pos[i as usize], centro) / r).min(1.0))))
            .sum();
        if total <= 0.0 {
            return;
        }
        let s = semente(m, centro);
        let mut g = Geodesica::default();
        print!("{nome:>28} {r:>6.2} |");
        for f in FACTORES {
            g.marcha(m, s, f * r);
            let cortado: f64 = bola
                .iter()
                .filter(|&&i| !g.alcanca(i))
                .map(|&i| f64::from(curva.weight((dist(pos[i as usize], centro) / r).min(1.0))))
                .sum();
            let pct = 100.0 * cortado / total;
            print!(" {:>5.2}%", if pct.abs() < 1e-9 { 0.0 } else { pct });
        }
        println!();
    };

    // ⚠️ **O lado APROVADO primeiro** — é ele que escolhe a constante.
    let esfera = shapes::uv_sphere(64, 128, 1.0);
    for r in [0.10f32, 0.20, 0.35, 0.50] {
        linha("esfera R=1 (APROVADO)", &esfera, [0.0, 0.0, 1.0], r);
    }
    let tubo = shapes::torus(96, 48, 1.0, 0.10);
    for r in [0.20f32, 0.40] {
        linha("tubo menor=0,10 (APROVADO)", &tubo, [1.10, 0.0, 0.0], r);
    }
    // A peça esculpida, que é a que o gate da orelha defende.
    let orelha = shapes::uv_sphere_noisy(48, 96, 1.0, 0.08);
    for r in [0.20f32, 0.40] {
        linha("esfera rugosa (APROVADO)", &orelha, [0.0, 0.0, 1.08], r);
    }

    // ⭐⭐⭐⭐ **A CRATERA — o lado aprovado que faltava, e o que o gate do
    // `Scene Project` de facto defende.** Uma superfície ESCULPIDA estica: o
    // corpus daquele pincel empurra a chapa `0,5` para baixo num raio de `0,35`,
    // e a parede do poço mede muito mais que a corda. Ali a geodésica excede o
    // raio do pincel por geometria REAL, e cortar é o pincel deixar de alcançar
    // a PRÓPRIA cratera.
    //
    // ⛔⛔ **A 1.ª redacção desta linha usou seis dabs de `Draw` e era RASA
    // DEMAIS** — com o envelope a força cheia ela abre `0,035`, contra os `0,5`
    // que o corpus abre, **14×** menos. Ela lia `0,00 %` a `1,10×` e dizia que a
    // cratera não era o problema, enquanto o gate do oráculo reprovava.
    // *Uma fixtura que não contém o fenómeno responde que ele não existe.*
    for fundura in [0.25f32, 0.50] {
        let cratera = cratera_de(fundura, 0.35);
        linha(
            &format!("CRATERA {fundura:.2} fundo (APROVADO)"),
            &cratera,
            [0.0, 0.0, -fundura],
            0.35,
        );
    }

    // O lado do DEFEITO.
    for folga in [0.05f32, 0.20] {
        let m = dois_dedos(folga);
        for r in [0.35f32, 0.50] {
            linha(
                &format!("dois dedos folga={folga:.2}"),
                &m,
                [-(folga * 0.5), 0.0, 0.0],
                r,
            );
        }
    }
    let fin = barbatana();
    linha("A BARBATANA", &fin, centro_do_carimbo(), RAIO);

    println!(
        "\n  LEITURA: a constante e' o menor factor cujas linhas APROVADAS leem\n  \
         `0,00%` em TODAS as celulas. As de DEFEITO tem de continuar altas, senao\n  \
         o corte nao cura nada."
    );
}

/// As duas esferas soltas da sonda irmã — a classe INALCANÇÁVEL.
fn dois_dedos(folga: f32) -> Mesh {
    let uma = shapes::uv_sphere(48, 96, 1.0);
    let n = uma.vert_count() as u32;
    let dx = 2.0 + folga;
    let mut pos: Vec<[f32; 3]> = Vec::with_capacity(uma.vert_count() * 2);
    for p in uma.positions() {
        pos.push([p[0] - dx * 0.5, p[1], p[2]]);
    }
    for p in uma.positions() {
        pos.push([p[0] + dx * 0.5, p[1], p[2]]);
    }
    let mut faces = uma.faces().to_vec();
    for f in uma.faces() {
        let v = f.verts();
        let d: Vec<u32> = v.iter().map(|i| i + n).collect();
        faces.push(match d.len() {
            3 => Face::tri(d[0], d[1], d[2]),
            _ => Face::quad(d[0], d[1], d[2], d[3]),
        });
    }
    Mesh::from_parts(pos, faces).expect("duas esferas soltas continuam uma malha valida")
}

// ---------------------------------------------------------------------------
// (5) A FRONTEIRA HONESTA — que traços a cura alcança, e quais não
// ---------------------------------------------------------------------------

/// ⭐⭐⭐⭐ **O tecto é `1,50 × R`, logo a cura tem uma FRONTEIRA — e ela mede-se.**
///
/// A superfície frente→costas mede `2d + t`, onde `d` é a distância à beira.
/// Um traço só é curado quando `2d + t > 1,50 × R`, ou seja `d > (1,5R − t)/2`.
/// Com `R = 0,40` e `t = 0,06` isso dá **`d > 0,27`**.
///
/// ⚠️ **Do outro lado da fronteira o defeito CONTINUA**, e é isso que esta
/// varredura põe por escrito: um carimbo muito perto da beira tem a superfície
/// genuinamente curta, e ali nem uma geodésica perfeita o cortaria — a única
/// saída é o artista usar um pincel menor.
#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn ate_que_distancia_da_beira_a_cura_alcanca() {
    println!("\n== (5) A FRONTEIRA — variando a distancia do carimbo a' BEIRA ==");
    println!("   peca {ESPESSURA:.2} de espessura · pincel R = {RAIO:.2}\n");
    println!(
        "{:>9} {:>7} {:>11} {:>9} | {:>9} {:>9} | {:>8}",
        "da beira", "sup.", "sup/R", "sup/ar", "costas", "% frente", "veredito"
    );
    println!("{}", "-".repeat(78));
    let repouso = barbatana();
    let a = repouso.positions();
    for da_beira in [0.10f32, 0.20, 0.25, 0.30, 0.40, 0.55, 0.70] {
        let centro = [MEIO - da_beira, 0.0, ESPESSURA * 0.5];
        let depois = carimba(&repouso, centro, [0.0, 0.0, -1.0]);
        let z = depois.positions();
        let frente = semente(&repouso, centro) as usize;
        let costas = semente(&repouso, [centro[0], centro[1], -ESPESSURA * 0.5]) as usize;
        let d_f = dist(a[frente], z[frente]);
        let d_c = dist(a[costas], z[costas]);
        let sup = 2.0 * da_beira + ESPESSURA;
        let pct = 100.0 * d_c / d_f.max(1e-9);
        let veredito = if pct < 1.0 { "CURADO" } else { "ainda passa" };
        println!(
            "{da_beira:>9.2} {sup:>7.3} {:>11.2} {:>9.2} | {d_c:>9.4} {pct:>8.1}% | {veredito:>8}",
            sup / RAIO,
            sup / ESPESSURA
        );
    }
    let razao = ph2d_sculpt3d::RAZAO_MAXIMA;
    println!(
        "\n  LEITURA: a lei e' a RAZAO `superficie/ar > {razao:.2}`, e numa chapa ela\n  \
         escreve-se `(2d + t)/t > {razao:.2}` ⇒ `d > {:.2} x t`. ⭐ Ela NAO depende do\n  \
         raio do pincel — so' a ESPESSURA da peca decide —, e e' por isso que todas\n  \
         as celulas ficam curadas, incluindo as que um tecto ABSOLUTO nunca\n  \
         alcancaria (a `0,10` da beira a superficie mede `0,65 x R`).\n  \
         Com t = {ESPESSURA:.2} o limite e' `d > {:.3}`.",
        (razao - 1.0) * 0.5,
        (razao - 1.0) * 0.5 * ESPESSURA,
    );
}

/// A grelha do corpus com um POÇO de `fundura` e raio `raio`, na forma que os
/// dabs do `Scene Project` lhe dão.
fn cratera_de(fundura: f32, raio: f32) -> Mesh {
    let plana = grelha_plana();
    let curva = ph2d_sculpt3d::Falloff::default();
    let pos: Vec<[f32; 3]> = plana
        .positions()
        .iter()
        .map(|p| {
            let r = p[0].hypot(p[1]);
            let w = if r >= raio {
                0.0
            } else {
                curva.weight(r / raio)
            };
            [p[0], p[1], -fundura * w]
        })
        .collect();
    Mesh::from_parts(pos, plana.faces().to_vec()).expect("a cratera")
}

/// A grelha `41×41` do corpus do projectar — a fixtura aprovada que estica.
fn grelha_plana() -> Mesh {
    const LADO: usize = 41;
    let passo = 2.0 / (LADO - 1) as f32;
    let mut pos = Vec::with_capacity(LADO * LADO);
    for i in 0..LADO {
        for j in 0..LADO {
            pos.push([-1.0 + passo * i as f32, -1.0 + passo * j as f32, 0.0]);
        }
    }
    let at = |i: usize, j: usize| (i * LADO + j) as u32;
    let mut faces = Vec::with_capacity(2 * (LADO - 1) * (LADO - 1));
    for i in 0..LADO - 1 {
        for j in 0..LADO - 1 {
            faces.push(Face::tri(at(i, j), at(i + 1, j), at(i + 1, j + 1)));
            faces.push(Face::tri(at(i, j), at(i + 1, j + 1), at(i, j + 1)));
        }
    }
    Mesh::from_parts(pos, faces).expect("a grelha do corpus")
}

// ---------------------------------------------------------------------------
// (6) A RAZÃO — a régua que separa a parede do outro lado da ORLA da pegada
// ---------------------------------------------------------------------------

/// ⭐⭐⭐⭐ **O tecto ABSOLUTO é a régua errada, e a medição diz porquê.**
///
/// Baixar `ALCANCE_TECTO` de `2,00` para `1,50` corta — medido no corpus do
/// `Scene Project` — **só vértices da ORLA da pegada** (`ar/R` entre `0,80` e
/// `1,00`), onde a queda já é ~zero. Eles não movem barro nenhum, mas alimentam
/// o **ajuste do plano** daquele verbo, logo a saída inteira desloca-se e o
/// placar do oráculo desce de `13` para `5`.
///
/// ⇒ *o tecto absoluto mede a mesma grandeza que o raio do pincel, logo apertá-lo
/// come sempre a orla antes de chegar ao defeito.*
///
/// ⭐⭐ **A grandeza que SEPARA é a RAZÃO `superfície / ar`** — e ela já estava
/// escrita na sonda irmã como a classe `LONGE`, com o produto a nunca a usar:
///
/// | | superfície | ar | razão |
/// |---|---|---|---|
/// | as costas de uma barbatana | `0,660` | `0,060` | **`11,0`** |
/// | a orla da pegada numa cratera | `~0,53` | `~0,34` | `~1,55` |
/// | a orla numa esfera rugosa | — | — | `~1,2`–`1,5` |
///
/// *Uma parede fina e uma orla esticada estão à MESMA distância absoluta e a uma
/// ordem de grandeza uma da outra em razão.*
#[test]
#[ignore = "sonda: de onde sai o factor da razao"]
fn com_que_razao_o_corte_separa_a_parede_da_orla() {
    thread_local! { static PISO_DO_TESTE: std::cell::Cell<f32> = const { std::cell::Cell::new(0.15) }; }
    PISO_DO_TESTE.with(|c| {
        c.set(
            std::env::var("PH2D_PISO")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0.15),
        )
    });
    use ph2d_mesh::QueryScratch;
    use ph2d_sculpt3d::Falloff;

    const FACTORES: [f32; 8] = [2.00, 2.50, 3.00, 3.50, 4.00, 5.00, 6.00, 8.00];
    let curva = Falloff::default();
    println!("\n== (6) A RAZAO — quanto do peso cada factor de `sup/ar` CORTA ==");
    print!("{:>28} {:>6} |", "peca", "raio");
    for f in FACTORES {
        print!(" {f:>6.2}x");
    }
    println!();
    println!("{}", "-".repeat(28 + 9 + FACTORES.len() * 8));

    let linha = |nome: &str, m: &Mesh, centro: [f32; 3], r: f32| {
        let mut scratch = QueryScratch::default();
        let mut bola = Vec::new();
        m.verts_in_sphere(centro, r, &mut scratch, &mut bola);
        let pos = m.positions();
        let total: f64 = bola
            .iter()
            .map(|&i| f64::from(curva.weight((dist(pos[i as usize], centro) / r).min(1.0))))
            .sum();
        if total <= 0.0 {
            return;
        }
        let s = semente(m, centro);
        // ⚠️⚠️ **Pelo PASSEIO POR ARESTAS, que é a lei que SHIPA.** A varredura
        // correu primeiro com a marcha de Kimmel–Sethian e o `3,50` saiu de lá;
        // quando a medição (8) mostrou que a marcha não se paga nesta lei, a
        // constante teve de ser re-derivada **com a lei que ficou**. *Uma
        // constante derivada de uma lei não sobrevive à troca dessa lei* — foi a
        // frase que esta wave já escreveu uma vez, sobre o tecto absoluto.
        let g = passeio_distancias(m, s, 3.0 * r);
        print!("{nome:>28} {r:>6.2} |");
        for f in FACTORES {
            let cortado: f64 = bola
                .iter()
                .filter(|&&i| {
                    let ar = dist(pos[i as usize], centro);
                    let geo = g[i as usize];
                    // ⚠️ **O piso é UMA ARESTA**: junto do centro o `ar` tende a
                    // zero e a razão explode sobre ruído. Abaixo dele a pergunta
                    // não faz sentido, e quem está a uma aresta do cursor está
                    // sempre do lado de cá.
                    ar > 1e-4 && geo > f * ar && geo > PISO_DO_TESTE.with(std::cell::Cell::get) * r
                })
                .map(|&i| f64::from(curva.weight((dist(pos[i as usize], centro) / r).min(1.0))))
                .sum();
            let pct = 100.0 * cortado / total;
            print!(" {:>5.2}%", if pct.abs() < 1e-9 { 0.0 } else { pct });
        }
        println!();
    };

    let esfera = shapes::uv_sphere(64, 128, 1.0);
    for r in [0.20f32, 0.35, 0.50] {
        linha("esfera R=1 (APROVADO)", &esfera, [0.0, 0.0, 1.0], r);
    }
    let tubo = shapes::torus(96, 48, 1.0, 0.10);
    linha("tubo menor=0,10 (APROVADO)", &tubo, [1.10, 0.0, 0.0], 0.40);
    // ⚠️ **A RUGOSIDADE é varrida até o lado aprovado parar de se mexer** — uma
    // peça esculpida pode ser mais áspera que a primeira fixtura que alguém
    // escolhe, e uma constante posta na borda de UMA rugosidade mede essa
    // rugosidade.
    for amp in [0.08f32, 0.12, 0.16, 0.20, 0.24] {
        let rugosa = shapes::uv_sphere_noisy(48, 96, 1.0, amp);
        for r in [0.20f32, 0.40] {
            linha(
                &format!("rugosa amp={amp:.2} (APROVADO)"),
                &rugosa,
                [0.0, 0.0, 1.0 + amp],
                r,
            );
        }
    }
    let esculpida = shapes::sculpt_sphere(1.0);
    for r in [0.20f32, 0.40] {
        linha("sculpt_sphere (APROVADO)", &esculpida, [0.0, 0.0, 1.0], r);
    }
    for fundura in [0.25f32, 0.50] {
        let c = cratera_de(fundura, 0.35);
        linha(
            &format!("CRATERA {fundura:.2} (APROVADO)"),
            &c,
            [0.0, 0.0, -fundura],
            0.35,
        );
    }
    for folga in [0.05f32, 0.20] {
        let m = dois_dedos(folga);
        linha(
            &format!("dois dedos folga={folga:.2}"),
            &m,
            [-(folga * 0.5), 0.0, 0.0],
            0.35,
        );
    }
    linha("A BARBATANA", &barbatana(), centro_do_carimbo(), RAIO);

    println!(
        "\n  LEITURA: o factor e' o menor cujas linhas APROVADAS leem `0,00%` —\n  \
         com as de DEFEITO ainda altas. Repare que as APROVADAS saturam MUITO\n  \
         antes das de defeito: e' esse vao que o tecto absoluto nao tinha."
    );
}

// ---------------------------------------------------------------------------
// (7) O RELÓGIO — a marcha é mais cara que o passeio que ela substitui?
// ---------------------------------------------------------------------------

/// ⭐⭐ **O preço da lei nova, por dab e em fracção do orçamento.**
///
/// ⚠️ **Em `--release` e com a máquina calma** — o debug lê `~20×` mais lento e
/// daria um tecto vinte vezes menor.
#[test]
#[ignore = "sonda: relogio -- rode com a maquina calma e o loadavg ao lado"]
fn o_preco_da_marcha_por_dab() {
    use std::time::Instant;
    println!(
        "\n/proc/loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("\n== (7) O RELOGIO — um dab de `Draw`, o KILL e' 8 ms ==");
    println!(
        "{:>10} {:>7} | {:>11} {:>11} | {:>10} {:>10} | {:>8}",
        "verts", "raio", "dab c/ masc", "dab s/ masc", "MARCHA", "passeio", "% de 8ms"
    );
    println!("{}", "-".repeat(84));
    for (rings, segs) in [(64usize, 128usize), (128, 256), (256, 512), (512, 1024)] {
        let repouso = shapes::uv_sphere(rings, segs, 1.0);
        let n = repouso.vert_count();
        for raio in [0.20f32, 0.40] {
            let mut t = [0.0f64; 2];
            for (k, mascara) in [true, false].into_iter().enumerate() {
                let b = Brush {
                    verb: Verb::Draw,
                    radius: raio,
                    strength: 0.5,
                    surface_only: mascara,
                    ..Brush::default()
                };
                let mut m = repouso.clone();
                let mut st = SculptStroke::default();
                st.begin(&m);
                let d = Dab::at([0.0, 0.0, 1.0], raio, [0.0, 0.0, -1.0]);
                st.dab(&mut m, &b, &d, Symmetry::default()); // aquecer
                const REP: u32 = 30;
                let t0 = Instant::now();
                for _ in 0..REP {
                    st.dab(&mut m, &b, &d, Symmetry::default());
                }
                t[k] = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(REP);
            }
            // ⭐ **A coluna que decide: a MARCHA contra o PASSEIO POR ARESTAS
            // que ela substitui**, os dois com o mesmo tecto e a mesma semente.
            // *«mais caro que nenhuma máscara» não é a pergunta — a pergunta é
            // se ela é mais cara do que a lei que estava lá.*
            let centro = [0.0, 0.0, 1.0];
            let sem = semente(&repouso, centro);
            let tecto = ph2d_sculpt3d::ALCANCE_TECTO * raio;
            let mut g = ph2d_mesh::Geodesica::default();
            g.marcha(&repouso, sem, tecto);
            const REP2: u32 = 60;
            let t0 = Instant::now();
            for _ in 0..REP2 {
                g.marcha(&repouso, sem, tecto);
            }
            let marcha = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(REP2);
            let t0 = Instant::now();
            for _ in 0..REP2 {
                std::hint::black_box(passeio_com_tecto(&repouso, sem, tecto));
            }
            let passeio = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(REP2);
            println!(
                "{n:>10} {raio:>7.2} | {:>9.4}ms {:>9.4}ms | {marcha:>8.4}ms {passeio:>8.4}ms | {:>7.2}%",
                t[0],
                t[1],
                100.0 * t[0] / 8.0
            );
        }
    }
    println!(
        "\n  LEITURA: a coluna `mascara OFF` e' o mesmo dab SEM a marcha — a\n  \
         diferenca entre as duas e' o preco inteiro desta lei."
    );
}

/// O passeio por ARESTAS com tecto — a lei que a marcha substituiu, escrita aqui
/// para a coluna de comparação do relógio.
fn passeio_com_tecto(mesh: &Mesh, semente: u32, tecto: f32) -> usize {
    let pos = mesh.positions();
    let mut d = vec![f32::INFINITY; pos.len()];
    d[semente as usize] = 0.0;
    let mut fila = BinaryHeap::new();
    fila.push(Perto(0.0, semente));
    let viz = &mesh.adjacency().vert_verts;
    let mut n = 0usize;
    while let Some(Perto(du, u)) = fila.pop() {
        if du > d[u as usize] {
            continue;
        }
        n += 1;
        for &v in viz.neighbours(u as usize) {
            let nd = du + dist(pos[u as usize], pos[v as usize]);
            if nd > tecto {
                continue;
            }
            if nd < d[v as usize] {
                d[v as usize] = nd;
                fila.push(Perto(nd, v));
            }
        }
    }
    n
}

// ---------------------------------------------------------------------------
// OS GATES — a lei, pela porta do PRODUTO
// ---------------------------------------------------------------------------

/// ⭐⭐⭐⭐ **UM CARIMBO NA FRENTE DE UMA PAREDE FINA NÃO MOVE AS COSTAS.**
///
/// A lei inteira num gesto, pelo caminho do produto. ⚠️ **As três metades são
/// obrigatórias:**
/// * a **frente move-se** — sem isto o gate pode ficar verde sobre um carimbo
///   inerte, que é a forma que esta crate já pagou meia dúzia de vezes;
/// * as **costas não** — a afirmação;
/// * e o **CONTROLO**: com a máscara desarmada as costas MOVEM-SE, senão o gate
///   não distingue *«a cura funciona»* de *«esta fixtura não tem o defeito»*.
#[test]
fn um_carimbo_na_frente_de_uma_parede_fina_nao_move_as_costas() {
    let repouso = barbatana();
    let centro = centro_do_carimbo();
    let a = repouso.positions();
    let frente = semente(&repouso, centro) as usize;
    let costas = semente(&repouso, [centro[0], centro[1], -ESPESSURA * 0.5]) as usize;

    let com = carimba_com(&repouso, centro, [0.0, 0.0, -1.0], true);
    let (f_com, c_com) = (
        dist(a[frente], com.positions()[frente]),
        dist(a[costas], com.positions()[costas]),
    );
    let sem = carimba_com(&repouso, centro, [0.0, 0.0, -1.0], false);
    let c_sem = dist(a[costas], sem.positions()[costas]);

    assert!(
        f_com > 0.5 * ESPESSURA,
        "a FRENTE mal se moveu ({f_com:.4}) — o gate esta' a medir um carimbo inerte"
    );
    assert!(
        c_com <= 1e-6,
        "as COSTAS moveram-se {c_com:.6} ({:.1}% da frente) — a mascara de alcance \
         deixou o carimbo atravessar a parede",
        100.0 * c_com / f_com
    );
    assert!(
        c_sem > 0.8 * f_com,
        "o CONTROLO falhou: com a mascara DESARMADA as costas so' moveram {c_sem:.4} \
         contra {f_com:.4} da frente -- ou a fixtura deixou de conter o defeito, ou \
         o interruptor esta' morto e a mascara corre sempre"
    );
}

/// ⛔⛔⛔ **O TECTO ABSOLUTO NÃO É A ALAVANCA, e este gate guarda a refutação.**
///
/// Baixar o [`ph2d_sculpt3d::ALCANCE_TECTO`] parecia a cura óbvia e foi
/// **construída, medida e recusada**: de `2,00` para `1,50` ela corta só a ORLA
/// da pegada (`ar/R` entre `0,80` e `1,00`), onde a queda já é ~zero — e essa
/// orla alimenta o ajuste de plano do `Scene Project`, cujo placar de oráculo
/// desceu de `13` para `5` fixturas.
///
/// ⚠️ **O sujeito deste gate é a CRATERA**, que é a peça onde as duas leis se
/// separam: numa superfície esculpida e esticada a distância pela superfície
/// excede o raio do pincel por geometria REAL, e um tecto absoluto apertado faz
/// o pincel deixar de alcançar a própria cratera. A RAZÃO não se move ali,
/// porque `superfície/ar` continua perto de `1`.
#[test]
fn a_lei_da_razao_nao_come_a_propria_cratera() {
    let cratera = cratera_de(0.50, 0.35);
    let centro = [0.0, 0.0, -0.50];
    let a = cratera.positions();
    let com = carimba_com(&cratera, centro, [0.0, 0.0, -1.0], true);
    let sem = carimba_com(&cratera, centro, [0.0, 0.0, -1.0], false);

    let (mut movidos_com, mut movidos_sem, mut pior) = (0usize, 0usize, 0.0f32);
    for ((r, c), d) in a.iter().zip(com.positions()).zip(sem.positions()) {
        let dc = dist(*r, *c);
        let ds = dist(*r, *d);
        if dc > 0.0 {
            movidos_com += 1;
        }
        if ds > 0.0 {
            movidos_sem += 1;
        }
        pior = pior.max((dc - ds).abs());
    }
    // ⚠️ **O piso é `40` e o medido é `61`** — e a diferença entre eles é a
    // razão de o piso existir: num poço fundo a pegada é PEQUENA por geometria
    // (a esfera do pincel apanha pouca parede, porque ela sobe e afasta-se), e
    // um piso escrito da intuição («uns cem») reprovaria sobre produto correcto.
    assert!(
        movidos_sem > 40,
        "a fixtura tem de conter um carimbo a serio ({movidos_sem} movidos sem mascara)"
    );
    assert_eq!(
        movidos_com,
        movidos_sem,
        "a mascara comeu {} vertices da PROPRIA CRATERA -- e' o defeito que o tecto \
         absoluto de `1,50` produzia, e a razao nao o pode reintroduzir",
        movidos_sem.saturating_sub(movidos_com)
    );
    assert!(
        pior <= 0.0,
        "numa cratera a mascara devia ser INERTE ao bit, e move {pior:.3e}"
    );
}

// ---------------------------------------------------------------------------
// (8) A MARCHA É PRECISA PARA ESTA LEI? — a pergunta que o relógio obriga
// ---------------------------------------------------------------------------

/// ⭐⭐⭐⭐ **A marcha custa `4,3 ×` o passeio por arestas. A lei da razão precisa
/// dela?**
///
/// A régua é `sup > 3,5 × ar`, e `3,5` é **grosseiro** ao lado do viés `√2` do
/// passeio. Se as duas leis derem a mesma tabela, a marcha não se paga AQUI — e
/// §5.0 manda medir se a composição já exprime o que se quer antes de construir.
#[test]
#[ignore = "sonda: a marcha contra o passeio, na lei que shipa"]
fn a_razao_precisa_da_marcha_ou_o_passeio_chega() {
    use ph2d_mesh::{Geodesica, QueryScratch};
    use ph2d_sculpt3d::{Falloff, RAZAO_MAXIMA};

    let curva = Falloff::default();
    println!("\n== (8) A LEI DA RAZAO: marcha contra passeio por arestas ==");
    println!("   razao = {RAZAO_MAXIMA:.2}\n");
    println!(
        "{:>28} {:>6} | {:>9} {:>9} | {:>9}",
        "peca", "raio", "MARCHA", "passeio", "delta"
    );
    println!("{}", "-".repeat(70));

    let linha = |nome: &str, m: &Mesh, centro: [f32; 3], r: f32| {
        let mut scratch = QueryScratch::default();
        let mut bola = Vec::new();
        m.verts_in_sphere(centro, r, &mut scratch, &mut bola);
        let pos = m.positions();
        let total: f64 = bola
            .iter()
            .map(|&i| f64::from(curva.weight((dist(pos[i as usize], centro) / r).min(1.0))))
            .sum();
        if total <= 0.0 {
            return;
        }
        let s = semente(m, centro);
        let tecto = ph2d_sculpt3d::ALCANCE_TECTO * r;
        let mut g = Geodesica::default();
        g.marcha(m, s, tecto);
        let andado = passeio_distancias(m, s, tecto);
        let corte = |sup: &dyn Fn(u32) -> f32| -> f64 {
            bola.iter()
                .filter(|&&i| {
                    let ar = dist(pos[i as usize], centro);
                    let d = sup(i);
                    !d.is_finite() || (ar > 1e-6 && d > RAZAO_MAXIMA * ar)
                })
                .map(|&i| f64::from(curva.weight((dist(pos[i as usize], centro) / r).min(1.0))))
                .sum()
        };
        let a = 100.0 * corte(&|i| g.distancia(i)) / total;
        let b = 100.0 * corte(&|i| andado[i as usize]) / total;
        println!(
            "{nome:>28} {r:>6.2} | {a:>8.2}% {b:>8.2}% | {:>8.2}pp",
            b - a
        );
    };

    let esfera = shapes::uv_sphere(64, 128, 1.0);
    for r in [0.20f32, 0.40] {
        linha("esfera R=1 (APROVADO)", &esfera, [0.0, 0.0, 1.0], r);
    }
    for amp in [0.08f32, 0.16, 0.24] {
        let rugosa = shapes::uv_sphere_noisy(48, 96, 1.0, amp);
        linha(
            &format!("rugosa amp={amp:.2} (APROVADO)"),
            &rugosa,
            [0.0, 0.0, 1.0 + amp],
            0.40,
        );
    }
    linha(
        "sculpt_sphere (APROVADO)",
        &shapes::sculpt_sphere(1.0),
        [0.0, 0.0, 1.0],
        0.40,
    );
    for fundura in [0.25f32, 0.50] {
        let c = cratera_de(fundura, 0.35);
        linha(
            &format!("CRATERA {fundura:.2} (APROVADO)"),
            &c,
            [0.0, 0.0, -fundura],
            0.35,
        );
    }
    linha(
        "tubo menor=0,10 (APROVADO)",
        &shapes::torus(96, 48, 1.0, 0.10),
        [1.10, 0.0, 0.0],
        0.40,
    );
    for folga in [0.05f32, 0.20] {
        let m = dois_dedos(folga);
        linha(
            &format!("dois dedos folga={folga:.2}"),
            &m,
            [-(folga * 0.5), 0.0, 0.0],
            0.35,
        );
    }
    linha("A BARBATANA", &barbatana(), centro_do_carimbo(), RAIO);

    println!(
        "\n  LEITURA: `delta` positivo = o passeio corta MAIS (o viés `√2` a inflar).\n  \
         Se as linhas APROVADAS ficarem a `0,00%` nas duas e a BARBATANA nao perder,\n  \
         a marcha nao se paga nesta lei — e ela custa `4,3x` o passeio."
    );
}

/// O passeio por ARESTAS com tecto, a devolver o campo.
fn passeio_distancias(mesh: &Mesh, semente: u32, tecto: f32) -> Vec<f32> {
    let pos = mesh.positions();
    let mut d = vec![f32::INFINITY; pos.len()];
    d[semente as usize] = 0.0;
    let mut fila = BinaryHeap::new();
    fila.push(Perto(0.0, semente));
    let viz = &mesh.adjacency().vert_verts;
    while let Some(Perto(du, u)) = fila.pop() {
        if du > d[u as usize] {
            continue;
        }
        for &v in viz.neighbours(u as usize) {
            let nd = du + dist(pos[u as usize], pos[v as usize]);
            if nd > tecto {
                continue;
            }
            if nd < d[v as usize] {
                d[v as usize] = nd;
                fila.push(Perto(nd, v));
            }
        }
    }
    d
}
