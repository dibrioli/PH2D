//! ⭐⭐⭐ **SONDA — o pincel mede distância PELO AR, e quanto isso custa nas duas
//! pontas: o DEFEITO que ele produz e o PREÇO de o curar.**
//!
//! # A pergunta
//!
//! Um dab junta os vértices dentro de uma **esfera** ([`Mesh::verts_in_sphere`])
//! e pesa cada um por `falloff(|p − c| / R)`. Numa peça FINA — um dedo, uma
//! casca, dois vincos que se aproximam — a esfera alcança o **outro lado**, que
//! pela superfície está longe. ⇒ *esculpir um dedo mexe o vizinho.*
//!
//! A cura é medir a distância **andando pela superfície**. Esta sonda não a
//! implementa: ela mede se o defeito existe e o que a cura custaria.
//!
//! # ⚠️ As DUAS colunas, e porque a segunda é a que decide
//!
//! Contar **vértices** vazados não basta: `30` vértices a peso `0,01` não são
//! defeito nenhum, e `5` a peso `0,9` são o report do artista. ⇒ a coluna que
//! manda é a **fracção do PESO aplicado** que cai onde a superfície não alcança.
//!
//! # ⚠️⚠️ O CHÃO DE RUÍDO da régua, e ele tem de ser medido ANTES do defeito
//!
//! O passeio por ARESTAS **sobrestima** a distância geodésica (num triângulo,
//! ir por dois lados é mais longe que atravessar) ⇒ a bola geodésica sai um
//! pouco **menor** que a verdadeira, e isso produz um vazamento **falso** mesmo
//! numa peça sem defeito nenhum. A **esfera convexa é o controlo**: ali as duas
//! réguas têm de quase concordar, e o que ela lê é o erro da própria sonda.
//! *Um chão de ruído nunca é maior que o defeito que se persegue.*
//!
//! # ⭐⭐⭐ O QUE ELA MEDIU (2026-09-10), e como isso ESCANHOU a wave
//!
//! | | o que a medição disse |
//! |---|---|
//! | **o defeito** | dois dedos com folga `0,05`: **`43,6 %`–`47,9 %`** do peso do carimbo cai no dedo ERRADO. O controlo convexo lê `0,00 %` |
//! | **a classe que decide** | **INALCANÇÁVEL** (a superfície não liga). O caso brando — parede fina, razão `π/2` — **não é mensurável** por este passeio: o viés dele é `√2`, que é maior |
//! | **o preço** | o passeio custa `6 %` a `34 %` de um dab que já corre, e no máximo **`0,04 %` de um quadro** ⇒ *o preço não decide contra* |
//! | **o tecto** | **`2,00 × R`** — o menor factor com o lado aprovado a `0,00 %` em todas as células, e o lado do defeito já saturado desde `1,50×` (planalto até `5,00×`) |
//!
//! ⇒ **a cura não é trocar a distância do falloff: é uma MÁSCARA DE ALCANCE.** O
//! peso continua a ser `falloff(|p − c| / R)` **ao bit** — logo a paridade com os
//! oráculos fica intacta por construção — e o viés `√2` nunca entra num peso,
//! porque o passeio só responde *alcança / não alcança*.
//!
//! ```text
//! cargo test -p ph2d-sculpt3d --release --test sonda_do_falloff_pela_superficie \
//!   -- --ignored --nocapture --test-threads=1
//! ```

use ph2d_mesh::{Mesh, QueryScratch, shapes};
use ph2d_sculpt3d::{Brush, Dab, Falloff, SculptStroke, Symmetry, Verb};
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::time::Instant;

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let d = [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
}

/// Uma entrada da fila de prioridade — `f32` não é `Ord`, então a ordem é
/// escrita à mão e o `BinaryHeap` é invertido (ele é max-heap).
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

/// **A DISTÂNCIA PELA SUPERFÍCIE, limitada a `raio`** — Dijkstra sobre as
/// arestas, a partir de `semente`, que pára ao passar do raio.
///
/// ⚠️ **Com HEAP e não com pilha**, e o número que o diz já foi pago noutro
/// canto desta casa: a 1.ª versão da régua do ápice usava pilha e custava
/// `71 s` contra `0,5 s` ([`ph2d_quadfill`]). *Uma travessia por pilha
/// re-expande o mesmo nó e o custo explode com a valência.*
///
/// Devolve `∞` em quem a superfície não alcança dentro do raio.
fn pela_superficie(mesh: &Mesh, semente: u32, raio: f32, d: &mut Vec<f32>) {
    let pos = mesh.positions();
    d.clear();
    d.resize(pos.len(), f32::INFINITY);
    d[semente as usize] = 0.0;
    let viz = &mesh.adjacency().vert_verts;
    let mut fila = BinaryHeap::new();
    fila.push(Perto(0.0, semente));
    while let Some(Perto(du, u)) = fila.pop() {
        // Um nó já melhorado depois de entrar na fila aparece duas vezes; a
        // segunda visita é descartada aqui, que é o que dispensa o `decrease-key`.
        if du > d[u as usize] {
            continue;
        }
        for &v in viz.neighbours(u as usize) {
            let nd = du + dist(pos[u as usize], pos[v as usize]);
            if nd < d[v as usize] && nd <= raio {
                d[v as usize] = nd;
                fila.push(Perto(nd, v));
            }
        }
    }
}

/// O vértice mais próximo de `p` — a semente do passeio. ⚠️ Numa sonda basta;
/// no produto o ponto vem do raio de picking, que já sabe a face.
fn semente(mesh: &Mesh, p: [f32; 3]) -> u32 {
    let mut melhor = (f32::INFINITY, 0u32);
    for (i, q) in mesh.positions().iter().enumerate() {
        let dd = dist(*q, p);
        if dd < melhor.0 {
            melhor = (dd, i as u32);
        }
    }
    melhor.1
}

/// O que o dab de hoje junta, e quanto do PESO dele a superfície não alcança.
///
/// ⚠️⚠️ **O critério é uma RAZÃO e não um binário, e a 1.ª redacção estava
/// errada nisso.** Ela contava *«a superfície não chega dentro de `R`»*, o que
/// mistura o defeito com o **erro de discretização** do passeio por arestas — e
/// medido, a esfera convexa (que não tem defeito nenhum) lia `1,94 %` e
/// `3,89 %`, contra `6,21 %` do toro: *o chão de ruído era da mesma ordem do que
/// se perseguia, logo a régua não separava nada.*
///
/// ⇒ duas classes, e a primeira é sem ambiguidade nenhuma:
/// * **INALCANÇÁVEL** — a superfície não liga os dois pontos *de forma alguma*
///   (`d_geo = ∞` sem tecto). Um dedo vizinho está aqui, e nenhum epsilon o
///   explica.
/// * **LONGE** — a superfície liga, mas por um caminho `> FATOR ×` o do ar. O
///   `FATOR` sai do CONTROLO, nunca de escolha.
struct Vazamento {
    verts_bola: usize,
    peso_total: f64,
    /// A superfície não liga os dois pontos (componente separada, ou volta longa
    /// demais para caber em qualquer tecto razoável).
    peso_inalcancavel: f64,
    /// A superfície liga, mas por um caminho `> FATOR_LONGE` vezes o do ar.
    peso_longe: f64,
    /// A maior razão `caminho pela superfície / caminho pelo ar` no carimbo.
    pior_razao: f32,
}

/// Quantas vezes o caminho pela superfície tem de ser mais longo que o do ar
/// para a diferença ser DEFEITO e não discretização.
///
/// ⛔⛔ **A 1.ª redacção pôs `1,25` supondo que o resíduo do controlo era pequeno,
/// e o controlo mediu `1,41x` — exactamente `√2`.** Não é ruído: numa grelha de
/// quads o passeio por arestas tem de ir por **dois lados** onde a superfície
/// atravessa a **diagonal**, e `√2` é o preço disso. Com a barra abaixo do viés,
/// a coluna lia `22 %`–`62 %` na esfera CONVEXA, que não tem defeito nenhum.
/// *Uma barra abaixo do viés do próprio instrumento mede a grelha, não a peça.*
///
/// ⇒ `1,60` fica acima do `√2` com folga e abaixo do `π/2 ≈ 1,57`… ⚠️ **e é aqui
/// que a coluna encontra o limite dela:** o `π/2` de uma parede fina é *menor*
/// que o viés `√2` + folga, logo **o caso brando não é mensurável por este
/// passeio**. Medi-lo pede geodésica a sério (método do calor, ou MMP), que é
/// uma máquina muito maior. ⇒ **a classe que decide esta wave é a
/// INALCANÇÁVEL**, e é ela que o controlo lê a `0,00 %` nos quatro raios.
const FATOR_LONGE: f32 = 1.60;

fn medir(mesh: &Mesh, centro: [f32; 3], raio: f32) -> Vazamento {
    let curva = Falloff::default();
    let mut scratch = QueryScratch::default();
    let mut bola = Vec::new();
    mesh.verts_in_sphere(centro, raio, &mut scratch, &mut bola);
    // ⚠️ **O passeio corre SEM TECTO** (`∞`), de propósito: com o tecto em `R`
    // não há como distinguir *«a volta é longa»* de *«não há volta»*, e as duas
    // pedem curas diferentes. O preço disto é medido na sonda (2), que corre o
    // passeio limitado, que é o que o produto usaria.
    let mut geo = Vec::new();
    pela_superficie(mesh, semente(mesh, centro), f32::INFINITY, &mut geo);
    let pos = mesh.positions();
    let mut v = Vazamento {
        verts_bola: bola.len(),
        peso_total: 0.0,
        peso_inalcancavel: 0.0,
        peso_longe: 0.0,
        pior_razao: 0.0,
    };
    for &i in &bola {
        let ar = dist(pos[i as usize], centro);
        let t = (ar / raio).min(1.0);
        let w = curva.weight(t);
        v.peso_total += f64::from(w);
        let g = geo[i as usize];
        if !g.is_finite() {
            v.peso_inalcancavel += f64::from(w);
            v.pior_razao = f32::INFINITY;
        } else if ar > 1e-6 {
            let razao = g / ar;
            v.pior_razao = v.pior_razao.max(razao);
            if razao > FATOR_LONGE {
                v.peso_longe += f64::from(w);
            }
        }
    }
    v
}

/// ⛔⛔ **O `shapes::cylinder` da casa NÃO contém o fenómeno, e a 1.ª versão
/// desta sonda usou-o:** ele tem **dois anéis** (topo e base) e a parede é uma
/// faixa de quads só ⇒ não existe vértice nenhum a meia altura, e um carimbo ali
/// juntava **zero** vértices. A tabela imprimia `0 0 NaN%` em oito células e
/// isso lê-se como *«não há defeito»*. *Uma fixtura sem vértices onde o pincel
/// toca não mede o pincel — mede a malha.*
///
/// ⇒ **DOIS DEDOS, que é o caso do report que isto existe para curar:** duas
/// esferas na MESMA malha, separadas por `folga`. A superfície não as liga de
/// forma alguma, então a razão é `∞` e nenhum epsilon a explica.
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
            3 => ph2d_mesh::Face::tri(d[0], d[1], d[2]),
            _ => ph2d_mesh::Face::quad(d[0], d[1], d[2], d[3]),
        });
    }
    Mesh::from_parts(pos, faces).expect("duas esferas soltas continuam uma malha valida")
}

// ---------------------------------------------------------------------------
// (1) O DEFEITO
// ---------------------------------------------------------------------------

#[test]
#[ignore = "sonda: imprime a tabela, nao afirma nada"]
fn quanto_do_carimbo_cai_onde_a_superficie_nao_alcanca() {
    println!("\n== (1) O DEFEITO — o dab de hoje contra a superficie ==");
    println!(
        "{:>26} {:>6} | {:>6} | {:>11} {:>9} | {:>10}",
        "peca", "raio", "verts", "INALCANCAVEL", "LONGE", "pior razao"
    );
    println!("{}", "-".repeat(78));

    let linha = |nome: &str, m: &Mesh, centro: [f32; 3], r: f32| {
        let v = medir(m, centro, r);
        if v.peso_total <= 0.0 {
            println!(
                "{nome:>26} {r:>6.2} | {:>6} | (o carimbo nao apanhou vertice nenhum -- fixtura MUDA)",
                v.verts_bola
            );
            return;
        }
        let pct = |x: f64| 100.0 * x / v.peso_total;
        let razao = if v.pior_razao.is_finite() {
            format!("{:.2}x", v.pior_razao)
        } else {
            "INF".to_string()
        };
        println!(
            "{nome:>26} {r:>6.2} | {:>6} | {:>10.2}% {:>8.2}% | {razao:>10}",
            v.verts_bola,
            pct(v.peso_inalcancavel),
            pct(v.peso_longe)
        );
    };

    // ⚠️ **O CONTROLO PRIMEIRO** — numa esfera convexa nao ha' defeito nenhum,
    // logo o que ela le' e' o erro da propria SONDA.
    let esfera = shapes::uv_sphere(64, 128, 1.0);
    for r in [0.10f32, 0.20, 0.35, 0.50] {
        linha("esfera R=1 (CONTROLO)", &esfera, [0.0, 0.0, 1.0], r);
    }

    // ⭐ O caso do REPORT: dois dedos que se tocam. A superficie nao os liga.
    for folga in [0.05f32, 0.20] {
        let m = dois_dedos(folga);
        let borda = [-(folga * 0.5), 0.0, 0.0]; // o ponto do dedo esquerdo virado ao direito
        for r in [0.20f32, 0.35, 0.50] {
            linha(&format!("dois dedos folga={folga:.2}"), &m, borda, r);
        }
    }

    // O TUBO fino: a parede tem razao `pi/2` por geometria, e e' o caso brando.
    for minor in [0.10f32, 0.25] {
        let t = shapes::torus(96, 48, 1.0, minor);
        for r in [0.20f32, 0.40] {
            linha(
                &format!("tubo minor={minor:.2}"),
                &t,
                [1.0 + minor, 0.0, 0.0],
                r,
            );
        }
    }

    println!(
        "\n  LEITURA: `INALCANCAVEL` nao tem ambiguidade -- a superficie nao liga os\n           dois pontos, e nenhum epsilon o explica. `LONGE` e' a classe que o\n           CONTROLO calibra: o que a esfera convexa le' ali e' o erro da sonda."
    );
}

// ---------------------------------------------------------------------------
// (2) O PREÇO
// ---------------------------------------------------------------------------

#[test]
#[ignore = "sonda: relogio -- rode com a maquina calma e o loadavg ao lado"]
fn o_preco_de_medir_pela_superficie() {
    println!(
        "\n/proc/loadavg: {}",
        std::fs::read_to_string("/proc/loadavg")
            .unwrap_or_default()
            .trim()
    );
    println!("\n== (2) O PRECO — um quadro tem 16,7 ms ==");
    println!(
        "{:>10} {:>7} | {:>10} {:>10} {:>10} | {:>9} {:>9}",
        "verts", "raio", "bola (ms)", "geo (ms)", "dab (ms)", "geo/dab", "% quadro"
    );
    println!("{}", "-".repeat(86));

    for (rings, segs) in [(32usize, 64usize), (64, 128), (128, 256)] {
        let mesh = shapes::uv_sphere(rings, segs, 1.0);
        let n = mesh.vert_count();
        for r in [0.10f32, 0.25] {
            let centro = [0.0, 0.0, 1.0];
            let sem = semente(&mesh, centro);
            let mut scratch = QueryScratch::default();
            let mut out = Vec::new();
            let mut d = Vec::new();

            // Aquecer os dois — o primeiro toque aloca, e medir a alocacao seria
            // medir outra coisa.
            mesh.verts_in_sphere(centro, r, &mut scratch, &mut out);
            pela_superficie(&mesh, sem, r, &mut d);

            const N: u32 = 50;
            let t0 = Instant::now();
            for _ in 0..N {
                mesh.verts_in_sphere(centro, r, &mut scratch, &mut out);
            }
            let bola = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(N);

            let t0 = Instant::now();
            for _ in 0..N {
                pela_superficie(&mesh, sem, r, &mut d);
            }
            let geo = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(N);

            // Um dab INTEIRO do produto, para a razao ter denominador honesto.
            let b = Brush {
                verb: Verb::Draw,
                radius: r,
                strength: 0.5,
                ..Brush::default()
            };
            let mut m2 = mesh.clone();
            let mut st = SculptStroke::default();
            st.begin(&m2);
            let dab = Dab::at(centro, r, [0.0, 0.0, -1.0]);
            st.dab(&mut m2, &b, &dab, Symmetry::default());
            let t0 = Instant::now();
            for _ in 0..N {
                st.dab(&mut m2, &b, &dab, Symmetry::default());
            }
            let tdab = t0.elapsed().as_secs_f64() * 1000.0 / f64::from(N);

            println!(
                "{n:>10} {r:>7.2} | {bola:>10.4} {geo:>10.4} {tdab:>10.4} | {:>9.2} {:>8.2}%",
                geo / tdab,
                100.0 * geo / 16.7
            );
        }
    }
    println!(
        "\n  LEITURA: `geo/dab` diz se a cura e' um detalhe ou o dobro do trabalho.\n  \
         ⚠️ O passeio anda O(k log k) nos vertices ALCANCADOS, e a bola anda\n  \
         O(faces das folhas do octree) — as duas crescem com o RAIO, nao com a peca."
    );
}

// ---------------------------------------------------------------------------
// (3) A CONSTANTE — com que tecto se chama «a superfície não alcança»
// ---------------------------------------------------------------------------

/// ⭐⭐⭐ **A medição (1) escanhou a cura, e o resultado é MUITO mais barato e
/// seguro do que trocar a distância do falloff.**
///
/// A classe que decide é a **INALCANÇÁVEL**, e responder a *«a superfície liga
/// isto?»* **não precisa de distância nenhuma** — só de alcance. ⇒ a cura é uma
/// **MÁSCARA**, não uma régua nova:
///
/// * o peso continua a ser `falloff(|p − c| / R)`, **ao bit** ⇒ toda a paridade
///   com os oráculos (a do tecido, a do SculptGL) fica intacta por construção;
/// * o viés `√2` do passeio **nunca entra no peso**, porque ele não é lido como
///   distância;
/// * quem a superfície não alcança dentro de um tecto **generoso** sai do
///   carimbo.
///
/// ⚠️ **O tecto tem de ser generoso, e é por causa do mesmo `√2`:** com o tecto
/// em `R` o passeio cortaria vértices da borda que a superfície ALCANÇA, só por
/// ir pela diagonal da grelha. ⇒ esta sonda varre o factor e a constante sai do
/// **vale medido que inclui o lado aprovado** — a esfera convexa tem de ler
/// `0,00 %` de corte, senão a cura mexe no que já está certo.
#[test]
#[ignore = "sonda: de onde sai o tecto do alcance"]
fn com_que_tecto_o_corte_nao_toca_a_peca_convexa() {
    println!("\n== (3) O TECTO — quanto do peso cada factor CORTA ==");
    println!(
        "{:>26} {:>6} | {:>7} {:>7} {:>7} {:>7} {:>7} {:>7}",
        "peca", "raio", "1,00xR", "1,25xR", "1,50xR", "2,00xR", "3,00xR", "5,00xR"
    );
    println!("{}", "-".repeat(84));

    let curva = Falloff::default();
    let fatores = [1.0f32, 1.25, 1.5, 2.0, 3.0, 5.0];

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
        let mut cols = String::new();
        for f in fatores {
            let mut geo = Vec::new();
            pela_superficie(m, semente(m, centro), f * r, &mut geo);
            let cortado: f64 = bola
                .iter()
                .filter(|&&i| !geo[i as usize].is_finite())
                .map(|&i| f64::from(curva.weight((dist(pos[i as usize], centro) / r).min(1.0))))
                .sum();
            // ⚠️ Um `-0,00%` numa tabela de CALIBRAÇÃO faz quem a lê duvidar do
            // zero; o resíduo de somar `f64` é achatado aqui, e só na impressão.
            let pct = 100.0 * cortado / total;
            cols.push_str(&format!(
                " {:>6.2}%",
                if pct.abs() < 1e-9 { 0.0 } else { pct }
            ));
        }
        println!("{nome:>26} {r:>6.2} |{cols}");
    };

    let esfera = shapes::uv_sphere(64, 128, 1.0);
    for r in [0.10f32, 0.20, 0.35, 0.50] {
        linha("esfera R=1 (APROVADO)", &esfera, [0.0, 0.0, 1.0], r);
    }
    let tubo = shapes::torus(96, 48, 1.0, 0.10);
    for r in [0.20f32, 0.40] {
        linha("tubo minor=0,10 (APROVADO)", &tubo, [1.10, 0.0, 0.0], r);
    }
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

    println!(
        "\n  LEITURA: a constante e' o menor factor cujas linhas APROVADAS leem\n  \
         `0,00%` em TODAS as celulas -- cortar peso numa peca convexa seria a cura\n  \
         a mexer no que ja' esta' certo. Nas linhas de DEDO ela tem de continuar\n  \
         alta, senao o corte nao cura nada."
    );
}
